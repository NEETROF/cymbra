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
import 'token_refresher.dart';
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

/// Endpoint provider — overridden per environment. Host/port/TLS can be supplied
/// at build time with `--dart-define`:
///   `CYMBRA_GRPC_HOST=…` / `CYMBRA_GRPC_PORT=…` — reach a backend on the dev
///     machine from a physical device on the LAN;
///   `CYMBRA_GRPC_SECURE=true` — front the channel with TLS (production, where
///     Caddy terminates HTTPS on 443). Defaults stay plaintext localhost.
/// A typical prod build: `--dart-define=CYMBRA_GRPC_HOST=api.example.com
/// --dart-define=CYMBRA_GRPC_PORT=443 --dart-define=CYMBRA_GRPC_SECURE=true`.
@Riverpod(keepAlive: true)
CymbraEndpoint cymbraEndpoint(Ref ref) => const CymbraEndpoint(
  host: String.fromEnvironment('CYMBRA_GRPC_HOST', defaultValue: 'localhost'),
  port: int.fromEnvironment('CYMBRA_GRPC_PORT', defaultValue: 50051),
  secure: bool.fromEnvironment('CYMBRA_GRPC_SECURE'),
);

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

/// Refresh-on-`UNAUTHENTICATED` retry orchestration, extracted from the grpc
/// plumbing so it is unit-testable with fakes (no channel needed).
///
/// Runs [call] with the current access token; on an unauthenticated failure it
/// asks the coordinated [refresh] for a fresh token and, per the outcome:
/// - [RefreshRefreshed] → retries [call] **once** with the new bearer;
/// - [RefreshRejected] → the session is terminal (cleared by the refresher), so
///   the original `UNAUTHENTICATED` is rethrown to route the user to entry;
/// - [RefreshTransient] → throws a non-`UNAUTHENTICATED` error (`UNAVAILABLE`) so
///   callers (and the session bootstrap) keep the user signed in and retry later
///   instead of signing out on a flaky network.
Future<T> authedCall<T>(
  Future<T> Function(String? bearer) call, {
  required Future<String?> Function() accessToken,
  required Future<RefreshOutcome> Function() refresh,
  bool Function(Object error) isUnauthenticated = isUnauthenticatedError,
}) async {
  final token = await accessToken();
  try {
    return await call(token);
  } catch (e) {
    if (!isUnauthenticated(e)) rethrow;
    final outcome = await refresh();
    switch (outcome) {
      case RefreshRefreshed(:final accessToken):
        return await call(accessToken);
      case RefreshRejected():
        rethrow;
      case RefreshTransient():
        throw const GrpcError.unavailable(
          'session refresh failed transiently; keeping the session',
        );
    }
  }
}

/// Bearer-token call options for a protected RPC (the interceptor's injection
/// step). `null`/empty token yields no header (the call will 401 and refresh).
CallOptions bearerOptions(String? token) => (token == null || token.isEmpty)
    ? CallOptions()
    : CallOptions(metadata: {'authorization': 'Bearer $token'});

/// The authenticated-RPC seam every gRPC adapter shares: it reads the current
/// access token and runs a protected call through [authedCall] (refresh once on
/// `UNAUTHENTICATED` via the coordinated [TokenRefresher]), mapping a
/// [GrpcError] to an [AuthException] for the UI.
///
/// Instances are **callable**, so an adapter holds one as `_authed` and invokes
/// `_authed((bearer) => stub.rpc(..., options: bearerOptions(bearer)))`. Sharing
/// this one tested implementation removes the per-adapter token/refresh plumbing.
class AuthedRunner {
  AuthedRunner({
    required TokenStore tokenStore,
    required TokenRefresher refresher,
  }) : _tokenStore = tokenStore,
       _refresher = refresher;

  final TokenStore _tokenStore;
  final TokenRefresher _refresher;

  Future<String?> _accessToken() async =>
      (await _tokenStore.readTokens())?.accessToken;

  Future<T> call<T>(Future<T> Function(String? bearer) rpc) async {
    try {
      return await authedCall(
        rpc,
        accessToken: _accessToken,
        refresh: _refresher.refresh,
      );
    } on GrpcError catch (e) {
      throw authExceptionFromGrpc(e);
    }
  }
}

// --- Production gRPC adapters ------------------------------------------------

/// Production [AuthService] over the generated `AuthServiceClient`. Every method
/// is a thin translate-and-map: build the request, call the stub, map a
/// [GrpcError] to an [AuthException]. Sign-in calls carry the `music` audience.
class GrpcAuthService implements AuthService {
  GrpcAuthService(ClientChannel channel, {required AuthedRunner authed})
    : _client = auth.AuthServiceClient(channel),
      _authed = authed;

  final auth.AuthServiceClient _client;
  // Only the authenticated RPCs (revokeAllSessions) go through `_authed`; the
  // public sign-in/refresh calls use `_map` directly.
  final AuthedRunner _authed;

  Future<T> _map<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on GrpcError catch (e) {
      throw authExceptionFromGrpc(e);
    }
  }

  @override
  Future<void> signUpLocal({
    required String email,
    required String password,
    String? locale,
  }) => _map(
    () => _client.signUpLocal(
      auth.SignUpLocalRequest(
        email: email,
        password: password,
        locale: locale,
      ),
    ),
  );

  @override
  Future<void> verifyEmail(String code) =>
      _map(() => _client.verifyEmail(auth.VerifyEmailRequest(token: code)));

  @override
  Future<void> resendVerification(String email, {String? locale}) => _map(
    () => _client.resendVerification(
      auth.ResendVerificationRequest(email: email, locale: locale),
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
  Future<void> logout(String refreshToken) => _map(
    () => _client.logout(auth.LogoutRequest(refreshToken: refreshToken)),
  );

  @override
  Future<void> revokeAllSessions() => _authed(
    (bearer) async => _client.revokeAllSessions(
      auth.RevokeAllSessionsRequest(),
      options: bearerOptions(bearer),
    ),
  );

  @override
  Future<void> requestPasswordReset(String email, {String? locale}) => _map(
    () => _client.requestPasswordReset(
      auth.RequestPasswordResetRequest(email: email, locale: locale),
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

  @override
  Future<void> linkIdentity(String idToken) => _authed(
    (bearer) async => _client.linkIdentity(
      auth.LinkIdentityRequest(idToken: idToken),
      options: bearerOptions(bearer),
    ),
  );

  @override
  Future<void> unlinkIdentity({
    required String provider,
    required String subject,
  }) => _authed(
    (bearer) async => _client.unlinkIdentity(
      auth.UnlinkIdentityRequest(provider: provider, subject: subject),
      options: bearerOptions(bearer),
    ),
  );

  @override
  Future<void> setLocalCredential({
    required String email,
    required String password,
  }) => _authed(
    (bearer) async => _client.setLocalCredential(
      auth.SetLocalCredentialRequest(email: email, password: password),
      options: bearerOptions(bearer),
    ),
  );
}

/// Production [AccountService] over the generated `UserServiceClient`. Protected
/// calls run through [authedCall] so a stale access token is refreshed once and
/// the call retried transparently.
class GrpcAccountService implements AccountService {
  GrpcAccountService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = user.UserServiceClient(channel),
       _authed = authed;

  final user.UserServiceClient _client;
  final AuthedRunner _authed;

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

  @override
  Future<List<LinkedIdentity>> listIdentities() => _authed(
    (bearer) async => (await _client.listIdentities(
      user.ListIdentitiesRequest(),
      options: bearerOptions(bearer),
    )).identities.map(_toLinkedIdentity).toList(),
  );

  LinkedIdentity _toLinkedIdentity(user.Identity i) => LinkedIdentity(
    provider: i.provider,
    subject: i.subject,
    // `linked_at` is unix seconds (backend `extract(epoch …)`).
    linkedAt: DateTime.fromMillisecondsSinceEpoch(i.linkedAt.toInt() * 1000),
  );
}

/// Shared coordinated token refresher (change: fix-token-refresh-silent-signout).
/// The single keepAlive instance serialises `Refresh` across every authenticated
/// adapter, so concurrent expiries never replay (and revoke) the same refresh
/// token. It owns the unauthenticated `Refresh` RPC directly — depending on the
/// channel + token store only, never on [authServiceProvider] — so the auth
/// service can consume it without a provider cycle.
@Riverpod(keepAlive: true)
TokenRefresher tokenRefresher(Ref ref) {
  final client = auth.AuthServiceClient(ref.watch(cymbraChannelProvider));
  return CoordinatedTokenRefresher(
    tokenStore: ref.watch(tokenStoreProvider),
    // A raw GrpcError propagates: the refresher classifies it by status code.
    refreshRpc: (refreshToken) async {
      final tp = await client.refresh(
        auth.RefreshRequest(refreshToken: refreshToken),
      );
      return AuthTokens(
        accessToken: tp.accessToken,
        refreshToken: tp.refreshToken,
      );
    },
  );
}

/// The shared authenticated-RPC runner (token read + coordinated refresh) that
/// every gRPC adapter is constructed with.
@Riverpod(keepAlive: true)
AuthedRunner authedRunner(Ref ref) => AuthedRunner(
  tokenStore: ref.watch(tokenStoreProvider),
  refresher: ref.watch(tokenRefresherProvider),
);

/// Production auth-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
AuthService authService(Ref ref) => GrpcAuthService(
  ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);

/// Production account-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
AccountService accountService(Ref ref) => GrpcAccountService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
