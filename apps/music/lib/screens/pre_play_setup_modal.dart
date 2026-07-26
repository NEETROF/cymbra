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
import '../services/platform_info.dart';
import '../state/notation_notifier.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/difficulty_badge.dart';
import '../widgets/otg_guidance.dart';
import '../widgets/setting_option_row.dart';

/// Shows the pre-play setup modal, centered over the player. Lets the user review
/// the score and set the hands, tempo, metronome and MIDI device before playing.
/// **Validate** applies the choices; the close (X) keeps the current settings.
/// Either way the caller stays on the player.
Future<void> showPrePlaySetup(BuildContext context) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => const PopScope(canPop: false, child: _PrePlaySetupDialog()),
);

class _PrePlaySetupDialog extends ConsumerStatefulWidget {
  const _PrePlaySetupDialog();

  @override
  ConsumerState<_PrePlaySetupDialog> createState() =>
      _PrePlaySetupDialogState();
}

class _PrePlaySetupDialogState extends ConsumerState<_PrePlaySetupDialog> {
  // Draft settings — applied only on Validate.
  late Hand _hands;
  late double _speed;
  late bool _metronome;
  late String? _port; // null = auto (first real device)

  @override
  void initState() {
    super.initState();
    final data = ref.read(playerProvider);
    _hands = data.selectedHands;
    _speed = data.speed;
    _metronome = data.metronomeEnabled;
    _port = data.connectedDevice;
  }

  void _apply() {
    final notifier = ref.read(playerProvider.notifier);
    final current = ref.read(playerProvider);
    if (_hands != current.selectedHands) notifier.setSelectedHands(_hands);
    if (_speed != current.speed) notifier.setSpeed(_speed);
    if (_metronome != current.metronomeEnabled) notifier.toggleMetronome();
    if (_port != current.connectedDevice) notifier.selectMidiPort(_port);
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final data = ref.watch(playerProvider);
    final entry = ref.watch(selectedScoreProvider);
    final meta = ref.watch(notationProvider).document?.meta;

    final title = entry?.title ?? meta?.title ?? l10n.nowPlaying('').trim();
    final composer = entry?.composer ?? meta?.composer;
    final maxHeight = MediaQuery.of(context).size.height * 0.92;

    return Dialog(
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 460, maxHeight: maxHeight),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _header(l10n, title, composer, entry?.level),
            Flexible(
              child: SingleChildScrollView(
                padding: const EdgeInsets.fromLTRB(20, 4, 20, 8),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _facts(l10n, data, entry),
                    if (data.hasMultipleStaves) _handsSection(l10n),
                    _tempoSection(l10n, data),
                    _midiSection(l10n, data),
                  ],
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 8, 20, 16),
              child: FilledButton(
                onPressed: _apply,
                child: Text(l10n.prePlayStart),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _header(
    AppLocalizations l10n,
    String title,
    String? composer,
    PracticeLevel? level,
  ) => Padding(
    padding: const EdgeInsets.fromLTRB(20, 16, 8, 4),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 20,
                  fontWeight: FontWeight.w700,
                ),
              ),
              if (composer != null && composer.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 2),
                  child: Text(
                    composer,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 14,
                    ),
                  ),
                ),
              if (level != null)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: DifficultyBadge(level: level, l10n: l10n),
                  ),
                ),
            ],
          ),
        ),
        IconButton(
          icon: const Icon(Icons.close, color: CymbraColors.onSurfaceVariant),
          tooltip: l10n.cancel,
          onPressed: () => Navigator.of(context).pop(),
        ),
      ],
    ),
  );

  /// Key / time-signature / marked-tempo chips — each labelled (so a novice knows
  /// what it is) and shown only when known.
  Widget _facts(AppLocalizations l10n, PlayerData data, CatalogEntry? entry) {
    // Note naming: letters (C, D, E…) in English, solfège (Do, Ré, Mi…) in the
    // Latin-language locales.
    final lang = Localizations.localeOf(context).languageCode;
    final key = _keyName(
      data.keyFifths,
      solfege: lang != 'en',
      frenchRe: lang == 'fr',
    );
    final chips = <Widget>[
      _chip(Icons.piano, '${l10n.prePlayKey} · $key'),
      _chip(
        Icons.timer_outlined,
        '${l10n.prePlayMeter} · ${data.beats}/${data.beatType}',
      ),
      if (entry?.tempoBpm != null)
        _chip(Icons.speed, l10n.tempo(entry!.tempoBpm!)),
    ];
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Wrap(spacing: 8, runSpacing: 8, children: chips),
    );
  }

  Widget _chip(IconData icon, String label) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
    decoration: BoxDecoration(
      color: CymbraColors.surfaceContainerHigh,
      borderRadius: BorderRadius.circular(8),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(icon, size: 16, color: CymbraColors.onSurfaceVariant),
        const SizedBox(width: 6),
        Text(label, style: const TextStyle(color: CymbraColors.onSurface)),
      ],
    ),
  );

  Widget _sectionTitle(String label) => Padding(
    padding: const EdgeInsets.only(top: 12, bottom: 2),
    child: Text(
      label,
      style: const TextStyle(
        color: CymbraColors.onSurfaceVariant,
        fontSize: 12,
        fontWeight: FontWeight.w700,
        letterSpacing: 0.4,
      ),
    ),
  );

  Widget _handsSection(AppLocalizations l10n) {
    const hands = Hand.values;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _sectionTitle(l10n.prePlayHands),
        // A 3-way toggle (Left / Right / Both) — compact, one row instead of
        // three radio rows.
        LayoutBuilder(
          builder: (context, constraints) {
            // Three equal segments filling the width (account for the borders).
            final segment = (constraints.maxWidth - 4) / hands.length;
            return ToggleButtons(
              isSelected: [for (final h in hands) h == _hands],
              onPressed: (i) => setState(() => _hands = hands[i]),
              borderRadius: BorderRadius.circular(10),
              borderColor: CymbraColors.surfaceContainerHighest,
              selectedBorderColor: CymbraColors.tertiary,
              color: CymbraColors.onSurfaceVariant,
              selectedColor: CymbraColors.onSurface,
              fillColor: CymbraColors.tertiary.withValues(alpha: 0.22),
              constraints: BoxConstraints(minHeight: 44, minWidth: segment),
              children: [for (final h in hands) Text(handLabel(l10n, h))],
            );
          },
        ),
      ],
    );
  }

  Widget _tempoSection(AppLocalizations l10n, PlayerData data) {
    final effectiveBpm = (data.bpm * _speed).round();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _sectionTitle(l10n.prePlayTempo),
        Row(
          children: [
            Expanded(
              child: Slider(
                value: _speed,
                min: 0.25,
                max: 2.0,
                divisions: 7,
                label: '${_speed.toStringAsFixed(2)}×',
                onChanged: (v) => setState(() => _speed = v),
              ),
            ),
            SizedBox(
              width: 96,
              child: Text(
                '${_speed.toStringAsFixed(2)}×  ·  ${l10n.tempo(effectiveBpm)}',
                textAlign: TextAlign.end,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 11,
                ),
              ),
            ),
          ],
        ),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(
            l10n.metronome,
            style: const TextStyle(color: CymbraColors.onSurface),
          ),
          value: _metronome,
          onChanged: (v) => setState(() => _metronome = v),
        ),
      ],
    );
  }

  Widget _midiSection(AppLocalizations l10n, PlayerData data) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _sectionTitle(l10n.settingsCategoryMidiDevice),
      SettingOptionRow(
        selected: _port == null,
        label: l10n.midiAutoFirstDevice,
        onTap: () => setState(() => _port = null),
      ),
      for (final port in data.midiPorts)
        SettingOptionRow(
          selected: _port == port,
          label: port,
          onTap: () => setState(() => _port = port),
        ),
      if (data.midiPorts.isEmpty)
        // Android: USB-OTG / charge-only-cable advice; other platforms: a plain
        // "no device" row.
        if (ref.watch(isAndroidProvider))
          const OtgGuidance()
        else
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Text(
              l10n.midiNoDeviceDetected,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ),
    ],
  );
}

/// The tonic name of a key from its number of sharps (+) / flats (−). Uses
/// letter names (C, D, E…) or solfège (Do, Ré/Re, Mi…) so it reads for a novice
/// in their language. Mode is unknown from the key signature alone, so this is
/// the conventional major-tonic reading.
String _keyName(int fifths, {required bool solfege, required bool frenchRe}) {
  if (!solfege) {
    return switch (fifths) {
      -7 => 'C♭',
      -6 => 'G♭',
      -5 => 'D♭',
      -4 => 'A♭',
      -3 => 'E♭',
      -2 => 'B♭',
      -1 => 'F',
      0 => 'C',
      1 => 'G',
      2 => 'D',
      3 => 'A',
      4 => 'E',
      5 => 'B',
      6 => 'F♯',
      7 => 'C♯',
      _ => 'C',
    };
  }
  final re = frenchRe ? 'Ré' : 'Re';
  return switch (fifths) {
    -7 => 'Do♭',
    -6 => 'Sol♭',
    -5 => '$re♭',
    -4 => 'La♭',
    -3 => 'Mi♭',
    -2 => 'Si♭',
    -1 => 'Fa',
    0 => 'Do',
    1 => 'Sol',
    2 => re,
    3 => 'La',
    4 => 'Mi',
    5 => 'Si',
    6 => 'Fa♯',
    7 => 'Do♯',
    _ => 'Do',
  };
}
