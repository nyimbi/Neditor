# AI-First Platform Roadmap

This roadmap turns the AI-first direction for NEditor into 50 concrete product and engineering changes. The goal is not "AI buttons around a Markdown editor"; the goal is a document workbench where composition, editing, revision, review, distribution, and governance can all be planned, executed, audited, and handed off through visible agent workflows.

## Operating Principles

- Keep the user in control of source files, approvals, and final acceptance.
- Make agent work visible as tasks, evidence, diffs, provenance, and rollback plans.
- Treat every generated section as a draft until a human reviewer accepts it.
- Prefer deterministic local analysis before provider calls.
- Preserve local-first ownership while supporting approved external providers and publishing targets.

## 1. Intent And Document Creation

1. **Intent-first start screen**: make AI Create the default path for new business documents, with document type, audience, outcome, owner, deadline, and distribution target collected before content generation.
2. **Adaptive intake questionnaire**: generate targeted questions from the selected document type, outline, current document, missing metadata, and requested output channels.
3. **Document-type playbooks**: expand playbooks beyond current starts into proposals, board papers, policies, SOPs, release notes, strategy memos, research briefs, grant applications, and technical papers.
4. **Context completeness scoring**: score whether the user has supplied enough audience, evidence, constraints, examples, tone, and approval context for a responsible first draft.
5. **Placeholder value manager**: provide a structured table for names, dates, numbers, sources, reviewers, client values, and repeated variables rather than relying only on freeform text.

## 2. Outline-First Composition

6. **Outline planning mode as a full creation surface**: let users create, reorder, nest, rename, and delete chapters, sections, subsections, and subsubsections before body text exists.
7. **Agent outline critique**: have the agent evaluate missing sections, duplicated sections, weak sequencing, decision gaps, and audience mismatch before drafting starts.
8. **Section-by-section drafting queue**: draft one section at a time from the outline with assigned reviewers, evidence expectations, and completion criteria.
9. **Drafting depth per section**: allow sections to be marked as summary, standard, detailed, technical, legal, or executive depth.
10. **Outline-to-review handoff**: attach reviewer mandates and QA checks to each outline item before drafting, so review expectations are not invented after content exists.

## 3. Agentic Editing And Revision

11. **Instruction-to-edit planner**: convert plain language such as "tighten this for the CFO" into scoped edit, revision, evidence, and review tasks.
12. **Selection-aware revision packets**: always preserve original selected text, proposed text, change summary, risk notes, and rollback instructions.
13. **Multi-pass revision modes**: support clarity, brevity, tone, evidence, legal caution, executive summary, accessibility, and humanization passes.
14. **Meaning-drift checks**: compare original and revised text for claims, numbers, commitments, dates, caveats, and obligations that changed.
15. **Edit acceptance queue**: let users accept, reject, or revise generated edits section by section rather than applying an opaque block.

## 4. Evidence, Citations, And Source Grounding

16. **Current-document evidence scan**: detect unresolved placeholders, citation TODOs, unreviewed AI markers, unresolved comments, approval metadata gaps, and placeholder links.
17. **Evidence-derived agent tasks**: turn those scan findings into lifecycle tasks with owners, next steps, notes, and completion status.
18. **Claim inventory**: extract numbers, dates, commitments, claims, and quoted facts into a review table that can be linked to sources.
19. **Citation TODO workflow**: give users commands to add, resolve, defer, or export citation TODOs with a visible blocker state.
20. **Source pack builder**: collect pasted notes, references, files, URLs, and reviewer comments into a structured context pack for provider handoff.

## 5. Review, QA, And Humanization

21. **Named reviewer agents**: keep separate editorial, evidence, risk, citation, governance, and export reviewers with clear findings and required actions.
22. **Humanization pass**: identify generic AI phrasing, overconfident claims, repetition, bland transitions, and unnatural structure before review.
23. **Quality gates by document type**: tailor QA checks for board memos, policies, reports, papers, blog posts, newsletters, and proposals.
24. **Review comment resolution workflow**: convert unresolved comments into tasks, allow resolution notes, and block final release while comments remain unresolved.
25. **Readiness dashboard**: show an overall score plus source grounding, governance, and distribution state with concrete next actions.

## 6. Distribution And Publishing

26. **Target-aware export runbooks**: generate preflight, handoff, and evidence requirements for HTML, PDF, DOCX, PPTX, Markdown bundles, blog, Substack, LaTeX, and Google Docs.
27. **Blog publishing package**: produce Markdown, standalone HTML, plain text, assets, RSS seed, slug, excerpt, tags, canonical URL, and manifest.
28. **Substack package**: produce Substack-safe HTML, Markdown, subject line, preview text, text fallback, asset checklist, and preview evidence.
29. **Google Docs package**: produce DOCX, HTML, Markdown, text, assets, import metadata, readback checklist, and Drive URL capture.
30. **LaTeX package**: produce TeX source, bibliography hints, labels, equation checks, compile checklist, and PDF/hash evidence when available.

## 7. Provider And Model Operations

31. **Redacted provider request packages**: build system/user prompts, model settings, context, reviewer tasks, and cURL starters without persisting secrets.
32. **Session-only API key handling**: allow provider execution without storing API keys in documents or workspace history.
33. **Provider response wrapper**: wrap returned content in local provenance, review status, fingerprints, and human-review warnings before insertion.
34. **Provider evidence validator**: require source commit, clean-tree proof, request/response hashes, 2xx status, and no API-key-looking secrets for accepted live proof.
35. **Local model gateway profile**: make localhost and private network model gateways first-class options with the same governance and provenance rules.

## 8. UI And Interaction Model

36. **AI Control Center as a persistent panel**: make the control center available outside the modal so users can monitor readiness while editing.
37. **Agent task board filtering**: filter lifecycle tasks by lane, status, owner, section, target, and evidence type.
38. **Command palette AI routing**: let users type natural-language commands and route them to Docs Live, AI Paste, review, export, outline, or provider handoff.
39. **Hover help everywhere**: ensure every button, icon, and disabled action explains what it does and why it may be unavailable.
40. **Guided demo system**: walk new users through AI creation, outline planning, section drafting, provider handoff, review, and export evidence using real app actions.

## 9. Governance, Audit, And Release Control

41. **Run history with fingerprints**: preserve generated and applied agent runs with source, instruction, context, and output fingerprints.
42. **Lifecycle task persistence**: keep queued, in-progress, needs-review, complete, blocked, and note state in local workspace history.
43. **Approval metadata enforcement**: warn or block distribution when status, reviewer, approvedAt, owner, or release target metadata is missing.
44. **Rollback planning**: attach rollback steps to replacement, selection edit, and append-packet modes.
45. **Release evidence bundle**: export manifests, reviewer notes, provider hashes, QA results, and approval metadata as a durable audit package.

## 10. Verification, Accessibility, And Enterprise Readiness

46. **AI runtime readiness checks**: verify secure context, speech recognition, microphone permission, and clipboard capability without storing audio or clipboard content.
47. **Accessibility-first agent workflows**: keep modals labeled, focus-managed, keyboard-operable, screen-reader-friendly, and reduced-motion aware.
48. **Native desktop workflow proof**: keep real Tauri webview smoke coverage for file operations, menus, Docs Live, review, outline, export, and native commands.
49. **Cross-platform evidence plan**: require Windows, Linux, and macOS proof for packaging, exports, file watching, native menus, and provider/runtime checks.
50. **Spec completion matrix as release gate**: keep `docs/spec-completion-matrix.md` current and treat remaining partial items as explicit release risks until verified.

## Near-Term Implementation Order

1. Finish local deterministic agent capabilities first: evidence scans, lifecycle tasks, outline critique, claim inventory, and persistent control-center surfaces.
2. Harden provider execution second: request packages, response wrappers, evidence validators, and local-model gateway profiles.
3. Deepen distribution third: Google Docs, Substack, blog, LaTeX, and release evidence bundles with target-specific manifests.
4. Close platform proof last: native desktop coverage, accessibility sign-off, real-device voice evidence, and cross-platform release checks.
