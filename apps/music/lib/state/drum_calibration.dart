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

/// The calibration pass as a pure state machine (change:
/// add-drum-input-calibration): which piece is being asked for, what has been
/// recorded so far, and how the pass ended.
///
/// No stream, no clock, no storage — the notifier owns those. Everything here
/// is a function of the previous state and one event, which is what makes the
/// two rules that matter testable without a kit: a step records the **next**
/// stroke and never a stale one, and a number another piece already claims is
/// reported rather than silently reassigned.
library;

import 'drum_input_mapping.dart';

/// Where a pass is: not begun, asking, or finished one of two ways.
enum CalibrationOutcome {
  /// Not begun. The surface shows the stored mapping instead, and a stroke
  /// changes nothing — a player reading their table must be able to play their
  /// kit without it silently recording.
  idle,

  /// Still asking. Nothing is written to storage in this state.
  running,

  /// Every step was answered or skipped — the only state whose result is
  /// stored (design D4: abandoning changes nothing).
  completed,

  /// Left before the end. The previously stored mapping stands untouched.
  abandoned,
}

/// A number offered for the current piece that another piece already holds.
///
/// Surfaced rather than resolved: on a real kit this usually means the player
/// struck the wrong pad, and quietly reassigning it would produce a mapping
/// wrong in two places at once (design D4).
class CalibrationConflict {
  const CalibrationConflict({required this.number, required this.heldBy});

  /// The number that arrived.
  final int number;

  /// The piece that already claims it.
  final String heldBy;

  @override
  bool operator ==(Object other) =>
      other is CalibrationConflict &&
      other.number == number &&
      other.heldBy == heldBy;

  @override
  int get hashCode => Object.hash(number, heldBy);
}

/// One pass over the pieces, in progress or finished.
class CalibrationState {
  const CalibrationState({
    required this.pieces,
    this.index = 0,
    this.recorded = const {},
    this.conflict,
    this.outcome = CalibrationOutcome.idle,
    this.armedAtMs = 0,
  });

  /// The pieces this pass asks for, in order.
  final List<String> pieces;

  /// Which one is being asked for. Equal to `pieces.length` once the last step
  /// has been answered — the pass is then [CalibrationOutcome.completed].
  final int index;

  /// What has been learned so far: piece identity → the number this device
  /// sends. Nothing is written to storage until the pass completes.
  final Map<String, int> recorded;

  /// The unresolved collision the last stroke produced, if any. While this is
  /// set the step has not advanced: the player strikes again or reassigns.
  final CalibrationConflict? conflict;

  final CalibrationOutcome outcome;

  /// The event timestamp this step armed at. A stroke stamped at or before it
  /// belongs to an earlier step and is discarded (design D4) — the same lesson
  /// as the Wait-Mode stale-stroke fix, where a stroke from a previous lap
  /// opened the next run's first onset.
  final int armedAtMs;

  bool get isRunning => outcome == CalibrationOutcome.running;

  /// Begin (or begin again) from the first piece, discarding anything a
  /// previous pass in this session had learned but not stored.
  CalibrationState start({required int atMs}) => CalibrationState(
    pieces: pieces,
    outcome: CalibrationOutcome.running,
    armedAtMs: atMs,
  );

  /// The piece being asked for, or null when the pass is over.
  String? get currentPiece =>
      index >= 0 && index < pieces.length ? pieces[index] : null;

  /// Steps answered or skipped so far — what a progress indicator reads.
  int get step => index;
  int get total => pieces.length;

  /// The mapping this pass has built. Read on completion; meaningless before,
  /// because an abandoned pass must change nothing.
  DrumInputMapping get mapping => DrumInputMapping(recorded);

  CalibrationState copyWith({
    int? index,
    Map<String, int>? recorded,
    CalibrationOutcome? outcome,
    int? armedAtMs,
    bool clearConflict = false,
    CalibrationConflict? conflict,
  }) => CalibrationState(
    pieces: pieces,
    index: index ?? this.index,
    recorded: recorded ?? this.recorded,
    conflict: clearConflict ? null : (conflict ?? this.conflict),
    outcome: outcome ?? this.outcome,
    armedAtMs: armedAtMs ?? this.armedAtMs,
  );

  /// The state after a stroke on [number], stamped [atMs] by the engine.
  ///
  /// Three outcomes, in order:
  /// * the pass is over, or the stroke is **stale** — nothing changes;
  /// * the number belongs to another piece — the conflict is reported and the
  ///   step stays put;
  /// * otherwise it is recorded and the pass moves on (completing on the last
  ///   step).
  CalibrationState afterStroke(int number, {required int atMs}) {
    final piece = currentPiece;
    if (!isRunning || piece == null) return this;
    // A stroke stamped at or before the moment this step armed was played for
    // an earlier one. Recording it would answer a question nobody had heard
    // yet, and on a kit that is one tap away from happening.
    if (atMs <= armedAtMs) return this;

    final heldBy = _pieceHolding(number, except: piece);
    if (heldBy != null) {
      return copyWith(
        conflict: CalibrationConflict(number: number, heldBy: heldBy),
      );
    }
    return _advance(recorded: {...recorded, piece: number}, atMs: atMs);
  }

  /// Resolve a reported conflict by moving the number to the current piece —
  /// the "it really is this pad" answer. The piece that held it loses its
  /// entry, so the mapping never claims one number twice.
  CalibrationState reassign({required int atMs}) {
    final c = conflict;
    final piece = currentPiece;
    if (!isRunning || c == null || piece == null) return this;
    return _advance(
      recorded: {...recorded, piece: c.number}..remove(c.heldBy),
      atMs: atMs,
    );
  }

  /// Dismiss the conflict and wait for another stroke — the "I hit the wrong
  /// pad" answer, which is the common one.
  CalibrationState strikeAgain({required int atMs}) =>
      isRunning ? copyWith(clearConflict: true, armedAtMs: atMs) : this;

  /// Move on without recording anything: a kit that has no such piece must be
  /// able to pass rather than invent one.
  CalibrationState skip({required int atMs}) =>
      isRunning ? _advance(recorded: recorded, atMs: atMs) : this;

  /// Step back and drop what that step had learned, so re-striking it starts
  /// clean rather than colliding with its own previous answer.
  CalibrationState back({required int atMs}) {
    if (!isRunning || index == 0) return this;
    final previous = pieces[index - 1];
    return copyWith(
      index: index - 1,
      recorded: {...recorded}..remove(previous),
      clearConflict: true,
      armedAtMs: atMs,
    );
  }

  /// Leave the pass. Nothing it learned is kept — the stored mapping is
  /// whatever it was before the pass began.
  CalibrationState abandon() =>
      isRunning ? copyWith(outcome: CalibrationOutcome.abandoned) : this;

  CalibrationState _advance({
    required Map<String, int> recorded,
    required int atMs,
  }) {
    final next = index + 1;
    return copyWith(
      index: next,
      recorded: recorded,
      clearConflict: true,
      armedAtMs: atMs,
      outcome: next >= pieces.length
          ? CalibrationOutcome.completed
          : CalibrationOutcome.running,
    );
  }

  String? _pieceHolding(int number, {required String except}) {
    for (final entry in recorded.entries) {
      if (entry.value == number && entry.key != except) return entry.key;
    }
    return null;
  }
}
