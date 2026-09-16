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
/// Safari extension through `cymbra-lingua://signin?provider=…`. It runs the requested
/// provider's native sheet and hands the id_token to the extension through the App Group;
/// the Cymbra session itself stays in the extension.
struct SignInView: View {
    let requested: SignInProvider
    let google: GoogleIdTokenSource?
    let handoff: IdTokenHandoff?
    let close: () -> Void

    @State private var phase: SignInPhase = .choosing
    @State private var working = false

    var body: some View {
        VStack(spacing: 18) {
            Text(SignInCopy.heading)
                .font(.title2.bold())
                .foregroundStyle(Palette.text)
            if phase == .done {
                Text(SignInCopy.done)
                    .foregroundStyle(Palette.green)
                Button(SignInCopy.close, action: close)
                    .buttonStyle(.plain)
                    .foregroundStyle(Palette.accent)
            } else {
                Text(SignInCopy.lede)
                    .foregroundStyle(Palette.muted)
                providerButton
                    .disabled(working)
                if working {
                    Text(SignInCopy.browserWaiting)
                        .foregroundStyle(Palette.muted)
                }
                if case let .failed(provider) = phase {
                    Text(SignInCopy.failure(provider))
                        .foregroundStyle(Palette.coral)
                }
                Button(SignInCopy.cancel, action: close)
                    .buttonStyle(.plain)
                    .foregroundStyle(Palette.accent)
            }
        }
        .multilineTextAlignment(.center)
        .padding(28)
        .frame(maxWidth: 420)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Palette.background)
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
                    Text(SignInCopy.button(.google))
                        .font(.body.weight(.semibold))
                        .frame(maxWidth: .infinity, minHeight: 48)
                }
                .buttonStyle(.plain)
                .foregroundStyle(Palette.accentInk)
                .background(Palette.accentStrong, in: RoundedRectangle(cornerRadius: 12))
            } else {
                Text(SignInCopy.unavailable(.google))
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

/// The Cymbra palette (apps/lingua-extension/src/styles/tokens.css).
private enum Palette {
    static let background = Color(hex: 0x0B1326)
    static let text = Color(hex: 0xDAE2FD)
    static let muted = Color(hex: 0x9AA1BA)
    static let accent = Color(hex: 0xD2BBFF)
    static let accentStrong = Color(hex: 0x7C3AED)
    static let accentInk = Color(hex: 0xEDE0FF)
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
