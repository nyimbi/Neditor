# AI-First Platform Roadmap

This roadmap turns the AI-first direction for NEditor into 50 concrete product and engineering changes. The target is not "AI buttons around a Markdown editor"; the target is a document operating system where intent capture, composition, editing, revision, review, distribution, governance, and learning are all orchestrated by visible agents with human control points.

AI-first means every important workflow can be expressed as: goal, context, source pack, memory, outline, tasks, drafts, diffs, review, approval, distribution, and audit evidence. The user should be able to start with a spoken or written business intent, collaborate with agents section by section, keep evidence attached, and leave with a document ready for the right channel.

## Operating Principles

- Keep the user in control of source files, approvals, final acceptance, and distribution.
- Make agent work visible as plans, tasks, evidence, diffs, provenance, QA findings, and rollback options.
- Treat every generated or rewritten section as a draft until a human accepts it.
- Prefer deterministic local analysis before provider calls, and make provider handoff explicit.
- Preserve local-first ownership while supporting approved external providers, collaborators, and publishing targets.
- Learn from each document through reviewable memory, not hidden personalization.

## Capability Model

Every AI-first feature should contribute to one of these durable capabilities:

- **Understand**: identify document type, audience, outcome, constraints, missing information, and reusable memory.
- **Plan**: build the outline, task graph, review gates, source requirements, and distribution route.
- **Compose**: draft section by section with placeholders, evidence, voice, and target depth.
- **Revise**: apply scoped edits with change rationale, drift checks, and acceptance queues.
- **Verify**: inspect claims, citations, comments, AI markers, accessibility, metadata, and export readiness.
- **Distribute**: package the document for review, publishing, archive, or handoff with manifests and audit evidence.
- **Learn**: capture accepted terminology, style, decisions, reviewer preferences, rejected directions, and channel rules for future runs.

## 1. Intent And Document Creation

1. **Intent-first creation cockpit**: make AI Create the primary start path for serious documents, collecting document type, audience, outcome, owner, deadline, source confidence, approval path, and distribution target before any content is drafted.
2. **Voice-to-brief conversation**: let users dictate rough thoughts while Docs Live turns them into a structured brief, asks clarifying questions, confirms placeholder values, and separates facts from assumptions.
3. **Adaptive intake questionnaire**: generate missing-context questions from the selected document type, outline, existing draft, source pack, reusable memory, governance requirements, and intended export or publishing channel.
4. **Document-type playbook library**: provide governed starts for proposals, board papers, policies, SOPs, release notes, strategy memos, research briefs, grant applications, technical papers, blog posts, newsletters, customer letters, and executive updates.
5. **Context completeness contract**: score whether audience, evidence, constraints, examples, tone, approvals, reviewer expectations, variables, and distribution requirements are sufficient for a responsible first draft.

## 2. Outline And Architecture

6. **Outline mode as a first-class editor**: let users create, reorder, nest, rename, collapse, expand, and delete chapters, sections, subsections, and subsubsections before body text exists.
7. **Agent outline architect**: have an agent critique missing sections, duplicated logic, weak sequencing, decision gaps, evidence gaps, stakeholder omissions, and audience mismatch before drafting begins.
8. **Section contract cards**: attach purpose, target reader, desired decision, evidence expectations, reviewer owner, drafting depth, risk level, and done criteria to each outline node.
9. **Outline variants and comparison**: generate alternative structures such as executive-first, problem-solution, evidence-led, legal-safe, technical-deep, investor-ready, or narrative publishing versions for user comparison.
10. **Outline-to-document scaffold**: convert accepted outlines into Markdown with front matter, placeholders, TODOs, review comments, section IDs, and draft prompts that remain traceable through later revisions.

## 3. Agentic Composition

11. **Section-by-section drafting queue**: draft one section at a time from the outline with explicit inputs, target depth, dependencies, evidence needs, reviewer mandates, and acceptance status.
12. **Multi-agent composition roles**: separate planner, researcher, drafter, editor, citation reviewer, risk reviewer, humanizer, and distribution agent outputs instead of collapsing all work into one opaque response.
13. **Placeholder-aware generation**: keep client names, dates, financial values, product names, legal terms, source IDs, and repeated variables as structured placeholders until the user confirms final values.
14. **Drafting depth controls**: allow each section to be summary, standard, detailed, technical, legal, persuasive, executive, educational, or publish-ready, with depth visible in the task plan.
15. **Composable draft history**: preserve generated versions, prompt summaries, section fingerprints, rationale, reviewer notes, and restore points so users can revisit or reuse prior draft attempts.

## 4. Editing, Revision, And Collaboration

16. **Instruction-to-edit planner**: convert natural language such as "tighten this for the CFO" or "make this publishable on Substack" into scoped edit tasks, target sections, risk notes, and acceptance criteria.
17. **Selection-aware revision packets**: preserve original selected text, proposed replacement, change summary, meaning-drift risks, evidence implications, and rollback instructions for every generated edit.
18. **Multi-pass revision pipeline**: support clarity, brevity, tone, structure, evidence, legal caution, executive summary, accessibility, translation prep, humanization, and final polish as separate selectable passes.
19. **Meaning-drift and commitment checks**: compare original and revised text for changed claims, numbers, obligations, caveats, dates, commitments, permissions, warranties, or risk language.
20. **Collaborative comment agents**: turn unresolved comments into tasks, suggest responses, identify conflicting reviewer feedback, propose compromise wording, and keep human resolution notes attached.

## 5. Evidence, Research, And Source Grounding

21. **Current-document evidence scanner**: detect unresolved placeholders, citation TODOs, unsupported claims, unreviewed AI markers, stale metadata, unresolved comments, broken links, and export blockers.
22. **Structured source pack builder**: collect pasted notes, files, URLs, claims, reviewer comments, tables, examples, and prior decisions into a typed context pack with provenance and confidence labels.
23. **Claim inventory and source ledger**: extract numbers, dates, names, commitments, quoted facts, forecasts, and assertions into a review table with source, confidence, owner, and verification status.
24. **Citation and bibliography agent**: recommend citation TODOs, resolve known references, flag missing bibliography fields, detect duplicate references, and prepare citation tasks for human confirmation.
25. **Research brief handoff**: produce a provider-ready research packet that includes the question, known facts, exclusions, required source quality, citation format, and prohibited assumptions.

## 6. Templates, Transforms, And Structured Work

26. **Business calculation template gallery**: provide fillable templates for margin, ROI, breakeven, CAC payback, runway, NPV, IRR, sensitivity analysis, cohort retention, pricing, utilization, and forecast scenarios.
27. **Scientific and mathematical template gallery**: provide fillable templates for units, dimensional checks, statistics, regressions, confidence intervals, probability, hypothesis tests, equations, and reproducible method notes.
28. **Transform template manager**: let users create, clone, tag, search, favorite, validate, and share calc, chart, diagram, table, timeline, roadmap, and schema templates.
29. **Agent-selected transforms**: allow agents to recommend or insert the right table, chart, calc, diagram, timeline, QR code, schema, or visual block based on the document goal and source data.
30. **Data-to-narrative bridge**: connect tables, calculations, charts, and claims so changed values trigger narrative review tasks, source refresh tasks, and export-readiness warnings.

## 7. Review, QA, And Humanization

31. **Document-type quality gates**: tailor QA checks for board memos, policies, reports, academic papers, blog posts, newsletters, proposals, release notes, legal-sensitive documents, and technical specs.
32. **Humanization and voice guardrails**: identify generic AI phrasing, empty transitions, inflated confidence, repetitive structure, weak verbs, unnatural summaries, and brand-voice drift before review.
33. **Risk and compliance reviewer**: flag unsupported promises, regulated claims, privacy issues, licensing issues, missing disclaimers, approval gaps, and distribution-sensitive language.
34. **Readiness dashboard**: show an overall readiness score plus source grounding, review status, approval metadata, unresolved tasks, export blockers, provider evidence, and distribution state.
35. **Pre-review rehearsal**: simulate likely reviewer questions, objections, redlines, and missing evidence requests so the user can strengthen the document before formal review.

## 8. Distribution, Publishing, And Handoff

36. **Target-aware export runbooks**: generate preflight, formatting, asset, metadata, accessibility, review, and evidence requirements for HTML, PDF, DOCX, PPTX, Markdown bundles, blog, Substack, LaTeX, and Google Docs.
37. **Blog and website publishing package**: produce Markdown, standalone HTML, plain text, assets, slug, excerpt, tags, canonical URL, RSS seed, social preview copy, and manifest.
38. **Substack newsletter package**: produce Substack-safe HTML, Markdown, subject line, preview text, text fallback, asset checklist, paywall notes, link audit, and preview evidence.
39. **Google Docs collaboration package**: produce DOCX, HTML, Markdown, text, assets, import metadata, comment handoff, readback checklist, sharing notes, and Drive URL capture.
40. **LaTeX and technical publishing package**: produce TeX source, bibliography hints, equation checks, labels, figure references, compile checklist, PDF/hash evidence, and journal or conference submission notes.

## 9. Provider, Runtime, And Automation

41. **Provider request workbench**: build redacted system/user prompts, model settings, source packs, document memory, reviewer tasks, output schemas, and cURL starters without persisting secrets.
42. **Session-only credential flow**: allow provider execution with API keys or local gateway tokens that never enter documents, run history, exported packages, or audit artifacts.
43. **Provider response wrapper**: wrap returned content in local provenance, review status, diff metadata, source fingerprints, provider hashes, and human-review warnings before insertion.
44. **Local model gateway profile**: make localhost and private-network models first-class options with the same tasking, provenance, evidence, redaction, and approval rules as cloud providers.
45. **Agent automation scheduler**: queue safe background tasks such as evidence scan, outline critique, transform validation, export preflight, accessibility check, and readiness refresh while keeping destructive actions manual.

## 10. Governance, Memory, And Enterprise Readiness

46. **Reusable document memory**: capture accepted terminology, style rules, decisions, rejected directions, reviewer preferences, distribution preferences, and source constraints as editable local memory for future runs.
47. **Run history and audit trail**: preserve agent plans, instructions, context, source packs, memory, outputs, applied edits, task status, fingerprints, provider evidence, and rollback steps.
48. **Approval metadata enforcement**: warn or block distribution when status, reviewer, owner, approvedAt, source confidence, unresolved comments, or release target metadata is incomplete.
49. **Accessible guided demo system**: walk new users through AI creation, outline planning, voice intake, section drafting, provider handoff, review governance, export evidence, and publishing packages with real app actions.
50. **Spec and release evidence gates**: keep the spec matrix, AI-roadmap validator, runtime readiness, accessibility checks, native desktop smoke, cross-platform packaging proof, and export evidence kit as release blockers until verified.

## Near-Term Implementation Order

1. Deepen the local agent substrate: intent cockpit, reusable memory, outline contracts, claim inventory, lifecycle task persistence, and persistent control center.
2. Make composition genuinely section-aware: queue drafts, attach section contracts, record versions, support multi-pass revisions, and preserve edit acceptance state.
3. Harden evidence and review: claim ledger, source pack confidence, citation workflows, humanization, meaning-drift checks, and pre-review rehearsal.
4. Finish distribution packages: blog, Substack, Google Docs, LaTeX, HTML, DOCX, PDF, PPTX, and Markdown bundle manifests with target-specific preflight.
5. Operationalize provider execution: redacted request workbench, response wrapper, session-only credentials, local gateway profile, evidence validator, and safe background automation.
6. Close enterprise readiness: guided demo, accessibility proof, native desktop proof, cross-platform packaging, runtime checks, and spec-matrix release gates.
