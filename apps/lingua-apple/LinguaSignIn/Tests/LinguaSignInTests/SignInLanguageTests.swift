import XCTest
@testable import LinguaSignIn

final class SignInLanguageTests: XCTestCase {
    private var suiteName = ""
    private var defaults: UserDefaults!
    private var bundles: URL!

    override func setUp() {
        super.setUp()
        suiteName = "lingua-language-tests-\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suiteName)
        bundles = FileManager.default.temporaryDirectory.appendingPathComponent("lingua-language-tests-\(UUID().uuidString)")
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suiteName)
        try? FileManager.default.removeItem(at: bundles)
        super.tearDown()
    }

    /// A bundle on disk as Xcode builds the app: `declared` as its CFBundleLocalizations, the
    /// project's development language, `en`, as its CFBundleDevelopmentRegion, and a `Base.lproj`
    /// holding the activation page. A path of its own each time, as `Bundle(path:)` caches by path.
    private func builtBundle(declaring declared: [String]) throws -> Bundle {
        let root = bundles.appendingPathComponent("\(UUID().uuidString).bundle")
        let contents = root.appendingPathComponent("Contents")
        let base = contents.appendingPathComponent("Resources/Base.lproj")
        try FileManager.default.createDirectory(at: base, withIntermediateDirectories: true)
        try Data("<html lang=\"fr\"></html>".utf8).write(to: base.appendingPathComponent("Main.html"))
        let info: [String: Any] = [
            "CFBundleIdentifier": "com.cymbra.lingua.tests.\(UUID().uuidString)",
            "CFBundlePackageType": "BNDL",
            "CFBundleDevelopmentRegion": "en",
            "CFBundleLocalizations": declared,
        ]
        try PropertyListSerialization.data(fromPropertyList: info, format: .xml, options: 0)
            .write(to: contents.appendingPathComponent("Info.plist"))
        return try XCTUnwrap(Bundle(path: root.path))
    }

    func testTheOfferedLanguagesAreTheDeclaredOnesTheAppSpeaks() {
        XCTAssertEqual(SignInLanguage.offered(by: ["fr"]), [.fr])
        XCTAssertEqual(SignInLanguage.offered(by: ["Base", "fr", "en"]), [.fr, .en])
        XCTAssertEqual(SignInLanguage.offered(by: ["fr", "en", "es", "de", "en-GB"]), [.fr, .en, .es])
        XCTAssertEqual(SignInLanguage.offered(by: []), [])
    }

    func testItalianIsOfferedAndPreferredOnceDeclared() {
        // localise-lingua-apple-host-italian D1: `it` read like the others, appended after `es`.
        XCTAssertEqual(SignInLanguage.allCases, [.fr, .en, .es, .it])
        XCTAssertEqual(SignInLanguage.offered(by: ["fr", "en", "es", "it"]), [.fr, .en, .es, .it])
        XCTAssertEqual(SignInLanguage.offered(by: ["fr", "en", "es", "it-IT", "IT"]), [.fr, .en, .es])
        XCTAssertEqual(SignInLanguage.preferred(["it", "en"], developmentRegion: "en"), .it)
        // An Italian device once en-it ships: the page and an older link's sheet are Italian…
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en, .es, .it], preferred: .it), .it)
        XCTAssertEqual(SignInLanguage.shown(.it, offered: [.fr, .en, .es, .it], preferred: .en), .it)
        // …and before it ships, English, as before (*A device in Italian before a pair glossed in Italian ships*).
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en, .es], preferred: .en), .en)
        XCTAssertEqual(SignInLanguage.shown(.it, offered: [.fr, .en, .es], preferred: .es), .es)
    }

    func testABuildDeclaringFrenchEnglishAndSpanishNeverShowsItalian() throws {
        // A build from before en-it (localise-lingua-apple-host-italian D1, D5) declared [fr, en, es],
        // so Italian was never shown, even when asked for; builds since enable-lingua-italian-speakers declare it.
        let bundle = try builtBundle(declaring: ["fr", "en", "es"])
        XCTAssertEqual(SignInLanguage.offered(in: bundle), [.fr, .en, .es])
        XCTAssertNotEqual(SignInLanguage.shown(.it, in: bundle), .it)
        XCTAssertNotEqual(SignInLanguage.shown(nil, in: bundle), .it)
        XCTAssertEqual(SignInLanguage.shown(.es, in: bundle), .es)
    }

    func testABuildDeclaringItalianShowsItalianWhenAskedFor() throws {
        let bundle = try builtBundle(declaring: ["fr", "en", "es", "it"])
        XCTAssertEqual(SignInLanguage.offered(in: bundle), [.fr, .en, .es, .it])
        XCTAssertEqual(SignInLanguage.shown(.it, in: bundle), .it)
        XCTAssertEqual(SignInLanguage.shown(.fr, in: bundle), .fr)
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
        // en-es ships (change 35): a Spanish-native reader's sheet is Spanish once Spanish is offered…
        XCTAssertEqual(SignInLanguage.shown(.es, offered: [.fr, .en, .es], preferred: .en), .es)
        // …and the device's preferred localisation when it is not.
        XCTAssertEqual(SignInLanguage.shown(.es, offered: [.fr, .en], preferred: .en), .en)
        // An older link, naming none: the preferred localisation.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en], preferred: .en), .en)
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr], preferred: .fr), .fr)
    }

    func testAPreferredLocalisationTheBuildDoesNotOfferFallsBackToEnglishOnceOfferedElseFrench() {
        // A French-only build on a device in English: Xcode's development region, en, reads as the
        // preferred localisation, and the page stays French, as before (the spec's scenario).
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr], preferred: .en), .fr)
        XCTAssertEqual(SignInLanguage.shown(.en, offered: [.fr], preferred: .en), .fr)
        // fr and es ship, not en: French, the fallback while English does not ship.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .es], preferred: .en), .fr)
        // Once English ships, English is the fallback, and a Spanish device's Spanish once that ships.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en], preferred: .en), .en)
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [.fr, .en, .es], preferred: .es), .es)
        // English is the fallback once offered, wherever the build lists it.
        XCTAssertEqual(SignInLanguage.fallback(among: [.es, .en]), .en)
        XCTAssertEqual(SignInLanguage.fallback(among: [.fr, .es]), .fr)
        // A bundle declaring none of the app's languages: French, what every device showed.
        XCTAssertEqual(SignInLanguage.shown(nil, offered: [], preferred: .en), .fr)
        XCTAssertEqual(SignInLanguage.shown(.es, offered: [], preferred: .es), .fr)
    }

    func testAFrenchOnlyBuildOffersFrenchAloneWhateverItsDevelopmentRegion() throws {
        let bundle = try builtBundle(declaring: ["fr"])
        // The trap: Foundation's list adds the development region, en, and Base.lproj's Base.
        XCTAssertTrue(bundle.localizations.contains("en"), String(describing: bundle.localizations))
        XCTAssertEqual(SignInLanguage.offered(in: bundle), [.fr])
        // Whatever this machine's languages: the page before the extension has run, an older link,
        // and a link naming English all show French (*A device in English before English ships*,
        // *A language the app does not offer*, *An older link*).
        XCTAssertEqual(SignInLanguage.shown(nil, in: bundle), .fr)
        XCTAssertEqual(SignInLanguage.shown(.en, in: bundle), .fr)
        XCTAssertEqual(SignInLanguage.shown(.es, in: bundle), .fr)
        XCTAssertEqual(SignInLanguage.shown(.fr, in: bundle), .fr)
    }

    func testABuildDeclaringEnglishShowsEnglishWhenAskedFor() throws {
        let bundle = try builtBundle(declaring: ["fr", "en"])
        XCTAssertEqual(SignInLanguage.offered(in: bundle), [.fr, .en])
        XCTAssertEqual(SignInLanguage.shown(.en, in: bundle), .en)
        XCTAssertEqual(SignInLanguage.shown(.fr, in: bundle), .fr)
        // Spanish not offered: the device's language when offered, English otherwise — never Spanish.
        XCTAssertTrue([.fr, .en].contains(SignInLanguage.shown(.es, in: bundle)))
    }

    func testABundleDeclaringNoLanguageOffersNone() throws {
        let bundle = try builtBundle(declaring: [])
        XCTAssertEqual(SignInLanguage.offered(in: bundle), [])
        XCTAssertEqual(SignInLanguage.shown(.en, in: bundle), .fr)
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

    func testTheStoreKeepsItalian() {
        defaults.set("it", forKey: InterfaceLanguageStore.key)
        XCTAssertEqual(InterfaceLanguageStore(defaults: defaults).current(), .it)
        let store = InterfaceLanguageStore(defaults: defaults)
        store.remember(.it)
        XCTAssertEqual(store.current(), .it)
    }

    func testTheNativeMessageKeepsItalian() {
        // localise-lingua-apple-host-italian D2: refused before this change, kept now; the extension
        // sends it only once its interface is shown in Italian (test/native-signin.spec.ts).
        let store = InterfaceLanguageStore(defaults: defaults)
        store.remember(.fr)
        XCTAssertEqual(
            NativeMessage.reply(to: ["type": "interface.language", "language": "it"], handoff: nil, languages: store) as? [String: String],
            ["language": "it"]
        )
        XCTAssertEqual(store.current(), .it)
        XCTAssertEqual(
            NativeMessage.reply(to: ["type": "interface.language", "language": "it-IT"], handoff: nil, languages: store) as? [String: String],
            ["error": "unknownLanguage"]
        )
        XCTAssertEqual(store.current(), .it)
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
