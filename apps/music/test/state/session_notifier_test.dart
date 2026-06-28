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
import 'package:music/services/grpc_client.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';

ProviderContainer makeContainer({
  required FakeTokenStore store,
  FakeAuthService? auth,
  FakeAccountService? account,
}) {
  final container = ProviderContainer(
    overrides: [
      tokenStoreProvider.overrideWithValue(store),
      authServiceProvider.overrideWithValue(auth ?? FakeAuthService()),
      accountServiceProvider.overrideWithValue(account ?? FakeAccountService()),
    ],
  );
  addTearDown(container.dispose);
  return container;
}

Account account({String? handle}) =>
    Account(userId: 'user-1', version: 1, handle: handle);

void main() {
  group('SessionNotifier hydration (task 3.5)', () {
    test('returning guest resolves to guest without backend calls', () async {
      final auth = FakeAuthService();
      final account = FakeAccountService();
      final c = makeContainer(
        store: FakeTokenStore(guest: true),
        auth: auth,
        account: account,
      );
      c.read(sessionNotifierProvider); // trigger build/hydrate
      await pumpEventQueue();

      expect(c.read(sessionNotifierProvider), isA<SessionGuest>());
      expect(account.calls, isEmpty); // no Cymbra ID call for a guest
    });

    test(
      'no stored session resolves to unauthenticated (entry screen)',
      () async {
        final c = makeContainer(store: FakeTokenStore());
        c.read(sessionNotifierProvider);
        await pumpEventQueue();
        expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
      },
    );

    test(
      'valid session resolves to authenticated and skips onboarding',
      () async {
        final c = makeContainer(
          store: FakeTokenStore(
            tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
          ),
          account: FakeAccountService(account: account(handle: 'alice')),
        );
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        final s = c.read(sessionNotifierProvider);
        expect(s, isA<SessionAuthenticated>());
        expect(s.needsHandle, isFalse);
      },
    );

    test('signed-in user without a handle needs onboarding', () async {
      final c = makeContainer(
        store: FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        ),
        account: FakeAccountService(account: account(handle: null)),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();
      expect(c.read(sessionNotifierProvider).needsHandle, isTrue);
    });

    test('failed refresh clears the session and routes to entry', () async {
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      final c = makeContainer(
        store: store,
        account: FakeAccountService(
          getError: const AuthException(AuthError.unauthenticated),
        ),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
      expect(store.tokens, isNull); // session cleared
    });

    test(
      'offline at startup keeps the user signed in (account unknown)',
      () async {
        final store = FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        );
        final c = makeContainer(
          store: store,
          account: FakeAccountService(
            getError: const AuthException(AuthError.unavailable),
          ),
        );
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        expect(c.read(sessionNotifierProvider), isA<SessionAuthenticated>());
        expect(store.tokens, isNotNull); // not punished for being offline
      },
    );
  });

  group('SessionNotifier transitions', () {
    test('continueAsGuest persists the choice', () async {
      final store = FakeTokenStore();
      final c = makeContainer(store: store);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(sessionNotifierProvider.notifier).continueAsGuest();
      expect(c.read(sessionNotifierProvider), isA<SessionGuest>());
      expect(store.guest, isTrue);
    });

    test('onSignedIn stores tokens and resolves the account', () async {
      final store = FakeTokenStore();
      final c = makeContainer(
        store: store,
        account: FakeAccountService(account: account(handle: 'bob')),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c
          .read(sessionNotifierProvider.notifier)
          .onSignedIn(const AuthTokens(accessToken: 'x', refreshToken: 'y'));
      expect(store.tokens?.accessToken, 'x');
      expect(c.read(sessionNotifierProvider), isA<SessionAuthenticated>());
    });

    test('signOut revokes online and clears locally', () async {
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      final auth = FakeAuthService();
      final c = makeContainer(
        store: store,
        auth: auth,
        account: FakeAccountService(account: account(handle: 'a')),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(sessionNotifierProvider.notifier).signOut();
      expect(auth.calls, contains('logout:r'));
      expect(store.tokens, isNull);
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });

    test(
      'abandonOnboarding deletes a brand-new (handle-less) account',
      () async {
        final store = FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        );
        final acct = FakeAccountService(account: account(handle: null));
        final c = makeContainer(store: store, account: acct);
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        await c.read(sessionNotifierProvider.notifier).abandonOnboarding();
        expect(acct.calls, contains('deleteAccount'));
        expect(store.tokens, isNull);
        expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
      },
    );

    test('abandonOnboarding only signs out an account with a handle', () async {
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      final auth = FakeAuthService();
      final acct = FakeAccountService(account: account(handle: 'alice'));
      final c = makeContainer(store: store, auth: auth, account: acct);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(sessionNotifierProvider.notifier).abandonOnboarding();
      expect(acct.calls, isNot(contains('deleteAccount')));
      expect(auth.calls, contains('logout:r'));
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });

    test('abandonOnboarding clears locally even if delete fails', () async {
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      final acct = FakeAccountService(
        account: account(handle: null),
        deleteError: const AuthException(AuthError.unavailable),
      );
      final c = makeContainer(store: store, account: acct);
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(sessionNotifierProvider.notifier).abandonOnboarding();
      expect(acct.calls, contains('deleteAccount'));
      expect(store.tokens, isNull); // local session cleared regardless
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });

    test('signOut while offline still clears the local session', () async {
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      final auth = FakeAuthService()
        ..logoutError = const AuthException(AuthError.unavailable);
      final c = makeContainer(
        store: store,
        auth: auth,
        account: FakeAccountService(account: account(handle: 'a')),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(sessionNotifierProvider.notifier).signOut();
      expect(store.tokens, isNull);
      expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
    });
  });

  group('guest gating guards (task 3.6)', () {
    test('isGuestSession / canUseOnlineServices reflect the session', () async {
      final c = makeContainer(store: FakeTokenStore(guest: true));
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      expect(c.read(isGuestSessionProvider), isTrue);
      expect(c.read(canUseOnlineServicesProvider), isFalse); // backend blocked
    });

    test('authenticated session may use online services', () async {
      final c = makeContainer(
        store: FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        ),
        account: FakeAccountService(account: account(handle: 'a')),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      expect(c.read(isGuestSessionProvider), isFalse);
      expect(c.read(canUseOnlineServicesProvider), isTrue);
    });
  });
}
