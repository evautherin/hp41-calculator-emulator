import XCTest
@testable import HP41GUI

@MainActor
final class AppPreferencesTests: XCTestCase {
    func testAllFourCanonicalThemesAreAvailable() {
        XCTAssertEqual(AppTheme.allCases.map(\.rawValue),
                       ["dark", "light", "classic-beige", "high-contrast"])
    }

    func testThemePersistsUsingLegacyCompatiblePreferenceKey() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let url = directory.appendingPathComponent("prefs.json")
        defer { try? FileManager.default.removeItem(at: directory) }

        let preferences = AppPreferences(url: url)
        preferences.setTheme(.classicBeige)
        XCTAssertEqual(AppPreferences(url: url).theme, .classicBeige)
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any])
        XCTAssertEqual(object["theme"] as? String, "classic-beige")
        XCTAssertNil(object["display_mode"])
    }

    func testUnknownOrCorruptThemeFallsBackToDark() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let url = directory.appendingPathComponent("prefs.json")
        defer { try? FileManager.default.removeItem(at: directory) }
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try #"{"theme":"neon","onboarding_done":false,"macos_launch_mode":"menu-bar","global_shortcut":"Control+Alt+Command+H"}"#.write(to: url, atomically: true, encoding: .utf8)
        XCTAssertEqual(AppPreferences(url: url).theme, .dark)
    }

    func testHighContrastPaletteExceedsWCAGAAAForPrimarySurfaces() {
        let palette = AppTheme.highContrast.palette
        XCTAssertGreaterThanOrEqual(palette.primaryText.contrastRatio(with: palette.key), 7)
        XCTAssertGreaterThanOrEqual(palette.displayText.contrastRatio(with: palette.displayBackground), 7)
    }

    func testOnboardingCompletionPersistsSeparatelyFromCalculatorState() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let url = directory.appendingPathComponent("prefs.json")
        defer { try? FileManager.default.removeItem(at: directory) }
        let preferences = AppPreferences(url: url)
        XCTAssertFalse(preferences.onboardingDone)
        preferences.completeOnboarding()
        XCTAssertTrue(AppPreferences(url: url).onboardingDone)
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any])
        XCTAssertEqual(object["onboarding_done"] as? Bool, true)
        XCTAssertNil(object["state"])
    }

    func testQuickStartHasExactlyFiveStableSteps() {
        XCTAssertEqual(OnboardingPage.all.map(\.id), [0, 1, 2, 3, 4])
        XCTAssertEqual(Set(OnboardingPage.all.map(\.title)).count, 5)
    }

    func testLaunchModeAndGlobalShortcutPersistWithLegacyKeys() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let url = directory.appendingPathComponent("prefs.json")
        defer { try? FileManager.default.removeItem(at: directory) }
        let preferences = AppPreferences(url: url)
        preferences.setMacOSLaunchMode("window")
        XCTAssertTrue(preferences.setGlobalShortcut("Control+Shift+K"))
        let loaded = AppPreferences(url: url)
        XCTAssertEqual(loaded.macosLaunchMode, "window")
        XCTAssertEqual(loaded.globalShortcut, "Control+Shift+K")
    }

    func testShortcutCaptureFormattingAndValidationMatchesTauriGrammar() {
        XCTAssertEqual(
            ShortcutAccelerator.capture(key: "h", control: true, alt: true, shift: false, command: true),
            "Control+Alt+Command+H"
        )
        XCTAssertEqual(ShortcutAccelerator.formatted("Control+Alt+Command+H"), "⌃⌥⌘H")
        XCTAssertTrue(ShortcutAccelerator.isValid("Command+Shift+F12"))
        XCTAssertFalse(ShortcutAccelerator.isValid("H"))
        XCTAssertFalse(ShortcutAccelerator.isValid("Command+Escape"))
        XCTAssertNil(ShortcutAccelerator.capture(key: "H", control: false, alt: false, shift: false, command: false))
    }
}
