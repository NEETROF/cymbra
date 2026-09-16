import XCTest
@testable import LinguaSignIn

final class GoogleOAuthTests: XCTestCase {
    private let google = GoogleOAuth(clientId: "123-abc.apps.googleusercontent.com")!

    private func query(_ url: URL) -> [String: String] {
        let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems ?? []
        return Dictionary(uniqueKeysWithValues: items.map { ($0.name, $0.value ?? "") })
    }

    func testOnlyAGoogleClientIdConfiguresGoogle() {
        XCTAssertNil(GoogleOAuth(clientId: nil))
        XCTAssertNil(GoogleOAuth(clientId: ""))
        XCTAssertNil(GoogleOAuth(clientId: ".apps.googleusercontent.com"))
        XCTAssertNil(GoogleOAuth(clientId: "com.cymbra.lingua"))
    }

    func testRedirectsToTheReversedClientId() {
        XCTAssertEqual(google.callbackScheme, "com.googleusercontent.apps.123-abc")
        XCTAssertEqual(google.redirectUri, "com.googleusercontent.apps.123-abc:/oauth2redirect")
    }

    func testChallengeFollowsRFC7636() {
        XCTAssertEqual(
            GoogleOAuth.challenge(for: "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        )
    }

    func testRandomTokensAreUrlSafeVerifiers() {
        let token = GoogleOAuth.randomToken()
        XCTAssertEqual(token.count, 43)
        XCTAssertNil(token.rangeOfCharacter(from: CharacterSet(charactersIn: "+/=")))
        XCTAssertNotEqual(token, GoogleOAuth.randomToken())
    }

    func testBeginsAnAuthorizationCodeRequestWithPKCE() {
        let attempt = google.begin(state: "s1", verifier: "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
        XCTAssertEqual(attempt.url.host, "accounts.google.com")
        let q = query(attempt.url)
        XCTAssertEqual(q["client_id"], "123-abc.apps.googleusercontent.com")
        XCTAssertEqual(q["redirect_uri"], "com.googleusercontent.apps.123-abc:/oauth2redirect")
        XCTAssertEqual(q["response_type"], "code")
        XCTAssertEqual(q["scope"], "openid email profile")
        XCTAssertEqual(q["code_challenge"], "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")
        XCTAssertEqual(q["code_challenge_method"], "S256")
        XCTAssertEqual(q["state"], "s1")
    }

    func testReadsTheCallback() {
        let attempt = google.begin(state: "s1", verifier: "v")
        let base = "com.googleusercontent.apps.123-abc:/oauth2redirect"
        XCTAssertEqual(google.callback(URL(string: "\(base)?state=s1&code=4/0Ab")!, for: attempt), .code("4/0Ab"))
        XCTAssertEqual(google.callback(URL(string: "\(base)?state=other&code=4/0Ab")!, for: attempt), .failed)
        XCTAssertEqual(google.callback(URL(string: "\(base)?state=s1")!, for: attempt), .failed)
        XCTAssertEqual(google.callback(URL(string: "\(base)?error=access_denied&state=s1")!, for: attempt), .cancelled)
        XCTAssertEqual(google.callback(URL(string: "\(base)?error=invalid_request")!, for: attempt), .failed)
    }

    func testExchangesTheCodeWithTheVerifierAndNoSecret() throws {
        let attempt = google.begin(state: "s1", verifier: "the-verifier")
        let request = google.tokenRequest(code: "4/0Ab+c", for: attempt)
        XCTAssertEqual(request.url?.absoluteString, "https://oauth2.googleapis.com/token")
        XCTAssertEqual(request.httpMethod, "POST")
        XCTAssertEqual(request.value(forHTTPHeaderField: "Content-Type"), "application/x-www-form-urlencoded")
        let body = try XCTUnwrap(request.httpBody.flatMap { String(data: $0, encoding: .utf8) })
        let fields = Set(body.split(separator: "&").map(String.init))
        XCTAssertEqual(fields, [
            "grant_type=authorization_code",
            "code=4%2F0Ab%2Bc",
            "client_id=123-abc.apps.googleusercontent.com",
            "redirect_uri=com.googleusercontent.apps.123-abc%3A%2Foauth2redirect",
            "code_verifier=the-verifier",
        ])
    }

    func testReadsTheIdTokenOfATokenResponse() {
        XCTAssertEqual(GoogleOAuth.idToken(fromTokenResponse: Data(#"{"id_token":"jwt","access_token":"a"}"#.utf8)), "jwt")
        XCTAssertNil(GoogleOAuth.idToken(fromTokenResponse: Data(#"{"access_token":"a"}"#.utf8)))
        XCTAssertNil(GoogleOAuth.idToken(fromTokenResponse: Data("not json".utf8)))
    }

    func testReportsTheProvidersTheAppOffers() {
        let message: [String: Any] = ["type": "auth.providers"]
        XCTAssertEqual(
            NativeMessage.reply(to: message, handoff: nil, googleClientId: "123-abc.apps.googleusercontent.com") as? [String: Bool],
            ["apple": true, "google": true]
        )
        XCTAssertEqual(
            NativeMessage.reply(to: message, handoff: nil, googleClientId: "") as? [String: Bool],
            ["apple": true, "google": false]
        )
    }
}
