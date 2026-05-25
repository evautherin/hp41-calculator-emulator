# Deferred Items — Phase 42

## Pre-existing Test Failures (Out of Scope for Plan 42-04)

### hp41-cli/tests/phase25_pending_input.rs::test_tone_prompt_auto_dispatch

**Status:** Pre-existing failure on `develop` branch before Plan 42-04 execution.
**Symptom:** "TONE 5 must push a TONE event; got events=[]" — panic at line 435.
**Discovery:** Encountered during Task 3 quality-gate verification (`cargo test -p hp41-cli`).
**Not caused by:** Any Plan 42-04 changes (README.md, key_coverage.rs, time_backward_compat.rs, smoke.spec.js).
**Recommendation:** Investigate TONE event_buffer draining in a future plan. The test expects a TONE event in the event_buffer after dispatching a TONE prompt op, but the buffer is empty — likely a timing issue or a Phase 39 drain-call regression.

