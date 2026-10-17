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

import 'dart:convert';

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'course_completion_notifier.dart';
import 'daily_session_core.dart';
import 'rating_activity_notifier.dart' show nowFnProvider;

part 'today_ledger.freezed.dart';
part 'today_ledger.g.dart';

/// What was done today, on this device, keyed by the local day (change:
/// add-music-daily-session, D2). A new local day is a new, empty ledger.
@freezed
sealed class TodayLedger with _$TodayLedger {
  const TodayLedger._();

  const factory TodayLedger({
    /// The [localDayKey] this ledger is for; empty before anything was done.
    @Default('') String localDay,

    /// Lessons completed today (a replay of a done lesson lands here too).
    @Default(<String>{}) Set<String> lessonIds,

    /// Courses whose piece step was run today, passed or not.
    @Default(<String>{}) Set<String> pieceStepKeys,

    /// Reviews finished today (`review:<course>#<block>`).
    @Default(<String>{}) Set<String> reviewKeys,

    /// Whether today's completion cue was raised — once a day.
    @Default(false) bool cueRaised,

    /// False until the persisted ledger was read.
    @Default(false) bool loaded,
  }) = _TodayLedger;

  Map<String, Object?> toJson() => {
    'localDay': localDay,
    'lessonIds': lessonIds.toList(),
    'pieceStepKeys': pieceStepKeys.toList(),
    'reviewKeys': reviewKeys.toList(),
    'cueRaised': cueRaised,
  };

  /// A persisted ledger, or null when unreadable.
  static TodayLedger? fromJson(Object? decoded) {
    if (decoded is! Map) return null;
    final day = decoded['localDay'];
    if (day is! String) return null;
    Set<String> ids(Object? raw) => {
      if (raw is List)
        for (final e in raw)
          if (e is String) e,
    };
    return TodayLedger(
      localDay: day,
      lessonIds: ids(decoded['lessonIds']),
      pieceStepKeys: ids(decoded['pieceStepKeys']),
      reviewKeys: ids(decoded['reviewKeys']),
      cueRaised: decoded['cueRaised'] == true,
      loaded: true,
    );
  }
}

const String _kLedgerKey = 'daily.ledger.v1';
const String _kGoalKey = 'daily.goal.v1';

/// Owns today's ledger: restores it, rolls it over when the local day changes
/// (any change — a clock set backwards starts a fresh day too), records what
/// the lesson player, the piece runner and the review report, and merges the
/// server's "first completed / first passed today" moments so a course done
/// today on another device counts here. Persisted through the preferences
/// seam.
@Riverpod(keepAlive: true)
class DailyLedger extends _$DailyLedger {
  late final Future<void> _restoring;

  @override
  TodayLedger build() {
    _restoring = _restore();
    // Reactive, never poked: the completion state carries the server's
    // moments after each reconciliation.
    ref.listen(courseCompletionProvider, (_, next) => _mergeServer(next));
    return const TodayLedger();
  }

  String _today() => localDayKey(ref.read(nowFnProvider)());

  /// [ledger] if it is today's, else an empty ledger for today.
  TodayLedger _forToday(TodayLedger ledger) {
    final today = _today();
    return ledger.localDay == today
        ? ledger
        : TodayLedger(localDay: today, loaded: ledger.loaded);
  }

  Future<void> _restore() async {
    TodayLedger? stored;
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(_kLedgerKey);
      if (raw != null) stored = TodayLedger.fromJson(jsonDecode(raw));
    } catch (_) {}
    // Union with anything marked before the restore finished (a race).
    var next = _forToday(state);
    if (stored != null && stored.localDay == next.localDay) {
      next = next.copyWith(
        lessonIds: {...stored.lessonIds, ...next.lessonIds},
        pieceStepKeys: {...stored.pieceStepKeys, ...next.pieceStepKeys},
        reviewKeys: {...stored.reviewKeys, ...next.reviewKeys},
        cueRaised: stored.cueRaised || next.cueRaised,
      );
    }
    state = next.copyWith(loaded: true);
  }

  /// A lesson completed (or replayed) today.
  Future<void> markLesson(String courseId) => _add(lessonIds: {courseId});

  /// A piece step run today, passed or not.
  Future<void> markPiece(String courseId) => _add(pieceStepKeys: {courseId});

  /// A review finished today.
  Future<void> markReview(String key) => _add(reviewKeys: {key});

  /// Today's completion cue was raised.
  Future<void> markCueRaised() async {
    await _restoring;
    final base = _forToday(state);
    if (base.cueRaised) return;
    await _write(base.copyWith(cueRaised: true));
  }

  Future<void> _add({
    Set<String> lessonIds = const {},
    Set<String> pieceStepKeys = const {},
    Set<String> reviewKeys = const {},
  }) async {
    final base = _forToday(state);
    final next = base.copyWith(
      lessonIds: {...base.lessonIds, ...lessonIds},
      pieceStepKeys: {...base.pieceStepKeys, ...pieceStepKeys},
      reviewKeys: {...base.reviewKeys, ...reviewKeys},
    );
    if (next == base) return;
    await _write(next);
  }

  Future<void> _write(TodayLedger next) async {
    state = next;
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(_kLedgerKey, jsonEncode(next.toJson()));
    } catch (_) {}
  }

  void _mergeServer(CourseCompletionState completion) {
    final today = _today();
    bool isToday(int ms) =>
        localDayKey(DateTime.fromMillisecondsSinceEpoch(ms)) == today;
    final lessons = {
      for (final e in completion.completedAtMs.entries)
        if (isToday(e.value)) e.key,
    };
    final pieces = {
      for (final e in completion.passedAtMs.entries)
        if (isToday(e.value)) e.key,
    };
    if (lessons.isEmpty && pieces.isEmpty) return;
    _add(lessonIds: lessons, pieceStepKeys: pieces);
  }
}

/// The learner's daily goal, a device setting defaulting to normal (D3).
@Riverpod(keepAlive: true)
class DailyGoalSetting extends _$DailyGoalSetting {
  @override
  DailyGoal build() {
    _restore();
    return DailyGoal.normal;
  }

  Future<void> _restore() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(_kGoalKey);
      if (raw != null) state = DailyGoal.parse(raw);
    } catch (_) {}
  }

  Future<void> set(DailyGoal goal) async {
    state = goal;
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(_kGoalKey, goal.name);
    } catch (_) {}
  }
}

/// The current local day, as the session sees it. Refreshed by the session
/// block's midnight timer, so a session left open overnight recomposes.
@Riverpod(keepAlive: true)
class LocalDay extends _$LocalDay {
  @override
  String build() => localDayKey(ref.read(nowFnProvider)());

  void refresh() {
    final key = localDayKey(ref.read(nowFnProvider)());
    if (key != state) state = key;
  }
}
