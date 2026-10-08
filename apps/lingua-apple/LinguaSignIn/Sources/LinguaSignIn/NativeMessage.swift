import Foundation

/// The extension's native messages (`browser.runtime.sendNativeMessage`), answered by
/// `SafariWebExtensionHandler`. Kept here so the protocol is unit-tested.
public enum NativeMessage {
    /// - `auth.takeIdToken` → `{provider, idToken}` for a waiting token, `{}` when none waits;
    /// - `auth.providers` → `{apple: true, google}`, Google once the app has a Google client;
    /// - `interface.language` `{language}` → `{language}`, the extension's interface language kept
    ///   in `languages` for the activation page (localise-lingua-apple-host D2); one the app does
    ///   not speak → `{error: "unknownLanguage"}`, nothing kept;
    /// - any other message → `{error: "unknownMessage"}`.
    public static func reply(
        to message: Any?,
        handoff: IdTokenHandoff?,
        googleClientId: String? = nil,
        languages: InterfaceLanguageStore? = nil
    ) -> [String: Any] {
        let body = message as? [String: Any]
        switch body?["type"] as? String {
        case "auth.takeIdToken":
            guard let token = handoff?.take() else { return [:] }
            return ["provider": token.provider.rawValue, "idToken": token.idToken]
        case "auth.providers":
            return ["apple": true, "google": GoogleOAuth(clientId: googleClientId) != nil]
        case "interface.language":
            guard let raw = body?["language"] as? String, let language = SignInLanguage(rawValue: raw) else {
                return ["error": "unknownLanguage"]
            }
            languages?.remember(language)
            return ["language": language.rawValue]
        default:
            return ["error": "unknownMessage"]
        }
    }
}
