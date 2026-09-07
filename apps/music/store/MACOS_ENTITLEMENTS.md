# macOS entitlements — why `network.server` is there

Apple's automated submission analysis flags
`com.apple.security.network.server` on every macOS submission and asks either to
remove it or to justify it. It cannot be removed: Google sign-in on macOS stops
working. Paste the text below into the Resolution Center reply **and** into the
App Review Information notes of the macOS platform, as the message asks.

## Reply text

> Cymbra Music offers two sign-in methods: Sign in with Apple, and Google.
>
> Google Sign-In on macOS uses the GoogleSignIn SDK, which is built on AppAuth.
> AppAuth receives the OAuth 2.0 authorization response through a loopback
> redirect: it opens a listener on 127.0.0.1 and waits for the browser to redirect
> to it, as described in RFC 8252, "OAuth 2.0 for Native Apps". Under the App
> Sandbox that bind is refused without com.apple.security.network.server, and the
> flow fails before the authorization window can appear.
>
> This is measured, not assumed. In a signed, sandboxed release build carrying only
> com.apple.security.network.client, "Continue with Google" shows a spinner and
> then a generic failure, and no authorization window ever opens. Adding
> com.apple.security.network.server — the only change between the two builds — lets
> the same build open the window and complete sign-in.
>
> The listener is bound to the loopback interface only and exists for the duration
> of a sign-in. The app runs no server, accepts no connection originating outside
> the machine, and opens no other listening socket. Sign in with Apple does not use
> it. The App Sandbox offers no narrower capability for loopback-only listening, so
> removing the entitlement would mean removing Google sign-in from the macOS app.

## What NOT to do

Do not "fix" the flag by deleting it from `Release.entitlements`. That was the
state before PR #252 and it ships a macOS build whose Google sign-in fails
silently — spinner, generic error, no window — which is far worse than answering
this message. `DebugProfile.entitlements` carries the same entitlement, so a
`flutter run -d macos --debug` will *not* reproduce the failure; use
`--release` (see `macos-google-signin-network-server` in the project notes).
