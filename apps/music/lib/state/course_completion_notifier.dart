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
  }) = _CourseCompletionState;

  /// Whether course [id] is marked completed.
  bool isCompleted(String id) => completed.contains(id);
}

const String _kCompletedKey = 'courses.completed.v1';

/// Owns the completed-course set: restores it, and marks a course completed once
/// (idempotent) when the lesson player reaches the end. Persistence goes through
/// the injectable [preferencesServiceProvider] seam, so it is unit-testable
/// without native storage.
@Riverpod(keepAlive: true)
class CourseCompletion extends _$CourseCompletion {
  @override
  CourseCompletionState build() {
    _restore();
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
    final service = ref.read(courseProgressServiceProvider);
    final Set<String> serverIds;
    try {
      serverIds = await service.completedCourseIds();
    } catch (_) {
      return; // offline / signed out → local only
    }
    if (!serverIds.every(state.completed.contains)) {
      state = state.copyWith(
        loaded: true,
        completed: {...state.completed, ...serverIds},
      );
    }
    for (final id in state.completed.difference(serverIds)) {
      try {
        await service.recordCompletion(id);
      } catch (_) {}
    }
  }

  Future<void> _restore() async {
    final prefs = ref.read(preferencesServiceProvider);
    var ids = <String>{};
    try {
      final raw = await prefs.getString(_kCompletedKey);
      if (raw != null) {
        final decoded = jsonDecode(raw);
        if (decoded is List) {
          ids = {
            for (final e in decoded)
              if (e is String) e,
          };
        }
      }
    } catch (_) {
      // Storage unavailable → an empty set, still marked loaded so the UI shows
      // (nothing is falsely "completed").
    }
    // Union with anything already marked in memory: a completion recorded before
    // the restore finished (a race) must not be clobbered by the restored set.
    state = state.copyWith(
      loaded: true,
      completed: {...ids, ...state.completed},
    );
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
