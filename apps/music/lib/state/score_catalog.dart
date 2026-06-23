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

part 'score_catalog.g.dart';

/// Practice difficulty of a bundled score.
enum PracticeLevel { beginner, intermediate, advanced }

/// Human-readable label for a [PracticeLevel].
extension PracticeLevelLabel on PracticeLevel {
  String get label => switch (this) {
    PracticeLevel.beginner => 'Beginner',
    PracticeLevel.intermediate => 'Intermediate',
    PracticeLevel.advanced => 'Advanced',
  };
}

/// One entry in the bundled score catalog: a public-domain MusicXML asset
/// tagged with display metadata and a practice level.
class CatalogEntry {
  /// Stable identifier (used as a list key and selection identity).
  final String id;
  final String title;
  final String composer;

  /// Bundle path of the uncompressed `.musicxml`/`.xml` asset.
  final String assetPath;
  final PracticeLevel level;

  const CatalogEntry({
    required this.id,
    required this.title,
    required this.composer,
    required this.assetPath,
    required this.level,
  });

  @override
  int get hashCode => id.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CatalogEntry &&
          runtimeType == other.runtimeType &&
          id == other.id;
}

/// The curated catalog of bundled scores. Const for the POC; exposed through a
/// provider so tests can override it with in-memory entries (no asset bundle).
@riverpod
List<CatalogEntry> scoreCatalog(Ref ref) => const [
  CatalogEntry(
    id: 'ode-to-joy',
    title: 'Ode to Joy (theme)',
    composer: 'Ludwig van Beethoven',
    assetPath: 'assets/scores/beginner/ode_to_joy.musicxml',
    level: PracticeLevel.beginner,
  ),
  CatalogEntry(
    id: 'twinkle',
    title: 'Twinkle, Twinkle, Little Star',
    composer: 'Traditional',
    assetPath: 'assets/scores/beginner/twinkle.musicxml',
    level: PracticeLevel.beginner,
  ),
  CatalogEntry(
    id: 'minuet-in-g',
    title: 'Minuet in G (BWV Anh. 114)',
    composer: 'Christian Petzold',
    assetPath: 'assets/scores/intermediate/minuet_in_g.musicxml',
    level: PracticeLevel.intermediate,
  ),
  CatalogEntry(
    id: 'prelude-e-minor',
    title: 'Prelude in E minor, Op. 28 No. 4',
    composer: 'Frédéric Chopin',
    assetPath: 'assets/scores/advanced/prelude_e_minor.musicxml',
    level: PracticeLevel.advanced,
  ),
];

/// The score the user picked in the library, or null before any selection.
/// Watched by the notation notifier to know which asset to load.
///
/// `keepAlive` so the selection survives the gap between the library setting it
/// and the partition screen mounting to watch it (an auto-dispose provider would
/// drop the state in that window).
@Riverpod(keepAlive: true)
class SelectedScore extends _$SelectedScore {
  @override
  CatalogEntry? build() => null;

  /// Records [entry] as the active score (drives the partition screen).
  void select(CatalogEntry entry) => state = entry;
}
