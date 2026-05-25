# Phase 40: Documentation & ADRs - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 40 documents all Time Pac (v3.2) architectural decisions, emulator divergences, and function reference material. Deliverables: fourth `docs-matrix` invocation (`docs/hp41-time-function-matrix.md`), divergences catalog (`docs/hp41-time-divergences.md`), ADRs for key Phase 38/39 decisions, README v3.2 soft-claim, CLAUDE.md `### v3.2 additions` block, and `docs/architecture-history.md` v3.2 narrative section. Follows the exact cadence established by Phase 30 (Math 1 docs) and Phase 35 (Stat 1 docs).

</domain>

<decisions>
## Implementation Decisions

### Function Matrix Extension (TIME-DOC-01)
- **D-40.1:** `scripts/docs-matrix/src/main.rs` gains a 4th basename dispatch branch: `"hp41-time-functions.json"` → title "# HP-41C Time Pac Function Matrix", src "`docs/hp41-time-functions.json`". Identical to the Phase 35 D-35.1 single-line addition pattern.
- **D-40.2:** `justfile` `docs-matrix` recipe gains 4th invocation: `docs/hp41-time-functions.json docs/hp41-time-function-matrix.md`. `docs-matrix-check` gains corresponding 4th diff block.

### Divergences Document (TIME-DOC-02)
- **D-40.3:** `docs/hp41-time-divergences.md` follows the 5-field entry shape (D-30.5 / D-35 template): OM citation, Our behavior, OM behavior, Rationale, See. Stable `D-40-NN` identifiers. Pitfall 18 citation discipline.
- **D-40.4:** Three-bucket structure parallel to Stat 1: "OM Divergences" (where we deviate from HP 00041-90036), "Emulator Extensions" (features not in the OM), "Behavioral Policies" (implementation choices that affect precision or state semantics). Known entries from Phase 38 CONTEXT:
  - Accuracy factor no-op (CORRECT/SETAF store value but CORRECT does nothing — host NTP clock)
  - Host system clock backing (SystemTime vs. crystal oscillator)
  - Stopwatch freeze-on-save (Instant cannot serialize; user must RUNSW to resume)
  - Interrupting control alarm deferral (D-38.4 — re-entrancy hazard)
  - Centisecond stopwatch resolution (matches OM but via different mechanism)
  - Any additional divergences discovered during Phase 38/39 execution should be included

### ADR Selection (TIME-DOC-03)
- **D-40.5:** Three standalone ADRs for v3.2, following D-30.6 long-form template with `## Alternatives Considered` section:
  1. `v3.2-001-clock-access-pattern.md` — Direct SystemTime in hp41-core (D-38.1). Alternatives: trait injection, frontend callback, `no_std` clock abstraction.
  2. `v3.2-002-live-display-architecture.md` — Pull-on-redraw pattern (D-38 carried.7 / D-39.1-2). Alternatives: async timer, push-from-core, polling thread.
  3. `v3.2-003-alarm-catalog-design.md` — Vec<AlarmEntry> on CalcState with event_buffer drain (D-38.8-11). Alternatives: separate alarm state struct, channel-based notification, trait-based alarm handler.
- **D-40.6:** Stopwatch state (D-38.6-7) and date parsing (D-carried.2) are documented in architecture-history.md but do NOT warrant standalone ADRs — they follow existing CalcState patterns without novel alternatives.

### README Soft-Claim (TIME-DOC-04)
- **D-40.7:** Add v3.2 soft-claim under `## Features` following the v3.1 `D-35.3` pattern: one bullet summarizing Time Pac capability. The hard-claim ("feature-complete per Owner's Manual HP 00041-90036") is deferred to Phase 42 quality gate graduation (same cadence as D-30.9 → D-32.5 and D-35.3 → D-37.11).
- **D-40.8:** Release table gains a `v3.2` row (date TBD — inserted after Phase 42 ships). For now, soft-claim in Features section only.

### CLAUDE.md v3.2 Additions (TIME-DOC-05)
- **D-40.9:** New `#### Phase 38 — ...` and `#### Phase 39 — ...` subsections under a new `### v3.2 additions (Time Pac Emulation, Phases 38–42)` block. Detail level matches v3.1 per-phase summaries. Phase 40 (this phase) gets a brief summary. Phases 41-42 get stubs ("IN PROGRESS" or TBD).
- **D-40.10:** Frozen invariants addendum: document the Phase 38 CalcState additions (time_offset_secs, stopwatch fields, alarm_catalog, clock_display_mode) with their serde shapes, and the XROM bit-2 arm (TIME_MODULE, XROM 26).

### Architecture History (TIME-DOC-06)
- **D-40.11:** New `## v3.2 additions (Time Pac Emulation, Phases 38–42 — 41–42 IN PROGRESS)` section in `docs/architecture-history.md`. Per-phase narrative for Phases 38 and 39 (shipped). Stubs for Phases 40-42.

### Claude's Discretion
- Exact wording of README soft-claim bullet (follow v3.1 tone)
- Order of divergence entries within each bucket (chronological by D-40-NN or grouped by function)
- Level of detail in architecture-history.md per-phase narrative (use v3.1 Phase 33/34 as calibration — ~20-40 lines per phase)
- Whether to number ADRs starting at v3.2-001 (yes — consistent with v3.0-001 and v3.1-001)
- Exact `## Alternatives Considered` content in ADRs — quote Phase 38 CONTEXT decisions verbatim per D-30.7

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 38/39 Context (source material for documentation)
- `.planning/phases/38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core/38-CONTEXT.md` — All D-38.* decisions (clock access, stopwatch state, alarm catalog, control alarm deferral, date parsing)
- `.planning/phases/39-hp41-cli-cli-integration-live-display/39-CONTEXT.md` — All D-39.* decisions (live display rendering, stopwatch keyboard mode, alarm draining, JSON canonical data, help overlay)

### v3.1 Documentation Precedent (pattern to follow)
- `docs/hp41-stat1-divergences.md` — 12-entry divergences catalog, 3-bucket structure, D-35-NN numbering (most recent precedent)
- `docs/hp41-math1-divergences.md` — First divergences catalog (D-30-NN numbering)
- `docs/adr/v3.1-001-rng-state-placement.md` — Long-form ADR template with Alternatives Considered (follow this format)
- `docs/adr/v3.0-001-op-strategy.md` — First v3.x ADR (title/status/owner/requirement-refs/downstream-consumer header pattern)

### Function Matrix Infrastructure
- `scripts/docs-matrix/src/main.rs` — Binary that generates matrices; needs 4th basename branch
- `justfile` — `docs-matrix` and `docs-matrix-check` recipes; need 4th invocation/diff block
- `docs/hp41-time-functions.json` — Already exists (Phase 39 D-39.9); input for matrix generation

### Files to Update
- `README.md` — Features section (soft-claim bullet) + release table (v3.2 row after Phase 42)
- `CLAUDE.md` — `### v3.2 additions` block under Frozen Invariants
- `docs/architecture-history.md` — New `## v3.2 additions` narrative section

### Research & Requirements
- `.planning/REQUIREMENTS.md` — TIME-DOC-01..06 (6 requirements for this phase)
- `.planning/ROADMAP.md` — Phase 40 success criteria (3 items)
- `.planning/research/SUMMARY.md` — v3.2 research summary (background for ADR Alternatives Considered)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripts/docs-matrix/src/main.rs:70-76` — basename dispatch branch; add 4th `else if` for `hp41-time-functions.json`
- `docs/hp41-stat1-divergences.md` — Copy structure verbatim as template for `hp41-time-divergences.md`
- `docs/adr/v3.1-001-rng-state-placement.md` — Copy header format for v3.2 ADRs
- CLAUDE.md `#### Phase 33` block — Copy structure for `#### Phase 38` block

### Established Patterns
- Basename dispatch in docs-matrix binary: string match on input filename → (title, src) tuple
- Justfile recipe pattern: 4 invocations of same binary with different in/out paths
- ADR filename: `v{major}.{minor}-{NNN}-{kebab-slug}.md` (e.g., `v3.2-001-clock-access-pattern.md`)
- Divergence numbering: `D-{phase}-{NN}` where phase = authoring phase (40 for this phase)
- README soft-claim pattern: one descriptive bullet, no "feature-complete" claim until quality gate

### Integration Points
- `scripts/docs-matrix/src/main.rs:70` — Insert new branch before fallback `else`
- `justfile:docs-matrix` — Append 4th invocation after stat1 line
- `justfile:docs-matrix-check` — Append 4th diff after stat1 check
- `docs/adr/` — New files: `v3.2-001-*.md`, `v3.2-002-*.md`, `v3.2-003-*.md`
- `README.md` Features section — Insert bullet after Stat 1 Pac mention

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond following the v3.0/v3.1 documentation pattern exactly. User confirmed all decisions are mechanical — patterns from Phase 30 (Math 1 docs) and Phase 35 (Stat 1 docs) provide complete precedent for every deliverable.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 40-documentation-adrs*
*Context gathered: 2026-05-25*
