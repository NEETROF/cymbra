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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'player_data.dart';
import 'session_summary.dart';

part 'lesson_context.g.dart';

/// What a lesson's `piece` step imposes on the player for ONE run (change:
/// add-music-piece-lessons): the settings, the bar to clear, the words for the
/// header — and where the result goes. A per-run override layer: nothing here
/// is ever written to the player's own preferences.
class LessonContext {
  const LessonContext({
    required this.lessonTitle,
    required this.hands,
    required this.speed,
    required this.waitMode,
    required this.metronome,
    required this.readingAid,
    required this.passPct,
    required this.onResult,
  });

  /// The lesson's title, shown in the player's header instead of the brand.
  final String lessonTitle;
  final Hand hands;
  final double speed;
  final bool waitMode;
  final bool metronome;
  final NoteReadingAid readingAid;

  /// Overall synchronisation percentage that passes the step.
  final int passPct;

  /// Receives each finished scored run of this lesson run; the step decides
  /// passed / visited from it.
  final void Function(SessionResult result) onResult;

  /// Whether [result] clears the step's bar.
  bool passes(SessionResult result) => result.overallSyncPct >= passPct;
}

/// The lesson context the NEXT player run is opened with (change:
/// add-music-piece-lessons). Armed by the piece step right before it opens the
/// score, read once by the player when it builds, cleared when the player is
/// left — exactly like the selected score, which is set before the player
/// builds. Null for every ordinary (free play) run.
@Riverpod(keepAlive: true)
class PendingLessonContext extends _$PendingLessonContext {
  @override
  LessonContext? build() => null;

  void arm(LessonContext context) => state = context;

  void clear() => state = null;
}
