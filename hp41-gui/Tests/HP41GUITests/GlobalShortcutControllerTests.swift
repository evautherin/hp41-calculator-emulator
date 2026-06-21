import XCTest
@testable import HP41GUI

#if os(macOS)
import Carbon

@MainActor
final class GlobalShortcutControllerTests: XCTestCase {
    func testSuccessfulReplacementUnregistersAndInstallsNewShortcut() throws {
        let registrar = FakeShortcutRegistrar()
        let controller = GlobalShortcutController(registrar: registrar)
        controller.start(with: "Control+H")
        try controller.replace(with: "Command+K")

        XCTAssertEqual(controller.accelerator, "Command+K")
        XCTAssertEqual(registrar.registered, ["Control+H", "Command+K"])
        XCTAssertEqual(registrar.unregisterCount, 2)
        XCTAssertNil(controller.errorMessage)
    }

    func testCollisionRollsBackPreviousRegistrationAndReportsError() throws {
        let registrar = FakeShortcutRegistrar()
        let controller = GlobalShortcutController(registrar: registrar)
        controller.start(with: "Control+H")
        registrar.failure = .registrationFailed(OSStatus(eventHotKeyExistsErr))

        XCTAssertThrowsError(try controller.replace(with: "Command+K"))
        XCTAssertEqual(controller.accelerator, "Control+H")
        XCTAssertEqual(registrar.registered, ["Control+H", "Control+H"])
        XCTAssertEqual(controller.errorMessage, "That shortcut is already used by another application.")
    }

    func testStopUnregistersActiveShortcut() {
        let registrar = FakeShortcutRegistrar()
        let controller = GlobalShortcutController(registrar: registrar)
        controller.start(with: "Control+H")
        controller.stop()
        XCTAssertNil(controller.accelerator)
        XCTAssertEqual(registrar.unregisterCount, 2)
    }
}

private final class FakeShortcutRegistrar: GlobalShortcutRegistering {
    var registered: [String] = []
    var unregisterCount = 0
    var failure: GlobalShortcutRegistrationError?

    func register(_ accelerator: String, action: @escaping () -> Void) throws {
        if let failure {
            self.failure = nil
            throw failure
        }
        registered.append(accelerator)
    }

    func unregister() { unregisterCount += 1 }
}
#endif
