import Foundation
#if os(macOS)
import AppKit
#endif

enum ShortcutAccelerator {
    static let defaultValue = "Control+Alt+Command+H"
    private static let modifiers = ["Control", "Alt", "Shift", "Command"]

    static func capture(key: String, control: Bool, alt: Bool,
                        shift: Bool, command: Bool) -> String? {
        guard let token = keyToken(key), control || alt || shift || command else { return nil }
        var parts: [String] = []
        if control { parts.append("Control") }
        if alt { parts.append("Alt") }
        if shift { parts.append("Shift") }
        if command { parts.append("Command") }
        parts.append(token)
        return parts.joined(separator: "+")
    }

    static func isValid(_ value: String) -> Bool {
        let parts = value.split(separator: "+").map(String.init)
        guard parts.count >= 2, let key = parts.last, keyToken(key) != nil else { return false }
        let mods = Array(parts.dropLast())
        return !mods.isEmpty && Set(mods).count == mods.count && mods.allSatisfy(modifiers.contains)
    }

    static func formatted(_ value: String) -> String {
        let symbols = ["Control": "⌃", "Alt": "⌥", "Shift": "⇧", "Command": "⌘"]
        return value.split(separator: "+").map { symbols[String($0)] ?? String($0) }.joined()
    }

    private static func keyToken(_ raw: String) -> String? {
        if raw == " " || raw.caseInsensitiveCompare("Space") == .orderedSame { return "Space" }
        let upper = raw.uppercased()
        if upper.count == 1, upper.first?.isLetter == true || upper.first?.isNumber == true { return upper }
        if let number = Int(upper.dropFirst()), upper.hasPrefix("F"), (1...12).contains(number) { return "F\(number)" }
        return nil
    }
}

enum NativeApplication {
    static func restart() {
        #if os(macOS)
        let configuration = NSWorkspace.OpenConfiguration()
        NSWorkspace.shared.openApplication(at: Bundle.main.bundleURL, configuration: configuration) { _, error in
            if error == nil { NSApplication.shared.terminate(nil) }
        }
        #endif
    }
}
