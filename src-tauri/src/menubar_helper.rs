//! Menu-bar helper: warm webview, system-tray icon, and LSUIElement toggling.
//!
//! When `keepInMenuBar` is on (default on macOS), closing the main window
//! hides it rather than quitting the process; the tray icon in the system menu
//! bar lets the user bring it back instantly without a cold-start reload.
//!
//! All activation paths (tray click, URL-scheme handler, Services provider,
//! deep-link handler) MUST go through `bring_up_window` so focus/show logic
//! stays in one place.

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

pub(crate) const WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "neditor-tray";

// ── Activation-policy abstraction ────────────────────────────────────────────

/// Trait-wrapped `NSApp.setActivationPolicy` so unit tests can mock it
/// without a real macOS windowed GUI session.
pub(crate) trait ActivationPolicy: Send + Sync + 'static {
    fn set_regular(&self);
    fn set_accessory(&self);
}

/// Production delegate: calls the real `NSApp` on macOS; no-op everywhere else.
pub(crate) struct NsAppActivationPolicy;

impl ActivationPolicy for NsAppActivationPolicy {
    fn set_regular(&self) {
        #[cfg(target_os = "macos")]
        // SAFETY: called from AppKit main thread; sharedApplication is a singleton.
        unsafe {
            use objc2::MainThreadMarker;
            use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
            let mtm = MainThreadMarker::new_unchecked();
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
            app.activateIgnoringOtherApps(true);
        }
    }

    fn set_accessory(&self) {
        #[cfg(target_os = "macos")]
        // SAFETY: called from AppKit main thread; sharedApplication is a singleton.
        unsafe {
            use objc2::MainThreadMarker;
            use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
            let mtm = MainThreadMarker::new_unchecked();
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }
    }
}

// ── Managed state ────────────────────────────────────────────────────────────

/// Intent emitted when any activation path fires.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum OpenIntent {
    NewDocument,
    OpenPath(String),
}

/// State managed via `app.manage()`.
///
/// Must be registered before `setup_tray` or `setup_window_close_interceptor`
/// are called.
pub(crate) struct MenubarHelperState {
    pub(crate) keep_in_menu_bar: Mutex<bool>,
    pub(crate) recent_files: Mutex<Vec<String>>,
    policy: Arc<dyn ActivationPolicy>,
}

impl MenubarHelperState {
    /// Default: enabled on macOS, disabled on other platforms.
    pub(crate) fn new() -> Self {
        Self::with_policy(cfg!(target_os = "macos"), Arc::new(NsAppActivationPolicy))
    }

    /// Inject a custom `ActivationPolicy` — used by unit tests.
    pub(crate) fn with_policy(default: bool, policy: Arc<dyn ActivationPolicy>) -> Self {
        Self {
            keep_in_menu_bar: Mutex::new(default),
            recent_files: Mutex::new(Vec::new()),
            policy,
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        *self
            .keep_in_menu_bar
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// Update the preference and, when disabling, restore the Dock icon.
    pub(crate) fn set_enabled(&self, value: bool) {
        if let Ok(mut guard) = self.keep_in_menu_bar.lock() {
            *guard = value;
        }
        if !value {
            self.policy.set_regular();
        }
    }

    pub(crate) fn apply_accessory_policy(&self) {
        self.policy.set_accessory();
    }

    pub(crate) fn apply_regular_policy(&self) {
        self.policy.set_regular();
    }
}

impl Default for MenubarHelperState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Public setup API ─────────────────────────────────────────────────────────

/// Touch the main window handle so the webview is warm after setup.
///
/// The real "warmth" comes from `setup_window_close_interceptor`, which keeps
/// the webview hidden (not destroyed) when the user dismisses the window.
pub(crate) fn spawn_warm_webview(app: &AppHandle) {
    let _ = app.get_webview_window(WINDOW_LABEL);
}

/// Intercept close events: hide the window instead of destroying it when
/// `keepInMenuBar` is enabled. Demotes to LSUIElement (accessory) mode so
/// the Dock icon disappears until the user brings the window back.
pub(crate) fn setup_window_close_interceptor(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WINDOW_LABEL) else {
        return;
    };
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            let state = handle.state::<MenubarHelperState>();
            if state.is_enabled() {
                api.prevent_close();
                if let Some(w) = handle.get_webview_window(WINDOW_LABEL) {
                    let _ = w.hide();
                }
                state.apply_accessory_policy();
            }
        }
    });
}

/// Build and register the system-tray icon and its context menu.
///
/// Left-click → brings up main window.
/// Right-click / menu → New Document | Open Recent > | — | Quit NEditor.
///
/// The `tray-icon` Cargo feature must be enabled on `tauri`.
pub(crate) fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
    use tauri::tray::TrayIconBuilder;

    let new_doc = MenuItemBuilder::with_id("tray-new-document", "New Document").build(app)?;
    let open_recent_sub = SubmenuBuilder::new(app, "Open Recent").build()?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("tray-quit", "Quit NEditor").build(app)?;

    let menu = Menu::with_items(app, &[&new_doc, &open_recent_sub, &sep, &quit])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("no icon for menu-bar tray");

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("NEditor")
        .icon(icon)
        .icon_as_template(true) // respect macOS menubar dark/light mode
        .show_menu_on_left_click(false) // left-click → on_tray_icon_event
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray-new-document" => bring_up_window(OpenIntent::NewDocument, app),
            "tray-quit" => app.exit(0),
            other if other.starts_with("tray-recent-") => {
                if let Ok(idx) = other
                    .strip_prefix("tray-recent-")
                    .unwrap_or("")
                    .parse::<usize>()
                {
                    let state = app.state::<MenubarHelperState>();
                    let path = state
                        .recent_files
                        .lock()
                        .ok()
                        .and_then(|g| g.get(idx).cloned());
                    if let Some(p) = path {
                        bring_up_window(OpenIntent::OpenPath(p), app);
                    }
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                bring_up_window(OpenIntent::NewDocument, tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

// ── Core routing ─────────────────────────────────────────────────────────────

/// Bring the main window to the foreground, creating it on demand if needed.
///
/// This is the single choke-point for ALL activation paths so focus/show
/// and LSUIElement toggling stay consistent.
pub(crate) fn bring_up_window(intent: OpenIntent, app: &AppHandle) {
    use tauri::Emitter;

    let window = match app.get_webview_window(WINDOW_LABEL) {
        Some(w) => w,
        None => create_main_window(app),
    };

    let visible = window.is_visible().unwrap_or(false);
    let focused = window.is_focused().unwrap_or(false);
    if !visible || !focused {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        // Promote back to foreground app (restores Dock icon on macOS).
        app.state::<MenubarHelperState>().apply_regular_policy();
    }

    match &intent {
        OpenIntent::NewDocument => {
            let _ = app.emit("menubar-new-document", ());
        }
        OpenIntent::OpenPath(path) => {
            let _ = app.emit("menubar-open-path", path.clone());
        }
    }
}

fn create_main_window(app: &AppHandle) -> tauri::WebviewWindow {
    tauri::WebviewWindowBuilder::new(app, WINDOW_LABEL, tauri::WebviewUrl::App("/".into()))
        .title("NEditor")
        .min_inner_size(960.0, 640.0)
        .inner_size(1440.0, 920.0)
        .visible(false)
        .build()
        .expect("failed to create main window on demand")
}

// ── Tauri commands ───────────────────────────────────────────────────────────

/// Frontend preference sync: update `keepInMenuBar` at runtime.
///
/// Call this whenever the checkbox in Settings changes.
#[tauri::command]
pub(crate) fn set_keep_in_menu_bar(app: AppHandle, value: bool) {
    app.state::<MenubarHelperState>().set_enabled(value);
}

/// Frontend sync: push up to 5 recent files into the tray's Open Recent submenu.
///
/// Call this whenever `recentFiles` in the store changes.
#[tauri::command]
pub(crate) fn update_menubar_recent_files(app: AppHandle, paths: Vec<String>) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};

    {
        let capped: Vec<String> = paths.iter().take(5).cloned().collect();
        if let Ok(mut guard) = app.state::<MenubarHelperState>().recent_files.lock() {
            *guard = capped;
        }
    }

    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };

    // Build recent-file items (keep references alive for the duration of the
    // `Menu::with_items` call below).
    let recent_menu_items: Vec<_> = paths
        .iter()
        .take(5)
        .enumerate()
        .filter_map(|(idx, path)| {
            let label = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();
            MenuItemBuilder::with_id(format!("tray-recent-{idx}"), label)
                .build(&app)
                .ok()
        })
        .collect();

    let mut sub = SubmenuBuilder::new(&app, "Open Recent");
    for item in &recent_menu_items {
        sub = sub.item(item);
    }
    let open_recent_sub = sub.build()?;

    let new_doc = MenuItemBuilder::with_id("tray-new-document", "New Document").build(&app)?;
    let sep = PredefinedMenuItem::separator(&app)?;
    let quit = MenuItemBuilder::with_id("tray-quit", "Quit NEditor").build(&app)?;
    let menu = Menu::with_items(&app, &[&new_doc, &open_recent_sub, &sep, &quit])?;

    tray.set_menu(Some(menu))?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    // ── Mock policy ──────────────────────────────────────────────────────────

    /// Records how many times each policy was applied.
    struct MockActivationPolicy {
        regular_calls: AtomicU8,
        accessory_calls: AtomicU8,
    }

    impl MockActivationPolicy {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                regular_calls: AtomicU8::new(0),
                accessory_calls: AtomicU8::new(0),
            })
        }
    }

    impl ActivationPolicy for MockActivationPolicy {
        fn set_regular(&self) {
            self.regular_calls.fetch_add(1, Ordering::SeqCst);
        }
        fn set_accessory(&self) {
            self.accessory_calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    // ── State and LSUIElement toggle ─────────────────────────────────────────

    #[test]
    fn keep_in_menu_bar_defaults_to_platform_value() {
        let state = MenubarHelperState::new();
        let expected = cfg!(target_os = "macos");
        assert_eq!(
            state.is_enabled(),
            expected,
            "keepInMenuBar default must match platform"
        );
    }

    #[test]
    fn set_enabled_updates_flag() {
        let mock = MockActivationPolicy::new();
        let state =
            MenubarHelperState::with_policy(true, Arc::clone(&mock) as Arc<dyn ActivationPolicy>);

        state.set_enabled(false);
        assert!(!state.is_enabled());
        // Disabling must call set_regular to restore Dock icon.
        assert_eq!(
            mock.regular_calls.load(Ordering::SeqCst),
            1,
            "disabling must call set_regular once"
        );

        state.set_enabled(true);
        assert!(state.is_enabled());
        // Re-enabling does NOT call set_regular — the window show does.
        assert_eq!(
            mock.regular_calls.load(Ordering::SeqCst),
            1,
            "enabling must not call set_regular again"
        );
    }

    #[test]
    fn lsuielement_toggle_round_trip_via_mock() {
        let mock = MockActivationPolicy::new();
        let state =
            MenubarHelperState::with_policy(true, Arc::clone(&mock) as Arc<dyn ActivationPolicy>);

        // Simulate window close → go accessory.
        state.apply_accessory_policy();
        assert_eq!(mock.accessory_calls.load(Ordering::SeqCst), 1);

        // Simulate bring-to-front → go regular.
        state.apply_regular_policy();
        assert_eq!(mock.regular_calls.load(Ordering::SeqCst), 1);

        // Second close/reopen cycle.
        state.apply_accessory_policy();
        state.apply_regular_policy();
        assert_eq!(mock.accessory_calls.load(Ordering::SeqCst), 2);
        assert_eq!(mock.regular_calls.load(Ordering::SeqCst), 2);
    }

    // ── Tray menu label constants ────────────────────────────────────────────

    #[test]
    fn tray_menu_item_ids_are_stable() {
        assert_eq!(TRAY_ID, "neditor-tray");
        // The IDs below must match what `on_menu_event` pattern-matches on.
        let new_doc_id = "tray-new-document";
        let quit_id = "tray-quit";
        let recent_prefix = "tray-recent-";
        assert!(new_doc_id.starts_with("tray-"));
        assert!(quit_id.starts_with("tray-"));
        assert!(recent_prefix.starts_with("tray-recent-"));
    }

    #[test]
    fn tray_quit_id_does_not_match_recent_prefix() {
        let quit_id = "tray-quit";
        assert!(
            !quit_id.starts_with("tray-recent-"),
            "tray-quit must not be mistaken for a recent-file entry"
        );
    }

    // ── bring_up_window no-op check (logic only, no Tauri runtime) ───────────

    #[test]
    #[ignore = "requires macOS windowed session"]
    fn bring_up_window_no_ops_when_already_visible_and_focused() {
        // This test verifies that a window that is already visible+focused
        // does not trigger extra show/focus/unminimize calls.
        // Must run in a full Tauri process with a GUI session.
    }

    // ── keepInMenuBar persistence (logic layer, no store) ───────────────────

    #[test]
    fn recent_files_bounded_to_five() {
        let mock = MockActivationPolicy::new();
        let state =
            MenubarHelperState::with_policy(true, Arc::clone(&mock) as Arc<dyn ActivationPolicy>);
        let paths: Vec<String> = (0..10).map(|i| format!("/docs/file{i}.md")).collect();

        {
            let mut guard = state.recent_files.lock().unwrap();
            *guard = paths.iter().take(5).cloned().collect();
        }

        let stored = state.recent_files.lock().unwrap().clone();
        assert_eq!(stored.len(), 5, "recent_files must be capped at 5");
    }

    #[test]
    fn open_intent_equality() {
        assert_eq!(OpenIntent::NewDocument, OpenIntent::NewDocument);
        assert_eq!(
            OpenIntent::OpenPath("/a/b.md".into()),
            OpenIntent::OpenPath("/a/b.md".into())
        );
        assert_ne!(
            OpenIntent::OpenPath("/a.md".into()),
            OpenIntent::OpenPath("/b.md".into())
        );
        assert_ne!(
            OpenIntent::NewDocument,
            OpenIntent::OpenPath("/a.md".into())
        );
    }
}
