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

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../state/placement_core.dart';
import '../state/placement_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The end of a placement run (change: add-music-placement-test): the
/// proposed starting unit, adjustable one up or down (or any unit, by its
/// chip), confirmed into the device's pointer — or "start from the
/// beginning". Replaces the lesson player's route, so leaving it lands back
/// on the door.
class PlacementResultScreen extends ConsumerStatefulWidget {
  const PlacementResultScreen({
    super.key,
    required this.outcome,
    required this.units,
    required this.unitTitles,
    this.missedTitle,
  });

  final PlacementOutcome outcome;

  /// The path's units in order.
  final List<String> units;
  final Map<String, InlineText> unitTitles;

  /// The prompt of the first missed probe, when one was missed.
  final String? missedTitle;

  @override
  ConsumerState<PlacementResultScreen> createState() =>
      _PlacementResultScreenState();
}

class _PlacementResultScreenState extends ConsumerState<PlacementResultScreen> {
  late int _index = math.max(0, widget.units.indexOf(widget.outcome.startUnit));

  void _confirm() {
    unawaited(ref.read(placementProvider.notifier).set(widget.units[_index]));
    Navigator.of(context).pop();
  }

  void _restart() {
    unawaited(ref.read(placementProvider.notifier).clear());
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final units = widget.units;
    final unit = units[_index];
    final n = unitNumberOf(units, unit);
    final title = resolveInline(widget.unitTitles[unit] ?? const {}, lang);
    final missed = widget.missedTitle;
    final summary =
        '${l10n.placementSolved(widget.outcome.solved)} · '
        '${missed == null ? l10n.placementAllSolved : l10n.placementMissedAt(missed)}';
    return Scaffold(
      key: const Key('placement-result'),
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(l10n.placementResultAppBar),
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.fromLTRB(20, 20, 20, 24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 560),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.placementResultTitle.toUpperCase(),
                  style: const TextStyle(
                    color: CymbraColors.primary,
                    fontSize: 13,
                    fontWeight: FontWeight.w800,
                    letterSpacing: 0.8,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  l10n.placementUnitNumber(n),
                  key: const Key('placement-unit-number'),
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 30,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                if (title.isNotEmpty)
                  Text(
                    title,
                    style: const TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 16,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                const SizedBox(height: 10),
                Text(
                  summary,
                  key: const Key('placement-summary'),
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 18),
                Row(
                  children: [
                    IconButton(
                      key: const Key('placement-minus'),
                      icon: const Icon(Icons.remove),
                      onPressed: _index > 0
                          ? () => setState(() => _index--)
                          : null,
                    ),
                    Expanded(
                      child: Wrap(
                        spacing: 6,
                        runSpacing: 6,
                        alignment: WrapAlignment.center,
                        children: [
                          for (var i = 0; i < units.length; i++)
                            ChoiceChip(
                              key: Key('placement-unit-${units[i]}'),
                              label: Text(
                                l10n.placementUnitShort(
                                  unitNumberOf(units, units[i]),
                                ),
                              ),
                              selected: i == _index,
                              visualDensity: VisualDensity.compact,
                              onSelected: (_) => setState(() => _index = i),
                            ),
                        ],
                      ),
                    ),
                    IconButton(
                      key: const Key('placement-plus'),
                      icon: const Icon(Icons.add),
                      onPressed: _index < units.length - 1
                          ? () => setState(() => _index++)
                          : null,
                    ),
                  ],
                ),
                const SizedBox(height: 10),
                Text(
                  l10n.placementSkippedNote,
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 12,
                  ),
                ),
                const SizedBox(height: 20),
                SizedBox(
                  width: double.infinity,
                  child: FilledButton(
                    key: const Key('placement-confirm'),
                    onPressed: _confirm,
                    child: Text(l10n.placementConfirm(n)),
                  ),
                ),
                const SizedBox(height: 4),
                Center(
                  child: TextButton(
                    key: const Key('placement-restart'),
                    onPressed: _restart,
                    child: Text(
                      l10n.placementRestart,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
