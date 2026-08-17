/// macOS Services provider integration.
///
/// NSServices entries in Info.plist declare two menu items:
///   "Open in NEditor"                     – NSMessage = openInNEditor
///   "New NEditor Document from Selection" – NSMessage = newNEditorDocumentFromSelection
///
/// This module registers `NEditorServicesProvider` as the NSApp services provider
/// so those items actually invoke Rust handlers rather than silently doing nothing.

// ── macOS implementation ──────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::OnceLock;

    use objc2::{define_class, msg_send, msg_send_id, rc::Retained, runtime::AnyObject};
    use objc2::{AnyThread, ClassType};
    use objc2_app_kit::{NSApplication, NSPasteboard};
    use objc2_foundation::{NSArray, NSError, NSString};
    use tauri::Emitter;

    pub(super) static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

    extern "C" {
        /// Notifies the system that the set of available services has changed.
        /// Must be called after setServicesProvider: so new entries are visible
        /// without requiring a logout.
        fn NSUpdateDynamicServices();
    }

    // ── file:// URL helpers ───────────────────────────────────────────────────

    /// Convert a `file://` URL string to an absolute POSIX path.
    pub(super) fn file_url_to_path(url: &str) -> Option<String> {
        let path = if let Some(p) = url.strip_prefix("file://localhost") {
            p
        } else if let Some(p) = url.strip_prefix("file://") {
            p
        } else {
            return None;
        };
        let path = path.trim_end_matches(['\n', '\r']);
        if !path.starts_with('/') {
            return None;
        }
        Some(percent_decode(path))
    }

    pub(super) fn percent_decode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let b = s.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'%' && i + 2 < b.len() {
                if let (Some(h), Some(l)) = (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                    out.push(char::from((h << 4) | l));
                    i += 3;
                    continue;
                }
            }
            out.push(b[i] as char);
            i += 1;
        }
        out
    }

    fn hex_val(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }

    // ── pasteboard helpers ────────────────────────────────────────────────────

    /// Read file paths from a Services pasteboard.
    ///
    /// Tries `public.file-url` per `NSPasteboardItem` first (modern Finder), then
    /// falls back to plain-text lines that exist on disk.
    fn read_file_paths(pb: &NSPasteboard) -> Vec<String> {
        let mut paths = Vec::new();

        let file_url_type = NSString::from_str("public.file-url");
        let items: Option<Retained<NSArray<AnyObject>>> =
            unsafe { msg_send_id![pb, pasteboardItems] };
        if let Some(ref items) = items {
            let count: usize = unsafe { msg_send![items, count] };
            for i in 0..count {
                let item: &AnyObject = unsafe { msg_send![items, objectAtIndex: i] };
                let s: Option<Retained<NSString>> =
                    unsafe { msg_send_id![item, stringForType: &*file_url_type] };
                if let Some(s) = s {
                    if let Some(p) = file_url_to_path(s.to_string().as_str()) {
                        paths.push(p);
                    }
                }
            }
        }

        // Fallback: plain-text – each non-empty line that exists as a path.
        if paths.is_empty() {
            let str_type = NSString::from_str("public.utf8-plain-text");
            let s: Option<Retained<NSString>> =
                unsafe { msg_send_id![pb, stringForType: &*str_type] };
            if let Some(s) = s {
                for line in s.to_string().lines() {
                    let t = line.trim();
                    if !t.is_empty() && std::path::Path::new(t).exists() {
                        paths.push(t.to_string());
                    }
                }
            }
        }

        paths
    }

    // ── ObjC class ────────────────────────────────────────────────────────────

    define_class!(
        /// Registered with `[NSApp setServicesProvider:]`.
        /// Selector names match `NSMessage` values in Info.plist NSServices entries.
        #[unsafe(super(objc2::runtime::NSObject))]
        #[name = "NEditorServicesProvider"]
        pub(super) struct NEditorServicesProvider;

        impl NEditorServicesProvider {
            /// Handles "Open in NEditor" – selector: openInNEditor:userData:error:
            ///
            /// Reads file URLs or paths from the pasteboard, validates them against
            /// workspace-scoping rules, queues them for open, then activates NEditor.
            #[unsafe(method(openInNEditor:userData:error:))]
            fn open_in_neditor(
                &self,
                pb: &NSPasteboard,
                _user_data: Option<&NSString>,
                _error: *mut *mut NSError,
            ) {
                let raw = read_file_paths(pb);
                let valid: Vec<String> = raw
                    .into_iter()
                    .filter(|p| {
                        // root=None → only symlink-escape check, no workspace scoping.
                        crate::filesystem::resolve_within_workspace(p, None, false).is_ok()
                    })
                    .collect();

                if !valid.is_empty() {
                    let _ = crate::cli_ipc::queue_paths_for_open(&valid);
                    if let Some(h) = APP_HANDLE.get() {
                        let h = h.clone();
                        let v = valid.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = h.emit("services-open-files", v);
                        });
                    }
                }

                unsafe {
                    let app: Retained<NSApplication> =
                        msg_send_id![NSApplication::class(), sharedApplication];
                    let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
                }
            }

            /// Handles "New NEditor Document from Selection" –
            /// selector: newNEditorDocumentFromSelection:userData:error:
            ///
            /// Reads text from the pasteboard and emits `services-new-document`
            /// so the frontend opens a new document pre-filled with that text.
            #[unsafe(method(newNEditorDocumentFromSelection:userData:error:))]
            fn new_from_selection(
                &self,
                pb: &NSPasteboard,
                _user_data: Option<&NSString>,
                _error: *mut *mut NSError,
            ) {
                let str_type = NSString::from_str("public.utf8-plain-text");
                let text_ns: Option<Retained<NSString>> =
                    unsafe { msg_send_id![pb, stringForType: &*str_type] };
                let text = match text_ns {
                    Some(s) => s.to_string(),
                    None => return,
                };
                if text.is_empty() {
                    return;
                }

                if let Some(h) = APP_HANDLE.get() {
                    let h = h.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("services-new-document", text);
                    });
                }

                unsafe {
                    let app: Retained<NSApplication> =
                        msg_send_id![NSApplication::class(), sharedApplication];
                    let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
                }
            }
        }
    );

    impl NEditorServicesProvider {
        pub(super) fn new() -> Retained<Self> {
            let this = Self::alloc();
            unsafe { msg_send_id![this, init] }
        }
    }

    // ── registration ──────────────────────────────────────────────────────────

    pub(super) fn register(app: &tauri::AppHandle) {
        // Tauri's setup hook is guaranteed to run on the main thread.
        // AppKit APIs are not thread-safe; assert this in debug builds.
        #[cfg(debug_assertions)]
        {
            extern "C" {
                fn pthread_main_np() -> std::os::raw::c_int;
            }
            debug_assert_ne!(
                unsafe { pthread_main_np() },
                0,
                "setup_services must be called on the main thread"
            );
        }

        let _ = APP_HANDLE.set(app.clone());

        let provider = NEditorServicesProvider::new();
        unsafe {
            let ns_app: Retained<NSApplication> =
                msg_send_id![NSApplication::class(), sharedApplication];
            // setServicesProvider: retains the provider object on the NSApp side.
            let _: () = msg_send![&*ns_app, setServicesProvider: &*provider];
            // Inform the system so the new services appear without a logout.
            NSUpdateDynamicServices();
        }
        // NSApp now holds a strong reference. Intentionally leak Rust's retain so
        // the object lives for the entire process lifetime without a static store.
        std::mem::forget(provider);
    }
}

// ── public entry points ───────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub(crate) fn setup_services(app: &tauri::AppHandle) {
    imp::register(app);
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn setup_services(_app: &tauri::AppHandle) {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::imp::{file_url_to_path, percent_decode};

    #[test]
    fn file_url_to_path_parses_correctly() {
        assert_eq!(
            file_url_to_path("file:///Users/alice/doc.md"),
            Some("/Users/alice/doc.md".to_string()),
            "bare file:// scheme"
        );
        assert_eq!(
            file_url_to_path("file://localhost/Users/alice/doc.md"),
            Some("/Users/alice/doc.md".to_string()),
            "file://localhost form"
        );
        assert_eq!(
            file_url_to_path("file:///Users/alice/my%20doc.md"),
            Some("/Users/alice/my doc.md".to_string()),
            "percent-encoded space"
        );
        assert_eq!(
            file_url_to_path("file:///path/with/%41/%42"),
            Some("/path/with/A/B".to_string()),
            "hex letter sequences"
        );
        assert_eq!(
            file_url_to_path("http://example.com/file"),
            None,
            "wrong scheme"
        );
        assert_eq!(file_url_to_path("not-a-url"), None, "no scheme");
        assert_eq!(file_url_to_path("file://"), None, "empty path after scheme");
    }

    #[test]
    fn percent_decode_handles_common_sequences() {
        assert_eq!(percent_decode("/path/my%20file.md"), "/path/my file.md");
        assert_eq!(percent_decode("/no-encoding"), "/no-encoding");
        assert_eq!(percent_decode("%2F"), "/");
        assert_eq!(percent_decode("%41%42"), "AB");
        // Truncated sequence (only one hex digit) passes through literally.
        assert_eq!(percent_decode("a%2"), "a%2");
        // Non-hex chars after % pass through literally.
        assert_eq!(percent_decode("%ZZ"), "%ZZ");
        assert_eq!(percent_decode(""), "");
    }

    /// Verifies that `define_class!` correctly registers `NEditorServicesProvider`
    /// and that an instance can be allocated.  Requires AppKit runtime.
    #[test]
    #[ignore = "requires AppKit runtime (NSApplication); run with --ignored on macOS"]
    fn services_provider_class_registers_and_allocates() {
        use objc2::{msg_send_id, rc::Retained, ClassType};
        use objc2_app_kit::NSApplication;
        // Bypass MainThreadMarker in test context; we know we're on the main thread.
        let _: Retained<NSApplication> =
            unsafe { msg_send_id![NSApplication::class(), sharedApplication] };
        let _prov = super::imp::NEditorServicesProvider::new();
        // Reaching here without abort/panic = class registered successfully.
    }
}
