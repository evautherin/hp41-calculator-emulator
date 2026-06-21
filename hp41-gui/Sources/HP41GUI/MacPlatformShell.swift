#if os(macOS)
import AppKit
import Foundation

enum MacLaunchMode: String, Equatable {
    case menuBar = "menu-bar"
    case window

    init(storedValue: String) {
        self = MacLaunchMode(rawValue: storedValue) ?? .menuBar
    }
}

struct MacPlatformShellConfiguration: Equatable {
    let launchMode: MacLaunchMode
    let onboardingDone: Bool

    var usesStatusItem: Bool { launchMode == .menuBar }
    var showsWindowAtLaunch: Bool { launchMode == .window || !onboardingDone }
    var hidesWindowWhenInactive: Bool { launchMode == .menuBar && onboardingDone }
}

enum MacPlatformShell {
    static let toggleWindow = Notification.Name("HP41ToggleCalculatorWindow")
}

@MainActor
final class MacApplicationDelegate: NSObject, NSApplicationDelegate {
    private var configuration = MacPlatformShellConfiguration(launchMode: .window, onboardingDone: true)
    private var statusItem: NSStatusItem?
    private var toggleObserver: NSObjectProtocol?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let preferences = AppPreferences()
        configuration = MacPlatformShellConfiguration(
            launchMode: MacLaunchMode(storedValue: preferences.macosLaunchMode),
            onboardingDone: preferences.onboardingDone
        )
        toggleObserver = NotificationCenter.default.addObserver(
            forName: MacPlatformShell.toggleWindow,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor in self?.toggleCalculatorWindow() }
        }

        if configuration.usesStatusItem {
            NSApplication.shared.setActivationPolicy(.accessory)
            installStatusItem()
        } else {
            NSApplication.shared.setActivationPolicy(.regular)
        }
        GlobalShortcutController.shared.start(with: preferences.globalShortcut)

        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            if self.configuration.showsWindowAtLaunch { self.showCalculatorWindow() }
            else { self.calculatorWindow()?.orderOut(nil) }
        }
    }

    func applicationDidResignActive(_ notification: Notification) {
        guard configuration.hidesWindowWhenInactive,
              NSApplication.shared.modalWindow == nil else { return }
        calculatorWindow()?.orderOut(nil)
    }

    func applicationWillTerminate(_ notification: Notification) {
        GlobalShortcutController.shared.stop()
        if let toggleObserver { NotificationCenter.default.removeObserver(toggleObserver) }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        configuration.launchMode == .window
    }

    private func installStatusItem() {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        item.button?.title = "41"
        item.button?.toolTip = "HP-41 Calculator"
        item.button?.setAccessibilityLabel("HP-41 Calculator")

        let menu = NSMenu()
        let toggle = NSMenuItem(
            title: "Show/Hide Calculator",
            action: #selector(toggleCalculatorWindow),
            keyEquivalent: ""
        )
        toggle.target = self
        menu.addItem(toggle)
        menu.addItem(.separator())
        let quit = NSMenuItem(
            title: "Quit HP-41 Calculator",
            action: #selector(NSApplication.terminate(_:)),
            keyEquivalent: "q"
        )
        quit.target = NSApplication.shared
        menu.addItem(quit)
        item.menu = menu
        statusItem = item
    }

    @objc private func toggleCalculatorWindow() {
        guard let window = calculatorWindow() else { return }
        if window.isVisible && NSApplication.shared.isActive {
            if configuration.launchMode == .window {
                window.miniaturize(nil)
                // Keep the regular app active so its global/menu shortcut can
                // restore the last window even when no window is on screen.
                NSApplication.shared.activate(ignoringOtherApps: true)
            } else { window.orderOut(nil) }
        }
        else { showCalculatorWindow() }
    }

    private func showCalculatorWindow() {
        guard let window = calculatorWindow() else { return }
        position(window)
        NSApplication.shared.activate(ignoringOtherApps: true)
        if window.isMiniaturized { window.deminiaturize(nil) }
        window.makeKeyAndOrderFront(nil)
    }

    private func calculatorWindow() -> NSWindow? {
        NSApplication.shared.windows.first { window in
            window.canBecomeMain && !(window is NSPanel)
        }
    }

    private func position(_ window: NSWindow) {
        guard configuration.usesStatusItem,
              let screen = statusItem?.button?.window?.screen ?? NSScreen.main else { return }
        let visible = screen.visibleFrame
        window.setFrameTopLeftPoint(NSPoint(x: visible.maxX - window.frame.width - 12, y: visible.maxY - 8))
    }
}
#endif
