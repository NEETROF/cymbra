import Foundation

/// The languages the host app speaks (localise-lingua-apple-host): the extension's interface
/// languages, `fr`, `en` and `es`, which the activation page (copy.js) and the sign-in sheet
/// (`SignInCopy`) each hold a copy in. Which of them a build *declares* is `CFBundleLocalizations`,
/// written by tool/app_localizations.sh from the natives of the shipped pairs; a screen shows the
/// language it is asked for only among those, the bundle's preferred localisation otherwise.
public enum SignInLanguage: String, CaseIterable, Codable, Sendable {
    case fr
    case en
    case es

    /// The languages a bundle declares, among these — `Bundle.localizations` reads
    /// `CFBundleLocalizations`; `Base` and anything else drop out.
    public static func offered(by localizations: [String]) -> [SignInLanguage] {
        localizations.compactMap(SignInLanguage.init(rawValue:))
    }

    /// The bundle's preferred localisation — the first of `preferredLocalizations`, which Foundation
    /// picks from the declared languages by the device's — mapped here; anything else (`Base`, a
    /// regional id) maps to the development region, and that to French when it is none of these.
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
    /// link or the App Group — when the bundle offers it; the preferred localisation when the bundle
    /// offers that; the first offered otherwise, French when none is. The last two matter on a
    /// French-only build: Xcode writes the project's development language, `en`, as its
    /// CFBundleDevelopmentRegion (tool/app_localizations.sh leaves that as it is), so a device in
    /// English reads `en` as its preferred localisation while the build offers French alone.
    public static func shown(_ requested: SignInLanguage?, offered: [SignInLanguage], preferred: SignInLanguage) -> SignInLanguage {
        if let requested, offered.contains(requested) { return requested }
        if offered.contains(preferred) { return preferred }
        return offered.first ?? .fr
    }

    public static func shown(_ requested: SignInLanguage?, in bundle: Bundle) -> SignInLanguage {
        shown(requested, offered: offered(by: bundle.localizations), preferred: preferred(in: bundle))
    }
}

/// The extension's interface language, as the extension last said it (localise-lingua-apple-host,
/// D2): its background sends `interface.language` at start and on each change (Safari only), the
/// native handler keeps it here — in the App Group suite the id_token hand-off uses — and the host
/// app's activation page reads it before it loads, so a French reader on a device in English keeps a
/// French page (M22). Nil until the extension has run once: the page then follows the device.
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
