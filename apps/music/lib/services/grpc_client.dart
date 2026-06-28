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

import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/auth.pbgrpc.dart' as auth;
import '../src/grpc/user.pbgrpc.dart' as user;
import 'account_service.dart';
import 'auth_service.dart';
import 'token_store.dart';

part 'grpc_client.g.dart';

/// gRPC endpoint of the Cymbra ID backend. Dev default is plaintext localhost;
/// override [cymbraEndpointProvider] for staging/production (TLS) wiring.
class CymbraEndpoint {
  final String host;
  final int port;
  final bool secure;

  const CymbraEndpoint({
    this.host = 'localhost',
    this.port = 50051,
    this.secure = false,
  });
}

/// Endpoint provider — overridden per environment.
@Riverpod(keepAlive: true)
CymbraEndpoint cymbraEndpoint(Ref ref) => const CymbraEndpoint();

/// Shared gRPC channel to the backend. Closed when the provider is disposed.
@Riverpod(keepAlive: true)
ClientChannel cymbraChannel(Ref ref) {
  final ep = ref.watch(cymbraEndpointProvider);
  final channel = ClientChannel(
    ep.host,
    port: ep.port,
    options: ChannelOptions(
      credentials: ep.secure
          ? const ChannelCredentials.secure()
          : const ChannelCredentials.insecure(),
    ),
  );
  ref.onDispose(() => channel.shutdown());
  return channel;
}

/// True for a gRPC `UNAUTHENTICATED` failure (status code 16) — the signal to
/// attempt a silent refresh.
bool isUnauthenticatedError(Object error) =>
    error is GrpcError && error.code == StatusCode.unauthenticated;

/// Map a [GrpcError] to a categorized [AuthException] for the UI.
AuthException authExceptionFromGrpc(GrpcError e) =>
    AuthException(authErrorFromCode(e.code), e.message);

/// Refresh-on-`UNAUTHENTICATED` retry orchestration (task 3.4), extracted from
/// the grpc plumbing so it is unit-testable with fakes (no channel needed).
///
/// Runs [call] with the current access token; on an unauthenticated failure it
/// asks [refreshAccessToken] for a new token, retries [call] **once** on success,
/// and invokes [onExpired] then rethrows when refresh gives up. The bearer header
/// is attached by [call] from the token it receives.
Future<T> authedCall<T>(
  Future<T> Function(String? bearer) call, {
  required Future<String?> Function() accessToken,
  required Future<String?> Function() refreshAccessToken,
  required void Function() onExpired,
  bool Function(Object error) isUnauthenticated = isUnauthenticatedError,
}) async {
  final token = await accessToken();
  try {
    return await call(token);
  } catch (e) {
    if (!isUnauthenticated(e)) rethrow;
    final fresh = await refreshAccessToken();
    if (fresh == null) {
      onExpired();
      rethrow;
    }
    return await call(fresh);
  }
}

/// Bearer-token call options for a protected RPC (the interceptor's injection
/// step). `null`/empty token yields no header (the call will 401 and refresh).
CallOptions bearerOptions(String? token) => (token == null || token.isEmpty)
    ? CallOptions()
    : CallOptions(metadata: {'authorization': 'Bearer $token'});

// --- Production gRPC adapters ------------------------------------------------

/// Production [AuthService] over the generated `AuthServiceClient`. Every method
/// is a thin translate-and-map: build the request, call the stub, map a
/// [GrpcError] to an [AuthException]. Sign-in calls carry the `music` audience.
class GrpcAuthService implements AuthService {
  GrpcAuthService(ClientChannel channel)
    : _client = auth.AuthServiceClient(channel);

  final auth.AuthServiceClient _client;

  Future<T> _map<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on GrpcError catch (e) {
      throw authExceptionFromGrpc(e);
    }
  }

  @override
  Future<void> signUpLocal({required String email, required String password}) =>
      _map(
        () => _client.signUpLocal(
          auth.SignUpLocalRequest(email: email, password: password),
        ),
      );

  @override
  Future<void> verifyEmail(String code) =>
      _map(() => _client.verifyEmail(auth.VerifyEmailRequest(token: code)));

  @override
  Future<void> resendVerification(String email) => _map(
    () => _client.resendVerification(
      auth.ResendVerificationRequest(email: email),
    ),
  );

  @override
  Future<AuthTokens> signInLocal({
    required String email,
    required String password,
  }) => _map(() async {
    final tp = await _client.signInLocal(
      auth.SignInLocalRequest(
        email: email,
        password: password,
        audience: kMusicAudience,
      ),
    );
    return AuthTokens(
      accessToken: tp.accessToken,
      refreshToken: tp.refreshToken,
    );
  });

  @override
  Future<AuthTokens> signInOidc(String idToken) => _map(() async {
    final tp = await _client.signInOidc(
      auth.SignInOidcRequest(idToken: idToken, audience: kMusicAudience),
    );
    return AuthTokens(
      accessToken: tp.accessToken,
      refreshToken: tp.refreshToken,
    );
  });

  @override
  Future<AuthTokens> refresh(String refreshToken) => _map(() async {
    final tp = await _client.refresh(
      auth.RefreshRequest(refreshToken: refreshToken),
    );
    return AuthTokens(
      accessToken: tp.accessToken,
      refreshToken: tp.refreshToken,
    );
  });

  @override
  Future<void> logout(String refreshToken) => _map(
    () => _client.logout(auth.LogoutRequest(refreshToken: refreshToken)),
  );

  @override
  Future<void> requestPasswordReset(String email) => _map(
    () => _client.requestPasswordReset(
      auth.RequestPasswordResetRequest(email: email),
    ),
  );

  @override
  Future<void> resetPassword({
    required String code,
    required String newPassword,
  }) => _map(
    () => _client.resetPassword(
      auth.ResetPasswordRequest(token: code, newPassword: newPassword),
    ),
  );
}

/// Production [AccountService] over the generated `UserServiceClient`. Protected
/// calls run through [authedCall] so a stale access token is refreshed once and
/// the call retried transparently.
class GrpcAccountService implements AccountService {
  GrpcAccountService({
    required ClientChannel channel,
    required TokenStore tokenStore,
    required AuthService authService,
  }) : _client = user.UserServiceClient(channel),
       _tokenStore = tokenStore,
       _authService = authService;

  final user.UserServiceClient _client;
  final TokenStore _tokenStore;
  final AuthService _authService;

  Future<String?> _accessToken() async =>
      (await _tokenStore.readTokens())?.accessToken;

  /// Refresh the session out-of-band (the Refresh RPC is unauthenticated), store
  /// the rotated pair, and return the fresh access token — or null (clearing the
  /// session) when refresh fails.
  Future<String?> _refreshAccess() async {
    final stored = await _tokenStore.readTokens();
    if (stored == null) return null;
    try {
      final fresh = await _authService.refresh(stored.refreshToken);
      await _tokenStore.writeTokens(fresh.toStored());
      return fresh.accessToken;
    } catch (_) {
      await _tokenStore.clear();
      return null;
    }
  }

  Future<T> _authed<T>(Future<T> Function(String? bearer) call) async {
    try {
      return await authedCall(
        call,
        accessToken: _accessToken,
        refreshAccessToken: _refreshAccess,
        onExpired: () {},
      );
    } on GrpcError catch (e) {
      throw authExceptionFromGrpc(e);
    }
  }

  Account _toAccount(user.Account a) => Account(
    userId: a.userId,
    version: a.version.toInt(),
    handle: a.hasHandle() ? a.handle : null,
    displayName: a.hasDisplayName() ? a.displayName : null,
  );

  @override
  Future<Account> getAccount() => _authed(
    (bearer) async => _toAccount(
      await _client.getAccount(
        user.GetAccountRequest(),
        options: bearerOptions(bearer),
      ),
    ),
  );

  @override
  Future<Account> updateHandle({
    required String handle,
    required int expectedVersion,
  }) => _authed(
    (bearer) async => _toAccount(
      await _client.updateAccount(
        user.UpdateAccountRequest(
          handle: handle,
          preferences: '{}',
          expectedVersion: Int64(expectedVersion),
        ),
        options: bearerOptions(bearer),
      ),
    ),
  );

  @override
  Future<bool> checkHandleAvailability(String handle) => _authed(
    (bearer) async => (await _client.checkHandleAvailability(
      user.CheckHandleAvailabilityRequest(handle: handle),
      options: bearerOptions(bearer),
    )).available,
  );

  @override
  Future<void> deleteAccount() => _authed(
    (bearer) async => _client.deleteAccount(
      user.DeleteAccountRequest(),
      options: bearerOptions(bearer),
    ),
  );
}

/// Production auth-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
AuthService authService(Ref ref) =>
    GrpcAuthService(ref.watch(cymbraChannelProvider));

/// Production account-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
AccountService accountService(Ref ref) => GrpcAccountService(
  channel: ref.watch(cymbraChannelProvider),
  tokenStore: ref.watch(tokenStoreProvider),
  authService: ref.watch(authServiceProvider),
);
