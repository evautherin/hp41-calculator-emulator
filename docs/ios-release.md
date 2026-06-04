# iOS Release Runbook (TestFlight + App Store)

How to ship the HP-41 iOS app. Two distinct tracks:

- **TestFlight build** (testing) — fully automated by the `ci-ios` workflow.
- **App Store version** (public release) — automated build + **manual** App Store Connect
  version page and Apple review.

> Signing background: Xcode 26's `-allowProvisioningUpdates` (Tauri's built-in export)
> returns 401 for ASC API keys, so the pipeline uses **manual signing** with an explicit
> Apple Distribution cert + App Store provisioning profile. Full recipe + gotchas:
> the `reference_ios_signing_testflight` memory and `docs/hp41cv-divergences.md`.

---

## What's automated vs manual

| Step | Who |
|------|-----|
| Build, sign, export IPA, upload to TestFlight | **`ci-ios` workflow** (`just ios-build-release`) |
| Build number (unique, increasing) | **Automatic** — `github.run_number` |
| TestFlight internal distribution | Automatic once the build is processed |
| App Store version page + metadata + **Submit for Review** | **Manual** (App Store Connect) — Apple requires it |

---

## One-time prerequisites

These are already done unless noted:

1. **GitHub repo secrets** (Settings → Secrets and variables → Actions):
   - `ASC_KEY_ID`, `ASC_ISSUER_ID`, `ASC_API_KEY_P8` — App Store Connect API key (team `2P4R8QSWT4`).
   - `IOS_DIST_CERT_P12`, `IOS_DIST_CERT_PASSWORD`, `IOS_PROVISION_PROFILE_B64` — Apple Distribution cert + App Store profile (`HP-41 App Store (ci-ios)`).
2. **App registered** in App Store Connect — bundle `ch.talent-factory.hp41` (app id `6776616594`).
3. ⚠️ **OPEN — required before the first PUBLIC App Store submission:**
   - **`PrivacyInfo.xcprivacy` in the shipped bundle.** App Store review *enforces* the
     required-reason-API manifest (C617.1, file-timestamp from autosave); a missing manifest
     is a warning on TestFlight (ITMS-91053) but a **rejection** at review. Tracked:
     `.planning/todos/pending/privacy-manifest-bundle-wiring.md`.
   - **App Store metadata** (one-time, then maintained): screenshots in the required sizes,
     description, keywords, support URL, **privacy policy URL**, age rating, category.
   - Optional: set **`ITSAppUsesNonExemptEncryption = false`** in `gen/apple/hp41-gui_iOS/Info.plist`
     (the app uses no non-exempt encryption) to skip the export-compliance prompt on every upload.

---

## Recurring: new TestFlight build (testing)

1. Land changes on `develop`.
2. Bump the **marketing version** in `hp41-gui/src-tauri/tauri.conf.json` → `"version"`
   (e.g. `4.1.0` → `4.2.0`) if this is a new version line; commit + push.
   - Tauri reads the bundle version from here, **not** `gen/apple/Info.plist`.
3. GitHub → **Actions → `ci-ios` → Run workflow → branch `develop`**.
4. Wait for the green run (~5–8 min). After Apple processing the build appears in
   App Store Connect → **TestFlight** with build number = the run number.
5. Internal testers can install immediately. (External testers require a one-time **Beta App Review**.)

## Recurring: new App Store version (public)

Do the TestFlight steps above first (they produce + upload the build), then:

1. App Store Connect → the app → **App Store** tab → **(+) Version or Platform** → create the
   version matching the marketing version from step 2.
2. Fill **"What's New in This Version"**; update screenshots/description only if changed.
3. Under **Build**, select the build uploaded by `ci-ios`.
4. Answer export compliance (unless `ITSAppUsesNonExemptEncryption` is preset) + content rights.
5. **Add for Review → Submit for Review.** Apple review is typically hours to ~1–2 days.
6. On **Approved**: release automatically, manually, or on a schedule.

---

## Versioning rules

- **Marketing version** (`tauri.conf.json` `version` → `CFBundleShortVersionString`) — the public
  `X.Y(.Z)` shown in the App Store. Bump per release.
- **Build number** (`CFBundleVersion`) — must be unique and strictly increasing *within a marketing
  version*. CI stamps it from `github.run_number` (via `IOS_BUILD_NUMBER` in `ios-build-release`);
  never set it by hand.

## Secret / signing rotation

- The Apple Distribution cert (`N7QPP6P52Q`) and App Store profile (`HP-41 App Store (ci-ios)`)
  were created via the ASC API. To rotate: create a new cert + profile, build a new `.p12`
  (`openssl pkcs12 -export -legacy …`), and update the three `IOS_*` secrets.
- ⚠️ `just ios-init` regenerates the Xcode project and **resets the release config from Manual
  back to Automatic** (and can duplicate `libapp.a`). Avoid it; if you must run it, re-apply
  `CODE_SIGN_STYLE = Manual` + the cert identity + `PROVISIONING_PROFILE_SPECIFIER` to the
  release config afterward.

## See also

- `.github/workflows/ci-ios.yml`, `.github/ios-export-options.plist`, `Justfile` (`ios-build-release`)
- `docs/hp41cv-divergences.md` (iOS divergences), `docs/adr/v4.1-*` (ADRs)
