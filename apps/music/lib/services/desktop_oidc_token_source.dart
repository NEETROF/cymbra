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

// Desktop (Windows/Linux) Google sign-in via the OAuth 2.0 authorization-code +
// PKCE loopback flow (RFC 8252; change: add-desktop-oauth-loopback). This file
// holds the *flow orchestration* — the security-sensitive sequencing over the
// pure core (desktop_oauth_core.dart) — behind three injectable seams so it is
// covered by fast tests with no real browser/HttpServer/network (task 3.5). The
// concrete `dart:io`/`http`/`url_launcher` implementations live in
// desktop_oauth_io.dart, wired only by the production provider.

import 'dart:math';

import 'package:flutter/foundation.dart';

import 'desktop_oauth_core.dart';
import 'oidc_token_source.dart';

/// Opens [url] in the system browser. Returns true if the launch was accepted.
typedef BrowserLauncher = Future<bool> Function(Uri url);

/// A single-use loopback HTTP listener on `127.0.0.1` (design D5). Bound to an
/// ephemeral port, it captures exactly one browser redirect then shuts down.
abstract class LoopbackServer {
  /// Bind an ephemeral `127.0.0.1` port and return the `http://127.0.0.1:<port>/`
  /// redirect URI to hand to Google.
  Future<String> bind();

  /// The redirect URI, valid after [bind].
  String get redirectUri;

  /// Wait for the browser redirect (serving a "you can close this tab" page),
  /// or return null if [timeout] elapses first. Shuts the server down either way.
  Future<Uri?> waitForRedirect(Duration timeout);

  /// Stop the server and free the port. Idempotent; safe to call after
  /// [waitForRedirect] or to cancel a bound-but-unused server.
  Future<void> close();
}

/// Exchanges an authorization [code] (+ PKCE [verifier]) for tokens at Google's
/// token endpoint and returns the `id_token`. Throws [DesktopOauthException] if
/// the endpoint rejects the exchange or the response carries no `id_token` — a
/// real error the UI should surface, not a silent cancel.
abstract class OauthTokenExchanger {
  Future<String> exchangeCode({
    required String code,
    required String verifier,
    required String redirectUri,
  });
}

/// Desktop [OidcTokenSource] implementing the browser-loopback flow. All I/O is
/// injected ([serverFactory], [launch], [exchanger]) so the orchestration is
/// unit-tested with fakes; the production provider supplies the real glue.
///
/// A user cancellation — closing the browser (timeout) or denying consent
/// (`error=access_denied`) — resolves to **null**, a no-op like a dismissed
/// native sheet. Every *other* failure ([state mismatch, missing code, a browser
/// that won't open, a rejected token exchange]) throws [DesktopOauthException]
/// so the UI surfaces it instead of failing silently. A bad/missing `state` is
/// rejected before any code exchange (design D5).
class DesktopOidcTokenSource implements OidcTokenSource {
  final String clientId;
  final Duration timeout;
  final LoopbackServer Function() serverFactory;
  final BrowserLauncher launch;
  final OauthTokenExchanger exchanger;
  final Random _rng;

  DesktopOidcTokenSource({
    required this.clientId,
    required this.serverFactory,
    required this.launch,
    required this.exchanger,
    this.timeout = const Duration(minutes: 3),
    Random? rng,
  }) : _rng = rng ?? Random.secure();

  /// Configured (a client id is present) and running on a desktop platform the
  /// native SDK doesn't cover. macOS keeps the native flow; iOS/Android too.
  @override
  bool get googleAvailable =>
      clientId.isNotEmpty &&
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.windows ||
          defaultTargetPlatform == TargetPlatform.linux);

  /// Apple is out of scope on desktop (its web flow needs a hosted return URL).
  @override
  bool get appleAvailable => false;

  @override
  Future<String?> appleIdToken() async => null;

  @override
  Future<String?> googleIdToken({bool forceChooser = false}) async {
    if (!googleAvailable) return null; // not configured — caller guards too
    final server = serverFactory();
    try {
      final redirectUri = await server.bind();
      final pkce = generatePkcePair(_rng);
      final state = generateState(_rng);
      final authUrl = buildAuthorizationUrl(
        clientId: clientId,
        redirectUri: redirectUri,
        state: state,
        codeChallenge: pkce.challenge,
        forceChooser: forceChooser,
      );
      final launched = await launch(authUrl);
      if (!launched) throw const DesktopOauthException('browser_launch_failed');

      final redirect = await server.waitForRedirect(timeout);
      if (redirect == null) return null; // timeout / user closed the browser

      final result = parseRedirect(redirect, expectedState: state);
      switch (result) {
        case RedirectSuccess(:final code):
          // Throws on a rejected exchange; otherwise yields the id_token.
          return await exchanger.exchangeCode(
            code: code,
            verifier: pkce.verifier,
            redirectUri: redirectUri,
          );
        case RedirectFailure(:final reason):
          // Denied consent is a user cancel (no-op); anything else is a real
          // error surfaced to the UI. Neither exchanges the code (design D5).
          if (reason == 'access_denied') return null;
          throw DesktopOauthException(reason);
      }
    } finally {
      await server.close(); // single-use; free the port on every path
    }
  }

  /// The loopback flow keeps no persistent session of its own — the browser's
  /// Google cookies are outside the app, and `forceChooser` re-prompts. Nothing
  /// to clear, so this is a no-op (the backend session is cleared elsewhere).
  @override
  Future<void> signOut() async {}
}
