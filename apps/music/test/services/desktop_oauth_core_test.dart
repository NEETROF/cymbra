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

import 'dart:convert';
import 'dart:math';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/desktop_oauth_core.dart';

void main() {
  group('PKCE', () {
    test('challenge is base64url(SHA-256(verifier)) with no padding', () {
      // RFC 7636 Appendix B test vector.
      const verifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
      const expected = 'E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM';
      expect(pkceChallengeFor(verifier), expected);
    });

    test('pkcePairFromBytes yields a matching verifier/challenge', () {
      final bytes = List<int>.generate(32, (i) => i);
      final pair = pkcePairFromBytes(bytes);
      expect(pair.verifier, base64UrlNoPad(bytes));
      expect(pair.challenge, pkceChallengeFor(pair.verifier));
      // Independently recompute the challenge from the verifier.
      final recomputed = base64UrlNoPad(
        sha256.convert(ascii.encode(pair.verifier)).bytes,
      );
      expect(pair.challenge, recomputed);
    });

    test('generatePkcePair uses the RNG (deterministic with a seed)', () {
      final a = generatePkcePair(Random(1));
      final b = generatePkcePair(Random(1));
      expect(a.verifier, b.verifier);
      expect(a.challenge, b.challenge);
      // No base64 padding leaks through.
      expect(a.verifier.contains('='), isFalse);
      expect(a.challenge.contains('='), isFalse);
    });
  });

  group('base64UrlNoPad', () {
    test('drops padding', () {
      expect(base64UrlNoPad([0]), 'AA'); // "AA==" without padding
      expect(base64UrlNoPad([0, 0]), 'AAA'); // "AAA=" without padding
    });
  });

  group('buildAuthorizationUrl', () {
    test('sets the required OAuth + PKCE params', () {
      final uri = buildAuthorizationUrl(
        clientId: 'client-123',
        redirectUri: 'http://127.0.0.1:1234/',
        state: 'st-abc',
        codeChallenge: 'chal-xyz',
      );
      expect(uri.origin + uri.path, kGoogleAuthEndpoint);
      final q = uri.queryParameters;
      expect(q['client_id'], 'client-123');
      expect(q['redirect_uri'], 'http://127.0.0.1:1234/');
      expect(q['response_type'], 'code');
      expect(q['scope'], 'openid email');
      expect(q['state'], 'st-abc');
      expect(q['code_challenge'], 'chal-xyz');
      expect(q['code_challenge_method'], 'S256');
      expect(q.containsKey('prompt'), isFalse);
    });

    test('adds prompt=select_account when forcing the chooser', () {
      final uri = buildAuthorizationUrl(
        clientId: 'c',
        redirectUri: 'http://127.0.0.1:1/',
        state: 's',
        codeChallenge: 'x',
        forceChooser: true,
      );
      expect(uri.queryParameters['prompt'], 'select_account');
    });

    test('honours custom scopes', () {
      final uri = buildAuthorizationUrl(
        clientId: 'c',
        redirectUri: 'http://127.0.0.1:1/',
        state: 's',
        codeChallenge: 'x',
        scopes: const ['openid', 'email', 'profile'],
      );
      expect(uri.queryParameters['scope'], 'openid email profile');
    });
  });

  group('generateState', () {
    test('is deterministic for a seeded RNG and unpadded', () {
      expect(generateState(Random(7)), generateState(Random(7)));
      expect(generateState(Random(7)).contains('='), isFalse);
    });
  });

  group('parseRedirect', () {
    Uri redirect(Map<String, String> q) =>
        Uri.parse('http://127.0.0.1:5555/').replace(queryParameters: q);

    test('returns the code on a valid, state-matching redirect', () {
      final r = parseRedirect(
        redirect({'state': 'good', 'code': 'auth-code-1'}),
        expectedState: 'good',
      );
      expect(r, isA<RedirectSuccess>());
      expect((r as RedirectSuccess).code, 'auth-code-1');
    });

    test('rejects a mismatched state before reading the code', () {
      final r = parseRedirect(
        redirect({'state': 'evil', 'code': 'auth-code-1'}),
        expectedState: 'good',
      );
      expect(r, isA<RedirectFailure>());
      expect((r as RedirectFailure).reason, 'state_mismatch');
    });

    test('rejects a missing state', () {
      final r = parseRedirect(
        redirect({'code': 'auth-code-1'}),
        expectedState: 'good',
      );
      expect((r as RedirectFailure).reason, 'state_mismatch');
    });

    test('surfaces an OAuth error param (state ok)', () {
      final r = parseRedirect(
        redirect({'state': 'good', 'error': 'access_denied'}),
        expectedState: 'good',
      );
      expect((r as RedirectFailure).reason, 'access_denied');
    });

    test('fails when the code is absent and there is no error', () {
      final r = parseRedirect(
        redirect({'state': 'good'}),
        expectedState: 'good',
      );
      expect((r as RedirectFailure).reason, 'missing_code');
    });
  });
}
