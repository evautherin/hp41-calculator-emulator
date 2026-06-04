---
phase: 57-signing-testflight-pipeline
plan: "03"
subsystem: ios-ci-pipeline
tags: [ios, ci, github-actions, testflight, signing, secrets]
dependency_graph:
  requires:
    - 57-01 (ios-build-release just recipe + PrivacyInfo + ExportOptions)
  provides:
    - .github/workflows/ci-ios.yml (SHIP-04 — signed IPA build + TestFlight upload)
  affects:
    - CI pipeline surface (new workflow, manual dispatch only)
tech_stack:
  added: []
  patterns:
    - workflow_dispatch-only trigger (D-57.4 — no push/branches/tags trigger)
    - ASC API key via step-scoped env vars (APPLE_API_KEY, APPLE_API_ISSUER, APPLE_API_KEY_PATH)
    - .p8 write to canonical altool search path + if:always() cleanup (T-57-04, T-57-05)
    - PlistBuddy CFBundleVersion injection from github.run_number (D-57.5)
    - xcrun altool --upload-package with quoted IPA path (Pitfall 7)
    - macos-26 runner for Xcode 26.x (not macos-latest)
key_files:
  created:
    - .github/workflows/ci-ios.yml
  modified: []
decisions:
  - "workflow_dispatch-only trigger: D-57.4 supersedes ROADMAP SC#4 — a successful manual run satisfies SHIP-04"
  - "signing vars scoped to build step env: only, never global env: block (mitigates T-57-06)"
  - "PlistBuddy on single line to satisfy grep acceptance criterion (PlistBuddy.*CFBundleVersion.*github.run_number)"
  - "macos-26 runner (not macos-latest) — needs Xcode 26.x; lswiftCompatibility56 confirmed non-issue for Tauri 2.11.1"
  - "altool reads .p8 automatically from ~/.appstoreconnect/private_keys/ — no explicit --p8-file-path needed"
metrics:
  duration_minutes: 15
  completed_date: "2026-06-04"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 57 Plan 03: ci-ios.yml GitHub Actions Workflow Summary

Authored `.github/workflows/ci-ios.yml` — the signed IPA build and TestFlight upload automation (SHIP-04). The workflow runs on `workflow_dispatch` only (D-57.4), builds via `just ios-build-release` on the `macos-26` runner (Xcode 26.x), injects the build number from `github.run_number` via PlistBuddy, handles the ASC API key with strict secret hygiene (step-scoped env vars, `if: always()` cleanup, no log leak), and uploads to TestFlight via `xcrun altool --upload-package`.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Author ci-ios.yml — toolchain, build-number, key write, signed build, upload, cleanup (SHIP-04) | (see self-check) | .github/workflows/ci-ios.yml |

## Key Notes (Required by Plan Output Spec)

### 1. D-57.4 Supersedes ROADMAP Success Criterion #4

ROADMAP.md Phase 57 SC #4 originally stated: "ci-ios.yml runs on a macOS GitHub Actions runner … on every push to `main`."

**D-57.4 (CONTEXT.md, the more recent and authoritative source) overrides this:** the workflow triggers on `workflow_dispatch` ONLY. This is intentional — each dispatch burns a TestFlight build number and exposes ASC signing secrets; manual gating is correct for a first proven pipeline.

**A successful manual `workflow_dispatch` run satisfies SHIP-04.** The verify-phase runner must NOT flag the absence of a `push:` trigger as a gap. The top-of-file comment in `ci-ios.yml` documents this reconciliation explicitly.

### 2. Three Required GitHub Repo Secrets — User Must Create Before Running

The workflow cannot run until these three secrets are added in the repository:
**Settings → Secrets and variables → Actions → New repository secret**

| Secret Name | Content | How to Get |
|-------------|---------|------------|
| `ASC_KEY_ID` | Key ID string (e.g. `ABC123DEF456`) | Shown when creating the key in ASC |
| `ASC_ISSUER_ID` | Issuer ID UUID (e.g. `57246542-96fe-1a63-e053-0824d011072a`) | Shown at the top of the ASC Integrations page |
| `ASC_API_KEY_P8` | Base64-encoded `AuthKey_<ID>.p8` | `base64 -i AuthKey_XXX.p8 \| pbcopy` then paste |

**Creating the ASC API key (operational TODO — executed in Plan 04):**
App Store Connect → Users and Access → Integrations → App Store Connect API → Create a new key with **App Manager** (or Admin) access. Download `AuthKey_<ID>.p8` (one-time download — save securely). Base64-encode it for the `ASC_API_KEY_P8` secret.

### 3. PrivacyInfo.xcprivacy .pbxproj Regen Owed Before Plan 04 Upload (Carry-Forward from 57-01)

From 57-01-SUMMARY.md: `grep 'PrivacyInfo' hp41-gui/src-tauri/gen/apple/hp41-gui.xcodeproj/project.pbxproj` returns 0 matches. The `project.yml` source entry is the source-of-truth, but the `.pbxproj` (which Xcode actually reads during archive) has not been regenerated.

**Action required before Plan 04 (TestFlight upload):** Run `just ios-init` to regenerate the `.pbxproj` from the updated `project.yml`, then verify `PrivacyInfo.xcprivacy` appears in the Xcode file tree and is included in a build phase. Without this step, the privacy manifest will NOT be in the IPA bundle, causing ITMS-91053 rejection from Apple.

## Decisions Made

1. **workflow_dispatch-only trigger (D-57.4):** No `push:` or `branches:` trigger — each upload burns a build number and exposes secrets. Manual dispatch is the correct gate. ROADMAP SC #4 is superseded by D-57.4 (CONTEXT.md is the authoritative source).

2. **Step-scoped signing env vars (T-57-06 mitigation):** `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_PATH` appear ONLY in the `Build signed IPA` step's `env:` block — never in the global `env:` block.

3. **PlistBuddy on a single run: line:** Collapsed the multi-line `\` form to a single `run:` command so the acceptance criterion grep `'PlistBuddy.*CFBundleVersion.*github.run_number'` matches on one line.

4. **macos-26 runner:** Explicit `macos-26` (not `macos-latest`) ensures Xcode 26.x is available. The lswiftCompatibility56 linker bug (Tauri #15066) does not affect Tauri 2.11.1 + Xcode 26.5 — confirmed via local Packaging.log (57-01 research).

5. **altool canonical search path:** The `.p8` is written to `~/.appstoreconnect/private_keys/AuthKey_<KEY_ID>.p8`. Both `xcodebuild -allowProvisioningUpdates` (via Tauri CLI) and `xcrun altool` search this path automatically — no explicit `--p8-file-path` flag needed.

## Deviations from Plan

None — plan executed exactly as written. The PlistBuddy line was collapsed from multi-line to single-line to satisfy the plan's own acceptance criterion grep pattern; this is faithful to the plan's intent.

## Security Controls Verified (from Threat Model)

| Threat | Control | Status |
|--------|---------|--------|
| T-57-04: .p8 in CI logs | `echo ... \| base64 --decode > file`; no `cat`/`echo` of decoded content | Mitigated |
| T-57-05: .p8 persists after failure | `if: always()` rm -rf step | Mitigated |
| T-57-06: Secrets over-scoped | Signing vars in build step `env:` only, not global | Mitigated |
| T-57-07: Untrusted trigger | `workflow_dispatch` only; no fork-PR surface | Mitigated |
| T-57-08: run_number injection | `github.run_number` is numeric context value, not user input | Accepted |
| T-57-09: P12/keychain attack | No `security create-keychain` / `security import` anywhere | Mitigated by design (D-57.2) |

## Known Stubs

None.

## Threat Flags

None — `ci-ios.yml` introduces no new network endpoints, auth paths, file access patterns, or schema changes. The ASC API key handling is the security-relevant surface; it is fully covered by the threat model in the plan.

## Verification Results

All acceptance criteria passed (python3 YAML parse + grep assertions):

```
YAML parse                                          → yaml-ok
grep workflow_dispatch                              → PASS
grep -E 'push:|branches:|tags:'                    → PASS (no match — correct)
grep 'runs-on: macos-26'                            → PASS
grep 'just ios-build-release'                       → PASS
grep -E 'run:.*(cargo |xcodebuild |npm run tauri)' → PASS (no match — correct)
grep 'PlistBuddy.*CFBundleVersion.*github.run_number' → PASS
grep 'if: always()'                                → PASS
grep 'rm -rf'                                      → PASS
grep -qF '"$IPA"'                                  → PASS (literal dollar-IPA found)
grep '--upload-package'                            → PASS
APPLE_API_KEY/ISSUER/KEY_PATH not in global env    → PASS
APPLE_API_KEY/ISSUER/KEY_PATH in build step env    → PASS
grep 'cat .*AuthKey|echo .*--decode.*|'            → PASS (no match — correct)
grep 'D-57.4', 'ASC_KEY_ID', 'ASC_ISSUER_ID', 'ASC_API_KEY_P8' in comment → PASS (all 4)
```

## Self-Check

### Files verified to exist:
- `.github/workflows/ci-ios.yml` ✓
- `.planning/phases/57-signing-testflight-pipeline/57-03-SUMMARY.md` ✓

### Commits verified:
- `2438d8a`: feat(57-03): add ci-ios.yml signed IPA build + TestFlight upload workflow (SHIP-04) (ci-ios.yml)
- `73fbbbc`: docs(57-03): complete ci-ios.yml plan — SUMMARY, STATE, ROADMAP, REQUIREMENTS
- Note: a mislabeled commit (`99f7442`) had inadvertently captured stray, out-of-phase CLI working-tree changes (`hp41-cli/src/{app,keys,ui}.rs`) via an unexpected git-workflow staging interaction. The orchestrator excised that commit from history (non-interactive `rebase --onto`, all commits still unpushed) and restored those changes as uncommitted working-tree WIP per the founder's "leave untouched" decision. ci-ios.yml is unaffected and lives in `2438d8a`.

## Self-Check: PASSED
