import XCTest
@testable import LinguaSignIn

final class IdTokenHandoffTests: XCTestCase {
    private var suiteName = ""
    private var defaults: UserDefaults!
    private var clock = Date(timeIntervalSince1970: 1_000_000)

    override func setUp() {
        super.setUp()
        suiteName = "lingua-signin-tests-\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suiteName)
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suiteName)
        super.tearDown()
    }

    private func handoff() -> IdTokenHandoff {
        IdTokenHandoff(defaults: defaults, now: { [unowned self] in self.clock })
    }

    private let apple = HandedIdToken(provider: .apple, idToken: "apple-id-token")

    func testATokenIsReadOnceThenDeleted() {
        let handoff = handoff()
        handoff.hand(apple)
        XCTAssertEqual(handoff.take(), apple)
        XCTAssertNil(handoff.take())
        XCTAssertNil(defaults.data(forKey: IdTokenHandoff.key))
    }

    func testTheExtensionReadsWhatTheAppHanded() {
        handoff().hand(apple)
        XCTAssertEqual(IdTokenHandoff(defaults: defaults, now: { [unowned self] in self.clock }).take(), apple)
    }

    func testATokenStaysReadableForFiveMinutes() {
        let handoff = handoff()
        handoff.hand(apple)
        clock.addTimeInterval(IdTokenHandoff.maxAge)
        XCTAssertEqual(handoff.take(), apple)
    }

    func testAnOlderTokenIsDiscardedAndDeleted() {
        let handoff = handoff()
        handoff.hand(apple)
        clock.addTimeInterval(IdTokenHandoff.maxAge + 1)
        XCTAssertNil(handoff.take())
        XCTAssertNil(defaults.data(forKey: IdTokenHandoff.key))
    }

    func testATokenDatedInTheFutureIsDiscarded() {
        let handoff = handoff()
        handoff.hand(apple)
        clock.addTimeInterval(-60)
        XCTAssertNil(handoff.take())
    }

    func testANewerTokenReplacesAWaitingOne() {
        let handoff = handoff()
        handoff.hand(apple)
        let google = HandedIdToken(provider: .google, idToken: "google-id-token")
        handoff.hand(google)
        XCTAssertEqual(handoff.take(), google)
        XCTAssertNil(handoff.take())
    }

    func testNothingWaitingAndUnreadableDataReadAsNoToken() {
        XCTAssertNil(handoff().take())
        defaults.set(Data("not json".utf8), forKey: IdTokenHandoff.key)
        XCTAssertNil(handoff().take())
        XCTAssertNil(defaults.data(forKey: IdTokenHandoff.key))
    }

    func testReplyHandsTheTokenOverOnce() {
        let handoff = handoff()
        handoff.hand(apple)
        let message: [String: Any] = ["type": "auth.takeIdToken"]
        XCTAssertEqual(
            NativeMessage.reply(to: message, handoff: handoff) as? [String: String],
            ["provider": "apple", "idToken": "apple-id-token"]
        )
        XCTAssertTrue(NativeMessage.reply(to: message, handoff: handoff).isEmpty)
    }

    func testReplyIsEmptyWithoutAnAppGroup() {
        XCTAssertTrue(NativeMessage.reply(to: ["type": "auth.takeIdToken"], handoff: nil).isEmpty)
    }

    func testReplyRefusesUnknownMessagesWithoutReadingTheToken() {
        let handoff = handoff()
        handoff.hand(apple)
        for message: Any? in [["type": "auth.other"], ["echo": "x"], "auth.takeIdToken", nil] {
            XCTAssertEqual(
                NativeMessage.reply(to: message, handoff: handoff) as? [String: String],
                ["error": "unknownMessage"]
            )
        }
        XCTAssertEqual(handoff.take(), apple)
    }
}
