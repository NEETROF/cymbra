import Foundation

/// When the app asks the App Store for a rating (add-lingua-comeback-hints D11): only through the
/// system's own request — StoreKit — never a prompt, card or dialog of the app's. The page says each
/// « Séance terminée » (`review.sessionFinished`); from the fifth session finished in the app, the
/// app asks, at most once per version of the app, and the system decides whether its prompt shows
/// (three times a year at most, never when the reader turned review requests off).
///
/// A version counts as having asked only once the request was actually made (`asked()`): a request
/// dropped because the reader started another session within the two seconds, or because no window
/// was on screen, is made at the next finished session instead.
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
    /// Moved by every session's start and end: a request an end scheduled is still to be made only
    /// while nothing moved it since.
    private var generation = 0

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

    /// Whether this version of the app may still ask: it has one, and has not asked yet.
    private var versionMayAsk: Bool {
        guard let version, !version.isEmpty else { return false }
        return defaults.string(forKey: Self.askedVersionKey) != version
    }

    /// A session finished: counted. A ticket when StoreKit is due — the fifth session or a later
    /// one, in a version of the app that has not asked yet — to check with `stillDue` when the
    /// request is about to be made. Nothing is recorded as asked here: `asked()` does, once the
    /// request is actually made.
    public func sessionFinished() -> Int? {
        let sessions = finishedSessions + 1
        defaults.set(sessions, forKey: Self.sessionsKey)
        generation += 1
        guard sessions >= Self.firstSession, versionMayAsk else { return nil }
        return generation
    }

    /// A session started: a request the last end scheduled is not made — the reader is reviewing.
    public func sessionStarted() {
        generation += 1
    }

    /// Whether the request `ticket` was given for is still to be made: no session started or ended
    /// since, and this version of the app has not asked.
    public func stillDue(_ ticket: Int) -> Bool {
        ticket == generation && versionMayAsk
    }

    /// StoreKit was asked, with a window on screen: this version of the app does not ask again.
    public func asked() {
        guard let version, !version.isEmpty else { return }
        defaults.set(version, forKey: Self.askedVersionKey)
    }
}
