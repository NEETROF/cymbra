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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/account_service.dart';
import '../services/auth_service.dart';
import '../services/grpc_client.dart';
import '../services/oidc_token_source.dart';
import 'connected_accounts_state.dart';

part 'connected_accounts_notifier.g.dart';

/// Drives the Connected accounts screen (change: add-account-identity-linking):
/// loads the linked identities and runs the link/unlink actions. The UI calls
/// these methods and reacts to [ConnectedAccountsState] (never awaits a return),
/// matching the app's notifier convention. After every mutating action the
/// identity list is re-fetched rather than mutated in place, so the displayed
/// state always matches the server (design: "stale list after link/unlink").
@riverpod
class ConnectedAccountsNotifier extends _$ConnectedAccountsNotifier {
  AccountService get _account => ref.read(accountServiceProvider);
  AuthService get _auth => ref.read(authServiceProvider);
  OidcTokenSource get _oidc => ref.read(oidcTokenSourceProvider);

  @override
  ConnectedAccountsState build() {
    // Resolve after build returns — never touch `state` synchronously here.
    Future.microtask(load);
    return const ConnectedAccountsState(identities: AsyncValue.loading());
  }

  /// (Re)load the linked identities.
  Future<void> load() async {
    state = state.copyWith(identities: const AsyncValue.loading());
    state = state.copyWith(
      identities: await AsyncValue.guard(_account.listIdentities),
    );
  }

  /// Link a Google identity: mint a fresh `id_token`, attach it, then re-fetch.
  Future<void> linkGoogle() =>
      _link(ConnectedAccountsAction.linkGoogle, () => _oidc.googleIdToken());

  /// Link an Apple identity.
  Future<void> linkApple() =>
      _link(ConnectedAccountsAction.linkApple, () => _oidc.appleIdToken());

  /// Shared OIDC link path: obtain the provider token (null = the user cancelled
  /// the native sheet → silent no-op), attach it, and refresh the list.
  Future<void> _link(
    ConnectedAccountsAction action,
    Future<String?> Function() mintToken,
  ) => _runAction(action, () async {
    final idToken = await mintToken();
    if (idToken == null) return false; // cancelled — nothing to do, no error
    await _auth.linkIdentity(idToken);
    return true;
  });

  /// Add an email + password credential (server sends a verification email).
  Future<void> linkEmailPassword({
    required String email,
    required String password,
    String? locale,
  }) => _runAction(ConnectedAccountsAction.setPassword, () async {
    await _auth.setLocalCredential(
      email: email,
      password: password,
      locale: locale,
    );
    return true;
  });

  /// Unlink an identity. The screen already blocks the last identity; the server
  /// still guards it (mapped to the "only sign-in method" message by the UI).
  Future<void> unlink({required String provider, required String subject}) =>
      _runAction(ConnectedAccountsAction.unlink, () async {
        await _auth.unlinkIdentity(provider: provider, subject: subject);
        return true;
      });

  /// Run a mutating [action] under the [busy] guard. When it returns true the
  /// identity list is re-fetched; an [AuthException] (or any error) is captured
  /// in [ConnectedAccountsState.actionError] for the listener to surface. Every
  /// completed action bumps [ConnectedAccountsState.actionSeq] so the listener
  /// fires even on a repeated identical error.
  Future<void> _runAction(
    ConnectedAccountsAction action,
    Future<bool> Function() run,
  ) async {
    if (state.busy) return;
    state = state.copyWith(busy: true, actionError: null);
    AuthException? error;
    var mutated = false;
    try {
      mutated = await run();
    } on AuthException catch (e) {
      error = e;
    } catch (_) {
      // A desktop OAuth / platform failure is not an AuthException; surface it as
      // a generic link failure rather than letting it escape.
      error = const AuthException(AuthError.unknown);
    }
    if (mutated && error == null) {
      state = state.copyWith(
        identities: await AsyncValue.guard(_account.listIdentities),
      );
    }
    state = state.copyWith(
      busy: false,
      actionSeq: state.actionSeq + 1,
      actionError: error,
      lastAction: action,
    );
  }
}
