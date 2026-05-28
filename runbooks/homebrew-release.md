# Homebrew Release Runbook

Use this runbook after a signed and notarized macOS artifact exists.

## Steps

1. Build the macOS release artifact from a clean checkout.
2. Sign and notarize the artifact.
3. Compute the artifact SHA-256.
4. Copy `packaging/homebrew/Casks/neditor.rb.template` into the Homebrew tap as
   `Casks/neditor.rb`.
5. Replace `__VERSION__` with the release version.
6. Replace `__SHA256__` with the signed artifact hash.
7. Audit and install locally:

```sh
brew audit --cask --new Casks/neditor.rb
brew install --cask Casks/neditor.rb
ned --help
brew uninstall --cask neditor
```

8. Return the completed cask and signed artifact evidence.
9. Validate with:

```sh
pnpm run check:homebrew
pnpm run check:release-signing
```

## Rules

- Do not close Homebrew release proof from the template cask.
- Do not use placeholder SHA values.
- Do not publish before signing and notarization evidence is accepted.
