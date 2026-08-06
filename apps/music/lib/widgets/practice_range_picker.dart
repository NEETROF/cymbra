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

import '../l10n/gen/app_localizations.dart';
import '../state/player_notifier.dart';
import '../state/practice_settings_store.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import 'practice_range_controls.dart';

/// Opens the measure-range picker for a **selective (practice) run** — the
/// second entry point to practice, offered by the end-of-run summary so a player
/// can drill what they just missed (change: add-measure-range-practice, D6).
///
/// Returns `true` when a range was chosen and applied (a practice run is now
/// armed), `false` when the player backed out. A piece with fewer than two
/// measures has no narrower range to pick, so the picker refuses to open and
/// returns `false`.
Future<bool> showPracticeRangePicker(BuildContext context) async {
  final chosen = await showDialog<bool>(
    context: context,
    builder: (_) => const _PracticeRangeDialog(),
  );
  return chosen ?? false;
}

class _PracticeRangeDialog extends ConsumerStatefulWidget {
  const _PracticeRangeDialog();

  @override
  ConsumerState<_PracticeRangeDialog> createState() =>
      _PracticeRangeDialogState();
}

class _PracticeRangeDialogState extends ConsumerState<_PracticeRangeDialog> {
  late int _from;
  late int _to;

  @override
  void initState() {
    super.initState();
    final data = ref.read(playerProvider);
    _from = data.practiceStartMeasure ?? 0;
    _to = data.practiceEndMeasure ?? (data.lastMeasureIndex ?? 0);
    if (!data.isSelectiveRun) _prefillFromSaved();
  }

  /// Pre-fills from this score's saved practice settings, clamped to its current
  /// measure count (design D7), so the picker opens on what was last drilled.
  Future<void> _prefillFromSaved() async {
    final scoreKey = pieceIdentityOf(
      ref.read(selectedScoreProvider),
      ref.read(playerProvider).title,
    );
    final saved = await ref.read(practiceSettingsStoreProvider).load(scoreKey);
    if (!mounted) return;
    final applied = saved?.clampedTo(ref.read(playerProvider).measureCount);
    if (applied == null) return;
    setState(() {
      _from = applied.startMeasure;
      _to = applied.endMeasure;
    });
  }

  void _start() {
    final notifier = ref.read(playerProvider.notifier);
    notifier.setPracticeRange(_from, _to);
    Navigator.of(context).pop(true);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final last = ref.watch(playerProvider).lastMeasureIndex;
    // Nothing to pick on a single-measure (or timing-less) piece.
    if (last == null || last < 1) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) Navigator.of(context).pop(false);
      });
      return const SizedBox.shrink();
    }
    // Cap the height so the action stays reachable on a short phone-landscape
    // viewport: the controls scroll, the buttons stay pinned below.
    final maxHeight = MediaQuery.of(context).size.height * 0.92;
    return Dialog(
      key: const Key('practice-range-picker'),
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 420, maxHeight: maxHeight),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                l10n.practicePickerTitle,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 18,
                  fontWeight: FontWeight.w700,
                ),
              ),
              Flexible(
                child: SingleChildScrollView(
                  child: PracticeRangeControls(
                    lastMeasure: last,
                    fromMeasure: _from,
                    toMeasure: _to,
                    onFromChanged: (v) => setState(() {
                      _from = v;
                      if (_to < v) _to = v;
                    }),
                    onToChanged: (v) => setState(() {
                      _to = v;
                      if (_from > v) _from = v;
                    }),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(false),
                    child: Text(l10n.cancel),
                  ),
                  const SizedBox(width: 8),
                  FilledButton(
                    key: const Key('practice-picker-start'),
                    onPressed: _start,
                    child: Text(l10n.practicePickerStart),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
