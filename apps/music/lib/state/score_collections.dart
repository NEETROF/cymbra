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
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/auth_service.dart';
import '../services/score_upload_service.dart';

part 'score_collections.g.dart';

/// Why a collection mutation failed, in terms the UI can localize (change:
/// add-private-score-catalog). The raw exception never reaches a screen.
enum CollectionError {
  /// The caller already has a collection with that name (case-insensitively).
  nameTaken,

  /// The name was empty or too long.
  invalidName,

  /// Anything else — transport, session, server.
  failed,
}

/// Maps a service failure onto a typed, localizable reason.
CollectionError collectionErrorOf(Object error) {
  if (error is AuthException) {
    return switch (error.error) {
      AuthError.alreadyExists => CollectionError.nameTaken,
      AuthError.invalidArgument => CollectionError.invalidName,
      _ => CollectionError.failed,
    };
  }
  return CollectionError.failed;
}

/// The caller's collections, newest first. The server is the source of truth, so
/// this is simply re-read after every mutation — that is what makes a collection
/// created on one device appear on another (no device-local divergence).
@riverpod
class ScoreCollections extends _$ScoreCollections {
  @override
  Future<List<ScoreCollection>> build() =>
      ref.watch(scoreUploadServiceProvider).listCollections();

  /// Create a collection, then refresh. Returns the typed reason on failure so a
  /// listener can show a localized message; `null` means it worked.
  Future<CollectionError?> create(String name) =>
      _mutate((s) => s.createCollection(name));

  Future<CollectionError?> rename(String id, String name) =>
      _mutate((s) => s.renameCollection(id, name));

  Future<CollectionError?> remove(String id) =>
      _mutate((s) => s.deleteCollection(id));

  Future<CollectionError?> addScore(String collectionId, String scoreId) =>
      _mutate((s) => s.addToCollection(collectionId, scoreId));

  Future<CollectionError?> removeScore(String collectionId, String scoreId) =>
      _mutate((s) => s.removeFromCollection(collectionId, scoreId));

  /// Run one mutation and re-read the list from the server. Invalidating
  /// **itself** is the allowed direction; no sibling provider is poked.
  Future<CollectionError?> _mutate(
    Future<void> Function(ScoreUploadService service) action,
  ) async {
    try {
      await action(ref.read(scoreUploadServiceProvider));
      ref.invalidateSelf();
      return null;
    } catch (e) {
      return collectionErrorOf(e);
    }
  }
}

/// The collection the private library is filtered by; `null` = the whole
/// library. Held apart from the collection list so switching filters never
/// refetches the collections themselves.
@riverpod
class CollectionFilter extends _$CollectionFilter {
  @override
  String? build() => null;

  void select(String? collectionId) => state = collectionId;
}

/// The caller's uploads inside [CollectionFilter]'s collection. Watched by the
/// library only while a filter is active.
@riverpod
Future<List<ContributedScore>> scoresInCollection(Ref ref) async {
  final collectionId = ref.watch(collectionFilterProvider);
  if (collectionId == null) return const [];
  return ref
      .watch(scoreUploadServiceProvider)
      .listMyScoresInCollection(collectionId);
}
