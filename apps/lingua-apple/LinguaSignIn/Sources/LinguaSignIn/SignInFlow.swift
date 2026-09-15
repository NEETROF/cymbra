import Foundation

/// The host app's sign-in link, `cymbra-lingua://signin?provider=apple|google`, opened by the
/// Safari extension (`hostAppSignInUrl` in apps/lingua-extension). The scheme is declared in
/// both apps' `CFBundleURLTypes`.
public enum SignInLink {
    public static let scheme = "cymbra-lingua"

    /// The provider a sign-in link asks for; nil for any other URL.
    public static func provider(from url: URL) -> SignInProvider? {
        guard url.scheme?.lowercased() == scheme, url.host?.lowercased() == "signin",
              let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems,
              let value = items.first(where: { $0.name == "provider" })?.value
        else { return nil }
        return SignInProvider(rawValue: value)
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

/// The sheet's French copy. A failure names its provider and never reads as a password error.
public enum SignInCopy {
    public static let heading = "Connexion à Cymbra Lingua"
    public static let lede = "Connecte-toi pour retrouver tes mots et tes révisions sur tous tes appareils."
    public static let done = "C'est fait ! Retourne dans Safari : Cymbra Lingua termine la connexion."
    public static let close = "Fermer"
    public static let cancel = "Annuler"

    public static func button(_ provider: SignInProvider) -> String {
        "Continuer avec \(name(provider))"
    }

    public static func failure(_ provider: SignInProvider) -> String {
        "La connexion avec \(name(provider)) n'a pas abouti. Réessaie dans un instant."
    }

    public static func unavailable(_ provider: SignInProvider) -> String {
        "La connexion avec \(name(provider)) n'est pas encore disponible dans cette version."
    }

    private static func name(_ provider: SignInProvider) -> String {
        switch provider {
        case .apple: return "Apple"
        case .google: return "Google"
        }
    }
}
