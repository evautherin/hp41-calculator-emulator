import XCTest

final class HP41GUIUITests: XCTestCase {
    private var app: XCUIApplication!
    private var statePath: String!
    private var prefsPath: String!
    private var rawFixturePath: String!
    private var rawExportPath: String!
    private var dataCardPath: String!

    override func setUpWithError() throws {
        continueAfterFailure = false
        statePath = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-ui-\(UUID().uuidString).json").path
        prefsPath = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-ui-prefs-\(UUID().uuidString).json").path
        rawFixturePath = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-ui-getkey-\(UUID().uuidString).raw").path
        rawExportPath = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-ui-export-\(UUID().uuidString).raw").path
        dataCardPath = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-ui-data-\(UUID().uuidString).card.json").path
        try #"{"theme":"dark","onboarding_done":true,"macos_launch_mode":"window","global_shortcut":"Control+Alt+Command+H"}"#
            .write(toFile: prefsPath, atomically: true, encoding: .utf8)
        // LBL "A"; GETKEY; END in the bridge's documented bare RAW subset.
        try Data([0xCF, 0xF1, 0x41, 0xCE, 0xC0, 0x00, 0x0D])
            .write(to: URL(fileURLWithPath: rawFixturePath))
        app = XCUIApplication()
        // A failed hide/menu-bar workflow can leave LaunchServices reporting
        // the test bundle as background-running even after its last window is
        // gone. Start every workflow from an explicit terminated state.
        app.terminate()
        app.launchEnvironment["HP41_STATE_PATH"] = statePath
        app.launchEnvironment["HP41_PREFS_PATH"] = prefsPath
        app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 5))
    }

    override func tearDownWithError() throws {
        app?.terminate()
        if let statePath { try? FileManager.default.removeItem(atPath: statePath) }
        if let prefsPath { try? FileManager.default.removeItem(atPath: prefsPath) }
        if let rawFixturePath { try? FileManager.default.removeItem(atPath: rawFixturePath) }
        if let rawExportPath { try? FileManager.default.removeItem(atPath: rawExportPath) }
        if let dataCardPath { try? FileManager.default.removeItem(atPath: dataCardPath) }
    }

    func testCalculatorWindowExposesNativeControls() {
        XCTAssertTrue(app.staticTexts["calculator-display"].exists)
        for identifier in ["key-on", "key-0", "key-1", "key-enter", "key-plus", "key-shift", "key-alpha_toggle", "key-sst", "key-r_s"] {
            XCTAssertTrue(app.buttons[identifier].exists, "Missing native button \(identifier)")
        }
    }

    func testRPNSequenceUpdatesRenderedDisplay() {
        tap("2"); tap("enter"); tap("3"); tap("plus")
        assertDisplay("5.0000")
    }

    func testShiftedPiIsOneShotInRenderedUI() {
        tap("shift")
        XCTAssertEqual(app.staticTexts["annunciator-f"].value as? String, "active")
        tap("0")
        assertDisplay("3.1416")
        XCTAssertEqual(app.staticTexts["annunciator-f"].value as? String, "inactive")
    }

    func testAlphaKeysUpdateRenderedDisplay() {
        tap("alpha_toggle"); tap("sigma_plus"); tap("recip")
        assertDisplay("AB")
    }

    func testPhysicalKeyboardDrivesCalculator() {
        app.windows.firstMatch.click()
        app.typeText("2")
        app.typeKey(.return, modifierFlags: [])
        app.typeText("3+")
        assertDisplay("5.0000")
    }

    func testOnTapSoftResetsWithoutErasingProgramMemory() {
        tap("prgm_mode"); tap("9"); tap("prgm_mode")
        tap("7"); tap("on")
        assertDisplay("0.0000")

        tap("prgm_mode")
        assertDisplay("000 9.000000000")
    }

    func testOnLongPressRequiresConfirmationBeforeFullReset() {
        tap("prgm_mode"); tap("9"); tap("prgm_mode")
        app.buttons["key-on"].press(forDuration: 0.8)

        XCTAssertTrue(app.staticTexts["MEMORY LOST"].waitForExistence(timeout: 2))
        app.buttons["Cancel"].click()
        tap("prgm_mode")
        assertDisplay("000 9.000000000")
        tap("prgm_mode")

        app.buttons["key-on"].press(forDuration: 0.8)
        XCTAssertTrue(app.buttons["Confirm"].waitForExistence(timeout: 2))
        app.buttons["Confirm"].click()
        tap("prgm_mode")
        assertDisplay("000 END")
    }

    func testProgramModeRecordsDisplaysAndRunsProgram() {
        tap("prgm_mode")
        assertDisplay("000 END")
        tap("2"); tap("enter"); tap("3"); tap("plus")
        assertDisplay("000 2.000000000")
        tap("sst")
        assertDisplay("001 ENTER")
        tap("shift"); tap("sst") // BST
        assertDisplay("000 2.000000000")
        tap("prgm_mode")
        tap("r_s")
        assertDisplay("5.0000")
    }

    func testExpandableProgramListingTracksPCAndOffersDesktopControls() {
        tap("prgm_mode"); tap("2"); tap("enter")

        XCTAssertEqual(app.staticTexts["program-counter"].label, "PC 000")
        app.buttons["program-listing-toggle"].click()
        XCTAssertTrue(app.staticTexts["program-step-0"].waitForExistence(timeout: 2))
        XCTAssertEqual(app.staticTexts["program-step-0"].label, "Current step 000 2.000000000")

        app.buttons["program-sst"].click()
        XCTAssertEqual(app.staticTexts["program-counter"].label, "PC 001")
        XCTAssertEqual(app.staticTexts["program-step-1"].label, "Current step 001 ENTER")

        app.buttons["program-bst"].click()
        XCTAssertEqual(app.staticTexts["program-counter"].label, "PC 000")
        XCTAssertTrue(app.buttons["program-run-stop"].exists)
    }

    func testXEQFunctionSearchExecutesCanonicalFunction() {
        tap("2"); tap("chs"); tap("xeq_prompt")
        let search = app.textFields["function-search"]
        XCTAssertTrue(search.waitForExistence(timeout: 2))
        search.typeText("ABS")
        search.typeKey(.return, modifierFlags: [])
        assertDisplay("2.0000")
    }

    func testStoreAndRecallParameterSheets() {
        tap("7"); tap("sto_prompt")
        let store = app.textFields["parameter-register"]
        XCTAssertTrue(store.waitForExistence(timeout: 2))
        store.typeText("5")
        store.typeKey(.return, modifierFlags: [])
        tap("clx"); tap("rcl_prompt")
        let recall = app.textFields["parameter-register"]
        XCTAssertTrue(recall.waitForExistence(timeout: 2))
        recall.typeText("5")
        recall.typeKey(.return, modifierFlags: [])
        assertDisplay("7.0000")
    }

    func testSettingsHelpPrinterAndGuideWorkflowsExposeStableAccessibility() {
        app.buttons["settings-open"].click()
        XCTAssertTrue(element("settings-sheet").waitForExistence(timeout: 2))
        XCTAssertTrue(element("theme-picker").exists)
        app.buttons["show-guide"].click()

        XCTAssertTrue(element("onboarding-guide").waitForExistence(timeout: 3))
        XCTAssertEqual(element("onboarding-progress").label, "1 of 5")
        app.buttons["onboarding-next"].click()
        XCTAssertEqual(element("onboarding-progress").label, "2 of 5")
        app.buttons["onboarding-close"].click()

        app.buttons["help-open"].click()
        XCTAssertTrue(element("help-reference").waitForExistence(timeout: 2))
        XCTAssertTrue(element("help-tab").exists)
        app.buttons["help-close"].click()

        app.buttons["printer-open"].click()
        XCTAssertTrue(element("printer-tape").waitForExistence(timeout: 2))
        XCTAssertTrue(element("printer-empty").exists)
        app.buttons["printer-close"].click()
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))
    }

    func testRegisterParameterSheetSupportsFocusAndKeyboardDismissal() {
        tap("7"); tap("sto_prompt")
        XCTAssertTrue(element("parameter-sheet").waitForExistence(timeout: 2))
        let field = app.textFields["parameter-register"]
        XCTAssertTrue(field.exists)
        field.typeText("5")
        app.buttons["parameter-cancel"].click()
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))
    }

    func testHelpRunUsesCanonicalCalculatorDispatch() {
        tap("2"); tap("chs")
        app.buttons["help-open"].click()
        let search = app.textFields["help-search"]
        XCTAssertTrue(search.waitForExistence(timeout: 2))
        search.typeText("ABS")

        let entry = element("help-entry-Abs")
        XCTAssertTrue(entry.waitForExistence(timeout: 2))
        entry.click()
        XCTAssertEqual(entry.value as? String, "expanded")
        let run = app.buttons["help-run-Abs"]
        XCTAssertTrue(run.waitForExistence(timeout: 2))
        run.click()
        assertDisplay("2.0000")
    }

    func testPrinterWorkflowExposesAccumulatedOutputSemantically() {
        tap("shift"); tap("7") // SF
        let flag = app.textFields["parameter-register"]
        XCTAssertTrue(flag.waitForExistence(timeout: 2))
        flag.typeText("55")
        flag.typeKey(.return, modifierFlags: [])

        tap("7")
        app.windows.firstMatch.click()
        app.typeText("P") // PRX

        XCTAssertTrue(element("printer-tape").waitForExistence(timeout: 2))
        let line = app.staticTexts["printer-line-0"]
        XCTAssertTrue(line.waitForExistence(timeout: 2))
        XCTAssertTrue((line.value as? String)?.contains("7.0000") == true)
        app.buttons["printer-clear"].click()
        XCTAssertTrue(element("printer-empty").waitForExistence(timeout: 2))
    }

    func testThemeSelectionUpdatesAccessibilityAndPersistsPreference() {
        app.buttons["settings-open"].click()
        let picker = element("theme-picker")
        XCTAssertTrue(picker.waitForExistence(timeout: 2))
        app.radioButtons["High Contrast"].click()
        XCTAssertEqual(picker.value as? String, "High Contrast")

        let persisted = NSPredicate { [prefsPath] _, _ in
            guard let prefsPath,
                  let data = FileManager.default.contents(atPath: prefsPath),
                  let text = String(data: data, encoding: .utf8) else { return false }
            return text.contains(#""theme" : "high-contrast""#)
        }
        expectation(for: persisted, evaluatedWith: NSObject())
        waitForExpectations(timeout: 2)
    }

    func testShortcutRecorderCapturesAndSavesModifiedKey() {
        app.buttons["settings-open"].click()
        app.buttons["shortcut-record-open"].click()
        XCTAssertTrue(element("shortcut-recorder").waitForExistence(timeout: 2))

        app.typeKey("k", modifierFlags: [.control, .option])
        let preview = app.staticTexts["shortcut-preview"]
        XCTAssertTrue((preview.value as? String)?.contains("K") == true)
        XCTAssertTrue(app.buttons["shortcut-save"].isEnabled)
        app.buttons["shortcut-save"].click()

        let shortcut = app.buttons["shortcut-record-open"]
        XCTAssertTrue(shortcut.waitForExistence(timeout: 2))
        XCTAssertTrue((shortcut.value as? String)?.contains("K") == true)
    }

    func testCalculatorMenuCommandsOpenNativeWorkflows() {
        chooseCalculatorMenuItem("Execute Function…")
        XCTAssertTrue(element("function-sheet").waitForExistence(timeout: 2))
        app.buttons["function-cancel"].click()

        chooseCalculatorMenuItem("Show Printer Tape")
        XCTAssertTrue(element("printer-tape").waitForExistence(timeout: 2))
        app.buttons["printer-close"].click()

        tap("7")
        chooseCalculatorMenuItem("Soft Reset")
        assertDisplay("0.0000")
    }

    func testProgramPauseExposesStatusAndAutomaticallyResumes() {
        tap("prgm_mode")
        tap("shift"); tap("sto_prompt") // LBL
        let label = app.textFields["parameter-label"]
        XCTAssertTrue(label.waitForExistence(timeout: 2))
        label.typeText("A")
        label.typeKey(.return, modifierFlags: [])
        tap("1")

        chooseCalculatorMenuItem("Execute Function…")
        let search = app.textFields["function-search"]
        XCTAssertTrue(search.waitForExistence(timeout: 2))
        search.typeText("PSE")
        search.typeKey(.return, modifierFlags: [])
        tap("2"); tap("plus"); tap("prgm_mode"); tap("r_s")

        let status = app.staticTexts["program-yield-status"]
        XCTAssertTrue(status.waitForExistence(timeout: 2))
        XCTAssertEqual(status.value as? String, "pse")
        assertDisplay("3.0000", timeout: 3)
        XCTAssertFalse(status.exists)
    }

    func testRawImportRunsGETKEYAndResumesFromOnScreenHardwareKey() {
        chooseCalculatorMenuItem("Import Program…")
        chooseFileInOpenPanel(rawFixturePath)
        let imported = app.alerts["Program Import Complete"]
        XCTAssertTrue(imported.waitForExistence(timeout: 3))
        imported.buttons["OK"].click()

        tap("r_s")
        let status = app.staticTexts["program-yield-status"]
        XCTAssertTrue(status.waitForExistence(timeout: 2))
        XCTAssertEqual(status.value as? String, "wait_for_key")
        tap("sigma_plus") // canonical HP-41 hardware code 11
        assertDisplay("11.0000")
        XCTAssertFalse(status.exists)
    }

    func testExplicitSaveRestoresRenderedStateAfterRelaunch() {
        tap("4")
        chooseCalculatorMenuItem("Save State")
        XCTAssertTrue(FileManager.default.fileExists(atPath: statePath))

        app.terminate()
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 5))
        assertDisplay("4")
    }

    func testFirstRunGuideCompletesPersistsAndDoesNotReappear() throws {
        app.terminate()
        try #"{"theme":"dark","onboarding_done":false,"macos_launch_mode":"window","global_shortcut":"Control+Alt+Command+H"}"#
            .write(toFile: prefsPath, atomically: true, encoding: .utf8)
        app.launch()

        let guide = element("onboarding-guide")
        XCTAssertTrue(guide.waitForExistence(timeout: 5))
        XCTAssertEqual(element("onboarding-progress").value as? String, "Page 1 of 5")
        XCTAssertFalse(app.buttons["onboarding-back"].isEnabled)
        for page in 2...5 {
            app.buttons["onboarding-next"].click()
            XCTAssertEqual(element("onboarding-progress").value as? String, "Page \(page) of 5")
        }
        app.buttons["onboarding-finish"].click()
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))

        app.terminate()
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 5))
        XCTAssertFalse(guide.waitForExistence(timeout: 1))
        XCTAssertTrue(app.buttons["key-enter"].exists)
    }

    func testRawExportWritesCanonicalProgramBytesThroughSavePanel() throws {
        tap("prgm_mode"); tap("sqrt"); tap("prgm_mode")
        chooseCalculatorMenuItem("Export Program…")
        chooseLocationInSavePanel(rawExportPath)

        let exported = app.alerts["Program Exported"]
        XCTAssertTrue(exported.waitForExistence(timeout: 3))
        exported.buttons["OK"].click()
        XCTAssertEqual(try Data(contentsOf: URL(fileURLWithPath: rawExportPath)),
                       Data([0x52, 0xC0, 0x00, 0x0D]))
    }

    func testDataCardExportAndImportRoundTripThroughNativePanels() throws {
        tap("4"); tap("sto_prompt")
        let store = app.textFields["parameter-register"]
        XCTAssertTrue(store.waitForExistence(timeout: 2))
        store.typeText("7")
        store.typeKey(.return, modifierFlags: [])

        chooseCalculatorMenuItem("Export Data Card…")
        chooseLocationInSavePanel(dataCardPath)
        let exported = app.alerts["Data Card Exported"]
        XCTAssertTrue(exported.waitForExistence(timeout: 3))
        exported.buttons["OK"].click()
        let json = try String(contentsOfFile: dataCardPath, encoding: .utf8)
        XCTAssertTrue(json.contains("hp41-data-v1"))

        tap("0"); tap("sto_prompt")
        let overwrite = app.textFields["parameter-register"]
        XCTAssertTrue(overwrite.waitForExistence(timeout: 2))
        overwrite.typeText("7")
        overwrite.typeKey(.return, modifierFlags: [])

        chooseCalculatorMenuItem("Import Data Card…")
        chooseFileInOpenPanel(dataCardPath)
        let imported = app.alerts["Data Card Imported"]
        XCTAssertTrue(imported.waitForExistence(timeout: 3))
        imported.buttons["OK"].click()
        tap("clx"); tap("rcl_prompt")
        let recall = app.textFields["parameter-register"]
        XCTAssertTrue(recall.waitForExistence(timeout: 2))
        recall.typeText("7")
        recall.typeKey(.return, modifierFlags: [])
        assertDisplay("4.0000")
    }

    func testEscapeDismissesNativeOverlaysAndNestedShortcutRecorder() {
        app.buttons["settings-open"].click()
        XCTAssertTrue(element("settings-sheet").waitForExistence(timeout: 2))
        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))

        app.buttons["help-open"].click()
        XCTAssertTrue(element("help-reference").waitForExistence(timeout: 2))
        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))

        app.buttons["printer-open"].click()
        XCTAssertTrue(element("printer-tape").waitForExistence(timeout: 2))
        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))

        app.buttons["settings-open"].click()
        app.buttons["shortcut-record-open"].click()
        XCTAssertTrue(element("shortcut-recorder").waitForExistence(timeout: 2))
        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(element("settings-sheet").waitForExistence(timeout: 2))
        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(app.buttons["key-enter"].waitForExistence(timeout: 2))
    }

    func testEscapeCancelsGETKEYWithHardwareSentinelZero() {
        chooseCalculatorMenuItem("Import Program…")
        chooseFileInOpenPanel(rawFixturePath)
        let imported = app.alerts["Program Import Complete"]
        XCTAssertTrue(imported.waitForExistence(timeout: 3))
        imported.buttons["OK"].click()

        tap("r_s")
        let status = app.staticTexts["program-yield-status"]
        XCTAssertTrue(status.waitForExistence(timeout: 2))
        app.typeKey(.escape, modifierFlags: [])
        assertDisplay("0.0000")
        XCTAssertFalse(status.exists)
    }

    func testCalculatorMenuCommandHidesAndRestoresWindow() {
        let window = app.windows.firstMatch
        XCTAssertTrue(window.isHittable)
        chooseCalculatorMenuItem("Show/Hide Calculator")
        let hidden = NSPredicate { object, _ in
            guard let element = object as? XCUIElement else { return false }
            return !element.isHittable
        }
        expectation(for: hidden, evaluatedWith: window)
        waitForExpectations(timeout: 2)

        app.typeKey("h", modifierFlags: [.command, .option])
        XCTAssertTrue(window.waitForExistence(timeout: 2))
        let shown = NSPredicate { object, _ in
            (object as? XCUIElement)?.isHittable == true
        }
        expectation(for: shown, evaluatedWith: window)
        waitForExpectations(timeout: 2)
    }

    func testStatefulControlsExposeSelectedExpandedAndModeValues() {
        XCTAssertEqual(app.buttons["key-shift"].value as? String, "inactive")
        tap("shift")
        XCTAssertEqual(app.buttons["key-shift"].value as? String, "active")
        tap("shift")

        XCTAssertEqual(app.buttons["key-prgm_mode"].value as? String, "inactive")
        tap("prgm_mode")
        XCTAssertEqual(app.buttons["key-prgm_mode"].value as? String, "active")
        XCTAssertEqual(app.buttons["program-listing-toggle"].value as? String, "collapsed")
        app.buttons["program-listing-toggle"].click()
        XCTAssertEqual(app.buttons["program-listing-toggle"].value as? String, "expanded")
        tap("prgm_mode")

        XCTAssertEqual(app.buttons["key-alpha_toggle"].value as? String, "inactive")
        tap("alpha_toggle")
        XCTAssertEqual(app.buttons["key-alpha_toggle"].value as? String, "active")
        tap("alpha_toggle")

        app.buttons["settings-open"].click()
        XCTAssertEqual(element("theme-picker").value as? String, "Dark")
        XCTAssertEqual(element("launch-mode-picker").value as? String, "Window")
        XCTAssertFalse((app.buttons["shortcut-record-open"].value as? String)?.isEmpty ?? true)
        app.typeKey(.escape, modifierFlags: [])

        tap("7"); tap("sto_prompt")
        let indirect = app.checkBoxes["parameter-indirect"]
        XCTAssertTrue(indirect.waitForExistence(timeout: 2))
        XCTAssertFalse(indirect.isSelected)
        indirect.click()
        XCTAssertTrue(indirect.isSelected)
        app.buttons["parameter-cancel"].click()
        XCTAssertTrue(app.buttons["key-enter"].isHittable)
    }

    func testScreenshotAndCalculatorIdentity() {
        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = "Native HP-41 SwiftUI"
        attachment.lifetime = .keepAlways
        add(attachment)
        XCTAssertTrue(app.staticTexts["calculator-model"].exists)
    }

    private func tap(_ key: String, file: StaticString = #filePath, line: UInt = #line) {
        let button = app.buttons["key-\(key)"]
        XCTAssertTrue(button.waitForExistence(timeout: 2), "Missing key \(key)", file: file, line: line)
        button.click()
    }

    private func element(_ identifier: String) -> XCUIElement {
        app.descendants(matching: .any)[identifier]
    }

    private func chooseCalculatorMenuItem(_ title: String) {
        app.menuBars.menuBarItems["Calculator"].click()
        let item = app.menuItems[title]
        XCTAssertTrue(item.waitForExistence(timeout: 2), "Missing Calculator menu item \(title)")
        item.click()
    }

    private func chooseFileInOpenPanel(_ path: String) {
        let dialog = app.dialogs.firstMatch
        XCTAssertTrue(dialog.waitForExistence(timeout: 2), "Open panel did not appear")
        dialog.typeKey("g", modifierFlags: [.command, .shift])
        let locationSheet = dialog.sheets.firstMatch
        XCTAssertTrue(locationSheet.waitForExistence(timeout: 2), "Go to location sheet did not appear")
        let location = locationSheet.textFields.firstMatch
        XCTAssertTrue(location.waitForExistence(timeout: 2), "Location field did not appear")
        location.typeText(path)
        location.typeKey(.return, modifierFlags: [])
        let open = dialog.buttons["Open"]
        XCTAssertTrue(open.waitForExistence(timeout: 2), "Open button did not appear")
        open.click()
    }

    private func chooseLocationInSavePanel(_ path: String) {
        let dialog = app.dialogs.firstMatch
        XCTAssertTrue(dialog.waitForExistence(timeout: 2), "Save panel did not appear")
        dialog.typeKey("g", modifierFlags: [.command, .shift])
        let locationSheet = dialog.sheets.firstMatch
        XCTAssertTrue(locationSheet.waitForExistence(timeout: 2), "Go to location sheet did not appear")
        let location = locationSheet.textFields.firstMatch
        XCTAssertTrue(location.waitForExistence(timeout: 2), "Location field did not appear")
        location.typeText(URL(fileURLWithPath: path).deletingLastPathComponent().path)
        location.typeKey(.return, modifierFlags: [])
        XCTAssertFalse(locationSheet.waitForExistence(timeout: 2))

        let namedField = dialog.textFields["Save As:"]
        let fileName = namedField.exists ? namedField : dialog.textFields.firstMatch
        XCTAssertTrue(fileName.waitForExistence(timeout: 2), "Save As field did not appear")
        fileName.click()
        fileName.typeKey("a", modifierFlags: .command)
        fileName.typeText(URL(fileURLWithPath: path).lastPathComponent)
        let save = dialog.buttons["Save"]
        XCTAssertTrue(save.waitForExistence(timeout: 2), "Save button did not appear")
        save.click()
    }

    private func assertDisplay(_ expected: String, timeout: TimeInterval = 2,
                               file: StaticString = #filePath, line: UInt = #line) {
        let display = app.staticTexts["calculator-display"]
        let predicate = NSPredicate { object, _ in
            guard let element = object as? XCUIElement else { return false }
            return element.label.contains(expected) || (element.value as? String)?.contains(expected) == true
        }
        expectation(for: predicate, evaluatedWith: display)
        waitForExpectations(timeout: timeout)
    }
}
