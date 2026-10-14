import Foundation

/// When the app asks the App Store for a rating (add-lingua-comeback-hints D11): only through the
/// system's own request — StoreKit — never a prompt, card or dialog of the app's. The page says each
/// « Séance terminée » (`review.sessionFinished`); from the fifth session finished in the app, the
/// app asks, at most once per version of the app, and the system decides whether its prompt shows
/// (three times a year at most, never when the reader turned review requests off).
///
/// The count is the app's own, in its `UserDefaults` — not the App Group the extension shares — and
/// nothing about it leaves the device.
public final class ReviewRequestPolicy {
    /// The finished sessions before the app first asks (Apple's sample waits for four).
    public static let firstSession = 5

    static let sessionsKey = "reviewRequest.finishedSessions"
    static let askedVersionKey = "reviewRequest.askedVersion"

    private let defaults: UserDefaults
    private let version: String?

    /// `version` is the app's `CFBundleShortVersionString`; without one the app never asks.
    public init(
        defaults: UserDefaults = .standard,
        version: String? = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
    ) {
        self.defaults = defaults
        self.version = version
    }

    /// The sessions finished in the app so far.
    public var finishedSessions: Int {
        defaults.integer(forKey: Self.sessionsKey)
    }

    /// A session finished: counted; true when StoreKit is to be asked now — the fifth session or a
    /// later one, in a version of the app that has not asked yet.
    public func sessionFinished() -> Bool {
        let sessions = finishedSessions + 1
        defaults.set(sessions, forKey: Self.sessionsKey)
        guard sessions >= Self.firstSession, let version, !version.isEmpty,
              defaults.string(forKey: Self.askedVersionKey) != version else { return false }
        defaults.set(version, forKey: Self.askedVersionKey)
        return true
    }
}
