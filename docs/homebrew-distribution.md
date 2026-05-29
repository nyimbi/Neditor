# Homebrew Distribution

NEditor is a native macOS desktop application, so Homebrew distribution should
use a cask from a project-owned tap, not a formula. The repository keeps a cask
template at `packaging/homebrew/Casks/neditor.rb.template` and validates it with
`pnpm run check:homebrew`.

## Release Contract

Do not publish or update a Homebrew tap until all of these are true:

- `pnpm run check:homebrew` passes with no configuration issues.
- `pnpm run check:release-readiness` has no local failures.
- macOS release artifacts are signed and notarized.
- The Homebrew cask uses the final release version and a real SHA-256 checksum.
- The downloadable artifact contains `NEditor.app`, exposes the `ned` command
  line helper, and matches the cask URL.
- The release notes identify any remaining external evidence gaps explicitly.

The expected tap layout is:

```text
homebrew-neditor/
  Casks/
    neditor.rb
```

After producing a signed and notarized macOS zip or DMG, materialize the cask
from the template instead of editing the SHA by hand:

```sh
pnpm run release:homebrew -- \
  --artifact /path/to/NEditor-<version>-macos.zip \
  --output /path/to/homebrew-neditor/Casks/neditor.rb
```

`release:homebrew` verifies the release version against `package.json`,
computes the artifact SHA-256, replaces the template placeholders, copies the
artifact into `.tmp/homebrew/external/` for evidence-kit validation, and writes
`.tmp/homebrew/materialize-cask-report.json` with the exact follow-up commands.
Then run:

```sh
NEDITOR_HOMEBREW_CASK=/path/to/homebrew-neditor/Casks/neditor.rb \
NEDITOR_HOMEBREW_ARTIFACT=/path/to/NEditor-<version>-macos.zip \
pnpm run check:homebrew
```

The cask installs `NEditor.app` and links `ned` so business users and automation
can open Markdown files or convert documents from Terminal without hunting for
the app bundle path.

`ned` can also print shell completion scripts without installing another
package. After installing the cask, users can run the command that matches their
shell, review the output, and place it in the appropriate shell completion
directory:

```sh
ned completions bash
ned completions zsh
ned completions fish
```

For release evidence kit returns, place the completed cask and signed artifact
under a return directory as:

```text
homebrew/
  neditor.rb
  NEditor-<version>-macos.zip
```

The release host can import those files with `pnpm run ingest:evidence -- --source
/path/to/return-dir`. Ingest stores them under `.tmp/homebrew/external/`, where
`pnpm run check:homebrew` auto-detects them if `NEDITOR_HOMEBREW_CASK` and
`NEDITOR_HOMEBREW_ARTIFACT` are not set.

If Homebrew is installed on the release host, also run:

```sh
brew audit --cask --new /path/to/homebrew-neditor/Casks/neditor.rb
brew install --cask /path/to/homebrew-neditor/Casks/neditor.rb
brew uninstall --cask neditor
```

## Quality Bar

Homebrew should be treated as a public distribution channel. A cask that only
points at an unsigned local build is not release quality. The validator reports
that state as a release blocker rather than silently accepting it.
