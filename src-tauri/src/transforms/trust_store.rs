use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

/// Error prefix the frontend detects to show the trust consent prompt.
pub(crate) const TRUST_REQUIRED_PREFIX: &str = "TRUST_REQUIRED:";

/// Metadata bound to a trusted engine binary.  Any change in size or mtime
/// causes the entry to be evicted on the next `is_trusted` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TrustEntry {
    /// Transform name that initiated the trust grant (for display).
    pub(crate) granter: String,
    /// Unix timestamp (seconds) when trust was granted.
    pub(crate) granted_at: u64,
    /// Binary size at grant time (bytes).
    pub(crate) engine_size: u64,
    /// Binary mtime at grant time (Unix seconds).
    pub(crate) engine_mtime: u64,
}

/// On-disk JSON envelope.  Keys are canonical path strings.
#[derive(Debug, Default, Serialize, Deserialize)]
struct TrustStoreFile {
    entries: HashMap<String, TrustEntry>,
}

/// Backend-owned trust registry for external transform engines.
///
/// Registered with Tauri `.manage()` so every invocation of
/// `run_external_transform` consults it instead of trusting the
/// frontend-supplied `trusted` boolean.
pub(crate) struct TransformTrustStore {
    inner: Mutex<HashMap<PathBuf, TrustEntry>>,
    store_path: PathBuf,
}

/// Shape returned to the frontend by `list_trusted_engines`.
#[derive(Debug, Serialize)]
pub(crate) struct TrustedEngineInfo {
    pub(crate) engine_path: String,
    pub(crate) granter: String,
    pub(crate) granted_at: u64,
    pub(crate) engine_size: u64,
    /// False when the binary size or mtime no longer matches what was
    /// recorded at grant time — i.e. the binary was replaced.
    pub(crate) valid: bool,
}

impl TransformTrustStore {
    /// Load from the platform data-local directory, or start empty.
    pub(crate) fn load_or_default() -> Self {
        Self::load_from(data_store_path())
    }

    /// Ephemeral instance backed by a unique temp file — for tests only.
    #[cfg(test)]
    pub(crate) fn ephemeral() -> Self {
        let store_path = std::env::temp_dir().join(format!(
            "neditor-trust-test-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        Self {
            inner: Mutex::new(HashMap::new()),
            store_path,
        }
    }

    /// Load from an arbitrary path — useful for tests that need isolated stores.
    pub(crate) fn load_from(store_path: PathBuf) -> Self {
        let inner = if store_path.is_file() {
            fs::read_to_string(&store_path)
                .ok()
                .and_then(|json| serde_json::from_str::<TrustStoreFile>(&json).ok())
                .map(|file| {
                    file.entries
                        .into_iter()
                        .filter_map(|(k, v)| {
                            // Re-canonicalize on load; skip stale entries for
                            // paths that no longer exist.
                            PathBuf::from(&k).canonicalize().ok().map(|p| (p, v))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        Self {
            inner: Mutex::new(inner),
            store_path,
        }
    }

    /// Returns `true` iff the engine at `engine_path` is in the store AND
    /// its binary fingerprint (size + mtime) still matches.  A mismatch
    /// evicts the entry so the user must re-grant explicitly.
    pub(crate) fn is_trusted(&self, engine_path: &Path) -> bool {
        let canonical = match engine_path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Check under lock, but release before potential persist call.
        let (trusted, should_evict) = {
            let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            match inner.get(&canonical) {
                None => (false, false),
                Some(entry) => {
                    let ok = fingerprint_matches(&canonical, entry);
                    (ok, !ok)
                }
            }
        };

        if should_evict {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.remove(&canonical);
            let _ = self.persist_locked(&*inner);
        }

        trusted
    }

    /// Record trust for `engine_path`, binding it to current binary metadata.
    ///
    /// Returns an error if the path cannot be canonicalized or its metadata
    /// read.
    pub(crate) fn grant(&self, engine_path: PathBuf, transform_name: String) -> Result<(), String> {
        let canonical = engine_path
            .canonicalize()
            .map_err(|e| format!("cannot canonicalize engine path: {e}"))?;
        let meta =
            fs::metadata(&canonical).map_err(|e| format!("cannot read engine metadata: {e}"))?;
        let engine_size = meta.len();
        let engine_mtime = mtime_secs(&meta);
        let granted_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = TrustEntry {
            granter: transform_name,
            granted_at,
            engine_size,
            engine_mtime,
        };
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.insert(canonical, entry);
        self.persist_locked(&*inner)
    }

    /// Remove the trust entry for `engine_path`.
    pub(crate) fn revoke(&self, engine_path: &Path) -> Result<(), String> {
        // canonicalize best-effort; if path is gone we try the raw string key
        let canonical = engine_path
            .canonicalize()
            .unwrap_or_else(|_| engine_path.to_path_buf());
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.remove(&canonical);
        self.persist_locked(&*inner)
    }

    /// List all stored trust entries with current validity.
    pub(crate) fn list(&self) -> Vec<TrustedEngineInfo> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner
            .iter()
            .map(|(path, entry)| TrustedEngineInfo {
                engine_path: path.display().to_string(),
                granter: entry.granter.clone(),
                granted_at: entry.granted_at,
                engine_size: entry.engine_size,
                valid: fingerprint_matches(path, entry),
            })
            .collect()
    }

    /// Serialize and write the store.  Caller must hold the inner lock and
    /// pass a reference to the guard's contents.
    fn persist_locked(&self, inner: &HashMap<PathBuf, TrustEntry>) -> Result<(), String> {
        let entries: HashMap<String, TrustEntry> = inner
            .iter()
            .map(|(k, v)| (k.display().to_string(), v.clone()))
            .collect();
        let json = serde_json::to_string_pretty(&TrustStoreFile { entries })
            .map_err(|e| format!("trust store serialize: {e}"))?;

        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("trust store dir create: {e}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
            }
        }

        fs::write(&self.store_path, &json).map_err(|e| format!("trust store write: {e}"))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.store_path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }
}

fn fingerprint_matches(path: &Path, entry: &TrustEntry) -> bool {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    meta.len() == entry.engine_size && mtime_secs(&meta) == entry.engine_mtime
}

fn mtime_secs(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn data_store_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("neditor")
        .join("trust-store.json")
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Grant trust to an external engine binary.  Reads current binary metadata
/// and binds the entry to it, so a later binary swap auto-evicts the entry.
#[tauri::command]
pub(crate) fn trust_external_engine(
    engine_path: String,
    transform_name: String,
    state: tauri::State<'_, TransformTrustStore>,
) -> Result<(), String> {
    let path = PathBuf::from(engine_path.trim());
    if !path.is_absolute() {
        return Err("Engine path must be absolute.".to_string());
    }
    state.grant(path, transform_name)
}

/// Remove a previously granted trust entry.
#[tauri::command]
pub(crate) fn revoke_external_engine(
    engine_path: String,
    state: tauri::State<'_, TransformTrustStore>,
) -> Result<(), String> {
    state.revoke(Path::new(engine_path.trim()))
}

/// Return all currently stored trust entries with live validity flags.
#[tauri::command]
pub(crate) fn list_trusted_engines(
    state: tauri::State<'_, TransformTrustStore>,
) -> Vec<TrustedEngineInfo> {
    state.list()
}
