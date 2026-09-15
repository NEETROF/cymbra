import AuthenticationServices
import LinguaSignIn
import os.log

/// Continue with Google through `ASWebAuthenticationSession`, without the Google Sign-In SDK
/// (add-lingua-connected-clients, D6). The client id comes from the `LinguaGoogleClientId`
/// Info.plist key (`LINGUA_GOOGLE_CLIENT_ID` build setting).
final class GoogleWebSignIn: NSObject, GoogleIdTokenSource, ASWebAuthenticationPresentationContextProviding {
    private let oauth: GoogleOAuth
    private let anchor: () -> ASPresentationAnchor
    private var session: ASWebAuthenticationSession?

    /// Nil when the build has no Google client: the sheet then says Google is not available.
    static func fromBundle(anchor: @escaping () -> ASPresentationAnchor) -> GoogleWebSignIn? {
        let clientId = Bundle.main.object(forInfoDictionaryKey: "LinguaGoogleClientId") as? String
        return GoogleOAuth(clientId: clientId).map { GoogleWebSignIn(oauth: $0, anchor: anchor) }
    }

    private init(oauth: GoogleOAuth, anchor: @escaping () -> ASPresentationAnchor) {
        self.oauth = oauth
        self.anchor = anchor
    }

    func idToken() async -> ProviderOutcome {
        let attempt = oauth.begin()
        let redirect: URL
        do {
            redirect = try await authorize(attempt)
        } catch {
            if (error as? ASWebAuthenticationSessionError)?.code == .canceledLogin { return .cancelled }
            os_log(.error, "Google authorization failed: %{public}@", String(describing: error))
            return .failed
        }
        switch oauth.callback(redirect, for: attempt) {
        case .cancelled:
            return .cancelled
        case .failed:
            os_log(.error, "Google redirect carried no usable code")
            return .failed
        case let .code(code):
            do {
                let (data, response) = try await URLSession.shared.data(for: oauth.tokenRequest(code: code, for: attempt))
                guard (response as? HTTPURLResponse)?.statusCode == 200, let token = GoogleOAuth.idToken(fromTokenResponse: data) else {
                    os_log(.error, "Google token exchange returned no id_token")
                    return .failed
                }
                return .idToken(token)
            } catch {
                os_log(.error, "Google token exchange failed: %{public}@", String(describing: error))
                return .failed
            }
        }
    }

    private func authorize(_ attempt: GoogleOAuth.Attempt) async throws -> URL {
        try await withCheckedThrowingContinuation { continuation in
            let session = ASWebAuthenticationSession(url: attempt.url, callbackURLScheme: oauth.callbackScheme) { url, error in
                if let url {
                    continuation.resume(returning: url)
                } else {
                    continuation.resume(throwing: error ?? URLError(.unknown))
                }
            }
            session.presentationContextProvider = self
            self.session = session
            if !session.start() {
                continuation.resume(throwing: URLError(.cannotLoadFromNetwork))
            }
        }
    }

    func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        anchor()
    }
}
