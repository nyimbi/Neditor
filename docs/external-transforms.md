# External Transform Setup

NEditor can render several fenced-code transforms with native Rust fallbacks. For higher fidelity, these engines can be configured as external executables:

- Graphviz / DOT
- D2
- PlantUML
- Pikchr

External engines are never trusted by default. Configure the executable path in Settings, enable trust for the specific engine, and keep the timeout/input-mode defaults unless the document requires otherwise.

## Safety Model

- Engine paths must point to real executable files.
- Each engine has an adapter profile for arguments, stdin/file mode, output handling, and diagnostics.
- Trust is per engine.
- Execution is bounded by timeout and output-size limits.
- Cache keys include the transform name, source hash, engine path, engine file
  size and modified time, adapter arguments, input mode, and renderer version.
- Failed external execution falls back to native rendering when a native fallback exists.

## macOS

Recommended package installs:

```sh
brew install graphviz d2 pikchr
brew install --cask temurin
brew install plantuml
```

Typical paths:

```text
/opt/homebrew/bin/dot
/opt/homebrew/bin/d2
/opt/homebrew/bin/pikchr
/opt/homebrew/bin/plantuml
```

PlantUML requires a working Java runtime. Keep PlantUML in file mode unless stdin mode has been tested for the installed version.

## Linux

Debian/Ubuntu packages:

```sh
sudo apt-get update
sudo apt-get install -y graphviz default-jre plantuml
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
```

Typical paths vary by installer. Prefer explicit executable paths such as:

```text
C:\Program Files\Graphviz\bin\dot.exe
C:\Users\<you>\AppData\Local\Microsoft\WinGet\Packages\...\d2.exe
C:\Program Files\PlantUML\plantuml.exe
```

If an engine is installed through a package manager shim, verify that the shim works from a normal terminal before trusting it in NEditor.

## Engine Defaults

| Engine | Input Mode | Expected Output | Notes |
| --- | --- | --- | --- |
| Graphviz / DOT | stdin | SVG stdout | Uses `dot -Tsvg` semantics. |
| D2 | stdin | SVG stdout | Uses SVG export profile. |
| Pikchr | stdin | SVG stdout | Native fallback covers simple diagrams. `pikchr-cli` executables receive a temporary `.pikchr` source file path as their positional argument. |
| PlantUML | file | SVG sidecar | File mode avoids version-specific stdin behavior. |

## Troubleshooting

- If an engine reports permission errors, verify the file is executable and not quarantined.
- If output is empty, increase diagnostics by running the same executable manually with a tiny sample.
- If execution times out, reduce diagram complexity before increasing the timeout.
- If trust is disabled, NEditor will not execute the engine and will use native fallback behavior where available.
- If cache output appears stale, change the source or engine path. NEditor also
  invalidates cached external output when the executable at the trusted path
  changes size or modified time.
