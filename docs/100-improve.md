# 100 High-Value Improvements For NEditor

This backlog describes the improvements, extensions, enhancements, and new
features that would make NEditor a world-class, AI-first business document
workbench. The intent is to prioritize work that materially improves document
creation, document quality, export readiness, business workflows, and
production deployability.

## AI-First Creation

1. **Intent-first document launcher**: start every serious document from
   outcome, audience, deadline, owner, source confidence, approval path, and
   distribution target instead of a blank page.
2. **Adaptive creation wizard**: change questions dynamically by document type,
   so a proposal, novel, RFP response, textbook, lesson plan, and script each
   get the right workflow.
3. **AI-generated questionnaire**: inspect the outline, context, source pack,
   and placeholders, then ask only the missing high-value questions.
4. **Suggested optimal answers**: provide context-aware draft answers for every
   wizard field so non-expert users can move forward quickly.
5. **Section-by-section drafting queue**: generate each section sequentially
   with visible inputs, assumptions, evidence, and review gates.
6. **AI quality pass pipeline**: run structure, evidence, clarity, tone,
   compliance, and export-readiness passes after drafting.
7. **AI humanization controls**: detect generic AI phrasing and replace it with
   specific, audience-aware, evidence-grounded writing.
8. **AI reviewer agents**: separate strategist, editor, evidence reviewer, risk
   reviewer, citation reviewer, and export reviewer roles.
9. **Agent task board**: show what the AI is doing, what it needs, what is
   blocked, what is ready, and what requires human acceptance.
10. **Provider-agnostic AI runtime**: make OpenAI, Anthropic, Ollama, Codex,
    Claude Code, OpenCode, local endpoints, and future providers
    interchangeable behind one governed interface.

## Business Document Intelligence

11. **Reusable business profile**: store company name, address, people, brand,
    credentials, legal text, tax identifiers, websites, and recurring facts.
12. **Document memory**: remember accepted terminology, tone, boilerplate,
    reviewers, recurring claims, and rejected directions per workspace.
13. **Snippet library**: provide reusable sections for executive summaries,
    risks, methods, bios, disclaimers, pricing notes, governance, and review
    handoffs.
14. **Template marketplace format**: support portable template packs with
    metadata, placeholders, examples, outline rules, and usage guidance.
15. **Document type playbooks**: guide proposals, RFPs, RFQs, tenders,
    tutorials, lesson plans, textbooks, novels, podcast scripts, movie scripts,
    business cases, and executive briefs.
16. **Business decision prompts**: ask domain-specific decisions such as win
    theme for proposals, buyer concern for RFPs, or narrative arc for novels.
17. **Brand kit manager**: manage colors, fonts, logos, headers, footers,
    covers, watermarks, layout presets, and export defaults.
18. **Approval metadata gate**: block external distribution until owner,
    reviewer, approval date, status, unresolved comments, and source confidence
    are complete.
19. **Versioned reusable clauses**: track standard clauses and warn when old
    approved language is used in a new document.
20. **Workspace onboarding wizard**: guide non-technical users through profile,
    AI setup, exports, templates, CLI setup, transform handlers, and support
    bundle creation.

## RFP And Procurement Excellence

21. **Native RFP ingestion**: accept PDF, DOCX, Markdown, plain text, and URL
    sources as first-class RFP inputs.
22. **Compliance checklist extractor**: convert mandatory requirements, tables,
    annexes, deadlines, disqualifiers, language requirements, and scored items
    into a checklist.
23. **Compliance matrix generator**: map every requirement to response section,
    owner, status, evidence, verification method, and source reference.
24. **Stated and implied intent analysis**: surface what the buyer explicitly
    asks for and what they are likely optimizing for.
25. **Scoring scheme extraction**: identify criteria, sub-criteria, weights,
    pass/fail gates, and page-allocation implications.
26. **Critical disqualifier panel**: isolate automatic-rejection traps at the
    top of the response workflow.
27. **RFP proposal outline generator**: create a score-maximizing technical
    proposal outline before drafting full text.
28. **Requirement coverage validator**: prove every mandatory requirement is
    addressed before export or submission.
29. **Attachment and annex tracker**: list all required forms, CVs,
    declarations, financials, certificates, schedules, and appendices.
30. **Win-theme builder**: turn RFP priorities into proposal themes,
    differentiators, proof points, and executive summary framing.

## Deep Research And Citations

31. **Deep research workspace**: plan searches, collect sources, summarize
    claims, and draft reports from evidence.
32. **Search provider choices**: support SearXNG, DuckDuckGo, Tavily, and
    private or local search indexes.
33. **Source document vault**: download and store cited documents beside the
    NEditor source document.
34. **Claim inventory**: extract claims needing evidence and link each one to
    a citation, source file, or unresolved citation task.
35. **Citation TODO workflow**: make unsupported claims visible, assignable,
    searchable, and resolvable.
36. **Bibliography population**: automatically generate BibTeX or CSL entries
    from collected source documents and URLs.
37. **Research length slider**: scale output from a one-page brief to a 200-page
    report with iterative expansion and coverage tracking.
38. **Source quality scoring**: rank sources by authority, recency, relevance,
    independence, and evidence strength.
39. **Evidence conflict detection**: flag claims where credible sources
    disagree or where the source does not support the draft language.
40. **Research audit packet**: export source list, downloaded files, search
    queries, evidence decisions, and bibliography state.

## Editor Ergonomics

41. **Editable outline mode**: CRUD chapters, sections, subsections, and
    subsubsections before prose exists.
42. **Outline-to-document skeleton**: generate headings, placeholders, table of
    contents markers, section briefs, and drafting tasks from outline mode.
43. **True folding**: fold headings, lists, fenced blocks, tables, includes,
    comments, and long generated sections.
44. **Powerful search and replace**: support regex, scoped search, heading
    search, symbol search, source search, and replace preview.
45. **Document map sidebar**: unify outline, comments, citations, figures,
    tables, equations, includes, glossary terms, and TODOs in one navigable
    panel.
46. **Multi-toolbar layout**: group file, writing, review, AI, export, table,
    layout, and reference commands into collapsible toolbar rows.
47. **Restore toolbar affordance**: always show a compact visible control to
    unhide collapsed toolbars.
48. **Maximize writing mode restore**: provide persistent restore controls and
    shortcuts after entering distraction-free writing mode.
49. **Resizable button labels**: let users choose icons, text, or both, plus
    compact, normal, and comfortable density.
50. **Keyboard command center**: expose every capability through searchable
    commands, aliases, help text, and keyboard shortcuts.

## Tables, Data, And Computation

51. **Two-way table editor**: keep grid edits and Markdown source edits
    synchronized without losing captions, formulas, alignment, or comments.
52. **XLSX import**: convert worksheets into Markdown tables or reusable data
    source references.
53. **CSV and XLSX export**: export selected Markdown tables to CSV and XLSX.
54. **SQL transform**: query SQLite, PostgreSQL, MySQL, DuckDB, and other
    configured databases, then insert results as Markdown tables.
55. **Safe database profiles**: manage database connections without storing
    secrets in documents.
56. **Formula-aware tables**: support totals, percentages, currency, formats,
    validation, summary rows, and export metadata.
57. **Business calculation templates**: provide ROI, NPV, IRR, margin, CAGR,
    utilization, break-even, weighted score, pricing, forecast, and budget
    templates.
58. **Scientific and mathematical templates**: provide matrices, regression,
    units, probability, statistics, physics, engineering, and optimization
    templates.
59. **Transform template manager**: create, edit, tag, search, install, and
    share calculation and transform templates.
60. **Data refresh workflow**: detect stale data-source outputs and refresh them
    safely with audit records.

## Layout And Beautiful Documents

61. **Multi-column layouts**: add first-class Markdown controls for columns,
    callouts, sidebars, page breaks, and section-specific layouts.
62. **LaTeX template integration**: map Markdown documents into curated LaTeX
    classes and template libraries.
63. **Page design presets**: ship board memo, consulting report, academic
    article, book chapter, proposal, newsletter, and policy presets.
64. **Professional cover builder**: generate brand-aware covers with title,
    subtitle, client, date, confidentiality, status, and version.
65. **Figure, table, and equation numbering**: provide cross-referenceable
    labels and export-safe numbering.
66. **Equation editor**: support visual equation insertion with LaTeX source
    round-trip.
67. **Chart designer**: create business charts from tables and data sources.
68. **Callout and admonition styles**: support risks, decisions, notes,
    warnings, recommendations, assumptions, evidence, and action boxes.
69. **Print preview mode**: approximate final pagination, margins, columns,
    headers, footers, and page breaks before export.
70. **Export visual QA dashboard**: compare rendered PDF, DOCX, HTML, EPUB, and
    LaTeX outputs against expected document structure.

## Export, Publishing, And Distribution

71. **HTML export polish**: produce standalone, branded, accessible HTML
    packages.
72. **EPUB export polish**: include navigation, metadata, cover, stylesheet,
    manifest, and validation.
73. **Google Docs import handoff**: upload a DOCX or HTML package and verify
    readback when authenticated.
74. **Substack package export**: produce clean copy-ready HTML, plaintext,
    metadata, preview text, and publishing checklist.
75. **Blog and CMS publishing**: support WordPress, Ghost, generic webhooks,
    static site bundles, and manual handoff packages.
76. **LaTeX and PDF build path**: export `.tex`, compile locally when available,
    and report missing toolchain pieces clearly.
77. **DOCX style mapping**: map headings, captions, references, comments,
    custom properties, and metadata into Word-friendly styles.
78. **Markdown bundle export**: include source, includes, assets,
    bibliography, manifest, hashes, and provenance.
79. **Distribution preflight**: run target-specific checks before publishing
    externally.
80. **Export profiles**: save reusable export configurations per client,
    workspace, document type, or distribution target.

## Voice, TTS, And Accessibility

81. **Voice command interface**: dictate document changes, outline changes,
    revision instructions, or workflow commands.
82. **Voice-first document wizard**: speak the document type, context,
    placeholders, and desired result.
83. **Read selected text aloud**: support OS-native TTS and optional external
    TTS engines such as Supertonic.
84. **Consent-gated TTS models**: offer model downloads only after user opt-in,
    with size, source, and storage location clearly shown.
85. **Voice correction loop**: let users say "make that more formal", "expand
    section 3", or "turn this into a board memo" naturally.
86. **Screen-reader QA mode**: validate document navigation, labels, toolbar
    state, dialogs, focus order, and status messages.
87. **Accessible command palette**: ensure every major workflow is reachable
    without a mouse.
88. **Keyboard-first table editing**: support spreadsheet-like navigation
    without breaking Markdown source.
89. **High-contrast and reduced-motion modes**: provide robust accessibility
    preferences for long writing sessions.
90. **Plain-language help overlays**: explain complex workflows in context
    without overwhelming the user.

## Productization, Setup, And Release

91. **Unified configurator**: centralize AI providers, TTS, transforms, Google
    auth, CLI setup, templates, export defaults, and external engines.
92. **Guided provider setup**: provide setup wizards for OpenAI, Ollama, Claude
    Code, Codex, OpenCode, Google, local endpoints, and future providers.
93. **Ollama model picker**: discover models from local or cloud Ollama
    endpoints and select models per workflow.
94. **Transform handler installer**: download, install, verify, update, and
    remove optional transform handlers from the configurator.
95. **`ned` CLI completion**: open files, convert formats, generate documents,
    inspect readiness, manage profiles, and manage templates from the command
    line.
96. **Deploy CLI menu item**: make `ned` globally available from the app with
    clear permission, target path, rollback, and status reporting.
97. **Default Markdown reader setup**: provide OS-specific checkbox setup and
    guided fallback steps.
98. **Homebrew release path**: support signed and notarized artifacts, cask
    generation, checksum validation, and tap documentation.
99. **Release evidence dashboard**: show exactly what is complete, blocked,
    manual, credentialed, cross-platform, stale, or ready to send.
100. **In-app demo and tutorial system**: guide new users through the showcase
     document so they see real beautiful outputs, tables, equations, images,
     citations, AI workflows, and export paths immediately.

## Recommended Implementation Sequence

1. Stabilize release and evidence gates so every production claim is backed by
   current artifacts.
2. Finish the unified configurator because AI, TTS, transforms, Google Docs,
   CLI, and optional engines all depend on setup clarity.
3. Make AI document creation genuinely adaptive by document type, context,
   outline, evidence, and distribution target.
4. Harden RFP, deep research, citation, and source-document workflows because
   these create the strongest business differentiation.
5. Polish layout, export, and visual QA until NEditor consistently produces
   documents that look better than standard office-suite output.
