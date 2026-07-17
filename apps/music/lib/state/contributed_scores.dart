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
import 'session_notifier.dart';

part 'contributed_scores.g.dart';

/// The signed-in user's raw uploads (all of them, favorite or not). Empty when
/// signed out. Invalidate to refresh after an upload, delete, or favorite toggle.
@riverpod
Future<List<ContributedScore>> myUploads(Ref ref) async {
  if (!ref.watch(canUseOnlineServicesProvider)) return const [];
  return ref.read(scoreUploadServiceProvider).listMyScores();
}

/// The signed-in user's contributed scores, as [CatalogEntry]s (byte-sourced from
/// the backend) so they slot into the same player path as bundled scores. Used by
/// the hub's "mes partitions" (all uploads). Empty when signed out.
@riverpod
Future<List<CatalogEntry>> myContributedScores(Ref ref) async {
  final scores = await ref.watch(myUploadsProvider.future);
  return [for (final s in scores) contributedEntry(s)];
}

/// Maps an upload to a [CatalogEntry] (byte-sourced, facets carried through).
CatalogEntry contributedEntry(ContributedScore s) {
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
  );
}

String _shortDate(DateTime d) =>
    '${d.day.toString().padLeft(2, '0')}/${d.month.toString().padLeft(2, '0')}/${d.year}';
