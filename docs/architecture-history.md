# HP-41 Emulator — Architecture History & Decision Narrative

This file is the **long-form companion to `CLAUDE.md`**. `CLAUDE.md` carries the frozen invariants every change must respect today; this file documents *how each invariant came to be*, phase by phase, so future contributors can reason about whether a proposed change is breaking new ground or stepping on a hard-won lesson.

For semantic search across this content, prefer:

```bash
gbrain search "<terms>" --source gstack-code-hp41-calculator-emulator
gbrain search "<terms>" --source gstack-brain-daniel
```

Authoritative source of decision rationale: `docs/adr/` (numbered ADRs for v3.0 onward).
Authoritative Math Pac I divergences: `docs/hp41-math1-divergences.md`.

---

## Milestone Status

| Version | Title | Shipped | Phases | Tag |
|---------|-------|---------|--------|-----|
| v1.0 | CLI Foundation | 2026-05-08 | 1–8 (45 plans) | v1.0 |
| v1.1 | CLI Feature Completeness | 2026-05-09 | 9–12 (13 plans) | v1.1 |
| v2.0 | Tauri GUI | 2026-05-10 | 13–18 (19 plans) | v2.0 |
| v2.1 | Keyboard Authenticity | 2026-05-13 | 19 (10 tasks, quick-task reconcile) | — |
| v2.2 | HP-41CV Feature Completeness | 2026-05-16 | 20–27 (8 phases) | v2.2 |
| v3.0 | Math Pac I Emulation | 2026-05-21 | 28–32 + polish batch | pending |

**Scope axis:** v2.x = ROM-Built-ins (~130 functions on the physical HP-41CV). v3.x = Module emulation (Math Pac I in v3.0; Stat / Time / Advantage queued for v3.1+).

---

## Phase History

- **v1.0 (1–8):** Foundation → Core Math → Programming Engine → TUI & Input → Persistence & UX → Science & Engineering → Hardening → Tech Debt Cleanup
- **v1.1 (9–12):** Infrastructure & EEX Fix → STO Arithmetic Modals → Print Emulation → Synthetic Programming
- **v2.0 (13–18):** Workspace Skeleton → IPC Layer → Display & Keyboard → SVG Skin → Persistence & Print Output → Program Listing & CI/CD
- **v2.1 (19):** Keyboard Authenticity
- **v2.2 (20–27):** Core Math; Flags/Display/Sound; Program Control & Memory; ALPHA; Indirect Addressing; CLI Integration & JSON pipeline; GUI Integration & Polish; Test Hardening
- **v3.0 (28–32):** XROM Framework + Math Pac I Core Ops; CLI Integration; Documentation & ADRs; GUI Integration; Test Hardening & Quality Gates

---

## Settled Architecture Decisions (per milestone)

These decisions are final — do not revisit without strong justification. The `CLAUDE.md` "Frozen Invariants" section summarises the load-bearing ones; below is the full rationale.

### Core engine (v1.0)

- **BCD/f64:** `rust_decimal` wrapping f64 with 10-significant-digit rounding. Custom BCD was evaluated and rejected. `HpNum` in `hp41-core/src/num.rs`.
- **Stack-lift:** `lift_enabled: bool` in `Stack`. Every one of ~130 operations declares `LiftEffect::Enable / Disable / Neutral` in `ops/`. The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** Fields extracted by string-splitting at the decimal point — **never** `floor()`/`fmod()` on f64. See `ops/program.rs::parse_counter()`.
- **TUI:** Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook. Filter `KeyEventKind::Release` on Windows immediately or every op fires twice.
- **No async in core:** Event loop is `poll(timeout) → update → redraw`, single-threaded throughout. The hp41-gui spawns a separate auto-save thread but `hp41-core` itself stays single-threaded.
- **Zero panics in `hp41-core`:** `#![deny(clippy::unwrap_used)]` is active at the crate root (`hp41-core/src/lib.rs`). All production code must use `.expect("reason")` or proper `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`. Mutex locks in `hp41-gui` use `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery.
- **Key bindings (Phase 8):** `'q'` → `Op::Sin`, `'g'` → `Op::Clreg`, `Delete` in ALPHA mode → `Op::AlphaClear`. `'S'` opens STO register modal (handled before `key_to_op()`). Quit is `Ctrl+C` only.
- **Coverage gate:** `just coverage` runs `cargo llvm-cov clean --workspace` first to discard stale `.profraw` data from worktree runs before measuring.

### v1.1 additions

- **EEX trailing-e (hardware-faithful):** `flush_entry_buf()` appends `"00"` before the parse chain (`Decimal::from_str` → `Decimal::from_scientific`); empty-buffer EEX inserts implicit mantissa `"1"`; `format_entry_buf_display()` in `hp41-cli/src/ui.rs` renders the underscore placeholder cursor. **Never** discard a trailing-e number silently.
- **STO arithmetic modal:** 3-step keyboard flow `S → +/−/×/÷ → R00–R99 | Y/Z/T/L`. `StackReg` enum + `Op::StoArithStack` variant in `ops/mod.rs`; `op_sto_arith_stack()` in `registers.rs`. `pending_input` routing block must remain ABOVE modal-opening interceptors (`S`/`R`/`Ctrl+A`) so an active modal is not silently discarded.
- **Print emulation:** `print_buffer: Vec<String>` field on `CalcState` with `#[serde(skip)]` (transient, never persisted). `ops/print.rs::{PRX, PRA, PRSTK}` push lines into the buffer; `println!`/`eprintln!` are forbidden inside `hp41-core`. `hp41-cli` drains via `call_dispatch_and_drain()` (interactive) and `drain_and_show_print_output()` (programmatic `run_program` paths — wire ALL `run_program()` call sites or print output gets dropped).
- **Synthetic programming:** `last_key_code`, `reg_m`, `reg_n`, `reg_o` fields on `CalcState`, all with `#[serde(default)]` for backward-compat with v1.0 save files. `HexModal(String)` 2-digit accumulation modal; `synthetic_byte_to_op()` validates against the 23-entry safe subset **before** `state.program.insert()` (security invariant T-12-W2-02). `keycode_to_hp41_code()` in `hp41-cli/src/keys.rs` uses row×10+col encoding. F5 / R / S code paths reset `last_key_code` to 0 BEFORE GETKEY runs.
- **MSRV:** declared at `[workspace.package]` (`rust-version = "1.88"`); member crates inherit via `rust-version.workspace = true`. CI MSRV job runs in parallel — no `needs:`.

### v2.0 additions (Tauri GUI)

- **Nested workspace isolation:** Root `Cargo.toml` `members = ["hp41-core", "hp41-cli"]` — never add `hp41-gui` here. `tauri` / `tauri-build` must appear ONLY in `hp41-gui/src-tauri/Cargo.toml`, never in root `[workspace.dependencies]`.
- **Bundle identifier:** `ch.talent-factory.hp41` (overrides scaffold default `com.tauri.dev`; avoids macOS sandbox/keychain issues).
- **IPC contract:** `dispatch_op(key_id: &str)` and `get_state()` Tauri v2 commands; response is `CalcStateView` (~170 bytes, JSON budget ≤300 bytes). `key_map::resolve()` in `hp41-gui/src-tauri/src/key_map.rs` maps string IDs to `Op` variants — frontend never touches Rust enums. `print_buffer` is drained on every command response.
- **SC-4 invariant (no core duplication):** the spirit is "no calculator/math logic duplicated in hp41-gui". The literal grep `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` currently matches `fn op_display_name(...)` in `prgm_display.rs` — that function is a display formatter, not calculator logic, so the spirit is preserved. When checking SC-4 manually, use the stricter pattern `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` which excludes display helpers. Note: `op_display_name` is duplicated in both `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` — every new `Op` variant must be added in both copies.
- **Tauri v2.11 permissions:** For inline app commands (not plugins), Tauri does NOT auto-generate `allow-<cmd>` permissions. Create TOML in `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml` with `[[permission]] identifier + commands.allow = ["fn_name"]`, then reference the kebab-case ID in `capabilities/default.json`. Run a `cargo check` first so the permission registry is generated.
- **SVG animation:** `transform-box: fill-box` + `transform-origin: center` on `.key` is REQUIRED for SVG `scale()` to animate from each key's own center; without it, keys translate from the canvas origin instead of shrinking in place.
- **busyRef debounce:** `useRef(false)` pattern in both `App.tsx` (handleClick) and `Keyboard.tsx` (handleKeyClick) — two-layer guard against concurrent `invoke()` calls. Always pair with `pressedKey` state machine using functional setState form to avoid stale closure (Pitfall 4).
- **Persistence sharing:** `hp41-gui` reads/writes the SAME `~/.hp41/autosave.json` file as `hp41-cli`. `serde(default)` on every `CalcState` field added since v1.0 keeps v1.x save files loadable. Auto-save thread releases the `AppState` Mutex BEFORE disk I/O (commit ff39017 fix).
- **`Op` variants land before TUI code:** Every new `Op` variant must appear in BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs` AND the `prgm_display.rs` exhaustive match before any caller (`hp41-cli` or `hp41-gui`) can compile.

### v2.1 additions (Keyboard authenticity, Phase 19)

- **Authentic 5×8 layout**: `hp41-gui/src/Keyboard.tsx` renders 4 top-row mode buttons + a 5-column × 8-row main grid (ENTER 2-wide) with one orange SHIFT key — replaces the prior 8-col landscape layout with cosmetic `f`/`g` keys. Total key count: 4 top-row + 35 main grid = 39 entries.
- **Three-label `KeyDef`**: each key carries `id`/`label` (primary), optional `shifted: { id, label }` (orange, above), and `alphaChar` (blue, below). Old `fShiftLabel` field is gone. `KeyDef` is exported from `Keyboard.tsx`.
- **One-shot SHIFT is frontend-only**: `shiftActive: boolean` lives in `App.tsx`; never appears in `CalcState`, `CalcStateView`, or IPC. After a shifted op fires, `setShiftActive(false)` resets. `Tab` and clicking SHIFT toggle; `Esc` cancels. SHIFT joins the annunciator list as a frontend-derived value.
- **Click resolution priority**: if the clicked key is SHIFT (`variant: 'shift'`), toggle `shiftActive` and return without dispatching. Otherwise resolve the effective id by `ALPHA + alphaChar` → `shiftActive + shifted.id` (consumes the one-shot) → `primary.id`. ALPHA mode overrides SHIFT (known divergence from real HP-41 — v2.2 deferred).
- **Stub-error pattern** (D-5): `key_map::resolve` returns `Err(GuiError { message: "'<id>' is planned for a future phase" })` for `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`. Also stubs `xeq_prompt`, `gto_prompt`, `lbl_prompt` explicitly — these would otherwise be silently swallowed by the label-bearing `xeq_`/`gto_`/`lbl_` prefixes in `resolve_parameterized`. Frontend surfaces a 2s toast overlay. NEVER silently discard — D-07 holds.
- **Modal-prompt ids** (`sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt`) are KEY_DEFS-only frontend ids; they fall through to `resolve_parameterized` and fail (numeric-suffix parse rejects `"prompt"`), surfacing as `unknown key: <id>` toast. v2.2 will route these to actual modals.
- **`run_stop` Tauri command**: new dedicated command symmetric with `sst_step`/`bst_step`, toggles `CalcState.is_running`. R/S key is now click-reachable for the first time (was `id: ''` in v2.0). Permission file: `permissions/run-stop.toml`. Frontend special-routes id `r_s` via the `invokeForKey` helper, NOT through `dispatch_op`. v2.1 scope is flag-toggle only; actual stepping deferred to v2.2.
- **`invokeForKey` + `extractErrMessage` helpers** in `App.tsx`: single source of truth for resolving an effective id to a Tauri command (`sst_step`/`bst_step`/`run_stop`/`dispatch_op`) and for extracting the `GuiError.message` field from Tauri rejections (`String(err)` would produce `"[object Object]"` for object-shaped errors).
- **`SST`/`BST`/`CL X/A` special routes**: `App.tsx`'s click handler routes these ids to dedicated paths — `sst`/`bst` → `sst_step`/`bst_step` (via `invokeForKey`), `clx_or_a` → `clx` or `alpha_clear` depending on `annunciators.alpha`. Adding a new such key in the future requires updating either `invokeForKey` (single-id route) or the `clx_or_a` branch in `handleClick` (alpha-aware route).
- **Toast overlay**: `App.tsx` renders `<div className="toast" role="status">{toastMsg}</div>` when `toastMsg` is set, with a 2s auto-dismiss `useEffect`. CSS lives in `App.css` with `.toast` + `@keyframes toast-fade`. Single-toast policy (newest replaces older — no queue).
- **No core/CLI changes**: Phase 19 is hp41-gui only. SC-4 invariant preserved. Save-file backward compat unchanged.

### v2.2 additions (HP-41CV Feature Completeness, Phases 20–25)

- **Phase 20–24 ROM ops landed in `hp41-core`** — ~90 new `Op` variants spanning unary math + polar conversions + RND/FRC/MOD/PI (Phase 20), flags/display-control/sound (Phase 21), program-control + memory + catalog + ASN (Phase 22), ALPHA ops (Phase 23), and the 11-variant `*Ind` indirect family (Phase 24). `hp41-core` was FROZEN from Plan 25-01 onward; the surgical 4→12 `builtin_card_op` extension in Plan 25-03 is the documented exception.
- **f-prefix one-shot model on `hp41-cli`** (D-25.1 / D-25.4): `App.shift_armed: bool` mirrors hp41-gui v2.1's `shiftActive` bit-for-bit per the D-25.6 CLI ↔ GUI parity invariant. ALPHA overrides f-prefix (D-25.5); one-shot consumption always clears `shift_armed` regardless of whether the resolver matched (Pitfall 5). Status bar prepends `f→` when armed.
- **Four conditional tests on f-arith keys** (D-25.7, hardware ground truth from the user's physical HP-41CV): `f -`/`f +`/`f *`/`f /` dispatch `Op::Test(TestKind::{XEqY,XLeY,XGtY,XEqZero})`. These four are the ONLY conditional tests on the physical keyboard; the remaining 8 ROM conditionals (X≠Y, X<Y, X≥Y, X≠0, X<0, X>0, X≤0, X≥0) route through the XEQ-by-Name modal (D-25.8 / D-25.9).
- **Hybrid `PendingInput` struct-variants** (D-25.11): `FlagPrompt { kind: FlagPromptKind, ind: bool, acc: String }` and `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }` collapse 34 logical operations (12 flag × {direct, IND} + 22 register × {direct, IND}) into 2 carrier variants. `FlagPromptKind` and `RegisterOpKind` wrap (not duplicate) `hp41_core::ops::FlagTestKind` / `StoArithKind` per D-25.13. `pending_prompt()` stays exhaustive over all 18 variants (no `_ =>`, no `unreachable!()`) — FN-CLI-04 compile-time guarantee.
- **IND-toggle via shift-0 inside an open modal** (D-25.12 / Pitfall 10): hardware-faithful per HP-41C/CV QRG p.14. The `App.shift_armed` bit is REUSED — pressing `f` arms shift inside a `FlagPrompt`/`RegisterPrompt`, then pressing `0` toggles the modal's `ind` field (not appended to the accumulator). End-of-2-digit dispatch is a single tuple-match decision point that picks `Op::*Ind(n)` vs `Op::*(n)` per D-25.12.
- **JSON-canonical data flow** (D-25.16): `docs/hp41cv-functions.json` is the single source of truth. `hp41-cli/src/help_data.rs` loads it via `include_str!` + `std::sync::OnceLock` (Phase 25-04 / Plan-04). Malformed JSON panics with `.expect("hp41cv-functions.json is malformed")` per D-25.17 (hard-build-blocker by design; Pitfall 7 mitigated by the smoke test). `scripts/docs-matrix/` (standalone non-workspace crate) regenerates `docs/hp41cv-function-matrix.md` via `just docs-matrix`; `just docs-matrix-check` is the CI drift-catch (Pitfall 8). Bidirectional Op-enum ↔ JSON parity is asserted by `hp41-cli/tests/function_matrix_parity.rs` (4 tests per Pitfall 6); key-coverage closure is `hp41-cli/tests/key_coverage.rs` per D-25.18.
- **`builtin_card_op` 4→12 extension** (D-25.8, Plan 25-03 surgical exception): the resolver in `hp41-core/src/ops/program.rs` grew from 4 v2.1 card-reader names to 4 + 8 conditional-test mnemonics (with ASCII + Unicode spellings). Visibility stays `pub(super) fn` per the W1 fix — no API widening. Mnemonic dispatch from both the keyboard (`xeq_by_name_local_resolve` fast-path) and programmatic `Op::Xeq("X<>Y?")` inside a saved program resolves identically (must_have truth #4 of Plan 03).
- **`KEY_REF_TABLE` is JSON-derived** (D-25.18): the right-panel discoverability listing in `hp41-cli/src/ui.rs::render_right_panel` reads from `help_data::help_entries()` filtered by non-null `key_path` — no parallel hand-curated table. Plan 25-04 deletes the legacy `pub const KEY_REF_TABLE` and replaces it with a derivation. Drift between bindings and discovery is caught by `key_coverage.rs` (every implemented JSON entry with `key_path != null` dispatches via `key_to_op` / `shifted_key_to_op` / modal-opener / `xeq_by_name_local_resolve` to a known `Op::` variant — no `InvalidOp`, no panics).
- **README soft-claim** (D-25.17): "feature-complete HP-41CV with documented divergences" + link to `docs/hp41cv-function-matrix.md`. Hard claim is deferred to Phase 27 conditional on coverage gate ≥95 % (FN-QUAL-01).
- **SC-4 invariant unchanged**: Phase 25 touches `hp41-cli`, `docs/`, `scripts/`, root `*.md`, and `justfile` only — NO `hp41-gui` changes (those land in Phase 26). The CLI ↔ GUI parity invariant D-25.6 is the contract Phase 26 must satisfy.
- **Save-file backward compat preserved**: NO new `CalcState` fields in Phase 25. v1.0–v2.1 save files continue to load without migration.

### v2.2 additions (Test Hardening, Phase 27)

- **Coverage gate raise (FN-QUAL-01):** `just coverage` enforces ≥ 95 % line coverage on `hp41-core` (raised atomically from 80 % per D-27.2 — gate-and-test atomicity invariant). The 5 new test files (`program_execution_coverage.rs`, `phase22_stats_size_shrink.rs`, `phase21_phase22_interactive_no_ops.rs`, `format_eng_edges.rs`, `numerical_accuracy.rs` v2.2 extension) close the gap with risk-weighted tests catching real bug classes per D-27.1, not coverage padding per D-27.3. Final achieved coverage: 95.25 % lines / 93.75 % regions. Largest single uplift: `ops/stats.rs` 84.04 % → 100 % lines via Pitfall-5 SIZE-shrink sentinels.
- **Numerical accuracy ≥ 98 % gate extended (FN-QUAL-02):** the 503-case v1.x baseline grew to 566 cases covering PI / P→R / R→P / RND / FRC / MOD / FACT per D-27.5. Quirky cases (FACT(70) → OutOfRange, MOD(7,-3) = 1 sign-follows-Y, FACT(0) = 1) carry Free42 / Owner's Manual citations per D-27.7 (27 total citations). The v1.x baseline non-regression floor is asserted independently: `baseline_passes >= 498` per D-27.6 (5 pre-existing HP-41 hardware-rounding divergences acceptable per the historical failure budget). Combined pass rate: 99.1 % (561/566).
- **E2E smoke via WebdriverIO + tauri-driver (FN-QUAL-05, D-27.15 AMENDED 2026-05-15):** the original D-27.15 named Playwright, but `tauri-driver` 2.0.6 speaks WebDriver classic which Playwright does NOT (CDP/native only). The spirit of D-27.15 (production binary + real IPC + Ubuntu only) is preserved via WebdriverIO 9.x — the Tauri v2 official E2E client. `hp41-gui/wdio.conf.cjs` spawns `tauri-driver` on `127.0.0.1:4444` with `framework: 'mocha'` and `mochaOpts.retries: 1` (D-27.16). `hp41-gui/e2e/smoke.spec.ts` clicks `2 ENTER 3 +` via `[data-key-id]` selectors and asserts `[data-testid="lcd-display"]` reads `5.0000` (literal ROADMAP scope per D-27.13 — no broader flows). Runs ONLY on Ubuntu in `.github/workflows/ci-gui.yml::e2e-linux` (ROADMAP cross-cutting line 205); macOS/Windows matrix jobs UNCHANGED. Apt deps added to both build job and e2e-linux job: `webkit2gtk-driver` (Pitfall 6) + `xvfb` (Assumption A5). Cargo bin cache (Pitfall 5) keyed on `tauri-driver-2.0.6` keeps cold installs rare. Required-for-merge is a repo branch-protection setting — manual follow-up tracked in 27-04-SUMMARY.md.
- **Vitest CI gating (D-27.14):** the 5 existing Vitest files (`App.test.tsx`, `Display14Seg.test.tsx`, `HelpOverlay.test.tsx`, `Keyboard.test.tsx`, `pending_input.test.ts`, 142 tests total) now gate on every CI push via `just gui-ci` appended `cd hp41-gui && npm test`. They pass locally since Phase 26 ship; the CI gate closes a quiet hole. `hp41-gui/vite.config.ts` `test.exclude` adds `e2e/**` so the WebdriverIO spec is not picked up by Vitest.
- **`data-testid="lcd-display"` on `Display14Seg.tsx`** (RESEARCH Pitfall 10): one-line edit (plus a `data-text={text}` fallback attribute) on the outermost `<svg>`. Allowed under SC-4 because `hp41-gui/src/` is OUTSIDE the SC-4 boundary (which constrains `hp41-gui/src-tauri/` only). The dual `data-text` is the assertion path the smoke prefers because the 14-segment LCD renders SVG path fills — no plain text content for a `toHaveText` assertion to read.
- **GUI coverage measured one-shot (D-27.4, measure-only — NOT a gate):** `cargo llvm-cov --manifest-path hp41-gui/src-tauri/Cargo.toml` reports 77.92 % lines / 78.57 % regions / 71.09 % functions on `hp41-gui/src-tauri/src/`. Vitest line coverage on `hp41-gui/src/` is intentionally NOT measured this phase — D-27.4's no-devDep clause forbids adding `@vitest/coverage-v8`. Advisory snapshot for v3.x reference only; no CI gate, no devDep added.
- **Frozen invariants preserved:** no `hp41-core/src/` source changes (frozen since Plan 25-01); no `hp41-gui/src-tauri/` source changes (SC-4 invariant); MSRV 1.88 unchanged (`tauri-driver` 2.0.6 MSRV 1.77 is compatible; WebdriverIO 9.x is a Node tool); `#![deny(clippy::unwrap_used)]` continues to apply (new test files carry `#![allow]` at file scope per the established Phase 1 onward pattern).
- **`// Catches: <bug class>` rationale (D-27.1):** every new test in Phase 27 plans 01–04 carries this doc comment naming the bug class it guards against. The `case!` macro invocations in `numerical_accuracy.rs` carry equivalent provenance via Free42 / OM citations.

### v3.0 additions (Math Pac I Emulation, Phases 28–32)

#### Phase 28 — XROM Framework + Math Pac I Core Ops (shipped 2026-05-16)

- **Op-strategy A locked (ADR-001 / C-28.1):** one `Op` variant per Math Pac I function — rejected `Op::XromCall(u16)` table dispatch (Option B) to preserve the 4-way exhaustive-match invariant (compile-time `prgm_display.rs` check) that has caught dozens of bugs since Phase 1. Full write-up: `docs/adr/v3.0-001-op-strategy.md`.
- **User-callback strict-reject policy locked (ADR-002 / C-28.2):** nested INTG/SOLVE/DIFEQ invocations are rejected at op entry with `HpError::InvalidOp` — matches Math Pac I OM 1979 hardware behaviour; avoids 4-deep `call_stack` overflow and cleanup-on-error complexity. Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 consulted only as sanity-check oracle, not copied. Full write-up: `docs/adr/v3.0-002-user-callback-policy.md`.
- **JSON-pipeline separate-file shape locked (ADR-005 / C-28.3):** separate `docs/hp41-math1-functions.json` sibling with identical schema plus `xrom: { module, module_id, function_id }` object per entry — zero migration churn on 130 v2.2 entries; aligns with future v3.1+ pacs each getting their own JSON. Full write-up: `docs/adr/v3.0-005-json-pipeline.md`.
- **`xrom_resolve` fires LAST in resolver chain (C-28.4):** after `builtin_card_op`, before `Err(InvalidOp)` — Pitfall 1 mitigation; prevents Math Pac I from shadowing existing built-in mnemonics. `tests/xrom_shadowing.rs` CI gate confirms.
- **ComplexStack overlay X/Y/Z/T (D-28.1, D-28.2):** ζ = X+iY, τ = Z+iT — OM-faithful zero-field-growth overlay; `complex_mode: bool` on `CalcState` auto-activates on first complex op, cleared by `XEQ "REAL"` (D-28.3 extension — not in OM 1979, documented in `docs/hp41-math1-divergences.md`).
- **`XEQ "REAL"` derived entry point (D-28.3):** new emulator extension to deactivate `complex_mode`; NOT in Math Pac I OM 1979; catalogued as D-30-05 in `docs/hp41-math1-divergences.md`.
- **`modal_prompt: Option<String>` dedicated field (D-28.4):** overrides XROM-09's original print_buffer wording — modal prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`) write to `modal_prompt` with `#[serde(skip)]`; `print_buffer` carries PRX/PRA/PRSTK output only. Clean lifecycle: set on prompt-open, cleared on prompt-resolve or modal-cancel.
- **R/S submits modal numeric input (D-28.5):** hardware-faithful per HP Math Pac I OM 1979 p.13 "Press R/S to continue"; reuses the existing v2.1 `run_stop` Tauri command on the GUI side; no new Op variant.
- **Hyperbolics XEQ-only — no dedicated keys (D-28.6):** SINH/COSH/TANH/ASINH/ACOSH/ATANH reachable via `XEQ "SINH"` etc. — mirrors real HP-41C with Math Pac I; f-prefix on SIN/COS/TAN is reserved for inverses (already wired in v2.2).
- **Cancellation plumbing in Phase 28; wiring in Phase 31 (D-28.7, D-28.8):** `cancel_requested: Arc<AtomicBool>` with `#[serde(skip)]` on `CalcState`; per-64-samples check in INTG/SOLVE/DIFEQ loops returns `Err(HpError::Canceled)` when set; `request_cancel` Tauri command + GUI cancel button deferred to Phase 31 / GUI-05. Zero subsequent edits to `hp41-core/src/ops/math1/` needed at Phase 31.
- **`HpError::Canceled` variant (D-28.9):** distinct from `HpError::Domain("DATA ERROR")` — cancellation is user-initiated, not a numerical failure; `Display` impl returns `"CANCELED"`; never serialized (save-file forward-compat unaffected).
- **5 new `CalcState` fields** (`xrom_modules`, `complex_mode`, `matrix_dim`, `matrix_active_reg`, `modal_prompt`, `modal_program`, `integ_state`, `solve_state`, `cancel_requested`) carry `#[serde(default)]` or `#[serde(skip)]` per the v2.2 backward-compat invariant. v1.0–v2.2 save files continue to load without migration.
- **~40 new `Op` variants** in `dispatch()` (ops/mod.rs), `execute_op()` (ops/program.rs), AND both `prgm_display.rs` copies — 4-way exhaustive-match invariant preserved per the "Op variants land before consumers" pattern documented above.

#### Phase 29 — CLI Integration (shipped 2026-05-17)

- **`xeq_by_name_local_resolve` → `xrom_resolve` (D-29.1 / C-28.4):** the CLI-local resolver in `hp41-cli/src/keys.rs` gains a final fallback into `hp41_core::ops::math1::xrom::xrom_resolve`, closing the third call site deferred by Phase 28; resolver-chain ordering (built-in card-op names win over xrom names) preserved.
- **`hp41-cli/src/help_data.rs` second `OnceLock<Vec<HelpEntry>>`:** mirrors the v2.2 D-25.16 hard-build-blocker pattern — `MATH1_HELP_ENTRIES` static, `help_entries_math1()` accessor, merged `help_entries_all()` iterator; malformed JSON panics with a distinct message; existing `phase25_help_data` tests unaffected.
- **`docs/hp41-math1-functions.json` authored in Phase 29 (D-29.1):** pulled forward from Phase 30 / DOC-01 because Phase 29 SC-2 (`?` overlay) and SC-4 (`KEY_REF_TABLE` discoverability) need real entries. ~55 entries with C-28.3 `xrom` block per entry. Phase 30 consumes the file read-only as input to the matrix-renderer.
- **~40 new `op_display_name` arms in `hp41-cli/src/prgm_display.rs`:** all Math Pac I `Op` variants added; exhaustive match maintained; no `_ =>` catch-all per FN-CLI-04 invariant.
- **`KEY_REF_TABLE` derived from JSON (D-25.18 pattern continues):** the right-panel discoverability listing in `hp41-cli/src/ui.rs::render_right_panel` reads from `help_data::help_entries_all()` via `keys::key_ref_entries()` — no parallel hand-curated table. **Post-v3.0 UX revert (2026-05-20):** the original Phase 29 SC-4 / D-29.2 / CLI-04 acceptance criterion (Math Pac I entries surface in the right-panel) is SUPERSEDED by an `entry.xrom.is_none()` filter that excludes all XROM-module functions from the panel. Once v3.0 shipped, user evaluation surfaced that the ~45 `XEQ "..."` rows crowded out the actual keyboard reference and pushed v2.2 entries off-screen on typical terminal sizes. Module functions remain fully discoverable via the `?` overlay's dedicated "Math 1 Pac (XROM 7)" section. Right-panel returns to ~15–25 rows regardless of how many XROM modules are loaded. Regression guarded by `hp41-cli/tests/phase29_key_ref_includes_math1.rs` (inverted to `key_ref_entries_excludes_math1_{sinh,matrix}`) + `hp41-cli/src/tests/keys_tests.rs::key_ref_entries_excludes_xrom_module_functions`.
- **Modal-prompt rendering via `pending_prompt()` (D-29.3):** signature widened to accept `state.modal_prompt`; renders on the existing status-bar line when `modal_program.is_some()`; LCD continues showing live X-register / `entry_buf` (D-29.4 — mirrors v2.2 RegisterPrompt UX).
- **R/S + Esc interception in `handle_key` (D-29.5, D-29.6):** R/S calls `submit_modal(state)`, Esc calls `cancel_modal(state)` (both `pub fn` in `hp41-core`) when `modal_program.is_some()`; interception happens BEFORE the v2.1 `run_stop` path; `pending_input` routing block remains ABOVE these interceptors (D-07 never-discard invariant).
- **No core/GUI changes in Phase 29:** SC-4 invariant trivially preserved; v2.2 surface unchanged.

#### Phase 30 — Documentation & ADRs (shipped 2026-05-17)

- **`scripts/docs-matrix` two-input extension (D-30.1, D-30.2, D-30.3):** justfile `docs-matrix` and `docs-matrix-check` recipes each gain a second `cargo run` invocation; binary signature stays 1-in/1-out; `Entry.xrom: Option<XromRef>` field added with `#[serde(default)]`; conditional XROM column emitted when `entries.iter().any(|e| e.xrom.is_some())` — hp41cv matrix output bit-for-bit unchanged (D-30.2 invariant).
- **`docs/hp41-math1-function-matrix.md` generated (~55 entries, D-30.2):** new sibling file; carries XROM column `Math 1 / 7-N` per entry; reachable via README v3.0 soft-claim link; `just docs-matrix-check` CI gate covers both matrices.
- **Three new ADRs (D-30.6, D-30.7):** `docs/adr/v3.0-001-op-strategy.md` (Op-strategy A vs B — A locked), `docs/adr/v3.0-002-user-callback-policy.md` (strict-reject nested INTG/SOLVE/DIFEQ; Pitfall 19 Free42 disclaim verbatim, 4 occurrences), `docs/adr/v3.0-005-json-pipeline.md` (separate `hp41-math1-functions.json` — locked); each ~6–7 KB long-form; `## Alternatives Considered` quotes Phase 28 CONTEXT.md verbatim per D-30.7.
- **`docs/hp41-math1-divergences.md` expanded (D-30.4, D-30.5):** three-bucket numbered catalog — OM Divergences (D-30-01..04), Emulator Extensions (D-30-05), Behavioral Policies (D-30-06..07); 7 entries total; 5-field shape per entry (OM citation / Our behavior / OM behavior / Rationale / See); every entry cites HP 00041-90034 or explicit "N/A — emulator extension" marker.
- **README v3.0 soft-claim (D-30.9):** `- Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry points, documented divergences)` under `## Features`. The hard "completeness" claim is deferred to Phase 32 conditional on QUAL-01 coverage gate ≥ 95% — same gating discipline as the v2.2 HP-41CV claim.
- **Free42 contamination guard policy documented (Pitfall 19):** ADR-002 carries the verbatim disclaim sentence "Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied." The CI enforcement script `scripts/check-free42-contamination.sh` is deferred to Phase 32 / QUAL-05.

#### Phase 31 — GUI Integration (shipped 2026-05-18)

GUI surface wires Math Pac I into the Tauri app: `xrom_resolve` reachable from `key_map::resolve`, modal-prompt rendering in React via `CalcStateView.modal_prompt`, `request_cancel` Tauri command + cancel button (D-28.7 wiring), `HelpOverlay.tsx` gets the "Math 1 Pac (XROM 7)" two-section collapsible layout. R/S 3-way routing (`submit_modal` when `modal_program_active`) lands in `App.tsx::invokeForKey`. SC-4 invariant preserved — no `op_*` / `flush_*` / `format_hpnum` in `src-tauri/`.

#### Phase 32 — Test Hardening & Quality Gates (shipped 2026-05-18)

- **Test/CI infrastructure delivered (QUAL-03, QUAL-04, QUAL-05, QUAL-07, QUAL-08 closed):** Wave 1 graduated the two vacuous Plan-28-01 meta-gates (`math1_op_test_count.rs` actively cross-checks all 45 Math Pac I `Op` variants against the 14 `tests/math1_*.rs` files per Pitfall 16; `xrom_shadowing.rs` actively cross-checks all 52 `MATH_1.ops` mnemonics against the 18-entry `BUILTIN_CARD_OP_NAMES` allowlist per Pitfall 1). New `tests/lint_math1_assertions.rs` enforces Pitfall 14 + Pitfall 17 assertion discipline (no `assert_eq!(decimal, decimal)` on iterated results; no manual `(a-b).abs() < EPSILON` patterns that bypass the `max_relative = 1e-7` discipline) with an adjacent contiguous-comment-block `// LINT-EXEMPT: <reason>` annotation heuristic.
- **Numerical accuracy ≥ 98 % gate held (QUAL-02, QUAL-06):** Wave 1 Plan 32-02 extended `numerical_accuracy.rs` from 434 → 571 `case!()` invocations (+137 risk-weighted cases across all 11 D-32.9 Math Pac I families — CMPLX 20, MAT 18, POLY 25, INTG 15, SOLVE 15, DIFEQ 12, HYP 10, TRI 8, FOUR 6, TRANS 3, REAL 2). Each new case carries an OM page citation (`// Source: HP 00041-90034 p.<n>`) or D-28.3 emulator-extension marker, plus a `// Catches: <bug class>` doc comment per D-27.1. Combined gate at 763/768 (99.3 %) well above 98 % floor; D-27.6 baseline floor `baseline_passes ≥ 498` preserved bit-for-bit.
- **E2E smoke extended with Math Pac I workflows (QUAL-03, D-32.2):** `hp41-gui/e2e/smoke.spec.js` now ships 3 `it()` blocks — the existing `2 ENTER 3 + → 5.0000` literal-ROADMAP smoke preserved bit-for-bit per D-32.4, plus two new Math Pac I workflows: `XEQ "SINH" 1 → 1.1752` (xrom_resolve path) and `XEQ "MATRIX" 2x2 DET → -2.0000` (modal pipeline path). Both new tests use a `browser.execute` fallback via `window.__TAURI_INTERNALS__.invoke('dispatch_op', { keyId: 'xeq_<NAME>' })` for the XEQ-by-name invocations because the on-screen `xeq_name` modal cannot type the letter 'N' (App.tsx::handleClick prioritizes `effectiveId === 'enter'` as submit before alphaChar routing). MATRIX uses real-click for digit entry and R/S submits between elements, exercising D-31.1 R/S 3-way routing and column-major iteration (input order 1,3,2,4 for `[[1,2],[3,4]]`). Total added CI time ~40s, runs only on Ubuntu via `ci-gui.yml::e2e-linux`.
- **Free42 GPL-contamination guard shipped (QUAL-05, D-32.7, D-32.8, Pitfall 19):** `scripts/check-free42-contamination.sh` greps `hp41-core/src/ops/math1/` for 12 distinctive Free42 / Intel BID / decNumber / GPL/AGPL identifiers (`phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License`) with an allowlist for the per-file disclaim header. Bash + `set -euo pipefail`. Wired via D-32.8 belt + suspenders into BOTH `just license-audit` / `just ci` AND a dedicated `.github/workflows/ci.yml::license-audit` parallel job. Bare `Free42` string deliberately EXCLUDED because 122 legitimate cross-check references exist.
- **Coverage gate ≥ 95 % MET — README hard-claim graduated (post-Phase-32 gap-closure run, 2026-05-18):** Plans 32-04..32-09 added ~70 risk-weighted error-branch tests across 9 new `hp41-core/tests/` files (`math1_poly_error_branches.rs`, `math1_trans_error_branches.rs`, `math1_four_error_branches.rs`, `math1_solve_error_branches.rs`, `math1_difeq_error_branches.rs`, `math1_matrix_error_branches.rs`, `math1_mod_extra_coverage.rs`, `math1_integ_error_branches.rs`, `program_error_branches.rs`) plus the CR-01 + WR-01..07 cleanups from `32-REVIEW.md`. Final measurement: **95.39 % lines / 94.26 % regions** on `hp41-core` (programmatically gated via `cargo llvm-cov --package hp41-core --fail-under-lines 95 --fail-under-regions 93`), with all `ops/math1/*.rs` files ≥ 90 % per ROADMAP SC-1. The README v3.0 line graduated to the OM-cited hard claim "v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034" per D-32.5.
- **Frozen invariants preserved across Phase 32:** no `hp41-core/src/` source changes (frozen since Plan 25-01); no `hp41-gui/src-tauri/src/` source changes (SC-4 invariant); MSRV 1.88 unchanged (`approx 0.5.1` was already a `[dev-dependencies]` entry since Phase 28); `#![deny(clippy::unwrap_used)]` continues to apply; save-file backward compat preserved.

#### Post-graduation polish before v3.0 ship (2026-05-20, quick-task batch on `develop`)

Six post-32-10 commits address visual/UX issues surfaced during the v3.0 release readiness review — none are formally GSD-planned phases (they ship as a quick-task batch analogous to v2.1's reconcile-by-quick-tasks pattern):

1. **`2b547b9`** fix MSRV-CI `uninlined_format_args` on 5 sites (`hp41-core/src/ops/program.rs:350` + 4 assert! messages — pre-existing failure from Plan 31-04, surgical `hp41-core` exception under the lint-only / no-behavior-change rule).
2. **`ca87222`** cargo fmt cleanup on 8 Wave 1 test files (subagent-worktree drift).
3. **`555eb8f`** E2E `XEQ "SINH" 1` + `XEQ "MATRIX" 2x2 DET` smoke tests assert on the `dispatch_op` response's `CalcStateView.display_str` instead of `[data-testid="lcd-display"]` data-text — the prior assertion path was incompatible with `__TAURI_INTERNALS__.invoke` because the frontend deliberately does not poll for state updates (D-11 — no polling) and an out-of-band invoke does not propagate to the React tree.
4. **`fe33be3`** right-panel `key_ref_entries()` gains an `entry.xrom.is_none()` filter — Phase 29 SC-4 / D-29.2 / CLI-04 acceptance criteria are formally SUPERSEDED because the ~45 `XEQ "..."` rows crowded out the actual keyboard reference; module functions remain fully discoverable via the `?` overlay's "Math 1 Pac (XROM 7)" section.
5. **`d3c68c5`** standard ratatui modal pattern — `frame.render_widget(Clear, overlay_area)` before `render_stateful_widget` on both `?` help and Ctrl+P programs overlays — fixes right-panel/stack/display bleed-through visible as e.g. `XEQ "X<Y?"X<Y?` appearing inside the help table's Math section.
6. **`201b3f7`** incremental substring search in the `?` overlay (`App.help_search_query`, `help_data::filter_help_rows`, 3-line search bar above the table, ↑↓/PgUp/PgDn navigation, `q`/`j`/`k` legacy nav suppressed once query is non-empty) — mirrors hp41-gui's `HelpOverlay.tsx` search input, prepares the overlay for v3.1 Stat / v3.2 Time / v3.3 Advantage Pacs which will push the row count past 200. 12 new tests landed (6 in `help_data::tests`, 6 in `app::tests`).

The v3.0 release ships at the cumulative HEAD of these six commits.

**Frozen invariants preserved across v3.0:**
- SC-4 invariant: every Phase 28–32 change respects the stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` (returns nothing). Math Pac I math lives in `hp41-core/src/ops/math1/`.
- 4-exhaustive-match invariant: every new `Op` variant landed in `dispatch()` + `execute_op()` + both `prgm_display.rs` copies before any caller could compile.
- `#![deny(clippy::unwrap_used)]` continues to apply in `hp41-core`; new test files in v3.0 carry `#[allow]` at file scope per the established pattern.
- Save-file backward compat: every new `CalcState` field added in Phase 28 carries `#[serde(default)]`; transient fields (`integ_state`, `solve_state`, `modal_program`, `modal_prompt`, `cancel_requested`) additionally carry `#[serde(skip)]`. v1.0–v2.2 save files continue to load without migration.
- MSRV 1.88 unchanged through Phase 28–32. `approx 0.5.1` is the only post-v2.2 dev-dep.

---

## Quality Gate History

| Gate | Target | v1.0 | v1.1 / v2.0 | v2.2 (Phase 27) | v3.0 (Phase 32) |
|------|--------|------|-------------|------------------|------------------|
| Cold-start | ≤ 0.5 s | 2.2 ms (M1) | unchanged (CLI); GUI not gated | unchanged | unchanged |
| Key latency | ≤ 50 ms median | ~65 ns/op | unchanged | unchanged | unchanged |
| Numerical accuracy | ≥ 98% | 99% (495/500) | unchanged | 99.1% (561/566) | 99.3% (763/768) |
| `hp41-core` coverage | ≥ 95% lines / ≥ 93% regions (v2.2 raised from 80%) | 94.87% | 92.5% / 89.9% | 95.25% / 93.75% | 95.39% / 94.26% |
| Panics in `hp41-core` | 0 | 0 | 0 | 0 | 0 |
| Free42 contamination | 0 distinctive symbols | n/a | n/a | n/a | 0 (CI-gated) |
| CI | Win 10+, macOS 12+, Ubuntu 22.04+ | `ci.yml` | + `ci-gui.yml` | unchanged | + `license-audit` job |
| MSRV | declared | — | 1.88 | 1.88 | 1.88 |
