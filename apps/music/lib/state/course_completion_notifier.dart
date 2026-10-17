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

import '../services/course_catalog_service.dart';
import '../services/preferences_service.dart';
import 'session_notifier.dart';

part 'course_completion_notifier.freezed.dart';
part 'course_completion_notifier.g.dart';

/// Which courses the user has completed (change: add-notation-courses).
///
/// This is the **local** source of truth (persisted in `shared_preferences`),
/// which the cross-device sync (tranche 5) will reconcile with the server on top
/// of. [loaded] is false until the persisted set arrives, so a tile never flashes
/// "not completed" for a returning user.
@freezed
sealed class CourseCompletionState with _$CourseCompletionState {
  const CourseCompletionState._();

  const factory CourseCompletionState({
    @Default(false) bool loaded,
    @Default(<String>{}) Set<String> completed,

    /// Courses whose piece step was passed at least once (change:
    /// add-music-piece-lessons). A course can be passed without being
    /// completed (the learner left after the run) and completed without
    /// being passed (visited).
    @Default(<String>{}) Set<String> passed,

    /// Best overall synchronisation reached on each course's piece steps.
    /// Monotonic: a later, lower run never lowers it.
    @Default(<String, double>{}) Map<String, double> bestPct,

    /// The server's first-completion / first-pass moments (epoch millis), as
    /// of the last reconciliation — never persisted locally. Today's ledger
    /// reads them so a course done today on another device counts here
    /// (change: add-music-daily-session).
    @Default(<String, int>{}) Map<String, int> completedAtMs,
    @Default(<String, int>{}) Map<String, int> passedAtMs,

    /// Best overall synchronisation per piece STEP, keyed by [pieceStepKey]
    /// — device-local (found while dogfooding: a lesson chaining two piece
    /// steps showed its second step "passed" before it was played, from the
    /// course-level best). The server keeps the course-level aggregate.
    @Default(<String, double>{}) Map<String, double> stepBest,
  }) = _CourseCompletionState;

  /// Whether course [id] is marked completed.
  bool isCompleted(String id) => completed.contains(id);

  /// Whether a piece step of course [id] was ever passed.
  bool isPassed(String id) => passed.contains(id);

  /// Completed, with a piece step run but never passed — the path's third
  /// state. Without any piece run the course is plainly completed.
  bool isVisited(String id) =>
      completed.contains(id) && bestPct.containsKey(id) && !passed.contains(id);

  /// Done for the "next lesson" choice: completed or passed.
  bool isDone(String id) => completed.contains(id) || passed.contains(id);
}

const String _kCompletedKey = 'courses.completed.v1';
const String _kPassedKey = 'courses.passed.v1';
const String _kBestKey = 'courses.best.v1';
const String _kStepBestKey = 'courses.stepbest.v1';

/// The key of one piece step of a course: the course and the step's index
/// among the steps the lesson player shows.
String pieceStepKey(String courseId, int stepIndex) => '$courseId#$stepIndex';

/// Owns the completed-course set: restores it, and marks a course completed once
/// (idempotent) when the lesson player reaches the end. Persistence goes through
/// the injectable [preferencesServiceProvider] seam, so it is unit-testable
/// without native storage.
@Riverpod(keepAlive: true)
class CourseCompletion extends _$CourseCompletion {
  /// The restore in flight: the server reconciliation waits for it, so a
  /// local-only completion or piece result recorded on a previous launch is
  /// pushed up instead of being read as absent.
  late final Future<void> _restoring;

  @override
  CourseCompletionState build() {
    _restoring = _restore();
    // Reconcile with the server whenever online access becomes available (sign-in
    // or reconnect), and once now if already online.
    ref.listen(canUseOnlineServicesProvider, (_, online) {
      if (online) _sync();
    });
    if (ref.read(canUseOnlineServicesProvider)) _sync();
    return const CourseCompletionState();
  }

  /// Reconciles with the cross-device server record: pulls the server's
  /// completed set into the local one, and pushes any local-only completions up
  /// (so a guest's completions land on their account after sign-in). Best-effort:
  /// any failure (offline, signed out) leaves the local set untouched.
  Future<void> _sync() async {
    await _restoring;
    final service = ref.read(courseProgressServiceProvider);
    final List<CourseProgressRecord> records;
    try {
      records = await service.progress();
    } catch (_) {
      return; // offline / signed out → local only
    }
    final serverIds = {
      for (final r in records)
        if (r.completed) r.courseId,
    };
    final serverPassed = {
      for (final r in records)
        if (r.passed) r.courseId,
    };
    final serverBest = <String, double>{
      for (final r in records) r.courseId: ?r.bestSyncPct,
    };
    // Merge: completed and passed are unions, the best is the greatest — so
    // two devices reconcile whatever the order their records arrive in.
    final mergedBest = {...state.bestPct};
    serverBest.forEach((id, b) {
      final local = mergedBest[id];
      if (local == null || b > local) mergedBest[id] = b;
    });
    final next = state.copyWith(
      loaded: true,
      completed: {...state.completed, ...serverIds},
      passed: {...state.passed, ...serverPassed},
      bestPct: mergedBest,
      completedAtMs: {
        for (final r in records)
          if (r.completedAtMs > 0) r.courseId: r.completedAtMs,
      },
      passedAtMs: {
        for (final r in records)
          if (r.passedAtMs > 0) r.courseId: r.passedAtMs,
      },
    );
    if (next != state) state = next;
    for (final id in state.completed.difference(serverIds)) {
      try {
        await service.recordCompletion(id);
      } catch (_) {}
    }
    // Push local-only piece results up (a guest's runs land on their account
    // after sign-in): a pass the server lacks, or a better best.
    for (final entry in state.bestPct.entries) {
      final id = entry.key;
      final passedHere = state.passed.contains(id);
      final serverB = serverBest[id];
      final needsPass = passedHere && !serverPassed.contains(id);
      final needsBest = serverB == null || entry.value > serverB;
      if (!needsPass && !needsBest) continue;
      try {
        await service.recordPieceResult(
          id,
          passed: passedHere,
          syncPct: entry.value,
        );
      } catch (_) {}
    }
  }

  Future<void> _restore() async {
    final prefs = ref.read(preferencesServiceProvider);
    final ids = await _restoreIds(prefs, _kCompletedKey);
    final passed = await _restoreIds(prefs, _kPassedKey);
    final best = await _restoreMap(prefs, _kBestKey);
    final stepBest = await _restoreMap(prefs, _kStepBestKey);
    // Union with anything already marked in memory: a completion recorded before
    // the restore finished (a race) must not be clobbered by the restored set.
    final mergedBest = {...best};
    state.bestPct.forEach((id, b) {
      final stored = mergedBest[id];
      if (stored == null || b > stored) mergedBest[id] = b;
    });
    final mergedStepBest = {...stepBest};
    state.stepBest.forEach((k, b) {
      final stored = mergedStepBest[k];
      if (stored == null || b > stored) mergedStepBest[k] = b;
    });
    state = state.copyWith(
      loaded: true,
      completed: {...ids, ...state.completed},
      passed: {...passed, ...state.passed},
      bestPct: mergedBest,
      stepBest: mergedStepBest,
    );
  }

  /// A persisted id → percentage map, or empty when absent/unreadable.
  static Future<Map<String, double>> _restoreMap(
    PreferencesService prefs,
    String key,
  ) async {
    try {
      final raw = await prefs.getString(key);
      if (raw != null) {
        final decoded = jsonDecode(raw);
        if (decoded is Map) {
          return {
            for (final e in decoded.entries)
              if (e.key is String && e.value is num)
                e.key as String: (e.value as num).toDouble(),
          };
        }
      }
    } catch (_) {}
    return {};
  }

  /// A persisted id set, or empty when absent/unreadable (storage unavailable →
  /// an empty set, still marked loaded so the UI shows; nothing is falsely done).
  static Future<Set<String>> _restoreIds(
    PreferencesService prefs,
    String key,
  ) async {
    try {
      final raw = await prefs.getString(key);
      if (raw != null) {
        final decoded = jsonDecode(raw);
        if (decoded is List) {
          return {
            for (final e in decoded)
              if (e is String) e,
          };
        }
      }
    } catch (_) {}
    return {};
  }

  /// Records one piece-step run of course [id] (change: add-music-piece-
  /// lessons): a pass is kept forever, the best only rises. Persisted locally,
  /// and pushed to the server when online so other devices see it. Never marks
  /// the course completed — reaching the end of the lesson does that.
  Future<void> recordPieceResult(
    String id, {
    required bool passed,
    required double syncPct,
    String? stepKey,
  }) async {
    final best = state.bestPct[id];
    final nextBest = best == null || syncPct > best ? syncPct : best;
    final nextPassed = passed ? {...state.passed, id} : state.passed;
    final stepB = stepKey == null ? null : state.stepBest[stepKey];
    final stepRises = stepKey != null && (stepB == null || syncPct > stepB);
    if (nextBest == best &&
        nextPassed.length == state.passed.length &&
        !stepRises) {
      return;
    }
    state = state.copyWith(
      passed: nextPassed,
      bestPct: {...state.bestPct, id: nextBest},
      stepBest: stepRises
          ? {...state.stepBest, stepKey: syncPct}
          : state.stepBest,
    );
    final prefs = ref.read(preferencesServiceProvider);
    try {
      await prefs.setString(_kPassedKey, jsonEncode(state.passed.toList()));
      await prefs.setString(_kBestKey, jsonEncode(state.bestPct));
      await prefs.setString(_kStepBestKey, jsonEncode(state.stepBest));
    } catch (_) {}
    if (ref.read(canUseOnlineServicesProvider)) {
      try {
        await ref
            .read(courseProgressServiceProvider)
            .recordPieceResult(id, passed: passed, syncPct: syncPct);
      } catch (_) {}
    }
  }

  /// Marks course [id] completed and persists the set. A no-op if already
  /// completed, so replays never churn storage.
  Future<void> markCompleted(String id) async {
    if (state.completed.contains(id)) return;
    final next = {...state.completed, id};
    state = state.copyWith(completed: next);
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(_kCompletedKey, jsonEncode(next.toList()));
    } catch (_) {}
    // Record on the server too, so it shows on the user's other devices.
    if (ref.read(canUseOnlineServicesProvider)) {
      try {
        await ref.read(courseProgressServiceProvider).recordCompletion(id);
      } catch (_) {}
    }
  }
}
