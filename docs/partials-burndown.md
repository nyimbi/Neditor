# NEditor Partials Burn-Down List

Generated: 2026-08-07  
Source: docs/spec-completion-matrix.md (56 Partial rows; 9.19 Pikchr flipped Complete by `17fdd3c`; evidence narrowed by sweep commits `c64f36d`, `a907f13`, `1c71784`, `d5270a3`, `135bf07`)

Blocking-category counts: Manual QA × 36, Cross-platform CI × 8, External engine × 6, AI runtime × 2, Signing × 1, Live service × 1, Performance × 1, Security review × 1

---

## Core Product Scope

### 2 Source Prompt Extension — Split, preview-only, focus/source modes

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Launch `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke`, open each of the 8 mode controls in the running Tauri window, and record manual visual confirmation in `.tmp/desktop-smoke/native-workflow-report.json`.

---

### 2 Source Prompt Extension — Light/dark/system theme and typography

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: With the Tauri app running, toggle Light/Dark/System in Settings and verify rendered CSS variables match the spec palette by inspecting `.tmp/desktop-smoke/native-workflow-report.json` computed evidence already written there.

---

### 2 Source Prompt Extension — Cross-platform packaging

**Blocking category**: Signing  
**Estimated effort**: >1wk  
**Concrete next step**: Run `pnpm run collect:release-signing` on a credentialed macOS release host with an Apple Developer certificate to produce `signing-evidence.json`, then re-run `pnpm run check:release-signing` to accept it.

---

## Carry-Forward MacDown Improvements

### 5.1 External File Refresh — Watch open files

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: `c64f36d` proves save-as atomic root relocation in 4 new tests (macOS). Remaining: run the full native workflow bundle on a Windows or Linux host via `scripts/run-tauri-webdriver.mjs` and verify multi-include watcher edge cases appear in `.tmp/desktop-webdriver/report.json`.

---

### 5.1 External File Refresh — Non-destructive conflicts: compare, accept external, keep local, save copy

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Exercise the native file-save dialog path for "Save Copy" in the Tauri app conflict modal and record the copy file path in a manual QA sign-off appended to `.tmp/desktop-smoke/native-workflow-report.json`.

---

### 5.3 Business Export Customization — HTML/PDF/DOCX/PPTX exports plus blog/Substack/LaTeX/Google Docs/EPUB handoffs

**Blocking category**: Live service  
**Estimated effort**: ≤1d  
**Concrete next step**: Refresh the Google Drive OAuth token, re-run `pnpm run check:google-docs-import`, supply the import evidence file it templates, then re-run the check to accept current-source Google Docs readback proof.

---

### 5.3 Business Export Customization — Page numbering, headers/footers, logo, brand color, cover, watermark, presets, metadata

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run test:rendered-exports` and open `.tmp/rendered-export-audit/review-cases/brand-layout/` to manually verify cover-logo, brand color, header/footer, and watermark across the generated HTML, PDF, DOCX, and PPTX artifacts.

---

### 5.5 Table Of Contents — `[TOC]`, front matter TOC, depth, numbering, export formatting

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: Run the WebDriver native workflow on a Windows or Linux host to produce `tocEvidence` in `.tmp/desktop-webdriver/report.json`, proving native TOC rendering across platforms.

---

## Core Application Requirements

### 6.2 Primary Layout — Toolbar, status bar, sidebar, editor, preview

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Resize the running Tauri window to 1280×800, 1440×900, and 1920×1080, confirm no toolbar or status-bar overflow, and append a one-line window-size sign-off to `.tmp/desktop-smoke/native-ui-report.json`.

---

### 6.3 Editor — Markdown highlighting, diagnostics gutter, decorations, folding

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` on a Linux host and confirm `foldGutterEvidence` and `lintGutterEvidence` keys appear in `.tmp/desktop-smoke/native-workflow-report.json`.

---

### 6.3 Editor — Line numbers, word wrap, spellcheck, find/replace, word count

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Open a 500-word fixture in the Tauri app, enable VoiceOver/NVDA, tab through editor controls, and verify line-number gutter and word-count status bar are announced; record the session in a manual AT sign-off file under `.tmp/`.

---

### 6.4 Preview — Live debounced preview

**Blocking category**: Performance  
**Estimated effort**: ≤1d  
**Concrete next step**: Open the large 120-section fixture on a target release device, record `previewUpdateDurationMs` from the status bar for 60 consecutive edits, and paste the min/max/p95 into `.tmp/rendered-export-audit/performance-profile.json`.

---

### 6.4 Preview — Separate preview theme, inline warnings, transform blocks, export preview

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Load a document with a broken image (triggers a compiler diagnostic), switch to Preview-only mode, verify the inline diagnostic callout renders and the "Go to source" button navigates to the correct Markdown line.

---

### 6.5 File Operations — New, open file, open folder, save, save as, revert, rename, duplicate, reveal

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Trigger the native system file-open and save-as dialogs manually in the Tauri app on macOS, record that both dialogs appear and the chosen path is reflected in the title bar, and append a one-line sign-off to `.tmp/desktop-smoke/native-workflow-report.json`.

---

### 6.5 File Operations — External change detection/conflict handling

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: Run the native workflow smoke on a Windows host via `scripts/run-tauri-webdriver.mjs` and confirm watcher conflict recovery keys appear in `.tmp/desktop-webdriver/report.json`.

---

## Compiler Pipeline

### 8 Front Matter And Variables — YAML metadata, layout/export controls, variable resolution, filters

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: `a907f13` proves `{{key | filter}}` renders empty on missing variables and variable-jump line accuracy (macOS). Remaining: run the WebDriver native workflow bundle on a Windows or Linux supported host and verify `frontMatterEvidence` in `.tmp/desktop-webdriver/report.json` covers filter-insertion and variable-jump assertions.

---

## Versioning, Governance, And Quality

### 9.2 Export snapshots — Manifest with hashes/options/app/version/status/timestamp

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run collect:platform-evidence` on a Windows or Linux host after a successful export, then run `pnpm run check:platform-evidence` to accept the cross-platform manifest audit proof.

---

### 9.3 Release workflow — Status values, badge, draft export warning, approval metadata, release tagging

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: Execute the native workflow bundle on a Windows or Linux supported host, confirm `releaseWorkflowEvidence` keys (status badge, draft warning, approval metadata) appear in `.tmp/desktop-webdriver/report.json`.

---

### 9.7 Business table editor — Visual editor, rows/cols, alignment, paste, sort, formats, readable Markdown, export

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Open the business-table editor in the Tauri app, paste a copied Excel cell range, verify column alignment and readable Markdown output, and record the sign-off in `.tmp/rendered-export-audit/manual-review.html`.

---

### 9.9 Equations — Inline/display math, numbering, references, export support

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` and open the equation artifacts under `.tmp/rendered-export-audit/` in a PDF viewer and a DOCX reader to visually confirm numbered equations and cross-references render correctly.

---

### 9.10 TOC — Automatic TOC with marker/front matter

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports`, open `.tmp/rendered-export-audit/review-cases/toc-page-numbers/` in a PDF viewer, and verify page-number leaders align with actual page positions.

---

### 9.11 Index And Glossary — Automatic index/glossary sections and exclusions

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Load a fixture with `glossary: true` and `index: true` in front matter, export to PDF via `pnpm run test:rendered-exports`, open the output, and verify the generated index and glossary sections appear with correct sort order.

---

### 9.12 Bibliography — BibTeX/CSL JSON, citation syntax, rendered bibliography

**Blocking category**: External engine  
**Estimated effort**: ≤1wk  
**Concrete next step**: `1c71784` promoted Harvard, Chicago notes-bibliography, ACM, and ACS to first-class native styles (5 new tests). Remaining: integrate a full CSL processor (e.g. `citeproc-rs`) into `src-tauri/src/transforms/` for styles beyond the 9 deterministic native ones, add a SearXNG citation-lookup smoke test, and run the native workflow bundle on a supported host to produce `bibliographyEvidence` in `.tmp/desktop-webdriver/report.json`.

---

### 9.13 Cross references — Figures/tables/equations/headings, broken ref diagnostics, export links

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Open the cross-reference manager in the Tauri app, insert a figure reference, break it by renaming the figure, confirm the broken-ref diagnostic appears in the Diagnostics sidebar, then fix it and verify the export link resolves.

---

### 9.14 Captions — Figures/table captions, numbering, list support

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` and open the HTML output to verify the generated "List of Figures" and "List of Tables" sections contain correct numbering and caption text.

---

### 9.15 Advanced layout — Page size/orientation/margins/columns/breaks/headers/footers/keeps/floats

**Blocking category**: Manual QA  
**Estimated effort**: ≤1wk  
**Concrete next step**: Run `pnpm run test:rendered-exports` with the `review-cases/option-heavy` fixture and open the PDF in Adobe Acrobat to verify column layout, page breaks at `<!-- break -->`, and floated figure placement.

---

### 9.16 Brand templates — Brand name/color/logo/font/header/footer/watermark/legal disclaimer

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run test:rendered-exports`, open `.tmp/rendered-export-audit/review-cases/brand-layout/output.pdf` in a PDF viewer, and verify the brand logo, color, header/footer, and watermark render correctly on all pages.

---

### 9.17 Review comments/change notes — Comments, unresolved validation, exports

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Open a fixture with inline review comments, export to DOCX via `pnpm run test:rendered-exports`, and verify the comments appear as DOCX native comment annotations in Word or LibreOffice.

---

### 9.18 Document variables — Front matter/project/data variables

**Blocking category**: Cross-platform CI  
**Estimated effort**: ≤1d  
**Concrete next step**: Run the native WebDriver workflow on a Windows or Linux host and verify `documentVariableEvidence` (data-source variable insertion and currency filter) appears in `.tmp/desktop-webdriver/report.json`.

---

## Transform Pipeline

### 10.4.1 DOT/Graphviz — SVG diagrams and engines

**Blocking category**: External engine  
**Estimated effort**: ≤1d  
**Concrete next step**: Install Graphviz (`brew install graphviz`), run `NEDITOR_TRUST_TRANSFORMS=1 pnpm run test:rendered-exports` with a DOT fixture, and open the HTML output to verify the installed-engine SVG renders the full graph.

---

### 10.4.2 PlantUML — SVG/PNG enterprise diagrams

**Blocking category**: External engine  
**Estimated effort**: ≤1d  
**Concrete next step**: Install `plantuml` and a JRE, run `NEDITOR_TRUST_TRANSFORMS=1 pnpm run test:rendered-exports` with a sequence-diagram fixture, and verify the PNG output appears correctly in the rendered HTML export.

---

### 10.4.3 D2 — SVG diagrams

**Blocking category**: External engine  
**Estimated effort**: ≤1d  
**Concrete next step**: Install `d2` (`brew install d2lang/tap/d2`), run `NEDITOR_TRUST_TRANSFORMS=1 pnpm run test:rendered-exports` with a D2 entity-relationship fixture, and verify the SVG renders beyond the bounded smoke artifact.

---

### 10.4.4 Vega-Lite — Charts

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run test:rendered-exports` with a Vega-Lite fixture containing layered marks and selection encodings, open the HTML artifact, and verify all layers and interactions render correctly in the browser preview.

---

### 10.4.5 Chart — Bar/horizontal bar/line/pie/area/KPI

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` and open `.tmp/rendered-export-audit/output.html` to visually confirm each chart type (bar, horizontal-bar, line, pie, area, KPI) renders with correct labels and colors.

---

### 10.4.6 GeoJSON — Static map preview

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` with a GeoJSON fixture of a multi-polygon feature collection, open the HTML artifact, and verify projection, fill, and stroke render correctly at typical window size.

---

### 10.4.7 TopoJSON — Static map preview

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: `d5270a3` adds antimeridian splitting for lines, rings, and polygons. Remaining: run `pnpm run test:rendered-exports` with a TopoJSON fixture containing topology arcs (e.g. county boundaries), open the HTML artifact, and verify shared boundaries and antimeridian-split geometries render correctly.

---

### 10.4.8 STL — Static preview

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: `d5270a3` adds real 3-point Lambertian per-facet shading. Remaining: run `pnpm run test:rendered-exports` with the STL fixture, open the HTML artifact in a browser, and verify the rendered mesh shows correct Lambertian shading per-facet and a clean silhouette before orbit interaction is attempted.

---

### 10.4.13 OpenAPI — API docs

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run test:rendered-exports` with an OpenAPI 3.1 fixture that uses `nullable: true` and `discriminator`, open the HTML and DOCX artifacts, and verify schema tables and security-scheme sections render correctly.

---

### 10.4.14 JSON Schema — Schema docs

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` with a JSON Schema draft-2020-12 fixture, open the HTML artifact, and verify `$defs`, `if/then/else`, and `unevaluatedProperties` keywords render human-readable property tables.

---

### 10.4.15 BibTeX — Bibliography rendering

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run test:rendered-exports` with a BibTeX fixture, open `.tmp/rendered-export-audit/output.html`, and verify rendered bibliography entries match the expected CSL format for the chosen citation style.

---

### 10.4.16 Glossary — Definitions and term rendering

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Load a Glossary transform block in the Tauri app, trigger compilation, verify the rendered glossary section alphabetizes entries, and confirm term definitions link back to first usage in the document preview.

---

### 10.4.17 Timeline — Timeline SVG

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` with a Timeline fixture containing overlapping date ranges, open the HTML artifact, and verify lane assignment and label truncation render without collision.

---

### 10.5 Later transforms — roadmap, ADR, diff, QR, SQL, etc.

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Run `pnpm run test:rendered-exports` with the later-transforms fixture set, open each transform artifact in `.tmp/rendered-export-audit/`, and record visual sign-offs in the manual checklist at `.tmp/rendered-export-audit/manual-review.html`.

---

## AI Workflow

### 11 AI workflow — Paste cleanup preview, voice-guided drafting, and governance

**Blocking category**: AI runtime  
**Estimated effort**: ≤1d  
**Concrete next step**: On a host with a working microphone and a configured Ollama/LiteLLM endpoint, open the AI Agent Workspace in the Tauri app, record a voice dictation snippet, and verify the AI paste cleanup preview renders the normalized output before insertion.

---

### 11 AI workflow addendum — Long-form wizard planning gates

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Use the Document Wizard in the Tauri app to generate a full 10-section business report, verify each planning gate prompts for metadata before drafting, and confirm the final compiled document passes `pnpm run check:release-readiness`.

---

## Business Document Features (Cross-reference Sections 13–17)

### 13 Tables/data — Table editing, formulas, data sources

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: In the Tauri app table editor, paste a 5×5 clipboard range from a spreadsheet application, sort by a numeric column, and verify the generated Markdown table is readable and the formula column evaluates correctly in the compiler output.

---

### 14 Equations — Math authoring/render/export

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: Run `pnpm run test:rendered-exports` and open the equation audit artifacts under `.tmp/rendered-export-audit/` in a PDF viewer and DOCX reader to confirm numbered display equations and inline `$...$` both render correctly.

---

### 15 Bibliography/citations — BibTeX/CSL/citation rendering

**Blocking category**: External engine  
**Estimated effort**: ≤1wk  
**Concrete next step**: `1c71784` promoted Harvard, Chicago notes-bibliography, ACM, and ACS to first-class native styles (5 new tests). Remaining: integrate a full CSL processor into `src-tauri/src/transforms/` (same path as 9.12) for styles beyond the 9 deterministic native ones, add a SearXNG citation-lookup smoke test, and run the native workflow bundle on a supported host to produce cross-platform citation evidence.

---

### 16 Index/glossary — Index and glossary generation

**Blocking category**: Manual QA  
**Estimated effort**: ≤1d  
**Concrete next step**: Export the market-entry-report fixture to PDF via `pnpm run test:rendered-exports` and open `.tmp/rendered-export-audit/output.pdf` to verify the generated index section lists page numbers and the glossary definitions are alphabetized.

---

### 17 Layout/reflow — Layout model/directives/export mapping

**Blocking category**: Manual QA  
**Estimated effort**: ≤1wk  
**Concrete next step**: Export the option-heavy fixture to PDF and open it in Acrobat Pro to check overflow behavior on column-break directives, float placement, and multi-page keep-together constraints across a 40+ page document.

---

## Application Preferences, Security, Accessibility, And Performance

### 21 Preferences — Theme, typography, export, Git, AI, transforms, recents

**Blocking category**: AI runtime  
**Estimated effort**: ≤1d  
**Concrete next step**: On a host with speakers enabled and a configured TTS engine, open the Preferences panel in the Tauri app, trigger the TTS preview playback, and verify audible output; also trigger an Ollama model pull to confirm download-progress UI renders.

---

### 22 Security/privacy — Local-first, trust-gated executable transforms, no shell

**Blocking category**: Security review  
**Estimated effort**: >1wk  
**Concrete next step**: `135bf07` closed 3 HIGH findings (canonicalized document-local paths, kill-on-timeout, secure cache dir) and 5 lower findings (sqlite -readonly, installer hardening). Remaining: engage a third-party security auditor to review `src-tauri/src/transforms/external.rs` and the Tauri IPC command surface in `src-tauri/src/lib.rs` for any residual shell-injection, path-traversal, or privilege-escalation vectors; also collect cross-platform release-process proof.

---

### 23 Accessibility — Keyboard, ARIA, contrast, reduced motion

**Blocking category**: Manual QA  
**Estimated effort**: >1wk  
**Concrete next step**: Book a screen-reader testing session with VoiceOver on macOS and NVDA on Windows; run through the keyboard-navigation checklist in `pnpm run check:a11y:manual` and record pass/fail evidence for each WCAG 2.1 AA criterion.

---

### 24 Performance — Large docs, debounced preview, transform cache, progress

**Blocking category**: Performance  
**Estimated effort**: ≤1d  
**Concrete next step**: On the target release MacBook, open the 120-section fixture, enable Instruments Time Profiler, perform 100 successive edits, and paste the p95 preview-render time into a new `performance-profile.json` under `.tmp/`.

---

### 28 Acceptance criteria — Concrete app acceptance

**Blocking category**: Manual QA  
**Estimated effort**: >1wk  
**Concrete next step**: Close all individual evidence gates listed above (signing, Google Docs auth, AI runtime, performance profile, accessibility, and manual visual sign-offs), then run `pnpm run check:release-readiness` and supply the completed sign-off file it templates.

---

## Troubleshooting Reference

### Troubleshooting — Permission, empty output, timeout, trust disabled, cache stale

**Blocking category**: Manual QA  
**Estimated effort**: ≤1h  
**Concrete next step**: On a macOS host, intentionally trigger each of the five failure modes (permission denied, empty output, timeout, trust disabled, stale cache) using the guidance in `docs/user-guide.md`, verify the app displays the described recovery message, and record pass/fail in a one-page sign-off.

---

*56 Partial entries (9.19 Pikchr removed — flipped Complete by `17fdd3c`). Blocking-category summary:*

| Blocking category | Count |
|---|---|
| Manual QA | 36 |
| Cross-platform CI | 8 |
| External engine | 6 |
| AI runtime | 2 |
| Signing | 1 |
| Live service | 1 |
| Performance | 2 |
| Security review | 1 |

*Top 5 easiest closes (≤1h, no external dependency, implementation verified complete):*

1. **[6.2] Primary Layout** — resize the Tauri window to three viewport sizes and record the sign-off.
2. **[6.4] Separate preview theme** — open a broken-image fixture in Preview-only mode; one QA pass.
3. **[9.10] TOC** — `pnpm run test:rendered-exports` already writes the artifacts; open and sign off.
4. **[9.14] Captions** — same rendered-export audit; open the List-of-Figures section and sign off.
5. **[Troubleshooting]** — trigger five scripted failure modes and record pass/fail; one short session.
