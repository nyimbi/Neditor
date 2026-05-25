---
title: Customer Success Automation Proposal
version: 1.0.0
status: approved
approvedBy: Commercial Director
approvedAt: 2026-05-20T13:00:00Z
classification: proposal
targetPersona:
  - Consultants
  - Product and engineering teams
positioning:
  model: local-first document-file workbench
  sourceOfTruth: Markdown source file
  cloudSync: false
toc: true
brand:
  name: Meridian Advisory
  color: "#7C3AED"
---

# Customer Success Automation Proposal

[TOC]

## Scope

The proposal covers onboarding automation, health scoring, and executive
renewal reporting.

```calc
implementation = 45000
training = 12000
support = 18000
total = implementation + training + support
discount = total * 0.10
net = total - discount
```

Total investment after discount is {{=net | currency}}.

| Workstream | Cost |
| --- | ---: |
| Implementation | {{=implementation | currency}} |
| Training | {{=training | currency}} |
| Support | {{=support | currency}} |
| Net total | {{=net | currency}} |

```csv caption="Quarterly rollout budget"
Quarter,Services,Travel,Total
Q1,28000,3000,=B1+C1
Q2,21000,2500,=B2+C2
Q3,14000,1500,=B3+C3
```
