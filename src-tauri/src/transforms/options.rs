use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(crate) struct TransformExecutionOptions {
    engine_paths: HashMap<String, String>,
    trusted_engines: HashMap<String, bool>,
    disabled_engines: HashMap<String, bool>,
    input_modes: HashMap<String, String>,
    document_dir: Option<PathBuf>,
    pub(crate) timeout_ms: Option<u64>,
}

impl TransformExecutionOptions {
    pub(crate) fn from_compile_options(
        options: Option<&Value>,
        document_path: Option<&Path>,
    ) -> Self {
        let document_dir = document_path
            .and_then(|path| path.parent())
            .map(Path::to_path_buf);
        let Some(options) = options else {
            return Self {
                document_dir,
                ..Self::default()
            };
        };
        Self {
            engine_paths: string_map_option(options, "transformEnginePaths"),
            trusted_engines: bool_map_option(options, "trustedTransformEngines"),
            disabled_engines: bool_map_option(options, "disabledTransformEngines"),
            input_modes: string_map_option(options, "transformInputModes"),
            document_dir,
            timeout_ms: options.get("transformTimeoutMs").and_then(Value::as_u64),
        }
    }

    pub(crate) fn engine_path(&self, name: &str) -> Option<String> {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.engine_paths.get(*key))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn trusted(&self, name: &str) -> bool {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.trusted_engines.get(*key))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn disabled(&self, name: &str) -> bool {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.disabled_engines.get(*key))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn input_mode(&self, name: &str) -> Option<String> {
        option_lookup_keys(name)
            .iter()
            .find_map(|key| self.input_modes.get(*key))
            .cloned()
    }

    /// Resolve `value` relative to the document directory and canonicalize the
    /// result so symlinks and `..` components are fully resolved.  Returns an
    /// error if the path does not exist or if canonicalization fails.
    ///
    /// Callers should call [`Self::document_relative_path_escapes`] first to
    /// get a clear error message; this method provides a second canonicalized
    /// path for the actual spawn call.
    pub(crate) fn resolve_document_path(&self, value: &str) -> Result<PathBuf, String> {
        let path = PathBuf::from(value);
        let candidate = if path.is_absolute() {
            path
        } else if let Some(document_dir) = &self.document_dir {
            document_dir.join(path)
        } else {
            path
        };
        // Canonicalize resolves symlinks and strips `..` so the path handed to
        // the spawned process is clean and fully absolute.
        candidate.canonicalize().map_err(|e| {
            format!(
                "Cannot resolve database path '{}': {e}",
                candidate.display()
            )
        })
    }

    /// Returns `true` when `value` resolves outside the document directory.
    ///
    /// Both absolute paths (e.g. `/home/user/.mozilla/firefox/places.sqlite`,
    /// `C:\Users\…\db.sqlite`, UNC `\\server\share\…`) and relative paths with
    /// `..` traversal or symlinks are checked via canonicalization.  A missing
    /// file, a broken symlink, or a failed canonicalize is treated as a
    /// potential escape and returns `true` (deny).
    pub(crate) fn document_relative_path_escapes(&self, value: &str) -> bool {
        let path = PathBuf::from(value);
        let Some(document_dir) = &self.document_dir else {
            return false;
        };
        // Resolve absolute paths directly; join relative ones to document_dir.
        // Both branches are then canonicalized and compared — this closes the
        // bypass (F10) where an absolute path bypassed the check entirely.
        let candidate = if path.is_absolute() {
            path
        } else {
            document_dir.join(&path)
        };
        match (document_dir.canonicalize(), candidate.canonicalize()) {
            (Ok(base), Ok(target)) => !target.starts_with(base),
            _ => true,
        }
    }
}

fn option_lookup_keys(name: &str) -> Vec<&str> {
    let aliases = transform_option_aliases(name);
    if aliases.is_empty() {
        vec![name]
    } else {
        aliases.to_vec()
    }
}

fn transform_option_aliases(name: &str) -> &'static [&'static str] {
    match name {
        "dot" | "graphviz" | "graph" => &["dot", "graphviz", "graph"],
        "vega-lite" | "vegalite" => &["vega-lite", "vegalite"],
        "json-schema" | "jsonschema" | "schema" => &["json-schema", "jsonschema", "schema"],
        "yaml" | "yml" => &["yaml", "yml"],
        "plantuml" => &["plantuml"],
        "d2" => &["d2"],
        "pikchr" => &["pikchr"],
        "circo" => &["circo"],
        "neato" => &["neato"],
        "fdp" => &["fdp"],
        "osage" => &["osage"],
        "twopi" => &["twopi"],
        "sql" => &["sql"],
        _ => &[],
    }
}

fn string_map_option(options: &Value, key: &str) -> HashMap<String, String> {
    options
        .get(key)
        .and_then(Value::as_object)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .as_str()
                        .map(|field| (name.clone(), field.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn opts_with_dir(dir: &std::path::Path) -> TransformExecutionOptions {
        TransformExecutionOptions {
            document_dir: Some(dir.to_path_buf()),
            ..Default::default()
        }
    }

    // ── F10: absolute-path bypass ─────────────────────────────────────────────

    /// An absolute path to a real location clearly outside the doc dir must
    /// be detected as an escape.
    #[cfg(unix)]
    #[test]
    fn absolute_path_outside_doc_dir_escapes() {
        let dir = tempfile::TempDir::new().unwrap();
        let opts = opts_with_dir(dir.path());
        // /tmp always exists and is outside any per-run TempDir on Unix.
        assert!(
            opts.document_relative_path_escapes("/tmp"),
            "/tmp must be detected as an escape"
        );
    }

    #[cfg(windows)]
    #[test]
    fn absolute_path_outside_doc_dir_escapes_windows() {
        let dir = tempfile::TempDir::new().unwrap();
        let opts = opts_with_dir(dir.path());
        assert!(opts.document_relative_path_escapes(r"C:\Windows"));
    }

    /// An absolute path that resolves **inside** the doc dir is allowed.
    #[test]
    fn absolute_path_inside_doc_dir_does_not_escape() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("data.sqlite");
        fs::write(&file, b"").unwrap();
        let opts = opts_with_dir(dir.path());
        let abs = file.to_str().unwrap();
        assert!(
            !opts.document_relative_path_escapes(abs),
            "absolute path to a file inside doc dir must not be flagged"
        );
    }

    // ── `..` traversal ────────────────────────────────────────────────────────

    /// A relative path with `..` that escapes must be detected.  The target
    /// file does not need to exist — canonicalize failure is treated as deny.
    #[test]
    fn dotdot_traversal_is_detected_as_escape() {
        let dir = tempfile::TempDir::new().unwrap();
        let opts = opts_with_dir(dir.path());
        assert!(
            opts.document_relative_path_escapes("../../etc/passwd"),
            ".. traversal must be detected"
        );
    }

    /// A relative path that stays inside the dir is allowed.
    #[test]
    fn relative_path_inside_dir_is_allowed() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("db.sqlite");
        fs::write(&file, b"").unwrap();
        let opts = opts_with_dir(dir.path());
        assert!(
            !opts.document_relative_path_escapes("db.sqlite"),
            "relative path to an existing file inside doc dir must not be flagged"
        );
    }

    // ── Symlink escape (Unix only) ────────────────────────────────────────────

    #[cfg(unix)]
    #[test]
    fn symlink_pointing_outside_dir_escapes() {
        let dir = tempfile::TempDir::new().unwrap();
        let link = dir.path().join("evil.sqlite");
        // Symlink points to /tmp — outside the doc dir.
        std::os::unix::fs::symlink("/tmp", &link).unwrap();
        let opts = opts_with_dir(dir.path());
        assert!(
            opts.document_relative_path_escapes("evil.sqlite"),
            "symlink pointing outside doc dir must be detected"
        );
    }

    // ── resolve_document_path ─────────────────────────────────────────────────

    #[test]
    fn resolve_document_path_canonicalizes_existing_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("db.sqlite");
        fs::write(&file, b"").unwrap();
        let opts = opts_with_dir(dir.path());
        let resolved = opts.resolve_document_path("db.sqlite").unwrap();
        assert!(resolved.is_absolute());
        assert!(resolved.exists());
    }

    #[test]
    fn resolve_document_path_returns_error_for_nonexistent() {
        let dir = tempfile::TempDir::new().unwrap();
        let opts = opts_with_dir(dir.path());
        assert!(
            opts.resolve_document_path("no-such-file.sqlite").is_err(),
            "missing file must return an error"
        );
    }
}

fn bool_map_option(options: &Value, key: &str) -> HashMap<String, bool> {
    options
        .get(key)
        .and_then(Value::as_object)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(name, value)| value.as_bool().map(|field| (name.clone(), field)))
                .collect()
        })
        .unwrap_or_default()
}
