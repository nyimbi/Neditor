# External Transform Setup

NEditor can render several fenced-code transforms with native Rust fallbacks. For higher fidelity, these engines can be configured as external executables:

- Graphviz / DOT (`dot`, `graphviz`, `circo`, `neato`, `fdp`, `osage`, `twopi`)
- D2
- PlantUML
- Pikchr
- SQLite (`sqlite3`) for read-only SQL table transforms

External engines are never trusted by default. Configure the executable path in Settings, enable trust for the specific engine, and keep the timeout/input-mode defaults unless the document requires otherwise.
You can also disable a configured external engine per transform. Disabled
engines are not executed, do not produce trust warnings, and fall back to the
embedded renderer when one exists.

Current platform evidence is tracked in
[External transform platform evidence](external-transform-platform-evidence.md).

## Safety Model

- Engine paths must point to real executable files.
- Each engine has an adapter profile for arguments, stdin/file mode, output handling, and diagnostics.
- Trust is per engine.
- Disabled engines are skipped before trust/path execution checks.
- Execution is bounded by timeout and output-size limits.
- Cache keys include the transform name, source hash, engine path, engine file
  size and modified time, adapter arguments, input mode, and renderer version.
- Failed external execution falls back to native rendering when a native fallback exists.

## macOS

Recommended package installs:

```sh
brew install graphviz d2 pikchr sqlite
brew install --cask temurin
brew install plantuml
```

Typical paths:

```text
/opt/homebrew/bin/dot
/opt/homebrew/bin/circo
/opt/homebrew/bin/neato
/opt/homebrew/bin/fdp
/opt/homebrew/bin/osage
/opt/homebrew/bin/twopi
/opt/homebrew/bin/d2
/opt/homebrew/bin/pikchr
/opt/homebrew/bin/plantuml
/opt/homebrew/bin/sqlite3
```

PlantUML requires a working Java runtime. Keep PlantUML in file mode unless stdin mode has been tested for the installed version.

## Linux

Debian/Ubuntu packages:

```sh
sudo apt-get update
sudo apt-get install -y graphviz default-jre plantuml sqlite3
```

D2 and Pikchr may need vendor packages or release binaries:

```sh
curl -fsSL https://d2lang.com/install.sh | sh -s --
cargo install pikchr-cli --locked
```

After installing release binaries, place them under a user-owned bin directory such as:

```text
~/.local/bin/d2
~/.local/bin/pikchr
~/.cargo/bin/pikchr-cli
```

Then configure those exact paths in NEditor.

## Windows

Recommended installs:

```powershell
winget install Graphviz.Graphviz
winget install Terrastruct.D2
winget install EclipseAdoptium.Temurin.21.JRE
winget install PlantUML.PlantUML
winget install SQLite.SQLite
winget install Rustlang.Rustup
cargo install pikchr-cli --locked
```

Typical paths vary by installer. Prefer explicit executable paths such as:

```text
C:\Program Files\Graphviz\bin\dot.exe
C:\Program Files\Graphviz\bin\circo.exe
C:\Program Files\Graphviz\bin\neato.exe
C:\Program Files\Graphviz\bin\fdp.exe
C:\Program Files\Graphviz\bin\osage.exe
C:\Program Files\Graphviz\bin\twopi.exe
C:\Users\<you>\AppData\Local\Microsoft\WinGet\Packages\...\d2.exe
C:\Program Files\PlantUML\plantuml.exe
C:\Users\<you>\AppData\Local\Microsoft\WinGet\Packages\...\sqlite3.exe
C:\Users\<you>\.cargo\bin\pikchr-cli.exe
```

If an engine is installed through a package manager shim, verify that the shim works from a normal terminal before trusting it in NEditor. For Pikchr proof through Cargo, run the probe with the explicit executable path:

```powershell
$env:NEDITOR_TEST_PIKCHR="C:\Users\<you>\.cargo\bin\pikchr-cli.exe"
pnpm run check:engines
```

## Engine Defaults

| Engine | Input Mode | Expected Output | Notes |
| --- | --- | --- | --- |
| Graphviz / DOT | stdin | SVG stdout | `dot` and `graphviz` fences use `dot -Tsvg` semantics. `circo`, `neato`, `fdp`, `osage`, and `twopi` are separate Graphviz engine entries with their own executable paths and the same no-shell `-Tsvg` adapter. |
| D2 | stdin | SVG stdout | Uses SVG export profile. |
| Pikchr | stdin | SVG stdout | Native fallback covers simple semicolon- or line-separated `box`, `circle`/`ellipse`, `diamond`, `cylinder`, `file`, and `arrow` statements with connector labels. `pikchr-cli` executables receive a temporary `.pikchr` source file path as their positional argument. |
| PlantUML | file | SVG or PNG sidecar | File mode avoids version-specific stdin behavior. Use `format=png`, `output=png`, or the `png` flag on a `plantuml` fence when a PNG artifact is required; otherwise NEditor requests SVG. |
| SQLite SQL | file | CSV stdout converted to Markdown table | `sql` fences run through trusted `sqlite3` only. NEditor passes the database path and query without a shell, accepts read-only `SELECT` or `WITH` statements, rejects multi-statement or mutating SQL, and renders the result as a Markdown table. |

## Troubleshooting

- In the app, open Help and search for "external transform troubleshooting" for
  the guided UI workflow that routes to Engine settings, Diagnostics, Templates,
  and Export readiness.
- If an engine reports permission errors, verify the file is executable and not quarantined. On macOS, only remove quarantine from a trusted installed tool. On Linux, confirm the executable bit and package path. On Windows, prefer the full `.exe` path over package-manager shims when diagnostics are ambiguous.
- If output is empty, increase diagnostics by running the same executable manually with a tiny sample.
- If execution times out, reduce diagram complexity before increasing the timeout. Increase timeouts only for trusted engines and rerun Probe after changing timeout or path settings.
- If trust is disabled, NEditor will not execute the engine and will use native fallback behavior where available.
- If cache output appears stale, change the source or engine path. NEditor also
  invalidates cached external output when the executable at the trusted path
  changes size or modified time.
- If PlantUML behaves differently from other engines, keep it in file mode;
  Graphviz and D2 normally use stdin, while Pikchr can use native fallback for
  simple business diagrams.
- If a SQL transform does not run, confirm the `sqlite3` executable path, the
  database file path, and whether the query starts with `SELECT` or `WITH`.
  NEditor is a document table workflow, not a database administration console,
  so inserts, updates, deletes, schema changes, and multi-statement batches are
  rejected by design.
