import Foundation

/// The languages the host app speaks (localise-lingua-apple-host): the extension's interface
/// languages, `fr`, `en` and `es`, which the activation page (copy.js) and the sign-in sheet
/// (`SignInCopy`) each hold a copy in. Which of them a build *offers* is its `CFBundleLocalizations`
/// alone, written by tool/app_localizations.sh from the natives of the shipped pairs; a screen shows
/// the language it is asked for only among those, then the device's, then English once English is
/// offered and French otherwise.
public enum SignInLanguage: String, CaseIterable, Codable, Sendable {
    case fr
    case en
    case es

    /// The languages a build offers, among these: what its `CFBundleLocalizations` declares, and
    /// nothing else — `Base` and any other value drop out.
    public static func offered(by declared: [String]) -> [SignInLanguage] {
        declared.compactMap(SignInLanguage.init(rawValue:))
    }

    /// The bundle's `CFBundleLocalizations`, read from its info dictionary. Never
    /// `Bundle.localizations`: that list adds the `.lproj` folders and the development region,
    /// and Xcode writes the project's development language, `en`, as the region of every build
    /// (tool/app_localizations.sh leaves it so on a French-only build), so a French-only build
    /// would offer English.
    public static func offered(in bundle: Bundle) -> [SignInLanguage] {
        offered(by: bundle.object(forInfoDictionaryKey: "CFBundleLocalizations") as? [String] ?? [])
    }

    /// What a screen shows when neither the language it is asked for nor the device's is offered:
    /// English once the build offers it, French otherwise (*Guided activation*).
    public static func fallback(among offered: [SignInLanguage]) -> SignInLanguage {
        offered.contains(.en) ? .en : .fr
    }

    /// The bundle's preferred localisation — the first of `preferredLocalizations`, which Foundation
    /// picks by the device's languages — mapped here; anything else (`Base`, a regional id) maps to
    /// the development region, and that to French when it is none of these. It may name a language
    /// the build does not offer (the development region, `en`, on a French-only build): `shown`
    /// counts it only among the offered.
    public static func preferred(_ preferredLocalizations: [String], developmentRegion: String?) -> SignInLanguage {
        if let first = preferredLocalizations.first, let language = SignInLanguage(rawValue: first) {
            return language
        }
        return developmentRegion.flatMap(SignInLanguage.init(rawValue:)) ?? .fr
    }

    public static func preferred(in bundle: Bundle) -> SignInLanguage {
        preferred(
            bundle.preferredLocalizations,
            developmentRegion: bundle.infoDictionary?["CFBundleDevelopmentRegion"] as? String
        )
    }

    /// The language a screen shows: `requested` — the extension's interface language, from a sign-in
    /// link or the App Group — when the build offers it; the device's, the preferred localisation,
    /// when the build offers that; the fallback otherwise. On a French-only build every answer is
    /// French: a device in English reads `en`, the development region, as its preferred
    /// localisation, which the build does not offer.
    public static func shown(_ requested: SignInLanguage?, offered: [SignInLanguage], preferred: SignInLanguage) -> SignInLanguage {
        if let requested, offered.contains(requested) { return requested }
        if offered.contains(preferred) { return preferred }
        return fallback(among: offered)
    }

    public static func shown(_ requested: SignInLanguage?, in bundle: Bundle) -> SignInLanguage {
        shown(requested, offered: offered(in: bundle), preferred: preferred(in: bundle))
    }
}

/// The extension's interface language, as the extension last said it (localise-lingua-apple-host,
/// D2): its background sends `interface.language` at start and on each change, once its key holds a
/// language (Safari only); the native handler keeps it here — in the App Group suite the id_token
/// hand-off uses — and the host app's activation page reads it before it loads, so a French reader
/// on a device in English keeps a French page (M22). Nil until the extension has said one: the page
/// then follows the device.
public final class InterfaceLanguageStore {
    static let key = "lingua.interfaceLanguage"

    private let defaults: UserDefaults

    public init(defaults: UserDefaults) {
        self.defaults = defaults
    }

    /// The App Group's store, or nil when the App Group is not available to this process.
    public static func shared() -> InterfaceLanguageStore? {
        UserDefaults(suiteName: IdTokenHandoff.appGroup).map { InterfaceLanguageStore(defaults: $0) }
    }

    public func remember(_ language: SignInLanguage) {
        defaults.set(language.rawValue, forKey: Self.key)
    }

    /// What the extension last said; nil when it never did, or when what is stored is not a language.
    public func current() -> SignInLanguage? {
        defaults.string(forKey: Self.key).flatMap(SignInLanguage.init(rawValue:))
    }
}
