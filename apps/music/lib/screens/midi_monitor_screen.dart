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
import '../state/drum_kit.dart';
import '../state/midi_monitor.dart';
import '../state/midi_monitor_notifier.dart';
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';

/// Opens the MIDI input monitor (change: add-drum-input-calibration).
///
/// Its **own** route rather than a section of the settings modal: that modal
/// pauses the session while it is open, which is precisely wrong for a surface
/// whose purpose is watching live input arrive.
Future<void> openMidiMonitor(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const MidiMonitorScreen()));

/// A live read-out of the MIDI events the app is receiving, and what it made of
/// each one.
///
/// The app plays back exactly the number an instrument sends and resolves a
/// stroke to a kit piece by that same number — correct for a module on the
/// General MIDI map, silently wrong for a kit whose pads have been reassigned
/// or whose zones have no standard number at all. A stroke the app cannot place
/// lights no pad, releases no Wait-Mode gate and earns no credit, and until now
/// nothing on screen ever showed what had actually arrived. This shows it.
///
/// A diagnostic, not a mapping: it makes the invisible visible and changes
/// nothing.
class MidiMonitorScreen extends ConsumerWidget {
  const MidiMonitorScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final entries = ref.watch(midiMonitorProvider);
    final player = ref.watch(playerProvider);

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(l10n.midiMonitorTitle),
        actions: [
          IconButton(
            key: const Key('midi-monitor-clear'),
            tooltip: l10n.midiMonitorClear,
            onPressed: () => ref.read(midiMonitorProvider.notifier).clear(),
            icon: const Icon(Icons.clear_all),
          ),
        ],
      ),
      body: Column(
        children: [
          _Header(device: player.connectedDevice, entryCount: entries.length),
          const Divider(height: 1, color: CymbraColors.outlineVariant),
          Expanded(
            child: entries.isEmpty
                ? _Empty(connected: player.midiConnected)
                : ListView.separated(
                    key: const Key('midi-monitor-list'),
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    itemCount: entries.length,
                    separatorBuilder: (_, _) => const Divider(
                      height: 1,
                      indent: 16,
                      endIndent: 16,
                      color: CymbraColors.outlineVariant,
                    ),
                    itemBuilder: (_, i) => _EntryRow(entry: entries[i]),
                  ),
          ),
        ],
      ),
    );
  }
}

/// What the monitor is watching, and the one sentence explaining what to do
/// with it.
class _Header extends StatelessWidget {
  const _Header({required this.device, required this.entryCount});

  final String? device;
  final int entryCount;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      color: CymbraColors.surfaceContainerLowest,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            device ?? l10n.midiNoDeviceDetected,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            l10n.midiMonitorHint,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }
}

class _Empty extends StatelessWidget {
  const _Empty({required this.connected});

  final bool connected;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Text(
          connected ? l10n.midiMonitorWaiting : l10n.midiMonitorNoDevice,
          key: const Key('midi-monitor-empty'),
          textAlign: TextAlign.center,
          style: const TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
      ),
    );
  }
}

/// One received event: the number as it arrived, then what the app made of it.
///
/// The raw number leads, in a monospaced face, because it is the fact the
/// player came here for and the one they will read back to us. Everything to
/// its right is interpretation.
class _EntryRow extends StatelessWidget {
  const _EntryRow({required this.entry});

  final MidiMonitorEntry entry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final inert = entry.isInert;
    return Opacity(
      // A note-off is bookkeeping; it is shown so a kit that sends none is
      // visibly a kit that sends none, but it must not compete with the attack.
      opacity: entry.isNoteOn ? 1 : 0.55,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 52,
              child: Text(
                '${entry.pitch}',
                // Tabular figures rather than a monospaced *family*: the app
                // ships no monospaced face, and naming one leaves the number —
                // the single most important thing on this screen — rendered as
                // empty boxes. This keeps the column aligned using the face we
                // actually have.
                style: TextStyle(
                  fontFeatures: const [FontFeature.tabularFigures()],
                  fontSize: 22,
                  fontWeight: FontWeight.w700,
                  color: inert ? CymbraColors.error : CymbraColors.onSurface,
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    _reading(l10n, entry),
                    style: TextStyle(
                      color: inert
                          ? CymbraColors.error
                          : CymbraColors.onSurface,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  // Only when the calibration changed the meaning: one number
                  // where nothing was translated keeps the row quiet, and the
                  // arrow appears exactly where a player needs to check what
                  // their mapping is doing (change: add-drum-input-calibration).
                  if (entry.wasTranslated)
                    Text(
                      key: const Key('midi-monitor-translated'),
                      l10n.midiMonitorTranslated(
                        entry.pitch,
                        entry.mappedPitch,
                      ),
                      style: const TextStyle(
                        color: CymbraColors.tertiary,
                        fontSize: 12,
                      ),
                    ),
                  const SizedBox(height: 2),
                  Text(
                    l10n.midiMonitorEventDetail(
                      entry.velocity,
                      entry.channel + 1,
                    ),
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ),
            Icon(
              entry.isNoteOn ? Icons.south_east : Icons.north_west,
              size: 16,
              color: CymbraColors.onSurfaceVariant,
            ),
          ],
        ),
      ),
    );
  }

  /// What the app made of this number, in the player's language.
  static String _reading(AppLocalizations l10n, MidiMonitorEntry e) =>
      switch (e.resolution) {
        MidiResolution.pitched => noteName(e.pitch),
        MidiResolution.matchedPiece =>
          _pieceLabel(l10n, e) ?? e.gmName ?? 'GM ${e.pitch}',
        // Named by the standard, but this score's kit does not use it: it will
        // sound, and it will do nothing else.
        MidiResolution.outsideThisKit => l10n.midiMonitorOutsideKit(
          e.gmName ?? 'GM ${e.pitch}',
        ),
        // Outside the standard map altogether — the shape of "I hit the pad and
        // nothing happened".
        MidiResolution.outsideTheMap => l10n.midiMonitorOutsideMap(
          kGmPercussionLowest,
          kGmPercussionHighest,
        ),
      };

  static String? _pieceLabel(AppLocalizations l10n, MidiMonitorEntry e) =>
      switch (e.pieceLabelKey) {
        'kitPieceHiHat' => l10n.kitPieceHiHat,
        'kitPieceSnare' => l10n.kitPieceSnare,
        'kitPieceTomHigh' => l10n.kitPieceTomHigh,
        'kitPieceTomHiMid' => l10n.kitPieceTomHiMid,
        'kitPieceTomLowMid' => l10n.kitPieceTomLowMid,
        'kitPieceTomLow' => l10n.kitPieceTomLow,
        'kitPieceTomFloorHigh' => l10n.kitPieceTomFloorHigh,
        'kitPieceTomFloorLow' => l10n.kitPieceTomFloorLow,
        'kitPieceRide' => l10n.kitPieceRide,
        'kitPieceCrash' => l10n.kitPieceCrash,
        'kitPieceCrash2' => l10n.kitPieceCrash2,
        'kitPieceSplash' => l10n.kitPieceSplash,
        'kitPieceChina' => l10n.kitPieceChina,
        'kitPieceKick' => l10n.kitPieceKick,
        _ => e.pieceGmName ?? (e.gmName ?? l10n.kitPieceKick),
      };
}

/// Scientific note name of a MIDI pitch (C4 = 60) — the keyboard reading, for a
/// keyboard score where kit vocabulary means nothing.
String noteName(int pitch) {
  const names = [
    'C',
    'C#',
    'D',
    'D#',
    'E',
    'F',
    'F#',
    'G',
    'G#',
    'A',
    'A#',
    'B',
  ];
  final octave = (pitch ~/ 12) - 1;
  return '${names[pitch % 12]}$octave';
}
