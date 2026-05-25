# Phase 40: Documentation & ADRs - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `scripts/docs-matrix/src/main.rs` | utility | transform | `scripts/docs-matrix/src/main.rs` (self-extension) | exact |
| `justfile` | config | batch | `justfile` docs-matrix recipes (self-extension) | exact |
| `docs/hp41-time-divergences.md` | docs | — | `docs/hp41-stat1-divergences.md` | exact |
| `docs/adr/v3.2-001-clock-access-pattern.md` | docs | — | `docs/adr/v3.1-001-rng-state-placement.md` | exact |
| `docs/adr/v3.2-002-live-display-architecture.md` | docs | — | `docs/adr/v3.1-001-rng-state-placement.md` | exact |
| `docs/adr/v3.2-003-alarm-catalog-design.md` | docs | — | `docs/adr/v3.1-001-rng-state-placement.md` | exact |
| `README.md` | docs | — | `README.md` Features + Stat 1 bullet (self-extension) | exact |
| `CLAUDE.md` | docs | — | `CLAUDE.md` `### v3.1 additions` block (self-extension) | exact |
| `docs/architecture-history.md` | docs | — | `docs/architecture-history.md` `## v3.1 additions` section | exact |

## Pattern Assignments

---

### `scripts/docs-matrix/src/main.rs` (utility, transform)

**Analog:** `scripts/docs-matrix/src/main.rs` lines 70-78 — the existing basename dispatch block

**Core dispatch pattern** (lines 70-78):
```rust
// Title dispatch uses the input JSON basename — keeps the binary 1-in/1-out per D-30.1.
// String-match is intentional: avoids a new CLI argument while supporting both JSON sources.
fn render_markdown(entries: &[Entry], json_path: &str) -> String {
    let basename = std::path::Path::new(json_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(json_path);
    let (title, src) = if basename.ends_with("hp41cv-functions.json") {
        ("# HP-41CV ROM Function Matrix", "`docs/hp41cv-functions.json`")
    } else if basename.ends_with("hp41-math1-functions.json") {
        ("# HP-41C Math Pac I Function Matrix", "`docs/hp41-math1-functions.json`")
    } else if basename.ends_with("hp41-stat1-functions.json") {
        ("# HP-41C Stat 1 Pac Function Matrix", "`docs/hp41-stat1-functions.json`")
    } else {
        ("# Function Matrix", "`{json_path}`")
    };
```

**What to add (D-40.1):** Insert a new `else if` branch BEFORE the final `else` fallback:
```rust
    } else if basename.ends_with("hp41-time-functions.json") {
        ("# HP-41C Time Pac Function Matrix", "`docs/hp41-time-functions.json`")
    } else {
```

The insertion point is line 76 in the current file — between the `hp41-stat1-functions.json` arm and the `else` fallback. No other changes to the file.

---

### `justfile` (config, batch)

**Analog:** `justfile` lines 183-208 — existing `docs-matrix` and `docs-matrix-check` recipes

**Current docs-matrix recipe pattern** (lines 183-194):
```just
# Regenerate all three function matrices from their canonical JSON sources (developer-side).
# Reads docs/hp41cv-functions.json -> docs/hp41cv-function-matrix.md (unchanged, D-30.2).
# Reads docs/hp41-math1-functions.json -> docs/hp41-math1-function-matrix.md (unchanged, D-30.1).
# Reads docs/hp41-stat1-functions.json -> docs/hp41-stat1-function-matrix.md (new, D-35.1).
[group('docs')]
docs-matrix:
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41cv-functions.json docs/hp41cv-function-matrix.md
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41-math1-functions.json docs/hp41-math1-function-matrix.md
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41-stat1-functions.json docs/hp41-stat1-function-matrix.md
```

**Current docs-matrix-check recipe pattern** (lines 196-208):
```just
[group('docs')]
docs-matrix-check:
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41cv-functions.json /tmp/hp41cv-function-matrix-check.md
    diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41-math1-functions.json /tmp/hp41-math1-function-matrix-check.md
    diff -u docs/hp41-math1-function-matrix.md /tmp/hp41-math1-function-matrix-check.md
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41-stat1-functions.json /tmp/hp41-stat1-function-matrix-check.md
    diff -u docs/hp41-stat1-function-matrix.md /tmp/hp41-stat1-function-matrix-check.md
```

**What to add (D-40.2):** Append a 4th invocation block at the end of each recipe:

In `docs-matrix`, after the stat1 line:
```just
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41-time-functions.json docs/hp41-time-function-matrix.md
```

In `docs-matrix-check`, after the stat1 diff block:
```just
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41-time-functions.json /tmp/hp41-time-function-matrix-check.md
    diff -u docs/hp41-time-function-matrix.md /tmp/hp41-time-function-matrix-check.md
```

Also update the comment at the top of `docs-matrix` to add the 4th invocation line (following the D-35.1 / D-30.1 comment cadence).

---

### `docs/hp41-time-divergences.md` (docs, three-bucket catalog)

**Analog:** `docs/hp41-stat1-divergences.md` — the most recent divergences catalog (13 entries across 3 buckets)

**Header block pattern** (lines 1-14 of stat1-divergences.md):
```markdown
# HP-41C Stat 1 Pac Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Stat 1 Pac module and the hardware-faithful behavior described in the
HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979).

**Status:** Established as comprehensive numbered catalog in Phase 35 / Plan 35-02 (STAT-DOC-03).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.
```
For Phase 40: substitute "Stat 1 Pac" → "Time Pac", "HP 00041-90030, 1979" → "HP 00041-90036, 1982" (or the correct HP part number from Phase 38 research), "Phase 35 / Plan 35-02 (STAT-DOC-03)" → "Phase 40 / Plan 40-xx (TIME-DOC-02)".

**"How to Use This Document" block pattern** (lines 16-46 of stat1-divergences.md):
```markdown
Each entry carries a stable `D-35-NN` identifier that can be used in cross-references
from source-code comments, ADRs, test files, and issue trackers. The ID encodes the
phase (35 = Phase 35 / STAT-DOC-03) and an ordinal sequence number within this document.

Every entry uses five fixed fields (D-30.5 shape, carried forward as D-35 template):

- **OM citation** — The HP 00041-90030 page-and-example that is the primary source, or
  `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41C hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. UX trade-off decision).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from `research/PITFALLS.md` (carried forward across v3.x).

The citation discipline (Pitfall 18 from `research/PITFALLS.md`, carried forward across
v3.x) requires every entry to carry at least one OM page reference, an explicit
`"N/A — emulator extension"` marker, or a primary-source citation...
```
For Phase 40: substitute all `D-35-NN` → `D-40-NN`, "phase (35 = Phase 35 / STAT-DOC-03)" → "phase (40 = Phase 40 / TIME-DOC-02)", OM part number to the Time Pac OM.

**Three-bucket section headers pattern** (lines 49, 71, 169 of stat1-divergences.md):
```markdown
## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. These are cases where the OM specifies or implies a particular outcome and our
emulator either matches or intentionally diverges from that specification.)*

---

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90030 (1979). These
are deliberate, documented additions that improve usability without conflicting with OM
behavior for OM-specified inputs. Every extension in this section is marked with
"N/A — emulator extension" in the OM citation field.)*

---

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.
These entries document cases where the emulator made a specific policy decision that
affects behavior in ways the OM either specifies explicitly or leaves to the implementation.)*
```

**Five-field entry shape pattern** (D-30.5 — from stat1-divergences.md D-35-07):
```markdown
### D-35-07: RAND / SEED LCG — v3.1 Emulator Extension

- **OM citation**: `N/A — emulator extension`. ...

- **Our behavior**: `XEQ "RAND"` generates ...

- **OM behavior**: `N/A — emulator extension`. ...

- **Rationale**: ...

- **See**: `docs/adr/v3.1-001-rng-state-placement.md` ...; `hp41-core/src/ops/stat1/rand.rs`; ...
```

**Known Time Pac entries to author (from D-40.4):**

Bucket 1 (OM Divergences): likely none, or entries discovered during Phase 38/39 execution.

Bucket 2 (Emulator Extensions): entries TBD from Phase 38/39 implementation (e.g., if any non-OM features were added).

Bucket 3 (Behavioral Policies) — known divergences from D-38.4 / D-40.4:
- `D-40-NN`: CORRECT/SETAF accuracy factor no-op (host NTP clock — CORRECT stores but does nothing)
- `D-40-NN`: Host system clock backing (SystemTime vs. crystal oscillator)
- `D-40-NN`: Stopwatch freeze-on-save (Instant cannot serialize; user must RUNSW to resume)
- `D-40-NN`: Interrupting control alarm deferral (D-38.4 — re-entrancy hazard)
- `D-40-NN`: Centisecond stopwatch resolution (matches OM but via different mechanism)

Numbering: start at `D-40-01` and number sequentially within each bucket.

---

### `docs/adr/v3.2-001-clock-access-pattern.md` (docs, ADR)

**Analog:** `docs/adr/v3.1-001-rng-state-placement.md` — most recent long-form ADR

**ADR header block pattern** (lines 1-8 of v3.1-001):
```markdown
# ADR-v3.1-001: RNG State Placement — `rand_seed: HpNum` on `CalcState` with `#[serde(default)]` WITHOUT `#[serde(skip)]`

**Status:** Locked 2026-05-22
**Owner:** Plan 33-01 + Plan 33-08
**Requirement refs:** STAT-RNG-01, STAT-RNG-02, STAT-RNG-03, STAT-RNG-04, Pitfall 20 (...)
**Downstream consumer:** `hp41-core/src/state.rs:201-202` (...); `hp41-core/src/ops/stat1/rand.rs` (...)
**ADR write-up prose:** Phase 35 / STAT-DOC-04 (this is the narrative ADR; the lock happened during Phase 33 discuss-phase 2026-05-22 / D-33.4)
```
For v3.2-001: Title = "Direct SystemTime in hp41-core — `std::time::SystemTime::now()` for Clock Access". Status = "Locked 2026-05-24". Owner = Plan 38-xx. Requirement refs = TIME-CLK-01..06, Pitfall 34 (clock in core). Downstream consumer = `hp41-core/src/ops/time/clock.rs`. ADR write-up prose = "Phase 40 / TIME-DOC-03".

**Section structure pattern** (from v3.1-001):
- `## Context` — ~400-600 words describing the design problem
- `## Decision` — clear statement of what was chosen, with the key mechanism
- `## Ready-to-paste <artifact>` (optional, only if there's a code snippet worth embedding)
- `## Consequences` — `### Positive`, `### Negative`, `### Neutral` subsections
- `## Alternatives Considered` — one subsection per rejected alternative, each with "**Why rejected:**" line
- `## Footnotes / References` — `[^N]:` footnotes

**Alternatives Considered pattern** (from v3.1-001 lines 167-220):
```markdown
## Alternatives Considered

### Option B: <alternative name>

The rejected alternative is described verbatim in 38-CONTEXT.md D-38.1 (locked
2026-05-24):

> **D-38.1:** ...verbatim quote from CONTEXT.md...

**Why rejected:** ...
```

Per D-40.5 and D-30.7, quote Phase 38 CONTEXT decisions verbatim in `## Alternatives Considered`. The three alternatives listed in D-40.5 for this ADR are: trait injection, frontend callback, `no_std` clock abstraction.

**ADR footer pattern** (last 3 lines of v3.1-001):
```markdown
*ADR-v3.1-001 locked: 2026-05-22. Plan 33-01 + Plan 33-08.*
*ADR write-up: Phase 35 / Plan 35-03 / STAT-DOC-04.*
*OM reference: HP 00041-90030, 1979 — ...*
```
For v3.2 ADRs: substitute the appropriate lock date, plan IDs, and OM reference (HP 00041-90036 Time Module for clock-related ADRs).

---

### `docs/adr/v3.2-002-live-display-architecture.md` (docs, ADR)

**Analog:** `docs/adr/v3.1-001-rng-state-placement.md` — same long-form template

**Title:** `ADR-v3.2-002: Live Display Architecture — Pull-on-Redraw Pattern for Clock/Stopwatch Display`

**Key context material (D-39.1, D-39.2, D-38 carried.7):**
- Pull-on-redraw: the existing 16ms poll loop already redraws ~62 times/second; clock/stopwatch state is computed freshly in `get_display_string()` on each redraw cycle
- No async timer, no push-from-core, no polling thread required

**Alternatives Considered (D-40.5 for this ADR):** async timer, push-from-core, polling thread. Quote D-39.1/D-39.2/D-38 carried.7 verbatim.

**Requirement refs:** TIME-DSP-01..05, TIME-SW-08, Pitfall 32 (live display)

Use the same section structure as v3.1-001 (Context / Decision / Consequences / Alternatives Considered / Footnotes).

---

### `docs/adr/v3.2-003-alarm-catalog-design.md` (docs, ADR)

**Analog:** `docs/adr/v3.1-001-rng-state-placement.md` — same long-form template

**Title:** `ADR-v3.2-003: Alarm Catalog Design — Vec<AlarmEntry> on CalcState with event_buffer Drain`

**Key context material (D-38.8-11):**
- `AlarmType` enum: `Message(String)` | `Control { label: String, interrupting: bool }`
- Direct `alarms: Vec<AlarmEntry>` on CalcState with `#[serde(default)]`
- `check_alarms()` drain pattern; alarm notifications pushed into `event_buffer`
- Repeat interval stored as `i64` seconds

**Alternatives Considered (D-40.5 for this ADR):** separate alarm state struct, channel-based notification, trait-based alarm handler. Quote D-38.8-11 verbatim from 38-CONTEXT.md.

**Requirement refs:** TIME-ALM-01..12, Pitfall 37 (alarm state machine)

---

### `README.md` (docs, soft-claim bullet)

**Analog:** `README.md` lines 54-55 — the existing v3.1 Stat 1 soft-claim bullet and function-matrix link

**Current v3.1 bullet pattern** (lines 54-55):
```markdown
- v3.1 ships Stat 1 Pac behavioral emulation, feature-complete per Owner's Manual HP 00041-90030 (13 programs, 26 XEQ entry points,
  RAND/SEED extension, [documented divergences](docs/hp41-stat1-divergences.md)) — see [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)
```

**What to add (D-40.7):** Append a v3.2 soft-claim bullet AFTER the v3.1 line, following the tone set by v3.1. Do NOT use "feature-complete per Owner's Manual" — that phrasing is the hard-claim and is deferred to Phase 42 graduation. Example shape:
```markdown
- v3.2 ships Time Pac behavioral emulation (35 XEQ entry points, real-time clock/stopwatch/alarm,
  [documented divergences](docs/hp41-time-divergences.md)) — see [Time Pac Function Matrix](docs/hp41-time-function-matrix.md)
```
(Exact wording is Claude's discretion per D-40.8; follow v3.1 tone — include entry-point count, key features, divergences link, matrix link.)

**Release table (D-40.8):** Do NOT add a v3.2 row yet — deferred to Phase 42 ship. No change to the release table in this phase.

---

### `CLAUDE.md` (docs, v3.2 additions block)

**Analog:** `CLAUDE.md` lines 120-170 — the `### v3.1 additions (Stat 1 Pac Emulation, Phases 33–37)` block

**Section header pattern** (line 120):
```markdown
### v3.1 additions (Stat 1 Pac Emulation, Phases 33–37)
```
For Phase 40: add `### v3.2 additions (Time Pac Emulation, Phases 38–42)` as a new top-level subsection under `## Frozen Invariants`, following v3.1's block.

**Opening sentence pattern** (from v3.1 block, CLAUDE.md line 122):
```markdown
*Origin: see `docs/architecture-history.md` §v3.1 additions for the long-form per-phase narrative; this block is the CLAUDE.md decision-summary surface (the FIRST-EVER `### v3.x additions` block per D-35.5; ...)*
```
For v3.2: adapt to reference `§v3.2 additions` and this is the SECOND `### v3.x additions` block.

**Per-phase subsection pattern** (from `#### Phase 33` at line 124 — ~15-25 bullet points, decision-summary level):
```markdown
#### Phase 38 — XROM Framework + Clock/Date/Stopwatch/Alarm Core (shipped 2026-05-24)

- **TIME_MODULE (XROM ID 26) registered (D-carried.5):** `TIME_MODULE.id = 26` + `XROM bit-2 arm` in `xrom_resolve`; fires LAST after bit-1 (STAT_1) and bit-0 (MATH_1) per Pitfall 1.
- **`default_xrom_modules() = 0b0000_0111` + `migrate_after_load()` (D-carried.5):** v3.1 save files with `xrom_modules: 0b0000_0011` auto-upgrade; canonical migration site `state.rs`.
- **Direct `std::time::SystemTime::now()` in hp41-core (D-38.1):** ...
- **`time_offset_secs: i64` on CalcState (D-38.2):** ...
- **Stopwatch fields with split serde shapes (D-38.6-7):** ...
- **`AlarmType` enum + `Vec<AlarmEntry>` on CalcState (D-38.8-11):** ...
- **Interrupting control alarm DEFERRED (D-38.4):** documented divergence...
- **~33 new `Op` variants** in `dispatch()` + `execute_op()` (4-way invariant items 1+2 complete)...
```

**Per-phase subsection for Phase 39** (shipped 2026-05-25):
```markdown
#### Phase 39 — CLI Integration + Live Display (shipped 2026-05-25)

- **`docs/hp41-time-functions.json` authored (D-39.9):** 35-entry canonical source; 7-category convention...
- **Fourth `OnceLock<Vec<HelpEntry>>` in `help_data.rs` (D-39.12):** `TIME_HELP_ENTRIES` + `help_entries_time()` + 4-pool chain...
- **35 new `op_display_name` arms in `hp41-cli/src/prgm_display.rs`** (4-way invariant item 3 complete)...
- **`?` help overlay "Time Pac (XROM 26)" section (D-39.13)**...
- **Live clock/stopwatch display (D-39.1-2):** pull-on-redraw, 16ms poll loop...
- **Stopwatch keyboard mode (D-39.4-5):** top-level `handle_key()` intercept, no new PendingInput variant...
- **Alarm event draining (D-39.6-8):** `check_alarms()` called every 16ms tick outside poll conditional...
```

**Phase 40 stub pattern** (from v3.1, "IN PROGRESS" pattern for incomplete phases):
```markdown
#### Phase 40 — Documentation & ADRs (shipped 2026-05-25)

- **`docs/hp41-time-divergences.md` authored (D-40.3):** three-bucket numbered catalog...
- **3 ADRs: v3.2-001 (clock access), v3.2-002 (live display), v3.2-003 (alarm catalog) (D-40.5)**...
- **`scripts/docs-matrix` fourth invocation (D-40.1-2):** `hp41-time-functions.json` → `hp41-time-function-matrix.md`...
- **README v3.2 soft-claim (D-40.7)**...
- **CLAUDE.md `### v3.2 additions` block authored (D-40.9)**...
```

**Phases 41-42 stubs:**
```markdown
#### Phase 41 — GUI Integration (IN PROGRESS)

#### Phase 42 — Test Hardening & Quality Gates (TBD)
```

**D-40.10 frozen invariants addendum:** Add a brief entry in the Frozen Invariants section documenting the Phase 38 CalcState additions:
```markdown
- **Phase 38 CalcState additions (D-38.1-11):** `time_offset_secs: i64` (`#[serde(default)]`), `stopwatch_mode: StopwatchMode` (`#[serde(default)]`), `stopwatch_accumulated: f64` (`#[serde(default)]`), `stopwatch_split: f64` (`#[serde(default)]`), `stopwatch_start: Option<Instant>` (`#[serde(default, skip)]` — transient), `alarms: Vec<AlarmEntry>` (`#[serde(default)]`), `clock_display_mode` and related flags. XROM bit-2 arm = `TIME_MODULE` (XROM 26).
```

---

### `docs/architecture-history.md` (docs, v3.2 narrative section)

**Analog:** `docs/architecture-history.md` lines 192-271 — the `## v3.1 additions` section (~80 lines, ~20-40 lines per phase)

**Section header pattern** (line 192):
```markdown
## v3.1 additions (Stat 1 Pac Emulation, Phases 33–37)
```
For Phase 40: add `## v3.2 additions (Time Pac Emulation, Phases 38–42 — 41–42 IN PROGRESS)` as a new section at the end of the file.

**Opening paragraph pattern** (line 194 — context-setting, one dense paragraph):
```markdown
Phases 33–37 ship the second XROM application module — the HP-41C Stat 1 Pac (HP part
number 00041-90030, Owner's Manual 1979) — as a behavioral emulation of 13 top-level
programs with 26 XEQ-by-name entry points across univariate / ANOVA / regression /
hypothesis / nonparametric / distribution / RNG families. The work extends every v3.0
invariant (XROM resolver chain, modal-workflow infrastructure, JSON-canonical pipeline,
save-file backward compat) and introduces one targeted exception...
```
For v3.2: substitute Time Pac (HP 00041-90036, 1982), 35 Op variants, clock/date/stopwatch/alarm families. Mention the key novel additions: `SystemTime` clock access, pull-on-redraw live display, `Vec<AlarmEntry>` alarm catalog, time offset field.

**Per-phase narrative paragraph pattern** (line 196 — `### Phase 33 — ...`, ~20-40 lines):

Each phase gets a `### Phase NN — Name (shipped YYYY-MM-DD)` heading followed by one or more dense paragraphs covering:
1. What landed and why (key decisions with D-NN.x codes)
2. Invariants preserved/extended
3. Key numbers (Op count, test count, coverage if relevant)

Calibrate to v3.1 Phases 33/34 depth (~20-40 lines per phase). Phase 38 and 39 are shipped; use their CONTEXT files as source material. Phases 40-42 get stubs.

**Frozen invariants summary pattern** (lines 266-274 — closing block):
```markdown
**Frozen invariants preserved across v3.1:**

- SC-4 invariant: ...
- 4-exhaustive-match invariant: ...
- `#![deny(clippy::unwrap_used)]` continues...
- Save-file backward compat: ...
```
For v3.2: add a parallel `**Frozen invariants preserved across v3.2:**` block at the end of the v3.2 section (after Phase 42 narrative ships; stub it for now).

---

## Shared Patterns

### ADR Long-Form Template
**Source:** `docs/adr/v3.1-001-rng-state-placement.md`
**Apply to:** All three v3.2 ADR files
```markdown
# ADR-v3.2-NNN: <Title>

**Status:** Locked <date>
**Owner:** Plan <plan-id>
**Requirement refs:** <reqs>, Pitfall <N> (<description>)
**Downstream consumer:** <primary files that implement this decision>
**ADR write-up prose:** Phase 40 / TIME-DOC-03 (this is the narrative ADR; the lock happened during Phase 38 discuss-phase <date> / D-38.N)

---

## Context

[400-600 words]

---

## Decision

[Clear statement of chosen approach]

---

## Consequences

### Positive
### Negative
### Neutral

---

## Alternatives Considered

### Option B: <name>

The rejected alternative is described verbatim in 38-CONTEXT.md D-38.N (locked <date>):

> **D-38.N:** [verbatim quote]

**Why rejected:** ...

---

## Footnotes / References

[^1]: ...

---

*ADR-v3.2-NNN locked: <date>. Plan <plan-id>.*
*ADR write-up: Phase 40 / Plan 40-xx / TIME-DOC-03.*
*OM reference: HP 00041-90036, 1982 — <relevant section>.*
```

### Divergence Entry 5-Field Shape
**Source:** `docs/hp41-stat1-divergences.md` D-35-07 through D-35-13
**Apply to:** All entries in `docs/hp41-time-divergences.md`
```markdown
### D-40-NN: <Title> — <brief descriptor>

- **OM citation**: HP 00041-90036 (1982), §<section> — <quote or "N/A — emulator extension/policy">

- **Our behavior**: <concrete description with code references>

- **OM behavior**: <what real HP-41CX hardware does, or "N/A — emulator extension/policy">

- **Rationale**: <why this choice; rejected alternatives>

- **See**: <ADR links, CONTEXT.md D-NN.x, source file paths, test file:line, Pitfall refs>
```

### Docs-Matrix Basename Dispatch Extension
**Source:** `scripts/docs-matrix/src/main.rs` lines 70-78
**Apply to:** `scripts/docs-matrix/src/main.rs` (single `else if` insertion)
The pattern is always: `else if basename.ends_with("<filename>.json")` → `("<title>", "<src-path>")`. No other changes needed.

### Justfile Recipe Extension
**Source:** `justfile` lines 183-208
**Apply to:** `justfile` `docs-matrix` and `docs-matrix-check` recipes
Each recipe extension is exactly 2-3 lines: one `cargo run` invocation + (for check) one `diff -u` line. Append after the last stat1 block.

### README Soft-Claim Bullet
**Source:** `README.md` lines 52-55 (v3.0 hard-claim + v3.1 hard-claim lines)
**Apply to:** `README.md` Features section
Pattern: `- v3.N ships <Module> behavioral emulation (<N programs>, <N XEQ entry points>, [documented divergences](<path>)) — see [<Module> Function Matrix](<matrix-path>)`
The v3.2 bullet must NOT include "feature-complete per Owner's Manual" — that is the hard-claim deferred to Phase 42.

### CLAUDE.md Per-Phase Summary Bullet List
**Source:** `CLAUDE.md` lines 124-170 (Phase 33-37 bullet lists)
**Apply to:** `CLAUDE.md` new `### v3.2 additions` subsections
Each bullet cites the decision ID (D-NN.x) and the canonical source file. Bold the subject, parenthetical the decision ID, colon-separate the description. Phases 41-42 get single-line stubs with "(IN PROGRESS)" or "(TBD)".

---

## No Analog Found

All 8 files have strong exact-match analogs. No file requires falling back to RESEARCH.md patterns.

---

## Metadata

**Analog search scope:** `docs/`, `docs/adr/`, `scripts/docs-matrix/src/`, `justfile`, `README.md`, `CLAUDE.md`, `docs/architecture-history.md`
**Files scanned:** 11 analog files read
**Pattern extraction date:** 2026-05-25
