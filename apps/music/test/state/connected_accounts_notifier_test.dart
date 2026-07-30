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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/state/connected_accounts_notifier.dart';
import 'package:music/state/connected_accounts_state.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';

LinkedIdentity _identity(String provider) => LinkedIdentity(
  provider: provider,
  subject: '$provider-sub',
  linkedAt: DateTime(2026, 1, 2),
);

/// A container with the Cymbra ID seams faked, keeping a live listener on the
/// (autoDispose) notifier so its `build()`-scheduled load isn't disposed away.
ProviderContainer _live({
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeOidcTokenSource? oidc,
}) {
  final c = authContainer(auth: auth, account: account, oidc: oidc);
  c.listen(connectedAccountsNotifierProvider, (_, _) {});
  return c;
}

void main() {
  group('ConnectedAccountsNotifier', () {
    test('load populates the identity list', () async {
      final account = FakeAccountService(
        identities: [_identity(LinkedIdentity.providerLocal)],
      );
      final c = _live(account: account);

      c.read(connectedAccountsNotifierProvider);
      await pumpEventQueue();

      final state = c.read(connectedAccountsNotifierProvider);
      expect(state.identities.hasValue, isTrue);
      expect(state.items.single.provider, LinkedIdentity.providerLocal);
      expect(state.hasLocal, isTrue);
      expect(state.hasGoogle, isFalse);
      expect(state.isLastIdentity, isTrue);
    });

    test('linkGoogle mints, attaches, and re-fetches the list', () async {
      final auth = FakeAuthService();
      final account = FakeAccountService(
        identities: [_identity(LinkedIdentity.providerLocal)],
      );
      final c = _live(auth: auth, account: account);
      final notifier = c.read(connectedAccountsNotifierProvider.notifier);
      await pumpEventQueue();

      // Model the server adding Google, returned by the refetch after linking.
      account.identities = [
        _identity(LinkedIdentity.providerLocal),
        _identity(LinkedIdentity.providerGoogle),
      ];
      await notifier.linkGoogle();

      final state = c.read(connectedAccountsNotifierProvider);
      expect(auth.calls, contains('linkIdentity:google-id-token'));
      // Two list fetches: initial load + refetch after the mutation.
      expect(account.calls.where((s) => s == 'listIdentities').length, 2);
      expect(state.hasGoogle, isTrue);
      expect(state.actionError, isNull);
    });

    test('a cancelled provider sheet is a silent no-op', () async {
      final auth = FakeAuthService();
      final oidc = FakeOidcTokenSource(googleToken: null); // user dismissed
      final account = FakeAccountService(
        identities: [_identity(LinkedIdentity.providerLocal)],
      );
      final c = _live(auth: auth, account: account, oidc: oidc);
      final notifier = c.read(connectedAccountsNotifierProvider.notifier);
      await pumpEventQueue();

      await notifier.linkGoogle();

      final state = c.read(connectedAccountsNotifierProvider);
      expect(auth.calls.where((s) => s.startsWith('linkIdentity')), isEmpty);
      expect(state.actionError, isNull);
      // No refetch — the list is unchanged.
      expect(account.calls.where((s) => s == 'listIdentities').length, 1);
    });

    test('ALREADY_EXISTS surfaces as an action error, list untouched', () async {
      final auth = FakeAuthService()
        ..linkErrors.add(const AuthException(AuthError.alreadyExists));
      final account = FakeAccountService(
        identities: [_identity(LinkedIdentity.providerLocal)],
      );
      final c = _live(auth: auth, account: account);
      final notifier = c.read(connectedAccountsNotifierProvider.notifier);
      await pumpEventQueue();

      await notifier.linkGoogle();

      final state = c.read(connectedAccountsNotifierProvider);
      expect(state.actionError?.error, AuthError.alreadyExists);
      expect(state.lastAction, ConnectedAccountsAction.linkGoogle);
      expect(state.hasGoogle, isFalse);
      // No refetch on failure.
      expect(account.calls.where((s) => s == 'listIdentities').length, 1);
    });

    test('unlink removes an identity and re-fetches', () async {
      final auth = FakeAuthService();
      final account = FakeAccountService(
        identities: [
          _identity(LinkedIdentity.providerLocal),
          _identity(LinkedIdentity.providerGoogle),
        ],
      );
      final c = _live(auth: auth, account: account);
      final notifier = c.read(connectedAccountsNotifierProvider.notifier);
      await pumpEventQueue();

      account.identities = [_identity(LinkedIdentity.providerLocal)];
      await notifier.unlink(
        provider: LinkedIdentity.providerGoogle,
        subject: 'google-sub',
      );

      final state = c.read(connectedAccountsNotifierProvider);
      expect(auth.calls, contains('unlinkIdentity:google:google-sub'));
      expect(state.hasGoogle, isFalse);
      expect(state.items.single.provider, LinkedIdentity.providerLocal);
    });

    test('the last identity refusal is captured as an action error', () async {
      final auth = FakeAuthService()
        ..unlinkError = const AuthException(AuthError.failedPrecondition);
      final account = FakeAccountService(
        identities: [_identity(LinkedIdentity.providerGoogle)],
      );
      final c = _live(auth: auth, account: account);
      final notifier = c.read(connectedAccountsNotifierProvider.notifier);
      await pumpEventQueue();

      await notifier.unlink(
        provider: LinkedIdentity.providerGoogle,
        subject: 'google-sub',
      );

      final state = c.read(connectedAccountsNotifierProvider);
      expect(state.actionError?.error, AuthError.failedPrecondition);
      expect(state.lastAction, ConnectedAccountsAction.unlink);
    });

    test('linkEmailPassword calls setLocalCredential then re-fetches', () async {
      final auth = FakeAuthService();
      final account = FakeAccountService(
        identities: [_identity(LinkedIdentity.providerGoogle)],
      );
      final c = _live(auth: auth, account: account);
      final notifier = c.read(connectedAccountsNotifierProvider.notifier);
      await pumpEventQueue();

      account.identities = [
        _identity(LinkedIdentity.providerGoogle),
        _identity(LinkedIdentity.providerLocal),
      ];
      await notifier.linkEmailPassword(
        email: 'me@x.dev',
        password: 'longenoughpassword',
      );

      final state = c.read(connectedAccountsNotifierProvider);
      expect(auth.calls, contains('setLocalCredential:me@x.dev'));
      expect(state.hasLocal, isTrue);
      expect(state.actionError, isNull);
    });
  });
}
