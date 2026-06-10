---
planted_during: v4.3 milestone kickoff (2026-06-06)
trigger_when: A cross-platform reach / distribution milestone is considered, OR an Android test device or willing tester becomes available
status: planted
---

# SEED-001: Android app

## Idea

Ship the existing GUI to Android via Tauri v2's Android mobile target — the sibling
of the iOS target already established in v4.1 (iOS Foundation). Same React + Rust
frontend, different mobile target.

## When to Surface

- A future milestone is scoped around platform reach or distribution.
- An Android device — or a willing tester — becomes available (none today).
- The externally-handled iOS App Store path is complete and Android becomes the
  next obvious surface.

## Why This Matters

- The Tauri v2 + React frontend is already mobile-proven on iOS (v4.1). Android is
  largely the same codebase against a different Tauri target, so the marginal cost
  may be lower than a from-scratch platform.
- Broadens reach to the larger global mobile platform.

## Open Questions / Unknowns (flagged by Daniel, 2026-06-06)

- **No real-device verification possible** — Daniel has no Android hardware. Acceptance
  would have to lean on emulator-only testing or an external tester. This is a hard
  blocker for "verified working on device" claims.
- **No Android expertise** — higher unknown effort; signing/Play-Store mechanics differ
  from the iOS pipeline already learned.
- **ROI unclear** — is the HP-41-emulator audience on Android large enough to justify
  the effort? Worth a small market/effort sanity check *before* committing the milestone.
