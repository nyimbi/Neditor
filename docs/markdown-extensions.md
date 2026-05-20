# Markdown Extensions

NEditor keeps Markdown readable while adding business-document syntax for
metadata, modular documents, governance, calculations, transforms, and export
evidence.

This page documents the author-facing syntax that is already represented in
the implementation, examples, or verification tests. The
[specification](specification.md) remains the authority for full product scope
and future extensions.

## Front Matter

Use YAML front matter at the top of a document:

```yaml
---
title: Q3 Board Paper
subtitle: Operating review and approval request
version: 1.0.0
status: approved
approvedBy: Board Secretary
approvedAt: 2026-05-20T09:00:00Z
classification: confidential
client: Acme Holdings
toc: true
citationStyle: author-year
brand:
  name: Acme Holdings
  color: "#1F6F55"
layout:
  header: "{{title}} | {{classification}}"
  footer: "Page {{page}} of {{pages}}"
---
```

Common fields:

| Field | Purpose |
| --- | --- |
| `title`, `subtitle`, `author`, `client`, `classification` | Document identity and metadata. |
| `version`, `status`, `approvedBy`, `approvedAt` | Review and release governance. |
| `toc`, `citationStyle` | Generated section and bibliography defaults. |
| `brand` | Export brand name, color, logo, fonts, and defaults. |
| `layout` | Header, footer, page, margin, column, and flow options. |
| `variables` | Project or document values used by `{{name}}` placeholders. |

## Includes

Master documents can include child Markdown files:

```md
!include chapters/introduction.md
{{include chapters/market-analysis.md}}
<!-- include: appendices/financials.md -->
```

Rules:

- Paths resolve relative to the current document.
- Child front matter is stripped when included.
- Missing files produce diagnostics.
- Circular includes produce diagnostics.
- Nested includes stop at a safe maximum depth.
- The include graph contributes to snapshots and export manifests.

## Generated Sections

Add markers where generated sections should appear:

```md
[TOC]
[INDEX]
[BIBLIOGRAPHY]
[LIST_OF_FIGURES]
[LIST_OF_TABLES]
```

`toc: true` in front matter can also request a table of contents.

Generated sections are built from the compiled document model, so fenced
examples are excluded from heading, caption, citation, and reference scans.

## Variables And Inline Formulas

Use front matter, project variables, calculation results, and default document
values with `{{name}}` syntax:

```md
Prepared for {{client}}.
```

Inline formulas use `{{= expression }}` and can include format filters:

```md
Margin: {{=margin | percent}}
After tax: {{=profit * 0.70 | currency}}
Rounded score: {{=score | round}}
```

Formula diagnostics include source ranges where possible.

## Calculation Blocks

Use `calc` fenced blocks for document-level calculations:

````md
```calc
revenue = 100
cost = 40
profit = revenue - cost
margin = profit / revenue
healthy = IF(revenue > cost, 1, 0)
```

Margin: {{=margin | percent}}
````

Supported patterns include arithmetic, percentages, named values, forward
references, table formulas, and dependency diagnostics. Circular dependencies
are reported as diagnostics.

## Tables And Data Sources

Standard Markdown tables remain readable source:

```md
| Metric | Q2 Actual | Q3 Plan |
| --- | ---: | ---: |
| Revenue | 1200000 | 1450000 |
| Gross margin | 0.61 | 0.64 |
```

CSV and TSV transform blocks can render tables and evaluate formula cells:

````md
```csv caption="Quarterly rollout budget"
Quarter,Implementation,Training,Total
Q1,12000,3000,=12000+3000
Q2,18000,4000,=18000+4000
```
````

The table editor can write clean Markdown after paste import, sorting, row and
column edits, alignment, totals, formula rows, and merged-cell metadata edits.

## Figures, Captions, And Cross References

Use extended image attributes for stable figure labels and captions:

```md
![System architecture](architecture.svg){#fig:architecture caption="System architecture"}
```

Reference labels from prose:

```md
See {@fig:architecture} for the system layout.
The result follows from equation {@eq:roi}.
```

NEditor tracks references to headings, figures, tables, equations, appendices,
and decisions. Broken references are reported in diagnostics and export
readiness.

## Equations

Use inline and display math for business and research documents:

```md
Inline confidence is $p = 0.81$.

$$
confidence = signal / noise
$$ {#eq:confidence caption="Confidence score"}
```

Missing labels or captions can produce readiness warnings when the equation is
used in a release-grade export.

## Citations And Bibliography

Use citation syntax in prose:

```md
Prior research on competitive advantage [@porter1985, p. 42] supports the plan.
```

Add bibliography data inline or load it through supported bibliography inputs.
Use `[BIBLIOGRAPHY]` where references should render:

```bibtex
@book{porter1985,
  title = {Competitive Advantage},
  author = {Porter, Michael E.},
  year = {1985}
}
```

```md
[BIBLIOGRAPHY]
```

Set `citationStyle` to `title`, `author-year`, `key`, or `numeric` in front
matter. Unsupported `citationStyle` or `cslStyle` names produce a warning and
fall back to title rendering until a native CSL adapter is added.

Diagnostics cover missing keys, duplicate bibliography keys, missing
bibliography sources, and unsupported citation styles. The references panel
exposes resolved entries and problems.

## Glossary And Index

Use a glossary block for definitions:

````md
```glossary
ARR: Annual recurring revenue.
NRR: Net revenue retention.
```
````

Add `[INDEX]` where the generated index should appear. Index terms can come
from headings, glossary terms, bold terms, repeated proper nouns, and explicit
index markers as support expands.

## Review Comments And Change Notes

Inline comments keep review evidence near the source:

```md
<!-- comment: author: Reviewer | at: 2026-05-20 | open | Confirm final margin assumptions. -->
```

Release readiness checks can report unresolved comments and malformed audit
metadata. Change notes use the same audit principle: author, timestamp, and
body text should be present before release-grade export.

## AI Provenance

Use `ai-source` blocks to preserve AI drafting context:

````md
```ai-source
provider: OpenAI
model: gpt-5.4
date: 2026-05-20
promptSummary: policy draft outline from internal notes
reviewedBy: Policy Team
reviewedAt: 2026-05-20T14:00:00Z
status: human-reviewed
```
````

AI-assisted sections can be marked as needing review or human-reviewed.
Readiness diagnostics report incomplete provenance metadata and invalid review
statuses.

## Transform Blocks

Fenced-code transforms produce static artifacts for preview and export.

Common transform names:

| Transform | Purpose |
| --- | --- |
| `calc` | Document calculations. |
| `chart` | Bar, line, pie, area, and KPI charts. |
| `mermaid`, `pikchr`, `dot`, `graphviz`, `d2`, `plantuml` | Diagrams with native fallback or trusted external engine support. |
| `csv`, `tsv`, `json`, `yaml` | Structured data rendering. |
| `openapi`, `json-schema` | API operations, request/response contracts, component schemas, nested fields, and schema constraints. |
| `bibtex`, `glossary`, `timeline`, `roadmap`, `adr`, `diff`, `qr` | Business-document artifacts and generated sections. |
| `vega-lite`, `geojson`, `topojson`, `stl` | Visual data previews with static export fallbacks. |

Example chart:

````md
```chart
type: bar
title: Regional revenue plan
data:
  - region: East
    revenue: 520
  - region: West
    revenue: 410
x: region
y: revenue
```
````

External engines are disabled until trusted. See
[External transform setup](external-transforms.md) for Graphviz, D2, PlantUML,
and Pikchr setup.

## Export Readiness Markers

Before export, NEditor validates the compiled document for issues such as:

- Required metadata and release approval.
- Draft status and dirty Git state.
- Missing includes, media, citations, labels, captions, and references.
- Formula and table formula errors.
- Transform engine trust, path, timeout, stderr, output, and cache details.
- Unresolved comments and malformed change notes.
- Incomplete AI provenance or missing human review metadata.

Readiness diagnostics are copied into export manifests so deliverables can be
audited after the artifact leaves the editor.
