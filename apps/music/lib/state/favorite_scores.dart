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

import 'contributed_scores.dart';
import 'saved_catalog_scores.dart';
import 'score_catalog.dart';
import 'session_notifier.dart';

part 'favorite_scores.g.dart';

/// The signed-in user's favorites — the home screen's content: the catalog
/// scores they saved from the hub PLUS their own uploads that are favorited,
/// as one unified list (no distinction by origin). Empty when signed out (the
/// home then shows the bundled demo catalog instead). Invalidated by save/remove
/// (catalog) or upload/favorite-toggle (uploads).
@riverpod
Future<List<CatalogEntry>> favoriteScores(Ref ref) async {
  if (!ref.watch(canUseOnlineServicesProvider)) return const [];
  final uploads = await ref.watch(myUploadsProvider.future);
  final saved = await ref.watch(savedCatalogScoresProvider.future);
  final handle = ref.watch(currentUserHandleProvider);
  return [
    for (final s in uploads)
      if (s.favorite) contributedEntry(s, uploaderHandle: handle),
    ...saved,
  ];
}
