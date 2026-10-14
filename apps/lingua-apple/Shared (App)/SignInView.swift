import AuthenticationServices
import LinguaSignIn
import SwiftUI
import os.log

/// Obtains a Google id_token natively for `com.cymbra.lingua`. Not wired yet: the Google
/// client and its implementation come with task 4.5; until then the sheet says Google is not
/// available rather than failing.
protocol GoogleIdTokenSource {
    func idToken() async -> ProviderOutcome
    /// Abandon an attempt in flight; its `idToken()` then answers `.cancelled`.
    func cancel()
}

/// The host app's sign-in sheet (add-lingua-connected-clients, design D6), opened by the
/// Safari extension through `cymbra-lingua://signin?provider=…&lang=…`. It runs the requested
/// provider's native sheet and hands the id_token to the extension through the App Group;
/// the Cymbra session itself stays in the extension. Its copy is in the extension's interface
/// language when the app offers it (localise-lingua-apple-host, D3); Apple's button keeps the
/// label the system gives it.
struct SignInView: View {
    let requested: SignInProvider
    let language: SignInLanguage
    let google: GoogleIdTokenSource?
    let handoff: IdTokenHandoff?
    let close: () -> Void

    @State private var phase: SignInPhase = .choosing
    @State private var working = false

    private var copy: SignInCopy { SignInCopy(language: language) }

    var body: some View {
        VStack(spacing: 18) {
            Text(copy.heading)
                .font(.title2.bold())
                .foregroundStyle(Palette.text)
            if phase == .done {
                Text(copy.done)
                    .foregroundStyle(Palette.green)
                Button(copy.close, action: close)
                    .buttonStyle(.plain)
                    .foregroundStyle(Palette.accent)
            } else {
                Text(copy.lede)
                    .foregroundStyle(Palette.muted)
                providerButton
                    .disabled(working)
                if working {
                    Text(copy.browserWaiting)
                        .foregroundStyle(Palette.muted)
                }
                if case let .failed(provider) = phase {
                    Text(copy.failure(provider))
                        .foregroundStyle(Palette.coral)
                }
                Button(copy.cancel, action: close)
                    .buttonStyle(.plain)
                    .foregroundStyle(Palette.accent)
            }
        }
        .multilineTextAlignment(.center)
        .padding(28)
        .frame(maxWidth: 420)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Palette.background)
        // The sheet's language for what SwiftUI formats and reads aloud; Apple's button keeps the
        // label the system gives it.
        .environment(\.locale, Locale(identifier: language.rawValue))
        // Cancelling or closing the sheet also abandons a browser attempt still in flight.
        .onDisappear { google?.cancel() }
    }

    @ViewBuilder
    private var providerButton: some View {
        switch requested {
        case .apple:
            SignInWithAppleButton(.continue) { request in
                request.requestedScopes = [.email]
            } onCompletion: { result in
                phase = SignInFlow.finish(.apple, appleOutcome(result), handoff: handoff)
            }
            .signInWithAppleButtonStyle(.white)
            .frame(height: 48)
        case .google:
            if let google {
                Button {
                    working = true
                    Task {
                        phase = SignInFlow.finish(.google, await google.idToken(), handoff: handoff)
                        working = false
                    }
                } label: {
                    Text(copy.button(.google))
                        .font(.body.weight(.semibold))
                        .frame(maxWidth: .infinity, minHeight: 48)
                }
                .buttonStyle(.plain)
                .foregroundStyle(Palette.accentInk)
                .background(Palette.accentStrong, in: RoundedRectangle(cornerRadius: 12))
            } else {
                Text(copy.unavailable(.google))
                    .foregroundStyle(Palette.muted)
            }
        }
    }

    private func appleOutcome(_ result: Result<ASAuthorization, Error>) -> ProviderOutcome {
        switch result {
        case let .success(authorization):
            let credential = authorization.credential as? ASAuthorizationAppleIDCredential
            guard let data = credential?.identityToken, let token = String(data: data, encoding: .utf8) else {
                os_log(.error, "Sign in with Apple returned no identity token")
                return .failed
            }
            return .idToken(token)
        case let .failure(error):
            if (error as? ASAuthorizationError)?.code == .canceled { return .cancelled }
            os_log(.error, "Sign in with Apple failed: %{public}@", String(describing: error))
            return .failed
        }
    }
}

/// The sheet the app opens for its own Cymbra session (add-lingua-review-translation-fallback, D8):
/// asked by its review page on the first press of « Traduction en ligne » (`auth.signIn`). Both
/// providers, Apple first; the id_token goes back to the page, which exchanges it — never to the
/// App Group, never to the Safari extension, whose session this does not touch. A third entry, below
/// Google, hands the sign-in to the page's email form (add-lingua-app-email-signin D2): the password is
/// typed there, never here. `finish` is called once: with the reader's choice, or with nothing when
/// they close the sheet.
struct AppSignInView: View {
    let language: SignInLanguage
    let google: GoogleIdTokenSource?
    let finish: (AppSignInChoice?) -> Void

    @State private var failed: SignInProvider?
    @State private var working = false

    private var copy: SignInCopy { SignInCopy(language: language) }

    var body: some View {
        VStack(spacing: 18) {
            Text(copy.heading)
                .font(.title2.bold())
                .foregroundStyle(Palette.text)
            Text(copy.appLede)
                .foregroundStyle(Palette.muted)
            SignInWithAppleButton(.continue) { request in
                request.requestedScopes = [.email]
            } onCompletion: { result in
                answered(.apple, appleOutcome(result))
            }
            .signInWithAppleButtonStyle(.white)
            .frame(height: 48)
            .disabled(working)
            if let google {
                Button {
                    working = true
                    Task {
                        let outcome = await google.idToken()
                        working = false
                        answered(.google, outcome)
                    }
                } label: {
                    Text(copy.button(.google))
                        .font(.body.weight(.semibold))
                        .frame(maxWidth: .infinity, minHeight: 48)
                }
                .buttonStyle(.plain)
                .foregroundStyle(Palette.accentInk)
                .background(Palette.accentStrong, in: RoundedRectangle(cornerRadius: 12))
                .disabled(working)
            }
            // A secondary choice: the page shows the extension's own email sign-in.
            Button { finish(.email) } label: {
                Text(copy.emailButton)
                    .font(.body.weight(.semibold))
                    .frame(maxWidth: .infinity, minHeight: 48)
            }
            .buttonStyle(.plain)
            .foregroundStyle(Palette.text)
            .background(Palette.panel, in: RoundedRectangle(cornerRadius: 12))
            .overlay(RoundedRectangle(cornerRadius: 12).stroke(Palette.border, lineWidth: 1))
            .disabled(working)
            if working {
                Text(copy.browserWaiting)
                    .foregroundStyle(Palette.muted)
            }
            if let failed {
                Text(copy.failure(failed))
                    .foregroundStyle(Palette.coral)
            }
            Button(copy.cancel) { finish(nil) }
                .buttonStyle(.plain)
                .foregroundStyle(Palette.accent)
        }
        .multilineTextAlignment(.center)
        .padding(28)
        .frame(maxWidth: 420)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Palette.background)
        .environment(\.locale, Locale(identifier: language.rawValue))
        .onDisappear { google?.cancel() }
    }

    /// A token closes the sheet with it; a cancel stays on the choice; a failure says so.
    private func answered(_ provider: SignInProvider, _ outcome: ProviderOutcome) {
        switch outcome {
        case .idToken:
            finish(.provider(provider, outcome))
        case .cancelled:
            failed = nil
        case .failed:
            failed = provider
        }
    }

    private func appleOutcome(_ result: Result<ASAuthorization, Error>) -> ProviderOutcome {
        switch result {
        case let .success(authorization):
            let credential = authorization.credential as? ASAuthorizationAppleIDCredential
            guard let data = credential?.identityToken, let token = String(data: data, encoding: .utf8) else {
                os_log(.error, "Sign in with Apple returned no identity token")
                return .failed
            }
            return .idToken(token)
        case let .failure(error):
            if (error as? ASAuthorizationError)?.code == .canceled { return .cancelled }
            os_log(.error, "Sign in with Apple failed: %{public}@", String(describing: error))
            return .failed
        }
    }
}

/// The Cymbra palette (apps/lingua-extension/src/styles/tokens.css).
private enum Palette {
    static let background = Color(hex: 0x0B1326)
    static let text = Color(hex: 0xDAE2FD)
    static let muted = Color(hex: 0x9AA1BA)
    static let accent = Color(hex: 0xD2BBFF)
    static let accentStrong = Color(hex: 0x7C3AED)
    static let accentInk = Color(hex: 0xEDE0FF)
    static let panel = Color(hex: 0x171F33)
    static let border = Color(hex: 0x2D3449)
    static let green = Color(hex: 0x4EDEA3)
    static let coral = Color(hex: 0xFFB4AB)
}

private extension Color {
    init(hex: UInt32) {
        self.init(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        )
    }
}
