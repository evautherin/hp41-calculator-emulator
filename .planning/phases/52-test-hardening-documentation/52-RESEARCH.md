# Phase 52: Test Hardening + Documentation — Research

**Researched:** 2026-05-28
**Domain:** Rust integration wiring, JSON help pipeline, backward-compat fixtures, isolation tests, meta-gates, ADRs
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-52.1:** X-MEM functions get a dedicated `docs/hp41-xmem-functions.json` pool with its own `OnceLock<Vec<HelpEntry>>` in `help_data.rs`, mirroring the 4 XROM-module pools. Dedicated section in `?` overlay.
- **D-52.2:** Standalone `docs/hp41-xmem-function-matrix.md` generated from the new pool, wired into `just docs-matrix` + `just docs-matrix-check`.
- **D-52.3:** All 8 X-MEM JSON entries carry full worked examples (`example` + `notes` fields), matching Phase 49 quality bar.
- **D-52.4:** XEQ-by-name only. Wire `builtin_card_op` in `hp41-core/src/ops/program.rs` so `XEQ "EMDIR"` etc. resolve. No dedicated physical keys or comfort shortcuts.
- **D-52.5:** Discoverability via `?` overlay + Phase 49 searchable reference (auto-driven by D-52.1 pool). No CATALOG 3 in this phase.
- **D-52.6:** Scoped/honest README claim — "Extended Memory: named PROGRAM + DATA file storage (HP-41CX X-Functions)". No "feature-complete" wording.
- **D-52.7:** Granular per-decision ADRs (matching prior ADR granularity). At minimum: X-MEM as OS built-ins/no XROM bit; fixed 600-register capacity; full-register-set SAVED/GETD.
- **D-52.8:** New `docs/hp41-xmem-divergences.md` — overwrite-on-duplicate divergence (D-51.6) + full-register-set SAVED/GETD divergence (D-51.5).
- **D-52.9:** Standard follow-through: CLAUDE.md + `docs/architecture-history.md` updates.
- **D-52.10:** `hp41-core/tests/fixtures/v33-autosave.json` fixture genuinely lacking `xmem_files`/`xmem_active_file`, plus `xmem_backward_compat.rs` test file.
- **D-52.11:** Isolation tests proving X-MEM ops never read/write `state.regs` or `adv_matrices` except via SAVED/GETD transfer.
- **D-52.12:** Confirm whether any explicit `migrate_after_load()` arm is needed beyond serde defaults.

### Claude's Discretion

- **D-52.13:** Extend op↔JSON parity test (8 X-MEM `Op` variants ↔ 8 JSON entries) + per-op test-count floor ≥5. Method: hand-curated `XMEM_OP_VARIANT_NAMES` array + `xrom_op_test_count.rs`-style scan of xmem source files.
- Fixture authenticity (D-52.10): capture from `v3.3` tag if feasible; otherwise hand-craft.
- EMDIR print-buffer line format, error-variant naming, exact ADR numbering.

### Deferred Ideas (OUT OF SCOPE)

- CATALOG 3 / Full Function Catalog — new Phase 53.
- Comfort key shortcuts for X-MEM ops.
- ASCII + STATUS X-MEM file types (XMEM-F01/F02, v4.1).
- OM-cited "feature-complete" README hard claim for X-MEM (deferred to v4.1).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| XMEM-08 | X-MEM state persists across save/load with `#[serde(default)]` backward compat | Fields already carry `#[serde(default)]`; v33-autosave.json fixture tests the serde default path; `migrate_after_load()` confirmed requires no new arm |
| XMEM-09 | X-MEM storage isolated from main registers and `adv_matrices` (D-43.5 pattern) | Isolation invariant declared in ops.rs header; isolation tests mirror `adv_backward_compat.rs` pattern |
| XMEM-10 | CLI + GUI integration (4-way exhaustive match, `op_display_name`, help overlay) | 4-way match arms already present (Phase 51); Phase 52 adds 8 arms to `builtin_card_op`; new `help_entries_xmem()` OnceLock; `help_entries_all()` extended |
</phase_requirements>

---

## Summary

Phase 52 is a pure integration and hardening phase — no new `Op` variants, no new data model. Phase 51 delivered the 8 X-MEM ops with all four exhaustive-match arms, the `xmem_files`/`xmem_active_file` fields on `CalcState` (both with `#[serde(default)]`), and 15 inline tests in `ops.rs`. Phase 52's job is to (a) wire XEQ-by-name resolution, (b) add the help JSON pool and matrix doc, (c) prove backward-compat and isolation with integration tests, (d) extend the meta-gates, and (e) write ADRs + divergences doc + scoped README claim.

**The single biggest integration point** is `builtin_card_op()` in `hp41-core/src/ops/program.rs` at line 1368. Adding 8 arms there (one per X-MEM mnemonic) simultaneously enables CLI XEQ-by-name (which calls `xeq_by_name_local_resolve` → `builtin_card_op`), GUI XEQ-by-name (which emits `Op::Xeq("EMDIR")` via the `xeq_<name>` strip_prefix arm in `key_map.rs:427` → `op_xeq` in hp41-core → `builtin_card_op`), and programmatic XEQ in running programs. One change unlocks all three access paths.

**The help JSON pool** follows a completely mechanical precedent: add `const XMEM_FUNCTIONS_JSON`, static `XMEM_HELP_ENTRIES: OnceLock<Vec<HelpEntry>>`, a narrow `help_entries_xmem()` accessor, and extend `help_entries_all()` to chain it sixth. The `?` overlay and right-panel automatically pick up the new pool without any overlay rendering code changes. The key caveat: X-MEM entries have `xrom: None` (they are OS built-ins, not XROM), which means they pass the `entry.xrom.is_none()` right-panel filter — they WILL appear in the right-panel key reference if they carry a `key_path`. Since X-MEM is XEQ-by-name only, their `key_path` should be `"XEQ \"EMDIR\""` format — consistent with XROM module entries and correctly excluded from the physical-key right-panel (which filters `key_path.is_none()` entries, not `xrom.is_some()` entries).

**Backward compat** is already handled by `#[serde(default)]` on both fields; `migrate_after_load()` requires no new arm because X-MEM has no XROM bit to set. The v33-autosave.json fixture is a minimal hand-craft of the v32-autosave.json, dropping `adv_matrices`/`adv_tvm_state` and keeping `xrom_modules: 31` (0b0001_1111) — it must NOT contain `xmem_files` or `xmem_active_file` fields to exercise the serde default path.

**Primary recommendation:** Execute as 4 parallel workstreams — (1) builtin_card_op wiring + XEQ-by-name tests, (2) help JSON pool + overlay wiring + meta-gate extensions, (3) backward-compat fixture + isolation tests, (4) docs (ADRs + divergences + README + architecture-history). Order: workstreams 1, 2, 3 can proceed concurrently; workstream 4 is pure docs and has no blockers.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| XEQ-by-name resolution | hp41-core (`builtin_card_op`) | — | Resolution is in hp41-core so both CLI and GUI share the same table; CLI delegates via `xeq_by_name_local_resolve`; GUI via `op_xeq` → `builtin_card_op` |
| GUI string-ID → Op dispatch | Frontend Server / GUI Tauri (`key_map::resolve`) | — | `xeq_<name>` strip_prefix already dispatches to `Op::Xeq(name)` which flows through op_xeq → builtin_card_op; no new code in key_map needed |
| Help overlay data | CLI (`help_data.rs` OnceLock pools) | GUI (reads same JSON via its own pool) | CLI has the canonical `help_data.rs`; GUI has its own equivalent pool. Both must be extended for X-MEM. |
| Backward-compat test | hp41-core integration tests | — | `tests/xmem_backward_compat.rs` exercises serde deserialization + `migrate_after_load()` at the crate boundary |
| Isolation guarantee | hp41-core (`ops/xmem/ops.rs` invariant) | hp41-core integration tests | Code invariant declared in module header; integration tests prove it for each op |
| Op↔JSON parity meta-gate | hp41-cli integration tests | — | `function_matrix_parity.rs` hosts all parity tests; `xrom_op_test_count.rs` hosts the per-op floor test |
| Matrix doc generation | `scripts/docs-matrix` + `justfile` | — | Mechanical extension of existing 5-pool pattern |
| ADRs + divergences docs | `docs/adr/` + `docs/hp41-xmem-divergences.md` | — | Pure doc write, no code coupling |

---

## Standard Stack

### Core (all already present — zero new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` / `serde_json` | workspace | Backward-compat fixture deserialization | Already used for all state persistence |
| `std::sync::OnceLock` | std | Lazy-init JSON pool | Established pattern (5 existing pools) |
| `include_str!` | Rust macro | Compile-time JSON embedding | D-25.17 hard-build-blocker semantics |

No new dependencies. This phase adds zero new `Cargo.toml` entries. [VERIFIED: codebase grep]

### Installation
No `cargo add` commands needed for this phase.

---

## Package Legitimacy Audit

> No external packages are installed in this phase. All changes are within the existing workspace. Audit: N/A.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User types: XEQ "EMDIR"
                │
    ┌───────────┴────────────┐
    │ CLI path               │ GUI path
    │                        │
    │ xeq_by_name_local_     │ key_map::resolve("xeq_EMDIR")
    │ resolve("EMDIR", bits) │   → strip_prefix("xeq_")
    │   → builtin_card_op    │   → Op::Xeq("EMDIR")
    │                        │       ↓
    └───────────┬────────────┘  op_xeq in hp41-core
                │                  → builtin_card_op("EMDIR")
                ↓
    builtin_card_op("EMDIR") → Some(Op::EmDir)   ← Phase 52 adds 8 arms here
                │
                ↓
    dispatch(state, Op::EmDir)
                │
                ↓
    op_emdir(state)  ← already implemented in Phase 51
                │
                ↓
    state.print_buffer  (EMDIR catalog output)
```

```
? overlay request
    │
    ↓
help_entries_all()
    chains: help_entries()           ← v2.2 built-ins
          .chain(help_entries_math1())  ← XROM 7
          .chain(help_entries_stat1())  ← XROM 2
          .chain(help_entries_time())   ← XROM 26
          .chain(help_entries_adv())    ← XROM 22+24
          .chain(help_entries_xmem())   ← Phase 52: X-MEM built-ins (no xrom field)
    │
    ↓
help_overlay_rows() → grouped by category → "Extended Memory" section
```

### Recommended Project Structure (new files only)

```
docs/
├── hp41-xmem-functions.json        # New: 8-entry X-MEM JSON pool (D-52.1)
├── hp41-xmem-function-matrix.md    # New: generated matrix doc (D-52.2)
├── hp41-xmem-divergences.md        # New: overwrite-on-dup + full-reg-set divergences (D-52.8)
└── adr/
    ├── v4.0-001-xmem-os-builtin.md    # New: X-MEM as OS built-ins, no XROM bit (D-52.7)
    ├── v4.0-002-xmem-capacity.md      # New: 600-register fixed capacity (D-52.7)
    └── v4.0-003-xmem-saved-getd.md    # New: full-reg-set SAVED/GETD (D-52.7)

hp41-core/src/
└── ops/program.rs                  # Modify: add 8 arms to builtin_card_op (D-52.4)

hp41-cli/src/
└── help_data.rs                    # Modify: add XMEM pool OnceLock + extend help_entries_all (D-52.1)

hp41-gui/src-tauri/src/
└── help_data.rs                    # Modify: same pool extension (D-52.1) — if GUI has its own help_data

hp41-core/tests/
├── fixtures/v33-autosave.json      # New: v3.3 fixture without xmem fields (D-52.10)
└── xmem_backward_compat.rs         # New: backward-compat + isolation tests (D-52.10, D-52.11)

hp41-cli/tests/
├── function_matrix_parity.rs       # Modify: add XMEM pool parity tests (D-52.13)
└── phase52_help_data_xmem.rs       # New: 8-entry pool smoke test (mirrors phase44_help_data_adv.rs)

justfile                             # Modify: add xmem pool to docs-matrix + docs-matrix-check (D-52.2)
```

### Pattern 1: builtin_card_op Extension (D-52.4)

**What:** Add 8 match arms to `pub fn builtin_card_op(name: &str) -> Option<Op>` in `hp41-core/src/ops/program.rs` at line 1447 (before the `_ => None` arm).

**When to use:** This is the single XEQ-by-name resolution point for all built-in ops. Adding here automatically propagates to CLI (`xeq_by_name_local_resolve`), GUI (`op_xeq`), and programmatic `XEQ` in running programs.

**Exact current end of function** (line 1447 in program.rs):
```rust
// Source: hp41-core/src/ops/program.rs, confirmed lines 1444-1449
        "PRSTK" => Some(Op::PRSTK),
        _ => None,
    }
}
```

**Pattern to add** (insert before `_ => None`):
```rust
// Phase 52 (v4.0): X-MEM built-in ops (HP-41CX Extended Functions)
"EMDIR"  => Some(Op::EmDir),
"EMROOM" => Some(Op::EmRoom),
"SAVEP"  => Some(Op::SaveP),
"GETP"   => Some(Op::GetP),
"SAVED"  => Some(Op::SaveD),
"GETD"   => Some(Op::GetD),
"EMREG"  => Some(Op::EmReg),
"SAVERX" => Some(Op::SaveRx),
```

**GUI key_map interaction:** The GUI `key_map::resolve_parameterized` already has the `xeq_<name>` → `Op::Xeq(name)` arm at line 427. When the frontend sends `"xeq_EMDIR"`, it produces `Op::Xeq("EMDIR")`, which `op_xeq` in hp41-core resolves via `builtin_card_op`. No changes to `key_map.rs` are needed.

**CLI interaction:** `xeq_by_name_local_resolve` at `hp41-cli/src/keys.rs:369` calls `builtin_card_op(name).or_else(|| xrom_resolve(...))`. The new arms are immediately picked up. No changes to `keys.rs` are needed.

### Pattern 2: Help JSON Pool (OnceLock + include_str! + help_entries_all extension)

**What:** The established 5-pool pattern in `hp41-cli/src/help_data.rs`. X-MEM is the 6th pool.

**Exact current pool chain** (lines 213–220 in help_data.rs):
```rust
// Source: hp41-cli/src/help_data.rs, confirmed lines 213-220
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
        .chain(help_entries_adv().iter())
}
```

**Pattern to add** (new constant + OnceLock + accessor, then extend `help_entries_all`):
```rust
// Phase 52 (v4.0): X-MEM built-in pool (D-52.1)
const XMEM_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-xmem-functions.json");
static XMEM_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed X-MEM help entries (lazily initialized, thread-safe via OnceLock).
/// Panics on malformed JSON — intentional D-25.17 hard-build-blocker behavior.
pub fn help_entries_xmem() -> &'static [HelpEntry] {
    XMEM_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(XMEM_FUNCTIONS_JSON)
            .expect("hp41-xmem-functions.json is malformed — fix the JSON")
    })
}
```

Then extend `help_entries_all()`:
```rust
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
        .chain(help_entries_adv().iter())
        .chain(help_entries_xmem().iter())  // Phase 52
}
```

**Right-panel filter caveat (critical):** `key_ref_entries()` in `keys.rs` (and its GUI equivalent) filters `entry.xrom.is_some()` entries OUT from the right-panel key-binding list. X-MEM entries have `xrom: None` (they are OS built-ins). This means they WILL appear in right-panel results IF they have a non-null `key_path`. Since X-MEM ops are XEQ-by-name only, their `key_path` values should be `"XEQ \"EMDIR\""` etc. — which means they DO qualify for the right panel (the filter is `xrom.is_some()`, not the reverse). This is correct behavior: XEQ-by-name entries appear in the right panel to aid discoverability. XROM module entries are excluded because their module-grouping is in the `?` overlay only.

**Confirmed right-panel exclusion logic** (`keys.rs:393-399`):
> XROM-module exclusion (post-v3.0): entries with `xrom.is_some()` — i.e. Math Pac I and any future XROM-module functions — are EXCLUDED from the right-panel.

X-MEM entries (`xrom: None`) pass this filter and appear in the right panel. The `?` overlay shows them under their own "Extended Memory" category section.

### Pattern 3: JSON Pool Entry Schema

X-MEM entries do NOT carry an `xrom` field (they are built-ins). The schema matches the v2.2 built-in entries in `hp41cv-functions.json`, extended with `example`/`notes` (which `HelpEntry` does not currently declare in `help_data.rs` — see caveat below).

**Confirmed `HelpEntry` struct fields** (from `help_data.rs`):
```rust
pub struct HelpEntry {
    pub op_variant: String,      // e.g. "EmDir"
    pub display_name: String,    // e.g. "EMDIR"
    pub category: String,        // "Extended Memory"
    pub status: String,          // "implemented"
    pub phase: Option<String>,   // "52"
    pub key_path: Option<String>,// "XEQ \"EMDIR\""
    pub description: String,     // <= 80 chars
    pub divergences: Vec<String>,// #[serde(default)]
    pub xrom: Option<XromEntry>, // None for X-MEM
}
```

**CAVEAT — example/notes fields:** The `HelpEntry` struct does NOT currently declare `example` or `notes` fields. These appear in the advantage JSON (`docs/hp41-advantage-functions.json` lines 10-11) and time JSON, but `HelpEntry` ignores them silently (serde `#[deny(unknown_fields)]` is NOT set, so unknown fields are ignored during deserialization). D-52.3 says to add `example`/`notes` to the X-MEM JSON. This is fine: serde will ignore them during overlay loading. The `scripts/docs-matrix` `Entry` struct also lacks these fields and will ignore them. If the planner wants `example`/`notes` to be rendered in the matrix doc, that requires extending the `docs-matrix` script and `HelpEntry` — otherwise they are present in JSON but silently dropped. **Recommendation (planner's call):** add `example: Option<String>` and `notes: Option<String>` with `#[serde(default)]` to `HelpEntry` and extend the matrix renderer, for consistency with the Phase 49 quality bar. This is optional work; the overlay and parity tests work correctly without it.

### Pattern 4: Backward-compat Fixture (D-52.10)

**What:** The v33-autosave.json fixture is based on v32-autosave.json with these changes:
- Remove: `adv_matrices`, `adv_matrix_i`, `adv_matrix_j`, `adv_tvm_state`, `adv_froot_state`, etc. (fields added in Phase 43 that the Phase 51/52 code adds via serde default if absent — actually these were NOT in v32, so they just need `xrom_modules: 31`)
- Keep: all v3.2 fields including `time_offset_secs`, `clock_12h`, `rand_seed: "0.5"`, `xrom_modules: 31`
- Must NOT contain: `xmem_files`, `xmem_active_file`
- `xrom_modules` must be `31` (0b0001_1111) — the v3.3 value after `migrate_after_load()`

**Confirmed v32-autosave.json structure** — it contains exactly: stack, regs (100 zeros), alpha_reg, alpha_mode, angle_mode, display_mode, entry_buf, program, prgm_mode, pc, call_stack, is_running, user_mode, key_assignments, assignments, text_regs, last_key_code, reg_m/n/o, flags, `xrom_modules: 7`, complex_mode, `rand_seed: "0.5"`, matrix_dim, matrix_active_reg, `time_offset_secs: 3600`, `clock_12h: true`.

**v33-autosave.json** should be identical to v32 but with `xrom_modules: 31` (v3.3 migrates bits 3+4 on). No Advantage Pac fields were in v32, so none to remove. The v33 fixture is a "real" v3.3 CalcState pre-Phase 51 — it has the v3.3 XROM modules but no X-MEM fields. [VERIFIED: codebase read of v32-autosave.json]

**Test structure** (mirror `adv_backward_compat.rs`):
```rust
static V33_FIXTURE: &str = include_str!("fixtures/v33-autosave.json");

#[test]
fn v33_save_loads_without_error() {
    let state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    assert!(state.xmem_files.is_empty());
    assert!(state.xmem_active_file.is_none());
}

#[test]
fn v33_save_migrate_after_load_is_idempotent() {
    let mut state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    state.migrate_after_load();
    // xrom_modules stays 31 (no new bits to set in v4.0)
    assert_eq!(state.xrom_modules, 0b0001_1111u8);
    // X-MEM fields still default
    assert!(state.xmem_files.is_empty());
    assert!(state.xmem_active_file.is_none());
}

#[test]
fn v33_save_xmem_fields_default_cleanly() {
    // Proves serde(default) path — field absent in JSON → empty/None
    let state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    assert!(state.xmem_files.is_empty(), "xmem_files must default to empty Vec");
    assert!(state.xmem_active_file.is_none(), "xmem_active_file must default to None");
}

#[test]
fn v33_save_rand_seed_preserved() {
    // rand_seed must survive without #[serde(skip)] (Pitfall 20 guard)
    let state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    let expected = Decimal::from_str("0.5").expect("literal parses");
    assert_eq!(state.rand_seed.inner(), expected);
}
```

### Pattern 5: Isolation Tests (D-52.11 / XMEM-09)

**What:** Targeted tests proving X-MEM ops read/write only `xmem_files`/`xmem_active_file`, never `state.regs` (except the sanctioned SAVED→capture_data_card and GETD→load_data_card transfers).

**Precedent:** The D-43.5 isolation pattern in `adv_backward_compat.rs` simply asserts `state.adv_matrices.is_empty()` after loading a fixture — proving the field stayed isolated. For X-MEM the isolation is more active (SAVED explicitly reads `state.regs`; GETD explicitly writes `state.regs`), so the test must:
1. Verify that EMDIR/EMROOM/SAVEP/GETP/EMREG/SAVERX do NOT modify `state.regs`.
2. Verify that SAVED reads `state.regs` correctly (expected transfer).
3. Verify that GETD writes `state.regs` correctly (expected transfer).
4. Verify that no X-MEM op ever reads/writes `state.adv_matrices`.

**Pattern:**
```rust
#[test]
fn xmem_ops_never_touch_adv_matrices() {
    let mut state = CalcState::default();
    // adv_matrices is empty; run all X-MEM ops and assert it stays empty
    let alpha = "TESTFILE".to_string();
    state.alpha_reg = alpha;
    let _ = op_savep(&mut state);
    let _ = op_emdir(&mut state);
    let _ = op_emroom(&mut state);
    assert!(state.adv_matrices.is_empty(), "X-MEM ops must not create adv_matrices entries");
}

#[test]
fn savep_getp_do_not_touch_state_regs() {
    let mut state = CalcState::default();
    state.regs[0] = HpNum::from(42i32);
    state.alpha_reg = "PROG".to_string();
    let _ = op_savep(&mut state);
    let _ = op_getp(&mut state);
    assert_eq!(state.regs[0], HpNum::from(42i32), "SAVEP/GETP must not modify state.regs");
}

#[test]
fn saved_getd_transfer_is_isolated() {
    let mut state = CalcState::default();
    state.regs[5] = HpNum::from(99i32);
    state.alpha_reg = "DATAFILE".to_string();
    op_saved(&mut state).unwrap();
    // Clear regs and verify GETD restores them
    state.regs[5] = HpNum::from(0i32);
    op_getd(&mut state).unwrap();
    assert_eq!(state.regs[5], HpNum::from(99i32), "GETD must restore reg[5]");
}
```

These can live in `hp41-core/tests/xmem_backward_compat.rs` (combined with the backward-compat tests) or in a separate `hp41-core/tests/xmem_isolation.rs` file.

### Pattern 6: Meta-gate Extension (D-52.13)

**What:** Extend `hp41-cli/tests/function_matrix_parity.rs` with an `XMEM_OP_VARIANT_NAMES` constant + 3 tests (inventory sentinel + forward parity + reverse parity).

**Critical difference from XROM parity tests:** X-MEM ops are resolved by `builtin_card_op()`, not `xrom_resolve()`. The reverse parity test must call `builtin_card_op(display_name)` instead of `xrom_resolve(display_name, bits)`.

**Pattern:**
```rust
const XMEM_OP_VARIANT_NAMES: &[&str] = &[
    "EmDir", "EmRoom", "SaveP", "GetP", "SaveD", "GetD", "EmReg", "SaveRx",
];

#[test]
fn test_xmem_op_inventory_count() {
    assert_eq!(XMEM_OP_VARIANT_NAMES.len(), 8,
        "XMEM_OP_VARIANT_NAMES drift — expected 8 X-MEM Op variants");
}

#[test]
fn test_every_xmem_op_has_xmem_json_entry() {
    let json_variants: HashSet<&str> = help_entries_xmem()
        .iter().map(|e| e.op_variant.as_str()).collect();
    let mut missing: Vec<&str> = Vec::new();
    for name in XMEM_OP_VARIANT_NAMES {
        if !json_variants.contains(name) { missing.push(name); }
    }
    assert!(missing.is_empty(), "X-MEM Op::* variants missing from docs/hp41-xmem-functions.json: {missing:?}");
}

#[test]
fn test_every_xmem_json_entry_has_builtin_resolver_match() {
    // X-MEM is NOT resolved by xrom_resolve — uses builtin_card_op instead.
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_xmem() {
        let resolved = hp41_core::ops::program::builtin_card_op(entry.display_name.as_str());
        if resolved.is_none() {
            orphans.push(format!("'{}' — not found in builtin_card_op", entry.display_name));
        }
    }
    assert!(orphans.is_empty(), "X-MEM JSON display_names not resolved by builtin_card_op: {orphans:?}");
}
```

**Pool partition test update:** `test_pool_partition_is_exhaustive` at line 515 of `function_matrix_parity.rs` currently recognizes module_id `None` (built-ins), 7, 2, 26, 22, 24 and fails on any other value. X-MEM entries have `xrom: None` (module_id = None), so they WILL be counted in the `builtin_count` bucket. The test currently asserts `builtin_count >= 130`. Adding 8 X-MEM entries to `help_entries_all()` makes `builtin_count` >= 138. The `>= 130` assertion still passes without any change. No update to the partition test is needed. [VERIFIED: by reading the test logic and confirming X-MEM entries have xrom=None]

**Per-op test-count floor (D-52.13 ≥5 gate):** The `xrom_op_test_count.rs` file scans source files for Op variant mentions (by `Op::Variant` or `op_snake_name` tokens). X-MEM ops are NOT XROM — they will not appear in `xrom.rs` and are not collected by the existing 5 collector functions. Two options for the per-op floor:
1. Add a new `collect_xmem_variant_names()` function to `xrom_op_test_count.rs` that scans `hp41-core/src/ops/xmem/ops.rs` (which houses all 15 inline tests and all 8 op function definitions). This is the cleanest extension.
2. Add a standalone `hp41-core/tests/xmem_op_test_count.rs` file.

Option 1 is preferred (consistent with the unified meta-gate file discipline from Phase 47). Current inline test counts from the codebase scan:
- EmDir: 2 mentions (1 test fn) — BELOW floor of 5, needs external tests
- EmRoom: 4 mentions (2 test fns) — BELOW floor of 5
- SaveP: 8 mentions (3 test fns, incl. no_room) — meets floor
- GetP: 5 mentions — meets floor
- SaveD: 6 mentions — meets floor
- GetD: 3 mentions — BELOW floor
- EmReg: 5 mentions — meets floor
- SaveRx: 2 mentions — BELOW floor (emreg_saverx is 1 combined test)

Phase 52 must add external integration tests to bring EmDir, EmRoom, GetD, and SaveRx to ≥5 test mentions each.

### Pattern 7: docs-matrix Extension (D-52.2)

**What:** The `scripts/docs-matrix/src/main.rs` renders a markdown table from any JSON file. It already has title dispatch by filename. Adding X-MEM requires:
1. Add a new filename arm in `render_markdown` for `"hp41-xmem-functions.json"`.
2. Add two lines to `docs-matrix` recipe in `justfile`.
3. Add two lines to `docs-matrix-check` recipe.

**Pattern for main.rs title dispatch** (add after the advantage arm, line ~79):
```rust
} else if basename.ends_with("hp41-xmem-functions.json") {
    ("# HP-41CX Extended Memory Function Matrix", "`docs/hp41-xmem-functions.json`")
}
```

**Pattern for justfile** (add to both `docs-matrix` and `docs-matrix-check` recipes):
```just
cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
    docs/hp41-xmem-functions.json docs/hp41-xmem-function-matrix.md
```

**X-MEM entries have no `xrom` field** → `has_xrom` will be false → the docs-matrix script renders the non-XROM table format (no XROM column). This is correct.

### Anti-Patterns to Avoid

- **Anti-pattern:** Adding X-MEM mnemonic arms to `key_map::resolve()` in the GUI. Not needed — the `xeq_<name>` strip_prefix arm at line 427 already handles any XEQ-by-name target. Adding explicit arms would create two resolution paths and violate the never-discard principle if they diverge.
- **Anti-pattern:** Setting `xrom: { module: "Extended Memory", module_id: N }` on X-MEM JSON entries. X-MEM is NOT an XROM. Setting an xrom field would (a) put them in the wrong partition in `test_pool_partition_is_exhaustive`, (b) exclude them from the right-panel, and (c) misrepresent the HP-41CX architecture.
- **Anti-pattern:** Folding X-MEM entries into `docs/hp41cv-functions.json`. D-52.1 is locked against this. Folding would break the partition test (`builtin_count` would grow by 8 and no sentinel guards it), conflate the `?` overlay section, and prevent separate op↔JSON parity.
- **Anti-pattern:** Calling `xrom_resolve` in the X-MEM reverse parity test. X-MEM names are in `builtin_card_op`, not `xrom_resolve`. The reverse parity test must use `builtin_card_op`.
- **Anti-pattern:** Adding `#[serde(skip)]` to `xmem_active_file`. This field is persistent by design (D-51.4) — EMREG/SAVERX must survive save/load. The existing code already has `#[serde(default)]` WITHOUT skip, matching the `adv_tvm_state` / `rand_seed` precedent (CLAUDE.md Pitfall 20).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Lazy singleton JSON parse | Custom atomic init | `std::sync::OnceLock::get_or_init` | Already established; thread-safe; zero deps |
| XEQ-by-name dispatch | New resolver function | Extend `builtin_card_op()` | The established single-dispatch-table pattern; CLI + GUI both delegate here |
| Matrix doc generation | New template renderer | Extend `scripts/docs-matrix/src/main.rs` | Already handles N JSON files; adding one more arm is trivial |
| Program round-trip | Custom bytecode format | `encode_program`/`decode_program` in `cardreader/raw.rs` | Already tested; SAVEP/GETP already use it |
| Register capture | Custom vec clone | `capture_data_card`/`load_data_card` in `cardreader/mod.rs` | Already tested; handles MIN_REGS_AFTER_LOAD = 100 correctly |

**Key insight:** This phase is purely wiring and proof. Every capability it needs already exists in tested form — the work is connecting it and proving the connections hold.

---

## Runtime State Inventory

> Not applicable. This is a greenfield integration phase; no rename/refactor/migration of existing runtime state is involved. The `xmem_files`/`xmem_active_file` fields are new fields with serde defaults — not renamed from anything.

**Nothing found in any category:** verified — this is not a rename/refactor phase.

---

## Common Pitfalls

### Pitfall 1: Wrong Resolver in the X-MEM Reverse Parity Test
**What goes wrong:** Calling `xrom_resolve(display_name, bits)` in the X-MEM reverse parity test — it always returns None, making the test report all 8 entries as orphans.
**Why it happens:** XROM parity tests use `xrom_resolve`; X-MEM uses `builtin_card_op`. The test structure looks identical but the resolver is different.
**How to avoid:** Use `hp41_core::ops::program::builtin_card_op(entry.display_name.as_str())` in the X-MEM reverse parity test. This function is already `pub`.
**Warning signs:** Reverse parity test reports all 8 entries as unresolvable immediately on first run.

### Pitfall 2: xrom Field on X-MEM JSON Entries Breaks Partition Test
**What goes wrong:** If X-MEM JSON entries accidentally include an `xrom` field (e.g. `"xrom": { "module_id": 99 }`), `test_pool_partition_is_exhaustive` will fail with "Unknown xrom.module_id values".
**Why it happens:** The partition test recognizes only module_id in {None, 7, 2, 26, 22, 24}. Any other value (including a hypothetical X-MEM module_id) triggers the rogue path.
**How to avoid:** X-MEM JSON entries must NOT have an `xrom` field. They are OS built-ins — their `xrom` deserializes as `None` via `#[serde(default)]`.
**Warning signs:** `test_pool_partition_is_exhaustive` fails immediately with "Unknown xrom.module_id" in error message.

### Pitfall 3: Missing migrate_after_load Arm Concern
**What goes wrong:** Planning team wastes time adding a migration arm to `migrate_after_load()` that isn't needed.
**Why it happens:** Every prior module (Stat 1, Time, Advantage) required a migration arm to flip an XROM bit. X-MEM has no XROM bit — it's a built-in, not a module.
**How to avoid:** Confirm: `migrate_after_load()` in `state.rs` at line 570–602 currently has 4 arms (v3.0→v3.1, v3.1→v3.2, v3.2→v3.3 bits 3+4, and stopwatch freeze). No v3.3→v4.0 arm is needed. The `xmem_files`/`xmem_active_file` fields default via `#[serde(default)]` — that IS the migration. [VERIFIED: read state.rs lines 570-602]
**Warning signs:** None — this is a non-issue if the planner reads the existing migrate_after_load implementation.

### Pitfall 4: Coverage Floor for EmDir/EmRoom/GetD/SaveRx
**What goes wrong:** The `xrom_op_test_count.rs` ≥5-mention gate (if extended to X-MEM) fails for EmDir (2 mentions), EmRoom (4 mentions), GetD (3 mentions), SaveRx (2 mentions).
**Why it happens:** Phase 51 shipped minimal inline tests; the per-op floor of 5 was not a Phase 51 target.
**How to avoid:** Phase 52 must add targeted integration tests in `xmem_backward_compat.rs` or a new `xmem_ops_tests.rs` that bring all 8 ops to ≥5 mentions. Do NOT add the per-op floor gate BEFORE adding the additional tests — that would make CI fail immediately. Order: add tests first, then gate.
**Warning signs:** The per-op floor gate fires on first run if added before supplementary tests.

### Pitfall 5: example/notes Fields Silently Dropped by docs-matrix
**What goes wrong:** The D-52.3 requirement to add worked examples to X-MEM JSON entries is satisfied in the JSON, but the matrix doc and overlay display do not show them.
**Why it happens:** `HelpEntry` in `help_data.rs` does not declare `example`/`notes` fields — serde ignores unknown fields (no `#[serde(deny_unknown_fields)]`). The `docs-matrix` `Entry` struct also lacks these fields.
**How to avoid:** If the planner wants examples visible in the matrix doc and overlay, add `example: Option<String>` and `notes: Option<String>` with `#[serde(default)]` to both `HelpEntry` (help_data.rs) and `Entry` (docs-matrix/src/main.rs), and extend the render function. This is optional — D-52.3 specifies the JSON must contain these fields; display in overlay/matrix is Claude's discretion. The Phase 49 searchable reference (separate system) auto-picks them up if it reads the JSON.
**Warning signs:** Examples are present in JSON but absent from `just docs-matrix` output or overlay.

### Pitfall 6: GUI help_data.rs Duplication
**What goes wrong:** The CLI `help_data.rs` is extended for the X-MEM pool, but the GUI has its own `help_data.rs` equivalent that is not updated.
**Why it happens:** The project has parallel help_data implementations (CLI and GUI, by design per D-25.6 / SC-4).
**How to avoid:** Confirm whether the GUI has its own `help_data.rs`. If yes, apply the same pool extension there. [VERIFY: check hp41-gui/src-tauri/src/ for help_data.rs]
**Warning signs:** CLI `?` overlay shows X-MEM entries; GUI `?` overlay does not.

---

## Code Examples

### builtin_card_op current end (confirmed)
```rust
// Source: hp41-core/src/ops/program.rs, lines 1444-1449
        "PRSTK" => Some(Op::PRSTK),
        _ => None,
    }
}
```

### OnceLock pool pattern (confirmed from help_data.rs lines 105-122)
```rust
const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");
static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}
```

### v32-autosave.json structure (confirmed — this is the template for v33)
```json
{
  "stack": { "x":"0","y":"0","z":"0","t":"0","lastx":"0","lift_enabled":true },
  "regs": [100 zero entries],
  "xrom_modules": 7,      ← change to 31 for v33
  "rand_seed": "0.5",
  "time_offset_secs": 3600,
  "clock_12h": true
  // NO xmem_files, NO xmem_active_file
}
```

### adv_backward_compat.rs pattern (confirmed lines 28-49)
```rust
static V32_FIXTURE: &str = include_str!("fixtures/v32-autosave.json");

#[test]
fn v32_save_advantage_fields_default_cleanly() {
    let mut state: CalcState = serde_json::from_str(V32_FIXTURE).expect("v32 fixture deserializes");
    state.migrate_after_load();
    assert!(state.adv_matrices.is_empty());
    assert_eq!(state.adv_matrix_i, 0u8);
}
```

### function_matrix_parity XMEM pattern (how to add — new in Phase 52)
```rust
// Add to hp41-cli/tests/function_matrix_parity.rs
use hp41_cli::help_data::help_entries_xmem;

const XMEM_OP_VARIANT_NAMES: &[&str] = &[
    "EmDir", "EmRoom", "SaveP", "GetP", "SaveD", "GetD", "EmReg", "SaveRx",
];

#[test]
fn test_xmem_op_inventory_count() {
    assert_eq!(XMEM_OP_VARIANT_NAMES.len(), 8, "...");
}
// + forward parity test using help_entries_xmem()
// + reverse parity test using builtin_card_op (NOT xrom_resolve)
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single JSON pool (hp41cv-functions.json) | 5 pools, each a separate OnceLock | Phase 28 (Math 1), extended through Phase 44 | Phase 52 adds pool #6 using the same mechanical pattern |
| XROM resolver for module functions | `builtin_card_op` for OS built-ins | Phase 25 established this distinction | X-MEM uses `builtin_card_op`; all 5 XROM modules use `xrom_resolve` |
| Per-module meta-gate files | Unified `xrom_op_test_count.rs` (Phase 47) | Phase 47 | X-MEM per-op floor gate should extend `xrom_op_test_count.rs` (or spawn `xmem_op_test_count.rs` if XROM purity is preferred) |

**Deprecated/outdated:**
- Nothing deprecated by this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | GUI has its own `help_data.rs` that needs to be extended for the X-MEM pool | Architecture Patterns, Pitfall 6 | If GUI shares the CLI help_data.rs, only one file needs updating — easier. If GUI has no help overlay, the GUI pool update is irrelevant. | 
| A2 | `example` and `notes` fields in JSON are silently ignored by serde (no `deny_unknown_fields`) | Pattern 3 / Pitfall 5 | If `#[serde(deny_unknown_fields)]` were added in future, X-MEM JSON with extra fields would panic on load — but this attr is not present now [VERIFIED: help_data.rs read] |
| A3 | The docs-matrix `Entry` struct uses `#[serde(default)]` on `xrom` and `divergences` but lacks `example`/`notes` | Pattern 7 / Pitfall 5 | If docs-matrix panics on unknown fields, the xmem matrix generation fails [LOW risk — serde ignores unknown by default] |

**Note on A1:** Confirmed by the project architecture (CLI and GUI are parallel). The question is whether the GUI `help_data.rs` is at `hp41-gui/src-tauri/src/help_data.rs` — this file was not directly read during research. **Planner should verify** by grepping for `OnceLock` in `hp41-gui/src-tauri/src/` before planning the GUI help_data task.

---

## Open Questions

1. **Does the GUI have its own help_data.rs?**
   - What we know: CLI has `hp41-cli/src/help_data.rs` with 5 pools. GUI has its own `?` overlay (Phase 31).
   - What's unclear: Whether the GUI reads the same JSON via its own OnceLock, or delegates to the CLI crate.
   - Recommendation: `grep -rn "OnceLock\|include_str.*functions.json" hp41-gui/src-tauri/src/` before planning.

2. **Should the per-op test-count floor gate extend `xrom_op_test_count.rs` or live in a new file?**
   - What we know: `xrom_op_test_count.rs` is the unified meta-gate for the 5 XROM modules. X-MEM is not XROM.
   - What's unclear: Whether mixing X-MEM into a file named `xrom_op_test_count.rs` creates naming confusion.
   - Recommendation: Extend `xrom_op_test_count.rs` with a comment block explaining X-MEM is "built-in, not XROM, but same test discipline"; rename the file in a future cleanup if needed.

3. **ADR numbering for v4.0 ADRs**
   - What we know: Last ADR is `v3.3-004-math1-visibility-promotion-policy.md`. v4.0 convention would be `v4.0-001-...`.
   - What's unclear: Whether the planner should follow strict sequence or use descriptive suffixes.
   - Recommendation: Follow `v4.0-001-xmem-os-builtin.md`, `v4.0-002-xmem-capacity.md`, `v4.0-003-xmem-register-transfer.md` — consistent with prior per-version numbering.

---

## Environment Availability

> Step 2.6: SKIPPED — this phase installs no external tools or services. All changes are pure Rust source code, JSON, Markdown, and justfile within the existing workspace. The existing CI infrastructure (`just test`, `just docs-matrix-check`, `just lint`) is already available. [VERIFIED: no new deps, no new tools]

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `cargo test` |
| Config file | none (workspace uses Cargo.toml test settings) |
| Quick run command | `just test-core --test xmem_backward_compat` |
| Full suite command | `just test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| XMEM-08 | v3.3 save file without xmem fields loads without error | integration | `cargo test -p hp41-core --test xmem_backward_compat` | ❌ Wave 0 |
| XMEM-08 | `xmem_files` defaults to empty Vec | integration | included in above | ❌ Wave 0 |
| XMEM-08 | `xmem_active_file` defaults to None | integration | included in above | ❌ Wave 0 |
| XMEM-09 | X-MEM ops never read/write `state.regs` (except SAVED/GETD) | integration | `cargo test -p hp41-core --test xmem_backward_compat` | ❌ Wave 0 |
| XMEM-09 | X-MEM ops never read/write `adv_matrices` | integration | included in above | ❌ Wave 0 |
| XMEM-10 | `XEQ "EMDIR"` resolves in CLI | unit | `cargo test -p hp41-cli --test xeq_builtin_resolver` (extend) | ✅ (extend) |
| XMEM-10 | `XEQ "SAVEP"` and all 8 ops resolve via builtin_card_op | unit | `cargo test -p hp41-core` (builtin_card_op_resolves_four_names — extend) | ✅ (extend) |
| XMEM-10 | All 8 X-MEM ops appear in `?` overlay (help_entries_all) | unit | `cargo test -p hp41-cli --test phase52_help_data_xmem` | ❌ Wave 0 |
| XMEM-10 | Op↔JSON parity: 8 variants ↔ 8 JSON entries | integration | `cargo test -p hp41-cli --test function_matrix_parity` (extend) | ✅ (extend) |
| XMEM-10 | Per-op test-count floor ≥5 for all 8 ops | meta-gate | `cargo test -p hp41-core --test xrom_op_test_count` (extend) | ✅ (extend) |
| XMEM-10 | `just docs-matrix-check` passes with xmem pool | drift-catch | `just docs-matrix-check` | ✅ (extend justfile) |

### Sampling Rate
- **Per task commit:** `just test-core --test xmem_backward_compat` (new tests only, ~seconds)
- **Per wave merge:** `just test` (full suite)
- **Phase gate:** `just test && just docs-matrix-check && just lint` — all green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/tests/fixtures/v33-autosave.json` — backward-compat fixture (XMEM-08)
- [ ] `hp41-core/tests/xmem_backward_compat.rs` — backward-compat + isolation tests (XMEM-08, XMEM-09)
- [ ] `hp41-cli/tests/phase52_help_data_xmem.rs` — 8-entry pool smoke test (XMEM-10)
- [ ] `docs/hp41-xmem-functions.json` — the JSON pool itself (required before any help or parity tests compile)
- [ ] `docs/hp41-xmem-function-matrix.md` — generated by `just docs-matrix` after JSON exists

---

## Security Domain

> This phase makes no changes to authentication, session management, access control, cryptography, or network-facing code. It adds JSON data files, Rust test files, and Markdown docs. No ASVS categories apply. Security domain: N/A.

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/ops/program.rs` — `builtin_card_op()` at line 1368, confirmed exact function signature and current end-of-match at line 1447 [VERIFIED: codebase read]
- `hp41-cli/src/help_data.rs` — all 5 OnceLock pool declarations + `help_entries_all()` chain, confirmed lines 85-220 [VERIFIED: codebase read]
- `hp41-cli/src/keys.rs` — `xeq_by_name_local_resolve()` at line 369, delegates to `builtin_card_op` [VERIFIED: codebase read]
- `hp41-gui/src-tauri/src/key_map.rs` — `resolve()` and `resolve_parameterized()`, `xeq_<name>` strip_prefix at line 427 [VERIFIED: codebase read]
- `hp41-core/src/state.rs` — `xmem_files` at line 432, `xmem_active_file` at line 442, `migrate_after_load()` at line 570 — no new arm needed [VERIFIED: codebase read]
- `hp41-core/src/ops/xmem/mod.rs` and `ops.rs` — `XmemFile`, `XMEM_CAPACITY=600`, 8 op implementations, 15 inline tests [VERIFIED: codebase read]
- `hp41-core/tests/adv_backward_compat.rs` — direct template for `xmem_backward_compat.rs` [VERIFIED: codebase read]
- `hp41-core/tests/xrom_op_test_count.rs` — per-op test-count floor pattern, confirmed scan strategy [VERIFIED: codebase read]
- `hp41-cli/tests/function_matrix_parity.rs` — all existing parity test patterns, confirmed pool partition test at line 515 [VERIFIED: codebase read]
- `hp41-cli/tests/xeq_builtin_resolver.rs` — XEQ-by-name resolver test structure [VERIFIED: codebase read]
- `hp41-core/tests/fixtures/v32-autosave.json` — exact v3.2 fixture structure for v33 template [VERIFIED: codebase read]
- `docs/hp41-advantage-functions.json` — JSON entry schema with `example`/`notes` fields [VERIFIED: codebase read]
- `scripts/docs-matrix/src/main.rs` — generator entry schema + title dispatch mechanism [VERIFIED: codebase read]
- `justfile` lines 185-220 — `docs-matrix` and `docs-matrix-check` recipes [VERIFIED: codebase read]

### Secondary (MEDIUM confidence)
- `docs/adr/` directory listing — v4.0 ADR numbering convention (v4.0-001, v4.0-002, ...) [CITED: directory listing]

### Tertiary (LOW confidence)
- None — all claims are verified from codebase.

---

## Metadata

**Confidence breakdown:**
- Integration points (builtin_card_op, key_map, help_data): HIGH — direct code read with line numbers
- Backward-compat mechanics: HIGH — read v32-autosave.json and migrate_after_load source
- Meta-gate patterns: HIGH — read function_matrix_parity.rs and xrom_op_test_count.rs in full
- Test-count floor gaps: HIGH — direct grep-based count per op
- docs-matrix extension: HIGH — read main.rs and justfile

**Research date:** 2026-05-28
**Valid until:** 2026-06-28 (stable project, 30-day horizon)
