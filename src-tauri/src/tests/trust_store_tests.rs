use super::*;
use std::os::unix::fs::PermissionsExt;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Write a small executable script and return its PathBuf.
#[cfg(unix)]
fn temp_exec(prefix: &str, body: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("neditor-ts-{prefix}-{unique}.sh"));
    fs::write(&path, body).expect("write temp exec");
    let mut perms = fs::metadata(&path).expect("temp exec meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod temp exec");
    path
}

// ── basic grant / revoke / is_trusted ────────────────────────────────────────

#[cfg(unix)]
#[test]
fn trust_store_grant_makes_path_trusted() {
    let engine = temp_exec("grant", "#!/bin/sh\necho ok\n");
    let store = TrustStore::ephemeral();
    assert!(
        !store.is_trusted(&engine),
        "should not be trusted before grant"
    );
    store
        .grant(engine.clone(), "dot".to_string())
        .expect("grant");
    assert!(store.is_trusted(&engine), "should be trusted after grant");
    let _ = fs::remove_file(engine);
}

#[cfg(unix)]
#[test]
fn trust_store_revoke_removes_trust() {
    let engine = temp_exec("revoke", "#!/bin/sh\necho ok\n");
    let store = TrustStore::ephemeral();
    store
        .grant(engine.clone(), "dot".to_string())
        .expect("grant");
    assert!(store.is_trusted(&engine));
    store.revoke(&engine).expect("revoke");
    assert!(
        !store.is_trusted(&engine),
        "should not be trusted after revoke"
    );
    let _ = fs::remove_file(engine);
}

// ── binary-swap eviction ──────────────────────────────────────────────────────

#[cfg(unix)]
#[test]
fn trust_store_auto_evicts_when_binary_size_changes() {
    let engine = temp_exec("swap", "#!/bin/sh\necho v1\n");
    let store = TrustStore::ephemeral();
    store
        .grant(engine.clone(), "dot".to_string())
        .expect("grant v1");
    assert!(store.is_trusted(&engine), "trusted before swap");

    // Rewrite with different byte-size → fingerprint mismatch.
    fs::write(
        &engine,
        "#!/bin/sh\necho v2-larger-content-to-change-size\n",
    )
    .expect("rewrite engine");
    let mut perms = fs::metadata(&engine).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&engine, perms).expect("chmod");

    assert!(
        !store.is_trusted(&engine),
        "trust should be auto-evicted after binary change"
    );
    // Entry was evicted — list should be empty.
    assert!(
        store.list().is_empty(),
        "evicted entry should be gone from list"
    );

    let _ = fs::remove_file(engine);
}

// ── persistence round-trip ────────────────────────────────────────────────────

#[cfg(unix)]
#[test]
fn trust_store_persists_and_reloads() {
    let engine = temp_exec("persist", "#!/bin/sh\necho ok\n");
    let store_path = std::env::temp_dir().join(format!(
        "neditor-trust-persist-{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    {
        let store = TrustStore::load_from(store_path.clone());
        store
            .grant(engine.clone(), "dot".to_string())
            .expect("grant");
        assert!(store.is_trusted(&engine));
    }

    // Load a fresh instance from the same path — entry should survive.
    {
        let store2 = TrustStore::load_from(store_path.clone());
        assert!(
            store2.is_trusted(&engine),
            "entry should be present after reload"
        );
    }

    let _ = fs::remove_file(engine);
    let _ = fs::remove_file(store_path);
}

// ── path canonicalization consistency ────────────────────────────────────────

#[cfg(unix)]
#[test]
fn trust_store_canonicalizes_symlinks() {
    let engine = temp_exec("canonical-target", "#!/bin/sh\necho ok\n");
    let link = std::env::temp_dir().join(format!(
        "neditor-trust-link-{}.sh",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::os::unix::fs::symlink(&engine, &link).expect("create symlink");

    let store = TrustStore::ephemeral();
    // Grant via the symlink path.
    store
        .grant(link.clone(), "dot".to_string())
        .expect("grant via symlink");

    // Should be trusted whether accessed via symlink or real path.
    assert!(store.is_trusted(&link), "trusted via symlink");
    assert!(store.is_trusted(&engine), "trusted via real path");

    let _ = fs::remove_file(link);
    let _ = fs::remove_file(engine);
}

// ── list ──────────────────────────────────────────────────────────────────────

#[cfg(unix)]
#[test]
fn trust_store_list_reports_validity() {
    let engine = temp_exec("list", "#!/bin/sh\necho ok\n");
    let store = TrustStore::ephemeral();
    store
        .grant(engine.clone(), "dot".to_string())
        .expect("grant");

    let entries = store.list();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert!(entry.valid, "entry should be valid while binary unchanged");
    assert_eq!(entry.granter, "dot");

    // Overwrite binary → list should show it as invalid.
    fs::write(&engine, "#!/bin/sh\necho changed\n").expect("overwrite");
    let mut perms = fs::metadata(&engine).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&engine, perms).expect("chmod");

    let stale_entries = store.list();
    assert_eq!(stale_entries.len(), 1);
    assert!(
        !stale_entries[0].valid,
        "stale entry should be marked invalid"
    );

    let _ = fs::remove_file(engine);
}
