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

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/desktop_oidc_token_source.dart';

/// Records the launched authorization URL and returns a scripted launch result.
class _FakeLauncher {
  Uri? launchedUrl;
  final bool result;
  _FakeLauncher({this.result = true});

  Future<bool> call(Uri url) async {
    launchedUrl = url;
    return result;
  }
}

/// Loopback server whose redirect is produced by [onWait] (evaluated lazily, so
/// it can read the `state` the flow just put on the launched URL). Tracks the
/// single-use bind/close lifecycle.
class _FakeLoopbackServer implements LoopbackServer {
  final Future<Uri?> Function() onWait;
  int bindCount = 0;
  int closeCount = 0;

  _FakeLoopbackServer(this.onWait);

  @override
  Future<String> bind() async {
    bindCount++;
    return redirectUri;
  }

  @override
  String get redirectUri => 'http://127.0.0.1:4321/';

  @override
  Future<Uri?> waitForRedirect(Duration timeout) => onWait();

  @override
  Future<void> close() async => closeCount++;
}

/// Records exchange calls and returns a scripted id_token.
class _FakeExchanger implements OauthTokenExchanger {
  final String? idToken;
  final List<Map<String, String>> calls = [];
  _FakeExchanger(this.idToken);

  @override
  Future<String?> exchangeCode({
    required String code,
    required String verifier,
    required String redirectUri,
  }) async {
    calls.add({'code': code, 'verifier': verifier, 'redirect': redirectUri});
    return idToken;
  }
}

void main() {
  setUp(() => debugDefaultTargetPlatformOverride = TargetPlatform.windows);
  tearDown(() => debugDefaultTargetPlatformOverride = null);

  // Build a source + its collaborators; [redirect] maps the launched URL's state
  // to the browser redirect the loopback server "receives".
  ({
    DesktopOidcTokenSource source,
    _FakeLauncher launcher,
    _FakeExchanger exch,
    _FakeLoopbackServer server,
  })
  build({
    String clientId = 'desktop-client',
    String? exchangedToken = 'id-token-xyz',
    bool launchResult = true,
    required Uri? Function(String? state) redirect,
  }) {
    final launcher = _FakeLauncher(result: launchResult);
    final exch = _FakeExchanger(exchangedToken);
    late _FakeLoopbackServer server;
    server = _FakeLoopbackServer(() async {
      final state = launcher.launchedUrl?.queryParameters['state'];
      return redirect(state);
    });
    final source = DesktopOidcTokenSource(
      clientId: clientId,
      serverFactory: () => server,
      launch: launcher.call,
      exchanger: exch,
      timeout: const Duration(milliseconds: 10),
    );
    return (source: source, launcher: launcher, exch: exch, server: server);
  }

  group('availability', () {
    test('googleAvailable on configured Windows/Linux, Apple never', () {
      final t = build(redirect: (_) => null);
      expect(t.source.googleAvailable, isTrue);
      expect(t.source.appleAvailable, isFalse);

      debugDefaultTargetPlatformOverride = TargetPlatform.linux;
      expect(t.source.googleAvailable, isTrue);
    });

    test('hidden when no client id is configured', () {
      final t = build(clientId: '', redirect: (_) => null);
      expect(t.source.googleAvailable, isFalse);
    });

    test('hidden on non-desktop platforms', () {
      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      final t = build(redirect: (_) => null);
      expect(t.source.googleAvailable, isFalse);
    });

    test('appleIdToken is always null; signOut is a no-op', () async {
      final t = build(redirect: (_) => null);
      expect(await t.source.appleIdToken(), isNull);
      await t.source.signOut(); // must not throw
    });
  });

  group('googleIdToken', () {
    test('success: opens browser, matches state, exchanges the code', () async {
      final t = build(
        redirect: (state) =>
            Uri.parse('http://127.0.0.1:4321/?state=$state&code=auth-1'),
      );

      final token = await t.source.googleIdToken();

      expect(token, 'id-token-xyz');
      // The browser was opened with the PKCE + OAuth params.
      final url = t.launcher.launchedUrl!;
      expect(url.queryParameters['client_id'], 'desktop-client');
      expect(url.queryParameters['code_challenge_method'], 'S256');
      expect(url.queryParameters.containsKey('prompt'), isFalse);
      // The captured code was exchanged; server was bound then closed.
      expect(t.exch.calls.single['code'], 'auth-1');
      expect(t.server.bindCount, 1);
      expect(t.server.closeCount, 1);
    });

    test('forceChooser adds prompt=select_account', () async {
      final t = build(
        redirect: (state) =>
            Uri.parse('http://127.0.0.1:4321/?state=$state&code=c'),
      );
      await t.source.googleIdToken(forceChooser: true);
      expect(
        t.launcher.launchedUrl!.queryParameters['prompt'],
        'select_account',
      );
    });

    test('user cancels (browser closed) → null, no exchange', () async {
      final t = build(
        redirect: (state) => Uri.parse(
          'http://127.0.0.1:4321/?state=$state&error=access_denied',
        ),
      );
      expect(await t.source.googleIdToken(), isNull);
      expect(t.exch.calls, isEmpty);
      expect(t.server.closeCount, 1);
    });

    test('timeout (no redirect) → null, no exchange, port freed', () async {
      final t = build(redirect: (_) => null);
      expect(await t.source.googleIdToken(), isNull);
      expect(t.exch.calls, isEmpty);
      expect(t.server.closeCount, 1);
    });

    test('state mismatch → null and code is never exchanged', () async {
      final t = build(
        redirect: (_) =>
            Uri.parse('http://127.0.0.1:4321/?state=forged&code=auth-1'),
      );
      expect(await t.source.googleIdToken(), isNull);
      expect(t.exch.calls, isEmpty);
    });

    test('browser launch refused → null, no exchange', () async {
      final t = build(
        launchResult: false,
        redirect: (state) =>
            Uri.parse('http://127.0.0.1:4321/?state=$state&code=c'),
      );
      expect(await t.source.googleIdToken(), isNull);
      expect(t.exch.calls, isEmpty);
      expect(t.server.closeCount, 1);
    });

    test('unconfigured source short-circuits to null', () async {
      final t = build(clientId: '', redirect: (_) => null);
      expect(await t.source.googleIdToken(), isNull);
      expect(t.launcher.launchedUrl, isNull);
    });
  });
}
