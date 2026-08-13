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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';

part 'coaching_notifier.freezed.dart';
part 'coaching_notifier.g.dart';

/// The one-time hints delivered through the shared coaching mechanism (change:
/// add-welcome-onboarding, D4). Each hint is shown at the **first relevant use**
/// of its feature and never again once dismissed.
///
/// The rating-deck hint keeps its **legacy** preferences key so a device that
/// already dismissed it before this change does not see it a second time.
enum CoachHint {
  /// Swipe/tap gestures of the rating deck (change: add-app-score-rating).
  ratingDeck('rating_coach_seen'),

  /// How points, badges and the shop work, on the first visit to the rewards
  /// surface (change: add-curation-rewards).
  rewards('coach_rewards_seen'),

  /// What making a profile public means, next to the visibility control
  /// (change: add-public-curator-profiles). The age gate itself is a required
  /// legal step and is deliberately **not** a one-time hint.
  goingPublic('coach_going_public_seen'),

  /// The guided in-context player sequence (D8) has run at least once.
  playerTour('coach_player_tour_seen'),

  /// That staff symbols can be long-pressed for help (change: add-notation-help).
  /// Shown the first time a score is viewed in the player.
  notationHelp('coach_notation_help_seen');

  const CoachHint(this.prefsKey);

  /// Preferences key under which this hint's "seen" flag lives.
  final String prefsKey;
}

/// One step of the guided player sequence, in the order it is walked. The first
/// steps point at real controls inside the pre-play setup surface; the last one
/// points at the transport bar behind it (the setup surface applies its drafts
/// and closes when the sequence reaches it — see `_CoachStepListener` in the
/// setup modal).
enum PlayerCoachStep {
  /// Picking the piano sound (`piano-sound-selection`).
  pianoSound,

  /// Seeing the connected MIDI instrument / selecting one manually (`midi`).
  midiDevice,

  /// Choosing which hand(s) to play (`hand-selection`).
  hands,

  /// Rewinding one measure / long-pressing to pick a practice passage
  /// (change: add-in-game-measure-selection). Lives on the player's transport
  /// bar, outside the setup surface.
  measureRewind;

  /// The step after this one, or `null` when this is the last.
  PlayerCoachStep? get next {
    final i = index + 1;
    return i < values.length ? values[i] : null;
  }
}

/// State of the shared coaching system.
///
/// [loaded] is false until the persisted flags arrive, so a hint never flashes
/// at a returning user. [step] is the active guided-sequence step (`null` when
/// the sequence is not running), and [replayArmed] records a replay requested
/// from the help surface, which runs the next time the player is opened.
@freezed
sealed class CoachingState with _$CoachingState {
  const CoachingState._();

  const factory CoachingState({
    @Default(false) bool loaded,
    @Default(<CoachHint>{}) Set<CoachHint> seen,
    PlayerCoachStep? step,
    @Default(false) bool replayArmed,
  }) = _CoachingState;

  /// Whether [hint] should be shown now: the flags are known and it was never
  /// dismissed on this device.
  bool shouldShow(CoachHint hint) => loaded && !seen.contains(hint);

  /// Whether the guided player sequence is currently running.
  bool get tourRunning => step != null;
}

/// The coaching controller (D9): owns the "seen" set and the guided-sequence
/// step, so every hint and the player walk-through share one mechanism, one
/// store, and one set of rules. All logic lives here — the overlay widget only
/// renders what this exposes — which keeps it unit-testable without native
/// storage (persistence goes through the injectable [preferencesServiceProvider]
/// seam: a fake in tests, `shared_preferences` in production).
@Riverpod(keepAlive: true)
class Coaching extends _$Coaching {
  @override
  CoachingState build() {
    _restore();
    return const CoachingState();
  }

  Future<void> _restore() async {
    final prefs = ref.read(preferencesServiceProvider);
    final seen = <CoachHint>{};
    try {
      for (final hint in CoachHint.values) {
        if (await prefs.getString(hint.prefsKey) == 'true') seen.add(hint);
      }
    } catch (_) {
      // Storage unavailable → treat everything as seen so we never nag on a
      // device whose preferences cannot record the dismissal.
      state = CoachingState(loaded: true, seen: CoachHint.values.toSet());
      return;
    }
    state = state.copyWith(loaded: true, seen: seen);
  }

  /// Records [hint] as seen so it never appears again. Best-effort persistence:
  /// the in-memory set already suppresses it for this session.
  Future<void> markSeen(CoachHint hint) async {
    if (state.seen.contains(hint)) return;
    state = state.copyWith(seen: {...state.seen, hint});
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(hint.prefsKey, 'true');
    } catch (_) {}
  }

  /// Starts the guided player sequence on the first player visit — or on a
  /// replay armed from help. A no-op when it is already running, when the flags
  /// are still loading, or when it has already run and no replay was requested,
  /// so re-entering the player never re-walks a user who knows the controls.
  void startPlayerTour() {
    if (!state.loaded || state.tourRunning) return;
    if (state.seen.contains(CoachHint.playerTour) && !state.replayArmed) return;
    state = state.copyWith(step: PlayerCoachStep.values.first);
  }

  /// Advances to the next control; finishing the last one ends the sequence.
  void nextStep() {
    final current = state.step;
    if (current == null) return;
    final next = current.next;
    if (next == null) {
      _endTour();
      return;
    }
    state = state.copyWith(step: next);
  }

  /// Leaves the sequence at any point — it never blocks playing, and it is not
  /// shown again (it stays replayable from help).
  void skipTour() {
    if (!state.tourRunning) return;
    _endTour();
  }

  /// Called when the pre-play setup surface — home of the sequence's early
  /// steps — leaves the screen. Ends the sequence so it never points at
  /// controls that are gone, EXCEPT when it has already advanced to the
  /// transport measure-rewind step, whose control lives on the player screen
  /// behind the surface: closing the setup is then precisely what reveals it.
  void setupSurfaceClosed() {
    final step = state.step;
    if (step == null || step == PlayerCoachStep.measureRewind) return;
    _endTour();
  }

  void _endTour() {
    state = state.copyWith(step: null, replayArmed: false);
    markSeen(CoachHint.playerTour);
  }

  /// Arms a replay from the help surface: the guided sequence runs again the
  /// next time the user opens a score (that is where the controls live).
  void armPlayerTourReplay() => state = state.copyWith(replayArmed: true);
}
