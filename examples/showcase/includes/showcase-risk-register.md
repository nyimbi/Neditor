# Risk Register Include {#sec:risk-register}

Table: Showcase risk register {#tbl:risk-register}

| Risk | Severity | Mitigation | Owner |
| --- | --- | --- | --- |
| Unsourced AI-generated claim enters a client deliverable | High | Citation TODOs, source-library audit, human review gate | Review owner |
| Export package misses approval metadata | Medium | Export readiness gate and release metadata scaffold | Release manager |
| Long document loses structural coherence | Medium | Outline mode, document outline library, section work queue | Lead writer |
| Technical transform depends on missing local engine | Medium | Transform handler setup, trusted-engine diagnostics, static fallback | Document engineer |

```pikchr
box "Draft"; arrow "review"; diamond "Ready?"; arrow "export"; box "Delivery"
```
