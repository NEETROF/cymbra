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

import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/oidc_token_source.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/session_state.dart';

import '../support/auth_fakes.dart';

ProviderContainer makeContainer({
  required FakeTokenStore store,
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeOidcTokenSource? oidc,
}) {
  final container = ProviderContainer(
    overrides: [
      tokenStoreProvider.overrideWithValue(store),
      authServiceProvider.overrideWithValue(auth ?? FakeAuthService()),
      accountServiceProvider.overrideWithValue(account ?? FakeAccountService()),
      oidcTokenSourceProvider.overrideWithValue(oidc ?? FakeOidcTokenSource()),
    ],
  );
  addTearDown(container.dispose);
  return container;
}

Account account({String? handle}) =>
    Account(userId: 'user-1', version: 1, handle: handle);

/// A transient (offline / deadline-exceeded) `GetAccount` failure — the one that
/// leaves the session degraded instead of signing the user out.
const _transient = AuthException(AuthError.unavailable, 'offline');

FakeTokenStore _signedInStore() => FakeTokenStore(
  tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
);

/// [FakeAccountService] whose *first* `getAccount` fails transiently and whose
/// later ones hang until the test completes them — the only way to hold the
/// single-flight window open and observe it.
class _HangingAfterFailure extends FakeAccountService {
  _HangingAfterFailure({super.account});

  final List<Completer<Account>> pending = [];
  int attempts = 0;

  @override
  Future<Account> getAccount() {
    attempts++;
    if (attempts == 1) return Future.error(_transient);
    final completer = Completer<Account>();
    pending.add(completer);
    return completer.future;
  }
}

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
      'a deleted account (notFound) clears the session and routes to entry',
      () async {
        // The stored token still verifies by signature, but the account row is
        // gone server-side → GetAccount returns notFound. Must not stay stuck
        // "signed in with unknown account".
        final store = FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        );
        final c = makeContainer(
          store: store,
          account: FakeAccountService(
            getError: const AuthException(AuthError.notFound),
          ),
        );
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
        expect(store.tokens, isNull); // session cleared
      },
    );

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

    test('signOut also forgets the cached Google account', () async {
      final oidc = FakeOidcTokenSource();
      final c = makeContainer(
        store: FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        ),
        account: FakeAccountService(account: account(handle: 'a')),
        oidc: oidc,
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await c.read(sessionNotifierProvider.notifier).signOut();
      expect(oidc.calls, contains('oidcSignOut'));
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

    test(
      'signOutEverywhere revokes all sessions then clears locally',
      () async {
        final store = FakeTokenStore(
          tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
        );
        final auth = FakeAuthService();
        final oidc = FakeOidcTokenSource();
        final c = makeContainer(
          store: store,
          auth: auth,
          account: FakeAccountService(account: account(handle: 'a')),
          oidc: oidc,
        );
        c.read(sessionNotifierProvider);
        await pumpEventQueue();

        await c.read(sessionNotifierProvider.notifier).signOutEverywhere();

        expect(auth.calls, contains('revokeAllSessions'));
        expect(oidc.calls, contains('oidcSignOut')); // shared teardown ran
        expect(store.tokens, isNull); // current device signed out immediately
        expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());
      },
    );

    test('a failed revoke keeps the user signed in (tokens kept)', () async {
      final store = FakeTokenStore(
        tokens: const StoredTokens(accessToken: 'a', refreshToken: 'r'),
      );
      final auth = FakeAuthService()
        ..revokeAllError = const AuthException(AuthError.unavailable);
      final c = makeContainer(
        store: store,
        auth: auth,
        account: FakeAccountService(account: account(handle: 'a')),
      );
      c.read(sessionNotifierProvider);
      await pumpEventQueue();

      await expectLater(
        c.read(sessionNotifierProvider.notifier).signOutEverywhere(),
        throwsA(isA<AuthException>()),
      );

      // Session preserved: not torn down locally while other devices stay live.
      expect(store.tokens, isNotNull);
      expect(c.read(sessionNotifierProvider), isA<SessionAuthenticated>());
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

  group('degraded-session recovery (change: fix-session-account-retry)', () {
    test('a transient failure at sign-in recovers with no user action', () {
      fakeAsync((fa) {
        final acct = FakeAccountService(account: account(handle: 'ada'))
          ..getErrors.add(_transient);
        final c = makeContainer(store: FakeTokenStore(), account: acct);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();

        unawaited(
          c
              .read(sessionNotifierProvider.notifier)
              .onSignedIn(
                const AuthTokens(accessToken: 'a', refreshToken: 'r'),
              ),
        );
        fa.flushMicrotasks();

        // Signed in, but with no identity: the state the user was stranded in.
        expect(c.read(sessionNotifierProvider).accountUnresolved, isTrue);
        expect(c.read(currentUserIdProvider), isNull);
        expect(c.read(currentUserHandleProvider), isNull);

        fa.elapse(kAccountRetryInitialDelay);
        fa.flushMicrotasks();

        expect(c.read(sessionNotifierProvider).accountUnresolved, isFalse);
        expect(c.read(currentUserHandleProvider), 'ada');
        expect(c.read(currentUserIdProvider), 'user-1');
      });
    });

    test('a transient failure at launch recovers with no user action', () {
      fakeAsync((fa) {
        final acct = FakeAccountService(account: account(handle: 'ada'))
          ..getErrors.add(_transient);
        final c = makeContainer(store: _signedInStore(), account: acct);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();

        expect(c.read(sessionNotifierProvider).accountUnresolved, isTrue);

        fa.elapse(kAccountRetryInitialDelay);
        fa.flushMicrotasks();

        expect(c.read(currentUserHandleProvider), 'ada');
      });
    });

    test('successive failures back off, doubling up to the cap', () {
      fakeAsync((fa) {
        final acct = FakeAccountService(account: account(handle: 'ada'))
          ..getErrors.addAll(List.filled(12, _transient));
        final c = makeContainer(store: _signedInStore(), account: acct);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();
        expect(acct.getAccountCalls, 1, reason: 'the initial resolution');

        // 2s, 4s, 8s, 16s, 32s, then pinned at the 60s cap.
        for (final seconds in [2, 4, 8, 16, 32, 60, 60]) {
          final before = acct.getAccountCalls;
          fa.elapse(Duration(seconds: seconds) - const Duration(seconds: 1));
          fa.flushMicrotasks();
          expect(
            acct.getAccountCalls,
            before,
            reason: 'nothing fires before the ${seconds}s delay',
          );

          fa.elapse(const Duration(seconds: 1));
          fa.flushMicrotasks();
          expect(
            acct.getAccountCalls,
            before + 1,
            reason: 'exactly one attempt at the ${seconds}s delay',
          );
        }
      });
    });

    test('no retry is issued while the app is out of the foreground', () {
      fakeAsync((fa) {
        final acct = FakeAccountService(account: account(handle: 'ada'))
          ..getErrors.addAll(List.filled(12, _transient));
        final c = makeContainer(store: _signedInStore(), account: acct);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();
        expect(acct.getAccountCalls, 1);

        c.read(sessionNotifierProvider.notifier).onBackground();
        fa.elapse(const Duration(minutes: 10));
        fa.flushMicrotasks();

        expect(
          acct.getAccountCalls,
          1,
          reason:
              'a backgrounded desktop app keeps running — it must not '
              'keep spending RPCs unseen',
        );
        expect(
          c.read(sessionNotifierProvider).accountUnresolved,
          isTrue,
          reason: 'backgrounding never signs anyone out',
        );
      });
    });

    test(
      'returning to the foreground retries at once and resets the backoff',
      () {
        fakeAsync((fa) {
          final acct = FakeAccountService(account: account(handle: 'ada'))
            ..getErrors.addAll(List.filled(12, _transient));
          final c = makeContainer(store: _signedInStore(), account: acct);
          final notifier = c.read(sessionNotifierProvider.notifier);
          c.read(sessionNotifierProvider);
          fa.flushMicrotasks();

          // Let the backoff grow past its first step (2s, then 4s).
          fa.elapse(const Duration(seconds: 2));
          fa.flushMicrotasks();
          fa.elapse(const Duration(seconds: 4));
          fa.flushMicrotasks();
          expect(acct.getAccountCalls, 3);

          notifier.onBackground();
          fa.elapse(const Duration(minutes: 10));
          fa.flushMicrotasks();
          expect(acct.getAccountCalls, 3);

          unawaited(notifier.onForeground());
          fa.flushMicrotasks();
          expect(
            acct.getAccountCalls,
            4,
            reason: 'a foreground return attempts immediately',
          );

          // Reset: the next attempt is the FIRST delay away, not the 8s the
          // backoff had climbed to before backgrounding.
          fa.elapse(kAccountRetryInitialDelay);
          fa.flushMicrotasks();
          expect(acct.getAccountCalls, 5, reason: 'the backoff was reset');
        });
      },
    );

    test(
      'returning to the foreground with a resolved account issues nothing',
      () {
        fakeAsync((fa) {
          final acct = FakeAccountService(account: account(handle: 'ada'));
          final c = makeContainer(store: _signedInStore(), account: acct);
          c.read(sessionNotifierProvider);
          fa.flushMicrotasks();
          expect(acct.getAccountCalls, 1);

          unawaited(c.read(sessionNotifierProvider.notifier).onForeground());
          fa.flushMicrotasks();

          expect(acct.getAccountCalls, 1);
        });
      },
    );

    test('concurrent triggers coalesce into one GetAccount', () {
      fakeAsync((fa) {
        final acct = _HangingAfterFailure(account: account(handle: 'ada'));
        final c = makeContainer(store: _signedInStore(), account: acct);
        final notifier = c.read(sessionNotifierProvider.notifier);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();
        expect(acct.attempts, 1, reason: 'the initial resolution failed');

        // The scheduled re-attempt fires and hangs: the single-flight window.
        fa.elapse(kAccountRetryInitialDelay);
        fa.flushMicrotasks();
        expect(acct.attempts, 2);
        expect(acct.pending, hasLength(1));

        // The user taps retry and the app foregrounds, both mid-flight.
        unawaited(notifier.refreshAccount());
        unawaited(notifier.onForeground());
        fa.flushMicrotasks();

        expect(
          acct.attempts,
          2,
          reason: 'both triggers join the in-flight resolution',
        );

        acct.pending.single.complete(account(handle: 'ada'));
        fa.flushMicrotasks();
        expect(c.read(currentUserHandleProvider), 'ada');
      });
    });

    test('signing out cancels a scheduled retry', () {
      fakeAsync((fa) {
        final acct = FakeAccountService(account: account(handle: 'ada'))
          ..getErrors.addAll(List.filled(12, _transient));
        final c = makeContainer(store: _signedInStore(), account: acct);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();
        expect(acct.getAccountCalls, 1);

        unawaited(c.read(sessionNotifierProvider.notifier).signOut());
        fa.flushMicrotasks();
        expect(c.read(sessionNotifierProvider), isA<SessionUnauthenticated>());

        final afterSignOut = acct.getAccountCalls;
        fa.elapse(const Duration(minutes: 10));
        fa.flushMicrotasks();

        expect(
          acct.getAccountCalls,
          afterSignOut,
          reason: 'no retry outlives the session it was retrying',
        );
      });
    });

    test('a terminal failure inside a retry signs out and stops the loop', () {
      fakeAsync((fa) {
        final acct = FakeAccountService(account: account(handle: 'ada'))
          ..getErrors.addAll(const [
            _transient,
            AuthException(AuthError.unauthenticated),
          ]);
        final c = makeContainer(store: _signedInStore(), account: acct);
        c.read(sessionNotifierProvider);
        fa.flushMicrotasks();
        expect(c.read(sessionNotifierProvider).accountUnresolved, isTrue);

        fa.elapse(kAccountRetryInitialDelay);
        fa.flushMicrotasks();

        expect(
          c.read(sessionNotifierProvider),
          isA<SessionUnauthenticated>(),
          reason: 'the loop can never keep a revoked session alive',
        );
        final afterTerminal = acct.getAccountCalls;
        fa.elapse(const Duration(minutes: 10));
        fa.flushMicrotasks();
        expect(acct.getAccountCalls, afterTerminal);
      });
    });
  });
}
