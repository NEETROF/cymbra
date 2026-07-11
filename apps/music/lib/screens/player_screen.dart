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

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../layout/device_class.dart';
import '../painters/partition_painter.dart';
import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../painters/staff_painter.dart';
import '../painters/synthesia_painter.dart';
import '../services/platform_info.dart';
import '../src/rust/api/musicxml.dart' show System;
import '../state/app_language.dart';
import '../state/app_locale.dart';
import '../state/notation_data.dart';
import '../state/notation_notifier.dart';
import '../state/performance_scoring.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../state/session_summary.dart';
import '../state/session_summary_store.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/language_selector.dart';
import '../widgets/scoring_overlay.dart';
import '../widgets/session_summary_modal.dart';

/// Main screen of the Cymbra player: top bar, rendering area
/// (Synthesia or Staff), keyboard, and transport bar.
class PlayerScreen extends ConsumerStatefulWidget {
  const PlayerScreen({super.key});

  @override
  ConsumerState<PlayerScreen> createState() => _PlayerScreenState();
}

class _PlayerScreenState extends ConsumerState<PlayerScreen>
    with SingleTickerProviderStateMixin {
  final FocusNode _focusNode = FocusNode();
  late final Ticker _ticker;
  Duration _lastTick = Duration.zero;

  /// Active on-screen-keyboard pointers → the pitch each is holding, so a finger
  /// release note-offs only its own pitch (independent multi-touch). Same-pitch
  /// from multiple sources is last-release-wins: releasing one source clears the
  /// shared [PlayerData.activeNotes] entry even if another still holds it — an
  /// accepted v1 simplification (chords use distinct pitches).
  final Map<int, int> _keyboardPointers = {};

  /// Keyboard height is derived from the available render height (not fixed) so
  /// it shrinks on short phone-landscape viewports and stays proportionate on
  /// larger screens. Tablet/desktop keep the prior band ([_maxKeyboardHeight]
  /// == the old fixed 150 px, so those layouts don't regress). Phones use a
  /// shorter band so the thin 88-key keyboard doesn't dominate the short
  /// landscape viewport and more height goes to the render area (waterfall /
  /// notation). The floor keeps the keys tappable.
  static const double _minKeyboardHeight = 96;
  static const double _maxKeyboardHeight = 150;
  static const double _keyboardHeightFraction = 0.34;
  static const double _minKeyboardHeightPhone = 78;
  static const double _maxKeyboardHeightPhone = 108;
  static const double _keyboardHeightFractionPhone = 0.28;

  /// Keyboard height for a render column of [availableHeight] pixels (the
  /// [LayoutBuilder] constraints below the top bar). The render area above the
  /// keyboard takes the remainder via [Expanded], staying > 0 as long as the
  /// viewport exceeds the (phone or default) floor.
  double _keyboardHeightFor(double availableHeight, {required bool isPhone}) {
    final fraction = isPhone
        ? _keyboardHeightFractionPhone
        : _keyboardHeightFraction;
    final min = isPhone ? _minKeyboardHeightPhone : _minKeyboardHeight;
    final max = isPhone ? _maxKeyboardHeightPhone : _maxKeyboardHeight;
    return (availableHeight * fraction).clamp(min, max);
  }

  /// Random source for the near-miss assist keys (q/s).
  final math.Random _rng = math.Random();

  /// Assist keys → the pitches each fired on key-down, so key-up note-offs
  /// exactly those (a near-miss picks a fresh random pitch each press).
  final Map<LogicalKeyboardKey, Set<int>> _assistPressed = {};

  /// Whether playback was running when the settings drawer opened, so closing it
  /// restores that state (the drawer pauses the session while open).
  bool _wasPlayingBeforeDrawer = false;

  /// Lets [_onEndDrawerChanged] reset the drawer to its category list each time
  /// it opens (its navigation state otherwise persists across open/close).
  final GlobalKey<_SettingsDrawerState> _settingsDrawerKey = GlobalKey();

  /// Pause the session while the settings drawer is open; restore the prior
  /// play/pause state when it closes. Also resets the drawer to its root.
  void _onEndDrawerChanged(bool isOpen) {
    final notifier = ref.read(playerProvider.notifier);
    if (isOpen) {
      _settingsDrawerKey.currentState?.resetToRoot();
      _wasPlayingBeforeDrawer = ref.read(playerProvider).isPlaying;
      if (_wasPlayingBeforeDrawer) notifier.setPlaying(false);
    } else {
      if (_wasPlayingBeforeDrawer) notifier.setPlaying(true);
      _wasPlayingBeforeDrawer = false;
    }
  }

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick)..start();
  }

  void _onTick(Duration elapsed) {
    final dt = (elapsed - _lastTick).inMicroseconds / 1000.0; // ms
    _lastTick = elapsed;
    if (dt > 0 && dt < 100) {
      final speed = ref.read(playerProvider).speed;
      ref.read(playerProvider.notifier).advance(dt * speed);
    }
  }

  /// Desktop keyboard = four practice-assist keys (AZERTY 2×2 cluster):
  ///   a = left-hand correct,  z = right-hand correct,
  ///   q = left-hand near-miss, s = right-hand near-miss.
  /// The correct keys play all notes expected for that hand at the playhead
  /// (satisfying Wait Mode); the near-miss keys play a random nearby wrong note.
  /// Exact, arbitrary notes are played with the on-screen keyboard instead.
  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    final key = event.logicalKey;
    final bool rightHand;
    final bool nearMiss;
    if (key == LogicalKeyboardKey.keyA) {
      rightHand = false;
      nearMiss = false;
    } else if (key == LogicalKeyboardKey.keyZ) {
      rightHand = true;
      nearMiss = false;
    } else if (key == LogicalKeyboardKey.keyQ) {
      rightHand = false;
      nearMiss = true;
    } else if (key == LogicalKeyboardKey.keyS) {
      rightHand = true;
      nearMiss = true;
    } else {
      return KeyEventResult.ignored;
    }

    final notifier = ref.read(playerProvider.notifier);
    if (event is KeyDownEvent) {
      if (_assistPressed.containsKey(key)) {
        return KeyEventResult.handled; // already held; ignore stray repeat
      }
      final pitches = _assistPitches(rightHand: rightHand, nearMiss: nearMiss);
      if (pitches.isEmpty) return KeyEventResult.handled;
      _assistPressed[key] = pitches;
      for (final p in pitches) {
        notifier.noteOn(p);
      }
      return KeyEventResult.handled;
    } else if (event is KeyUpEvent) {
      final pitches = _assistPressed.remove(key);
      if (pitches != null) {
        for (final p in pitches) {
          notifier.noteOff(p);
        }
      }
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored; // ignore repeats
  }

  /// Pitches an assist key should sound now: the expected notes for [rightHand],
  /// or — for a [nearMiss] — a single random pitch near one of them that never
  /// equals an expected note. Empty when nothing is expected for that hand.
  Set<int> _assistPitches({required bool rightHand, required bool nearMiss}) {
    final data = ref.read(playerProvider);
    final expected = data.expectedNotesForHand(
      data.elapsedMs,
      rightHand: rightHand,
    );
    if (expected.isEmpty) return const {};
    if (!nearMiss) return expected;
    final bounds = data.keyboardBounds;
    return {
      nearMissPitch(
        expected.first,
        lowBound: bounds.low,
        highBound: bounds.high,
        avoid: expected,
        nextRandom: _rng.nextInt,
      ),
    };
  }

  // --- On-screen keyboard (mouse / touch) -------------------------------
  // Routes pointer presses through the same note-on/off path as MIDI and the
  // computer keyboard, so on-screen play drives feedback and the Wait Mode gate
  // identically. Works during playback and when stopped.

  void _onKeyboardPointerDown(
    PointerDownEvent event,
    PianoLayout layout,
    double keyboardHeight,
  ) {
    final pitch = layout.pitchAt(event.localPosition, keyboardHeight);
    if (pitch == null) return;
    _keyboardPointers[event.pointer] = pitch;
    ref.read(playerProvider.notifier).noteOn(pitch);
  }

  void _onKeyboardPointerUp(PointerEvent event) {
    final pitch = _keyboardPointers.remove(event.pointer);
    if (pitch == null) return;
    ref.read(playerProvider.notifier).noteOff(pitch);
  }

  @override
  void dispose() {
    _ticker.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // A scored run just finished: persist the summary and show the modal.
    ref.listen(performanceScorerProvider.select((s) => s.lastResult), (
      prev,
      next,
    ) {
      if (next != null && !identical(next, prev)) {
        _onScoredRunFinished(next);
      }
    });
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _onKey,
      child: Scaffold(
        // Settings live in an end drawer (slides in from the right). Opening it
        // pauses the session; closing restores the prior play/pause state.
        endDrawer: _SettingsDrawer(key: _settingsDrawerKey),
        onEndDrawerChanged: _onEndDrawerChanged,
        // On a phone the bottom safe-area inset (home-indicator zone) wastes
        // scarce landscape height below the transport bar, so we let the bar
        // extend into it (its own small margin keeps a hair of clearance, and
        // the centred controls sit clear of the thin indicator). Tablet/desktop
        // keep the full safe area.
        body: SafeArea(
          bottom: !context.isPhoneLayout,
          child: Column(
            children: [
              const _TopBar(),
              Expanded(
                child: Consumer(
                  builder: (context, ref, child) {
                    final data = ref.watch(playerProvider);
                    return LayoutBuilder(
                      builder: (context, constraints) {
                        final bounds = data.keyboardBounds;
                        final layout = PianoLayout(
                          width: constraints.maxWidth,
                          lowPitch: bounds.low,
                          highPitch: bounds.high,
                        );
                        final keyboardHeight = _keyboardHeightFor(
                          constraints.maxHeight,
                          isPhone: context.isPhoneLayout,
                        );
                        // Synthesia always shows the keyboard (its cascade aligns
                        // to the keys); the notation modes honour the user's
                        // hide-keyboard setting, handing the freed height to the
                        // score.
                        final showKeyboard =
                            data.mode == RenderMode.synthesia ||
                            data.keyboardVisible;
                        return Column(
                          children: [
                            // Clip the render area so a painter (e.g. high notes /
                            // beams in Staff mode) never draws over the top bar or
                            // the keyboard below.
                            Expanded(
                              child: ClipRect(
                                child: _buildRenderArea(
                                  layout,
                                  data,
                                  isPhone: context.isPhoneLayout,
                                ),
                              ),
                            ),
                            if (showKeyboard)
                              SizedBox(
                                height: keyboardHeight,
                                child: Listener(
                                  key: const Key('onscreen-keyboard'),
                                  onPointerDown: (e) => _onKeyboardPointerDown(
                                    e,
                                    layout,
                                    keyboardHeight,
                                  ),
                                  onPointerUp: _onKeyboardPointerUp,
                                  onPointerCancel: _onKeyboardPointerUp,
                                  child: CustomPaint(
                                    size: Size(
                                      constraints.maxWidth,
                                      keyboardHeight,
                                    ),
                                    painter: PianoKeyboardPainter(
                                      layout: layout,
                                      activeNotes: data.activeNotes,
                                      requiredNotes: data.expectedKeys,
                                      leftHandNotes: data.expectedKeysForHand(
                                        rightHand: false,
                                      ),
                                    ),
                                  ),
                                ),
                              ),
                          ],
                        );
                      },
                    );
                  },
                ),
              ),
              _TransportBar(),
            ],
          ),
        ),
      ),
    );
  }

  /// Persists the finished run and presents the summary modal, then clears the
  /// result and applies the player's chosen action (retry restarts playback).
  Future<void> _onScoredRunFinished(SessionResult result) async {
    await ref.read(sessionSummaryStoreProvider).save(result);
    if (!mounted) return;
    final action = await showSessionSummary(context, result);
    if (!mounted) return;
    ref.read(performanceScorerProvider.notifier).clearLastResult();
    if (action == SummaryAction.retry) {
      final player = ref.read(playerProvider.notifier);
      player.restart();
      player.setPlaying(true);
    }
  }

  Widget _buildRenderArea(
    PianoLayout layout,
    PlayerData data, {
    required bool isPhone,
  }) {
    // The engraved two-stave Partition view needs vertical room the short phone
    // landscape viewport doesn't have (it's unreadable there), so it's
    // unavailable on phones and falls back to the staff view. The mode toggle
    // also hides the Partition segment on phones, so this is only reached if the
    // mode was set on a larger screen before switching to a phone layout.
    if (data.mode == RenderMode.partition && !isPhone) {
      return const _PartitionView();
    }
    if (data.mode == RenderMode.synthesia) {
      return Stack(
        children: [
          Positioned.fill(
            child: CustomPaint(
              painter: SynthesiaPainter(
                layout: layout,
                notes: data.visibleNotes,
                elapsedMs: data.elapsedMs,
                activeNotes: data.activeNotes,
              ),
            ),
          ),
          // Gamified sync gauge + hit sparks (shown only during a scored run).
          Positioned.fill(child: ScoringOverlay(layout: layout)),
          if (data.blocked) const _WaitOverlay(),
        ],
      );
    }
    // Standard staff mode (synchronized, horizontal scrolling).
    return Stack(
      children: [
        Positioned.fill(
          child: Container(
            color: CymbraColors.surfaceContainerLow,
            child: CustomPaint(
              painter: StaffPainter(
                notes: data.visibleNotes,
                elapsedMs: data.elapsedMs,
                activeNotes: data.activeNotes,
                bpm: data.bpm,
                songEndMs: data.songEndMs,
                keyFifths: data.keyFifths,
                beats: data.beats,
                beatType: data.beatType,
                measureStartMs: data.measureStartMs,
              ),
              size: Size.infinite,
            ),
          ),
        ),
        Positioned.fill(child: ScoringOverlay(layout: layout)),
      ],
    );
  }
}

/// Top bar: title, indicators and mode toggle.
///
/// A `const` [ConsumerWidget] that watches **only** the title and tempo — not
/// the playhead — so the player's per-frame rebuilds (while playing) do not
/// rebuild the top bar. Each interactive control is its own `const` consumer
/// watching its own slice, which keeps the open settings menu stable on touch
/// devices (an ever-rebuilding [MenuAnchor] flickered on iPad).
class _TopBar extends ConsumerWidget {
  const _TopBar();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Watch only the title here; the tempo/metronome chip watches its own slices
    // so the per-beat pulse doesn't rebuild the whole top bar.
    final title = ref.watch(playerProvider.select((d) => d.title));
    final l10n = AppLocalizations.of(context);
    // On a phone the landscape viewport is short, so the top bar compacts:
    // tighter padding, smaller type, and a narrower lead gap free vertical and
    // horizontal space for the render area. Tablet/desktop keep their sizes.
    // The IconButton/chips keep their own 48 px tap targets regardless.
    final isPhone = context.isPhoneLayout;
    final hPad = isPhone ? 10.0 : 16.0;
    final vPad = isPhone ? 6.0 : 12.0;
    final titleSize = isPhone ? 15.0 : 18.0;
    final subtitleSize = isPhone ? 11.0 : 12.0;
    final leadGap = isPhone ? 8.0 : 16.0;
    final trailGap = isPhone ? 8.0 : 12.0;
    return Container(
      padding: EdgeInsets.symmetric(horizontal: hPad, vertical: vPad),
      decoration: const BoxDecoration(
        color: CymbraColors.surfaceContainerLowest,
        border: Border(
          bottom: BorderSide(color: CymbraColors.outlineVariant, width: 1),
        ),
      ),
      child: Row(
        children: [
          // Wired only when reached from the library (a route to pop back to).
          if (Navigator.of(context).canPop())
            IconButton(
              icon: const Icon(Icons.arrow_back, color: CymbraColors.onSurface),
              tooltip: l10n.backToLibrary,
              onPressed: () => Navigator.of(context).maybePop(),
            )
          else
            const Icon(Icons.arrow_back, color: CymbraColors.onSurface),
          SizedBox(width: leadGap),
          // Expanded (instead of a fixed Column + Spacer) so the title absorbs
          // the free space and shrinks gracefully on narrow windows; the texts
          // ellipsize rather than overflowing the top bar.
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Cymbra Music',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    color: CymbraColors.primary,
                    fontSize: titleSize,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                Text(
                  l10n.nowPlaying(title ?? '—'),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: subtitleSize,
                  ),
                ),
              ],
            ),
          ),
          SizedBox(width: trailGap),
          // MIDI connection status (read-only at a glance); the device itself is
          // chosen from the settings menu.
          const _MidiStatusIndicator(),
          const SizedBox(width: 8),
          // Tap to toggle the metronome; pulses on each beat. Mode-independent
          // (lives in the shared top bar), so it works in Synthesia/Staff/Partition.
          const _TempoChip(),
          const SizedBox(width: 8),
          // Consolidated music settings (MIDI device, keyboard size, hand). Lives
          // in the mode-independent top bar, so it is reachable in Synthesia,
          // Staff and Partition alike.
          const _SettingsMenu(),
          const SizedBox(width: 8),
          // Rendering mode toggle.
          const _ModeToggle(),
        ],
      ),
    );
  }
}

/// Gear button that opens the settings **end drawer** (slides in from the
/// right). A drawer is a modal route with a scrim, so — unlike the dropdown
/// menus that flickered on iPad — it cannot dismiss itself; opening it also
/// pauses the session (see [_PlayerScreenState._onEndDrawerChanged]).
class _SettingsMenu extends StatelessWidget {
  const _SettingsMenu();

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: const Icon(Icons.tune, color: CymbraColors.onSurface),
      tooltip: AppLocalizations.of(context).settings,
      onPressed: () => Scaffold.of(context).openEndDrawer(),
    );
  }
}

/// The settings categories shown in the drawer. Used as a stable key so the
/// selected category survives localization (the display titles are translated,
/// but the key that drives navigation is not).
enum _SettingsCategory {
  midiDevice,
  keyboardSize,
  keyboardVisibility,
  hand,
  language,
}

/// A setting category shown in the drawer's top-level list: its stable key,
/// localized title, icon, and a short label of the value currently in effect.
typedef _Category = ({
  _SettingsCategory key,
  String title,
  IconData icon,
  String current,
});

/// The settings drawer: a master-detail panel. The first screen lists the
/// setting **categories** (MIDI device, Keyboard size, Hand); tapping one shows
/// just that category's values, so the options are never all on screen at once.
/// Reads/updates the live [PlayerData] selection.
class _SettingsDrawer extends ConsumerStatefulWidget {
  const _SettingsDrawer({super.key});

  @override
  ConsumerState<_SettingsDrawer> createState() => _SettingsDrawerState();
}

class _SettingsDrawerState extends ConsumerState<_SettingsDrawer> {
  /// The category whose values are shown; null shows the category list.
  _SettingsCategory? _category;

  /// Returns the drawer to its top-level category list (called when it opens).
  void resetToRoot() {
    if (mounted && _category != null) setState(() => _category = null);
  }

  /// Localized title for a settings [category], resolved independently of the
  /// master list so the detail header renders even when a category is
  /// conditionally hidden (e.g. Hand for single-staff pieces).
  String _categoryTitle(AppLocalizations l10n, _SettingsCategory category) =>
      switch (category) {
        _SettingsCategory.midiDevice => l10n.settingsCategoryMidiDevice,
        _SettingsCategory.keyboardSize => l10n.settingsCategoryKeyboardSize,
        _SettingsCategory.keyboardVisibility =>
          l10n.settingsCategoryKeyboardVisibility,
        _SettingsCategory.hand => l10n.settingsCategoryHand,
        _SettingsCategory.language => l10n.settingsCategoryLanguage,
      };

  String _keyboardVisibilityLabel(AppLocalizations l10n, bool visible) =>
      visible ? l10n.keyboardShown : l10n.keyboardHidden;

  String _handLabel(AppLocalizations l10n, Hand hand) => switch (hand) {
    Hand.left => l10n.handLeft,
    Hand.right => l10n.handRight,
    Hand.both => l10n.handBoth,
  };

  String _rangeLabel(AppLocalizations l10n, KeyboardRangeMode m) =>
      m == KeyboardRangeMode.auto
      ? l10n.keyboardAutoFit
      : l10n.keyboardKeys(m.label);

  /// A radio-style value row with a leading "selected" check.
  Widget _option({
    required bool selected,
    required String label,
    required VoidCallback? onTap,
  }) => ListTile(
    leading: Icon(
      selected ? Icons.check_circle : Icons.radio_button_unchecked,
      size: 20,
      color: selected ? CymbraColors.tertiary : CymbraColors.onSurfaceVariant,
    ),
    title: Text(label, style: const TextStyle(color: CymbraColors.onSurface)),
    onTap: onTap,
  );

  /// A language row: the flag is the visible content, with an accessible label
  /// so screen readers announce the language name rather than the emoji.
  Widget _languageOption({
    required bool selected,
    required String flag,
    required String semanticLabel,
    required VoidCallback? onTap,
  }) => Semantics(
    label: semanticLabel,
    selected: selected,
    button: true,
    child: ListTile(
      leading: Icon(
        selected ? Icons.check_circle : Icons.radio_button_unchecked,
        size: 20,
        color: selected ? CymbraColors.tertiary : CymbraColors.onSurfaceVariant,
      ),
      title: Text(flag, style: const TextStyle(fontSize: 26)),
      onTap: onTap,
    ),
  );

  /// The value rows for [category], built from the current selection.
  List<Widget> _valuesFor(
    _SettingsCategory category, {
    required AppLocalizations l10n,
    required List<String> midiPorts,
    required String? connectedDevice,
    required KeyboardRangeMode keyboardRange,
    required bool keyboardVisible,
    required Hand selectedHands,
    required AppLanguage activeLanguage,
    required Player notifier,
    required bool isAndroid,
  }) {
    switch (category) {
      case _SettingsCategory.midiDevice:
        return [
          _option(
            selected: connectedDevice == null,
            label: l10n.midiAutoFirstDevice,
            onTap: () => notifier.selectMidiPort(null),
          ),
          for (final p in midiPorts)
            _option(
              selected: p == connectedDevice,
              label: p,
              onTap: () => notifier.selectMidiPort(p),
            ),
          // When no port is found on Android, the cause is usually USB OTG being
          // off or a charge-only cable — neither of which the app can fix — so
          // surface actionable guidance instead of a dead-end "No device" row.
          // Other platforms keep the plain empty row. The guidance clears on its
          // own once a port appears, since this branch only runs when empty.
          if (midiPorts.isEmpty && isAndroid)
            const _OtgGuidance()
          else if (midiPorts.isEmpty)
            _option(
              selected: false,
              label: l10n.midiNoDeviceDetected,
              onTap: null,
            ),
        ];
      case _SettingsCategory.keyboardSize:
        return [
          for (final m in KeyboardRangeMode.values)
            _option(
              selected: m == keyboardRange,
              label: _rangeLabel(l10n, m),
              onTap: () => notifier.setKeyboardRange(m),
            ),
        ];
      case _SettingsCategory.keyboardVisibility:
        return [
          for (final visible in const [true, false])
            _option(
              selected: visible == keyboardVisible,
              label: _keyboardVisibilityLabel(l10n, visible),
              onTap: () => notifier.setKeyboardVisible(visible),
            ),
        ];
      case _SettingsCategory.hand:
        return [
          for (final h in Hand.values)
            _option(
              selected: h == selectedHands,
              label: _handLabel(l10n, h),
              onTap: () => notifier.setSelectedHands(h),
            ),
        ];
      case _SettingsCategory.language:
        return [
          for (final language in AppLanguage.values)
            _languageOption(
              selected: language == activeLanguage,
              flag: language.flag,
              semanticLabel: languageName(l10n, language),
              onTap: () =>
                  ref.read(appLocaleProvider.notifier).select(language),
            ),
        ];
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final notifier = ref.read(playerProvider.notifier);
    // Behind a provider so tests can drive the Android/non-Android empty-state
    // guidance deterministically (the test VM reports the host OS otherwise).
    final isAndroid = ref.watch(isAndroidProvider);
    // The active language marks the selected flag; state is always a supported
    // locale, so this never falls back in practice.
    final activeLanguage =
        AppLanguage.fromCode(ref.watch(appLocaleProvider).languageCode) ??
        AppLanguage.en;
    final (
      midiPorts,
      connectedDevice,
      keyboardRange,
      keyboardVisible,
      selectedHands,
      twoStaves,
      mode,
    ) = ref.watch(
      playerProvider.select(
        (d) => (
          d.midiPorts,
          d.connectedDevice,
          d.keyboardRange,
          d.keyboardVisible,
          d.selectedHands,
          d.hasMultipleStaves,
          d.mode,
        ),
      ),
    );
    // The hide-keyboard toggle only makes sense in the notation modes; Synthesia
    // needs the keyboard for its cascade, so the category is omitted there.
    final canHideKeyboard = mode != RenderMode.synthesia;

    // Top-level categories (with the value currently in effect as a subtitle).
    final categories = <_Category>[
      (
        key: _SettingsCategory.midiDevice,
        title: _categoryTitle(l10n, _SettingsCategory.midiDevice),
        icon: Icons.piano,
        current: connectedDevice ?? l10n.settingsAuto,
      ),
      (
        key: _SettingsCategory.keyboardSize,
        title: _categoryTitle(l10n, _SettingsCategory.keyboardSize),
        icon: Icons.straighten,
        current: keyboardRange == KeyboardRangeMode.auto
            ? l10n.settingsAuto
            : keyboardRange.label,
      ),
      if (canHideKeyboard)
        (
          key: _SettingsCategory.keyboardVisibility,
          title: _categoryTitle(l10n, _SettingsCategory.keyboardVisibility),
          icon: keyboardVisible ? Icons.piano : Icons.piano_off,
          current: _keyboardVisibilityLabel(l10n, keyboardVisible),
        ),
      if (twoStaves)
        (
          key: _SettingsCategory.hand,
          title: _categoryTitle(l10n, _SettingsCategory.hand),
          icon: Icons.front_hand,
          current: _handLabel(l10n, selectedHands),
        ),
      (
        key: _SettingsCategory.language,
        title: _categoryTitle(l10n, _SettingsCategory.language),
        icon: Icons.language,
        current: activeLanguage.flag,
      ),
    ];

    final Widget body;
    if (_category == null) {
      // Master view: the list of categories.
      body = ListView(
        padding: EdgeInsets.zero,
        children: [
          _DrawerHeader(title: l10n.settings),
          for (final c in categories)
            ListTile(
              leading: Icon(c.icon, color: CymbraColors.onSurfaceVariant),
              title: Text(
                c.title,
                style: const TextStyle(color: CymbraColors.onSurface),
              ),
              subtitle: Text(
                c.current,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
              trailing: const Icon(
                Icons.chevron_right,
                color: CymbraColors.onSurfaceVariant,
              ),
              onTap: () => setState(() => _category = c.key),
            ),
        ],
      );
    } else {
      // Detail view: just the selected category's values, with a back affordance.
      body = ListView(
        padding: EdgeInsets.zero,
        children: [
          _DrawerHeader(
            title: _categoryTitle(l10n, _category!),
            onBack: () => setState(() => _category = null),
          ),
          ..._valuesFor(
            _category!,
            l10n: l10n,
            midiPorts: midiPorts,
            connectedDevice: connectedDevice,
            keyboardRange: keyboardRange,
            keyboardVisible: keyboardVisible,
            selectedHands: selectedHands,
            activeLanguage: activeLanguage,
            notifier: notifier,
            isAndroid: isAndroid,
          ),
        ],
      );
    }

    return Drawer(
      backgroundColor: CymbraColors.surfaceContainerHigh,
      child: SafeArea(child: body),
    );
  }
}

/// Drawer header: a title, optionally preceded by a back button (in the detail
/// view), with a bottom divider.
class _DrawerHeader extends StatelessWidget {
  final String title;
  final VoidCallback? onBack;
  const _DrawerHeader({required this.title, this.onBack});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(onBack != null ? 4 : 20, 14, 12, 12),
          child: Row(
            children: [
              if (onBack != null) ...[
                IconButton(
                  tooltip: AppLocalizations.of(context).settingsBack,
                  icon: const Icon(
                    Icons.arrow_back,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                  onPressed: onBack,
                ),
                const SizedBox(width: 4),
              ],
              Expanded(
                child: Text(
                  title,
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 18,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
            ],
          ),
        ),
        const Divider(height: 1, color: CymbraColors.outlineVariant),
      ],
    );
  }
}

/// Android-only guidance shown in the MIDI device list when no port is detected.
///
/// USB OTG is a system/hardware toggle the app cannot enable itself, and a
/// charge-only cable looks identical to a data one — so when Android enumerates
/// no MIDI port the actionable move is to point the user at those two causes.
/// Kept out of the plain "No device detected" row (used on other platforms)
/// because the OTG/cable advice is Android-specific.
class _OtgGuidance extends StatelessWidget {
  const _OtgGuidance();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(
            Icons.usb_off,
            size: 20,
            color: CymbraColors.onSurfaceVariant,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.midiOtgTitle,
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  l10n.midiOtgBody,
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 13,
                    height: 1.35,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Switch between the rendering modes (Synthesia / Staff / Partition).
class _ModeToggle extends ConsumerWidget {
  const _ModeToggle();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final mode = ref.watch(playerProvider.select((d) => d.mode));
    final notifier = ref.read(playerProvider.notifier);
    final l10n = AppLocalizations.of(context);
    // On a phone the three labelled segments are too wide for the landscape top
    // bar, so it collapses to icon-only segments (label kept as the tooltip);
    // tablet/desktop keep the labels. Labels only (no per-segment icons) off
    // phone to stay within narrow tablet widths now that there are three modes.
    final isPhone = context.isPhoneLayout;
    ButtonSegment<RenderMode> segment(
      RenderMode value,
      String label,
      IconData icon,
    ) => ButtonSegment(
      value: value,
      label: isPhone ? null : Text(label),
      icon: isPhone ? Icon(icon) : null,
      tooltip: isPhone ? label : null,
    );
    return SegmentedButton<RenderMode>(
      style: ButtonStyle(
        visualDensity: VisualDensity.compact,
        backgroundColor: WidgetStateProperty.resolveWith(
          (s) => s.contains(WidgetState.selected)
              ? CymbraColors.primaryContainer
              : CymbraColors.surfaceContainerHigh,
        ),
      ),
      // Partition (engraved two-stave) is unusable in the short phone landscape
      // viewport, so it's dropped from the toggle on phones. If the mode was set
      // to Partition on a larger screen, the selection falls back to Staff (what
      // the render area also shows) so the button keeps a valid selection.
      segments: [
        segment(
          RenderMode.synthesia,
          l10n.modeSynthesia,
          Icons.waterfall_chart,
        ),
        segment(RenderMode.staff, l10n.modeStaff, Icons.music_note),
        if (!isPhone)
          segment(RenderMode.partition, l10n.modePartition, Icons.article),
      ],
      selected: {
        (isPhone && mode == RenderMode.partition) ? RenderMode.staff : mode,
      },
      onSelectionChanged: (s) => notifier.setMode(s.first),
      showSelectedIcon: false,
    );
  }
}

/// The header **Tempo** chip, doubling as the metronome toggle.
///
/// Tapping it flips [Player.toggleMetronome]; when the metronome is enabled the
/// chip takes an active (primary-tinted) style and **pulses once per beat**,
/// harder on the accented downbeat. The pulse is the visual half of the beat (the
/// audible click is the other) and, living in the mode-independent top bar, it is
/// visible the same way in Synthesia, Staff and Partition. Watches only its own
/// slices so the per-beat pulse never rebuilds the rest of the top bar.
class _TempoChip extends ConsumerStatefulWidget {
  const _TempoChip();

  @override
  ConsumerState<_TempoChip> createState() => _TempoChipState();
}

class _TempoChipState extends ConsumerState<_TempoChip>
    with SingleTickerProviderStateMixin {
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 200),
  );
  bool _accent = false;

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final (bpm, enabled, beatCount, lastAccent) = ref.watch(
      playerProvider.select(
        (d) => (d.bpm, d.metronomeEnabled, d.beatCount, d.lastBeatAccent),
      ),
    );
    // Fire one pulse per beat: restart the decay animation whenever the beat
    // counter ticks. Done as a listen (not in build's body) so it reacts to the
    // change rather than the rebuild.
    ref.listen(playerProvider.select((d) => d.beatCount), (_, _) {
      _accent = lastAccent;
      _pulse.forward(from: 0);
    });

    final l10n = AppLocalizations.of(context);
    return Semantics(
      button: true,
      toggled: enabled,
      label: l10n.metronome,
      child: InkWell(
        onTap: () => ref.read(playerProvider.notifier).toggleMetronome(),
        borderRadius: BorderRadius.circular(8),
        child: AnimatedBuilder(
          animation: _pulse,
          builder: (context, child) {
            // Pulse intensity decays 1 → 0 over the animation; the accent pulses
            // brighter. Zero when the metronome is off.
            final intensity = enabled ? (1 - _pulse.value) : 0.0;
            final glow = intensity * (_accent ? 0.9 : 0.45);
            final baseColor = enabled
                ? Color.alphaBlend(
                    CymbraColors.primary.withValues(alpha: 0.18),
                    CymbraColors.surfaceContainerHigh,
                  )
                : CymbraColors.surfaceContainerHigh;
            return Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: Color.alphaBlend(
                  CymbraColors.primary.withValues(alpha: glow),
                  baseColor,
                ),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: enabled
                      ? CymbraColors.primary
                      : CymbraColors.surfaceContainerHigh,
                  width: 1,
                ),
              ),
              child: child,
            );
          },
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.speed,
                size: 16,
                color: enabled
                    ? CymbraColors.primary
                    : CymbraColors.onSurfaceVariant,
              ),
              const SizedBox(width: 6),
              Text(
                l10n.tempo(bpm),
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 13,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// MIDI connection status (read-only): a coloured dot + icon and a short state
/// label — green when connected, amber while a device is detected but not yet
/// connected, gray when none. The connected device's *name* is not shown here;
/// the device is listed and chosen from the settings menu (see [_SettingsMenu]).
class _MidiStatusIndicator extends ConsumerWidget {
  const _MidiStatusIndicator();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Watch only the connection state, not the playhead, so this chip is not
    // rebuilt on every player frame.
    final (connected, hasPorts) = ref.watch(
      playerProvider.select((d) => (d.midiConnected, d.midiPorts.isNotEmpty)),
    );
    final l10n = AppLocalizations.of(context);

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

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.6)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Status dot.
          Container(
            width: 9,
            height: 9,
            decoration: BoxDecoration(
              color: color,
              shape: BoxShape.circle,
              boxShadow: [
                BoxShadow(color: color.withValues(alpha: 0.7), blurRadius: 6),
              ],
            ),
          ),
          const SizedBox(width: 8),
          Icon(icon, size: 16, color: color),
          // On a phone the landscape width is tight, so the status collapses to
          // the dot + icon (the same info at a glance); the full label returns
          // on tablet/desktop. The state is also spelled out in the settings
          // menu, so nothing is lost.
          if (!context.isPhoneLayout) ...[
            const SizedBox(width: 6),
            ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 200),
              child: Text(
                label,
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
    );
  }
}

/// Floating transport bar: restart, play/pause, speed, loop, Wait Mode.
class _TransportBar extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final data = ref.watch(playerProvider);
    final notifier = ref.read(playerProvider.notifier);

    // On a phone the landscape height is scarce, so the transport bar slims
    // down: tighter margin/padding, a smaller play button, and denser icon
    // buttons — reclaiming vertical space for the render area. Tablet/desktop
    // keep the roomier floating pill.
    final isPhone = context.isPhoneLayout;
    final density = isPhone ? VisualDensity.compact : VisualDensity.standard;
    final playRadius = isPhone ? 19.0 : 26.0;
    final playIcon = isPhone ? 22.0 : 28.0;
    final gapL = isPhone ? 8.0 : 16.0;
    final gapS = isPhone ? 4.0 : 8.0;

    return Container(
      key: const Key('transport-bar'),
      // Hug the bottom edge on phones (small top gap off the keyboard, minimal
      // gap below); the roomier all-round pill stays on tablet/desktop.
      margin: isPhone
          ? const EdgeInsets.only(left: 12, right: 12, top: 4, bottom: 2)
          : const EdgeInsets.all(16),
      padding: isPhone
          ? const EdgeInsets.symmetric(horizontal: 12, vertical: 2)
          : const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: CymbraColors.outlineVariant),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          IconButton(
            visualDensity: density,
            onPressed: notifier.restart,
            icon: const Icon(
              Icons.skip_previous,
              color: CymbraColors.onSurface,
            ),
          ),
          SizedBox(width: gapS),
          // Play / pause.
          GestureDetector(
            onTap: notifier.togglePlay,
            child: CircleAvatar(
              radius: playRadius,
              backgroundColor: CymbraColors.primaryContainer,
              child: Icon(
                data.isPlaying ? Icons.pause : Icons.play_arrow,
                color: Colors.white,
                size: playIcon,
              ),
            ),
          ),
          SizedBox(width: gapL),
          // Speed.
          IconButton(
            visualDensity: density,
            onPressed: () => notifier.setSpeed(data.speed - 0.25),
            icon: const Icon(
              Icons.remove,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
          Text(
            '${(data.speed * 100).round()}% SPD',
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontWeight: FontWeight.w600,
            ),
          ),
          IconButton(
            visualDensity: density,
            onPressed: () => notifier.setSpeed(data.speed + 0.25),
            icon: const Icon(Icons.add, color: CymbraColors.onSurfaceVariant),
          ),
          SizedBox(width: gapS),
          // Wait Mode.
          TextButton.icon(
            style: TextButton.styleFrom(visualDensity: density),
            onPressed: notifier.toggleWaitMode,
            icon: Icon(
              data.waitMode ? Icons.hourglass_top : Icons.hourglass_disabled,
              color: data.waitMode
                  ? CymbraColors.secondary
                  : CymbraColors.onSurfaceVariant,
            ),
            label: Text(
              'Wait',
              style: TextStyle(
                color: data.waitMode
                    ? CymbraColors.secondary
                    : CymbraColors.onSurfaceVariant,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Overlay shown when Wait Mode freezes the cascade.
class _WaitOverlay extends StatelessWidget {
  const _WaitOverlay();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
        decoration: BoxDecoration(
          color: CymbraColors.surfaceContainer.withValues(alpha: 0.85),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: CymbraColors.secondary),
        ),
        child: const Text(
          '⏸  Play the expected note to continue',
          style: TextStyle(color: CymbraColors.secondary, fontSize: 16),
        ),
      ),
    );
  }
}

/// Engraved-notation (Partition) render mode: draws the laid-out MusicXML of the
/// loaded score and re-lays it out as the available width changes. Shows a
/// loading/empty state when no score notation is available (e.g. the demo).
class _PartitionView extends ConsumerStatefulWidget {
  const _PartitionView();

  @override
  ConsumerState<_PartitionView> createState() => _PartitionViewState();
}

class _PartitionViewState extends ConsumerState<_PartitionView> {
  final ScrollController _scroll = ScrollController();

  /// The last scroll target we animated to, so we only scroll when the cursor
  /// moves to a new line (not every frame, which would restart the animation).
  double? _lastScrollTarget;

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  /// Index of the system containing [measureIndex], or null if not found.
  int? _systemOf(int measureIndex, List<System> systems) {
    for (var i = 0; i < systems.length; i++) {
      if (systems[i].measures.contains(measureIndex)) return i;
    }
    return null;
  }

  /// Auto-scroll **per staff line (system)**, not per measure: the vertical
  /// target depends only on which system the cursor is in, so the view advances
  /// once when the playhead moves to a new line and stays put while it crosses
  /// measures within the same line (no back-and-forth jitter). The current line
  /// is centred in the viewport; look-ahead is provided by the next-line overlay
  /// (see [_buildNextLineOverlay]), not by scrolling ahead. Only while playing,
  /// so manual scrolling is undisturbed when paused.
  void _followCursor(
    PlayerData data,
    List<System> systems,
    PartitionPainter painter,
  ) {
    if (!data.isPlaying) return;
    final cursor = data.measureAt(data.elapsedMs);
    if (cursor == null) return;
    final sysIndex = _systemOf(cursor.index, systems);
    if (sysIndex == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scroll.hasClients) return;
      final max = _scroll.position.maxScrollExtent;
      if (max <= 0) return; // everything fits — no scrolling
      final viewport = _scroll.position.viewportDimension;
      final target =
          (painter.systemTopY(sysIndex) +
                  painter.systemStride / 2 -
                  viewport / 2)
              .clamp(0.0, max);
      // Only scroll when the line changes — re-issuing every frame would restart
      // (and stall) the animation.
      if (_lastScrollTarget != null &&
          (target - _lastScrollTarget!).abs() < 4) {
        return;
      }
      _lastScrollTarget = target;
      _scroll.animateTo(
        target,
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeInOut,
      );
    });
  }

  /// A small "next up" overlay showing the first two measures of the **next**
  /// line, pinned top-left. It appears only once the playhead is past the middle
  /// of the current line (so the top-left, already-played area is free to cover)
  /// and only when there is a next line. Returns null otherwise.
  Widget? _buildNextLineOverlay(
    PlayerData data,
    NotationData notation,
    double width,
    PartitionPainter mainPainter,
  ) {
    final cursor = data.measureAt(data.elapsedMs);
    if (cursor == null) return null;
    final systems = notation.systems;
    final sysIndex = _systemOf(cursor.index, systems);
    if (sysIndex == null || sysIndex + 1 >= systems.length) return null;

    final current = systems[sysIndex];
    final pos = current.measures.indexOf(cursor.index);
    if (pos < 0) return null;
    final lineProgress = (pos + cursor.fraction) / current.measures.length;
    if (lineProgress < 0.5) return null; // only near the end of the line

    // Don't cover the score when the next line is already visible on screen
    // (e.g. a tall viewport shows it below the current line) — the overlay is
    // only useful when the next line is still below the fold.
    if (_scroll.hasClients) {
      final vpTop = _scroll.offset;
      final vpBottom = vpTop + _scroll.position.viewportDimension;
      final nextTop = mainPainter.systemTopY(sysIndex + 1);
      final nextBottom = nextTop + mainPainter.systemStride;
      final visible =
          (nextBottom < vpBottom ? nextBottom : vpBottom) -
          (nextTop > vpTop ? nextTop : vpTop);
      if (visible >= mainPainter.systemStride * 0.6) return null;
    }

    // Engrave the FULL next system at the same width as the main view (so the
    // notes are exactly the same size — no down-scaling) and clip the overlay to
    // its first measure (a two-measure peek was too wide). The clip width follows
    // the painter's justification: an approximate header plus that measure's
    // share of the system width.
    final next = systems[sysIndex + 1];
    final measures = notation.document!.measures;
    var total = 0.0;
    for (final m in next.measures) {
      total += measures[m].minWidth;
    }
    final firstMin = measures[next.measures.first].minWidth;
    const headerApprox = 96.0; // clef + key + time, roughly
    final usable = (width - headerApprox).clamp(0.0, width);
    final boxWidth =
        headerApprox + (total > 0 ? firstMin / total : 1.0) * usable;
    return _NextLineOverlay(
      painter: PartitionPainter(
        document: notation.document!,
        systems: [next],
        selectedHands: data.selectedHands,
      ),
      fullWidth: width,
      boxWidth: boxWidth,
    );
  }

  @override
  Widget build(BuildContext context) {
    final notation = ref.watch(notationProvider);
    final data = ref.watch(playerProvider);

    if (notation.error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            'Could not load this score:\n${notation.error}',
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.error),
          ),
        ),
      );
    }
    if (!notation.hasDocument) {
      return const Center(
        child: Text(
          'No partition loaded — pick a score from the library.',
          style: TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
      );
    }

    return Container(
      color: CymbraColors.surfaceContainerLow,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final width = constraints.maxWidth;
          WidgetsBinding.instance.addPostFrameCallback((_) {
            ref.read(notationProvider.notifier).setAvailableWidth(width);
          });
          final painter = PartitionPainter(
            document: notation.document!,
            systems: notation.systems,
            elapsedMs: data.elapsedMs,
            measureStartMs: data.measureStartMs,
            songEndMs: data.songEndMs,
            activeNotes: data.activeNotes,
            selectedHands: data.selectedHands,
          );
          _followCursor(data, notation.systems, painter);
          final overlay = _buildNextLineOverlay(data, notation, width, painter);
          return Stack(
            children: [
              SingleChildScrollView(
                controller: _scroll,
                child: CustomPaint(
                  key: const Key('partition-canvas'),
                  painter: painter,
                  size: Size(width, painter.heightFor(width)),
                ),
              ),
              if (overlay != null) Positioned(left: 8, top: 8, child: overlay),
            ],
          );
        },
      ),
    );
  }
}

/// "Next up" peek: the first measures of the upcoming line, scaled down into a
/// small framed box (pinned top-left over the already-played start of the line).
class _NextLineOverlay extends StatelessWidget {
  final PartitionPainter painter;

  /// Width the system is engraved at — the same as the main view, so the notes
  /// are rendered at identical size (no scaling).
  final double fullWidth;

  /// Visible width of the peek (clips to roughly the first two measures).
  final double boxWidth;

  const _NextLineOverlay({
    required this.painter,
    required this.fullWidth,
    required this.boxWidth,
  });

  @override
  Widget build(BuildContext context) {
    final height = painter.heightFor(fullWidth);
    return Container(
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.94),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: CymbraColors.outlineVariant),
      ),
      padding: const EdgeInsets.fromLTRB(8, 4, 8, 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'NEXT',
            style: TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 9,
              fontWeight: FontWeight.w700,
              letterSpacing: 1.5,
            ),
          ),
          const SizedBox(height: 2),
          // The system is painted at [fullWidth] (full size) but only [boxWidth]
          // is shown; OverflowBox lets the wider canvas extend under the clip.
          SizedBox(
            width: boxWidth,
            height: height,
            child: ClipRect(
              child: OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: 0,
                maxWidth: fullWidth,
                minHeight: 0,
                maxHeight: height,
                child: SizedBox(
                  width: fullWidth,
                  height: height,
                  child: CustomPaint(
                    painter: painter,
                    size: Size(fullWidth, height),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
