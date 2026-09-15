import XCTest
@testable import LinguaSignIn

final class SignInFlowTests: XCTestCase {
    private var suiteName = ""
    private var defaults: UserDefaults!

    override func setUp() {
        super.setUp()
        suiteName = "lingua-signin-flow-tests-\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suiteName)
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suiteName)
        super.tearDown()
    }

    func testReadsTheProviderOfASignInLink() {
        XCTAssertEqual(SignInLink.provider(from: URL(string: "cymbra-lingua://signin?provider=apple")!), .apple)
        XCTAssertEqual(SignInLink.provider(from: URL(string: "cymbra-lingua://signin?provider=google")!), .google)
    }

    func testIgnoresOtherLinks() {
        for link in [
            "cymbra-lingua://signin",
            "cymbra-lingua://signin?provider=github",
            "cymbra-lingua://settings?provider=apple",
            "https://cymbra.app/signin?provider=apple",
        ] {
            XCTAssertNil(SignInLink.provider(from: URL(string: link)!), link)
        }
    }

    func testHandsTheTokenOverAndReportsDone() {
        let handoff = IdTokenHandoff(defaults: defaults)
        XCTAssertEqual(SignInFlow.finish(.apple, .idToken("tok"), handoff: handoff), .done)
        XCTAssertEqual(handoff.take(), HandedIdToken(provider: .apple, idToken: "tok"))
    }

    func testACancelShowsNothingAndHandsNothing() {
        let handoff = IdTokenHandoff(defaults: defaults)
        XCTAssertEqual(SignInFlow.finish(.google, .cancelled, handoff: handoff), .choosing)
        XCTAssertNil(handoff.take())
    }

    func testAFailureIsWordedForItsProvider() {
        let handoff = IdTokenHandoff(defaults: defaults)
        XCTAssertEqual(SignInFlow.finish(.google, .failed, handoff: handoff), .failed(.google))
        XCTAssertEqual(SignInFlow.finish(.apple, .idToken(""), handoff: handoff), .failed(.apple))
        XCTAssertEqual(SignInFlow.finish(.apple, .idToken("tok"), handoff: nil), .failed(.apple))
        XCTAssertNil(handoff.take())
    }

    func testFailureCopyNamesTheProviderAndNeverAPassword() {
        for provider in [SignInProvider.apple, .google] {
            let copy = SignInCopy.failure(provider)
            XCTAssertTrue(copy.contains(provider == .apple ? "Apple" : "Google"))
            XCTAssertFalse(copy.lowercased().contains("mot de passe"))
        }
    }
}
