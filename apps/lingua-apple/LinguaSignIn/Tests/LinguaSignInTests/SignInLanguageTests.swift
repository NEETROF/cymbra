import XCTest
@testable import LinguaSignIn

final class SignInLanguageTests: XCTestCase {
    private var suiteName = ""
    private var defaults: UserDefaults!

    override func setUp() {
        super.setUp()
        suiteName = "lingua-language-tests-\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suiteName)
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suiteName)
        super.tearDown()
    }

    func testTheOfferedLanguagesAreTheDeclaredOnesTheAppSpeaks() {
        XCTAssertEqual(SignInLanguage.offered(by: ["fr"]), [.fr])
        XCTAssertEqual(SignInLanguage.offered(by: ["Base", "fr", "en"]), [.fr, .en])
        XCTAssertEqual(SignInLanguage.offered(by: ["fr", "en", "es", "de", "en-GB"]), [.fr, .en, .es])
        XCTAssertEqual(SignInLanguage.offered(by: []), [])
    }

    func testThePreferredLocalisationMapsOrFallsBackToTheDevelopmentRegion() {
        XCTAssertEqual(SignInLanguage.preferred(["en", "fr"], developmentRegion: "fr"), .en)
        XCTAssertEqual(SignInLanguage.preferred(["es"], developmentRegion: "en"), .es)
        // Base, a regional id, or nothing at all: the development region.
        XCTAssertEqual(SignInLanguage.preferred(["Base"], developmentRegion: "fr"), .fr)
        XCTAssertEqual(SignInLanguage.preferred(["en-GB"], developmentRegion: "en"), .en)
        XCTAssertEqual(SignInLanguage.preferred([], developmentRegion: "es"), .es)
        // A development region the app does not speak, or none: French, what every device showed.
        XCTAssertEqual(SignInLanguage.preferred(["Base"], developmentRegion: "de"), .fr)
        XCTAssertEqual(SignInLanguage.preferred([], developmentRegion: nil), .fr)
    }

    func testAScreenShowsTheRequestedLanguageOnlyAmongTheOffered() {
        // Every shipped pair glossed in French: the sheet asked for English shows French.
        XCTAssertEqual(SignInLanguage.shown(.en, offered: [.fr], preferred: .fr), .fr)
        // es-en ships: a Spanish-native reader's sheet is Spanish once Spanish is offered…
        XCTAssertEqual(SignInLanguage.shown(.es, offered: [.fr, .en, .es], preferred: .en), .es)
        // …and the device's preferred localisation when it is not.
        XCTAssertEqual(SignInLanguage.shown(.es, offered: [.fr, .en], preferred: .en), .en)
        // An older link, naming none: the preferred localisation.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en], preferred: .en), .en)
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr], preferred: .fr), .fr)
    }

    func testAPreferredLocalisationTheBuildDoesNotOfferFallsBackToTheFirstOffered() {
        // A French-only build on a device in English: Xcode's development region, en, reads as the
        // preferred localisation, and the page stays French, as before (the spec's scenario).
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr], preferred: .en), .fr)
        XCTAssertEqual(SignInLanguage.shown(.en, offered: [.fr], preferred: .en), .fr)
        // fr and es ship, not en: French, the fallback while English does not ship.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .es], preferred: .en), .fr)
        // Once English ships, English is the fallback, and a Spanish device's Spanish once that ships.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en], preferred: .en), .en)
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en, .es], preferred: .es), .es)
        // A bundle declaring none of the app's languages: French, what every device showed.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [], preferred: .en), .fr)
        XCTAssertEqual(SignInLanguage.shown(.es, offered: [], preferred: .es), .fr)
    }

    func testTheStoreKeepsWhatTheExtensionLastSaid() {
        let store = InterfaceLanguageStore(defaults: defaults)
        XCTAssertNil(store.current())
        store.remember(.es)
        XCTAssertEqual(store.current(), .es)
        XCTAssertEqual(InterfaceLanguageStore(defaults: defaults).current(), .es)
        store.remember(.fr)
        XCTAssertEqual(store.current(), .fr)
    }

    func testTheStoreReadsAnythingElseAsNothing() {
        defaults.set("de", forKey: InterfaceLanguageStore.key)
        XCTAssertNil(InterfaceLanguageStore(defaults: defaults).current())
        defaults.set(42, forKey: InterfaceLanguageStore.key)
        XCTAssertNil(InterfaceLanguageStore(defaults: defaults).current())
    }

    func testTheNativeMessageKeepsTheLanguage() {
        let store = InterfaceLanguageStore(defaults: defaults)
        let reply = NativeMessage.reply(to: ["type": "interface.language", "language": "es"], handoff: nil, languages: store)
        XCTAssertEqual(reply as? [String: String], ["language": "es"])
        XCTAssertEqual(store.current(), .es)
        XCTAssertEqual(
            NativeMessage.reply(to: ["type": "interface.language", "language": "fr"], handoff: nil, languages: store) as? [String: String],
            ["language": "fr"]
        )
        XCTAssertEqual(store.current(), .fr)
    }

    func testTheNativeMessageRefusesALanguageTheAppDoesNotSpeak() {
        let store = InterfaceLanguageStore(defaults: defaults)
        store.remember(.es)
        for message: [String: Any] in [
            ["type": "interface.language", "language": "de"],
            ["type": "interface.language", "language": "EN"],
            ["type": "interface.language", "language": 1],
            ["type": "interface.language"],
        ] {
            XCTAssertEqual(
                NativeMessage.reply(to: message, handoff: nil, languages: store) as? [String: String],
                ["error": "unknownLanguage"],
                String(describing: message)
            )
        }
        XCTAssertEqual(store.current(), .es)
        // Without an App Group the message is answered and nothing kept.
        XCTAssertEqual(
            NativeMessage.reply(to: ["type": "interface.language", "language": "en"], handoff: nil) as? [String: String],
            ["language": "en"]
        )
    }
}
