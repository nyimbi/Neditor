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
- The downloadable artifact contains `NEditor.app` and matches the cask URL.
- The release notes identify any remaining external evidence gaps explicitly.

The expected tap layout is:

```text
homebrew-neditor/
  Casks/
    neditor.rb
```

After producing a signed and notarized macOS zip or DMG, replace
`__VERSION__` and `__SHA256__` in the template, then run:

```sh
NEDITOR_HOMEBREW_CASK=/path/to/homebrew-neditor/Casks/neditor.rb pnpm run check:homebrew
```

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
