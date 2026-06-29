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

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'token_store.g.dart';

/// A persisted Cymbra ID session: the access/refresh token pair.
class StoredTokens {
  final String accessToken;
  final String refreshToken;

  const StoredTokens({required this.accessToken, required this.refreshToken});
}

/// Seam over platform secure storage for the account session (task 3.1).
///
/// Holds the token pair (Keychain/Keystore-backed in production) and the
/// "continue as guest" choice, so the session layer can hydrate at startup
/// without touching the backend. Kept abstract so [SessionNotifier] and the
/// gRPC interceptor are testable with an in-memory fake (no platform channel).
abstract class TokenStore {
  /// Read the stored token pair, or null when there is no signed-in session.
  Future<StoredTokens?> readTokens();

  /// Persist (replace) the token pair. Clears the guest flag — a real session
  /// supersedes a prior guest choice.
  Future<void> writeTokens(StoredTokens tokens);

  /// Whether the user previously chose to continue as a guest.
  Future<bool> isGuest();

  /// Persist the guest choice (no tokens are stored for a guest).
  Future<void> setGuest();

  /// Erase the session entirely (tokens and guest flag) — used on sign-out,
  /// deletion, or a failed refresh.
  Future<void> clear();
}

/// Production [TokenStore] backed by `flutter_secure_storage`. Tokens never land
/// in plain preferences or logs (spec: "Secure session storage").
///
/// Keychain/Keystore can be unavailable (no device lock, a missing entitlement,
/// etc.). Per the design's "secure-storage availability" risk, every operation
/// is best-effort: reads fall back to "no session" and writes are swallowed
/// rather than crashing the app. A `PlatformException` here must never bubble up
/// to the UI.
class SecureTokenStore implements TokenStore {
  SecureTokenStore([FlutterSecureStorage? storage])
    : _storage =
          storage ??
          // macOS: use the legacy keychain (not the data-protection keychain),
          // which works under the app sandbox with ad-hoc signing — the data-
          // protection keychain needs a `keychain-access-groups` entitlement and
          // therefore a development certificate (errSecMissingEntitlement -34018).
          const FlutterSecureStorage(
            mOptions: MacOsOptions(useDataProtectionKeyChain: false),
          );

  final FlutterSecureStorage _storage;

  static const _kAccess = 'cymbra.access_token';
  static const _kRefresh = 'cymbra.refresh_token';
  static const _kGuest = 'cymbra.guest';

  /// Run a best-effort storage write, swallowing platform failures so a flaky
  /// Keychain never crashes the app (the session simply won't persist).
  Future<void> _bestEffort(Future<void> Function() op) async {
    try {
      await op();
    } catch (e) {
      debugPrint('SecureTokenStore: storage write failed ($e); continuing.');
    }
  }

  Future<String?> _read(String key) async {
    try {
      return await _storage.read(key: key);
    } catch (e) {
      debugPrint(
        'SecureTokenStore: storage read failed ($e); treating as empty.',
      );
      return null;
    }
  }

  @override
  Future<StoredTokens?> readTokens() async {
    final access = await _read(_kAccess);
    final refresh = await _read(_kRefresh);
    if (access == null || refresh == null) return null;
    return StoredTokens(accessToken: access, refreshToken: refresh);
  }

  @override
  Future<void> writeTokens(StoredTokens tokens) => _bestEffort(() async {
    await _storage.delete(key: _kGuest);
    await _storage.write(key: _kAccess, value: tokens.accessToken);
    await _storage.write(key: _kRefresh, value: tokens.refreshToken);
  });

  @override
  Future<bool> isGuest() async => (await _read(_kGuest)) == 'true';

  @override
  Future<void> setGuest() => _bestEffort(() async {
    await _storage.delete(key: _kAccess);
    await _storage.delete(key: _kRefresh);
    await _storage.write(key: _kGuest, value: 'true');
  });

  @override
  Future<void> clear() => _bestEffort(() async {
    await _storage.delete(key: _kAccess);
    await _storage.delete(key: _kRefresh);
    await _storage.delete(key: _kGuest);
  });
}

/// Production token-store provider. Override in tests with an in-memory fake.
@Riverpod(keepAlive: true)
TokenStore tokenStore(Ref ref) => SecureTokenStore();
