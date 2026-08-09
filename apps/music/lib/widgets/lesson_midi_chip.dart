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
import '../layout/device_class.dart';
import '../state/midi_status_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The lesson player's MIDI status pill (change: add-notation-courses) —
/// the game's indicator language verbatim (status dot + usb icon + label in a
/// bordered pill; green connected, amber while ports await, gray when none),
/// with one lesson-specific power: tapping it opens the keyboard choice, so a
/// learner plugs their piano without leaving the lesson.
class LessonMidiChip extends ConsumerWidget {
  const LessonMidiChip({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final status = ref.watch(midiStatusProvider);
    final connected = status.connected != null;
    final hasPorts = status.ports.isNotEmpty;

    final Color color;
    final String label;
    final IconData icon;
    if (connected) {
      color = CymbraColors.tertiary;
      icon = Icons.usb;
      label = l10n.midiConnected;
    } else if (hasPorts) {
      color = CymbraColors.secondary;
      icon = Icons.usb;
      label = l10n.midiConnecting;
    } else {
      color = CymbraColors.outline;
      icon = Icons.usb_off;
      label = l10n.midiStatusNone;
    }

    return Center(
      child: PopupMenuButton<String>(
        key: const Key('lesson-midi-chip'),
        tooltip: status.connected ?? label,
        // Ports can appear at any moment (hot-plug): re-read them exactly when
        // the learner looks.
        onOpened: () => ref.read(midiStatusProvider.notifier).refresh(),
        onSelected: (port) =>
            ref.read(midiStatusProvider.notifier).select(port),
        itemBuilder: (context) {
          final s = ref.read(midiStatusProvider);
          if (s.ports.isEmpty) {
            return [
              PopupMenuItem<String>(
                enabled: false,
                child: Text(l10n.midiNoDeviceDetected),
              ),
            ];
          }
          return [
            for (final port in s.ports)
              CheckedPopupMenuItem<String>(
                key: Key('lesson-midi-port-$port'),
                value: port,
                checked: port == s.connected,
                child: Text(port, overflow: TextOverflow.ellipsis),
              ),
          ];
        },
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: color.withValues(alpha: 0.6)),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 9,
                height: 9,
                decoration: BoxDecoration(
                  color: color,
                  shape: BoxShape.circle,
                  boxShadow: [
                    BoxShadow(
                      color: color.withValues(alpha: 0.7),
                      blurRadius: 6,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 8),
              Icon(icon, size: 16, color: color),
              // Tight phone app bars collapse to dot + icon, like the game.
              if (!context.isPhoneLayout) ...[
                const SizedBox(width: 6),
                ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 160),
                  child: Text(
                    status.connected ?? label,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 13,
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
