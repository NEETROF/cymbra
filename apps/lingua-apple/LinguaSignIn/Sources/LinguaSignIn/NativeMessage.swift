import Foundation

/// The extension's native messages (`browser.runtime.sendNativeMessage`), answered by
/// `SafariWebExtensionHandler`. Kept here so the protocol is unit-tested.
public enum NativeMessage {
    /// - `auth.takeIdToken` → `{provider, idToken}` for a waiting token, `{}` when none waits;
    /// - `auth.providers` → `{apple: true, google}`, Google once the app has a Google client;
    /// - any other message → `{error: "unknownMessage"}`.
    public static func reply(to message: Any?, handoff: IdTokenHandoff?, googleClientId: String? = nil) -> [String: Any] {
        let type = (message as? [String: Any])?["type"] as? String
        switch type {
        case "auth.takeIdToken":
            guard let token = handoff?.take() else { return [:] }
            return ["provider": token.provider.rawValue, "idToken": token.idToken]
        case "auth.providers":
            return ["apple": true, "google": GoogleOAuth(clientId: googleClientId) != nil]
        default:
            return ["error": "unknownMessage"]
        }
    }
}
