import CryptoKit
import Foundation

/// Continue with Google without the Google Sign-In SDK (add-lingua-connected-clients, D6): the
/// authorization-code flow with PKCE for Google's iOS OAuth client, shown by the app in
/// `ASWebAuthenticationSession`. An iOS client has no secret, and its redirect uses the
/// reversed client id, which the session intercepts (no `CFBundleURLTypes` entry needed).
public struct GoogleOAuth: Equatable, Sendable {
    static let clientSuffix = ".apps.googleusercontent.com"
    static let authorizeEndpoint = URL(string: "https://accounts.google.com/o/oauth2/v2/auth")!
    static let tokenEndpoint = URL(string: "https://oauth2.googleapis.com/token")!

    public let clientId: String

    /// Nil unless `clientId` is an OAuth client id (`<id>.apps.googleusercontent.com`): an empty
    /// `LINGUA_GOOGLE_CLIENT_ID` build setting means Google is not configured.
    public init?(clientId: String?) {
        guard let clientId, clientId.hasSuffix(Self.clientSuffix), clientId.count > Self.clientSuffix.count else {
            return nil
        }
        self.clientId = clientId
    }

    /// `com.googleusercontent.apps.<id>`: the redirect scheme Google gives an iOS client.
    public var callbackScheme: String {
        "com.googleusercontent.apps." + clientId.dropLast(Self.clientSuffix.count)
    }

    public var redirectUri: String {
        "\(callbackScheme):/oauth2redirect"
    }

    /// One sign-in attempt: the page to show, and the secrets checked on the way back.
    public struct Attempt: Equatable, Sendable {
        public let url: URL
        public let state: String
        public let verifier: String
    }

    /// What Google's redirect says.
    public enum Callback: Equatable, Sendable {
        case code(String)
        case cancelled
        case failed
    }

    public func begin(state: String = GoogleOAuth.randomToken(), verifier: String = GoogleOAuth.randomToken()) -> Attempt {
        var components = URLComponents(url: Self.authorizeEndpoint, resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "client_id", value: clientId),
            URLQueryItem(name: "redirect_uri", value: redirectUri),
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "scope", value: "openid email profile"),
            URLQueryItem(name: "code_challenge", value: Self.challenge(for: verifier)),
            URLQueryItem(name: "code_challenge_method", value: "S256"),
            URLQueryItem(name: "state", value: state),
            URLQueryItem(name: "prompt", value: "select_account"),
        ]
        return Attempt(url: components.url!, state: state, verifier: verifier)
    }

    /// A code when the redirect carries one for this attempt's `state`; a cancel when the reader
    /// declined (`access_denied`); a failure otherwise.
    public func callback(_ url: URL, for attempt: Attempt) -> Callback {
        let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems ?? []
        func value(_ name: String) -> String? { items.first(where: { $0.name == name })?.value }
        if let error = value("error") { return error == "access_denied" ? .cancelled : .failed }
        guard value("state") == attempt.state, let code = value("code"), !code.isEmpty else { return .failed }
        return .code(code)
    }

    /// The token exchange for a code: PKCE verifier, no client secret.
    public func tokenRequest(code: String, for attempt: Attempt) -> URLRequest {
        var request = URLRequest(url: Self.tokenEndpoint)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
        request.httpBody = Self.form([
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", clientId),
            ("redirect_uri", redirectUri),
            ("code_verifier", attempt.verifier),
        ])
        return request
    }

    /// The id_token of a token response; nil when it has none.
    public static func idToken(fromTokenResponse data: Data) -> String? {
        struct Response: Decodable {
            let idToken: String?
            enum CodingKeys: String, CodingKey { case idToken = "id_token" }
        }
        guard let token = (try? JSONDecoder().decode(Response.self, from: data))?.idToken, !token.isEmpty else {
            return nil
        }
        return token
    }

    /// 32 random bytes, base64url: a PKCE verifier (43 characters) or a `state`.
    public static func randomToken() -> String {
        var generator = SystemRandomNumberGenerator()
        return base64url(Data((0..<32).map { _ in UInt8.random(in: .min ... .max, using: &generator) }))
    }

    static func challenge(for verifier: String) -> String {
        base64url(Data(SHA256.hash(data: Data(verifier.utf8))))
    }

    static func base64url(_ data: Data) -> String {
        data.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }

    /// `application/x-www-form-urlencoded`, escaping everything but unreserved characters.
    static func form(_ pairs: [(String, String)]) -> Data {
        let unreserved = CharacterSet(charactersIn: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~")
        let body = pairs
            .map { "\($0.0)=\($0.1.addingPercentEncoding(withAllowedCharacters: unreserved) ?? "")" }
            .joined(separator: "&")
        return Data(body.utf8)
    }
}
