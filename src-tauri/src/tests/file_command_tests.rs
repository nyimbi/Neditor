use super::*;

#[test]
fn file_duplicate_and_rename_commands_move_content() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-file-test-{unique}"));
    fs::create_dir_all(&root).expect("create test dir");
    let source = root.join("source.md");
    let copy = root.join("copy.md");
    let renamed = root.join("renamed.md");
    fs::write(&source, "hello").expect("write source");

    let duplicated = duplicate_file(DuplicateFileRequest {
        from: path_to_string(&source),
        to: path_to_string(&copy),
    })
    .expect("duplicate file");
    assert_eq!(duplicated.text, "hello");

    let metadata = rename_file(RenameFileRequest {
        from: path_to_string(&copy),
        to: path_to_string(&renamed),
    })
    .expect("rename file");
    assert!(metadata.exists);
    assert!(renamed.exists());
    fs::remove_dir_all(root).expect("clean test dir");
}

#[test]
fn save_file_rejects_stale_expected_hash() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-save-conflict-test-{unique}"));
    fs::create_dir_all(&root).expect("create test dir");
    let doc = root.join("doc.md");
    fs::write(&doc, "external").expect("write external content");

    let result = save_file(SaveFileRequest {
        path: path_to_string(&doc),
        text: "local".to_string(),
        expected_hash: Some(sha256_hex(b"old")),
    });

    assert!(result
        .expect_err("stale save should fail")
        .contains("File changed on disk"));
    assert_eq!(fs::read_to_string(&doc).expect("read doc"), "external");
    fs::remove_dir_all(root).expect("clean save conflict test dir");
}

#[test]
fn stable_file_ipc_aliases_open_save_as_and_watch_paths() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-ipc-alias-test-{unique}"));
    fs::create_dir_all(root.join("chapters")).expect("create chapters");
    fs::create_dir_all(root.join("appendices")).expect("create appendices");
    let doc = root.join("doc.md");
    let included = root.join("chapters").join("intro.md");
    let nested = root.join("appendices").join("risk.md");
    let copy = root.join("copy.md");
    fs::write(&doc, "# Root\n!include chapters/intro.md").expect("write root");
    fs::write(&included, "# Intro\n{{include ../appendices/risk.md}}").expect("write include");
    fs::write(&nested, "# Risk").expect("write nested include");

    let opened = open_file(path_to_string(&doc)).expect("open file alias");
    assert!(opened.text.contains("# Root"));

    let saved = save_file_as(SaveFileRequest {
        path: path_to_string(&copy),
        text: "# Copy".to_string(),
        expected_hash: Some("stale-hash-ignored-for-save-as".to_string()),
    })
    .expect("save file as alias");
    assert_eq!(saved.text, "# Copy");

    let watched = watch_file(WatchFileRequest {
        root: path_to_string(&doc),
        included: vec![path_to_string(&included), path_to_string(&included)],
    })
    .expect("watch file command");
    assert_eq!(watched.paths.len(), 3);
    assert!(watched.paths.iter().all(|metadata| metadata.exists));
    assert_eq!(watched.paths[0].role, "root");
    assert_eq!(watched.paths[1].role, "include");
    assert!(watched
        .paths
        .iter()
        .any(|metadata| metadata.path.ends_with("chapters/intro.md")));
    assert!(watched
        .paths
        .iter()
        .any(|metadata| metadata.path.ends_with("appendices/risk.md")));
    fs::remove_dir_all(root).expect("clean ipc alias test dir");
}

#[cfg(feature = "native-watch")]
#[test]
fn notify_watcher_ignores_access_only_events() {
    assert!(!notify_event_should_emit(&notify::EventKind::Access(
        notify::event::AccessKind::Any
    )));
    assert!(notify_event_should_emit(&notify::EventKind::Modify(
        notify::event::ModifyKind::Any
    )));
}

#[test]
fn workspace_listing_skips_hidden_and_build_artifacts() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-workspace-test-{unique}"));
    fs::create_dir_all(root.join("chapters")).expect("create chapters");
    fs::create_dir_all(root.join(".git")).expect("create hidden dir");
    fs::create_dir_all(root.join("node_modules")).expect("create node modules");
    fs::write(root.join("root.md"), "# Root").expect("write root doc");
    fs::write(root.join("chapters").join("intro.md"), "# Intro").expect("write child doc");
    fs::write(root.join(".secret.md"), "# Secret").expect("write hidden file");
    fs::write(root.join("node_modules").join("package.md"), "# Dependency")
        .expect("write ignored dependency doc");
    fs::write(root.join("binary.bin"), [0, 1, 2, 3]).expect("write binary");

    let entries = list_workspace_files(WorkspaceFileRequest {
        root: path_to_string(&root),
    })
    .expect("workspace listing");
    let paths = entries
        .iter()
        .map(|entry| entry.relative_path.as_str())
        .collect::<Vec<_>>();

    assert!(paths.contains(&"root.md"));
    assert!(paths.contains(&"chapters"));
    assert!(paths.contains(&"chapters/intro.md"));
    assert!(!paths.iter().any(|path| path.contains(".secret")));
    assert!(!paths.iter().any(|path| path.contains("node_modules")));
    assert!(!paths.iter().any(|path| path.contains("binary.bin")));
    fs::remove_dir_all(root).expect("clean workspace test dir");
}

#[test]
fn git_history_diff_commit_tag_and_restore_workflow() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-git-test-{unique}"));
    fs::create_dir_all(&root).expect("create git test dir");
    run_git(&root, &["init"]).expect("git init");
    run_git(&root, &["config", "user.email", "neditor@example.test"]).expect("git email");
    run_git(&root, &["config", "user.name", "NEditor Test"]).expect("git name");
    let doc = root.join("doc.md");
    fs::write(&doc, "one\n").expect("write initial doc");
    run_git(&root, &["add", "doc.md"]).expect("git add");
    run_git(&root, &["commit", "-m", "Initial document"]).expect("git commit");
    fs::write(&doc, "two\n").expect("write changed doc");

    let status = get_git_status(Some(path_to_string(&doc))).expect("git status");
    assert!(status.inside_repo);
    assert!(status.dirty);
    assert!(status.summary.iter().any(|entry| entry.contains("doc.md")));

    let diff = git_diff(GitPathRequest {
        path: path_to_string(&doc),
    })
    .expect("git diff");
    assert!(diff.contains("-one"));
    assert!(diff.contains("+two"));

    commit_document_changes(GitCommitRequest {
        path: path_to_string(&doc),
        message: "Update document".to_string(),
    })
    .expect("commit command");
    let history = git_history(GitPathRequest {
        path: path_to_string(&doc),
    })
    .expect("history command");
    assert!(history.len() >= 2);

    let tag = tag_release(GitTagRequest {
        path: path_to_string(&doc),
        tag: format!("test-{unique}"),
        message: "Test release".to_string(),
    })
    .expect("tag command");
    assert!(tag.starts_with("test-"));

    let restored = restore_git_revision(GitRestoreRequest {
        path: path_to_string(&doc),
        revision: history
            .last()
            .expect("initial history entry")
            .revision
            .clone(),
    })
    .expect("restore revision");
    assert_eq!(restored.text, "one\n");
    fs::remove_dir_all(root).expect("clean git test dir");
}

#[test]
fn git_restore_and_tag_reject_option_shaped_refs() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-git-ref-safety-test-{unique}"));
    fs::create_dir_all(&root).expect("create git ref safety test dir");
    run_git(&root, &["init"]).expect("git init");
    let doc = root.join("doc.md");
    fs::write(&doc, "safe\n").expect("write doc");

    let tag_error = tag_release(GitTagRequest {
        path: path_to_string(&doc),
        tag: "--force".to_string(),
        message: "Unsafe release".to_string(),
    })
    .expect_err("option-shaped tag should be rejected before git invocation");
    assert!(tag_error.contains("Git tag cannot start with '-'"));

    let revision_error = restore_git_revision(GitRestoreRequest {
        path: path_to_string(&doc),
        revision: "--output=/tmp/neditor-ref-injection".to_string(),
    })
    .expect_err("option-shaped revision should be rejected before git invocation");
    assert!(revision_error.contains("Git revision cannot start with '-'"));
    assert_eq!(fs::read_to_string(&doc).expect("read doc"), "safe\n");

    let syntax_error = restore_git_revision(GitRestoreRequest {
        path: path_to_string(&doc),
        revision: "HEAD@{1}".to_string(),
    })
    .expect_err("reflog syntax should be rejected for explicit restore refs");
    assert!(syntax_error.contains("Git revision contains unsupported ref syntax"));
    fs::remove_dir_all(root).expect("clean git ref safety test dir");
}

#[cfg(unix)]
#[test]
fn git_restore_refuses_symlink_targets() {
    use std::os::unix::fs::symlink;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-git-symlink-test-{unique}"));
    let outside = std::env::temp_dir().join(format!("neditor-git-symlink-outside-{unique}.md"));
    fs::create_dir_all(&root).expect("create git symlink test dir");
    run_git(&root, &["init"]).expect("git init");
    run_git(&root, &["config", "user.email", "neditor@example.test"]).expect("git email");
    run_git(&root, &["config", "user.name", "NEditor Test"]).expect("git name");
    let doc = root.join("doc.md");
    fs::write(&doc, "tracked\n").expect("write tracked doc");
    run_git(&root, &["add", "doc.md"]).expect("git add");
    run_git(&root, &["commit", "-m", "Initial document"]).expect("git commit");

    fs::write(&outside, "outside\n").expect("write outside target");
    fs::remove_file(&doc).expect("remove tracked doc");
    symlink(&outside, &doc).expect("link worktree doc to outside target");

    let error = restore_git_revision(GitRestoreRequest {
        path: path_to_string(&doc),
        revision: "HEAD".to_string(),
    })
    .expect_err("restore should not follow symlink targets");
    assert!(error.contains("Refusing to restore through a symlink"));
    assert_eq!(
        fs::read_to_string(&outside).expect("read outside target"),
        "outside\n"
    );
    fs::remove_dir_all(root).expect("clean git symlink test dir");
    fs::remove_file(outside).expect("clean outside target");
}
