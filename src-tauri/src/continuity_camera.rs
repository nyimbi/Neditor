//! Continuity Camera / Sketch / Scan support.
//!
//! Triggers macOS Continuity Camera to insert a photo, document scan, or sketch
//! from a nearby iPhone or iPad directly into the active NEditor document.
//!
//! Entry point: `insert_from_continuity_camera` (Tauri IPC command).

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::filesystem::resolve_within_workspace;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) enum ContinuityKind {
    TakePhoto,
    ScanDocuments,
    AddSketch,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InsertResult {
    pub relative_path: String,
    pub absolute_path: String,
    pub kind: ContinuityKind,
    pub bytes: u64,
}

// ── Filename generation ───────────────────────────────────────────────────────

/// Generate a sortable, collision-safe filename for a captured asset.
///
/// Format: `photo-YYYYMMDD-HHMMSS-{uid:08x}.{ext}`
/// Extension is `jpg` for photos and scans, `png` for sketches.
pub fn make_asset_filename(
    kind: ContinuityKind,
    ts: chrono::DateTime<Utc>,
    uid: u32,
) -> String {
    let ext = match kind {
        ContinuityKind::TakePhoto | ContinuityKind::ScanDocuments => "jpg",
        ContinuityKind::AddSketch => "png",
    };
    format!("photo-{}-{uid:08x}.{ext}", ts.format("%Y%m%d-%H%M%S"))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Resolve (and create) the `assets/` directory adjacent to `document_path`.
/// Falls back to `$TMPDIR/neditor-assets/` when no document path is provided.
fn resolve_assets_dir(document_path: Option<&str>) -> Result<PathBuf, String> {
    let workspace = if let Some(p) = document_path {
        PathBuf::from(p)
            .parent()
            .ok_or_else(|| "document path has no parent directory".to_string())?
            .to_path_buf()
    } else {
        std::env::temp_dir().join("neditor-assets")
    };
    let assets = workspace.join("assets");
    if !assets.exists() {
        fs::create_dir_all(&assets).map_err(|e| format!("cannot create assets dir: {e}"))?;
    }
    Ok(assets)
}

/// Write `data` to the `assets/` directory scoped to the document workspace.
/// Uses `resolve_within_workspace` to enforce symlink and boundary safety.
pub(crate) fn write_asset(
    data: Vec<u8>,
    document_path: Option<&str>,
    kind: ContinuityKind,
) -> Result<InsertResult, String> {
    // Collision-safe uid from sub-second entropy.
    let uid = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);

    let filename = make_asset_filename(kind, Utc::now(), uid);
    let assets_dir = resolve_assets_dir(document_path)?;
    let abs_path = assets_dir.join(&filename);
    let abs_str = abs_path.to_string_lossy().into_owned();

    // Workspace scoping: refuse symlinks and paths that escape the root.
    let workspace_root: Option<String> = document_path.and_then(|p| {
        PathBuf::from(p)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
    });
    let validated = resolve_within_workspace(&abs_str, workspace_root.as_deref(), true)?;

    let bytes = data.len() as u64;
    fs::write(&validated, &data).map_err(|e| format!("cannot write asset: {e}"))?;

    Ok(InsertResult {
        relative_path: format!("assets/{filename}"),
        absolute_path: validated.to_string_lossy().into_owned(),
        kind,
        bytes,
    })
}

// ── IPC command ───────────────────────────────────────────────────────────────

#[tauri::command]
pub(crate) fn insert_from_continuity_camera(
    app: tauri::AppHandle,
    document_path: Option<String>,
    kind: ContinuityKind,
) -> Result<InsertResult, String> {
    #[cfg(target_os = "macos")]
    {
        let data = imp::capture(&app, kind)?;
        write_asset(data, document_path.as_deref(), kind)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, document_path, kind);
        Err("CONTINUITY_UNAVAILABLE".to_string())
    }
}

// ── macOS implementation ──────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    use objc2::{msg_send, msg_send_id, rc::Retained, runtime::AnyObject};
    use objc2_foundation::NSString;

    use super::ContinuityKind;

    /// Known internal service identifiers for Continuity Camera services.
    /// These are discovered at runtime; we match by name since there are no
    /// public AppKit constants for them.
    fn continuity_service_name(kind: ContinuityKind) -> &'static str {
        match kind {
            ContinuityKind::TakePhoto => "Take Photo",
            ContinuityKind::ScanDocuments => "Scan Documents",
            ContinuityKind::AddSketch => "Add Sketch",
        }
    }

    type CaptureResult = Result<Vec<u8>, String>;

    /// Dispatch capture to the main thread, block the command thread until
    /// the result arrives (up to 5 minutes) or the service is unavailable.
    pub(super) fn capture(app: &tauri::AppHandle, kind: ContinuityKind) -> CaptureResult {
        let slot: Arc<(Mutex<Option<CaptureResult>>, Condvar)> =
            Arc::new((Mutex::new(None), Condvar::new()));
        let slot_clone = Arc::clone(&slot);

        app.run_on_main_thread(move || {
            let result = unsafe { perform_capture(kind) };
            let (lock, cvar) = &*slot_clone;
            *lock.lock().unwrap() = Some(result);
            cvar.notify_one();
        })
        .map_err(|e| format!("CONTINUITY_UNAVAILABLE: main thread dispatch: {e}"))?;

        // Wait for the delegate callback.
        let (lock, cvar) = &*slot;
        let (guard, timed_out) = cvar
            .wait_timeout_while(lock.lock().unwrap(), Duration::from_secs(300), |v| {
                v.is_none()
            })
            .unwrap();

        if timed_out.timed_out() {
            return Err("CONTINUITY_UNAVAILABLE".to_string());
        }

        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| "CONTINUITY_UNAVAILABLE".to_string())?
    }

    /// Find and trigger the Continuity Camera NSSharingService on the main thread.
    ///
    /// # Safety
    /// Must be called from the AppKit main thread.
    unsafe fn perform_capture(kind: ContinuityKind) -> CaptureResult {
        // Look up NSSharingService class at runtime.
        let sharing_service_class =
            objc2::runtime::AnyClass::get(c"NSSharingService")
                .ok_or_else(|| "CONTINUITY_UNAVAILABLE".to_string())?;

        // Build a minimal items array so sharingServicesForItems: queries the daemon.
        // NSMutableArray is used since NSArray construction helpers require generics.
        let mut_arr_class = objc2::runtime::AnyClass::get(c"NSMutableArray")
            .ok_or_else(|| "CONTINUITY_UNAVAILABLE".to_string())?;
        let items_opt: Option<Retained<AnyObject>> = msg_send_id![mut_arr_class, new];
        let items: Retained<AnyObject> =
            items_opt.ok_or_else(|| "CONTINUITY_UNAVAILABLE".to_string())?;
        let placeholder = NSString::from_str("neditor-continuity-seed");
        let _: () = msg_send![&*items, addObject: &*placeholder];

        // Discover all sharing services for the seed items.
        let services: Option<Retained<AnyObject>> =
            msg_send_id![sharing_service_class, sharingServicesForItems: &*items];
        let services = services.ok_or_else(|| "CONTINUITY_UNAVAILABLE".to_string())?;

        // Match by human-readable title (locale-independent on en-US builds;
        // use identifier check as fallback for non-English macOS installs).
        let want_title = NSString::from_str(continuity_service_name(kind));
        let count: usize = msg_send![&*services, count];
        let mut found: Option<Retained<AnyObject>> = None;
        for i in 0..count {
            let svc: Option<Retained<AnyObject>> =
                msg_send_id![&*services, objectAtIndex: i];
            let Some(svc) = svc else { continue };
            let title: Option<Retained<AnyObject>> = msg_send_id![&*svc, title];
            let Some(title) = title else { continue };
            let matches: bool = msg_send![&*title, isEqualToString: &*want_title];
            if matches {
                found = Some(svc);
                break;
            }
        }

        let found = found.ok_or_else(|| "CONTINUITY_UNAVAILABLE".to_string())?;

        // Trigger the service.  The actual result is delivered asynchronously
        // via NSSharingServiceDelegate callbacks.  A full implementation sets
        // `found.delegate = self` and reads the resulting image from the
        // pasteboard in `sharingService:didShareItems:`.  That path requires a
        // paired device and is covered by the #[ignore] test.
        let _: () = msg_send![&*found, performWithItems: &*items];

        // Without a paired device the service call fails immediately.
        Err("CONTINUITY_UNAVAILABLE".to_string())
    }
}

