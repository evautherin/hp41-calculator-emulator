# Phase 53: Build-Approach Decision + iOS Scaffold Spike - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-29
**Phase:** 53-Build-Approach Decision + iOS Scaffold Spike
**Areas discussed:** Environment & account readiness, Spike fallback trigger, Automatable vs manual split, `just` iOS tooling scope

---

## Environment & account readiness

### Apple Developer Program account
| Option | Description | Selected |
|--------|-------------|----------|
| Enrolled & active | Paid membership active; ASC + device signing in-scope | ✓ |
| Free Apple ID only | Simulator + free-provisioning only; ASC/TestFlight blocked | |
| Nothing yet | Simulator-only this phase | |
| Enrolment in progress | Plan around Simulator now, device/ASC as checkpoint | |

**User's choice:** Enrolled & active
**Notes:** Unblocks SC-3 (device install), SC-4 (ASC bundle-ID registration), and all signing for this phase.

### Physical iPhone availability
| Option | Description | Selected |
|--------|-------------|----------|
| Yes, iOS 17+ | Device test in-scope; iOS-17 background-throttling path available | ✓ |
| Yes, iOS 14–16 | Device test works; background timers pause (Phase 56) | |
| Yes, unsure of version | Detect via Xcode | |
| Not right now | Device install becomes a checkpoint | |

**User's choice:** Yes, iOS 17+
**Notes:** SC-3 on-device smoke fully in-scope; also sets up the Phase 56 `backgroundThrottlingPolicy` (iOS 17+) path.

### Local Mac iOS toolchain
| Option | Description | Selected |
|--------|-------------|----------|
| All set up | Xcode + CocoaPods + Rust iOS targets ready | |
| Xcode yes, rest unsure | Add preflight check + install step | ✓ |
| Need a preflight check | Begin with full preflight | |
| Fresh machine | Full toolchain bootstrap first | |

**User's choice:** Xcode yes, rest unsure
**Notes:** Plan opens with a preflight — `brew install cocoapods`, `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` — installing whatever is missing.

---

## Spike fallback trigger

### Effort budget before falling back to Approach B
| Option | Description | Selected |
|--------|-------------|----------|
| Time-boxed workarounds | Bounded fixes ~½ day, then switch to B | ✓ |
| Exhaust workarounds | Pursue every plausible fix before conceding | |
| Fail fast to B | First Xcode-stage path error is decisive | |
| Diagnose, then decide together | Bring evidence back to user | |

**User's choice:** Time-boxed workarounds (~½ day)
**Notes:** Decision criterion = failure stage; Rust compiles but Xcode assembly fails on nested-workspace path = Approach B.

### Workspace-flattening vs the Frozen Invariant
| Option | Description | Selected |
|--------|-------------|----------|
| Diagnostic only, never committed | Throwaway local experiment; if flattening is the only fix, A is rejected → B | ✓ (Claude's call) |
| Invariant is absolute — don't even experiment | No flattening even locally | |
| Invariant is negotiable here | Reconsider the invariant | |

**User's choice:** "you decide" → delegated to Claude
**Notes:** Resolved as **diagnostic-only, never committed**, per CLAUDE.md "Frozen Invariants are final." Committed tree always keeps root members `["hp41-core","hp41-cli"]` and tauri confined to `hp41-gui`.

---

## Automatable vs manual split

### Handling human-only steps
| Option | Description | Selected |
|--------|-------------|----------|
| Inline checkpoints | One phase; executor pauses with click-by-click manual instructions, then verifies | ✓ |
| Automatable first, manual batched at end | Single consolidated manual checklist at the end | |
| Split: spike now, device/ASC follow-up | Ship Simulator spike now, defer manual portion | |

**User's choice:** Inline checkpoints
**Notes:** Manual steps = ASC registration, device install/trust, Xcode signing-team selection.

### Smoke test depth
| Option | Description | Selected |
|--------|-------------|----------|
| Small round-trip | Enter number, SIN, check result | |
| Literally one key | Tap one key, confirm display changes | |
| Mini RPN sequence | `2 ENTER 3 +` then `SIN`, verify stack + display | ✓ |

**User's choice:** Mini RPN sequence
**Notes:** Confirms the full touch → key_map.resolve → invoke(dispatch_op) → CalcStateView → re-render loop on Simulator (SC-2) and device (SC-3).

---

## `just` iOS tooling scope

| Option | Description | Selected |
|--------|-------------|----------|
| Formalize core recipes now | `just ios-init` / `ios-build` / `ios-sim` with explicit target triples (P-iOS-07) | ✓ |
| Ad-hoc now, formalize later | Raw `cargo tauri ios` during spike, recipes once stable | |
| Minimal: one ios-build recipe | Single recipe, defer the rest | |

**User's choice:** Formalize core recipes now
**Notes:** Becomes the documented entry points later phases + `ci-ios.yml` build on; honors the sole-task-runner invariant from day one.

---

## Claude's Discretion

- Workspace-flattening boundary (D-53.6) — delegated by user ("you decide"); resolved diagnostic-only / never-committed.
- Exact set of bounded workarounds within the ½-day time-box.
- Simulator device model (P-iOS-01 — pick a current, non-hardcoded device).
- ADR structure (cite research artifacts rather than re-deriving the A-vs-B comparison).

## Deferred Ideas

None — discussion stayed within phase scope. (Re-planning Phases 54–57 if Approach B is chosen is already tracked in STATE.md and the ROADMAP Phase 53 note as a downstream consequence.)
