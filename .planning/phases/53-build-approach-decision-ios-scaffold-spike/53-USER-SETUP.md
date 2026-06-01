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
| Development team selected in Xcode | for target `hp41-gui_iOS` | ⬜ Plan 53-04 |
| Physical iPhone (iOS 17+) trusted + network-visible in Xcode | — | ⬜ Plan 53-04 |
| Distribution certificate + provisioning profile + ASC API key | — | ⬜ Phase 57 (NOT this phase) |

### Plan 53-02 — completed checklist
- [x] Apple Developer → Identifiers → register **Explicit** App ID `ch.talent-factory.hp41` (not wildcard).
- [x] App Store Connect → Apps → New App → select the bundle ID → create the app record.

### Plan 53-04 — pending checklist (human-only)
- [ ] Plug in the iPhone (iOS 17+), tap **Trust This Computer**.
- [ ] Xcode → Window → Devices and Simulators → enable **Connect via network** (P-iOS-02).
- [ ] Open `hp41-gui/src-tauri/gen/apple/hp41-gui.xcodeproj`.
- [ ] App target → Signing & Capabilities → **Automatically manage signing** on → set **Team**; Bundle Identifier reads `ch.talent-factory.hp41`; no signing error.
- [ ] Build + run on the device (Xcode Run, or `just ios-dev device="<name>"` once the LAN handshake is set up).

### Verification
- App ID visible: Apple Developer → Identifiers shows `ch.talent-factory.hp41` (explicit).
- App record visible: App Store Connect → Apps shows the new record.
- (53-04) Xcode shows a selected Team and no "requires a development team" error; the app installs + launches on the device and the RPN smoke `2 ENTER 3 + → 5`, then `SIN` dispatches through `hp41-core`.

### Security
- ⚠️ Do **not** commit any `.p12`, `.mobileprovision`, `.p8`/API key, or `.xcode.env.local`. None are created in Phase 53. Distribution signing material is introduced via GitHub Secrets in Phase 57 (never committed).
