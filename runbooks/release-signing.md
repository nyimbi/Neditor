# Release Signing Runbook

Use this runbook only on a credentialed release host.

## Preconditions

- The source tree is clean.
- Signing credentials are available only on the release host.
- The release artifact is built from the exact source commit under review.
- No private key, certificate password, token, or notarization credential will
  be copied into evidence.

## Steps

1. Build the platform release artifact.
2. Sign and notarize or attest the artifact using the platform release process.
3. Hash the final distributable artifact.
4. Run platform verification commands such as code-signature verification,
   notarization staple checks, installer verification, or package manager audit
   as applicable.
5. Write validator-shaped signing evidence with:

```sh
pnpm run collect:release-signing -- --platform <platform> --artifact <path> --proof <path>
```

6. Validate with:

```sh
pnpm run check:release-signing
```

7. Ingest the returned evidence and rerun release readiness.

## Return Paths

- `.tmp/release-signing/external/darwin/signing-evidence.json`
- `.tmp/release-signing/external/win32/signing-evidence.json`
- `.tmp/release-signing/external/linux/signing-evidence.json`
