use std::collections::HashSet;

/// All `menu_item(app, "neditor-...", ...)` IDs in `build_neditor_menu` must be unique.
/// Duplicates cause silent menu-event routing bugs.
#[test]
fn native_menu_item_ids_are_unique() {
    let source = include_str!("../lib.rs");
    let ids = collect_neditor_menu_ids(source);
    assert!(
        !ids.is_empty(),
        "should find neditor-* menu item ids in lib.rs"
    );
    let unique: HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(
        ids.len(),
        unique.len(),
        "duplicate native menu item ids detected: {:?}",
        {
            let mut seen = HashSet::new();
            ids.iter()
                .filter(|id| !seen.insert(*id))
                .copied()
                .collect::<Vec<_>>()
        }
    );
}

/// Every ID collected from `build_neditor_menu` must start with `neditor-`
/// so that the `on_menu_event` handler forwards it as a `neditor-menu-command` event.
#[test]
fn native_menu_item_ids_use_neditor_prefix() {
    let source = include_str!("../lib.rs");
    let ids = collect_neditor_menu_ids(source);
    for id in ids {
        assert!(
            id.starts_with("neditor-"),
            "menu item id '{id}' must start with 'neditor-'"
        );
    }
}

/// The native menu builder is guarded by `#[cfg(target_os = "macos")]` so it
/// is not installed on Windows and Linux (where the in-app menu bar is used).
#[test]
fn native_menu_is_macos_only() {
    let source = include_str!("../lib.rs");
    assert!(
        source.contains("#[cfg(target_os = \"macos\")]"),
        "lib.rs must contain a #[cfg(target_os = \"macos\")] guard for the native menu builder"
    );
    // Verify the guard appears before the .menu( call
    let guard_pos = source
        .find("#[cfg(target_os = \"macos\")]")
        .expect("guard present");
    let menu_call_pos = source
        .find(".menu(build_neditor_menu)")
        .expect(".menu( call present");
    assert!(
        guard_pos < menu_call_pos,
        "the #[cfg(target_os = \"macos\")] guard must appear before .menu(build_neditor_menu)"
    );
}

/// Collect all string literals used as the `id` argument to `menu_item(app, "<id>", ...)`.
fn collect_neditor_menu_ids(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            // Match: .item(&menu_item(app, "neditor-...",
            if trimmed.starts_with(".item(&menu_item(app, \"") {
                let after = trimmed.strip_prefix(".item(&menu_item(app, \"")?;
                let id = after.split('"').next()?;
                Some(id)
            } else {
                None
            }
        })
        .collect()
}
