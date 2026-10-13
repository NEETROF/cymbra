import XCTest
@testable import LinguaSignIn

final class ReviewRelayStoreTests: XCTestCase {
    private var directory: URL!
    private var clock = Date(timeIntervalSince1970: 1_000)

    override func setUp() {
        super.setUp()
        directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("lingua-review-tests-\(UUID().uuidString)", isDirectory: true)
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: directory)
        super.tearDown()
    }

    private func store() -> ReviewRelayStore {
        ReviewRelayStore(directory: directory, now: { [unowned self] in self.clock })
    }

    func testNoStateBeforeTheExtensionSharesOne() {
        XCTAssertNil(store().state())
    }

    func testTheAppReadsTheStateTheExtensionSaved() {
        store().saveState("{\"v\":1}")
        XCTAssertEqual(store().state(), "{\"v\":1}")
        store().saveState("{\"v\":1,\"newer\":true}")
        XCTAssertEqual(store().state(), "{\"v\":1,\"newer\":true}")
    }

    func testTheExtensionReadsTheOpsTheAppWroteThenClearsThem() {
        let stamp = store().writeOps("ops-1")
        XCTAssertEqual(stamp, 1_000_000)
        let pending = store().pendingOps()
        XCTAssertEqual(pending?.ops, "ops-1")
        XCTAssertEqual(pending?.writtenAt, stamp)
        store().clearOps(writtenAt: stamp)
        XCTAssertNil(store().pendingOps())
    }

    func testAClearOfOlderOpsKeepsTheOpsWrittenSince() {
        let first = store().writeOps("ops-1")
        let second = store().writeOps("ops-2") // the same millisecond: stamped later all the same
        XCTAssertGreaterThan(second, first)
        store().clearOps(writtenAt: first)
        XCTAssertEqual(store().pendingOps()?.ops, "ops-2")
    }

    func testUnreadableOpsReadAsNoneAndAreDropped() throws {
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try Data("not json".utf8).write(to: directory.appendingPathComponent(ReviewRelayStore.opsFile))
        XCTAssertNil(store().pendingOps())
        store().clearOps(writtenAt: 0)
        XCTAssertFalse(FileManager.default.fileExists(atPath: directory.appendingPathComponent(ReviewRelayStore.opsFile).path))
        store().clearOps(writtenAt: 0) // nothing there: nothing to do
    }

    // MARK: The extension's native messages

    func testTheHandlerKeepsTheStateAndHandsTheOpsBackOnce() {
        let review = store()
        XCTAssertTrue(NativeMessage.reply(to: ["type": "review.saveState", "state": "S"], handoff: nil, review: review).isEmpty)
        XCTAssertEqual(review.state(), "S")
        XCTAssertTrue(NativeMessage.reply(to: ["type": "review.readOps"], handoff: nil, review: review).isEmpty)

        let stamp = review.writeOps("O")
        let read = NativeMessage.reply(to: ["type": "review.readOps"], handoff: nil, review: review)
        XCTAssertEqual(read["ops"] as? String, "O")
        XCTAssertEqual(read["writtenAt"] as? Int64, stamp)
        // As Safari hands a JavaScript number over: a Double in an NSNumber.
        let cleared = NativeMessage.reply(
            to: ["type": "review.clearOps", "writtenAt": NSNumber(value: Double(stamp))], handoff: nil, review: review
        )
        XCTAssertTrue(cleared.isEmpty)
        XCTAssertNil(review.pendingOps())
    }

    func testTheHandlerRefusesAStatelessSaveAndIgnoresAClearWithoutAStamp() {
        let review = store()
        XCTAssertEqual(
            NativeMessage.reply(to: ["type": "review.saveState"], handoff: nil, review: review) as? [String: String],
            ["error": "badState"]
        )
        review.writeOps("O")
        XCTAssertTrue(NativeMessage.reply(to: ["type": "review.clearOps"], handoff: nil, review: review).isEmpty)
        XCTAssertEqual(review.pendingOps()?.ops, "O")
    }

    func testTheHandlerAnswersWithoutAnAppGroup() {
        XCTAssertTrue(NativeMessage.reply(to: ["type": "review.saveState", "state": "S"], handoff: nil).isEmpty)
        XCTAssertTrue(NativeMessage.reply(to: ["type": "review.readOps"], handoff: nil).isEmpty)
    }

    // MARK: The app's page

    func testThePageReadsTheStateAndWritesItsOps() {
        let review = store()
        XCTAssertTrue(AppBridgeMessage.reply(to: ["type": "review.state"], store: review).isEmpty)
        review.saveState("S")
        XCTAssertEqual(AppBridgeMessage.reply(to: ["type": "review.state"], store: review)["state"] as? String, "S")

        XCTAssertTrue(AppBridgeMessage.reply(to: ["type": "review.pendingOps"], store: review).isEmpty)
        let written = AppBridgeMessage.reply(to: ["type": "review.writeOps", "ops": "O"], store: review)
        XCTAssertEqual(written["writtenAt"] as? Int64, review.pendingOps()?.writtenAt)
        let pending = AppBridgeMessage.reply(to: ["type": "review.pendingOps"], store: review)
        XCTAssertEqual(pending["ops"] as? String, "O")
    }

    func testThePageIsRefusedWhatIsNotItsProtocol() {
        let review = store()
        XCTAssertEqual(
            AppBridgeMessage.reply(to: ["type": "review.writeOps"], store: review) as? [String: String],
            ["error": "badOps"]
        )
        XCTAssertEqual(
            AppBridgeMessage.reply(to: ["type": "review.writeOps", "ops": "O"], store: nil) as? [String: String],
            ["error": "badOps"]
        )
        for message: Any? in [["type": "auth.takeIdToken"], "review.state", nil] {
            XCTAssertEqual(
                AppBridgeMessage.reply(to: message, store: review) as? [String: String],
                ["error": "unknownMessage"]
            )
        }
        XCTAssertTrue(AppBridgeMessage.reply(to: ["type": "review.state"], store: nil).isEmpty)
    }
}
