# Phase 53: Build-Approach Decision + iOS Scaffold Spike - Pattern Map

**Mapped:** 2026-05-31
**Files analyzed:** 6 (3 committed edits, 1 new ADR, 1 generated tree, 1 reference-only)
**Analogs found:** 3 / 3 (every committed/created file has a strong in-repo analog; the generated `gen/apple/` tree has none by nature)

> **Phase character:** This is a build-pipeline + decision spike, NOT a feature
> phase. There is **no new Rust logic, no new `Op` variant, no React/TS change**.
> The 4-way exhaustive-match invariant, SC-4, and the serde-default invariant are
> all trivially preserved (no new code paths). The patterns to copy are therefore
> **toolchain/recipe/doc patterns**, not code-logic patterns.

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/Cargo.toml` | config (manifest) | n/a (build config) | itself — the existing `[lib]` block (lines 14-16) | exact (in-file edit) |
| `justfile` (add `ios-init`, `ios-build`, `ios-sim`, opt. `ios-dev`) | config (task runner) | batch / shell-invoke | `gui-*` recipe group, `justfile` lines 122-189 | exact (same file, same idiom) |
| `docs/adr/v4.1-002-build-approach.md` | doc (ADR) | n/a (decision record) | `docs/adr/v4.1-001-macos-menu-bar-mode.md` + `docs/adr/v4.0-001-xmem-os-builtin.md` | exact (ADR template) |
| `hp41-gui/src-tauri/.gitignore` | config (vcs) | n/a | itself — current line 5 `gen/` | exact (in-file edit; **decision needed**) |
| `hp41-gui/src-tauri/gen/apple/**` | generated artifact (Xcode project, Podfile, project.yml) | n/a | **none** — created by `cargo tauri ios init` | no analog (generated) |
| `hp41-gui/src-tauri/tauri.conf.json` | config | n/a | reference-only (no change — bundle ID already correct) | n/a (read-only) |

> **Naming correction (load-bearing):** CONTEXT.md / ROADMAP refer to the ADR as
> `v4.1-001-build-approach.md`, but `docs/adr/v4.1-001-macos-menu-bar-mode.md`
> **already exists** (shipped with PR #21). The build-approach ADR **must** be
> filed as **`docs/adr/v4.1-002-build-approach.md`**. Confirmed via `ls docs/adr/`:
> the highest v4.1 slot taken is `v4.1-001`.

---

## Pattern Assignments

### `hp41-gui/src-tauri/Cargo.toml` (config, in-file edit — D-53.2)

**Analog:** the file's own existing `[lib]` block.

**Current state** (lines 14-16) — **no `crate-type` line**:
```toml
[lib]
name = "hp41_gui_lib"
path = "src/lib.rs"
```

**Target state** (the single Rust structural change for Approach A — add ONE line,
do not touch `name`/`path`):
```toml
[lib]
name = "hp41_gui_lib"
path = "src/lib.rs"
crate-type = ["staticlib", "cdylib", "rlib"]
```

**Why `rlib` stays in the list:** `staticlib` is what the generated Xcode project
links on iOS; `cdylib` is needed for the desktop dynamic-link path; **`rlib` must
remain** so the existing `[[bin]]` (`hp41-gui`, `src/main.rs`, line 10-11) and
`[dev-dependencies]` tests can still link against the lib crate on desktop. Dropping
`rlib` would break `just gui-ci` (`cargo test --manifest-path …`).

**Frozen-invariant guard (verify after edit):** `tauri` / `tauri-build` stay
confined to THIS file (lines 19, 33). Root `Cargo.toml` members are NOT touched —
`tauri ios init` operates entirely inside `hp41-gui/src-tauri/`. The `[workspace]`
header at line 1 (`resolver = "2"`) is what keeps this a nested standalone workspace;
leave it exactly as-is.

**Pre-existing readiness (do NOT re-add):** `lib.rs:37` already has
`#[cfg_attr(mobile, tauri::mobile_entry_point)]` on `pub fn run()` (line 38). The
mobile entry point is wired — the `crate-type` line is the ONLY source change.

---

### `justfile` — new `ios-*` recipe group (config, D-53.9)

**Analog:** the `gui-*` group, `justfile` lines 122-189. Copy its exact idioms.

**Idioms to copy from the `gui-*` recipes:**

1. **Section banner + `[group(...)]` attribute** (lines 122-126):
```just
# ─── GUI (Tauri v2) ─────────────────────────────────────────────────────────

# GUI: install npm dependencies (run once after cloning or after package.json changes)
[group('gui')]
gui-install:
	cd hp41-gui && npm install
```
Mirror this exactly: add a `# ─── iOS (Tauri v2 Mobile) ───…` banner and tag every
new recipe `[group('ios')]`.

2. **`cd hp41-gui && …` working-directory idiom** (lines 132, 144, 150) — every GUI
recipe `cd`s into `hp41-gui` (the nested-workspace root) before invoking the Tauri
CLI. This directly satisfies **P-iOS-03** prevention ("do not invoke Tauri CLI from
the repository root"). The iOS recipes MUST follow the same `cd hp41-gui && …` shape.

3. **`npm run tauri …` vs `cargo tauri …`:** the GUI recipes use `npm run tauri dev`
/ `npm run tauri build` (lines 132, 145) so the Tauri CLI from `@tauri-apps/cli` (a
devDependency) is on PATH. RESEARCH.md/ARCHITECTURE.md examples use the
`cargo tauri ios …` form. **Planner decision:** prefer `npm run tauri ios …` for
consistency with `gui-dev`/`gui-build` (CLI already pinned in package-lock.json), OR
`cargo tauri ios …` if `tauri-cli` is installed globally. Either way, bake **explicit
target triples** in per D-53.9 to mitigate P-iOS-07 (`aarch64-apple-ios` device,
`aarch64-apple-ios-sim` Apple-Silicon sim).

4. **Tab indentation:** `justfile` recipe bodies use a literal TAB (not spaces) —
match the existing recipes byte-for-byte or `just` will error.

5. **Self-installing / precondition-guard pattern** (lines 134-145, 178-189): `gui-build`
runs `npm ci` first; `gui-e2e` has a hard `test -x … || (echo "ERROR…" && exit 1)`
precondition. Apply the same discipline to `ios-*`: a preflight that installs missing
pieces (`brew install cocoapods`, `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
per D-53.10) and/or a guard that surfaces a missing-toolchain error at recipe entry
rather than mid-build.

**Reference recipe shapes from ARCHITECTURE.md (lines 95-110) — adapt the working
directory to the `gui-*` `cd hp41-gui &&` idiom, NOT `cd hp41-gui/src-tauri`):**
```just
[group('ios')]
ios-init:
	cd hp41-gui && npm run tauri ios init

[group('ios')]
ios-dev device="":
	cd hp41-gui && npm run tauri ios dev {{device}}

[group('ios')]
ios-build:
	cd hp41-gui && npm run tauri ios build
```

**P-iOS-01 mitigation to fold into `ios-sim`:** add an
`xcrun simctl list devices available` verification step (do NOT hardcode a device
name like "iPhone 13"). **P-iOS-09 mitigation:** the Xcode build phase needs
`~/.cargo/bin` on PATH — document via a comment in the recipe and/or
`gen/apple/.xcode.env.local`.

**Never-bare-cargo invariant:** these recipes ARE the documented entry points; the
"`just` is the sole task runner" rule (CLAUDE.md Tech Stack) means Phases 54-57 and
`ci-ios.yml` invoke `just ios-*`, never raw `cargo tauri ios`.

---

### `docs/adr/v4.1-002-build-approach.md` (doc, NEW ADR)

**Analogs:** `docs/adr/v4.1-001-macos-menu-bar-mode.md` (concise recent ADR — best
structural match for a build/architecture decision) and
`docs/adr/v4.0-001-xmem-os-builtin.md` (the rigorous template with Footnotes/References).

**Header block to copy** (from `v4.1-001-macos-menu-bar-mode.md` lines 1-6):
```markdown
# ADR v4.1-002 — iOS Build Approach (Tauri v2 Mobile vs SwiftUI + UniFFI)

**Status:** Accepted
**Date:** 2026-05-3X
**Context:** v4.1 iOS Foundation milestone — Phase 53 scaffold spike
```

**Section skeleton** (union of the two analogs — both use `## Context` →
`## Decision` → `## Consequences` → `## Alternatives considered`):
```markdown
## Context
## Decision
## Consequences   (v4.0 ADR splits Positive / Negative / Neutral — optional here)
## Alternatives considered
## Footnotes / References   (v4.0 ADR style — cite research artifacts here)
```

**Pattern: cite, don't re-derive (D-53.6 / specifics line 99).** The v4.0 ADR's
Footnotes section (lines 162-181) references planning artifacts by path
(`.planning/phases/51-x-mem-core/51-CONTEXT.md`, `hp41-core/src/ops/program.rs`).
Mirror this: the build-approach ADR cites `.planning/research/SUMMARY.md`,
`STACK.md §5` (decision matrix), `ARCHITECTURE.md` (Approach A vs B detail),
`PITFALLS.md P-iOS-03` for the A-vs-B comparison rather than re-deriving it.

**Pattern: the Decision records the OBSERVED spike outcome** (D-53.5). The ADR's
Decision section states which approach the `cargo tauri ios build` spike confirmed,
keyed on the **failure stage**: clean build → Approach A confirmed; Rust compiles but
Xcode assembly fails on the nested-workspace path (#5865) after the ½-day time-box →
Approach B. Write the Decision section AFTER the spike runs — not before.

**Pattern: explicit Frozen-Invariant accounting.** Both analogs close by confirming
no `hp41-core` / IPC / `Op` changes (v4.1-001 line 46; v4.0-001 "Backward compat
unaffected"). The build-approach ADR must record that **both** approaches preserve
the root-members invariant and `tauri`-confinement (per ARCHITECTURE.md "Frozen
Invariant Check" table, lines 571-585), and that D-53.6 workspace-flattening was
diagnostic-only / never committed.

---

### `hp41-gui/src-tauri/.gitignore` (config, in-file edit — DECISION REQUIRED)

**Analog:** the file itself (5 lines).

**Current state — `gen/` is fully ignored:**
```gitignore
# Rust build output
target/

# Tauri build-time generated files (schema, capabilities)
gen/
```

**The tension (load-bearing for the planner):** `tauri ios init` writes the new
Xcode project to `hp41-gui/src-tauri/gen/apple/`, but the current rule ignores **all**
of `gen/`. Tauri maintainer guidance (ARCHITECTURE.md lines 74-75, SUMMARY source
discussion #8323) is to **commit `gen/apple/project.yml`** (the XcodeGen source of
truth) while letting `gen/apple/Pods/`, `Externals/`, and `build/` stay ignored
(Tauri writes a nested `.gitignore` inside `gen/apple/` for those).

**Planner must decide and record in the ADR** (CONTEXT.md "Integration Points":
"Decide gitignore vs commit during planning"). Two viable patterns:
- **(a) Narrow the existing rule** so `gen/schemas/` stays ignored but `gen/apple/`
  is tracked (e.g., `gen/schemas/` + rely on Tauri's nested `gen/apple/.gitignore`).
- **(b) Keep `gen/` ignored entirely** and treat `gen/apple/` as fully regeneratable
  via `just ios-init` (idempotent per ARCHITECTURE.md line 75). Simpler, but loses
  the committed `project.yml` source-of-truth.

This is the only `.gitignore` touch; do not introduce a root-level `.gitignore`
change (root `.gitignore` line 35 already ignores `src-tauri/target/`).

---

### `hp41-gui/src-tauri/gen/apple/**` (generated — NO ANALOG)

Created wholesale by `cargo tauri ios init` (runs `cargo-mobile2`). Contains the
generated `.xcodeproj`, `Podfile`, `project.yml`, and Swift glue. **No in-repo
analog exists** (no iOS scaffold has ever been generated — `gen/` currently holds
only `schemas/`). Do not hand-author these files; they come from the tool. The
planner should treat the contents as opaque generated output and reference the
ARCHITECTURE.md `gen/apple/` layout (lines 64-75) for what to expect.

---

### `hp41-gui/src-tauri/tauri.conf.json` (reference-only — NO CHANGE)

Already has `productName "HP-41 Calculator"` and `identifier "ch.talent-factory.hp41"`.
Per D-53.3 this phase **registers** that bundle ID in App Store Connect (a human
checkpoint, P-iOS-14) — it does **not** edit the file. Listed here only so the
planner does not schedule an edit. The hyphen-free identifier sidesteps the known
Tauri bundle-ID bug (STACK.md line 88).

---

## Shared Patterns

### Nested-workspace discipline (applies to Cargo.toml + justfile + ADR)
**Source:** `hp41-gui/src-tauri/Cargo.toml:1` (`[workspace]` header), `justfile:28-35`
(the `clean` recipe documents the dual-workspace `target/` reality).
**Apply to:** every recipe and the ADR.
- Tauri iOS work happens **inside `hp41-gui/`** (`cd hp41-gui && …`), never from repo root (P-iOS-03).
- Root members stay `["hp41-core", "hp41-cli"]`; the `[workspace]` header in
  `src-tauri/Cargo.toml` is what prevents Cargo from rolling the GUI into the root
  workspace — never remove it.
```just
# justfile:33-35 — the canonical "two separate target/ trees" acknowledgement
clean:
	cargo clean
	cargo clean --manifest-path hp41-gui/src-tauri/Cargo.toml
```

### `just`-is-sole-task-runner (applies to all recipes + ADR)
**Source:** CLAUDE.md "Tech Stack" + every `[group(...)]`-tagged recipe in `justfile`.
**Apply to:** `ios-init` / `ios-build` / `ios-sim` / `ios-dev`.
New iOS commands are exposed only through `just ios-*`; downstream CI (`ci-ios.yml`,
Phase 57) and Phases 54-56 invoke the recipes, never bare `cargo tauri ios`.

### ADR-cites-research, records-observed-outcome (applies to the ADR)
**Source:** `docs/adr/v4.0-001-xmem-os-builtin.md` Footnotes (lines 162-181) +
`docs/adr/v4.1-001-macos-menu-bar-mode.md` (concise Decision/Consequences shape).
**Apply to:** `v4.1-002-build-approach.md` — reference `.planning/research/*` for the
A-vs-B comparison; the Decision section captures the spike's failure-stage outcome
(D-53.5), written after the spike, not before.

### Explicit-target-triple discipline (applies to justfile)
**Source:** D-53.9 + P-iOS-07 (STACK.md target table lines 26-30).
**Apply to:** `ios-build` / `ios-sim`.
`aarch64-apple-ios` (device) and `aarch64-apple-ios-sim` (Apple-Silicon simulator)
are distinct; using the device triple for the simulator yields the silent
"building for iOS Simulator, but linking in object file built for iOS" linker error.
Bake the correct triple into each recipe and add an `xcrun simctl list devices
available` check to `ios-sim` (P-iOS-01).

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `hp41-gui/src-tauri/gen/apple/**` | generated Xcode project / Podfile / project.yml | n/a | Produced by `cargo tauri ios init`; no prior iOS scaffold exists in the repo. Opaque generated output — do not hand-author. Expected layout in ARCHITECTURE.md lines 64-75. |

> All **committed/edited** files (Cargo.toml, justfile, the ADR, .gitignore) have
> exact in-repo analogs. The only "no analog" item is tool-generated by design.

---

## Metadata

**Analog search scope:** `hp41-gui/src-tauri/` (Cargo.toml, .gitignore, lib.rs,
gen/), `justfile` (root), `docs/adr/` (full ADR set).
**Files scanned:** 8 (5 source/config + 3 ADRs read for template extraction).
**Key verifications performed:**
- `ls docs/adr/` → confirmed `v4.1-001` taken → ADR must be `v4.1-002`.
- `grep mobile_entry_point hp41-gui/src-tauri/src/lib.rs` → confirmed `lib.rs:37`
  already wired (no `lib.rs` edit needed).
- `ls hp41-gui/src-tauri/gen/` → only `schemas/`; no `apple/` yet.
- `hp41-gui/src-tauri/.gitignore:5` → `gen/` currently ignored wholesale
  (gitignore decision flagged for the planner).
**Pattern extraction date:** 2026-05-31
</content>
</invoke>
