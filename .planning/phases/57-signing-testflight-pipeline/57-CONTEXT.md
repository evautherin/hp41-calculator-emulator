# Phase 57: Signing + TestFlight Pipeline - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Produce a **signed, distributable iOS build** and the CI machinery that uploads it to TestFlight (internal distribution). This is the final phase of the v4.1 iOS Foundation milestone.

Delivers SHIP-01..05:
- SHIP-01: distribution cert + provisioning profile → signed IPA
- SHIP-02: `PrivacyInfo.xcprivacy` in `gen/apple/` (`NSPrivacyAccessedAPICategoryFileTimestamp` / reason `C617.1`)
- SHIP-03: app icon + launch screen
- SHIP-04: `ci-ios.yml` GitHub Actions workflow (macOS runner) builds the signed IPA and uploads via `xcrun altool`
- SHIP-05: a build distributed to testers via TestFlight (internal)

**Engine is frozen:** `hp41-core` is feature-complete at v4.0. No new calculator functions. The iOS build is another adapter on the unchanged core; the workspace Frozen Invariant (root members `["hp41-core","hp41-cli"]`, `tauri` confined to `hp41-gui/src-tauri/`) is preserved.

</domain>

<decisions>
## Implementation Decisions

### CI signing & App Store Connect authentication
- **D-57.1:** Use an **App Store Connect API key** (the `.p8` private key + issuer ID + key ID) for both code-signing provisioning and the TestFlight upload. Keep `CODE_SIGN_STYLE: Automatic` (already in `project.yml`); drive `xcodebuild` with `-allowProvisioningUpdates` so the runner fetches/creates the distribution profile on demand. Reuse the **same** API key for `xcrun altool` upload (`--apiKey` / `--apiIssuer`).
- **D-57.2:** No fastlane match, no certificate git repo, no raw P12/`.mobileprovision` import into a temp keychain — this avoids the P-iOS-16 keychain-on-CI failure surface.
- **D-57.3:** Secrets: store the three ASC API-key components as GitHub repository secrets (e.g. `ASC_KEY_ID`, `ASC_ISSUER_ID`, `ASC_API_KEY_P8` base64). The `.p8` must be written to disk on the runner (e.g. `~/.appstoreconnect/private_keys/AuthKey_<KEYID>.p8` or `--apiKeyPath`) and cleaned up after. Exact secret names to be finalized in planning; document them so the user can add them in repo Settings (operational todo — the user must create the ASC API key in App Store Connect → Users and Access → Integrations).

### CI trigger model
- **D-57.4:** `ci-ios.yml` runs on **`workflow_dispatch` only** (manual "Run workflow"). No tag-trigger, no per-PR upload. Rationale: each upload burns a TestFlight build number and exposes signing secrets; on-demand keeps both intentional. A `v*`-tag trigger can be added in a later milestone once the pipeline is proven.

### Build-number scheme
- **D-57.5:** `CFBundleVersion` (build number) = **`github.run_number`**, injected at build time (PlistBuddy/agvtool on the generated `Info.plist`), never committed back. `github.run_number` increments monotonically and never resets → satisfies TestFlight's strictly-increasing requirement per marketing version.
- **D-57.6:** `CFBundleShortVersionString` (marketing version) stays **manual**, tracking the milestone (e.g. `4.1`). Not auto-derived.

### App icon & launch screen
- **D-57.7:** Design a **real HP-41-styled app icon** this phase (1024×1024 master → regenerate the full `AppIcon.appiconset`, replacing the current auto-generated placeholder) plus a **branded launch screen** (replace/refine `LaunchScreen.storyboard`). A placeholder icon is NOT acceptable for the TestFlight foundation build.

### Locked by requirements (not re-discussed)
- **D-57.8:** `PrivacyInfo.xcprivacy` created in `gen/apple/` declaring `NSPrivacyAccessedAPICategoryFileTimestamp` with reason code `C617.1` (file timestamp access — from autosave file I/O). Required to avoid ITMS-91053 on first upload (P-iOS-19).
- **D-57.9:** TestFlight distribution is **internal only** (internal testers group); external/public TestFlight and App Store submission are explicitly out of scope (deferred to v4.2+).

### Claude's Discretion
- Exact GitHub secret variable names, the precise `xcodebuild`/`altool` invocation, and where in the existing `just` recipe layer the CI build hooks in — left to research/planning, constrained by "never call cargo directly; `just` is the sole task runner" and the existing `.xcode.env` PATH pattern.
- Whether `ExportOptions.plist` is edited in place (`method: app-store-connect`/`app-store`) or generated in CI.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### iOS build & signing foundation (Phase 53 outcomes)
- `docs/adr/v4.1-002-build-approach.md` — Approach A (Tauri v2 Mobile) confirmed; the build path this pipeline signs.
- `hp41-gui/src-tauri/gen/apple/project.yml` — current signing config (`DEVELOPMENT_TEAM 2P4R8QSWT4`, `CODE_SIGN_STYLE: Automatic`); the file CI signing builds on.
- `hp41-gui/src-tauri/gen/apple/ExportOptions.plist` — currently `method: debugging`; must change for distribution export.
- `hp41-gui/src-tauri/gen/apple/.xcode.env` — committed PATH shim (`$HOME/.cargo/bin`) so Xcode build phases find `cargo`; reuse this pattern for the CI runner (per STATE accumulated context, P-iOS-09 RESOLVED 53-04).
- `hp41-gui/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/` — existing (placeholder) icon set to be regenerated.
- `hp41-gui/src-tauri/gen/apple/LaunchScreen.storyboard` — existing launch screen to brand.
- `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` — where `CFBundleVersion` / `CFBundleShortVersionString` live.

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` §"Signing & TestFlight" — SHIP-01..05 (authoritative acceptance).
- `.planning/ROADMAP.md` §"Phase 57" — phase goal and plan-of-record.

### Existing CI patterns to mirror / reuse
- `.github/workflows/ci-gui.yml` — 3-OS GUI matrix + E2E smoke; pattern for a macOS job and the no-signing build step (if a build-only check is added later).
- `.github/workflows/release-gui-binaries.yml` — existing tauri-action GUI release workflow; closest analog for a signed-artifact pipeline.
- `.github/workflows/release.yml` — milestone release/tag-reachability logic; reference for any future `v*`-tag trigger (NOT used now per D-57.4).

### Project guardrails
- `CLAUDE.md` §"Tech Stack" — `just` is the sole task runner; never call `cargo` directly in CI or docs. GUI recipes: `just gui-build` / `just gui-ci`; iOS: `just ios-build`.
- `CLAUDE.md` §"Git Workflow" — milestone PR merge (develop → main) is ALWAYS a merge commit (relevant when closing v4.1 after this phase).

### iOS pitfalls in scope for this phase (from STATE.md accumulated context)
- P-iOS-16 (keychain access fails on macOS CI runners) — mitigated by D-57.1/D-57.2 (API key, no keychain import).
- P-iOS-18 (works in Simulator, fails on device/TestFlight — 5 root causes checklist).
- P-iOS-19 (`PrivacyInfo.xcprivacy` required; ITMS-91053) — D-57.8.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`gen/apple/` is tracked (not gitignored):** the entire Xcode project, entitlements, `ExportOptions.plist`, icon set, and launch screen are committed — CI checks them out directly rather than regenerating, so edits here are durable.
- **`.xcode.env` PATH shim:** already solves "Xcode build phase can't find cargo" (P-iOS-09); the same env approach applies to the GitHub Actions macOS runner.
- **Automatic signing already configured:** `CODE_SIGN_STYLE: Automatic` + `DEVELOPMENT_TEAM 2P4R8QSWT4` mean the API-key + `-allowProvisioningUpdates` path needs minimal `project.yml` change.
- **App ID `ch.talent-factory.hp41` already registered** in App Store Connect (Phase 53) — provisioning-profile creation is unblocked.
- **Full AppIcon.appiconset already present** — regeneration replaces existing slots rather than creating the structure from scratch.

### Established Patterns
- `just` recipes wrap all build steps (`just ios-build` exists); CI should invoke `just`, not raw `cargo tauri`/`xcodebuild` where a recipe exists or can be added.
- Existing GitHub workflows live in `.github/workflows/` with two-layer CI (`ci.yml`, `ci-gui.yml`); `ci-ios.yml` is the new sibling.

### Integration Points
- New `ci-ios.yml` on a macOS runner: checkout → toolchain (Rust iOS targets + node) → `.xcode.env`-style PATH → `just ios-build` (release) → set `CFBundleVersion` from run number → export signed IPA with ASC API key → `xcrun altool` upload.
- `PrivacyInfo.xcprivacy` is a new file under `gen/apple/` and must be added to the Xcode project so it ships inside the bundle.

</code_context>

<specifics>
## Specific Ideas

- **Icon:** "real HP-41-styled icon" — visual direction is the HP-41C's look (likely the gold-on-dark / brushed faceplate aesthetic of the emulator itself). Exact art direction to be confirmed when the master is produced; founder wants it to look like a real product, not a generated placeholder.
- **Upload tool is `xcrun altool`** per SHIP-04 (with `--apiKey`/`--apiIssuer` from the same ASC key).

</specifics>

<deferred>
## Deferred Ideas

- **`v*`-tag-triggered TestFlight uploads** — reconsidered for a later milestone once the manual pipeline is proven (D-57.4).
- **External / public TestFlight + App Store submission** (STORE-01/02) — already deferred to v4.2+ in REQUIREMENTS.md.
- **Desktop macOS notarization / signed GUI binaries** — separate concern (the "macOS UNSIGNED until 3 cert secrets" item); not part of this iOS phase.
- **`.raw` file picker on iOS** — no native picker in `tauri-plugin-dialog`; deferred to v4.2+.

None of the above are in this phase's scope.

</deferred>

---

*Phase: 57-signing-testflight-pipeline*
*Context gathered: 2026-06-04*
