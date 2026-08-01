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
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'catalog_service.dart';
import 'offline_key_provider.dart';

part 'offline_server_secret_service.g.dart';

/// Supplies the server-issued per-user offline secret (change:
/// add-offline-score-cache) — one input to the offline-cache key derivation.
/// Fetched from the backend when reachable and cached in the OS keystore, so an
/// offline launch still resolves the last-known value.
abstract class OfflineServerSecretService {
  /// The secret for [userId]: refreshed from the backend when online, otherwise
  /// the last cached value; `null` when it was never cached and the backend is
  /// unreachable (so the byte cache stays fail-closed on first-ever offline use).
  Future<Uint8List?> secretFor(String userId);

  /// Drop the cached secret for [userId] (sign-out / purge).
  Future<void> clear(String userId);
}

/// Production impl: fetches over the authenticated `GetOfflineCacheKey` RPC and
/// caches the result in the secure keystore.
class BackendOfflineServerSecretService implements OfflineServerSecretService {
  BackendOfflineServerSecretService({
    required CatalogService catalog,
    required SecureBytesStore store,
  }) : _catalog = catalog,
       _store = store;

  final CatalogService _catalog;
  final SecureBytesStore _store;

  String _key(String userId) => 'cymbra.offline.server_secret.$userId';

  @override
  Future<Uint8List?> secretFor(String userId) async {
    try {
      // The secret is stable server-side, so refreshing opportunistically when
      // online is idempotent; it also picks up a server-side rotation.
      final fresh = await _catalog.getOfflineCacheKey();
      await _store.write(_key(userId), fresh);
      return fresh;
    } catch (e) {
      // Offline / backend unreachable → the last cached value (may be null on a
      // first-ever offline run, which keeps the byte cache disabled).
      debugPrint('offline server secret fetch failed ($e); using cached value.');
      return _store.read(_key(userId));
    }
  }

  @override
  Future<void> clear(String userId) => _store.delete(_key(userId));
}

@Riverpod(keepAlive: true)
OfflineServerSecretService offlineServerSecretService(Ref ref) =>
    BackendOfflineServerSecretService(
      catalog: ref.watch(catalogServiceProvider),
      store: ref.watch(secureBytesStoreProvider),
    );
