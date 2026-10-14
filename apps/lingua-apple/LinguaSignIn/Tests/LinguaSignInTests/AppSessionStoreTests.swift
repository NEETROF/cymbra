import Security
import XCTest
@testable import LinguaSignIn

/// An in-memory refresh-token store: the protocol's behaviour without a Keychain, so the bridge's
/// replies are tested apart from the Keychain itself (a special case: a fake that behaves across calls).
private final class MemoryStore: RefreshTokenStore {
    var token: String?
    var refuses = false
    func read() -> String? { token }
    func write(_ token: String) -> Bool {
        guard !refuses else { return false }
        self.token = token
        return true
    }
    func clear() { token = nil }
}

final class AppSessionStoreTests: XCTestCase {
    func testTheSessionMessagesReadWriteAndClearTheRefreshToken() {
        let store = MemoryStore()
        XCTAssertEqual(AppSessionMessage.reply(to: ["type": "auth.readRefresh"], store: store) as NSDictionary, [:])
        XCTAssertEqual(AppSessionMessage.reply(to: ["type": "auth.writeRefresh", "refresh": "r1"], store: store) as NSDictionary, [:])
        XCTAssertEqual(AppSessionMessage.reply(to: ["type": "auth.readRefresh"], store: store) as NSDictionary, ["refresh": "r1"])
        XCTAssertEqual(AppSessionMessage.reply(to: ["type": "auth.clear"], store: store) as NSDictionary, [:])
        XCTAssertNil(store.token)
    }

    func testARefusedOrMalformedMessageIsAnError() {
        let store = MemoryStore()
        let cases: [(Any?, String)] = [
            (["type": "auth.writeRefresh"], "notKept"),
            (["type": "auth.writeRefresh", "refresh": 3], "notKept"),
            (["type": "auth.other"], "unknownMessage"),
            ("auth.clear", "unknownMessage"),
        ]
        for (message, error) in cases {
            XCTAssertEqual(AppSessionMessage.reply(to: message, store: store)["error"] as? String, error, "\(String(describing: message))")
        }
        store.refuses = true
        XCTAssertEqual(AppSessionMessage.reply(to: ["type": "auth.writeRefresh", "refresh": "r"], store: store)["error"] as? String, "notKept")
        XCTAssertEqual(AppSessionMessage.reply(to: ["type": "auth.clear"], store: nil)["error"] as? String, "noKeychain")
    }

    func testOnlyTheSessionMessagesAreHandledHere() {
        for type in ["auth.readRefresh", "auth.writeRefresh", "auth.clear"] {
            XCTAssertTrue(AppSessionMessage.handles(["type": type]), type)
        }
        for message in [["type": "auth.signIn"], ["type": "review.state"], [:]] as [Any] {
            XCTAssertFalse(AppSessionMessage.handles(message))
        }
        XCTAssertFalse(AppSessionMessage.handles(nil))
    }

    func testTheSheetAnswersAProviderAndItsTokenOrNothing() {
        XCTAssertEqual(AppSessionMessage.signInReply(.apple, .idToken("t")) as NSDictionary, ["provider": "apple", "idToken": "t"])
        XCTAssertEqual(AppSessionMessage.signInReply(.google, .idToken("g")) as NSDictionary, ["provider": "google", "idToken": "g"])
        for (provider, outcome) in [(SignInProvider.apple, ProviderOutcome.cancelled), (.google, .failed), (.apple, .idToken(""))] {
            XCTAssertEqual(AppSessionMessage.signInReply(provider, outcome) as NSDictionary, [:])
        }
        XCTAssertEqual(AppSessionMessage.signInReply(nil, nil) as NSDictionary, [:])
    }

    func testTheAppSheetSaysItIsForOnlineTranslationInEveryLanguage() {
        let ledes = [SignInLanguage.fr, .en, .es].map { SignInCopy(language: $0).appLede }
        XCTAssertEqual(Set(ledes).count, 3)
        XCTAssertTrue(ledes[0].contains("traduction en ligne"))
        for language in [SignInLanguage.fr, .en, .es] {
            XCTAssertNotEqual(SignInCopy(language: language).appLede, SignInCopy(language: language).lede)
        }
    }

    /// The real Keychain, under a service of the test's own, cleaned up. Skipped where the process has
    /// no keychain to write to (a locked or absent one on a build machine).
    func testTheKeychainKeepsReplacesAndForgetsTheToken() throws {
        let store = KeychainRefreshTokenStore(service: "com.cymbra.lingua.tests.\(UUID().uuidString)")
        defer { store.clear() }
        guard store.write("first") else { throw XCTSkip("no writable keychain for this process") }
        XCTAssertEqual(store.read(), "first")
        XCTAssertTrue(store.write("second"))
        XCTAssertEqual(store.read(), "second")
        XCTAssertFalse(store.write(""))
        store.clear()
        XCTAssertNil(store.read())
    }
}
