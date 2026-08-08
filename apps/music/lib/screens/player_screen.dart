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
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../layout/device_class.dart';
import '../painters/notation_palette.dart';
import '../painters/partition_painter.dart';
import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../painters/staff_painter.dart';
import '../painters/synthesia_painter.dart';
import '../src/rust/api/musicxml.dart' show System;
import '../state/notation_data.dart';
import '../state/notation_notifier.dart';
import '../state/score_catalog.dart';
import '../state/performance_scoring.dart';
import '../state/play_sync_notifier.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../state/player_preferences.dart';
import '../state/post_play_rating_notifier.dart';
import '../state/session_summary.dart';
import '../state/session_summary_store.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/countdown_overlay.dart';
import '../widgets/mistake_replay.dart';
import '../widgets/playback_progress_bar.dart';
import '../widgets/reading_aid.dart';
import '../widgets/score_chip.dart';
import '../widgets/scoring_overlay.dart';
import '../widgets/post_play_rating.dart';
import '../widgets/session_summary_modal.dart';
import 'pre_play_setup_modal.dart';
import 'score_load_message.dart';

/// Main screen of the Cymbra player: top bar, rendering area
/// (Synthesia or Staff), keyboard, and transport bar.
class PlayerScreen extends ConsumerStatefulWidget {
  const PlayerScreen({super.key});

  @override
  ConsumerState<PlayerScreen> createState() => _PlayerScreenState();
}

class _PlayerScreenState extends ConsumerState<PlayerScreen>
    with TickerProviderStateMixin {
  final FocusNode _focusNode = FocusNode();
  late final Ticker _ticker;
  Duration _lastTick = Duration.zero;

  /// Breathing cycle of the expected-key highlight while Wait Mode holds
  /// playback — the non-intrusive wait indicator (no banner over the play
  /// surface). Runs only while the gate is blocked.
  late final AnimationController _waitPulse;

  /// Whether the pre-play setup modal has been shown for this opening (one per
  /// pushed `PlayerScreen`, so it re-appears on every open).
  bool _setupShown = false;

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

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick)..start();
    _waitPulse = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1100),
    );
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
    _waitPulse.dispose();
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
    // Show the pre-play setup modal once, as soon as the score has loaded. The
    // openScore guard usually pre-loads before this screen mounts, so the
    // build-body call catches the already-loaded case; the listener covers a
    // load that resolves after mount.
    ref.listen(notationProvider.select((n) => n.hasDocument), (_, hasDoc) {
      if (hasDoc) _maybeShowSetup();
    });
    // Breathe the expected-key highlight only while the Wait Mode gate holds
    // playback; back to the steady highlight the moment it releases.
    ref.listen(playerProvider.select((d) => d.blocked), (_, blocked) {
      if (blocked) {
        _waitPulse.repeat();
      } else {
        _waitPulse
          ..stop()
          ..value = 0;
      }
    });
    _maybeShowSetup();
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _onKey,
      child: PopScope(
        // Intercept the exit ONLY while a rating prompt is actually owed, so the
        // native back gesture (and iOS's interactive swipe-back) keeps working
        // untouched in every other case — a bundled score, a guest, an
        // already-rated piece, or a run too short to have an opinion about.
        // The top bar's `maybePop` funnels through here too, so both exit paths
        // share one code path.
        canPop: !ref.watch(postPlayRatingEligibleProvider(reachedEnd: false)),
        onPopInvokedWithResult: (didPop, _) {
          if (!didPop) unawaited(_requestExit());
        },
        child: _buildPlayer(context),
      ),
    );
  }

  /// Leave the player, offering to rate the piece on the way out when it is
  /// eligible (change: add-post-play-rating-prompt).
  ///
  /// The sheet is **non-blocking**: whatever the user does with it — rate, skip,
  /// tap outside, back — the pop happens. Re-entrant calls (a second back press
  /// while the sheet is up) pop immediately rather than stacking prompts.
  Future<void> _requestExit() async {
    if (_leaving) return;
    _leaving = true;
    final navigator = Navigator.of(context);
    if (ref.read(postPlayRatingEligibleProvider(reachedEnd: false))) {
      await showPostPlayRatingSheet(context);
    }
    if (!mounted) return;
    // `pop`, not `maybePop`: the sheet has had its one chance and the exit is not
    // negotiable from here.
    navigator.pop();
  }

  /// True once an exit is under way, so a second back press while the sheet is up
  /// does not stack another prompt.
  bool _leaving = false;

  Widget _buildPlayer(BuildContext context) {
    return Scaffold(
      // Settings open as the pre-play popup (the gear button), so there is no
      // end drawer here — it is the same modal shown at the start of the game.
      // On a phone the bottom safe-area inset (home-indicator zone) wastes
      // scarce landscape height below the transport bar, so we let the bar
      // extend into it (its own small margin keeps a hair of clearance, and
      // the centred controls sit clear of the thin indicator). Tablet/desktop
      // keep the full safe area.
      body: Stack(
        children: [
          SafeArea(
            bottom: !context.isPhoneLayout,
            child: Builder(
              builder: (context) {
                // Touch form factors (phone + tablet) rail the transport
                // controls on the right; desktop keeps the bottom bar.
                final useRail = context.deviceClass != DeviceClass.desktop;
                final Widget renderArea = Consumer(
                  builder: (context, ref, child) {
                    final data = ref.watch(playerProvider);
                    // Load state of the selected score, surfaced in every
                    // render mode (not just Partition): a fetch in flight
                    // shows a spinner, a failure shows an error banner.
                    final notation = ref.watch(notationProvider);
                    final hasSelection =
                        ref.watch(selectedScoreProvider) != null;
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
                        // Synthesia always shows the keyboard (its cascade
                        // aligns to the keys); the Portée honours the user's
                        // hide-keyboard setting; the engraved Partition never
                        // shows it — the notation's own expected-note emphasis
                        // already says what to play, and the freed height is
                        // what keeps the current + next lines on screen.
                        final showKeyboard =
                            data.mode == RenderMode.synthesia ||
                            (data.mode == RenderMode.staff &&
                                data.keyboardVisible);
                        // Reading aid: the awaited notes' names ride on their
                        // own keys, so the aid takes no screen space of its own.
                        final naming = namingConventionOf(context);
                        final aid = readingAidViewOf(
                          data,
                          solfege: naming.solfege,
                          frenchRe: naming.frenchRe,
                        );
                        return Column(
                          children: [
                            // Clip the render area so a painter (e.g. high notes /
                            // beams in Staff mode) never draws over the top bar or
                            // the keyboard below.
                            Expanded(
                              child: Stack(
                                children: [
                                  ClipRect(
                                    key: const Key('render-area'),
                                    child: _buildRenderArea(
                                      layout,
                                      data,
                                      notation,
                                      hasSelection: hasSelection,
                                      isPhone: context.isPhoneLayout,
                                    ),
                                  ),
                                  // The rhythm card floats over the bottom of
                                  // the score, only while the gate holds — so
                                  // it costs the render area no height at all.
                                  const ReadingAidOverlay(),
                                ],
                              ),
                            ),
                            // Thin score-duration/playhead bar, directly above
                            // the keyboard (or closing the render area when
                            // the keyboard is hidden). Hides itself when the
                            // loaded score has no timing.
                            const PlaybackProgressBar(),
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
                                  // Rebuilds each pulse frame only while the
                                  // Wait Mode gate is blocked (the controller
                                  // is stopped otherwise).
                                  child: AnimatedBuilder(
                                    animation: _waitPulse,
                                    builder: (context, _) => CustomPaint(
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
                                        chosenWindow: data.keyboardChosenWindow,
                                        noteLabels: aid.names,
                                        solfege: naming.solfege,
                                        frenchRe: naming.frenchRe,
                                        labelFontFamily: DefaultTextStyle.of(
                                          context,
                                        ).style.fontFamily,
                                        // 0→1→0 breathing over each cycle.
                                        waitPulse:
                                            0.5 -
                                            0.5 *
                                                math.cos(
                                                  2 *
                                                      math.pi *
                                                      _waitPulse.value,
                                                ),
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
                );
                // Phone (landscape-locked): keep the transport controls off
                // the bottom home-indicator zone by railing them on the right,
                // giving the render area + keyboard the full height. The tablet
                // rails them too (consistency + freed height); only the desktop
                // keeps the classic bottom bar.
                return Column(
                  children: [
                    const _TopBar(),
                    if (useRail)
                      Expanded(
                        child: Row(
                          children: [
                            Expanded(child: renderArea),
                            const _TransportBar(axis: Axis.vertical),
                          ],
                        ),
                      )
                    else ...[
                      Expanded(child: renderArea),
                      const _TransportBar(),
                    ],
                  ],
                );
              },
            ),
          ),
          // Race-game style get-ready countdown, centred over everything.
          const Positioned.fill(child: CountdownOverlay()),
        ],
      ),
    );
  }

  /// Shows the pre-play setup modal once per opening, after the score has loaded.
  /// Safe to call from build: it never shows synchronously — it schedules the
  /// dialog on the next frame and is guarded so it fires at most once.
  void _maybeShowSetup() {
    if (_setupShown || !ref.read(notationProvider).hasDocument) return;
    _setupShown = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) showPrePlaySetup(context);
    });
  }

  /// Persists the finished run and presents the summary modal, then clears the
  /// result and applies the player's chosen action. Choosing "replay" opens the
  /// mistake replay on the real score and returns to the summary afterwards, so
  /// the player can still retry or close.
  Future<void> _onScoredRunFinished(SessionResult result) async {
    await ref.read(sessionSummaryStoreProvider).save(result);
    // Capture the session into the durable play-activity outbox (change: add-play-
    // activity-profile) — before any network attempt, so the stat is never lost;
    // the sender delivers it reliably (retry-until-acked). A no-op for guests.
    unawaited(
      ref.read(playSyncNotifierProvider.notifier).captureSession(result),
    );
    // Capture the score context now — the piece is unchanged after the run.
    final score = ReplayScore.fromPlayer(ref.read(playerProvider));
    while (true) {
      if (!mounted) return;
      final action = await showSessionSummary(context, result);
      if (!mounted) return;
      if (action == SummaryAction.replay) {
        await showMistakeReplay(context, score, result);
        continue; // back to the summary after the replay
      }
      ref.read(performanceScorerProvider.notifier).clearLastResult();
      if (action == SummaryAction.retry) {
        // Fresh start from the top → replays the get-ready countdown.
        ref.read(playerProvider.notifier).restartFromTop();
      } else {
        // Quit: leave play mode and return to the previous screen (library).
        Navigator.of(context).maybePop();
      }
      return;
    }
  }

  Widget _buildRenderArea(
    PianoLayout layout,
    PlayerData data,
    NotationData notation, {
    required bool hasSelection,
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
          // Synthesia always shows the keyboard, so effects are always anchored.
          Positioned.fill(child: ScoringOverlay(layout: layout)),
          Positioned.fill(
            child: _ScoreLoadOverlay(
              notation: notation,
              hasSelection: hasSelection,
            ),
          ),
        ],
      );
    }
    // Standard staff mode (synchronized, horizontal scrolling). The score-size
    // setting scales the notation (noteScale) and narrows the look-ahead
    // window by the same factor so note size and spacing grow together.
    final sizeFactor = ref
        .watch(playerPreferencesProvider.select((p) => p.scoreSize))
        .factor;
    final palette = NotationPalette.of(
      ref.watch(playerPreferencesProvider.select((p) => p.notationTheme)),
    );
    return Stack(
      children: [
        Positioned.fill(
          child: Container(
            color: palette.background,
            child: CustomPaint(
              painter: StaffPainter(
                notes: data.visibleNotes,
                rests: data.visibleRests,
                elapsedMs: data.elapsedMs,
                activeNotes: data.activeNotes,
                bpm: data.bpm,
                songEndMs: data.songEndMs,
                keyFifths: data.keyFifths,
                measureKeyFifths: data.measureKeyFifths,
                beats: data.beats,
                beatType: data.beatType,
                measureStartMs: data.measureStartMs,
                noteScale: sizeFactor,
                lookAheadMs: StaffPainter.defaultLookAheadMs / sizeFactor,
                palette: palette,
              ),
              size: Size.infinite,
            ),
          ),
        ),
        // In the staff view the sparks anchor to the keyboard line, so hide them
        // when the keyboard is hidden (the gauge still shows).
        Positioned.fill(
          child: ScoringOverlay(
            layout: layout,
            showEffects: data.keyboardVisible,
          ),
        ),
        Positioned.fill(
          child: _ScoreLoadOverlay(
            notation: notation,
            hasSelection: hasSelection,
          ),
        ),
      ],
    );
  }
}

/// Loading / error feedback for the selected score, overlaid on the time-based
/// render modes (Synthesia, Staff) which otherwise paint a silent blank surface
/// while a fetch is in flight or after it fails. A load in progress shows a
/// spinner; a failure shows an error banner; anything else (no selection, or a
/// loaded document) renders nothing so the painter shows through.
class _ScoreLoadOverlay extends StatelessWidget {
  const _ScoreLoadOverlay({required this.notation, required this.hasSelection});

  final NotationData notation;
  final bool hasSelection;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    if (notation.failure != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            scoreLoadFailureMessage(l10n, notation.failure!),
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.error),
          ),
        ),
      );
    }
    // A selected score with neither a document nor an error yet is still loading.
    if (hasSelection && !notation.hasDocument) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator(),
            const SizedBox(height: 12),
            Text(
              l10n.playerScoreLoading,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ],
        ),
      );
    }
    return const SizedBox.shrink();
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
          // Live score (sync % + combo) during a scored run — the single HUD
          // location, so nothing ever floats over the play surface.
          ScoreChip(compact: isPhone),
          const SizedBox(width: 8),
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

/// Gear button that reopens the **pre-play setup popup** as the in-game settings
/// surface — the same modal shown at the start of the game, for consistency (no
/// separate drawer / language menu). It pauses the session while open and
/// restores the prior play/pause state when it closes.
class _SettingsMenu extends ConsumerWidget {
  const _SettingsMenu();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return IconButton(
      icon: const Icon(Icons.tune, color: CymbraColors.onSurface),
      tooltip: AppLocalizations.of(context).settings,
      onPressed: () async {
        final notifier = ref.read(playerProvider.notifier);
        final wasPlaying = ref.read(playerProvider).isPlaying;
        if (wasPlaying) notifier.setPlaying(false);
        await showPrePlaySetup(context, inGame: true);
        if (wasPlaying) notifier.setPlaying(true);
      },
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

/// Floating transport controls: restart, play/pause, speed, Wait Mode.
///
/// On a phone the app runs landscape-locked, where the bottom edge is the iOS
/// home-indicator zone — a horizontal bar there overlaps it. So touch form
/// factors lay these out as a vertical [Axis.vertical] side rail on the right
/// (like the rating deck), clearing the indicator and handing the freed height
/// back to the keyboard: slim on the phone, roomier on the tablet (bigger play
/// button, "Wait" label, "% SPD"). Only the desktop keeps the horizontal
/// floating pill along the bottom.
class _TransportBar extends ConsumerWidget {
  const _TransportBar({this.axis = Axis.horizontal});

  /// Layout direction: the desktop bottom bar ([Axis.horizontal]) or the
  /// phone/tablet side rail ([Axis.vertical]).
  final Axis axis;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final data = ref.watch(playerProvider);
    final notifier = ref.read(playerProvider.notifier);

    // On a phone the landscape space is scarce, so the controls slim down:
    // tighter margin/padding, a smaller play button, and denser icon buttons.
    final isPhone = context.isPhoneLayout;
    final vertical = axis == Axis.vertical;
    final density = isPhone ? VisualDensity.compact : VisualDensity.standard;
    final playRadius = isPhone ? 19.0 : 26.0;
    final playIcon = isPhone ? 22.0 : 28.0;
    final gapL = isPhone ? 8.0 : 16.0;
    final gapS = isPhone ? 4.0 : 8.0;

    final restart = IconButton(
      visualDensity: density,
      // Restart from the top, replaying the get-ready countdown.
      onPressed: notifier.restartFromTop,
      icon: const Icon(Icons.skip_previous, color: CymbraColors.onSurface),
    );
    // Play / pause. Play arms the get-ready countdown (from the top); pause
    // stops immediately.
    final playPause = GestureDetector(
      onTap: data.isPlaying
          ? () => notifier.setPlaying(false)
          : notifier.startPlayback,
      child: CircleAvatar(
        radius: playRadius,
        backgroundColor: CymbraColors.primaryContainer,
        child: Icon(
          data.isPlaying ? Icons.pause : Icons.play_arrow,
          color: Colors.white,
          size: playIcon,
        ),
      ),
    );
    final speedDown = IconButton(
      visualDensity: density,
      onPressed: () => notifier.setSpeed(data.speed - 0.25),
      icon: const Icon(Icons.remove, color: CymbraColors.onSurfaceVariant),
    );
    final speedUp = IconButton(
      visualDensity: density,
      onPressed: () => notifier.setSpeed(data.speed + 0.25),
      icon: const Icon(Icons.add, color: CymbraColors.onSurfaceVariant),
    );
    // The slim phone rail drops the "SPD" suffix; the roomier tablet rail and
    // the desktop bottom bar keep it.
    final speedLabel = Text(
      isPhone
          ? '${(data.speed * 100).round()}%'
          : '${(data.speed * 100).round()}% SPD',
      style: const TextStyle(
        color: CymbraColors.onSurface,
        fontWeight: FontWeight.w600,
      ),
    );
    // Wait Mode — an icon-only toggle on the slim phone rail, labelled
    // everywhere else (tablet rail + desktop bottom bar).
    final waitColor = data.waitMode
        ? CymbraColors.secondary
        : CymbraColors.onSurfaceVariant;
    final waitIcon = Icon(
      data.waitMode ? Icons.hourglass_top : Icons.hourglass_disabled,
      color: waitColor,
    );
    final wait = isPhone
        ? IconButton(
            visualDensity: density,
            tooltip: 'Wait',
            onPressed: notifier.toggleWaitMode,
            icon: waitIcon,
          )
        : TextButton.icon(
            style: TextButton.styleFrom(visualDensity: density),
            onPressed: notifier.toggleWaitMode,
            icon: waitIcon,
            label: Text('Wait', style: TextStyle(color: waitColor)),
          );

    return Container(
      key: const Key('transport-bar'),
      // Rail (phone + tablet): hug the right edge, centred vertically. Bottom
      // bar (desktop only): the roomier all-round floating pill. The phone
      // branch of the horizontal case is kept for robustness if reused.
      margin: vertical
          ? const EdgeInsets.symmetric(vertical: 8, horizontal: 4)
          : isPhone
          ? const EdgeInsets.only(left: 12, right: 12, top: 4, bottom: 2)
          : const EdgeInsets.all(16),
      padding: vertical
          ? const EdgeInsets.symmetric(horizontal: 2, vertical: 8)
          : isPhone
          ? const EdgeInsets.symmetric(horizontal: 12, vertical: 2)
          : const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: CymbraColors.outlineVariant),
      ),
      child: vertical
          ? Column(
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: [
                playPause,
                SizedBox(height: gapL),
                restart,
                SizedBox(height: gapL),
                speedUp,
                speedLabel,
                speedDown,
                SizedBox(height: gapS),
                wait,
              ],
            )
          : Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                restart,
                SizedBox(width: gapS),
                playPause,
                SizedBox(width: gapL),
                speedDown,
                speedLabel,
                speedUp,
                SizedBox(width: gapS),
                wait,
              ],
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
  /// measures within the same line (no back-and-forth jitter). The scroll
  /// budget goes to the **current + next** pair: the pair is centred when the
  /// viewport is tall enough, and on shorter viewports the current line hugs
  /// the top so the next line — the notes the player must read in a moment —
  /// stays fully visible below. The already-played line above earns no space
  /// (it is dimmed anyway); look-ahead is the scroll position, not an overlay.
  /// Only while playing, so manual scrolling is undisturbed when paused.
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
      final lead = ((viewport - painter.systemStride * 2) / 2).clamp(
        0.0,
        viewport * 0.30,
      );
      var target = (painter.systemTopY(sysIndex) - lead).clamp(0.0, max);
      // Guarantee the NEXT line's bass staff: if the pair overflows the
      // viewport, scroll further — sacrificing up to the current line's top
      // padding (never its staves) — so the upcoming notes stay readable
      // down to the bass clef.
      if (sysIndex + 1 < systems.length) {
        final nextBottom =
            painter.systemTopY(sysIndex + 1) + painter.systemHeight;
        final needed = nextBottom - viewport; // min offset for a full next line
        if (needed > target) {
          final cap = painter.systemTopY(sysIndex) + painter.systemTopPad;
          target = math.min(needed, cap).clamp(0.0, max);
        }
      }
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

  @override
  Widget build(BuildContext context) {
    final notation = ref.watch(notationProvider);
    final data = ref.watch(playerProvider);

    if (notation.failure != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            scoreLoadFailureMessage(
              AppLocalizations.of(context),
              notation.failure!,
            ),
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

    final sizeFactor = ref
        .watch(playerPreferencesProvider.select((p) => p.scoreSize))
        .factor;
    final palette = NotationPalette.of(
      ref.watch(playerPreferencesProvider.select((p) => p.notationTheme)),
    );
    return Container(
      color: palette.background,
      child: LayoutBuilder(
        builder: (context, constraints) {
          // The scoring HUD lives in the top bar (nothing floats over the
          // engraving), so the full width is available. Systems are laid out
          // against the width divided by the score-size factor; painting at the
          // real width with a scaled staff space then restores exact size, so
          // glyphs and spacing grow together and lines re-wrap.
          final engraveWidth = constraints.maxWidth;
          final staffSpace = 12.0 * sizeFactor;
          WidgetsBinding.instance.addPostFrameCallback((_) {
            ref
                .read(notationProvider.notifier)
                .setAvailableWidth(engraveWidth / sizeFactor);
          });
          final painter = PartitionPainter(
            document: notation.document!,
            systems: notation.systems,
            elapsedMs: data.elapsedMs,
            measureStartMs: data.measureStartMs,
            songEndMs: data.songEndMs,
            activeNotes: data.activeNotes,
            selectedHands: data.selectedHands,
            staffSpace: staffSpace,
            palette: palette,
          );
          _followCursor(data, notation.systems, painter);
          return SingleChildScrollView(
            controller: _scroll,
            // Rebuild the canvas as the view scrolls so the painter can cull
            // to the visible systems (only ~2–3 lines are engraved per frame
            // instead of the whole score — the fix for large-score lag).
            child: ListenableBuilder(
              listenable: _scroll,
              builder: (context, _) {
                // The position isn't fully attached on the first build(s):
                // guard pixels/viewport before reading them, falling back to
                // the layout height (paints from the top — offset 0).
                final pos = _scroll.hasClients ? _scroll.position : null;
                final viewTop = pos != null && pos.hasPixels ? pos.pixels : 0.0;
                final viewHeight = pos != null && pos.hasViewportDimension
                    ? pos.viewportDimension
                    : constraints.maxHeight;
                return CustomPaint(
                  key: const Key('partition-canvas'),
                  painter: PartitionPainter(
                    document: notation.document!,
                    systems: notation.systems,
                    elapsedMs: data.elapsedMs,
                    measureStartMs: data.measureStartMs,
                    songEndMs: data.songEndMs,
                    activeNotes: data.activeNotes,
                    selectedHands: data.selectedHands,
                    viewTop: viewTop,
                    viewBottom: viewTop + viewHeight,
                    staffSpace: staffSpace,
                    palette: palette,
                  ),
                  size: Size(engraveWidth, painter.heightFor(engraveWidth)),
                );
              },
            ),
          );
        },
      ),
    );
  }
}
