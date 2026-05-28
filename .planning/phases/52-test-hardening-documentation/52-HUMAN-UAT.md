---
status: partial
phase: 52-test-hardening-documentation
source: [52-VERIFICATION.md, 52-REVIEW.md]
started: 2026-05-28
updated: 2026-05-28
---

## Current Test

[awaiting human decision on help-text accuracy fixes]

## Tests

### 1. EMREG `?`-overlay description is factually correct
expected: The `EmReg` entry in `docs/hp41-xmem-functions.json` describes what `op_emreg` actually does — recall register N (N from X) from the **active** X-MEM DATA file and push the value to X.
result: [pending]
detail: Current description/example/notes wrongly describe EMREG as a register-*count* function ("return number of registers used by X-MEM file named by ALPHA in X"), conflating it with EMROOM. Implementation is correct; only the human-readable help fields are wrong. (Source: 52-VERIFICATION.md)

### 2. SAVED/GETD `?`-overlay description register count is accurate
expected: The `SaveD`/`GetD` entries do not hardcode "100 registers"; they reflect that the op saves/loads `state.regs`, which is dynamically sized by `SIZE` (1–319).
result: [pending]
detail: Current descriptions hardcode "100 registers", which is incorrect for users who changed register allocation. (Source: 52-REVIEW.md WR-03)

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
