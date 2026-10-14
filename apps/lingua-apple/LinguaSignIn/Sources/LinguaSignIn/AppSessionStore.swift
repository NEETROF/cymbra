import Foundation
import Security

/// Where the app keeps the refresh token of its own Cymbra session (add-lingua-review-translation-fallback,
/// D8): the session the app opens for « Traduction en ligne » in its review, a family of its own — never
/// the Safari extension's, which is never handed over. The access token stays in the page's memory;
/// only the refresh token, which outlives a launch, is kept here.
public protocol RefreshTokenStore: AnyObject {
    func read() -> String?
    /// Keep `token`, replacing the one kept; whether it was kept.
    @discardableResult func write(_ token: String) -> Bool
    func clear()
}

/// The app's Keychain: one generic password, readable only by the app, on this device only, once
/// the device has been unlocked after a restart. Not synced through iCloud Keychain: a session is a
/// device's.
public final class KeychainRefreshTokenStore: RefreshTokenStore {
    public static let defaultService = "com.cymbra.lingua.app-session"
    let service: String
    let account: String

    public init(service: String = KeychainRefreshTokenStore.defaultService, account: String = "refresh") {
        self.service = service
        self.account = account
    }

    private var query: [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }

    public func read() -> String? {
        var request = query
        request[kSecReturnData as String] = true
        request[kSecMatchLimit as String] = kSecMatchLimitOne
        var result: AnyObject?
        guard SecItemCopyMatching(request as CFDictionary, &result) == errSecSuccess,
              let data = result as? Data, let token = String(data: data, encoding: .utf8), !token.isEmpty
        else { return nil }
        return token
    }

    @discardableResult
    public func write(_ token: String) -> Bool {
        guard !token.isEmpty else { return false }
        let data = Data(token.utf8)
        let updated = SecItemUpdate(query as CFDictionary, [kSecValueData as String: data] as CFDictionary)
        if updated == errSecSuccess { return true }
        guard updated == errSecItemNotFound else { return false }
        var item = query
        item[kSecValueData as String] = data
        item[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        return SecItemAdd(item as CFDictionary, nil) == errSecSuccess
    }

    public func clear() {
        SecItemDelete(query as CFDictionary)
    }
}

/// The page's session messages (`auth.readRefresh`, `auth.writeRefresh`, `auth.clear`), answered by
/// `ViewController` alongside the relay's; `auth.signIn` presents the sheet there, being UI. Kept here
/// so the protocol is unit-tested; the page's side is `apps/lingua-extension/src/app/account.ts`.
public enum AppSessionMessage {
    /// Whether `message` is one of the session messages answered here.
    public static func handles(_ message: Any?) -> Bool {
        switch (message as? [String: Any])?["type"] as? String {
        case "auth.readRefresh", "auth.writeRefresh", "auth.clear": return true
        default: return false
        }
    }

    /// - `auth.readRefresh` → `{refresh}`, or `{}` when none is kept;
    /// - `auth.writeRefresh` `{refresh}` → `{}` once kept, `{error: "notKept"}` otherwise;
    /// - `auth.clear` → `{}`;
    /// - anything else, or no store → `{error: …}`.
    public static func reply(to message: Any?, store: RefreshTokenStore?) -> [String: Any] {
        let body = message as? [String: Any]
        guard let store else { return ["error": "noKeychain"] }
        switch body?["type"] as? String {
        case "auth.readRefresh":
            guard let token = store.read() else { return [:] }
            return ["refresh": token]
        case "auth.writeRefresh":
            guard let token = body?["refresh"] as? String, store.write(token) else { return ["error": "notKept"] }
            return [:]
        case "auth.clear":
            store.clear()
            return [:]
        default:
            return ["error": "unknownMessage"]
        }
    }

    /// What `auth.signIn` answers once the sheet closed: the provider and its id_token, or `{}` when
    /// the reader closed it or the provider failed — the page then sends nothing.
    public static func signInReply(_ provider: SignInProvider?, _ outcome: ProviderOutcome?) -> [String: Any] {
        guard let provider, case let .idToken(token)? = outcome, !token.isEmpty else { return [:] }
        return ["provider": provider.rawValue, "idToken": token]
    }
}
