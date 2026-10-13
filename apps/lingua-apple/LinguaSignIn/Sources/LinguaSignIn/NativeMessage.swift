import Foundation

/// The extension's native messages (`browser.runtime.sendNativeMessage`), answered by
/// `SafariWebExtensionHandler`. Kept here so the protocol is unit-tested.
public enum NativeMessage {
    /// - `auth.takeIdToken` → `{provider, idToken}` for a waiting token, `{}` when none waits;
    /// - `auth.providers` → `{apple: true, google}`, Google once the app has a Google client;
    /// - `interface.language` `{language}` → `{language}`, the extension's interface language kept
    ///   in `languages` for the activation page (localise-lingua-apple-host D2); one the app does
    ///   not speak → `{error: "unknownLanguage"}`, nothing kept;
    /// - `review.saveState` `{state}` → `{}`, the extension's state kept in `review` for the app's
    ///   review (add-lingua-apple-review D3); a message without one → `{error: "badState"}`;
    /// - `review.readOps` → `{ops, writtenAt}`, the changes a review in the app made, or `{}`;
    /// - `review.clearOps` `{writtenAt}` → `{}`, those changes dropped once the extension saved them;
    /// - any other message → `{error: "unknownMessage"}`.
    public static func reply(
        to message: Any?,
        handoff: IdTokenHandoff?,
        googleClientId: String? = nil,
        languages: InterfaceLanguageStore? = nil,
        review: ReviewRelayStore? = nil
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
        case "review.saveState":
            guard let state = body?["state"] as? String, !state.isEmpty else { return ["error": "badState"] }
            review?.saveState(state)
            return [:]
        case "review.readOps":
            guard let pending = review?.pendingOps() else { return [:] }
            return ["ops": pending.ops, "writtenAt": pending.writtenAt]
        case "review.clearOps":
            // A JavaScript number arrives as an NSNumber: read it whole, whatever its boxing.
            guard let writtenAt = (body?["writtenAt"] as? NSNumber)?.int64Value else { return [:] }
            review?.clearOps(writtenAt: writtenAt)
            return [:]
        default:
            return ["error": "unknownMessage"]
        }
    }
}
