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

import 'package:music/services/account_service.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/oidc_token_source.dart';
import 'package:music/services/token_refresher.dart';
import 'package:music/services/token_store.dart';

/// In-memory [TokenStore] for tests — no platform channel.
class FakeTokenStore implements TokenStore {
  StoredTokens? tokens;
  bool guest;

  FakeTokenStore({this.tokens, this.guest = false});

  @override
  Future<StoredTokens?> readTokens() async => tokens;

  @override
  Future<void> writeTokens(StoredTokens t) async {
    tokens = t;
    guest = false;
  }

  @override
  Future<bool> isGuest() async => guest;

  @override
  Future<void> setGuest() async {
    tokens = null;
    guest = true;
  }

  @override
  Future<void> clear() async {
    tokens = null;
    guest = false;
  }
}

/// Scriptable [TokenRefresher] fake: returns a canned [RefreshOutcome] and counts
/// how many times it was asked to refresh (to assert single-flight / no-refresh).
class FakeTokenRefresher implements TokenRefresher {
  FakeTokenRefresher(this.outcome);

  RefreshOutcome outcome;
  int calls = 0;

  @override
  Future<RefreshOutcome> refresh() async {
    calls++;
    return outcome;
  }
}

/// Scriptable [AuthService] fake: returns canned tokens, throws canned errors,
/// and records every call so tests can assert audience/credentials behaviour.
class FakeAuthService implements AuthService {
  AuthTokens tokens;

  /// When set, the next matching call throws this instead of succeeding.
  AuthException? signInError;
  AuthException? signUpError;
  AuthException? verifyError;
  AuthException? resendError;
  AuthException? resetError;
  AuthException? logoutError;
  AuthException? revokeAllError;
  AuthException? unlinkError;

  /// Errors thrown by successive `linkIdentity` calls, consumed in order — lets a
  /// test model "fails once (stale token) then succeeds after a re-mint". A null
  /// entry (or running past the end) is a success.
  final List<AuthException?> linkErrors = [];

  final List<String> calls = [];

  FakeAuthService({
    this.tokens = const AuthTokens(
      accessToken: 'access-1',
      refreshToken: 'refresh-1',
    ),
  });

  @override
  Future<void> signUpLocal({
    required String email,
    required String password,
    String? locale,
  }) async {
    calls.add('signUpLocal:$email');
    if (signUpError != null) throw signUpError!;
  }

  @override
  Future<void> verifyEmail(String code) async {
    calls.add('verifyEmail:$code');
    if (verifyError != null) throw verifyError!;
  }

  @override
  Future<void> resendVerification(String email, {String? locale}) async {
    calls.add('resendVerification:$email');
    if (resendError != null) throw resendError!;
  }

  @override
  Future<AuthTokens> signInLocal({
    required String email,
    required String password,
  }) async {
    calls.add('signInLocal:$email');
    if (signInError != null) throw signInError!;
    return tokens;
  }

  @override
  Future<AuthTokens> signInOidc(String idToken) async {
    calls.add('signInOidc:$idToken');
    if (signInError != null) throw signInError!;
    return tokens;
  }

  @override
  Future<void> logout(String refreshToken) async {
    calls.add('logout:$refreshToken');
    if (logoutError != null) throw logoutError!;
  }

  @override
  Future<void> revokeAllSessions() async {
    calls.add('revokeAllSessions');
    if (revokeAllError != null) throw revokeAllError!;
  }

  @override
  Future<void> requestPasswordReset(String email, {String? locale}) async {
    calls.add('requestPasswordReset:$email');
  }

  @override
  Future<void> resetPassword({
    required String code,
    required String newPassword,
  }) async {
    calls.add('resetPassword:$code');
    if (resetError != null) throw resetError!;
  }

  int _linkCount = 0;

  @override
  Future<void> linkIdentity(String idToken) async {
    calls.add('linkIdentity:$idToken');
    final err = _linkCount < linkErrors.length ? linkErrors[_linkCount] : null;
    _linkCount++;
    if (err != null) throw err;
  }

  @override
  Future<void> unlinkIdentity({
    required String provider,
    required String subject,
  }) async {
    calls.add('unlinkIdentity:$provider:$subject');
    if (unlinkError != null) throw unlinkError!;
  }

  AuthException? setLocalCredentialError;

  @override
  Future<void> setLocalCredential({
    required String email,
    required String password,
  }) async {
    calls.add('setLocalCredential:$email');
    if (setLocalCredentialError != null) throw setLocalCredentialError!;
  }
}

/// Scriptable [AccountService] fake: holds an account, a set of taken handles,
/// and optional errors. Records calls for assertions.
class FakeAccountService implements AccountService {
  Account? account;
  Set<String> takenHandles;
  AuthException? getError;
  AuthException? updateError;
  AuthException? deleteError;
  AuthException? listIdentitiesError;

  /// The account [getAccount] returns *after* a [deleteAccount] — models the
  /// account switch in the collision-link flow (delete the orphan, then resolve
  /// the existing account). Defaults to null (a plain deletion clears the account).
  Account? postDeleteAccount;

  /// Identities returned by [listIdentities]; mutate to model link/unlink refetch.
  List<LinkedIdentity> identities;

  final List<String> calls = [];

  FakeAccountService({
    this.account,
    Set<String>? takenHandles,
    this.getError,
    this.updateError,
    this.deleteError,
    this.listIdentitiesError,
    List<LinkedIdentity>? identities,
  }) : takenHandles = takenHandles ?? <String>{},
       identities = identities ?? <LinkedIdentity>[];

  @override
  Future<Account> getAccount() async {
    calls.add('getAccount');
    if (getError != null) throw getError!;
    return account ?? (throw const AuthException(AuthError.notFound));
  }

  @override
  Future<Account> updateHandle({
    required String handle,
    required int expectedVersion,
  }) async {
    calls.add('updateHandle:$handle');
    if (updateError != null) throw updateError!;
    if (takenHandles.contains(handle.toLowerCase())) {
      throw const AuthException(AuthError.alreadyExists);
    }
    final updated = Account(
      userId: account?.userId ?? 'user-1',
      version: expectedVersion + 1,
      handle: handle,
      displayName: account?.displayName,
    );
    account = updated;
    return updated;
  }

  @override
  Future<bool> checkHandleAvailability(String handle) async {
    calls.add('checkHandleAvailability:$handle');
    return !takenHandles.contains(handle.toLowerCase());
  }

  @override
  Future<void> deleteAccount() async {
    calls.add('deleteAccount');
    if (deleteError != null) throw deleteError!;
    account = postDeleteAccount;
  }

  @override
  Future<List<LinkedIdentity>> listIdentities() async {
    calls.add('listIdentities');
    if (listIdentitiesError != null) throw listIdentitiesError!;
    return List.unmodifiable(identities);
  }
}

/// Scriptable [OidcTokenSource] fake: returns canned id_tokens (null = the user
/// cancelled the native sheet) without any platform channel. When [googleError]
/// is set, `googleIdToken` throws it — modelling the desktop loopback flow, which
/// throws (e.g. `DesktopOauthException`) on a real failure rather than returning
/// null.
class FakeOidcTokenSource implements OidcTokenSource {
  String? googleToken;
  String? appleToken;
  Object? googleError;
  @override
  bool googleAvailable;
  @override
  bool appleAvailable;

  final List<String> calls = [];

  FakeOidcTokenSource({
    this.googleToken = 'google-id-token',
    this.appleToken = 'apple-id-token',
    this.googleError,
    this.googleAvailable = true,
    this.appleAvailable = true,
  });

  @override
  Future<String?> googleIdToken({bool forceChooser = false}) async {
    calls.add(forceChooser ? 'google:forceChooser' : 'google');
    if (googleError != null) throw googleError!;
    return googleToken;
  }

  @override
  Future<String?> appleIdToken() async {
    calls.add('apple');
    return appleToken;
  }

  @override
  Future<void> signOut() async {
    calls.add('oidcSignOut');
  }
}
