import XCTest
@testable import HP41GUI

final class NativePlatformTests: XCTestCase {
    func testDeclaredSwiftPackageTargetUsesCompileTimePlatformFacts() {
        #if os(macOS)
        XCTAssertEqual(NativePlatform.current, .macOS)
        XCTAssertTrue(NativePlatform.isMacOS)
        XCTAssertFalse(NativePlatform.isIOS)
        #else
        XCTAssertEqual(NativePlatform.current, .iOS)
        XCTAssertFalse(NativePlatform.isMacOS)
        XCTAssertTrue(NativePlatform.isIOS)
        #endif
    }

    func testTouchTargetPolicyIsPlatformNative() {
        XCTAssertEqual(NativePlatform.minimumTouchTarget, NativePlatform.isIOS ? 44 : 0)
    }

    #if os(macOS)
    func testMenuBarShellPolicyHidesCompletedOnboardingWindowUntilRequested() {
        let policy = MacPlatformShellConfiguration(launchMode: .menuBar, onboardingDone: true)
        XCTAssertTrue(policy.usesStatusItem)
        XCTAssertFalse(policy.showsWindowAtLaunch)
        XCTAssertTrue(policy.hidesWindowWhenInactive)
    }

    func testFirstRunOverridesMenuBarWindowHiding() {
        let policy = MacPlatformShellConfiguration(launchMode: .menuBar, onboardingDone: false)
        XCTAssertTrue(policy.usesStatusItem)
        XCTAssertTrue(policy.showsWindowAtLaunch)
        XCTAssertFalse(policy.hidesWindowWhenInactive)
    }

    func testWindowShellPolicyUsesNormalVisibleApplicationWindow() {
        let policy = MacPlatformShellConfiguration(launchMode: .window, onboardingDone: true)
        XCTAssertFalse(policy.usesStatusItem)
        XCTAssertTrue(policy.showsWindowAtLaunch)
        XCTAssertFalse(policy.hidesWindowWhenInactive)
        XCTAssertEqual(MacLaunchMode(storedValue: "unexpected"), .menuBar)
    }
    #endif
}
