export interface AcademicTemplate {
	id: string
	category: string
	label: string
	description: string
	content: string
}

export const ACADEMIC_TEMPLATES: AcademicTemplate[] = [
	{
		id: "journal-article",
		category: "Academic Writing",
		label: "Journal Article",
		description: "Full journal article shell (IMRaD structure)",
		content: `---
title: ""
authors:
  - name: ""
    affiliation: ""
    orcid: ""
    email: ""
    corresponding: true
abstract: ""
keywords: []
date: ""
journal: ""
doi: ""
status: draft
---

# {{title}}

## Abstract

**Background:**

**Methods:**

**Results:**

**Conclusions:**

**Keywords:**

---

## 1. Introduction



## 2. Methods

### 2.1 Study Design


### 2.2 Data Collection


### 2.3 Statistical Analysis


## 3. Results



## 4. Discussion



## 5. Conclusion



## Acknowledgements



## Funding

This work was supported by [funding source and grant number].

## Data Availability Statement

The data that support the findings of this study are available from [source] upon reasonable request.

## Conflict of Interest

The authors declare no competing interests.

## References

\`\`\`bibtex

\`\`\`
`,
	},
	{
		id: "structured-abstract",
		category: "Academic Writing",
		label: "Structured Abstract (IMRaD)",
		description: "Background / Methods / Results / Conclusions abstract block",
		content: `## Abstract

**Background:** [State the problem or gap in knowledge this study addresses.]

**Methods:** [Describe the study design, participants, interventions, and outcome measures.]

**Results:** [Summarise the main findings with key statistics.]

**Conclusions:** [State the main conclusion and its implications.]

`,
	},
	{
		id: "lab-notebook-entry",
		category: "Lab Notebook",
		label: "Lab Notebook Entry",
		description: "Structured experiment log with protocol, observations, and results",
		content: `---
title: "Experiment: "
date: ""
experimenter: ""
protocol-version: "1.0"
notebook: ""
tags: [experiment]
status: draft
---

# Experiment: {{title}}

**Date:** {{date}}
**Experimenter:** {{experimenter}}

---

## Hypothesis



## Materials

| Item | Quantity | Supplier | Lot # |
|------|----------|----------|-------|
|      |          |          |       |

## Protocol

1.
2.
3.

## Safety Notes



## Observations



## Raw Data

\`\`\`csv
Time,Value,Notes

\`\`\`

## Results



## Discussion



## Conclusion



## Next Steps

- [ ]

## References

`,
	},
	{
		id: "nih-specific-aims",
		category: "Grant Applications",
		label: "NIH Specific Aims",
		description: "NIH Specific Aims page (~1 page limit)",
		content: `---
title: "NIH Specific Aims — [Project Title]"
pi: ""
institution: ""
funding-opportunity: ""
date: ""
status: draft
---

# Specific Aims

[Opening paragraph — state the problem, knowledge gap, and long-term goal. 2–3 sentences. Conclude with: "The **overall objective** of this application is to…"]

**Central Hypothesis:** [State your central hypothesis clearly and explain the basis for it.]

**Rationale:** [Explain why this project matters and what gap it fills.]

---

**Aim 1:** [Title of Aim 1]

[2–3 sentences describing the aim, approach, and expected outcome.]

**Aim 2:** [Title of Aim 2]

[2–3 sentences describing the aim, approach, and expected outcome.]

**Aim 3 (if applicable):** [Title of Aim 3]

[2–3 sentences describing the aim, approach, and expected outcome.]

---

**Innovation:** [1–2 sentences on what is novel about this approach.]

**Impact:** [1–2 sentences on the expected outcomes and broader significance. End with expected deliverable.]

`,
	},
	{
		id: "nsf-project-summary",
		category: "Grant Applications",
		label: "NSF Project Summary",
		description: "NSF Project Summary (Overview, Intellectual Merit, Broader Impacts — 1 page)",
		content: `---
title: "NSF Project Summary — [Project Title]"
pi: ""
institution: ""
program: ""
date: ""
status: draft
---

# Project Summary

## Overview

[2–3 sentences describing the project scope, objectives, and general approach.]

## Intellectual Merit

[Describe the potential to advance knowledge and understanding within its own field or across different fields. Address the five review criteria: significance, investigator qualifications, innovation, approach, and environment. ~150 words.]

## Broader Impacts

[Describe the potential to benefit society and contribute to the achievement of specific, desired societal outcomes. Include education and training, diversity, data sharing, and societal benefit. ~150 words.]

`,
	},
	{
		id: "eu-horizon-summary",
		category: "Grant Applications",
		label: "EU Horizon Section",
		description: "EU Horizon Europe evaluation criteria: Excellence, Impact, Implementation",
		content: `---
title: "EU Horizon Proposal — [Project Title]"
acronym: ""
call: ""
coordinator: ""
date: ""
status: draft
---

# {{title}}

## 1. Excellence

### 1.1 Objectives and Ambition

[Describe the project's specific objectives. Explain the ambition beyond the state of the art.]

### 1.2 Methodology

[Describe the research and innovation methodology, including the approach to validation and demonstration.]

### 1.3 Originality and Innovative Aspects

[Explain what is novel and how the project goes beyond current state of the art.]

## 2. Impact

### 2.1 Expected Outcomes and Impacts

[Describe the expected scientific, economic, and societal impacts.]

### 2.2 Measures to Maximise Impact

[Describe the dissemination, exploitation, and communication plan.]

## 3. Implementation

### 3.1 Work Plan

[Describe the work packages, milestones, and deliverables.]

### 3.2 Consortium

[Describe the consortium composition and each partner's role.]

### 3.3 Resources and Budget

[Justify the resources requested.]

`,
	},
	{
		id: "peer-review-response",
		category: "Academic Writing",
		label: "Peer Review Response Letter",
		description: "Structured response to reviewer comments with point-by-point replies",
		content: `---
title: "Response to Reviewers — [Manuscript Title]"
manuscript-id: ""
journal: ""
date: ""
status: draft
---

# Response to Reviewers

**Manuscript:** {{title}}
**Manuscript ID:** {{manuscript-id}}
**Journal:** {{journal}}

---

Dear Editor and Reviewers,

We thank the editors and reviewers for their careful reading of our manuscript and their constructive comments. We have revised the manuscript accordingly. Below, we provide a point-by-point response to each comment.

*Reviewer comments are shown in italics. Our responses follow each comment.*

---

## Reviewer 1

**Comment 1.1:**
*[Paste reviewer comment here]*

**Response:**
[Your response here. If text was changed in the manuscript, quote the new text: "We have revised the text to read: '…'"]

**Change:** [Describe the specific change made, including page/line numbers if available.]

---

**Comment 1.2:**
*[Paste reviewer comment here]*

**Response:**


**Change:**

---

## Reviewer 2

**Comment 2.1:**
*[Paste reviewer comment here]*

**Response:**


**Change:**

---

We hope these revisions address the reviewers' concerns. We look forward to your decision.

Sincerely,

[Corresponding author name]
[Institution]
[Email]

`,
	},
	{
		id: "author-metadata-block",
		category: "Academic Writing",
		label: "Author Metadata Block",
		description: "Author affiliations, ORCID, corresponding author, funding, COI",
		content: `---
authors:
  - name: ""
    affiliation: "1"
    orcid: "0000-0000-0000-0000"
    email: ""
    corresponding: true
  - name: ""
    affiliation: "2"
    orcid: ""
    corresponding: false
affiliations:
  - id: "1"
    name: ""
    address: ""
    country: ""
  - id: "2"
    name: ""
    address: ""
    country: ""
funding:
  - agency: ""
    grant: ""
    recipient: ""
coi: "The authors declare no competing interests."
data-availability: "Data are available from the corresponding author upon reasonable request."
ethics: ""
---
`,
	},
	{
		id: "data-availability",
		category: "Academic Writing",
		label: "Data Availability Statement",
		description: "Standardised data availability statement variants",
		content: `## Data Availability Statement

<!-- Choose one variant and delete the others -->

**Open access variant:**
The datasets generated and/or analysed during the current study are available in the [repository name] repository, [DOI or URL].

**On-request variant:**
The datasets generated during and/or analysed during the current study are not publicly available due to [reason] but are available from the corresponding author on reasonable request.

**Restricted variant:**
The data that support the findings of this study are available from [third party name] but restrictions apply to the availability of these data, which were used under licence for the current study, and so are not publicly available. Data are however available from the authors upon reasonable request and with permission of [third party name].

**No data variant:**
Data sharing is not applicable to this article as no datasets were generated or analysed during the current study.

`,
	},
	{
		id: "systematic-review",
		category: "Academic Writing",
		label: "Systematic Review / Literature Review",
		description: "PRISMA-aligned systematic review structure",
		content: `---
title: "Systematic Review: [Topic]"
protocol-registration: ""
date: ""
authors: []
status: draft
---

# {{title}}

## Abstract

**Background:**

**Methods:** We searched [databases] from [date range]. Studies were included if [inclusion criteria].

**Results:** We identified [N] studies. [Summary of findings.]

**Conclusions:**

## 1. Introduction

### 1.1 Background

### 1.2 Review Questions

**Primary question:**

**Secondary questions:**
1.

## 2. Methods

### 2.1 Protocol Registration

This review was registered with PROSPERO (registration number: {{protocol-registration}}).

### 2.2 Eligibility Criteria

**Inclusion criteria:**
- Study design:
- Population:
- Intervention/exposure:
- Comparator:
- Outcomes:
- Language:

**Exclusion criteria:**
-

### 2.3 Search Strategy

Databases searched: [PubMed, Embase, Cochrane, etc.]
Date range:
Search terms:

\`\`\`
[Paste full search string here]
\`\`\`

### 2.4 Study Selection

### 2.5 Data Extraction

### 2.6 Quality Assessment

### 2.7 Synthesis

## 3. Results

### 3.1 Search Results

### 3.2 Study Characteristics

### 3.3 Risk of Bias

### 3.4 Findings

## 4. Discussion

## 5. Conclusions

## References

\`\`\`bibtex

\`\`\`
`,
	},
]

export function academicTemplateById(id: string): AcademicTemplate | undefined {
	return ACADEMIC_TEMPLATES.find((t) => t.id === id)
}

export function academicTemplatesByCategory(): Record<string, AcademicTemplate[]> {
	const result: Record<string, AcademicTemplate[]> = {}
	for (const t of ACADEMIC_TEMPLATES) {
		if (!result[t.category]) result[t.category] = []
		result[t.category].push(t)
	}
	return result
}
