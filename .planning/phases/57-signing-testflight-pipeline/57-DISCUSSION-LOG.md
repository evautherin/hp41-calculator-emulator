# Phase 57: Signing + TestFlight Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-04
**Phase:** 57-signing-testflight-pipeline
**Areas discussed:** CI signing & secrets, CI trigger model, Build-number scheme, App icon & launch screen

---

## CI signing & secrets

| Option | Description | Selected |
|--------|-------------|----------|
| ASC API key + automatic | Keep automatic signing; ASC API key (.p8/issuer/key-id) for `-allowProvisioningUpdates` and reused for `altool`. Dodges P-iOS-16. | ✓ |
| fastlane match | Distribution cert + profile in an encrypted git repo; switch to manual signing; adds MATCH_PASSWORD. | |
| Raw P12 + profile secrets | Base64 cert/profile imported into a temp keychain each run; classic P-iOS-16 surface. | |

**User's choice:** ASC API key + automatic
**Notes:** Minimal keychain handling, no cert repo; same key used for the TestFlight upload via `xcrun altool`.

---

## CI trigger model

| Option | Description | Selected |
|--------|-------------|----------|
| Manual dispatch only | `workflow_dispatch` only; uploads on demand. No accidental builds, secrets used only when triggered. | ✓ |
| On v* tag push | Mirror release.yml; every tag burns a build number. | |
| Build on PR, upload on dispatch | Two-tier: no-signing build on PRs + gated signed upload. | |

**User's choice:** Manual dispatch only
**Notes:** Tag-trigger can be added later once the pipeline is proven.

---

## Build-number scheme

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub run number | `CFBundleVersion = github.run_number`, set at build time; marketing version manual. Zero bookkeeping. | ✓ |
| Git commit count | `git rev-list --count HEAD`; needs full fetch in CI. | |
| Manual bump committed | Bump+commit build number per upload; easy to forget. | |

**User's choice:** GitHub run number
**Notes:** Always increments, never resets; marketing version (`CFBundleShortVersionString`) stays manual at the milestone (e.g. 4.1).

---

## App icon & launch screen

| Option | Description | Selected |
|--------|-------------|----------|
| Real HP-41 icon now | Design a proper HP-41-styled icon (1024× master → regen appiconset) + branded launch screen this phase. | ✓ |
| Keep current placeholder | Ship the existing auto-generated icon; defer branding. | |
| Verify current, polish if weak | Inspect the current asset, decide after seeing it. | |

**User's choice:** Real HP-41 icon now
**Notes:** Placeholder not acceptable for the TestFlight foundation build; first impression for testers matters and avoids rework.

---

## Claude's Discretion

- Exact GitHub secret variable names and the precise `xcodebuild`/`altool` invocation.
- Where the CI build hooks into the existing `just` recipe layer.
- Whether `ExportOptions.plist` is edited in place or generated in CI.

## Deferred Ideas

- `v*`-tag-triggered TestFlight uploads — later milestone.
- External/public TestFlight + App Store submission — v4.2+ (already deferred).
- Desktop macOS notarization — separate concern.
- `.raw` file picker on iOS — v4.2+.
