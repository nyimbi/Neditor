# Security Audit — External Transform Surface

Scope: `src-tauri/src/transforms/external.rs`, `src-tauri/src/transforms/sql.rs`, `src-tauri/src/transforms/options.rs`, `src-tauri/src/transform_install.rs`. Cross-checked against `docs/security-threat-model.md` and `docs/external-transforms.md`. Audit-only; no code changed.

Risk level: **HIGH** (one High, one High, two Medium, several Low/Info)

Summary
- Critical: 0
- High: 2
- Medium: 3
- Low / Info: 5

---

## Findings — `transforms/external.rs`

### F1. Child process not killed on timeout — resource exhaustion / DoS (HIGH)
Location: `src-tauri/src/transforms/external.rs:640-698`.

The worker thread owns the `Child` handle (`move ||` at line 661). When `rx.recv_timeout(timeout)` fires (line 675), we return the timeout error to the caller but never call `child.kill()`. The spawned Graphviz/D2/PlantUML/Pikchr process keeps running until it exits on its own or the OS kills it. `wait_with_output` in the worker also blocks the worker thread indefinitely. Since PlantUML can consume unbounded memory on hostile input, and Graphviz layout can be super-linear, an attacker (or a naive user pasting a bad diagram) can accumulate zombie processes with each render, exhausting memory and file descriptors.

Contrast with `transforms/sql.rs:138` which correctly calls `child.kill()` on timeout.

Exploit: A document containing a pathologically bad DOT/PUML block, re-rendered on every keystroke, spawns a new long-running `dot`/`java` process per render. Memory pressure grows without bound.

Fix: Wrap the child in `Arc<Mutex<Child>>` shared between the worker and main thread, or spawn via `command.spawn()?` and hold a killable handle on the main thread; on timeout, call `child.kill()` then `child.wait()`. Alternatively use `wait_timeout` crate or the ChildTerminator pattern.

```rust
// GOOD (sketch)
let child = command.spawn()?;
let child_id = child.id();
let handle = std::sync::Arc::new(std::sync::Mutex::new(Some(child)));
let handle_worker = handle.clone();
std::thread::spawn(move || {
    let mut guard = handle_worker.lock().unwrap();
    if let Some(mut c) = guard.take() {
        // write stdin, then wait_with_output
        let _ = tx.send(c.wait_with_output().map_err(|e| e.to_string()));
    }
});
match rx.recv_timeout(timeout) {
    Ok(r) => r,
    Err(_) => {
        if let Some(mut c) = handle.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        Err("timeout".into())
    }
}
```

---

### F2. Disk cache in world-writable `/tmp` — cache poisoning → HTML injection (HIGH)
Location: `src-tauri/src/transforms/external.rs:506-508` (`external_transform_disk_cache_root`) and `:395-411` (`cached_disk_external_transform`).

`std::env::temp_dir().join("neditor-transform-cache-v1")` on Linux resolves to `/tmp/neditor-transform-cache-v1`. `fs::create_dir_all` succeeds if the directory already exists — a local attacker on a multi-user host can pre-create the directory with permissive ownership before NEditor starts, then drop crafted `<cache_key>.json` files.

The cached JSON is deserialized directly into `TransformArtifact`; its `html` field is later injected verbatim into the DOM (see `execute_external_transform` line 796). Because `cache_key = sha256(name, engine_path, engine_file_size, engine_mtime, adapter_args, input_mode, source_hash)`, an attacker who can predict *any* diagram source a user will render (all inputs are recoverable from public docs, examples, or shell history) can plant a matching cache file and cause NEditor to render arbitrary HTML/SVG instead of the real Graphviz/PlantUML output.

Mitigation by CSP: `tauri.conf.json` sets `script-src 'self'` with no `'unsafe-inline'`, so injected `<script>` will not execute. However attacker can still: (a) inject deceptive SVG (phishing content presented as legitimate diagram output), (b) render off-brand or defamatory imagery, (c) exfiltrate via `<a target=_blank href="https://evil">` clicks, (d) mislead the manifest hash chain by making output_hash reflect attacker content that then gets stored in export manifests as "verified" evidence — corrupting the audit trail the threat model relies on.

The threat model in `docs/security-threat-model.md:59` claims "Cache keys include source, engine path, input mode, and adapter behavior" as if that were sufficient; it is not, because the cache *location* is untrusted.

Fix: place the cache under a user-scoped, mode-0700 directory: Tauri's `app_local_data_dir` (or `dirs::cache_dir()`), not `env::temp_dir()`. If temp_dir must be used, resolve via `tempfile::Builder` and refuse to open if the directory is not owned by the current uid with mode 0700.

```rust
// GOOD
fn external_transform_disk_cache_root(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_cache_dir().unwrap().join("neditor-transform-cache-v1")
}
```

---

### F3. Symlink/TOCTOU on temp input & PlantUML sidecar output (MEDIUM)
Location: `src-tauri/src/transforms/external.rs:606-618` (input) and `:627-631` + `:709-717` (sidecar output).

Temp filenames embed `sha256(body)`, PID, and `SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()`. On Linux with a shared `/tmp`, an attacker that (a) can observe process creation timing and (b) knows or can guess the document body can pre-create a symlink at the predicted path. `fs::write` follows symlinks; PlantUML's sidecar write likewise follows the pre-planted symlink at `<input>.<svg|png>`, and `fs::read(path)` on line 710 reads whatever the symlink now points to (attacker-controlled or victim-readable).

Impact:
1. Write primitive: attacker chooses target of the write of `request.body` (limited to 1 MiB of DOT/D2/PlantUML source).
2. Read primitive: for the PlantUML sidecar branch, `fs::read(temp_output)` returns whatever bytes the symlink resolves to, which is then rendered into the doc — leaks arbitrary file contents readable by the NEditor uid.

Predictability is limited by nanosecond entropy and hash of body, but this is a defense-in-depth failure — Rust's own std docs warn against `fs::write` to shared temp locations.

Fix: use `tempfile::NamedTempFile::new_in(secure_dir)` with `O_EXCL`; or on Unix, `OpenOptions::new().write(true).create_new(true).custom_flags(libc::O_NOFOLLOW | libc::O_EXCL).open(...)`. Prefer a per-run subdirectory created with mode 0700 inside the app cache dir.

---

### F4. TOCTOU between engine-path validation and `spawn` (LOW)
Location: `src-tauri/src/transforms/external.rs:105-117`, spawn at `:640`.

Metadata (`is_file`, executable bit) is checked before `Command::new(engine_path).spawn()`. A local attacker with write access to a directory on the engine path can swap the binary between validation and spawn. Requires prior local write access, so severity is Low; still worth noting because the threat model touts these checks as a mitigation.

Fix: on Unix, `open()` the file with `O_PATH | O_CLOEXEC`, verify inode+mode via `fstat`, then `fexecve` (via `nix::unistd::fexecve` or `posix_spawn`). Or accept residual risk and document that the "regular executable file" check is best-effort.

---

### F5. Sidecar output path derived from attacker-influenced input path (LOW)
Location: `src-tauri/src/transforms/external.rs:630`.

`temp_output = input_path.with_extension(output_suffix)` computes a path in `/tmp` that PlantUML must write to. Combined with F3, a symlink at `neditor-plantuml-<hash>-<pid>-<nanos>.svg` in `/tmp` is a write-through-symlink target for PlantUML itself (whose own IO is not under our control). Fixed by the F3 remediation.

---

### F6. Trust check trusts the frontend `trusted: bool` verbatim (LOW / by-design)
Location: `src-tauri/src/transforms/external.rs:86-91`, request field `:30`.

`run_external_transform` reads `request.trusted` from the IPC caller. Any code executing inside the webview can pass `trusted: true`. The design places the trust gate in the Vue layer/settings. This is consistent with the documented model (`docs/external-transforms.md:11`) but means a webview XSS or a malicious script loaded via a rendered SVG becomes a code-execution primitive (through the attacker-controlled `engine_path`).

The current CSP (`script-src 'self'`, no `'unsafe-inline'`) mitigates webview XSS, but the coupling should be documented and asserted. Consider moving trust state to a backend-owned map keyed by canonicalized engine path so the frontend cannot claim trust it was not granted, and to make the audit trail authoritative.

---

### F7. Adapter arg lists lose defense against `.bat`/`.cmd` on Windows (LOW)
Location: `src-tauri/src/transforms/external.rs:633-634` on Windows targets.

Rust ≤1.77 (`CVE-2024-24576`, "BatBadBut") argument-escaping bug: when the child is a `.bat`/`.cmd`, `Command::args` used the naive quoting, allowing argument injection. Fixed in Rust 1.77.2 by escaping and by returning an error for particularly unsafe cases. Two implications:
- If the toolchain that built the shipped binary is <1.77.2, any user who trusts a `.bat` wrapper (Windows Graphviz sometimes ships with one) is exposed.
- Even after the fix, Rust's escaping is best-effort for `.bat`; safer to reject `.bat`/`.cmd` engine paths outright, or invoke via `cmd.exe /c` with our own quoting.

Fix: pin `rust-toolchain.toml` to ≥1.77.2 and, on Windows, reject engine paths whose extension is `.bat`/`.cmd` with an explicit error asking the user to point to the real `.exe`.

---

### F8. Diagram sources can trigger local-file reads via engine `include` directives (LOW / by-design)
Location: engine input is user document text (`request.body`) at `:616` / `:666`.

PlantUML honors `!include` and `!includeurl`; D2 supports imports; Graphviz's `libcgraph` respects `include` in some builds. Because the temp source file lives in `/tmp` (or stdin runs in NEditor's cwd), these engines will read files reachable by NEditor's uid. A document authored by an untrusted third party (imported RFP, template from the web, etc.) can therefore exfiltrate local files by embedding their contents into the rendered SVG.

The threat model implicitly treats documents as user-authored; if that assumption ever changes, this becomes MEDIUM. At minimum, document this so integrations that ingest third-party Markdown (`rfp_import.rs`, `pandoc_import.rs`) know to run transforms untrusted.

Fix (if desired): run engines in a working directory that contains only the temp source; for PlantUML pass `-security-profile` (or run via the safe wrapper); document that `!includeurl` remains network-capable and cannot be disabled without engine cooperation.

---

### F9. Diagnostic `related` line leaks full engine path into cache/manifest (INFO)
Location: `src-tauri/src/transforms/external.rs:812-853`.

`engine_path` is echoed into diagnostics that flow into the export manifest evidence. If the engine lives under `/home/<user>/…` or `C:\Users\<user>\…`, this leaks the username into shipped artifacts. Not a vulnerability per se; consider redacting the home-directory prefix in exported manifests.

---

## Findings — `transforms/sql.rs`

### F10. Absolute `database=` paths bypass the "document-local" restriction (HIGH)
Location: `src-tauri/src/transforms/sql.rs:83-97` combined with `src-tauri/src/transforms/options.rs:81-100`.

`document_relative_path_escapes` returns `false` when `path.is_absolute()` is true (options.rs:86-88). The `resolve_document_path` helper (options.rs:70-79) then passes the absolute path through untouched. Net effect: a document containing:

```markdown
```sql database="/home/victim/.mozilla/firefox/…/places.sqlite"
SELECT url, title FROM moz_places;
```
```

runs a SELECT against *any* SQLite database the NEditor uid can read (Firefox history, Signal `db.sqlite`, Slack cache, iMessage `chat.db` on macOS, Keychains stored as SQLite, browser cookies, wallet databases). The output CSV is then rendered into the document and can be exfiltrated with any subsequent export.

Meanwhile the threat model (`docs/security-threat-model.md:71-73`) and `docs/external-transforms.md:167` claim SQL blocks must "use a document-local `database` path that resolves inside the document folder". The code contradicts that promise for absolute paths.

Exploit: a shared/received `.md` file that opens as normal and previews as expected can silently read the local user's browser history or password-manager DB on first preview if the user has already trusted `sqlite3` (a one-time global toggle).

Fix — reject absolute paths outright when the document has a `document_dir`, or require them to canonicalize inside `document_dir`:

```rust
// GOOD
pub(crate) fn document_relative_path_escapes(&self, value: &str) -> bool {
    let path = PathBuf::from(value);
    let Some(document_dir) = &self.document_dir else { return false; };
    let candidate = if path.is_absolute() { path } else { document_dir.join(&path) };
    match (document_dir.canonicalize(), candidate.canonicalize()) {
        (Ok(base), Ok(target)) => !target.starts_with(base),
        _ => true,
    }
}
```

If untrusted-DB-by-absolute-path is a legitimate feature, gate it behind a *separate* explicit per-document trust and remove the misleading "document-local" language from the threat model.

---

### F11. `read_only_select` normalization allows leading-comment bypass (MEDIUM)
Location: `src-tauri/src/transforms/sql.rs:201-215`.

`read_only_select` normalizes by:
1. trim BOM,
2. trim,
3. trim_end `;`,
4. trim_start,
5. `to_ascii_lowercase()`,
6. require `starts_with("select ")` or `"with "`.

But block comments and line comments are not stripped before the `starts_with` check. `SELECT /* comment */ 1` passes (starts_with "select"). However `/* */ SELECT 1` fails (starts with `/`), which is fine.

Then `contains_blocked_sql_keyword` runs `sql_without_quoted_segments` which strips quotes but **does not strip comments**. So a query like:

```sql
SELECT 1 /* insert */
```

would trip the false-positive block on "insert" inside a comment — annoying but safe. The concerning direction is the *other* way: can a mutation hide from the blocklist? SQLite treats a leading `SELECT` followed by `;` as one statement; `has_non_trailing_statement_separator` catches trailing `INSERT` after `;`. `WITH x AS (SELECT ...) INSERT INTO y ...` is a real SQLite construct — blocked by keyword search. `SELECT ... UNION SELECT * FROM (INSERT ...)` — SQLite parses `INSERT` as identifier here, so blocklist would false-positive but it would be blocked; safe direction.

Residual concern: the ASCII lowercase + word-boundary matcher operates on the raw query text *including comments and identifiers*. An identifier or string literal containing "insert" in code paths that hit the outer branch would already be masked by `sql_without_quoted_segments`; comments are not. This means legitimate queries with mutation words in `--` or `/* */` comments get rejected. Not a security bug, but a robustness bug worth noting.

Fix: strip SQL comments in `sql_without_quoted_segments` before the keyword scan; alternatively, pass `sqlite3 -bail -readonly` and rely on SQLite's read-only mode as the authoritative gate.

**Higher-value fix**: sqlite3 CLI supports `-readonly` since 3.22 (2018). Adding it (`.args(["-readonly", "-header", "-csv"])`) makes the whole keyword blocklist a belt-on-suspenders check instead of the primary defense. That single flag closes an entire category of "did the parser catch this?" bugs.

---

### F12. Query stdin write can block indefinitely if child stalls (LOW)
Location: `src-tauri/src/transforms/sql.rs:121-130`.

`stdin.write_all(query.as_bytes())` runs on the caller thread synchronously. If `sqlite3` is a hostile binary or a shim that blocks reading stdin, `write_all` blocks forever without contributing to the timeout that starts on the poll loop at line 131. Low severity because we require a user-selected trusted absolute executable path; but the pattern in `external.rs:661` (write on a worker thread with a channel deadline) is stricter and should be adopted here for consistency.

---

## Findings — `transform_install.rs`

### F13. `install_transform_handlers` spawns installer threads without cancellation (LOW)
Location: `src-tauri/src/transform_install.rs:83-118`.

The install thread runs `brew`/`winget`/`cargo install` sequentially and cannot be cancelled from the UI. Multiple concurrent invocations aren't serialized either — clicking "Install" twice runs two concurrent `brew install` sessions, which can corrupt Homebrew state. Program name and args are hard-coded static strings, so no injection risk.

Fix: guard with `OnceLock<Mutex<()>>` around the install thread; return a busy error if already running. Optionally expose a cancel command that kills the child.

### F14. Installer child inherits the parent's environment (LOW)
Location: `src-tauri/src/transform_install.rs:86-91`.

`Command::new(step.program)` inherits `PATH` and everything else. On Windows in particular, a poisoned `PATH` (e.g., a `brew.bat` in cwd shadow) or `PATHEXT`/`ComSpec` tampering can pivot execution. Not exploitable without local write access; document expected env or clear the env explicitly.

### F15. `brew`/`winget`/`cargo` are resolved via `PATH` — no absolute path (INFO)
Location: `transform_install.rs:86, 145, 310, 158`.

Program names `"brew"`, `"winget"`, `"cargo"` are looked up via `PATH`. If the user's `PATH` places an attacker-controlled directory first, NEditor will run that binary. This contradicts the strong "absolute path only" stance in `external.rs`. Consider using `which::which` and displaying/confirming the resolved path before spawning.

---

## Cross-check against `docs/security-threat-model.md`

| Claim | Reality | Notes |
| --- | --- | --- |
| "Execution is bounded by timeout and output-size limits." (§Malicious External Transform) | Timeout returns to caller but does **not** kill the child (F1). Output size limit only enforced *after* the process exits, so a rogue engine can still stream 4 GB before we notice. | Partial. |
| "Adapters construct fixed argument lists and do not interpolate shell strings." | Accurate — args are static; user input is body-only via stdin or a temp file. | OK. |
| "Cache keys include source, engine path, input mode, and adapter behavior." | Keys are correct, but the cache *location* (`/tmp/neditor-transform-cache-v1`) is untrusted on multi-user systems (F2). | Misleading. |
| "SQL blocks must use a document-local `database` path that resolves inside the document folder." | Contradicted by code: absolute paths bypass the check entirely (F10). | **False**. |
| "Only `SELECT` and `WITH` queries are accepted." | True at the surface parser; sqlite3 `-readonly` not passed (F11). | OK, but weak. |
| "The SQLite process is invoked directly without a shell and with a bounded timeout." | Accurate; timeout does kill the child (unlike F1). | OK. |
| "Engine paths must point to regular executable files; directories, project files, and shell command text are rejected before spawn." | True at check time; TOCTOU window before spawn (F4). | OK modulo caveat. |

---

## OK list — things checked and found clean

- **No shell interpolation** anywhere. `Command::new + args()` used throughout; no `sh -c`, no format-string command building.
- **No hardcoded API keys / passwords / tokens** in the transforms surface (`grep -Ei "api[_-]?key|secret|password|token" src-tauri/src/transforms/` finds only Serde field names).
- **PlantUML server URL injection**: N/A — PlantUML is invoked locally with `-pipe`/file mode; no URL argument path exists.
- **Argument arrays hard-coded per adapter** (`external.rs:234-296`); user-controllable inputs (`body`, `engine_path`) do not flow into `.arg(...)` positions where flag parsing would be attacker-influenced (except the temp file *path*, which is under our control and free of shell metacharacters).
- **Input size limit** (`MAX_TRANSFORM_INPUT_BYTES = 1 MiB`) enforced before spawn (`external.rs:123-130`).
- **Output size limit** enforced after read (`external.rs:726-739`) — bounds what makes it into the DOM.
- **`engine_path.is_absolute()`** required (`external.rs:100`); relative names / `$PATH` lookup rejected.
- **Executable bit** validated on Unix (`external.rs:349-361`).
- **SQL statement stacking** — `SELECT 1; DROP TABLE …` correctly rejected (`sql.rs:217-260`).
- **SQL keyword matcher** uses word boundaries; quoted identifiers/strings correctly ignored (`sql.rs:262-322`).
- **SQL stderr** goes through `escape_html` in `error_block` (`sql.rs:324-329`) → no HTML injection via error text.
- **Rendered output** is wrapped in a `<section>` with attacker-influenced `name` HTML-escaped (`external.rs:796-800`); `name` is also whitelisted to a fixed set (`external_transform_supported`).
- **CSP** (`tauri.conf.json:24`) blocks inline scripts and constrains connect-src; substantially blunts cache-poisoning impact (F2) to non-scripting content.
- **Trust gate** exists at both frontend and Rust boundaries (`external.rs:86-91`, `sql.rs:58-64`); it's just enforced weakly (F6).
- **Sidecar cleanup** — temp files removed in both success and failure paths (`external.rs:643-724`).
- **Installer program names** — plan lookup validates `plan_id` against a fixed list; args are static (`transform_install.rs:67-76, 139-164`).
- **`escape_html`** used consistently for name/alt text in the rendered `<section>`.

---

## Recommended remediation priority

1. **F10 (SQL absolute-path bypass)** — HIGH, ship in patch release; documented promise is broken and exploit is a plausible file.
2. **F1 (child not killed on timeout)** — HIGH, ship in patch release; DoS is trivial to trigger.
3. **F2 (disk cache in `/tmp`)** — HIGH on multi-user hosts, MEDIUM on single-user desktops; move to app cache dir.
4. **F11 (add sqlite3 `-readonly`)** — MEDIUM, defense-in-depth, one-line change.
5. **F3/F5 (temp-file symlink handling)** — MEDIUM; adopt `tempfile` crate.
6. **F6 (backend-owned trust)** — LOW, architectural; align with threat-model language.
7. **F4, F7, F8, F12–F15** — LOW / INFO, batch into a hardening pass.

