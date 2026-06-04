# Phase 53 — User Setup (Apple Developer / iOS)

External Apple services require manual, human-only configuration for this phase.
**No secrets are stored in the repo.** Distribution signing (certs, profiles, API keys)
is deferred to Phase 57.

## Service: Apple Developer / App Store Connect

| Item | Value | Status |
|------|-------|--------|
| Apple Developer Program membership | enrolled + active (D-53.10) | ✅ |
| Explicit App ID | `ch.talent-factory.hp41` (matches `tauri.conf.json` `identifier`) | ✅ registered (Plan 53-02, 2026-06-01) |
| App Store Connect app record | `ch.talent-factory.hp41` | ✅ created (Plan 53-02, 2026-06-01) |
| Development team selected (`2P4R8QSWT4`, Talent Factory AG) | committed in `project.yml` | ✅ Plan 53-04 (2026-06-02) |
| Physical iPhone (iOS 17+) trusted + Developer Mode on | iPhone 15 Pro "DS" | ✅ Plan 53-04 (2026-06-02) |
| Distribution certificate + provisioning profile + ASC API key | — | ⬜ Phase 57 (NOT this phase) |

### Plan 53-02 — completed checklist
- [x] Apple Developer → Identifiers → register **Explicit** App ID `ch.talent-factory.hp41` (not wildcard).
- [x] App Store Connect → Apps → New App → select the bundle ID → create the app record.

### Plan 53-04 — completed checklist
- [x] Plugged in the iPhone (iOS 17+), trusted the computer.
- [x] Selected the signing **Team** (Talent Factory AG / `2P4R8QSWT4`); no signing error.
- [x] Enabled **Developer Mode** on the device (Settings → Privacy & Security → Developer Mode → restart).
- [x] Built a signed IPA via `just ios-build` and installed + launched it via `xcrun devicectl`.
- [x] On-device RPN smoke `2 ENTER 3 + → 5`, then `SIN → 0.0872` (sin 5° in DEG) confirmed.

> **Device-install note (carry-forward):** Xcode's **debug ⌘R** panics on the missing
> Tauri dev-server addr file (it expects `tauri ios dev` running). For a standalone
> install use the **release `just ios-build` IPA + `xcrun devicectl install/launch`**
> flow (bundles the frontend, no dev server), or run `tauri ios dev` for hot reload.

### Verification
- App ID visible: Apple Developer → Identifiers shows `ch.talent-factory.hp41` (explicit).
- App record visible: App Store Connect → Apps shows the new record.
- (53-04) Xcode shows a selected Team and no "requires a development team" error; the app installs + launches on the device and the RPN smoke `2 ENTER 3 + → 5`, then `SIN` dispatches through `hp41-core`.

### Security
- ⚠️ Do **not** commit any `.p12`, `.mobileprovision`, `.p8`/API key, or `.xcode.env.local`. None are created in Phase 53. Distribution signing material is introduced via GitHub Secrets in Phase 57 (never committed).
