# Rendered Export Review Runbook

Use this runbook when a work order requires human or native-viewer review of
HTML, PDF, DOCX, PPTX, Markdown bundle, blog/Substack, LaTeX, Google Docs, or
EPUB export artifacts.

## Generate Artifacts

Run:

```sh
pnpm run test:rendered-exports
```

The audit bundle is written under `.tmp/rendered-export-audit/`.

## Review Checklist

1. Open `.tmp/rendered-export-audit/manual-review.html`.
2. Review the primary export artifacts and each review case named in the page.
3. Inspect PDF output in a native PDF viewer when available.
4. Inspect DOCX and PPTX output in native Office-compatible viewers when
   available.
5. Inspect EPUB package structure and reader rendering when the assigned work
   order includes EPUB evidence.
6. Confirm headers, footers, page numbers, cover metadata, watermarking,
   branding, captions, references, tables, formulas, citations, generated
   sections, and export manifests as applicable to the work order.
7. Fill a copy of
   `.tmp/rendered-export-audit/visual-review-signoff.template.json`.
8. Validate it with:

```sh
NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/signoff.json pnpm run test:rendered-exports -- --validate-signoff-only
```

## Evidence To Return

- Completed visual-review sign-off JSON.
- Any screenshots or native-viewer notes named by the sign-off.
- The refreshed `.tmp/rendered-export-audit/visual-review-summary.json`.

## Rules

- Do not claim native-viewer proof from package inspection alone.
- Do not use stale artifacts from a different Git commit.
- Do not include private documents in screenshots.
