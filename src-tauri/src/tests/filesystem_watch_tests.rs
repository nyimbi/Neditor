use super::*;
use filesystem_watch::{
    prime_watcher_state_for_test, watcher_signature_for_test, FileWatcherState, WatchFileRequest,
    WatchFileResponse,
};

/// `watch_file` must report the correct root path so that `start_file_watcher`
/// computes the right signature after a save-as path change.
#[test]
fn watch_file_reflects_new_root_after_save_as() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("neditor-watch-test-{unique}"));
    fs::create_dir_all(&dir).expect("create test dir");

    let old_path = dir.join("draft.md");
    let new_path = dir.join("final.md");
    fs::write(&old_path, "# Draft").expect("write draft");
    fs::write(&new_path, "# Final").expect("write final");

    let old_root = path_to_string(&old_path);
    let new_root = path_to_string(&new_path);

    let old_response: WatchFileResponse = watch_file(WatchFileRequest {
        root: old_root.clone(),
        open_roots: vec![],
        included: vec![],
    })
    .expect("watch_file old");
    assert!(
        old_response
            .paths
            .iter()
            .any(|p| p.path == old_root && p.exists),
        "watch_file must include the original root as an existing path"
    );

    let new_response: WatchFileResponse = watch_file(WatchFileRequest {
        root: new_root.clone(),
        open_roots: vec![],
        included: vec![],
    })
    .expect("watch_file new");
    assert!(
        new_response
            .paths
            .iter()
            .any(|p| p.path == new_root && p.exists),
        "watch_file must include the save-as destination as an existing path"
    );

    // The signatures that `start_file_watcher` would derive must differ so it
    // knows to rebuild the watcher rather than early-returning.
    let old_sig: String = old_response
        .paths
        .iter()
        .filter(|p| p.exists)
        .map(|p| p.path.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let new_sig: String = new_response
        .paths
        .iter()
        .filter(|p| p.exists)
        .map(|p| p.path.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(
        old_sig, new_sig,
        "signatures for different roots must differ so the watcher rebuilds on save-as"
    );

    fs::remove_dir_all(&dir).expect("clean test dir");
}

/// After a save-as the watcher state must carry the new path's signature, not
/// the old one.  This mirrors what `start_file_watcher` does when
/// `syncFileWatcher` is called by the frontend after the path changes.
#[test]
fn watcher_state_signature_moves_on_save_as() {
    let state = FileWatcherState::default();

    // Prime state as if the watcher was started on the original document.
    prime_watcher_state_for_test(&state, "/docs/draft.md");
    assert_eq!(
        watcher_signature_for_test(&state).as_deref(),
        Some("/docs/draft.md"),
        "initial signature must match the original root"
    );

    // Simulate start_file_watcher being called with the save-as destination.
    // The atomic swap replaces the old ActiveFileWatcher in one assignment.
    prime_watcher_state_for_test(&state, "/docs/final.md");
    assert_eq!(
        watcher_signature_for_test(&state).as_deref(),
        Some("/docs/final.md"),
        "signature must move to the save-as destination"
    );
    assert_ne!(
        watcher_signature_for_test(&state).as_deref(),
        Some("/docs/draft.md"),
        "old root must not linger in watcher state after save-as"
    );
}
