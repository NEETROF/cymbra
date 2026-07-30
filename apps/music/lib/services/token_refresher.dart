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

import 'package:grpc/grpc.dart';

import 'auth_service.dart';
import 'token_store.dart';

/// The outcome of a coordinated token refresh (change:
/// fix-token-refresh-silent-signout). Distinguishing these three cases is what
/// stops a flaky network from silently signing the user out.
sealed class RefreshOutcome {
  const RefreshOutcome();
}

/// The session was refreshed: [accessToken] is the new bearer, and the rotated
/// pair has already been persisted.
class RefreshRefreshed extends RefreshOutcome {
  final String accessToken;
  const RefreshRefreshed(this.accessToken);
}

/// The refresh token is genuinely expired or revoked (`UNAUTHENTICATED` /
/// `INVALID_ARGUMENT`). The local session has been cleared; the caller must
/// route the user back to the entry screen.
class RefreshRejected extends RefreshOutcome {
  const RefreshRejected();
}

/// The refresh could not complete for a transient reason (offline, timeout,
/// `UNAVAILABLE`, or an unexpected error). The stored session is left **intact**
/// so a later retry can recover it — the user is NOT signed out.
class RefreshTransient extends RefreshOutcome {
  const RefreshTransient();
}

/// The unauthenticated `Refresh` RPC: exchange a refresh token for a fresh pair.
/// Throwing an [AuthException] (or [GrpcError]) is how failures are classified.
typedef RefreshRpc = Future<AuthTokens> Function(String refreshToken);

/// Coordinates silent session refresh across every authenticated service seam
/// (change: fix-token-refresh-silent-signout).
///
/// Two guarantees fix the random silent sign-out:
/// - **Single-flight**: at most one `Refresh` is in flight for the stored
///   session. Concurrent callers await the same result instead of each replaying
///   the stored refresh token — which the backend rotates and treats as reuse,
///   revoking the whole session family.
/// - **Failure classification**: only a real refresh-token rejection clears the
///   session; a transient/offline failure leaves it intact.
abstract class TokenRefresher {
  /// Refresh the stored session, coalescing concurrent callers into one RPC.
  Future<RefreshOutcome> refresh();
}

/// Production [TokenRefresher]. The single instance (a keepAlive provider) is the
/// shared coordination point for all authenticated adapters.
class CoordinatedTokenRefresher implements TokenRefresher {
  CoordinatedTokenRefresher({
    required TokenStore tokenStore,
    required RefreshRpc refreshRpc,
  }) : _tokenStore = tokenStore,
       _refreshRpc = refreshRpc;

  final TokenStore _tokenStore;
  final RefreshRpc _refreshRpc;

  /// The in-flight refresh, shared by every concurrent caller. Null between
  /// refreshes so the next expiry starts a fresh single flight.
  Future<RefreshOutcome>? _inFlight;

  @override
  Future<RefreshOutcome> refresh() {
    final existing = _inFlight;
    if (existing != null) return existing;
    final started = _run();
    _inFlight = started;
    started.whenComplete(() {
      if (identical(_inFlight, started)) _inFlight = null;
    });
    return started;
  }

  Future<RefreshOutcome> _run() async {
    final stored = await _tokenStore.readTokens();
    // No stored session to refresh — already effectively signed out.
    if (stored == null) return const RefreshRejected();
    try {
      final fresh = await _refreshRpc(stored.refreshToken);
      await _tokenStore.writeTokens(fresh.toStored());
      return RefreshRefreshed(fresh.accessToken);
    } on AuthException catch (e) {
      return _classify(e.error);
    } on GrpcError catch (e) {
      return _classify(authErrorFromCode(e.code));
    } catch (_) {
      // A non-gRPC failure (e.g. no connectivity throws a SocketException):
      // transient, so never clear the session on it.
      return const RefreshTransient();
    }
  }

  /// Terminal only for an actual refresh-token rejection; everything else
  /// (unavailable, deadline exceeded, unknown) is transient and keeps the
  /// session.
  Future<RefreshOutcome> _classify(AuthError error) async {
    if (error == AuthError.unauthenticated ||
        error == AuthError.invalidArgument) {
      await _tokenStore.clear();
      return const RefreshRejected();
    }
    return const RefreshTransient();
  }
}
