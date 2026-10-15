import XCTest
@testable import LinguaSignIn

/// The app asks for an App Store rating only through StoreKit (add-lingua-comeback-hints D11): from
/// the fifth session finished in the app, at most once per version, counted in the app's own
/// defaults.
final class ReviewRequestPolicyTests: XCTestCase {
    private var defaults: UserDefaults!
    private var suite: String!

    override func setUp() {
        super.setUp()
        suite = "ReviewRequestPolicyTests.\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suite)
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suite)
        super.tearDown()
    }

    /// Finish a session and, when StoreKit is due, make the request at once (a window on screen).
    private func finishAndAsk(_ policy: ReviewRequestPolicy) -> Bool {
        guard let ticket = policy.sessionFinished(), policy.stillDue(ticket) else { return false }
        policy.asked()
        return true
    }

    func testTheFifthSessionInTheAppAsksStoreKit() {
        let policy = ReviewRequestPolicy(defaults: defaults, version: "1.6.0")
        let asked = (1...5).map { _ in finishAndAsk(policy) }
        XCTAssertEqual(asked, [false, false, false, false, true])
        XCTAssertEqual(policy.finishedSessions, 5)
    }

    func testOncePerVersion() {
        let policy = ReviewRequestPolicy(defaults: defaults, version: "1.6.0")
        for _ in 1...5 { _ = finishAndAsk(policy) }
        XCTAssertFalse((1...20).contains { _ in finishAndAsk(policy) }, "the same version asks once")
        // A new version of the app: the next finished session asks again.
        let updated = ReviewRequestPolicy(defaults: defaults, version: "1.7.0")
        XCTAssertTrue(finishAndAsk(updated))
        XCTAssertFalse(finishAndAsk(updated))
    }

    func testTheCountSurvivesTheApp() {
        for _ in 1...4 { _ = finishAndAsk(ReviewRequestPolicy(defaults: defaults, version: "1.6.0")) }
        XCTAssertTrue(finishAndAsk(ReviewRequestPolicy(defaults: defaults, version: "1.6.0")))
    }

    func testWithoutAVersionTheAppNeverAsks() {
        for version in [nil, ""] as [String?] {
            let policy = ReviewRequestPolicy(defaults: defaults, version: version)
            XCTAssertFalse((1...10).contains { _ in policy.sessionFinished() != nil })
            policy.asked()
            XCTAssertNil(defaults.string(forKey: ReviewRequestPolicy.askedVersionKey))
        }
    }

    func testARequestNeverMadeIsNotCountedAsAsked() {
        // The fifth end is due, but no window was on screen when the two seconds ran out: the
        // version has not asked, so the next end asks.
        let policy = ReviewRequestPolicy(defaults: defaults, version: "1.6.0")
        for _ in 1...4 { _ = policy.sessionFinished() }
        XCTAssertNotNil(policy.sessionFinished())
        XCTAssertNil(defaults.string(forKey: ReviewRequestPolicy.askedVersionKey))
        let sixth = policy.sessionFinished()
        XCTAssertNotNil(sixth)
        XCTAssertTrue(policy.stillDue(sixth!))
        policy.asked()
        XCTAssertFalse(policy.stillDue(sixth!))
        XCTAssertNil(policy.sessionFinished(), "asked: the same version asks no more")
    }

    func testASessionStartedWithinTheDelayDropsTheRequest() {
        let policy = ReviewRequestPolicy(defaults: defaults, version: "1.6.0")
        for _ in 1...4 { _ = policy.sessionFinished() }
        let fifth = policy.sessionFinished()!
        policy.sessionStarted() // « Encore 10 » within the two seconds
        XCTAssertFalse(policy.stillDue(fifth))
        // That session's end asks, two seconds after it; the dropped request was never made.
        let sixth = policy.sessionFinished()!
        XCTAssertFalse(policy.stillDue(fifth), "an older end's request stays dropped")
        XCTAssertTrue(policy.stillDue(sixth))
    }

    func testThePageSaysASessionFinishedAndIsAnswered() {
        var told = 0
        let reply = AppBridgeMessage.reply(to: ["type": "review.sessionFinished"], store: nil, onSessionFinished: { told += 1 })
        XCTAssertTrue(reply.isEmpty)
        XCTAssertEqual(told, 1)
        // Without a listener it is still the protocol's, never an unknown message.
        XCTAssertTrue(AppBridgeMessage.reply(to: ["type": "review.sessionFinished"], store: nil).isEmpty)
        // Other messages tell nothing.
        _ = AppBridgeMessage.reply(to: ["type": "review.state"], store: nil, onSessionFinished: { told += 1 })
        XCTAssertEqual(told, 1)
    }

    func testThePageSaysASessionStartedAndIsAnswered() {
        var started = 0
        var finished = 0
        let reply = AppBridgeMessage.reply(
            to: ["type": "review.sessionStarted"],
            store: nil,
            onSessionFinished: { finished += 1 },
            onSessionStarted: { started += 1 }
        )
        XCTAssertTrue(reply.isEmpty)
        XCTAssertEqual([started, finished], [1, 0])
        XCTAssertTrue(AppBridgeMessage.reply(to: ["type": "review.sessionStarted"], store: nil).isEmpty)
    }
}
