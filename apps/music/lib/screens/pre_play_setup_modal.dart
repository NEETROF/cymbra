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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../layout/device_class.dart';
import '../services/platform_info.dart';
import '../state/audio_routing.dart';
import '../state/coaching_notifier.dart';
import '../state/notation_notifier.dart';
import '../state/note_label.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../state/player_preferences.dart';
import '../state/score_catalog.dart';
import '../state/selected_piano.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/coach_mark.dart';
import '../widgets/difficulty_badge.dart';
import '../widgets/leaderboard_view.dart';
import '../widgets/otg_guidance.dart';
import '../widgets/setting_option_row.dart';
import '../widgets/sound_output_section.dart';
import '../widgets/sound_selector_field.dart';

/// Shows the pre-play setup modal, centered over the player. Lets the user review
/// the score and set the sound, hands, tempo, metronome, MIDI device and keyboard
/// display before playing. **Validate** applies the choices; the close (X) keeps
/// the current settings. Either way the caller stays on the player.
///
/// It doubles as the in-game settings surface (the gear button reopens it): pass
/// [inGame] so the action reads "Apply" rather than "Play".
Future<void> showPrePlaySetup(BuildContext context, {bool inGame = false}) =>
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => PopScope(
        canPop: false,
        // The guided player sequence (change: add-welcome-onboarding, D8) walks
        // the controls that live in this surface, so it is armed from here.
        child: _PlayerTourStarter(
          // The sound-output section's side effects (change:
          // add-audio-output-routing) live here, at the section's root, rather
          // than in any build method.
          child: SoundOutputListener(
            child: _PrePlaySetupDialog(inGame: inGame),
          ),
        ),
      ),
    );

/// Dedicated listener (CLAUDE.md rule): starts the guided player sequence once
/// the coaching flags are known and this surface — which holds the controls it
/// points at — is on screen. Starting is idempotent: it is a no-op when the
/// sequence already ran (unless a replay was armed from help) or is running.
class _PlayerTourStarter extends ConsumerStatefulWidget {
  const _PlayerTourStarter({required this.child});

  final Widget child;

  @override
  ConsumerState<_PlayerTourStarter> createState() => _PlayerTourStarterState();
}

class _PlayerTourStarterState extends ConsumerState<_PlayerTourStarter> {
  @override
  Widget build(BuildContext context) {
    // The flags load asynchronously: react when they arrive, and cover the
    // already-loaded case (every visit after the first) directly.
    ref.listen(coachingProvider.select((s) => s.loaded), (_, loaded) {
      if (loaded) _start();
    });
    if (ref.read(coachingProvider).loaded) _start();
    return widget.child;
  }

  /// Deferred to the next frame so the controls are laid out before the
  /// spotlight looks for their rects.
  void _start() => WidgetsBinding.instance.addPostFrameCallback((_) {
    if (mounted) ref.read(coachingProvider.notifier).startPlayerTour();
  });
}

/// Dedicated listener (CLAUDE.md rule): reacts when the guided player sequence
/// advances to [PlayerCoachStep.measureRewind] — the one step whose control
/// (the transport rewind button, change: add-in-game-measure-selection) lives
/// on the player screen behind the setup dialog rather than inside it. The
/// dialog then applies its drafts and closes via [onRewindStep], so the
/// spotlight lands on a control the user can actually see.
class _CoachStepListener extends ConsumerWidget {
  const _CoachStepListener({required this.onRewindStep, required this.child});

  final VoidCallback onRewindStep;
  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(coachingProvider.select((s) => s.step), (prev, next) {
      if (prev != next && next == PlayerCoachStep.measureRewind) {
        onRewindStep();
      }
    });
    return child;
  }
}

class _PrePlaySetupDialog extends ConsumerStatefulWidget {
  const _PrePlaySetupDialog({required this.inGame});

  /// Whether reopened in-game (changes the primary action's label).
  final bool inGame;

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
  late String _soundId;
  late KeyboardRangeMode _range;
  late bool _keyboardVisible;
  late NoteReadingAid _readingAid;
  ScoreSize? _scoreSizeDraft;
  late NotationTheme _notationTheme;
  late bool _invertedKit;

  /// When true, the modal body shows this piece's leaderboard in place of the
  /// setup controls (a toggle via the trophy in the header) — so the board is
  /// viewed inline, not as a modal stacked on this modal (change: add-play-
  /// leaderboards).
  bool _showBoard = false;

  /// Captured in [initState] so [dispose] never touches `ref` after teardown.
  late final Coaching _coaching;

  @override
  void initState() {
    super.initState();
    _coaching = ref.read(coachingProvider.notifier);
    // Re-read the platform's outputs each time the modal opens: desktop has no
    // route-change events, so without this the sound-output list would stay a
    // snapshot from app launch — a piano plugged in since would never appear.
    // Fire-and-forget; the section re-renders when the fresh state lands.
    unawaited(ref.read(audioRoutingProvider.notifier).refresh());
    final data = ref.read(playerProvider);
    _hands = data.selectedHands;
    _speed = data.speed;
    _metronome = data.metronomeEnabled;
    // The user's *choice*, not the device that happens to be connected. Seeding
    // this from `connectedDevice` made the control unusable: "auto" was
    // re-displayed as the connected port on every open, so a choice never
    // appeared to stick — and Apply then always saw a change and re-selected the
    // port, which on Android leaks a MIDI port and eventually drops the whole USB
    // device (audio included).
    _port = ref.read(playerPreferencesProvider).midiPort;
    _soundId = ref.read(selectedPianoProvider);
    _range = data.keyboardRange;
    _keyboardVisible = data.keyboardVisible;
    _readingAid = data.readingAid;
    _invertedKit = data.invertedKit;
    // Resolved lazily on first build: the phone/tablet default needs the
    // inherited layout context, unavailable in initState.
    _notationTheme = ref.read(playerPreferencesProvider).notationTheme;
  }

  @override
  void dispose() {
    // Leaving this surface takes the coached controls off screen, so the guided
    // sequence ends here rather than pointing at nothing (it stays replayable
    // from help) — unless it already advanced to the transport rewind step,
    // whose control lives on the player screen this close reveals. Deferred: a
    // provider must not be written while the tree is being finalized.
    final coaching = _coaching;
    scheduleMicrotask(coaching.setupSurfaceClosed);
    super.dispose();
  }

  void _apply() {
    final notifier = ref.read(playerProvider.notifier);
    final current = ref.read(playerProvider);
    if (_hands != current.selectedHands) notifier.setSelectedHands(_hands);
    if (_speed != current.speed) notifier.setSpeed(_speed);
    if (_metronome != current.metronomeEnabled) notifier.toggleMetronome();
    // Compared against the stored *preference*, so applying without touching the
    // control re-selects nothing. Comparing against the connected device instead
    // meant "auto" always looked like a change, re-opening the MIDI port on every
    // Apply — which is what eventually took the USB audio down with it.
    if (_port != ref.read(playerPreferencesProvider).midiPort) {
      notifier.selectMidiPort(_port);
    }
    if (_range != current.keyboardRange) notifier.setKeyboardRange(_range);
    if (_keyboardVisible != current.keyboardVisible) {
      notifier.setKeyboardVisible(_keyboardVisible);
    }
    if (_readingAid != current.readingAid) notifier.setReadingAid(_readingAid);
    if (_invertedKit != current.invertedKit) {
      notifier.setInvertedKit(enabled: _invertedKit);
    }
    if (_soundId != ref.read(selectedPianoProvider)) {
      ref.read(selectedPianoProvider.notifier).select(_soundId);
    }
    final scoreSizeDraft = _scoreSizeDraft;
    if (scoreSizeDraft != null &&
        scoreSizeDraft != ref.read(playerPreferencesProvider).scoreSize) {
      ref.read(playerPreferencesProvider.notifier).setScoreSize(scoreSizeDraft);
    }
    if (_notationTheme != ref.read(playerPreferencesProvider).notationTheme) {
      ref
          .read(playerPreferencesProvider.notifier)
          .setNotationTheme(_notationTheme);
    }
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
    final phone = context.isPhoneLayout;
    final maxHeight = MediaQuery.of(context).size.height * 0.92;

    final facts = _facts(l10n, data, entry);
    final sound = _soundSection(l10n);
    // For percussion the selector reads hands / feet, offered despite the
    // single staff whenever the score has both — the split a drummer actually
    // practises (change: add-drum-kit-view).
    final hands =
        (data.isPercussion ? data.hasHandsAndFeet : data.hasMultipleStaves)
        ? _handsSection(l10n, percussion: data.isPercussion)
        : null;
    final metronome = _metronomeTile(l10n);
    final tempo = _tempoTile(l10n, data);
    final midi = _midiSection(l10n, data);
    // Where the app's audio goes (change: add-audio-output-routing). Unlike the
    // drafted settings around it, this one applies immediately: the point of
    // picking an output is hearing the change.
    final soundOutput = SoundOutputSection(title: _sectionTitle);
    // The range apparatus does not apply to a drum kit (an unordered set of
    // pieces, not an interval): the chooser is not offered and the stored mode
    // stays untouched for the next keyboard score. The inverted-kit layout
    // takes its place; the reading aid rides the Wait-Mode gate, absent for
    // percussion until add-drum-scoring.
    final Widget? keyboardSize = data.isPercussion
        ? null
        : _keyboardSizeSection(l10n);
    final Widget? invertedKit = data.isPercussion
        ? _invertedKitSection(l10n)
        : null;
    final Widget? readingAid = data.isPercussion
        ? null
        : _readingAidSection(l10n);
    _scoreSizeDraft ??= resolveScoreSize(
      ref.read(playerPreferencesProvider).scoreSize,
      isPhone: phone,
    );
    final scoreSize = _scoreSizeSection(l10n);
    final scoreTheme = _scoreThemeSection(l10n);
    // The keyboard toggle only applies to the Portée: Synthesia needs the
    // keyboard for its cascade, and the engraved Partition never shows it
    // (the freed height keeps the current + next lines on screen).
    final keyboardVisibility = data.mode == RenderMode.staff
        ? _keyboardVisibilityTile(l10n)
        : null;

    Widget scrollCol(List<Widget> children) => SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: children,
      ),
    );

    Widget col(List<Widget?> children) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [for (final c in children) ?c],
    );

    // Phone-landscape is short and now carries many controls, so the whole body
    // scrolls as one unit (the Play button stays pinned below). The everyday
    // controls come first in two compact columns; the sound picker — rarely
    // changed — sits last, full width (its dropdown menu follows the field width,
    // so long font names + the "Add…" item need the room). Larger screens keep a
    // single column with the sound last for the same reason.
    final Widget body = phone
        ? SingleChildScrollView(
            child: col([
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: col([
                      ?hands,
                      metronome,
                      ?keyboardSize,
                      ?invertedKit,
                      scoreSize,
                      scoreTheme,
                    ]),
                  ),
                  const SizedBox(width: 24),
                  Expanded(child: col([midi, tempo, ?keyboardVisibility])),
                ],
              ),
              if (readingAid != null) ...[
                const SizedBox(height: 8),
                readingAid,
              ],
              const SizedBox(height: 8),
              sound,
              soundOutput,
            ]),
          )
        : scrollCol([
            facts,
            ?hands,
            metronome,
            tempo,
            ?readingAid,
            ?keyboardSize,
            ?invertedKit,
            scoreSize,
            scoreTheme,
            ?keyboardVisibility,
            midi,
            sound,
            soundOutput,
          ]);

    // Inline leaderboard: the trophy swaps the body to the board (no stacked
    // modal). Only an accepted catalog score has a board (`catalogId`), so a
    // toggled-on board falls back to the setup if that ever changes.
    final showingBoard = _showBoard && entry?.catalogId != null;
    final Widget shownBody = showingBoard
        ? LeaderboardView(scoreId: entry!.catalogId!, title: '')
        : body;

    final header = _header(l10n, title, composer, entry, phone);
    final button = Padding(
      padding: EdgeInsets.only(top: phone ? 8 : 12),
      child: FilledButton(
        key: const Key('pre-play-primary'),
        onPressed: _apply,
        child: Text(widget.inGame ? l10n.prePlayApply : l10n.prePlayStart),
      ),
    );

    // Phone: full-screen so all controls fit without scrolling. Larger screens:
    // a centered, content-sized dialog.
    final Widget dialog = phone
        ? Dialog.fullscreen(
            backgroundColor: CymbraColors.surfaceContainerLow,
            child: SafeArea(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(20, 10, 20, 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    header,
                    const SizedBox(height: 4),
                    if (!showingBoard) ...[facts, const SizedBox(height: 4)],
                    Expanded(child: shownBody),
                    button,
                  ],
                ),
              ),
            ),
          )
        : Dialog(
            backgroundColor: CymbraColors.surfaceContainerLow,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(16),
            ),
            child: ConstrainedBox(
              constraints: BoxConstraints(maxWidth: 460, maxHeight: maxHeight),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
                child: Column(
                  // When the board is shown the modal fills a STABLE height
                  // (max), so switching modes — or the loading→data swap —
                  // never resizes the dialog; the list scrolls within. The
                  // setup view stays content-sized.
                  mainAxisSize: showingBoard
                      ? MainAxisSize.max
                      : MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    header,
                    const SizedBox(height: 4),
                    if (showingBoard)
                      Expanded(child: shownBody)
                    else
                      Flexible(child: shownBody),
                    button,
                  ],
                ),
              ),
            ),
          );
    // When the guided sequence advances to the transport rewind step — whose
    // control lives on the player screen BEHIND this dialog — the setup applies
    // its drafts and closes, revealing the control the spotlight points at.
    return _CoachStepListener(onRewindStep: _apply, child: dialog);
  }

  Widget _header(
    AppLocalizations l10n,
    String title,
    String? composer,
    CatalogEntry? entry,
    bool phone,
  ) => Row(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              maxLines: phone ? 1 : 2,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                color: CymbraColors.onSurface,
                fontSize: phone ? 16 : 20,
                fontWeight: FontWeight.w700,
              ),
            ),
            if (composer != null && composer.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 2),
                child: Text(
                  composer,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: phone ? 12 : 14,
                  ),
                ),
              ),
            if (entry?.level != null)
              Padding(
                padding: EdgeInsets.only(top: phone ? 4 : 8),
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: DifficultyBadge(level: entry!.level, l10n: l10n),
                ),
              ),
          ],
        ),
      ),
      // Toggle this piece's leaderboard INLINE (change: add-play-leaderboards) —
      // the body swaps between the setup controls and the board, no stacked modal.
      // Only an accepted catalog score has a shared board (`catalogId`). The icon
      // flips to a back arrow while the board is shown, to return to the settings.
      if (entry?.catalogId != null)
        IconButton(
          icon: Icon(
            _showBoard ? Icons.arrow_back : Icons.emoji_events,
            color: CymbraColors.onSurfaceVariant,
          ),
          tooltip: _showBoard
              ? MaterialLocalizations.of(context).backButtonTooltip
              : l10n.leaderboardTitle,
          visualDensity: VisualDensity.compact,
          onPressed: () => setState(() => _showBoard = !_showBoard),
        ),
      IconButton(
        icon: const Icon(Icons.close, color: CymbraColors.onSurfaceVariant),
        tooltip: l10n.cancel,
        visualDensity: VisualDensity.compact,
        onPressed: () => Navigator.of(context).pop(),
      ),
    ],
  );

  /// Key / time-signature / marked-tempo chips — each labelled (so a novice knows
  /// what it is) and shown only when known.
  Widget _facts(AppLocalizations l10n, PlayerData data, CatalogEntry? entry) {
    // Note naming: letters (C, D, E…) in English, solfège (Do, Ré, Mi…) in the
    // Latin-language locales.
    final lang = Localizations.localeOf(context).languageCode;
    final key = keyName(
      data.keyFifths,
      solfege: lang != 'en',
      frenchRe: lang == 'fr',
    );
    final chips = <Widget>[
      // A key signature names pitches; a drum part has none (change:
      // add-drum-kit-view) — the chip would read "Key · C" on every groove.
      if (!data.isPercussion) _chip(Icons.piano, '${l10n.prePlayKey} · $key'),
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

  Widget _handsSection(AppLocalizations l10n, {bool percussion = false}) {
    const hands = Hand.values;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _sectionTitle(l10n.prePlayHands),
        // A 3-way toggle (Left / Right / Both) — compact, one row instead of
        // three radio rows. Registered as a coach target so the guided sequence
        // can point at the real control.
        CoachTarget(
          anchor: CoachAnchor.hands,
          child: LayoutBuilder(
            builder: (context, constraints) {
              // Three equal segments filling the width (account for borders).
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
                children: [
                  for (final h in hands)
                    Text(handLabel(l10n, h, percussion: percussion)),
                ],
              );
            },
          ),
        ),
      ],
    );
  }

  /// Instrument sound picker (combobox), correlated to the score's instrument
  /// (piano for now). A draft — applied on Validate like the other settings.
  Widget _soundSection(AppLocalizations l10n) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _sectionTitle(l10n.settingsCategoryPiano),
      CoachTarget(
        anchor: CoachAnchor.pianoSound,
        child: SoundSelectorField(
          value: _soundId,
          onChanged: (id) => setState(() => _soundId = id),
        ),
      ),
    ],
  );

  /// The inverted-kit layout (change: add-drum-kit-view): reverses the lane
  /// order and the pad strip together. Labelled by the KIT's setup, never the
  /// player — many left-handed drummers play a standard kit.
  Widget _invertedKitSection(AppLocalizations l10n) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _sectionTitle(l10n.invertedKitTitle),
      SwitchListTile(
        key: const Key('inverted-kit-switch'),
        contentPadding: const EdgeInsets.symmetric(horizontal: 4),
        title: Text(
          l10n.invertedKitLabel,
          style: const TextStyle(color: CymbraColors.onSurface, fontSize: 14),
        ),
        subtitle: Text(
          l10n.invertedKitHint,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 12,
          ),
        ),
        value: _invertedKit,
        activeThumbColor: CymbraColors.tertiary,
        onChanged: (v) => setState(() => _invertedKit = v),
      ),
    ],
  );

  String _rangeLabel(AppLocalizations l10n, KeyboardRangeMode m) =>
      m == KeyboardRangeMode.auto
      ? l10n.keyboardAutoFit
      : l10n.keyboardKeys(m.label);

  /// On-screen keyboard size (a dropdown, like the MIDI device).
  Widget _keyboardSizeSection(AppLocalizations l10n) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _sectionTitle(l10n.settingsCategoryKeyboardSize),
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: BoxDecoration(
          color: CymbraColors.surfaceContainerHigh,
          borderRadius: BorderRadius.circular(10),
        ),
        child: DropdownButtonHideUnderline(
          child: DropdownButton<KeyboardRangeMode>(
            isExpanded: true,
            value: _range,
            dropdownColor: CymbraColors.surfaceContainerHigh,
            iconEnabledColor: CymbraColors.onSurfaceVariant,
            style: const TextStyle(color: CymbraColors.onSurface),
            items: [
              for (final m in KeyboardRangeMode.values)
                DropdownMenuItem<KeyboardRangeMode>(
                  value: m,
                  child: Text(_rangeLabel(l10n, m)),
                ),
            ],
            onChanged: (m) => setState(() => _range = m ?? _range),
          ),
        ),
      ),
    ],
  );

  String _readingAidLabel(AppLocalizations l10n, NoteReadingAid a) =>
      switch (a) {
        NoteReadingAid.off => l10n.readingAidOff,
        NoteReadingAid.name => l10n.readingAidName,
        NoteReadingAid.nameAndRhythm => l10n.readingAidNameAndRhythm,
      };

  /// Beginner reading aid: how much the player is told about the note the score
  /// is waiting for. A 3-way toggle like the hand chooser, with a line saying
  /// *when* it shows up so the choice is not a mystery.
  Widget _readingAidSection(AppLocalizations l10n) {
    const levels = NoteReadingAid.values;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _sectionTitle(l10n.readingAidTitle),
        LayoutBuilder(
          builder: (context, constraints) {
            final segment = (constraints.maxWidth - 4) / levels.length;
            return ToggleButtons(
              key: const Key('reading-aid-toggle'),
              isSelected: [for (final a in levels) a == _readingAid],
              onPressed: (i) => setState(() => _readingAid = levels[i]),
              borderRadius: BorderRadius.circular(10),
              borderColor: CymbraColors.surfaceContainerHighest,
              selectedBorderColor: CymbraColors.tertiary,
              color: CymbraColors.onSurfaceVariant,
              selectedColor: CymbraColors.onSurface,
              fillColor: CymbraColors.tertiary.withValues(alpha: 0.22),
              // Fixed segments: these labels are longer than the hand
              // chooser's, so each is pinned to its third and scaled down
              // rather than pushing the row past the modal's width.
              constraints: BoxConstraints(
                minHeight: 44,
                minWidth: segment,
                maxWidth: segment,
              ),
              children: [
                for (final a in levels)
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 6),
                    child: FittedBox(
                      fit: BoxFit.scaleDown,
                      child: Text(
                        _readingAidLabel(l10n, a),
                        textAlign: TextAlign.center,
                      ),
                    ),
                  ),
              ],
            );
          },
        ),
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Text(
            l10n.readingAidSubtitle,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 12,
            ),
          ),
        ),
      ],
    );
  }

  /// Notation size (S/M/L), applied to both the Partition and staff views.
  Widget _scoreSizeSection(AppLocalizations l10n) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _sectionTitle(l10n.settingsCategoryScoreSize),
      SegmentedButton<ScoreSize>(
        segments: [
          for (final (size, label) in [
            (ScoreSize.small, l10n.scoreSizeSmall),
            (ScoreSize.medium, l10n.scoreSizeMedium),
            (ScoreSize.large, l10n.scoreSizeLarge),
          ])
            ButtonSegment(
              value: size,
              // Scale down rather than wrap: "Moyenne" broke onto two lines
              // in the narrow phone column.
              label: FittedBox(
                fit: BoxFit.scaleDown,
                child: Text(label, maxLines: 1),
              ),
            ),
        ],
        selected: {?_scoreSizeDraft},
        onSelectionChanged: (s) => setState(() => _scoreSizeDraft = s.first),
      ),
    ],
  );

  /// Notation theme (dark surface / paper), applied to both notation views.
  Widget _scoreThemeSection(AppLocalizations l10n) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _sectionTitle(l10n.settingsCategoryScoreTheme),
      SegmentedButton<NotationTheme>(
        segments: [
          for (final (theme, label) in [
            (NotationTheme.dark, l10n.scoreThemeDark),
            (NotationTheme.paper, l10n.scoreThemePaper),
          ])
            ButtonSegment(
              value: theme,
              label: FittedBox(
                fit: BoxFit.scaleDown,
                child: Text(label, maxLines: 1),
              ),
            ),
        ],
        selected: {_notationTheme},
        onSelectionChanged: (s) => setState(() => _notationTheme = s.first),
      ),
    ],
  );

  /// On-screen keyboard visibility (a switch, like the metronome).
  Widget _keyboardVisibilityTile(AppLocalizations l10n) => SwitchListTile(
    contentPadding: EdgeInsets.zero,
    title: Text(
      l10n.settingsCategoryKeyboardVisibility,
      style: const TextStyle(color: CymbraColors.onSurface),
    ),
    subtitle: Text(
      _keyboardVisible ? l10n.keyboardShown : l10n.keyboardHidden,
      style: const TextStyle(color: CymbraColors.onSurfaceVariant),
    ),
    value: _keyboardVisible,
    onChanged: (v) => setState(() => _keyboardVisible = v),
  );

  /// Metronome on/off — placed before the tempo control.
  Widget _metronomeTile(AppLocalizations l10n) => SwitchListTile(
    contentPadding: EdgeInsets.zero,
    title: Text(
      l10n.metronome,
      style: const TextStyle(color: CymbraColors.onSurface),
    ),
    value: _metronome,
    onChanged: (v) => setState(() => _metronome = v),
  );

  /// Playback-speed slider (full width) with the resulting tempo (BPM) and the
  /// ×-multiplier, so the value stays understandable. The fixed-width value box
  /// wraps rather than overflowing in the narrow two-column phone layout.
  Widget _tempoTile(AppLocalizations l10n, PlayerData data) {
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
            const SizedBox(width: 8),
            SizedBox(
              width: 72,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Text(
                    '$effectiveBpm BPM',
                    style: const TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 13,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  Text(
                    '${_speed.toStringAsFixed(2)}×',
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 11,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _midiSection(AppLocalizations l10n, PlayerData data) {
    // Selected value must match a dropdown item; fall back to Auto (null) if the
    // current device isn't in the enumerated ports.
    final value = data.midiPorts.contains(_port) ? _port : null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _sectionTitle(l10n.settingsCategoryMidiDevice),
        CoachTarget(
          anchor: CoachAnchor.midiDevice,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(
              color: CymbraColors.surfaceContainerHigh,
              borderRadius: BorderRadius.circular(10),
            ),
            child: DropdownButtonHideUnderline(
              // Keyed: the sound-output section below carries a `String?`
              // dropdown too, so the type alone no longer identifies this one.
              child: DropdownButton<String?>(
                key: const Key('midi-device'),
                isExpanded: true,
                value: value,
                dropdownColor: CymbraColors.surfaceContainerHigh,
                iconEnabledColor: CymbraColors.onSurfaceVariant,
                style: const TextStyle(color: CymbraColors.onSurface),
                items: [
                  DropdownMenuItem<String?>(
                    value: null,
                    child: Text(l10n.midiAutoFirstDevice),
                  ),
                  for (final port in data.midiPorts)
                    DropdownMenuItem<String?>(value: port, child: Text(port)),
                ],
                onChanged: (v) => setState(() => _port = v),
              ),
            ),
          ),
        ),
        if (data.midiPorts.isEmpty)
          // Android: USB-OTG / charge-only-cable advice; other platforms: a
          // plain "no device" hint.
          if (ref.watch(isAndroidProvider))
            const OtgGuidance()
          else
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                l10n.midiNoDeviceDetected,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            ),
      ],
    );
  }
}
