import Foundation

/// The extension's native messages (`browser.runtime.sendNativeMessage`), answered by
/// `SafariWebExtensionHandler`. Kept here so the protocol is unit-tested.
public enum NativeMessage {
    /// `auth.takeIdToken` → `{provider, idToken}` for a waiting token, `{}` when none waits;
    /// any other message → `{error: "unknownMessage"}`. Never carries anything else.
    public static func reply(to message: Any?, handoff: IdTokenHandoff?) -> [String: Any] {
        let type = (message as? [String: Any])?["type"] as? String
        switch type {
        case "auth.takeIdToken":
            guard let token = handoff?.take() else { return [:] }
            return ["provider": token.provider.rawValue, "idToken": token.idToken]
        default:
            return ["error": "unknownMessage"]
        }
    }
}
