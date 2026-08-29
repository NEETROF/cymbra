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
import '../state/drum_calibration.dart';
import '../state/drum_calibration_notifier.dart';
import '../state/drum_input_mapping_notifier.dart';
import '../state/drum_kit.dart';
import '../state/midi_status_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/kit_piece_labels.dart';

/// Opens the kit calibration surface (change: add-drum-input-calibration).
///
/// Its **own** route, like the monitor and for the same reason: the settings
/// modal pauses the session while it is open, and this is a surface a player
/// plays into.
Future<void> openDrumCalibration(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const DrumCalibrationScreen()));

/// One destination for "what my kit sends": the stored mapping as a table, and
/// the guided pass that fills it in.
///
/// Both live here because they are two views of one thing — a player who opens
/// this to check an entry is one tap from re-learning it, and a player who
/// finishes a pass lands on the table that proves what it learned.
class DrumCalibrationScreen extends ConsumerWidget {
  const DrumCalibrationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final port = ref.watch(midiStatusProvider.select((s) => s.connected));
    final pass = ref.watch(drumCalibrationProvider);

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(l10n.calibrationTitle),
      ),
      body: switch (port) {
        // Nothing to store a mapping against, and nothing to play into.
        null => _NoDevice(message: l10n.calibrationNoDevice),
        _ when pass.isRunning => _Pass(state: pass),
        _ => _MappingTable(port: port, justFinished: pass.outcome),
      },
    );
  }
}

/// The guided pass: one piece at a time, named, with the ways out that matter —
/// a kit that lacks a piece skips it, a wrong pad steps back.
class _Pass extends ConsumerWidget {
  const _Pass({required this.state});

  final CalibrationState state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final notifier = ref.read(drumCalibrationProvider.notifier);
    final piece = state.currentPiece;
    if (piece == null) return const SizedBox.shrink();
    final conflict = state.conflict;

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          LinearProgressIndicator(
            value: state.total == 0 ? 0 : state.step / state.total,
            backgroundColor: CymbraColors.surfaceContainerHigh,
            color: CymbraColors.tertiary,
          ),
          const SizedBox(height: 12),
          Text(
            l10n.calibrationStepOf(state.step + 1, state.total),
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 12,
            ),
          ),
          const Spacer(),
          Text(
            key: const Key('calibration-prompt'),
            l10n.calibrationPrompt(kitPieceLabelOf(l10n, piece)),
            textAlign: TextAlign.center,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 28,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 16),
          if (conflict != null)
            _Conflict(
              conflict: conflict,
              onStrikeAgain: notifier.strikeAgain,
              onReassign: notifier.reassign,
            )
          else
            Text(
              key: const Key('calibration-waiting'),
              l10n.calibrationWaiting,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 14,
              ),
            ),
          const Spacer(),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              TextButton(
                key: const Key('calibration-back'),
                onPressed: state.step == 0 ? null : notifier.back,
                child: Text(l10n.calibrationBack),
              ),
              TextButton(
                key: const Key('calibration-abandon'),
                onPressed: () {
                  notifier.abandon();
                  Navigator.of(context).maybePop();
                },
                child: Text(l10n.calibrationAbandon),
              ),
              FilledButton(
                key: const Key('calibration-skip'),
                onPressed: notifier.skip,
                child: Text(l10n.calibrationSkip),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// A number another piece already holds. Reported, never resolved silently: on
/// a real kit this usually means the wrong pad was struck, and a quiet
/// reassignment would leave the mapping wrong in two places at once.
class _Conflict extends StatelessWidget {
  const _Conflict({
    required this.conflict,
    required this.onStrikeAgain,
    required this.onReassign,
  });

  final CalibrationConflict conflict;
  final VoidCallback onStrikeAgain;
  final VoidCallback onReassign;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Column(
      key: const Key('calibration-conflict'),
      children: [
        Text(
          l10n.calibrationConflict(kitPieceLabelOf(l10n, conflict.heldBy)),
          textAlign: TextAlign.center,
          style: const TextStyle(color: CymbraColors.error, fontSize: 14),
        ),
        const SizedBox(height: 8),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextButton(
              key: const Key('calibration-strike-again'),
              onPressed: onStrikeAgain,
              child: Text(l10n.calibrationStrikeAgain),
            ),
            const SizedBox(width: 8),
            TextButton(
              key: const Key('calibration-reassign'),
              onPressed: onReassign,
              child: Text(l10n.calibrationReassign),
            ),
          ],
        ),
      ],
    );
  }
}

/// The stored mapping, piece by piece — and what each number would otherwise
/// have meant, so an entry recorded from the wrong pad is visible rather than
/// merely wrong.
class _MappingTable extends ConsumerWidget {
  const _MappingTable({required this.port, required this.justFinished});

  final String port;
  final CalibrationOutcome justFinished;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final mapping = ref.watch(activeDrumMappingProvider);
    final store = ref.read(drumInputMappingStoreProvider.notifier);
    final calibration = ref.read(drumCalibrationProvider.notifier);
    // Listed in the pass's own order, so the table and the pass read the same
    // way — and any entry from a build that knew more pieces still shows, at
    // the end, rather than silently disappearing.
    final ordered = [
      for (final id in kCalibrationPieceOrder)
        if (mapping.byPiece.containsKey(id)) id,
      for (final id in mapping.byPiece.keys)
        if (!kCalibrationPieceOrder.contains(id)) id,
    ];

    return ListView(
      key: const Key('calibration-mapping-list'),
      padding: const EdgeInsets.all(16),
      children: [
        if (justFinished == CalibrationOutcome.completed)
          _Finished(count: mapping.byPiece.length),
        Text(
          l10n.calibrationMappingTitle,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 12,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.4,
          ),
        ),
        const SizedBox(height: 8),
        if (ordered.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 12),
            child: Text(
              key: const Key('calibration-mapping-empty'),
              l10n.calibrationMappingEmpty,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 14,
              ),
            ),
          )
        else
          for (final id in ordered)
            _MappingRow(
              key: Key('calibration-row-$id'),
              pieceId: id,
              number: mapping.byPiece[id]!,
              onClear: () => store.clearPiece(port, id),
            ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('calibration-start'),
          onPressed: calibration.start,
          child: Text(
            ordered.isEmpty ? l10n.calibrationOpen : l10n.calibrationRestart,
          ),
        ),
        if (ordered.isNotEmpty) ...[
          const SizedBox(height: 8),
          TextButton(
            key: const Key('calibration-clear-all'),
            onPressed: () => store.clear(port),
            child: Text(l10n.calibrationClearAll),
          ),
        ],
      ],
    );
  }
}

class _MappingRow extends StatelessWidget {
  const _MappingRow({
    required this.pieceId,
    required this.number,
    required this.onClear,
    super.key,
  });

  final String pieceId;
  final int number;
  final VoidCallback onClear;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final standard = canonicalGmOfPiece(pieceId);
    // What the sent number means on the standard map — the line that makes a
    // mis-recorded entry legible ("your snare sends 31, which normally means
    // Sticks") instead of merely wrong.
    final wouldMean = gmPercussionName(number);
    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(
        kitPieceLabelOf(l10n, pieceId),
        style: const TextStyle(color: CymbraColors.onSurface, fontSize: 15),
      ),
      subtitle: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.calibrationMappingRow(number, standard ?? number),
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 12,
            ),
          ),
          if (wouldMean != null)
            Text(
              l10n.calibrationMappingWouldMean(number, wouldMean),
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 11,
              ),
            ),
        ],
      ),
      trailing: TextButton(
        onPressed: onClear,
        child: Text(l10n.calibrationClearPiece),
      ),
    );
  }
}

class _Finished extends StatelessWidget {
  const _Finished({required this.count});

  final int count;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      key: const Key('calibration-finished'),
      padding: const EdgeInsets.only(bottom: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.calibrationDoneTitle,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 20,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            l10n.calibrationDoneBody(count),
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 14,
            ),
          ),
        ],
      ),
    );
  }
}

class _NoDevice extends StatelessWidget {
  const _NoDevice({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.all(32),
      child: Text(
        key: const Key('calibration-no-device'),
        message,
        textAlign: TextAlign.center,
        style: const TextStyle(
          color: CymbraColors.onSurfaceVariant,
          fontSize: 14,
        ),
      ),
    ),
  );
}
