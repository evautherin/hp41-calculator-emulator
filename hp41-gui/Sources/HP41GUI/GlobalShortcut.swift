#if os(macOS)
import Carbon
import Foundation

enum GlobalShortcutRegistrationError: LocalizedError, Equatable {
    case invalidAccelerator(String)
    case registrationFailed(OSStatus)

    var errorDescription: String? {
        switch self {
        case .invalidAccelerator(let value):
            "Invalid global shortcut: \(value)"
        case .registrationFailed(let status) where status == eventHotKeyExistsErr:
            "That shortcut is already used by another application."
        case .registrationFailed(let status):
            "The global shortcut could not be registered (error \(status))."
        }
    }
}

protocol GlobalShortcutRegistering: AnyObject {
    func register(_ accelerator: String, action: @escaping () -> Void) throws
    func unregister()
}

final class CarbonGlobalShortcutRegistrar: GlobalShortcutRegistering {
    private static let signature: OSType = 0x4850_3431 // "HP41"
    private static var nextID: UInt32 = 1
    private static var actions: [UInt32: () -> Void] = [:]
    private static var eventHandler: EventHandlerRef?

    private var reference: EventHotKeyRef?
    private var identifier: UInt32?

    func register(_ accelerator: String, action: @escaping () -> Void) throws {
        guard let descriptor = Self.descriptor(for: accelerator) else {
            throw GlobalShortcutRegistrationError.invalidAccelerator(accelerator)
        }
        unregister()
        Self.installHandlerIfNeeded()
        let id = Self.nextID
        Self.nextID &+= 1
        let hotKeyID = EventHotKeyID(signature: Self.signature, id: id)
        var hotKey: EventHotKeyRef?
        let status = RegisterEventHotKey(
            descriptor.keyCode,
            descriptor.modifiers,
            hotKeyID,
            GetApplicationEventTarget(),
            0,
            &hotKey
        )
        guard status == noErr, let hotKey else {
            throw GlobalShortcutRegistrationError.registrationFailed(status)
        }
        reference = hotKey
        identifier = id
        Self.actions[id] = action
    }

    func unregister() {
        if let reference { UnregisterEventHotKey(reference) }
        if let identifier { Self.actions.removeValue(forKey: identifier) }
        reference = nil
        identifier = nil
    }

    deinit { unregister() }

    private static func installHandlerIfNeeded() {
        guard eventHandler == nil else { return }
        var specification = EventTypeSpec(
            eventClass: OSType(kEventClassKeyboard),
            eventKind: UInt32(kEventHotKeyPressed)
        )
        InstallEventHandler(
            GetApplicationEventTarget(),
            { _, event, _ in
                guard let event else { return OSStatus(eventNotHandledErr) }
                var hotKeyID = EventHotKeyID()
                let status = GetEventParameter(
                    event,
                    EventParamName(kEventParamDirectObject),
                    EventParamType(typeEventHotKeyID),
                    nil,
                    MemoryLayout<EventHotKeyID>.size,
                    nil,
                    &hotKeyID
                )
                guard status == noErr,
                      hotKeyID.signature == CarbonGlobalShortcutRegistrar.signature,
                      let action = CarbonGlobalShortcutRegistrar.actions[hotKeyID.id] else {
                    return status
                }
                DispatchQueue.main.async(execute: action)
                return noErr
            },
            1,
            &specification,
            nil,
            &eventHandler
        )
    }

    private static func descriptor(for accelerator: String) -> (keyCode: UInt32, modifiers: UInt32)? {
        guard ShortcutAccelerator.isValid(accelerator) else { return nil }
        let parts = accelerator.split(separator: "+").map(String.init)
        guard let key = parts.last, let keyCode = keyCodes[key.uppercased()] else { return nil }
        var modifiers: UInt32 = 0
        for modifier in parts.dropLast() {
            switch modifier {
            case "Control": modifiers |= UInt32(controlKey)
            case "Alt": modifiers |= UInt32(optionKey)
            case "Shift": modifiers |= UInt32(shiftKey)
            case "Command": modifiers |= UInt32(cmdKey)
            default: return nil
            }
        }
        return (keyCode, modifiers)
    }

    private static let keyCodes: [String: UInt32] = [
        "A": 0, "B": 11, "C": 8, "D": 2, "E": 14, "F": 3, "G": 5,
        "H": 4, "I": 34, "J": 38, "K": 40, "L": 37, "M": 46, "N": 45,
        "O": 31, "P": 35, "Q": 12, "R": 15, "S": 1, "T": 17, "U": 32,
        "V": 9, "W": 13, "X": 7, "Y": 16, "Z": 6,
        "0": 29, "1": 18, "2": 19, "3": 20, "4": 21,
        "5": 23, "6": 22, "7": 26, "8": 28, "9": 25,
        "SPACE": 49,
        "F1": 122, "F2": 120, "F3": 99, "F4": 118, "F5": 96, "F6": 97,
        "F7": 98, "F8": 100, "F9": 101, "F10": 109, "F11": 103, "F12": 111,
    ]
}

@MainActor
final class GlobalShortcutController {
    static let shared = GlobalShortcutController(registrar: CarbonGlobalShortcutRegistrar())

    private let registrar: GlobalShortcutRegistering
    private(set) var accelerator: String?
    private(set) var errorMessage: String?

    init(registrar: GlobalShortcutRegistering) {
        self.registrar = registrar
    }

    func start(with accelerator: String) {
        do { try replace(with: accelerator) }
        catch { errorMessage = error.localizedDescription }
    }

    func replace(with newAccelerator: String) throws {
        let previous = accelerator
        registrar.unregister()
        do {
            try registrar.register(newAccelerator) {
                NotificationCenter.default.post(name: MacPlatformShell.toggleWindow, object: nil)
            }
            accelerator = newAccelerator
            errorMessage = nil
        } catch {
            accelerator = nil
            if let previous {
                try? registrar.register(previous) {
                    NotificationCenter.default.post(name: MacPlatformShell.toggleWindow, object: nil)
                }
                accelerator = previous
            }
            errorMessage = error.localizedDescription
            throw error
        }
    }

    func stop() {
        registrar.unregister()
        accelerator = nil
    }
}
#endif
