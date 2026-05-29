# Phase 53: Build-Approach Decision + iOS Scaffold Spike - Context

**Gathered:** 2026-05-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove out the iOS build path and lock the build-approach decision. Concretely: scaffold the Tauri iOS target inside the nested `hp41-gui` workspace, run the decisive `cargo tauri ios build` spike (confirms Approach A or triggers the Approach B fallback), boot the app in the iOS Simulator and on a physical iPhone with the engine dispatching live, register the bundle ID in App Store Connect, and record the outcome in ADR `docs/adr/v4.1-001-build-approach.md`.

**In scope:** `crate-type` line on `[lib]`; `tauri ios init`; toolchain preflight/installs; Simulator + on-device smoke; App Store Connect bundle-ID registration; ADR; `just ios-*` recipes.

**Out of scope (own phases):** persistence path (Phase 54), touch UI / haptics / ALPHA entry / audio (Phase 55), lifecycle + clock (Phase 56), signing pipeline / `PrivacyInfo.xcprivacy` / app icon / `ci-ios.yml` / TestFlight upload (Phase 57). **No new calculator functions** — the engine is feature-complete at v4.0; this is form-factor only.

</domain>

<decisions>
## Implementation Decisions

### Build approach (carried forward from research — locked)
- **D-53.1:** Approach A (Tauri v2 Mobile — reuse the React UI + all 10 Tauri commands + `hp41-core`) is the default, **contingent on the spike**. Fallback is Approach B (native SwiftUI + Rust FFI via UniFFI 0.31.1) only if the spike fails. See `.planning/research/SUMMARY.md`.
- **D-53.2:** The only Rust source change for Approach A is adding `crate-type = ["staticlib", "cdylib", "rlib"]` to `[lib]` in `hp41-gui/src-tauri/Cargo.toml`. `hp41-core/Cargo.toml` is untouched under both approaches.
- **D-53.3:** Deployment target iOS 14.0 (Tauri v2 default). Bundle ID `ch.talent-factory.hp41` is already in `tauri.conf.json` — this phase registers it in App Store Connect, it is not changed.

### Spike fallback trigger
- **D-53.4:** On a `cargo tauri ios build` failure at the **Xcode assembly step** (#5865 — path error *after* successful Rust compilation), apply a **time-boxed (~½ day) set of bounded workarounds** first: newer/patched `tauri-cli`, Podfile/path tweaks, temporary symlink, etc. If still failing after the time-box, declare Approach A dead, write the ADR for Approach B, and proceed.
- **D-53.5:** The decision criterion for the ADR is the failure *stage*: Rust compiles but Xcode assembly fails on the nested-workspace path = Approach B. A clean build = Approach A confirmed.

### Frozen-Invariant boundary during the spike
- **D-53.6:** Workspace-flattening (folding `hp41-gui` into the root workspace) is **diagnostic-only and must never be committed**. It is permitted solely as a throwaway *local* experiment to confirm that nesting is the root cause of a #5865 failure. **If flattening is the only thing that makes Approach A build, that is the signal that Approach A is rejected → fall back to Approach B.** The committed tree always preserves the Frozen Invariant (root members `["hp41-core", "hp41-cli"]`; `tauri`/`tauri-build` confined to `hp41-gui/src-tauri/Cargo.toml`).

### Execution shape
- **D-53.7:** Single coherent phase with **inline checkpoints**. The executor performs the automatable work (preflight, `crate-type`, `tauri ios init`, Simulator boot, ADR draft, `just` recipes), then **pauses at explicit checkpoint tasks with click-by-click instructions** for the human-only steps — App Store Connect bundle-ID registration (App ID + app record), plugging in / trusting the physical iPhone, and selecting the signing team in Xcode — then resumes and verifies.
- **D-53.8:** Smoke test = a **mini RPN sequence** (`2 ENTER 3 +`, then `SIN`), verifying both the stack and the display, run on the Simulator (SC-2) and on the device (SC-3). This proves the full loop: touch → `key_map.resolve()` → `invoke("dispatch_op")` → WKWebView bridge → Rust command → `CalcState` → `CalcStateView` → React re-render.

### `just` iOS tooling
- **D-53.9:** Formalize the core iOS recipes in this phase: `just ios-init`, `just ios-build`, `just ios-sim` (and `ios-dev` if useful), with **explicit target triples baked in** (`aarch64-apple-ios` device, `aarch64-apple-ios-sim` Apple-Silicon sim) to mitigate P-iOS-07. These become the documented entry points that Phases 54–57 and `ci-ios.yml` build on, honoring the "`just` is the sole task runner" invariant from day one.

### Toolchain readiness (environment facts)
- **D-53.10:** Apple Developer Program is **enrolled & active**, a physical iPhone on **iOS 17+** is available, and **Xcode is installed** — so all four success criteria (incl. SC-3 device install and SC-4 ASC registration) are achievable this phase. CocoaPods and the Rust iOS targets are **unverified**, so the plan opens with a preflight (`brew install cocoapods`, `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`) that installs whatever is missing.

### Claude's Discretion
- **D-53.6** (workspace-flattening boundary) was explicitly delegated by the user ("you decide"). Resolved as diagnostic-only / never-committed per CLAUDE.md's "Frozen Invariants are final" rule. Claude retains latitude on the exact set of bounded workarounds attempted within the ½-day time-box, the specific Simulator device model used (P-iOS-01 — pick a current, non-hardcoded device), and how the ADR is structured (it should reference the research artifacts rather than re-deriving the A-vs-B comparison).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone research (drives this entire phase)
- `.planning/research/SUMMARY.md` — build-approach recommendation, spike rationale, 3 hard prerequisites, suggested build order
- `.planning/research/ARCHITECTURE.md` — Approach A integration detail: `crate-type` change, `tauri ios init` → `gen/apple/` flow, data flow identical to desktop
- `.planning/research/STACK.md` — tool/version matrix: `tauri-cli` 2.11.x, CocoaPods, Rust iOS target triples, iOS 14.0 deployment target, signing toolchain (deferred to Phase 57)
- `.planning/research/PITFALLS.md` — full pitfall catalog; Phase 53 ones are P-iOS-01 (hardcoded sim device), P-iOS-02 (device debug LAN/Devices setup), P-iOS-03 (#5865 nested-workspace bundler — the gating spike), P-iOS-07 (`-sim` vs device target triple), P-iOS-09 (Xcode build phase can't find `cargo`/PATH), P-iOS-14 (bundle ID not registered before CI)
- `.planning/research/FEATURES.md` — feature scope (mostly later phases; touch/HIG context)

### Requirements & state
- `.planning/REQUIREMENTS.md` — BUILD-01..04 (this phase's requirements) + full v4.1 requirement set
- `.planning/STATE.md` — "Accumulated Context" pre-resolved decisions + iOS pitfall table
- `.planning/ROADMAP.md` §"Phase 53" — goal, 4 success criteria, the note that the spike outcome may reshape Phases 54–57 (esp. 55 if Approach B)

### ADR to be written this phase
- `docs/adr/v4.1-001-build-approach.md` — records which approach was chosen and why (decision criterion = failure stage per D-53.5). Must be written before Phases 54–57 are planned in detail.

### Code touch points (verified during scout)
- `hp41-gui/src-tauri/Cargo.toml` — `[lib]` (`name = "hp41_gui_lib"`, `path = "src/lib.rs"`) currently has **no** `crate-type` line → add per D-53.2
- `hp41-gui/src-tauri/src/lib.rs` §line 34 — `#[cfg_attr(mobile, tauri::mobile_entry_point)]` already present (mobile entry point wired)
- `hp41-gui/src-tauri/tauri.conf.json` §2-3 — `productName "HP-41 Calculator"`, `identifier "ch.talent-factory.hp41"` (already set; register in ASC)
- `justfile` §116+ — existing `gui-*` recipes establish the pattern for the new `ios-*` recipes (D-53.9); no `ios` recipes exist yet
- `hp41-gui/src-tauri/CLAUDE.md` invariants — workspace structure / Frozen Invariant (D-53.6 guard); bundle ID `ch.talent-factory.hp41`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Entire `hp41-gui` desktop app** — React UI, `key_map::resolve()`, all 10 Tauri IPC commands (`dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel`, `tick_time`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`), `CalcStateView` contract — all carry over unchanged under Approach A.
- **`#[cfg_attr(mobile, tauri::mobile_entry_point)]` at `lib.rs:34`** — the mobile entry point is already in place; no new entry-point code needed.
- **`gui-*` justfile recipes** — template/pattern for the new `ios-*` recipes.

### Established Patterns
- **Nested standalone workspace** — `hp41-gui/src-tauri/Cargo.toml` starts with `[workspace]`; this is exactly the nesting #5865 stresses. `tauri ios init` runs entirely inside `hp41-gui/src-tauri/` and generates `gen/apple/` without touching the root workspace.
- **Frozen Invariant (compile-time enforced)** — root `Cargo.toml` members must stay `["hp41-core", "hp41-cli"]`; `tauri`/`tauri-build` confined to `hp41-gui/src-tauri/Cargo.toml`. Both build approaches preserve it.
- **`just` as sole task runner** — no bare `cargo` in CI/docs; new iOS work follows the same discipline (D-53.9).

### Integration Points
- `gen/apple/` (created by `tauri ios init`) — does not exist yet; new Xcode project + Podfile land here. Decide gitignore vs commit during planning.
- Data flow on iOS is identical to desktop (touch → `key_map.resolve` → `invoke` → Rust command → `CalcStateView` → re-render) — the smoke test (D-53.8) exercises exactly this path.

</code_context>

<specifics>
## Specific Ideas

- The decisive spike command is literally `cargo tauri ios build` run inside `hp41-gui/` on macOS — its failure *stage* (Rust-compile vs Xcode-assembly) is the A-vs-B oracle.
- ADR should cite the research artifacts (`.planning/research/*`) for the A-vs-B comparison rather than re-deriving it; its job is to record the *observed* spike outcome and the chosen path.
- App Store Connect step is "register `ch.talent-factory.hp41` as an explicit App ID (Identifiers) + create the app record" — a ~5-minute web-portal task, surfaced as an inline checkpoint with exact steps.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. (Re-planning of Phases 54–57 if Approach B is chosen is already captured as a pending todo in STATE.md and as a note on the ROADMAP Phase 53 entry; it is a downstream consequence, not a deferred idea.)

</deferred>

---

*Phase: 53-Build-Approach Decision + iOS Scaffold Spike*
*Context gathered: 2026-05-29*
