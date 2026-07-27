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

import '../services/catalog_service.dart';
import 'score_catalog.dart';
import 'session_notifier.dart';

part 'saved_catalog_scores.g.dart';

/// A monotonic bump signal for library changes made *outside* this provider (e.g.
/// saving/un-saving a score from the hub). Mutations bump it **after** they
/// persist; dependents `ref.listen` it and refresh themselves — so no one has to
/// invalidate a sibling provider (architecture rule 2).
@riverpod
class LibraryRevision extends _$LibraryRevision {
  @override
  int build() => 0;

  void bump() => state = state + 1;
}

/// Maps a backend [CatalogHit] to a [CatalogEntry] (byte-sourced from the
/// catalog by [CatalogEntry.catalogId]) so a saved catalog score slots into the
/// same library grouping and player path as bundled and contributed scores.
CatalogEntry catalogEntryFromHit(CatalogHit h) => CatalogEntry(
  id: 'catalog-${h.id}',
  title: (h.title != null && h.title!.isNotEmpty) ? h.title! : 'Sans titre',
  composer: h.composer ?? '',
  level: h.level ?? PracticeLevel.beginner,
  catalogId: h.id,
  source: h.source,
  arranger: h.arranger,
  minNoteValue: h.minNoteValue,
  tempoBpm: h.tempoBpm,
  noteCount: h.noteCount,
  lowestMidi: h.lowestMidi,
  highestMidi: h.highestMidi,
  timeSig: h.timeSig,
  keyFifths: h.keyFifths,
);

/// The signed-in user's saved catalog scores, as [CatalogEntry]s, newest-saved
/// first. Empty when signed out (the home section is then not shown).
///
/// Owns the "remove from library" mutation so widgets never call the catalog
/// service directly — they call this notifier, which reloads itself
/// (`AsyncValue.guard` keeps a failure in the state, not thrown).
@riverpod
class SavedCatalogScores extends _$SavedCatalogScores {
  @override
  Future<List<CatalogEntry>> build() {
    // Refresh when a library change happened elsewhere (e.g. the hub's save
    // toggle bumps the revision after it persists) — reactive, not invalidated.
    ref.listen(libraryRevisionProvider, (_, _) => ref.invalidateSelf());
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return Future.value(const <CatalogEntry>[]);
    }
    return _fetch();
  }

  Future<List<CatalogEntry>> _fetch() async {
    final hits = await ref.read(catalogServiceProvider).listSaved();
    return [for (final h in hits) catalogEntryFromHit(h)];
  }

  /// Remove a saved catalog score from the caller's library, then reload.
  Future<void> remove(String catalogId) async {
    state = await AsyncValue.guard(() async {
      await ref.read(catalogServiceProvider).remove(catalogId);
      return _fetch();
    });
  }
}
