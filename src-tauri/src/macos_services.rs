/// macOS Services provider integration.
///
/// The two services are declared in Info.plist via
/// `tauri.conf.json → bundle.macOS.info.NSServices`:
///   • "Open in NEditor"            – accepts file paths / text → queues for open
///   • "New NEditor document from selection" – accepts text → emits create-new event
///
/// Full NSApplication.setServicesProvider() registration via objc2 `define_class!`
/// requires runtime class registration that is version-sensitive and is left as a
/// TODO stub here.  The Info.plist NSServices entries are fully declared so both
/// services appear in the macOS Services menu; actual invocation is a no-op until
/// the class is registered.
///
/// To complete: implement `NEditorServicesProvider` using objc2's `define_class!`
/// macro and call `NSApp.setServicesProvider(provider)` inside `setup_services`.
#[cfg(target_os = "macos")]
pub(crate) fn setup_services(_app: &tauri::AppHandle) {
    // TODO: Register NEditorServicesProvider ObjC class with NSApp.
    // The NSServices plist entries are already active; once this is wired up,
    // invocations of "Open in NEditor" and "New NEditor document from selection"
    // will call back into Rust via queue_paths_for_open / emit create-new event.
    //
    // Skeleton (needs objc2 "define_class!" macro + appropriate method selectors):
    //
    // define_class!(
    //     #[unsafe(super(NSObject))]
    //     struct NEditorServicesProvider;
    //
    //     unsafe impl NEditorServicesProvider {
    //         #[unsafe(method(openInNEditor:userData:error:))]
    //         fn open_in_neditor(&self, pb: &NSPasteboard, _ud: &NSString, _err: *mut *mut NSError) {
    //             if let Some(text) = pb.string_for_type(NSPasteboardTypeString) {
    //                 let _ = queue_paths_for_open(&[text.to_string()]);
    //             }
    //         }
    //         #[unsafe(method(newNEditorDocumentFromSelection:userData:error:))]
    //         fn new_from_selection(&self, pb: &NSPasteboard, _ud: &NSString, _err: *mut *mut NSError) {
    //             if let Some(text) = pb.string_for_type(NSPasteboardTypeString) {
    //                 let _ = APP_HANDLE.get().map(|h| h.emit("services-new-document", text.to_string()));
    //             }
    //         }
    //     }
    // );
    // let provider = NEditorServicesProvider::new();
    // unsafe { NSApp().setServicesProvider(Some(&provider)); }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn setup_services(_app: &tauri::AppHandle) {}
