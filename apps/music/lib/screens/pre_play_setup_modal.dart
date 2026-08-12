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
import '../state/coaching_notifier.dart';
import '../state/notation_notifier.dart';
import '../state/note_label.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../state/player_preferences.dart';
import '../state/practice_settings_store.dart';
import '../state/score_catalog.dart';
import '../state/selected_piano.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/coach_mark.dart';
import '../widgets/difficulty_badge.dart';
import '../widgets/leaderboard_view.dart';
import '../widgets/otg_guidance.dart';
import '../widgets/practice_range_controls.dart';
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

  /// Draft practice settings (change: add-measure-range-practice). [_selective]
  /// is the full-run vs section-run choice; the measure bounds are 0-based
  /// indices into `measureStartMs` (displayed 1-based).
  late bool _selective;
  late int _fromMeasure;
  late int _toMeasure;

  /// Whether the modal is on its **second step** — the passage settings
  /// (change: add-measure-range-practice). Choosing "Section" turns the primary
  /// action into "Next"; that step then gets the whole modal, so the engraved
  /// score is big enough to actually pick bars on, instead of being squeezed in
  /// under the general settings.
  bool _practiceStep = false;

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
    final data = ref.read(playerProvider);
    _hands = data.selectedHands;
    _speed = data.speed;
    _metronome = data.metronomeEnabled;
    _port = data.connectedDevice;
    _soundId = ref.read(selectedPianoProvider);
    _range = data.keyboardRange;
    _keyboardVisible = data.keyboardVisible;
    _readingAid = data.readingAid;
    // Resolved lazily on first build: the phone/tablet default needs the
    // inherited layout context, unavailable in initState.
    _notationTheme = ref.read(playerPreferencesProvider).notationTheme;
    // Practice range: pre-fill from the run's current range (which the per-score
    // saved settings have already seeded), else the whole piece.
    final last = data.lastMeasureIndex ?? 0;
    _selective = data.isSelectiveRun;
    _fromMeasure = data.practiceStartMeasure ?? 0;
    _toMeasure = data.practiceEndMeasure ?? last;
    // Per-score saved settings pre-fill the picker (change: add-measure-range-
    // practice, D7) — the RANGE only, never the run type: a reopened score still
    // defaults to a full run, as the setup spec requires.
    if (!_selective) unawaited(_prefillFromSaved());
  }

  @override
  void dispose() {
    // Leaving this surface takes the coached controls off screen, so the guided
    // sequence ends here rather than pointing at nothing (it stays replayable
    // from help). Deferred: a provider must not be written while the tree is
    // being finalized.
    final coaching = _coaching;
    scheduleMicrotask(coaching.skipTour);
    super.dispose();
  }

  /// Loads this score's saved practice settings and pre-fills the (draft)
  /// picker with them, clamped to the piece's current measure count so a
  /// re-imported score can't leave the steppers pointing at bars that no longer
  /// exist. A no-op when nothing was saved or the modal is already gone.
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
      _fromMeasure = applied.startMeasure;
      _toMeasure = applied.endMeasure;
    });
  }

  void _apply() {
    final notifier = ref.read(playerProvider.notifier);
    final current = ref.read(playerProvider);
    // Practice range first: it moves the playhead, so a hand change (which also
    // reseeds it) must not be undone by it.
    if (_selective) {
      notifier.setPracticeRange(_fromMeasure, _toMeasure);
    } else if (current.hasPracticeRange) {
      notifier.clearPracticeRange();
    }
    if (_hands != current.selectedHands) notifier.setSelectedHands(_hands);
    if (_speed != current.speed) notifier.setSpeed(_speed);
    if (_metronome != current.metronomeEnabled) notifier.toggleMetronome();
    if (_port != current.connectedDevice) notifier.selectMidiPort(_port);
    if (_range != current.keyboardRange) notifier.setKeyboardRange(_range);
    if (_keyboardVisible != current.keyboardVisible) {
      notifier.setKeyboardVisible(_keyboardVisible);
    }
    if (_readingAid != current.readingAid) notifier.setReadingAid(_readingAid);
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
    final hands = data.hasMultipleStaves ? _handsSection(l10n) : null;
    final metronome = _metronomeTile(l10n);
    final tempo = _tempoTile(l10n, data);
    final midi = _midiSection(l10n, data);
    // Where the app's audio goes (change: add-audio-output-routing). Unlike the
    // drafted settings around it, this one applies immediately: the point of
    // picking an output is hearing the change.
    final soundOutput = SoundOutputSection(title: _sectionTitle);
    final keyboardSize = _keyboardSizeSection(l10n);
    final readingAid = _readingAidSection(l10n);
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
    // Full-run vs section (practice) + the measure-range picker.
    final practice = _practiceSection(l10n, data);

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
                      keyboardSize,
                      scoreSize,
                      scoreTheme,
                    ]),
                  ),
                  const SizedBox(width: 24),
                  Expanded(child: col([midi, tempo, ?keyboardVisibility])),
                ],
              ),
              const SizedBox(height: 8),
              readingAid,
              ?practice,
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
            readingAid,
            ?practice,
            keyboardSize,
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
        : (_practiceStep ? _practiceStepBody(l10n, data) : body);

    final header = _header(l10n, title, composer, entry, phone);
    // Choosing "Section" makes the primary action open the passage STEP instead
    // of starting: the bars are picked there, on a full-size score.
    final goesToPracticeStep = _selective && !_practiceStep && !showingBoard;
    final button = Padding(
      padding: EdgeInsets.only(top: phone ? 8 : 12),
      child: FilledButton(
        key: const Key('pre-play-primary'),
        onPressed: goesToPracticeStep
            ? () => setState(() => _practiceStep = true)
            : _apply,
        child: Text(
          goesToPracticeStep
              ? l10n.prePlayNext
              : (widget.inGame ? l10n.prePlayApply : l10n.prePlayStart),
        ),
      ),
    );

    // Phone: full-screen so all controls fit without scrolling. Larger screens:
    // a centered, content-sized dialog.
    if (phone) {
      return Dialog.fullscreen(
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
      );
    }
    return Dialog(
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 460, maxHeight: maxHeight),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
          child: Column(
            // When the board is shown the modal fills a STABLE height (max), so
            // switching modes — or the loading→data swap — never resizes the
            // dialog; the list scrolls within. The setup view stays content-sized.
            mainAxisSize: showingBoard ? MainAxisSize.max : MainAxisSize.min,
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
      // On the passage step, a way back to the general settings — the step is a
      // detour, not a dead end.
      if (_practiceStep)
        IconButton(
          key: const Key('practice-step-back'),
          icon: const Icon(
            Icons.arrow_back,
            color: CymbraColors.onSurfaceVariant,
          ),
          tooltip: MaterialLocalizations.of(context).backButtonTooltip,
          visualDensity: VisualDensity.compact,
          onPressed: () => setState(() => _practiceStep = false),
        ),
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

  /// Full-run vs section (practice) choice + the measure-range steppers
  /// (change: add-measure-range-practice, D5/D6). Available in **every** render
  /// mode — the tap-on-score picker is Partition-only, so these steppers are the
  /// universal surface. Omitted when the piece has no measure table (the demo
  /// score) or a single measure — there is no narrower range to pick.
  Widget? _practiceSection(AppLocalizations l10n, PlayerData data) {
    final last = data.lastMeasureIndex;
    if (last == null || last < 1) return null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _sectionTitle(l10n.practiceSectionTitle),
        LayoutBuilder(
          builder: (context, constraints) {
            final segment = (constraints.maxWidth - 4) / 2;
            return ToggleButtons(
              key: const Key('practice-run-type'),
              isSelected: [!_selective, _selective],
              onPressed: (i) => setState(() {
                _selective = i == 1;
                // Back to a full run: the passage step no longer applies.
                if (!_selective) _practiceStep = false;
                // A "passage" spanning the whole piece is not a passage: it
                // would start a SCORED run while the modal says practice is not
                // scored. Seed a real, narrow one (the first two bars) so the
                // choice is honest the moment it is made — the score picker on
                // the next step is how the player then places it.
                if (_selective && _fromMeasure == 0 && _toMeasure == last) {
                  _toMeasure = 1;
                }
              }),
              borderRadius: BorderRadius.circular(10),
              borderColor: CymbraColors.surfaceContainerHighest,
              selectedBorderColor: CymbraColors.tertiary,
              color: CymbraColors.onSurfaceVariant,
              selectedColor: CymbraColors.onSurface,
              fillColor: CymbraColors.tertiary.withValues(alpha: 0.22),
              constraints: BoxConstraints(minHeight: 44, minWidth: segment),
              children: [
                Text(l10n.practiceRunFull),
                Text(l10n.practiceRunSelective),
              ],
            );
          },
        ),
      ],
    );
  }

  /// The **second step**: everything about the passage — the engraved score to
  /// pick the bars on, the steppers, and the loop settings. It owns the whole
  /// modal body, so the score gets real room (the reason this is a step rather
  /// than one more section stacked under the general settings).
  Widget _practiceStepBody(AppLocalizations l10n, PlayerData data) {
    final last = data.lastMeasureIndex ?? 0;
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          PracticeRangeControls(
            lastMeasure: last,
            fromMeasure: _fromMeasure,
            toMeasure: _toMeasure,
            // The step has the height to spare, so the score is worth showing
            // large — this is where the bars are actually chosen.
            scoreHeight: 320,
            // Keep the range ordered while editing: moving one bound past the
            // other pushes it along (the notifier also normalizes, defensively).
            onFromChanged: (v) => setState(() {
              _fromMeasure = v;
              if (_toMeasure < v) _toMeasure = v;
            }),
            onToChanged: (v) => setState(() {
              _toMeasure = v;
              if (_fromMeasure > v) _fromMeasure = v;
            }),
          ),
        ],
      ),
    );
  }

  Widget _handsSection(AppLocalizations l10n) {
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
                children: [for (final h in hands) Text(handLabel(l10n, h))],
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
