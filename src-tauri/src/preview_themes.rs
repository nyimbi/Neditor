/// Preview theme management: list bundled + user themes, read CSS, hot-reload watcher.
///
/// Bundled themes live in `src-tauri/themes/preview/*.css` (shipped as Tauri resources).
/// User themes live in `{data_local_dir}/neditor/preview-themes/*.css`.
/// If a user theme has the same id (stem) as a bundled theme, the user theme takes
/// precedence in `list_preview_themes`.
///
/// `watch_preview_theme` / `unwatch_preview_theme` use the `notify` crate to watch
/// a single CSS file and emit the `preview-theme-changed` Tauri event on modification.
use dirs;
#[cfg(feature = "native-watch")]
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
#[cfg(feature = "native-watch")]
use tauri::Emitter;
use tauri::{AppHandle, Manager, State};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PreviewTheme {
    pub(crate) id: String,
    pub(crate) name: String,
    /// "bundled" or "user"
    pub(crate) source: String,
    #[serde(rename = "cssPath")]
    pub(crate) css_path: String,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Serialize)]
struct ThemeChangedPayload {
    id: String,
    css: String,
}

#[derive(Default)]
pub(crate) struct PreviewThemeWatcherState {
    watcher: Mutex<Option<ActiveThemeWatcher>>,
}

struct ActiveThemeWatcher {
    #[cfg(feature = "native-watch")]
    _watcher: RecommendedWatcher,
    #[allow(dead_code)]
    id: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Path where user preview-theme CSS files are stored.
pub(crate) fn user_themes_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("neditor").join("preview-themes"))
}

/// Resolve the bundled themes directory from the app resource directory.
fn bundled_themes_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .resource_dir()
        .ok()
        .map(|r| r.join("themes").join("preview"))
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn pretty_name(id: &str) -> String {
    id.replace('-', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn description_from_css(css: &str) -> Option<String> {
    // Read first comment block: /* ... */
    let start = css.find("/*")?;
    let end = css[start..].find("*/").map(|p| start + p)?;
    let comment = css[start + 2..end].trim();
    let first_line = comment.lines().next()?.trim();
    if first_line.is_empty() {
        None
    } else {
        Some(first_line.to_string())
    }
}

fn load_themes_from_dir(dir: &Path, source: &str) -> Vec<PreviewTheme> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut themes: Vec<PreviewTheme> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("css"))
        .map(|e| {
            let path = e.path();
            let id = stem(&path);
            let css_body = fs::read_to_string(&path).unwrap_or_default();
            let description = description_from_css(&css_body);
            PreviewTheme {
                name: pretty_name(&id),
                id,
                source: source.to_string(),
                css_path: path.to_string_lossy().into_owned(),
                description,
            }
        })
        .collect();
    themes.sort_by(|a, b| a.id.cmp(&b.id));
    themes
}

/// Resolve the CSS file path for a theme id, preferring user themes.
pub(crate) fn resolve_theme_css_path(id: &str, app: &AppHandle) -> Option<PathBuf> {
    // User theme first
    if let Some(user_dir) = user_themes_dir() {
        let candidate = user_dir.join(format!("{id}.css"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    // Bundled theme
    if let Some(bundled_dir) = bundled_themes_dir(app) {
        let candidate = bundled_dir.join(format!("{id}.css"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Reject paths that escape the allowed directories (path traversal guard).
fn path_is_safe(path: &Path, allowed_dirs: &[PathBuf]) -> bool {
    let Ok(canonical) = path.canonicalize() else {
        return false;
    };
    allowed_dirs.iter().any(|dir| canonical.starts_with(dir))
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub(crate) fn list_preview_themes(app: AppHandle) -> Result<Vec<PreviewTheme>, String> {
    let mut bundled = bundled_themes_dir(&app)
        .map(|d| load_themes_from_dir(&d, "bundled"))
        .unwrap_or_default();
    let user = user_themes_dir()
        .map(|d| load_themes_from_dir(&d, "user"))
        .unwrap_or_default();

    // User themes take precedence: remove bundled entries with same id
    let user_ids: std::collections::HashSet<&str> = user.iter().map(|t| t.id.as_str()).collect();
    bundled.retain(|t| !user_ids.contains(t.id.as_str()));

    let mut all = bundled;
    all.extend(user);
    all.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(all)
}

#[tauri::command]
pub(crate) fn read_preview_theme_css(app: AppHandle, id: String) -> Result<String, String> {
    // Validate id: no path separators, no '.'
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("Invalid theme id".to_string());
    }
    let path = resolve_theme_css_path(&id, &app)
        .ok_or_else(|| format!("Preview theme not found: {id}"))?;

    // Safety check: path must be within allowed dirs
    let mut allowed: Vec<PathBuf> = Vec::new();
    if let Some(d) = user_themes_dir() {
        allowed.push(d);
    }
    if let Some(d) = bundled_themes_dir(&app) {
        allowed.push(d);
    }
    if !path_is_safe(&path, &allowed) {
        return Err("Theme path escapes allowed directories".to_string());
    }

    fs::read_to_string(&path).map_err(|e| format!("Cannot read theme CSS: {e}"))
}

#[tauri::command]
pub(crate) fn open_user_themes_dir() -> Result<String, String> {
    let dir = user_themes_dir().ok_or("Cannot determine user data directory")?;
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create user themes directory: {e}"))?;
    let dir_str = dir.to_string_lossy().into_owned();

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Cannot open directory: {e}"))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Cannot open directory: {e}"))?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Cannot open directory: {e}"))?;
    }

    Ok(dir_str)
}

#[tauri::command]
#[cfg(feature = "native-watch")]
pub(crate) fn watch_preview_theme(
    app: AppHandle,
    state: State<PreviewThemeWatcherState>,
    id: String,
) -> Result<(), String> {
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("Invalid theme id".to_string());
    }
    let path = resolve_theme_css_path(&id, &app)
        .ok_or_else(|| format!("Preview theme not found: {id}"))?;

    // Must be a user theme to be watchable (bundled themes don't change at runtime)
    let user_dir = user_themes_dir().ok_or("Cannot determine user data directory")?;
    let canonical_path = path.canonicalize().map_err(|e| e.to_string())?;
    let canonical_user = user_dir.canonicalize().unwrap_or(user_dir.clone());
    if !canonical_path.starts_with(&canonical_user) {
        return Err("Only user preview themes can be watched".to_string());
    }

    let watch_path = path.clone();
    let emit_app = app.clone();
    let emit_id = id.clone();
    let emit_path = path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| {
            if let Ok(event) = result {
                use notify::EventKind;
                let is_modify = matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_));
                if is_modify {
                    if let Ok(css) = fs::read_to_string(&emit_path) {
                        let _ = emit_app.emit(
                            "preview-theme-changed",
                            ThemeChangedPayload {
                                id: emit_id.clone(),
                                css,
                            },
                        );
                    }
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Cannot create file watcher: {e}"))?;

    watcher
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Cannot watch theme file: {e}"))?;

    let mut state_lock = state
        .watcher
        .lock()
        .map_err(|_| "Theme watcher state lock poisoned")?;
    *state_lock = Some(ActiveThemeWatcher {
        _watcher: watcher,
        id,
    });
    Ok(())
}

#[tauri::command]
#[cfg(feature = "native-watch")]
pub(crate) fn unwatch_preview_theme(state: State<PreviewThemeWatcherState>) -> Result<(), String> {
    let mut lock = state
        .watcher
        .lock()
        .map_err(|_| "Theme watcher state lock poisoned")?;
    *lock = None;
    Ok(())
}

// Stub commands for non-native-watch builds so generate_handler! still compiles.
#[tauri::command]
#[cfg(not(feature = "native-watch"))]
pub(crate) fn watch_preview_theme(
    _app: AppHandle,
    _state: State<PreviewThemeWatcherState>,
    _id: String,
) -> Result<(), String> {
    Err("File watching not available in this build".to_string())
}

#[tauri::command]
#[cfg(not(feature = "native-watch"))]
pub(crate) fn unwatch_preview_theme(_state: State<PreviewThemeWatcherState>) -> Result<(), String> {
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_theme_dir(themes: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for (name, css) in themes {
            fs::write(dir.path().join(format!("{name}.css")), css).unwrap();
        }
        dir
    }

    #[test]
    fn load_themes_from_dir_returns_sorted_themes() {
        let dir = make_theme_dir(&[
            ("dark", "/* Dark theme */\nbody { background: #000; }"),
            ("light", "/* Light theme */\nbody { background: #fff; }"),
        ]);
        let themes = load_themes_from_dir(dir.path(), "bundled");
        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].id, "dark");
        assert_eq!(themes[0].source, "bundled");
        assert_eq!(themes[0].description, Some("Dark theme".to_string()));
        assert_eq!(themes[1].id, "light");
    }

    #[test]
    fn user_theme_overrides_bundled_same_id() {
        let bundled_dir = make_theme_dir(&[("github-light", "/* bundled */")]);
        let user_dir = make_theme_dir(&[("github-light", "/* user */")]);

        let bundled = load_themes_from_dir(bundled_dir.path(), "bundled");
        let user = load_themes_from_dir(user_dir.path(), "user");

        let user_ids: std::collections::HashSet<&str> =
            user.iter().map(|t| t.id.as_str()).collect();
        let mut filtered_bundled = bundled;
        filtered_bundled.retain(|t| !user_ids.contains(t.id.as_str()));

        let mut all = filtered_bundled;
        all.extend(user);
        all.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(all.len(), 1, "only one theme with that id");
        assert_eq!(all[0].source, "user", "user theme should win");
    }

    #[test]
    fn path_is_safe_rejects_traversal() {
        let dir = TempDir::new().unwrap();
        let allowed = vec![dir.path().canonicalize().unwrap()];
        // A file outside the allowed dir
        let outside = PathBuf::from("/tmp/outside.css");
        // We can't canonicalize a non-existent path, so path_is_safe returns false
        assert!(!path_is_safe(&outside, &allowed));
    }

    #[test]
    fn pretty_name_converts_kebab_case() {
        assert_eq!(pretty_name("github-light"), "Github Light");
        assert_eq!(pretty_name("serif-manuscript"), "Serif Manuscript");
    }

    #[test]
    fn description_from_css_reads_first_comment() {
        let css = "/* GitHub Light theme for NEditor preview */\nbody { color: #24292e; }";
        let desc = description_from_css(css);
        assert_eq!(
            desc,
            Some("GitHub Light theme for NEditor preview".to_string())
        );
    }

    #[test]
    fn description_from_css_returns_none_when_no_comment() {
        let css = "body { color: #000; }";
        assert!(description_from_css(css).is_none());
    }
}
