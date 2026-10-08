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

    func testReadsTheLanguageOfASignInLink() {
        XCTAssertEqual(SignInLink.language(from: URL(string: "cymbra-lingua://signin?provider=apple&lang=fr")!), .fr)
        XCTAssertEqual(SignInLink.language(from: URL(string: "cymbra-lingua://signin?provider=google&lang=en")!), .en)
        XCTAssertEqual(SignInLink.language(from: URL(string: "cymbra-lingua://signin?lang=es&provider=apple")!), .es)
        // The provider still reads with a language beside it.
        XCTAssertEqual(SignInLink.provider(from: URL(string: "cymbra-lingua://signin?provider=apple&lang=es")!), .apple)
    }

    func testAnOlderOrForeignLanguageReadsAsNone() {
        // An older link names no language; any page can open the link, so an unknown value is
        // dropped rather than shown or passed on.
        for link in [
            "cymbra-lingua://signin?provider=apple",
            "cymbra-lingua://signin?provider=apple&lang=",
            "cymbra-lingua://signin?provider=apple&lang=de",
            "cymbra-lingua://signin?provider=apple&lang=EN",
            "cymbra-lingua://signin?provider=apple&lang=en-US",
            "cymbra-lingua://signin?provider=apple&lang=%3Cscript%3E",
            "cymbra-lingua://settings?provider=apple&lang=en",
        ] {
            XCTAssertNil(SignInLink.language(from: URL(string: link)!), link)
        }
    }

    func testTheFrenchCopyIsWhatTheSheetShowed() {
        // Byte for byte (M23): the strings the sheet held before the table.
        let copy = SignInCopy(language: .fr)
        XCTAssertEqual(copy.heading, "Connexion à Cymbra Lingua")
        XCTAssertEqual(copy.lede, "Connecte-toi pour retrouver tes mots et tes révisions sur tous tes appareils.")
        XCTAssertEqual(copy.done, "C'est fait ! Retourne dans Safari : Cymbra Lingua termine la connexion.")
        XCTAssertEqual(
            copy.browserWaiting,
            "Termine la connexion dans la fenêtre de ton navigateur. Si rien ne s'ouvre, annule, quitte ton navigateur et réessaie."
        )
        XCTAssertEqual(copy.close, "Fermer")
        XCTAssertEqual(copy.cancel, "Annuler")
        XCTAssertEqual(copy.button(.apple), "Continuer avec Apple")
        XCTAssertEqual(copy.button(.google), "Continuer avec Google")
        XCTAssertEqual(copy.failure(.apple), "La connexion avec Apple n'a pas abouti. Réessaie dans un instant.")
        XCTAssertEqual(copy.failure(.google), "La connexion avec Google n'a pas abouti. Réessaie dans un instant.")
        XCTAssertEqual(copy.unavailable(.google), "La connexion avec Google n'est pas encore disponible dans cette version.")
    }

    func testFailureCopyNamesTheProviderAndNeverAPassword() {
        // In every language: a failure names its provider, and no message reads as a password error.
        let password = ["mot de passe", "password", "contraseña"]
        for language in SignInLanguage.allCases {
            let copy = SignInCopy(language: language)
            for provider in [SignInProvider.apple, .google] {
                let name = provider == .apple ? "Apple" : "Google"
                XCTAssertTrue(copy.failure(provider).contains(name), "\(language) failure")
                XCTAssertTrue(copy.button(provider).contains(name), "\(language) button")
                XCTAssertTrue(copy.unavailable(provider).contains(name), "\(language) unavailable")
                XCTAssertFalse(copy.failure(provider).contains("{provider}"), "\(language) placeholder left")
            }
            let messages = [copy.heading, copy.lede, copy.done, copy.browserWaiting, copy.close, copy.cancel,
                            copy.failure(.apple), copy.unavailable(.google)]
            for message in messages {
                XCTAssertFalse(message.isEmpty, "\(language)")
                for word in password {
                    XCTAssertFalse(message.lowercased().contains(word), "\(language): \(message)")
                }
            }
        }
    }

    func testTheDraftsAreNotTheFrench() {
        let fr = SignInCopy(language: .fr)
        for language in [SignInLanguage.en, .es] {
            let copy = SignInCopy(language: language)
            XCTAssertNotEqual(copy.heading, fr.heading, "\(language)")
            XCTAssertNotEqual(copy.lede, fr.lede, "\(language)")
            XCTAssertNotEqual(copy.done, fr.done, "\(language)")
            XCTAssertNotEqual(copy.browserWaiting, fr.browserWaiting, "\(language)")
            XCTAssertNotEqual(copy.close, fr.close, "\(language)")
            XCTAssertNotEqual(copy.cancel, fr.cancel, "\(language)")
            XCTAssertNotEqual(copy.failure(.apple), fr.failure(.apple), "\(language)")
        }
        XCTAssertNotEqual(SignInCopy(language: .en).heading, SignInCopy(language: .es).heading)
        // Spanish speaks to the reader (tú), never to readers (vosotros).
        let es = SignInCopy(language: .es)
        let spanish = [es.lede, es.done, es.browserWaiting, es.failure(.apple)].joined(separator: " ").lowercased()
        for form in ["vosotros", "vuestr", "volved", "vuelvan", "cancelad", "cerrad"] {
            XCTAssertFalse(spanish.contains(form), form)
        }
        XCTAssertTrue(spanish.contains("vuelve"))
    }
}
