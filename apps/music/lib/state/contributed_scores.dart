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

import '../services/score_upload_service.dart';
import 'score_catalog.dart';
import 'score_upload_notifier.dart';
import 'session_notifier.dart';

part 'contributed_scores.g.dart';

/// The signed-in user's raw uploads (all of them, favorite or not). Empty when
/// signed out.
///
/// Owns the upload mutations (favorite toggle, delete) so widgets never call the
/// upload service directly — they call this notifier, which reloads itself
/// (`AsyncValue.guard` keeps failures in the state, never thrown). It also
/// *listens* for a completed upload and refreshes itself, rather than the uploader
/// reaching across to invalidate it (architecture rules: no sibling invalidation,
/// UI never calls services, actions surface via state not awaited returns).
@riverpod
class MyUploads extends _$MyUploads {
  @override
  Future<List<ContributedScore>> build() {
    // The dependent listens to what it depends on: a new successful upload
    // (`result` transitions non-null) refreshes the list.
    ref.listen(scoreUploadNotifierProvider.select((s) => s.result), (
      prev,
      next,
    ) {
      if (next != null && next != prev) ref.invalidateSelf();
    });
    // `watch` so sign-in/out rebuilds the list; mutations reload via `_fetch`.
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return Future.value(const <ContributedScore>[]);
    }
    return ref.read(scoreUploadServiceProvider).listMyScores();
  }

  Future<List<ContributedScore>> _fetch() {
    if (!ref.read(canUseOnlineServicesProvider)) {
      return Future.value(const <ContributedScore>[]);
    }
    return ref.read(scoreUploadServiceProvider).listMyScores();
  }

  /// Favorite / un-favorite one of the caller's uploads, then reload. A failure
  /// lands in the state (surfaced by a listener), never thrown to the caller.
  Future<void> toggleFavorite(
    String contributedId, {
    required bool favorite,
  }) async {
    state = await AsyncValue.guard(() async {
      await ref
          .read(scoreUploadServiceProvider)
          .setFavorite(contributedId, favorite);
      return _fetch();
    });
  }

  /// Delete one of the caller's uploads (destructive), then reload.
  Future<void> delete(String contributedId) async {
    state = await AsyncValue.guard(() async {
      await ref.read(scoreUploadServiceProvider).deleteScore(contributedId);
      return _fetch();
    });
  }
}

/// The signed-in user's contributed scores, as [CatalogEntry]s (byte-sourced from
/// the backend) so they slot into the same player path as bundled scores. Used by
/// the hub's "mes partitions" (all uploads). Empty when signed out.
@riverpod
Future<List<CatalogEntry>> myContributedScores(Ref ref) async {
  final scores = await ref.watch(myUploadsProvider.future);
  final handle = ref.watch(currentUserHandleProvider);
  return [for (final s in scores) contributedEntry(s, uploaderHandle: handle)];
}

/// Maps an upload to a [CatalogEntry] (byte-sourced, facets carried through).
/// [uploaderHandle] drives the "{handle} · Cymbra" attribution on the card.
CatalogEntry contributedEntry(ContributedScore s, {String? uploaderHandle}) {
  final hasTitle = s.title != null && s.title!.isNotEmpty;
  final hasComposer = s.composer != null && s.composer!.isNotEmpty;
  var composer = '';
  if (hasComposer) {
    composer = s.composer!;
  } else if (!hasTitle) {
    composer = _shortDate(s.createdAt);
  }

  return CatalogEntry(
    id: 'contrib-${s.id}',
    title: hasTitle ? s.title! : 'Sans titre',
    // Fall back to the upload date so multiple untitled uploads stay
    // distinguishable in the list (option A).
    composer: composer,
    level: s.level,
    contributedId: s.id,
    // Facets so the contributed cover is as faithful as a catalog one.
    minNoteValue: s.minNoteValue,
    tempoBpm: s.tempoBpm,
    noteCount: s.noteCount,
    lowestMidi: s.lowestMidi,
    highestMidi: s.highestMidi,
    timeSig: s.timeSig,
    keyFifths: s.keyFifths,
    favorite: s.favorite,
    uploaderHandle: uploaderHandle,
  );
}

String _shortDate(DateTime d) =>
    '${d.day.toString().padLeft(2, '0')}/${d.month.toString().padLeft(2, '0')}/${d.year}';
