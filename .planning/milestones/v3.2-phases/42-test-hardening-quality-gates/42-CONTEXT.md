# Phase 42: Test Hardening & Quality Gates - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Quality infrastructure and CI enforcement for the Time Pac (XROM 26) surface shipped in Phases 38–41. No new user-facing features — every deliverable is a test, meta-gate, CI verification, or README graduation. 11 requirements: TIME-QUAL-01..11.

**What this phase delivers:**

1. **Coverage hold** (TIME-QUAL-01): `hp41-core` region coverage ≥ 93% after Time Pac additions (~4636 LOC, 35 ops). Denominator dilution acknowledged per v3.1 precedent; region coverage is the primary gate.

2. **Date arithmetic accuracy suite** (TIME-QUAL-02): Oracle-verified edge cases for DATE+, DDAYS, DOW covering leap years, century boundaries, and Gregorian calendar extremes.

3. **Per-Op test-count meta-gate** (TIME-QUAL-03): ≥ 5 tests per Time Pac Op variant at CI time.

4. **Backward-compat migration test** (TIME-QUAL-04): v3.1 save file (`xrom_modules: 0b0000_0011`) loads in v3.2; `migrate_after_load` sets bit 2 → `0b0000_0111`.

5. **Free42 contamination re-verification** (TIME-QUAL-05): already extended to 18 tokens covering `time/` in Phase 38. Phase 42 re-verifies the gate holds.

6. **E2E smoke extension** (TIME-QUAL-06): WebdriverIO spec extended with one Time Pac workflow on Ubuntu-only (`ci-gui.yml::e2e-linux`).

7. **function_matrix_parity 4-pool extension** (TIME-QUAL-07): already extended in Phase 39; Phase 42 re-verifies the 4-pool partition test passes.

8. **XROM shadowing 3-module cross-check** (TIME-QUAL-08): already extended in Phase 39 (D-39.14); Phase 42 re-verifies.

9. **README hard-claim graduation** (TIME-QUAL-09): "feature-complete per Owner's Manual 00041-90035" — same D-30.9 → D-32.5 → D-37.11 graduation pattern.

10. **Stopwatch timing accuracy** (TIME-QUAL-10): ±10ms over tested window; Instant-based monotonic clock.

11. **Alarm past-due detection latency** (TIME-QUAL-11): fires within one dispatch cycle.

**In scope:**
- `hp41-core/tests/xrom_op_test_count.rs` (NEW, unified — replaces `math1_op_test_count.rs` + `stat1_op_test_count.rs`)
- `hp41-core/tests/lint_xrom_assertions.rs` (NEW, unified — replaces `lint_math1_assertions.rs` + `lint_stat1_assertions.rs`)
- Date arithmetic accuracy cases (oracle-verified, exact-match)
- `hp41-core/tests/time_backward_compat.rs` (v3.1 save-file migration test)
- `hp41-core/tests/fixtures/v31-autosave.json` (v3.1 fixture with `xrom_modules: 3`)
- Coverage-gap test files for `time/*.rs` files below 90% (as needed per measurement)
- Stopwatch timing accuracy test (short-window CI variant)
- Alarm dispatch-cycle latency test
- `hp41-gui/e2e/smoke.spec.js` Time Pac workflow extension (Ubuntu-only)
- Quality-gate programmatic verification (`just ci` + `just gui-ci` + coverage gates pass)
- README hard-claim graduation

**Out of scope (explicit):**
- Any `hp41-core/src/ops/time/*.rs` implementation changes — Time Pac algorithms are sealed since Phase 38
- Any `hp41-core/src/ops/math1/` or `hp41-core/src/ops/stat1/` changes
- Any `hp41-cli/src/` changes
- Any `hp41-gui/src-tauri/src/` changes beyond E2E spec extension
- Any `hp41-gui/src/` (React frontend) changes beyond E2E spec extension
- `docs/` changes beyond potential hp41-time-divergences.md entry for bounded stopwatch test
- New XROM modules or Op variants

</domain>

<decisions>
## Implementation Decisions

### Meta-gate Unification (TIME-QUAL-03, TIME-QUAL-05, TIME-QUAL-08)
- **D-42.1:** **Unify meta-gate files across all three XROM modules.** Create `xrom_op_test_count.rs` scanning Math 1 + Stat 1 + Time module variants; create `lint_xrom_assertions.rs` scanning assertion discipline across all three module test trees. Delete the old `math1_op_test_count.rs`, `stat1_op_test_count.rs`, `lint_math1_assertions.rs`, `lint_stat1_assertions.rs` in the same plan. This fulfills the Phase 37 deferred commitment ("if a third XROM module lands, consider unifying"). Advantage Pac (v3.3) benefits for free.
- **D-42.2:** **XROM shadowing test (`xrom_shadowing.rs`) remains as-is** — it already covers all 3 modules per D-39.14. No changes needed. Phase 42 re-verifies it passes.
- **D-42.3:** **`function_matrix_parity.rs` already covers 4 pools** (cv + math1 + stat1 + time) per Phase 39. Phase 42 re-verifies it passes. No changes needed.

### Date Arithmetic Accuracy (TIME-QUAL-02)
- **D-42.4:** Date accuracy cases use exact-match assertions (no tolerance) since calendar arithmetic is deterministic integer math. Whether these extend `numerical_accuracy.rs` or live in a separate file is Claude's discretion — see D-42.10.
- **D-42.5:** Oracle source for date arithmetic correctness is Claude's discretion — see D-42.11.
- **D-42.6:** Edge case allocation across DATE+/DDAYS/DOW is Claude's discretion — see D-42.12. TIME-QUAL-02 explicitly requires: leap years (Feb 29 2000, 2100, 2400), century boundaries, Gregorian calendar start (Oct 15 1582), and the DATE+/DDAYS/DOW trio.

### Stopwatch Timing Accuracy (TIME-QUAL-10)
- **D-42.7:** Stopwatch timing test approach is Claude's discretion — see D-42.13. TIME-QUAL-10 requires ±10ms over a test window. `std::time::Instant` provides OS-level monotonic clock guarantees.

### Backward Compatibility (TIME-QUAL-04)
- **D-42.8:** Create `hp41-core/tests/fixtures/v31-autosave.json` — a CalcState JSON with `xrom_modules: 3` (v3.1 default — Math 1 + Stat 1 enabled, Time not yet enabled). Test loads it, calls `migrate_after_load`, asserts `state.xrom_modules == 0b0000_0111` (all three modules enabled), verifies clock/alarm/stopwatch state defaults cleanly. Mirrors Phase 37's `v30-autosave.json` pattern.

### Plan Wave Structure
- **D-42.9:** **5-6 plans (lean).** Wave 1: Unified meta-gates (create xrom_op_test_count.rs + lint_xrom_assertions.rs, delete old files). Wave 2: Coverage measurement + gap closure. Wave 3: Date accuracy suite + stopwatch timing + alarm latency. Wave 4: Backward-compat + E2E smoke + README graduation + final quality-gate verification. Claude slices within this structure.

### E2E Smoke Workflow (TIME-QUAL-06)
- **D-42.14:** E2E Time Pac workflow selection is Claude's discretion — see D-42.15.

### Prior Decisions (carried forward)
- **D-carried.1:** E2E smoke is Ubuntu-only (`ci-gui.yml::e2e-linux`) per D-37.2 precedent.
- **D-carried.2:** README hard-claim graduation follows D-30.9 → D-32.5 → D-37.11 cadence. Hard-claim text: "feature-complete per Owner's Manual 00041-90035."
- **D-carried.3:** `#![deny(clippy::unwrap_used)]` in `hp41-core`; test files carry `#[allow(clippy::unwrap_used)]` at file scope per established pattern.
- **D-carried.4:** Zero new runtime or dev-dependencies.
- **D-carried.5:** MSRV 1.88 unchanged.
- **D-carried.6:** Free42 contamination guard already covers `time/` tree (18 tokens, extended in Phase 38). Phase 42 re-verifies.
- **D-carried.7:** XROM shadowing already covers all 3 modules (D-39.14). Phase 42 re-verifies.
- **D-carried.8:** Coverage gap approach: measure first via `cargo llvm-cov` per-file, then write targeted tests only for files below 90% (Phase 32/37 precedent).

### Claude's Discretion
- **D-42.10 (scan scope for unified xrom_op_test_count.rs):** Claude decides whether to scan inline tests (inside `src/ops/{math1,stat1,time}/*.rs`) in addition to external `tests/` files. Time has 209 inline tests and 0 external time_* test files currently, so inline scanning is likely necessary for accurate counts. Recommendation: dual-scan all modules (inline + external).
- **D-42.11 (date accuracy oracle source):** Claude picks the most practical oracle source — Python datetime/calendar, published JDN tables (Meeus), or both. Recommendation: Python datetime for derivation convenience, with published Meeus tables as sanity-check for JDN boundary cases.
- **D-42.12 (date edge case allocation):** Claude allocates cases across DATE+/DDAYS/DOW based on algorithmic risk. TIME-QUAL-02 lists the minimum edges; Claude may add extras based on JDN formula failure modes. Recommendation: ~20-30 cases covering all listed edges plus Y2K boundary, large offsets, and known-DOW historical dates.
- **D-42.13 (stopwatch timing test approach):** Claude picks the most practical approach for CI. Recommendation: short-window CI test (1-2 seconds, ±5ms) plus a `#[ignore]` 60-second variant for manual validation. Document that the 60s ±10ms claim follows from Instant monotonicity and shorter-window proof.
- **D-42.14 (meta-gate migration — delete old files in same plan or separately):** Claude picks based on risk. Recommendation: delete in same plan (atomic — unified file replaces old files, CI catches any regressions immediately).
- **D-42.15 (E2E smoke Time Pac workflow):** Claude picks between DDAYS and DATE+ based on which exercises the most code paths with the simplest key sequence. Recommendation: DDAYS (two dates on stack → day count — exercises date parsing, JDN arithmetic, clear numeric assertion on LCD).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 37 (direct structural analog — v3.1 test hardening)
- `.planning/milestones/v3.1-phases/37-test-hardening-quality-gates/37-CONTEXT.md` — 1:1 structural template for Phase 42; D-37.1 through D-37.11 decisions; plan wave structure; deferred unification commitment that Phase 42 fulfills
- `.planning/milestones/v3.0-phases/32-test-hardening/32-CONTEXT.md` — v3.0 test hardening; original meta-gate pattern (math1_op_test_count.rs, lint_math1_assertions.rs)

### Phase 38/39 Context (the surface Phase 42 tests)
- `.planning/phases/38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core/38-CONTEXT.md` — Phase 38 decisions; Time Module core implementation; CalcState fields; Free42 guard extension
- `.planning/phases/39-hp41-cli-cli-integration-live-display/39-CONTEXT.md` — Phase 39 decisions; XROM shadowing 3-module extension (D-39.14); function_matrix_parity 4-pool extension

### Existing test infrastructure (Phase 42 replaces or extends)
- `hp41-core/tests/math1_op_test_count.rs` — TO BE REPLACED by unified `xrom_op_test_count.rs`
- `hp41-core/tests/stat1_op_test_count.rs` — TO BE REPLACED by unified `xrom_op_test_count.rs`
- `hp41-core/tests/lint_math1_assertions.rs` — TO BE REPLACED by unified `lint_xrom_assertions.rs`
- `hp41-core/tests/lint_stat1_assertions.rs` — TO BE REPLACED by unified `lint_xrom_assertions.rs`
- `hp41-core/tests/numerical_accuracy.rs` — 791-case suite; may be extended with date cases (D-42.10)
- `hp41-core/tests/stat1_backward_compat.rs` — v3.0→v3.1 migration test (pattern template for v3.1→v3.2)
- `hp41-core/tests/fixtures/v30-autosave.json` — v3.0 fixture (pattern template for v3.1 fixture)
- `hp41-core/tests/xrom_shadowing.rs` — already covers 3 modules; re-verify only
- `hp41-cli/tests/function_matrix_parity.rs` — already covers 4 pools; re-verify only
- `hp41-gui/e2e/smoke.spec.js` — existing E2E spec; extend with Time Pac workflow
- `hp41-gui/wdio.conf.cjs` — WebdriverIO config

### Time module source (coverage measurement targets)
- `hp41-core/src/ops/time/alarm.rs` — alarm catalog + check_alarms + event_buffer drain
- `hp41-core/src/ops/time/alpha_time.rs` — ATIME/ATIME24/ADATE
- `hp41-core/src/ops/time/clock.rs` — TIME/DATE/SETIME/SETDATE/CLKT/CLKTD/CLOCK/CLK12/CLK24/CORRECT/T+X
- `hp41-core/src/ops/time/date_arith.rs` — DATE+/DDAYS/DOW/DMY/MDY + JDN formula
- `hp41-core/src/ops/time/mod.rs` — SETAF/RCLAF + module dispatch
- `hp41-core/src/ops/time/modal.rs` — TimeStep enum + modal dispatch
- `hp41-core/src/ops/time/stopwatch.rs` — RUNSW/STOPSW/RCLSW/SETSW/SW/SWPT/STPW

Total: 7 source files, ~4636 LOC, 209 inline tests.

### XROM registration (variant detection source for unified meta-gate)
- `hp41-core/src/ops/math1/xrom.rs` — `MATH_1.ops` (45 entries), `STAT_1.ops` (26 entries), `TIME_MODULE.ops` (35 entries)

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — TIME-QUAL-01..11 (11 requirements mapped to Phase 42)
- `.planning/ROADMAP.md` — Phase 42 goal, success criteria (5 items), depends-on

### Scripts and CI
- `scripts/check-free42-contamination.sh` — 18-token grep (Phase 38 extended); re-verify
- `.github/workflows/ci.yml` — CLI + license-audit CI
- `.github/workflows/ci-gui.yml` — 3-OS matrix + `e2e-linux` job
- `justfile` — `just ci`, `just gui-ci` recipes

### Quality Gates (current baselines)
- `CLAUDE.md` `## Quality Gates` table — targets and current values
- `.planning/STATE.md` `## Performance Metrics` — v3.1 baselines (line coverage 93.91%, region coverage 95.84%, numerical accuracy 98.86%)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`math1_op_test_count.rs`** — structural template for the unified `xrom_op_test_count.rs`. Key change: scan all 3 module sections in `xrom.rs` (MATH_1.ops, STAT_1.ops, TIME_MODULE.ops) instead of one. Glob all `{math1,stat1,time}_*.rs` test files + inline test modules in `src/ops/{math1,stat1,time}/*.rs`.
- **`lint_math1_assertions.rs`** — structural template for unified `lint_xrom_assertions.rs`. Same two lints (`no_decimal_assert_eq` + `no_manual_tolerance_pattern`); scan scope extended to all three module test trees.
- **`stat1_backward_compat.rs`** — direct template for `time_backward_compat.rs`. Pattern: load fixture JSON, call `migrate_after_load`, assert `xrom_modules` bits, verify Time ops resolve.
- **`hp41-gui/e2e/smoke.spec.js`** — existing spec with keyboard interaction pattern. `data-key-id` clicks, `data-text` LCD assertions, `extractErrMessage` helper.
- **`numerical_accuracy.rs` `case!` macro** — if date cases extend this file, reuse the macro with `tol: 0.0` for exact match.

### Established Patterns
- **Meta-gate-before-coverage (v3.0 Phase 32):** meta-gates land BEFORE coverage-gap tests so new test files are scanned immediately. Phase 42 follows same wave ordering.
- **Coverage-measurement-then-targeted-gap-closure (v3.0/v3.1):** run `cargo llvm-cov` per-file, identify files below 90%, write targeted tests for those files only.
- **`#[allow(clippy::unwrap_used)]` at file scope** in all `hp41-core/tests/*.rs` files.
- **Fixture-based backward-compat tests:** `v20-autosave.json` (v2.0), `v30-autosave.json` (v3.0) exist. Phase 42 adds `v31-autosave.json`.
- **E2E spec shape:** describe block, `browser.execute` + `$('[data-key-id="..."]').click()` + `$('[data-testid="lcd-display"]').getAttribute('data-text')`.

### Integration Points
- **`xrom.rs` lines 197-242 (TIME_MODULE.ops):** variant name extraction source for unified meta-gate
- **`hp41-core/tests/` directory:** new unified files + coverage-gap files + backward-compat + timing test
- **`hp41-core/tests/fixtures/`:** new `v31-autosave.json`
- **`hp41-gui/e2e/smoke.spec.js`:** new `it()` or `describe()` block for Time Pac workflow

</code_context>

<specifics>
## Specific Ideas

- **Unified meta-gate variant detection:** scan `xrom.rs` for all three `XromModule` ops slices. Parse `Some(Op::Math1*)`, `Some(Op::Stat1*)`, `Some(Op::Time*)` variant names. Count test mentions across inline + external test files per module. Report per-module and aggregate. ≥ 5 mentions per variant.

- **v3.1 save-file fixture:** serialize a `CalcState` with `xrom_modules: 3` (v3.1 default), `rand_seed` set to a known value (v3.1 field), no time/alarm/stopwatch state. The backward-compat test loads it, calls `migrate_after_load`, asserts:
  - `state.xrom_modules == 0b0000_0111` (Time bit activated)
  - `state.time_offset_secs == 0` (default)
  - `state.alarms.is_empty()` (default)
  - `state.stopwatch_mode == StopwatchMode::Idle` (default)
  - `state.rand_seed` preserved (v3.1 field survives v3.2 migration)

- **Recommended plan wave structure (D-42.9):**
  - **Wave 1 (Plan 42-01):** Unified meta-gates — create `xrom_op_test_count.rs` + `lint_xrom_assertions.rs`, delete 4 old files, verify CI green
  - **Wave 2 (Plan 42-02):** Coverage measurement + gap closure — run per-file coverage, write targeted tests for time/*.rs files below 90%
  - **Wave 3 (Plan 42-03):** Date accuracy suite + stopwatch timing + alarm latency tests
  - **Wave 4 (Plan 42-04):** Backward-compat test (v3.1 fixture) + E2E smoke + Free42/XROM re-verification + quality-gate graduation + README hard-claim

</specifics>

<deferred>
## Deferred Ideas

- **CI-automated coverage gate** — currently coverage is measured locally; a CI step that automatically gates on ≥ 93% region is a future ergonomic improvement but not required for v3.2 ship
- **Cross-platform E2E smoke** — Ubuntu-only per D-carried.1; macOS/Windows E2E would require headless display setup
- **Signed binary releases** — deferred to post-v3.2 per PROJECT.md

None are scope creep from this discussion — all are pre-existing deferred items from prior milestones.

</deferred>

---

*Phase: 42-test-hardening-quality-gates*
*Context gathered: 2026-05-25*
