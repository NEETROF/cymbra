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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../courses/course_manifest.dart';
import '../services/course_catalog_service.dart';
import 'course_completion_notifier.dart';
import 'daily_session_core.dart';
import 'today_ledger.dart';

part 'daily_session_notifier.g.dart';

/// Today's session (change: add-music-daily-session): the pure composition
/// fed with the catalogue, the completion state, today's ledger, the goal and
/// the manifests it asks for. Loading while the catalogue loads; empty when
/// there are no courses.
@riverpod
Future<DailySession> dailySession(Ref ref) async {
  final listings = await ref.watch(coursesProvider.future);
  final completion = ref.watch(courseCompletionProvider);
  final ledger = ref.watch(dailyLedgerProvider);
  final goal = ref.watch(dailyGoalSettingProvider);
  final dayKey = ref.watch(localDayProvider);
  if (listings.isEmpty) {
    return DailySession(items: const [], goal: goal, pathFinished: false);
  }
  final wanted = dailySessionManifestsWanted(
    listings: listings,
    completion: completion,
    ledger: ledger,
    dayKey: dayKey,
  );
  // Each manifest comes from the course cache (offline-safe, never throws).
  final loaded = await Future.wait([
    for (final id in wanted)
      ref.watch(courseManifestProvider(id).future).then((m) => (id, m)),
  ]);
  final manifests = <String, CourseManifest>{
    for (final (id, m) in loaded) id: ?m,
  };
  return composeDailySession(
    listings: listings,
    completion: completion,
    manifests: manifests,
    ledger: ledger,
    goal: goal,
    dayKey: dayKey,
  );
}
