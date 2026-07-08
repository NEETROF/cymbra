// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Build-time OIDC configuration, supplied with `--dart-define` (tasks 6.3/6.4).
//
// Both are EMPTY/OFF by default, which hides the corresponding entry button so
// the native SDK is never invoked unconfigured — otherwise Google's SDK throws
// an uncatchable native exception ("GIDClientID is set in Info.plist") that
// terminates the app. Provide the values once the OAuth credentials exist:
//
//   flutter run -d macos \
//     --dart-define=GOOGLE_CLIENT_ID=<ios-client>.apps.googleusercontent.com \
//     --dart-define=GOOGLE_SERVER_CLIENT_ID=<web-client>.apps.googleusercontent.com \
//     --dart-define=APPLE_SIGN_IN_ENABLED=true
//
// Option A: serverClientId (the web client) is passed on every platform, so the
// id_token audience is always the web client — set the backend's
// CYMBRA_GOOGLE_AUDIENCE to that web client id. Android needs only
// GOOGLE_SERVER_CLIENT_ID; iOS/macOS also need GOOGLE_CLIENT_ID (+ the reversed
// URL scheme). See the app README for the per-platform steps.

/// Google OAuth **iOS** client ID for the native sign-in SDK on Apple platforms
/// (empty ⇒ Google hidden on iOS/macOS). Not used on Android.
const String kGoogleClientId = String.fromEnvironment('GOOGLE_CLIENT_ID');

/// Google **server** (Web) OAuth client ID, passed as `serverClientId`. It sets
/// the id_token audience on every platform (so the backend trusts a single
/// audience) and is required on Android to obtain an id_token at all (empty ⇒
/// Google hidden on Android).
const String kGoogleServerClientId = String.fromEnvironment(
  'GOOGLE_SERVER_CLIENT_ID',
);

/// Whether Sign in with Apple is enabled (requires the "Sign in with Apple"
/// capability + a development certificate; off by default).
const bool kAppleSignInEnabled = bool.fromEnvironment('APPLE_SIGN_IN_ENABLED');

// Desktop (Windows/Linux) Google sign-in via the browser-loopback OAuth flow
// (change: add-desktop-oauth-loopback). The native google_sign_in plugin has no
// Windows/Linux implementation, so those platforms use the RFC 8252 flow instead.
// Empty client id ⇒ Google stays hidden on desktop (as today).
//
//   flutter run -d windows \
//     --dart-define=DESKTOP_GOOGLE_CLIENT_ID=<client>.apps.googleusercontent.com \
//     [--dart-define=DESKTOP_GOOGLE_CLIENT_SECRET=<secret>]   # only for a Web client
//
// Client-type note (design D3, unresolved): a **Desktop app** OAuth client is a
// public client — leave the secret empty (pure PKCE), but its audience differs
// from the web client, so the backend must accept it. A **Web** client keeps the
// single audience (Option A) but is confidential — its secret must be supplied
// here and is then shipped in the binary (a security trade-off). The secret is
// sent on the token exchange only when non-empty, so the same build serves both.

/// Google OAuth client ID for the desktop browser-loopback flow (Windows/Linux).
/// Empty ⇒ Google hidden on desktop.
const String kDesktopGoogleClientId = String.fromEnvironment(
  'DESKTOP_GOOGLE_CLIENT_ID',
);

/// Optional client secret for the desktop token exchange. Required only when the
/// chosen OAuth client is a *Web application* client (confidential); leave empty
/// for a *Desktop app* client (public, PKCE-only).
const String kDesktopGoogleClientSecret = String.fromEnvironment(
  'DESKTOP_GOOGLE_CLIENT_SECRET',
);
