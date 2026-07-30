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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/l10n/gen/app_localizations_en.dart';
import 'package:music/screens/auth/auth_messages.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/state/connected_accounts_state.dart';

void main() {
  final l10n = AppLocalizationsEn();

  group('authErrorMessage — UNAUTHENTICATED is no longer the password copy', () {
    test('the default for UNAUTHENTICATED is the neutral sign-in message', () {
      final msg = authErrorMessage(
        l10n,
        const AuthException(AuthError.unauthenticated),
      );
      expect(msg, l10n.authErrSignInFailed);
      expect(msg, isNot(l10n.authErrUnauthenticated)); // not "Incorrect email…"
    });

    test('the local sign-in flow still reads the password copy via fallback', () {
      final msg = authErrorMessage(
        l10n,
        const AuthException(AuthError.unauthenticated),
        fallback: l10n.authErrUnauthenticated,
      );
      expect(msg, 'Incorrect email or password.');
    });
  });

  group('linkErrorMessage — action-aware link/unlink copy', () {
    test('a social-link collision reads "already linked elsewhere"', () {
      final msg = linkErrorMessage(
        l10n,
        const AuthException(AuthError.alreadyExists),
        ConnectedAccountsAction.linkGoogle,
      );
      expect(msg, l10n.authErrAlreadyLinkedElsewhere);
    });

    test('a "set password" collision reads "email already in use"', () {
      final msg = linkErrorMessage(
        l10n,
        const AuthException(AuthError.alreadyExists),
        ConnectedAccountsAction.setPassword,
      );
      expect(msg, l10n.authErrAlreadyExists);
    });

    test('a refused unlink reads the last-identity copy, not "unverified"', () {
      final msg = linkErrorMessage(
        l10n,
        const AuthException(AuthError.failedPrecondition),
        ConnectedAccountsAction.unlink,
      );
      expect(msg, l10n.authErrOnlySignInMethod);
      expect(msg, isNot(l10n.authErrUnverified));
    });
  });
}
