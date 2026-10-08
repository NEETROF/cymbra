import Foundation

/// The host app's sign-in link, `cymbra-lingua://signin?provider=apple|google&lang=fr|en|es`,
/// opened by the Safari extension (`hostAppSignInUrl` in apps/lingua-extension). The scheme is
/// declared in both apps' `CFBundleURLTypes`.
public enum SignInLink {
    public static let scheme = "cymbra-lingua"

    /// The provider a sign-in link asks for; nil for any other URL.
    public static func provider(from url: URL) -> SignInProvider? {
        query(ofSignIn: url, "provider").flatMap(SignInProvider.init(rawValue:))
    }

    /// The interface language a sign-in link names (localise-lingua-apple-host D3), among the
    /// languages the app speaks; nil for an older link naming none, and for any other value — any
    /// page can open the link, so the raw value is never shown or passed on.
    public static func language(from url: URL) -> SignInLanguage? {
        query(ofSignIn: url, "lang").flatMap(SignInLanguage.init(rawValue:))
    }

    private static func query(ofSignIn url: URL, _ name: String) -> String? {
        guard url.scheme?.lowercased() == scheme, url.host?.lowercased() == "signin",
              let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems
        else { return nil }
        return items.first(where: { $0.name == name })?.value
    }
}

/// What a provider's native sheet answered.
public enum ProviderOutcome: Equatable, Sendable {
    case idToken(String)
    case cancelled
    case failed
}

/// What the sign-in sheet shows.
public enum SignInPhase: Equatable, Sendable {
    case choosing
    case done
    case failed(SignInProvider)
}

public enum SignInFlow {
    /// The sheet's next phase once a provider answered: done once the id_token is handed to the
    /// extension; back to choosing on a cancel (nothing to show); a failure worded for that
    /// provider otherwise — including when the App Group is unavailable.
    public static func finish(_ provider: SignInProvider, _ outcome: ProviderOutcome, handoff: IdTokenHandoff?) -> SignInPhase {
        switch outcome {
        case .cancelled:
            return .choosing
        case .failed:
            return .failed(provider)
        case let .idToken(token):
            guard !token.isEmpty, let handoff else { return .failed(provider) }
            handoff.hand(HandedIdToken(provider: provider, idToken: token))
            return .done
        }
    }
}

/// The sheet's copy, in one of the languages the app speaks (localise-lingua-apple-host D3): the
/// French is what the sheet showed before the table, byte for byte; the English and Spanish are
/// drafts after it (tú, no vosotros). A failure names its provider and never reads as a password
/// error, in every language. Apple's own button is labelled by the system, not from here.
public struct SignInCopy: Sendable {
    public let language: SignInLanguage

    public init(language: SignInLanguage) {
        self.language = language
    }

    public var heading: String { table.heading }
    public var lede: String { table.lede }
    public var done: String { table.done }
    public var browserWaiting: String { table.browserWaiting }
    public var close: String { table.close }
    public var cancel: String { table.cancel }

    public func button(_ provider: SignInProvider) -> String {
        Self.name(provider, in: table.button)
    }

    public func failure(_ provider: SignInProvider) -> String {
        Self.name(provider, in: table.failure)
    }

    public func unavailable(_ provider: SignInProvider) -> String {
        Self.name(provider, in: table.unavailable)
    }

    private var table: Table {
        switch language {
        case .fr: return Self.fr
        case .en: return Self.en
        case .es: return Self.es
        }
    }

    /// A sentence naming a provider holds `{provider}` where its name goes, so a translation may
    /// put it elsewhere.
    private static func name(_ provider: SignInProvider, in sentence: String) -> String {
        let name: String
        switch provider {
        case .apple: name = "Apple"
        case .google: name = "Google"
        }
        return sentence.replacingOccurrences(of: "{provider}", with: name)
    }

    private struct Table: Sendable {
        let heading: String
        let lede: String
        let done: String
        let browserWaiting: String
        let close: String
        let cancel: String
        let button: String
        let failure: String
        let unavailable: String
    }

    private static let fr = Table(
        heading: "Connexion à Cymbra Lingua",
        lede: "Connecte-toi pour retrouver tes mots et tes révisions sur tous tes appareils.",
        done: "C'est fait ! Retourne dans Safari : Cymbra Lingua termine la connexion.",
        browserWaiting:
            "Termine la connexion dans la fenêtre de ton navigateur. Si rien ne s'ouvre, annule, quitte ton navigateur et réessaie.",
        close: "Fermer",
        cancel: "Annuler",
        button: "Continuer avec {provider}",
        failure: "La connexion avec {provider} n'a pas abouti. Réessaie dans un instant.",
        unavailable: "La connexion avec {provider} n'est pas encore disponible dans cette version."
    )

    // A draft after the French (US English).
    private static let en = Table(
        heading: "Sign in to Cymbra Lingua",
        lede: "Sign in to find your words and your reviews on all your devices.",
        done: "Done! Go back to Safari: Cymbra Lingua is finishing the sign-in.",
        browserWaiting:
            "Finish signing in in your browser window. If nothing opens, cancel, quit your browser and try again.",
        close: "Close",
        cancel: "Cancel",
        button: "Continue with {provider}",
        failure: "Signing in with {provider} did not go through. Try again in a moment.",
        unavailable: "Signing in with {provider} is not available in this version yet."
    )

    // A draft after the French (tú, no vosotros).
    private static let es = Table(
        heading: "Inicia sesión en Cymbra Lingua",
        lede: "Inicia sesión para encontrar tus palabras y tus repasos en todos tus dispositivos.",
        done: "¡Listo! Vuelve a Safari: Cymbra Lingua está terminando de iniciar sesión.",
        browserWaiting:
            "Termina de iniciar sesión en la ventana de tu navegador. Si no se abre nada, cancela, cierra tu navegador y vuelve a intentarlo.",
        close: "Cerrar",
        cancel: "Cancelar",
        button: "Continuar con {provider}",
        failure: "No se pudo iniciar sesión con {provider}. Vuelve a intentarlo en un momento.",
        unavailable: "Iniciar sesión con {provider} todavía no está disponible en esta versión."
    )
}
