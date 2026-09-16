import Foundation

/// The provider whose sheet produced an id_token. Raw values match the extension's `Provider`.
public enum SignInProvider: String, Codable, Sendable {
    case apple
    case google
}

/// An id_token handed from the host app to its Safari extension.
public struct HandedIdToken: Equatable, Sendable {
    public let provider: SignInProvider
    public let idToken: String

    public init(provider: SignInProvider, idToken: String) {
        self.provider = provider
        self.idToken = idToken
    }
}

/// The hand-off of an id_token from the host app to its Safari extension
/// (add-lingua-connected-clients, design D6). Safari has no `identity.launchWebAuthFlow`, so
/// the app runs the Apple or Google sheet and leaves the id_token in the App Group; the
/// extension's native handler takes it. A token is readable once and for at most five
/// minutes (an Apple id_token lives ten): whatever is read, or found expired, is deleted.
public final class IdTokenHandoff {
    /// Shared by the app and the extension (App Group entitlement on both). A sandboxed macOS
    /// app reaches, on every supported macOS, the groups prefixed with its team id; iOS uses
    /// the registered `group.` id.
    #if os(macOS)
    public static let appGroup = "VMFJ6KRW77.com.cymbra.lingua"
    #else
    public static let appGroup = "group.com.cymbra.lingua"
    #endif
    /// How long a handed token stays readable.
    public static let maxAge: TimeInterval = 5 * 60

    static let key = "lingua.handedIdToken"

    private let defaults: UserDefaults
    private let now: () -> Date

    public init(defaults: UserDefaults, now: @escaping () -> Date = { Date() }) {
        self.defaults = defaults
        self.now = now
    }

    /// The App Group's hand-off, or nil when the App Group is not available to this process.
    public static func shared() -> IdTokenHandoff? {
        UserDefaults(suiteName: appGroup).map { IdTokenHandoff(defaults: $0) }
    }

    /// Hand a token over, replacing any earlier one still waiting.
    public func hand(_ token: HandedIdToken) {
        let stored = Stored(provider: token.provider, idToken: token.idToken, handedAt: now().timeIntervalSince1970)
        defaults.set(try? JSONEncoder().encode(stored), forKey: Self.key)
    }

    /// The waiting token, deleted as it is read. Nil when none waits, or when the one waiting
    /// is older than `maxAge`, dated in the future, or unreadable — deleted all the same.
    public func take() -> HandedIdToken? {
        guard let data = defaults.data(forKey: Self.key) else { return nil }
        defaults.removeObject(forKey: Self.key)
        guard let stored = try? JSONDecoder().decode(Stored.self, from: data), !stored.idToken.isEmpty else {
            return nil
        }
        let age = now().timeIntervalSince1970 - stored.handedAt
        guard age >= 0, age <= Self.maxAge else { return nil }
        return HandedIdToken(provider: stored.provider, idToken: stored.idToken)
    }

    private struct Stored: Codable {
        let provider: SignInProvider
        let idToken: String
        let handedAt: TimeInterval
    }
}
