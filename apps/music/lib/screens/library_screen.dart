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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import 'player_screen.dart';

/// Start screen: the bundled score catalog, grouped by practice level. Tapping
/// an entry records it as the selected score and opens the partition screen.
class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final catalog = ref.watch(scoreCatalogProvider);

    return Scaffold(
      backgroundColor: CymbraColors.surfaceContainerLow,
      appBar: AppBar(
        title: const Text('Cymbra — Score Library'),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: ListView(
        padding: const EdgeInsets.symmetric(vertical: 8),
        children: [
          for (final level in PracticeLevel.values)
            ..._levelSection(
              context,
              ref,
              level,
              catalog.where((e) => e.level == level).toList(),
            ),
        ],
      ),
    );
  }

  List<Widget> _levelSection(
    BuildContext context,
    WidgetRef ref,
    PracticeLevel level,
    List<CatalogEntry> entries,
  ) {
    if (entries.isEmpty) return const [];
    return [
      Padding(
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
        child: Text(
          level.label,
          style: const TextStyle(
            color: CymbraColors.primary,
            fontSize: 15,
            fontWeight: FontWeight.w800,
            letterSpacing: 0.5,
          ),
        ),
      ),
      for (final entry in entries) _EntryTile(entry: entry),
    ];
  }
}

class _EntryTile extends ConsumerWidget {
  final CatalogEntry entry;
  const _EntryTile({required this.entry});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ListTile(
      leading: const Icon(Icons.music_note, color: CymbraColors.secondary),
      title: Text(
        entry.title,
        style: const TextStyle(color: CymbraColors.onSurface),
      ),
      subtitle: Text(
        entry.composer,
        style: const TextStyle(color: CymbraColors.onSurfaceVariant),
      ),
      trailing: const Icon(
        Icons.chevron_right,
        color: CymbraColors.onSurfaceVariant,
      ),
      onTap: () {
        ref.read(selectedScoreProvider.notifier).select(entry);
        Navigator.of(
          context,
        ).push(MaterialPageRoute<void>(builder: (_) => const PlayerScreen()));
      },
    );
  }
}
