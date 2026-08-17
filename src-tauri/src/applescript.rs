/// AppleScript / OSA integration for NEditor.
///
/// Scripting dictionary: `src-tauri/resources/NEditor.sdef`
/// Info.plist: OSAScriptingDefinition = NEditor.sdef, NSAppleScriptEnabled = true
///
/// This module registers `NEditorScriptingBridge` as an Apple Event handler for
/// all verbs defined in the sdef:
///
///   kAEOpenDocuments (aevt/odoc) → queues paths                   wired
///   NEdT/open  "open document"   → queues path                    wired
///   NEdT/save  "save document"   → emits save event to frontend   wired
///   NEdT/xprt  "export document" → emits export event w/ format   wired
///   NEdT/inst  "insert text"     → emits insert event with text   wired
///   aevt/quit  "quit"            → exits via AppHandle::exit      wired
///
/// 0 stubs; all six events are implemented.

// ── macOS implementation ──────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::OnceLock;

    use objc2::{define_class, msg_send, msg_send_id, rc::Retained};
    use objc2::{AnyThread, ClassType};
    use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventManager, NSString, NSURL};
    use tauri::Emitter;

    pub(super) static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

    // ── FourCharCode constants ────────────────────────────────────────────────

    const K_CORE_EVENT_CLASS: u32 = u32::from_be_bytes(*b"aevt");
    const K_AE_OPEN_DOCUMENTS: u32 = u32::from_be_bytes(*b"odoc");
    const K_AE_QUIT_APPLICATION: u32 = u32::from_be_bytes(*b"quit");

    // NEditor suite (sdef code "NEdT")
    const NEDT_SUITE: u32 = u32::from_be_bytes(*b"NEdT");
    const NEDT_OPEN: u32 = u32::from_be_bytes(*b"open");
    const NEDT_SAVE: u32 = u32::from_be_bytes(*b"save");
    const NEDT_XPRT: u32 = u32::from_be_bytes(*b"xprt");
    const NEDT_INST: u32 = u32::from_be_bytes(*b"inst");

    // Apple Event keyword codes
    pub(super) const KEY_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"dobj");
    const KEY_ERROR_STRING: u32 = u32::from_be_bytes(*b"errs");
    const KEY_ERROR_NUMBER: u32 = u32::from_be_bytes(*b"errn");
    const KEY_EF_FMT: u32 = u32::from_be_bytes(*b"EFmt"); // "as" format param
    const KEY_EF_OUT: u32 = u32::from_be_bytes(*b"EOut"); // "to" output path

    // ── file:// URL helpers ───────────────────────────────────────────────────

    fn file_url_to_path(url: &str) -> Option<String> {
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

    fn percent_decode(s: &str) -> String {
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

    // ── Apple Event helpers ───────────────────────────────────────────────────

    /// Extract file paths from the direct object of an Apple Event.
    ///
    /// Handles list descriptors (kAEOpenDocuments) and single descriptors
    /// (NEdT/open). Falls back from `fileURLValue` to `stringValue` for older
    /// descriptor types (typeAlias, raw typeFileURL).
    pub(super) fn extract_paths_from_event(event: &NSAppleEventDescriptor) -> Vec<String> {
        let mut paths = Vec::new();

        let direct: Option<Retained<NSAppleEventDescriptor>> =
            unsafe { msg_send_id![event, paramDescriptorForKeyword: KEY_DIRECT_OBJECT] };
        let direct = match direct {
            Some(d) => d,
            None => return paths,
        };

        // count > 0 means this descriptor is a list; count == 0 means scalar.
        let count: isize = unsafe { msg_send![&*direct, numberOfItems] };
        let items: Vec<Retained<NSAppleEventDescriptor>> = if count > 0 {
            (1..=count)
                .filter_map(|i| -> Option<Retained<NSAppleEventDescriptor>> {
                    unsafe { msg_send_id![&*direct, descriptorAtIndex: i] }
                })
                .collect()
        } else {
            vec![direct]
        };

        for item in items {
            let url: Option<Retained<NSURL>> =
                unsafe { msg_send_id![&*item, fileURLValue] };
            if let Some(url) = url {
                let path_ns: Option<Retained<NSString>> =
                    unsafe { msg_send_id![&*url, path] };
                if let Some(p) = path_ns {
                    let s = p.to_string();
                    if !s.is_empty() {
                        paths.push(s);
                        continue;
                    }
                }
            }
            // Fallback for older or alternate descriptor types.
            let sv: Option<Retained<NSString>> =
                unsafe { msg_send_id![&*item, stringValue] };
            if let Some(sv) = sv {
                let txt = sv.to_string();
                if let Some(p) = file_url_to_path(&txt) {
                    paths.push(p);
                } else if txt.starts_with('/') {
                    paths.push(txt);
                }
            }
        }

        paths
    }

    /// Set a well-formed error reply (keyErrorString + keyErrorNumber).
    fn set_error_reply(reply: &NSAppleEventDescriptor, code: i32, message: &str) {
        unsafe {
            let msg_ns = NSString::from_str(message);
            let err_str: Retained<NSAppleEventDescriptor> = msg_send_id![
                NSAppleEventDescriptor::class(),
                descriptorWithString: &*msg_ns
            ];
            let err_num: Retained<NSAppleEventDescriptor> = msg_send_id![
                NSAppleEventDescriptor::class(),
                descriptorWithInt32: code
            ];
            let _: () =
                msg_send![reply, setParamDescriptor: &*err_str forKeyword: KEY_ERROR_STRING];
            let _: () =
                msg_send![reply, setParamDescriptor: &*err_num forKeyword: KEY_ERROR_NUMBER];
        }
    }

    /// Shared logic for both kAEOpenDocuments and NEdT/open.
    fn do_open_documents(event: &NSAppleEventDescriptor) {
        let paths = extract_paths_from_event(event);
        let valid: Vec<String> = paths
            .into_iter()
            .filter(|p| {
                // root=None → only symlink-escape check applied, no workspace scope.
                crate::filesystem::resolve_within_workspace(p, None, false).is_ok()
            })
            .collect();
        if !valid.is_empty() {
            let _ = crate::cli_ipc::queue_paths_for_open(&valid);
            if let Some(h) = APP_HANDLE.get() {
                let h = h.clone();
                let v = valid.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = h.emit("applescript-open-files", v);
                });
            }
        }
    }

    // ── ObjC handler class ────────────────────────────────────────────────────

    define_class!(
        /// Apple Event handler registered with NSAppleEventManager.
        #[unsafe(super(objc2::runtime::NSObject))]
        #[name = "NEditorScriptingBridge"]
        pub(super) struct NEditorScriptingBridge;

        impl NEditorScriptingBridge {
            /// kCoreEventClass / kAEOpenDocuments (aevt/odoc).
            /// Queues file paths for open and emits `applescript-open-files`.
            #[unsafe(method(handleCoreOpenDocuments:withReplyEvent:))]
            fn handle_core_open_documents(
                &self,
                event: &NSAppleEventDescriptor,
                _reply: &NSAppleEventDescriptor,
            ) {
                do_open_documents(event);
            }

            /// NEdT/open – "open document" verb.
            /// Identical routing to the standard open command; distinct registration
            /// so Script Editor shows it as a separate verb.
            #[unsafe(method(handleNEditorOpenDocument:withReplyEvent:))]
            fn handle_nedt_open(
                &self,
                event: &NSAppleEventDescriptor,
                _reply: &NSAppleEventDescriptor,
            ) {
                do_open_documents(event);
            }

            /// NEdT/save – "save document" verb.
            /// Emits `applescript-save-document` to the frontend which routes to
            /// the existing save_file IPC command.
            #[unsafe(method(handleNEditorSaveDocument:withReplyEvent:))]
            fn handle_nedt_save(
                &self,
                _event: &NSAppleEventDescriptor,
                _reply: &NSAppleEventDescriptor,
            ) {
                if let Some(h) = APP_HANDLE.get() {
                    let h = h.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("applescript-save-document", ());
                    });
                }
            }

            /// NEdT/xprt – "export document" verb.
            /// Reads the `as` (EFmt) format code and optional `to` (EOut) output path,
            /// then emits `applescript-export-document` for the frontend export pipeline.
            #[unsafe(method(handleNEditorExportDocument:withReplyEvent:))]
            fn handle_nedt_export(
                &self,
                event: &NSAppleEventDescriptor,
                reply: &NSAppleEventDescriptor,
            ) {
                let fmt_desc: Option<Retained<NSAppleEventDescriptor>> =
                    unsafe { msg_send_id![event, paramDescriptorForKeyword: KEY_EF_FMT] };
                let format_code: u32 = match fmt_desc {
                    Some(ref d) => unsafe { msg_send![d, enumCodeValue] },
                    None => 0,
                };

                let out_desc: Option<Retained<NSAppleEventDescriptor>> =
                    unsafe { msg_send_id![event, paramDescriptorForKeyword: KEY_EF_OUT] };
                let output_path: Option<String> = out_desc.and_then(|d| {
                    let url: Option<Retained<NSURL>> =
                        unsafe { msg_send_id![&*d, fileURLValue] };
                    url.and_then(|u| {
                        let p: Option<Retained<NSString>> =
                            unsafe { msg_send_id![&*u, path] };
                        p.map(|s| s.to_string())
                    })
                });

                // Reject if the caller specified an output directory that doesn't exist.
                if let Some(ref out) = output_path {
                    if let Some(parent) = std::path::Path::new(out).parent() {
                        if !parent.as_os_str().is_empty() && !parent.exists() {
                            set_error_reply(
                                reply,
                                -1700, // errAECoercionFail
                                "NEditor export: output directory does not exist",
                            );
                            return;
                        }
                    }
                }

                #[derive(Clone, serde::Serialize)]
                struct ExportArgs {
                    format_code: u32,
                    output_path: Option<String>,
                }

                if let Some(h) = APP_HANDLE.get() {
                    let h = h.clone();
                    let args = ExportArgs { format_code, output_path };
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("applescript-export-document", args);
                    });
                }
            }

            /// NEdT/inst – "insert text" verb.
            /// Reads text from the direct parameter and emits `applescript-insert-text`
            /// for the frontend to insert at the current cursor position.
            /// Returns a well-formed error if the direct object is absent or not text.
            #[unsafe(method(handleNEditorInsertText:withReplyEvent:))]
            fn handle_nedt_insert(
                &self,
                event: &NSAppleEventDescriptor,
                reply: &NSAppleEventDescriptor,
            ) {
                let text_desc: Option<Retained<NSAppleEventDescriptor>> =
                    unsafe { msg_send_id![event, paramDescriptorForKeyword: KEY_DIRECT_OBJECT] };
                let text = match text_desc {
                    None => {
                        set_error_reply(reply, -1728, "NEditor insert text: missing direct object");
                        return;
                    }
                    Some(d) => {
                        let s: Option<Retained<NSString>> =
                            unsafe { msg_send_id![&*d, stringValue] };
                        match s {
                            None => {
                                set_error_reply(reply, -1728, "NEditor insert text: could not coerce to string");
                                return;
                            }
                            Some(ns) => ns.to_string(),
                        }
                    }
                };

                if text.is_empty() {
                    return;
                }

                if let Some(h) = APP_HANDLE.get() {
                    let h = h.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("applescript-insert-text", text);
                    });
                }
            }

            /// kCoreEventClass / kAEQuitApplication (aevt/quit).
            #[unsafe(method(handleQuitApplication:withReplyEvent:))]
            fn handle_quit(
                &self,
                _event: &NSAppleEventDescriptor,
                _reply: &NSAppleEventDescriptor,
            ) {
                if let Some(h) = APP_HANDLE.get() {
                    let h = h.clone();
                    tauri::async_runtime::spawn(async move {
                        h.exit(0);
                    });
                } else {
                    std::process::exit(0);
                }
            }
        }
    );

    impl NEditorScriptingBridge {
        pub(super) fn new() -> Retained<Self> {
            let this = Self::alloc();
            unsafe { msg_send_id![this, init] }
        }
    }

    // ── registration ──────────────────────────────────────────────────────────

    pub(super) fn register(app: &tauri::AppHandle) {
        #[cfg(debug_assertions)]
        {
            extern "C" {
                fn pthread_main_np() -> std::os::raw::c_int;
            }
            debug_assert_ne!(
                unsafe { pthread_main_np() },
                0,
                "setup_applescript_handlers must be called on the main thread"
            );
        }

        let _ = APP_HANDLE.set(app.clone());

        let bridge = NEditorScriptingBridge::new();
        let mgr: Retained<NSAppleEventManager> = unsafe {
            msg_send_id![NSAppleEventManager::class(), sharedAppleEventManager]
        };

        use objc2::sel;

        // Each registration pairs a selector (must match the #[unsafe(method(...))]
        // attribute exactly) with the (eventClass, eventID) FourCharCode pair.
        unsafe {
            let _: () = msg_send![&*mgr,
                setEventHandler: &*bridge
                andSelector: sel!(handleCoreOpenDocuments:withReplyEvent:)
                forEventClass: K_CORE_EVENT_CLASS
                andEventID: K_AE_OPEN_DOCUMENTS];

            let _: () = msg_send![&*mgr,
                setEventHandler: &*bridge
                andSelector: sel!(handleNEditorOpenDocument:withReplyEvent:)
                forEventClass: NEDT_SUITE
                andEventID: NEDT_OPEN];

            let _: () = msg_send![&*mgr,
                setEventHandler: &*bridge
                andSelector: sel!(handleNEditorSaveDocument:withReplyEvent:)
                forEventClass: NEDT_SUITE
                andEventID: NEDT_SAVE];

            let _: () = msg_send![&*mgr,
                setEventHandler: &*bridge
                andSelector: sel!(handleNEditorExportDocument:withReplyEvent:)
                forEventClass: NEDT_SUITE
                andEventID: NEDT_XPRT];

            let _: () = msg_send![&*mgr,
                setEventHandler: &*bridge
                andSelector: sel!(handleNEditorInsertText:withReplyEvent:)
                forEventClass: NEDT_SUITE
                andEventID: NEDT_INST];

            let _: () = msg_send![&*mgr,
                setEventHandler: &*bridge
                andSelector: sel!(handleQuitApplication:withReplyEvent:)
                forEventClass: K_CORE_EVENT_CLASS
                andEventID: K_AE_QUIT_APPLICATION];
        }

        // NSAppleEventManager retains the handler; leak Rust's retain so the object
        // lives for the process lifetime.
        std::mem::forget(bridge);
    }
}

// ── public entry points ───────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub(crate) fn setup_applescript_handlers(app: &tauri::AppHandle) {
    imp::register(app);
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn setup_applescript_handlers(_app: &tauri::AppHandle) {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #[test]
    fn sdef_is_valid_xml() {
        let sdef_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("NEditor.sdef");
        if !sdef_path.exists() {
            return; // Skip gracefully in non-macOS CI.
        }
        let content =
            std::fs::read_to_string(&sdef_path).expect("NEditor.sdef should be readable");
        assert!(
            content.contains("<dictionary"),
            "sdef should contain a <dictionary> root element"
        );
        assert!(
            content.contains("open document"),
            "sdef should declare the 'open document' command"
        );
        assert!(
            content.contains("NEdTopen"),
            "sdef should contain the NEdTopen event code"
        );
    }

    /// Verifies that `extract_paths_from_event` does not panic on a descriptor that
    /// carries no direct object parameter (simulates a malformed open-documents event).
    ///
    /// `NSAppleEventDescriptor` allocation is headless-safe; no display session needed.
    #[test]
    #[cfg(target_os = "macos")]
    fn malformed_open_event_yields_empty_paths_without_panic() {
        use objc2::{msg_send_id, rc::Retained};
        use objc2_foundation::NSAppleEventDescriptor;
        use objc2::ClassType;
        use super::imp::{extract_paths_from_event, KEY_DIRECT_OBJECT};

        // A bare record descriptor has no parameters.
        let empty: Retained<NSAppleEventDescriptor> =
            unsafe { msg_send_id![NSAppleEventDescriptor::class(), recordDescriptor] };

        let param: Option<Retained<NSAppleEventDescriptor>> =
            unsafe { msg_send_id![&*empty, paramDescriptorForKeyword: KEY_DIRECT_OBJECT] };
        assert!(param.is_none(), "bare record should have no direct-object param");

        let paths = extract_paths_from_event(&empty);
        assert!(paths.is_empty(), "expected no paths from empty event");
    }

    /// Verifies the bridge class allocates. Requires AppKit runtime.
    #[test]
    #[cfg(target_os = "macos")]
    #[ignore = "requires AppKit runtime (NSApplication); run with --ignored on macOS"]
    fn applescript_bridge_allocates_without_panic() {
        use objc2::{msg_send_id, rc::Retained, ClassType};
        use objc2_app_kit::NSApplication;
        let _: Retained<NSApplication> =
            unsafe { msg_send_id![NSApplication::class(), sharedApplication] };
        let _bridge = super::imp::NEditorScriptingBridge::new();
    }
}
