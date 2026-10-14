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

    func testTheFifthSessionInTheAppAsksStoreKit() {
        let policy = ReviewRequestPolicy(defaults: defaults, version: "1.6.0")
        let asked = (1...5).map { _ in policy.sessionFinished() }
        XCTAssertEqual(asked, [false, false, false, false, true])
        XCTAssertEqual(policy.finishedSessions, 5)
    }

    func testOncePerVersion() {
        let policy = ReviewRequestPolicy(defaults: defaults, version: "1.6.0")
        for _ in 1...5 { _ = policy.sessionFinished() }
        XCTAssertFalse((1...20).contains { _ in policy.sessionFinished() }, "the same version asks once")
        // A new version of the app: the next finished session asks again.
        let updated = ReviewRequestPolicy(defaults: defaults, version: "1.7.0")
        XCTAssertTrue(updated.sessionFinished())
        XCTAssertFalse(updated.sessionFinished())
    }

    func testTheCountSurvivesTheApp() {
        for _ in 1...4 { _ = ReviewRequestPolicy(defaults: defaults, version: "1.6.0").sessionFinished() }
        XCTAssertTrue(ReviewRequestPolicy(defaults: defaults, version: "1.6.0").sessionFinished())
    }

    func testWithoutAVersionTheAppNeverAsks() {
        for version in [nil, ""] as [String?] {
            let policy = ReviewRequestPolicy(defaults: defaults, version: version)
            XCTAssertFalse((1...10).contains { _ in policy.sessionFinished() })
        }
    }

    func testThePageSaysASessionFinishedAndIsAnswered() {
        var told = 0
        let reply = AppBridgeMessage.reply(to: ["type": "review.sessionFinished"], store: nil) { told += 1 }
        XCTAssertTrue(reply.isEmpty)
        XCTAssertEqual(told, 1)
        // Without a listener it is still the protocol's, never an unknown message.
        XCTAssertTrue(AppBridgeMessage.reply(to: ["type": "review.sessionFinished"], store: nil).isEmpty)
        // Other messages tell nothing.
        _ = AppBridgeMessage.reply(to: ["type": "review.state"], store: nil) { told += 1 }
        XCTAssertEqual(told, 1)
    }
}
