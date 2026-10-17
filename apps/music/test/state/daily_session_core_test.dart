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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/daily_session_core.dart';
import 'package:music/state/today_ledger.dart';

import '../support/daily_fixtures.dart';

const _day = '2026-10-17';

DailySession _compose({
  CourseCompletionState completion = const CourseCompletionState(),
  Map<String, CourseManifest>? manifests,
  TodayLedger ledger = const TodayLedger(),
  DailyGoal goal = DailyGoal.normal,
  String dayKey = _day,
}) => composeDailySession(
  listings: dailyListings,
  completion: completion,
  manifests: manifests ?? dailyManifests(),
  ledger: ledger,
  goal: goal,
  dayKey: dayKey,
);

DailyItem? _item(DailySession s, DailyItemKind kind) =>
    s.items.where((i) => i.kind == kind).firstOrNull;

/// Units 1 done (a3 passed), unit 2 started: the learner is on b2.
const _midPath = CourseCompletionState(
  loaded: true,
  completed: {'a1', 'a2', 'b1'},
  passed: {'a3'},
  bestPct: {'a3': 91},
);

void main() {
  group('the local day', () {
    test('is the device calendar day, zero-padded', () {
      expect(localDayKey(DateTime(2026, 3, 7, 23, 59)), '2026-03-07');
      expect(localDayKey(DateTime(2026, 12, 31, 0, 0, 1)), '2026-12-31');
    });

    test('ordinals count days, so the round-robin moves one step a day', () {
      expect(dayOrdinal('2026-10-18') - dayOrdinal('2026-10-17'), 1);
      expect(dayOrdinal('garbage'), 0);
    });

    test('the next midnight is the start of the following local day', () {
      final next = nextLocalMidnight(DateTime(2026, 10, 17, 22, 30));
      expect(next, DateTime(2026, 10, 18));
    });
  });

  group('keys and goals', () {
    test('a review key round-trips', () {
      final key = reviewKeyOf('b1', 1);
      expect(key, 'review:b1#1');
      expect(parseReviewKey(key), (courseId: 'b1', index: 1));
      expect(parseReviewKey('b1'), isNull);
      expect(parseReviewKey('review:b1#x'), isNull);
    });

    test('a goal parses by name and defaults to normal', () {
      expect(DailyGoal.parse('full'), DailyGoal.full);
      expect(DailyGoal.parse('nope'), DailyGoal.normal);
      expect(DailyGoal.parse(null), DailyGoal.normal);
      expect(DailyGoal.light.count, 1);
      expect(DailyGoal.full.count, 3);
    });
  });

  group('composition', () {
    test('no courses → no session', () {
      final s = composeDailySession(
        listings: const [],
        completion: const CourseCompletionState(),
        manifests: const {},
        ledger: const TodayLedger(),
        goal: DailyGoal.normal,
        dayKey: _day,
      );
      expect(s.items, isEmpty);
      expect(s.completedToday, isFalse);
    });

    test('start of the path: the first lesson, nothing else yet', () {
      final s = _compose();
      expect(s.items.map((i) => i.kind), [DailyItemKind.lesson]);
      expect(s.items.single.courseId, 'a1');
      expect(s.items.single.done, isFalse);
      expect(s.pathFinished, isFalse);
      expect(s.doneCount, 0);
    });

    test('mid-path: the lesson, its own piece step, a review from the last '
        'finished unit', () {
      final s = _compose(completion: _midPath);
      expect(_item(s, DailyItemKind.lesson)?.courseId, 'b2');
      final piece = _item(s, DailyItemKind.piece)!;
      expect(piece.courseId, 'b2');
      expect(piece.piece, isA<PieceBlock>());
      expect(piece.title, {'en': 'Piece of b2'});
      final review = _item(s, DailyItemKind.review)!;
      expect(['a1', 'a2'], contains(review.courseId));
      expect(review.block, isNotNull);
      expect(isDrillBlock(review.block!), isTrue);
      expect(review.title['en'], anyOf('Play the G', 'Tap the beat'));
      expect(s.items, hasLength(3));
    });

    test('a lesson without a piece step: a passed piece to replay, '
        'round-robin by day', () {
      const completion = CourseCompletionState(
        loaded: true,
        completed: {'a1', 'a2', 'b1', 'b2'},
        passed: {'a3', 'b2'},
      );
      final today = _compose(completion: completion, dayKey: '2026-10-17');
      final tomorrow = _compose(completion: completion, dayKey: '2026-10-18');
      expect(_item(today, DailyItemKind.lesson)?.courseId, 'c1');
      final p1 = _item(today, DailyItemKind.piece)!;
      final p2 = _item(tomorrow, DailyItemKind.piece)!;
      expect({p1.courseId, p2.courseId}, {'a3', 'b2'});
      expect(p1.done, isFalse);
    });

    test('a finished path: a piece, a review from the latest unit with a '
        'drill, and a replay suggestion — no lesson', () {
      const completion = CourseCompletionState(
        loaded: true,
        completed: {'a1', 'a2', 'b1', 'c1'},
        passed: {'a3', 'b2', 'c2'},
      );
      final s = _compose(completion: completion);
      expect(s.pathFinished, isTrue);
      expect(_item(s, DailyItemKind.lesson), isNull);
      expect(_item(s, DailyItemKind.piece), isNotNull);
      // Unit 3 has no drill: the review falls back to unit 2.
      expect(_item(s, DailyItemKind.review)?.courseId, 'b1');
      final replay = _item(s, DailyItemKind.replay)!;
      expect(replay.done, isFalse);
      expect(dailyListings.map((l) => l.id), contains(replay.courseId));
    });

    test('everything done today reads done and completes the goal', () {
      const completion = CourseCompletionState(
        loaded: true,
        completed: {'a1', 'a2', 'b1', 'b2'},
        passed: {'a3', 'b2'},
      );
      final ledger = TodayLedger(
        localDay: _day,
        lessonIds: const {'b2'},
        pieceStepKeys: const {'b2'},
        reviewKeys: {reviewKeyOf('a1', 1)},
      );
      final s = _compose(completion: completion, ledger: ledger);
      // The lesson completed today stays the session's lesson, shown done —
      // not the next one along.
      expect(_item(s, DailyItemKind.lesson)?.courseId, 'b2');
      expect(s.items.every((i) => i.done), isTrue);
      expect(s.doneCount, 3);
      final review = _item(s, DailyItemKind.review)!;
      expect(review.key, 'review:a1#1');
      expect(review.title, {'en': 'Play the G'});
      expect(s.completedToday, isTrue);
      expect(
        _compose(
          completion: completion,
          ledger: ledger,
          goal: DailyGoal.full,
        ).completedToday,
        isTrue,
      );
    });

    test('the goal is the ring\'s denominator only', () {
      final ledger = TodayLedger(localDay: _day, lessonIds: const {'b2'});
      final light = _compose(
        completion: _midPath,
        ledger: ledger,
        goal: DailyGoal.light,
      );
      expect(light.doneCount, 1);
      expect(light.completedToday, isTrue, reason: '1 of 1');
      expect(light.items, hasLength(3), reason: 'the other items stay');
      final normal = _compose(completion: _midPath, ledger: ledger);
      expect(normal.completedToday, isFalse, reason: '1 of 2');
    });

    test('yesterday\'s ledger counts for nothing', () {
      final ledger = TodayLedger(
        localDay: '2026-10-16',
        lessonIds: const {'b2'},
        pieceStepKeys: const {'b2'},
      );
      final s = _compose(completion: _midPath, ledger: ledger);
      expect(s.doneCount, 0);
      expect(_item(s, DailyItemKind.lesson)?.courseId, 'b2');
      expect(_item(s, DailyItemKind.lesson)?.done, isFalse);
    });

    test('a missing manifest never drops the item: it opens the lesson', () {
      // The lesson's own manifest is unknown: no own piece step is known, so
      // the passed piece (whose manifest IS known) is offered instead.
      final s = _compose(
        completion: _midPath,
        manifests: dailyManifests(['a1', 'a2', 'a3']),
      );
      expect(_item(s, DailyItemKind.piece)?.courseId, 'a3');
      // A piece run today whose manifest is unknown is still listed, by the
      // lesson's title, with nothing to run directly.
      final s2 = _compose(
        completion: _midPath,
        manifests: dailyManifests(['a1', 'a2', 'a3']),
        ledger: TodayLedger(localDay: _day, pieceStepKeys: const {'b2'}),
      );
      final piece = _item(s2, DailyItemKind.piece)!;
      expect(piece.done, isTrue);
      expect(piece.piece, isNull);
      expect(piece.title, {'en': 'Lesson b2'});
    });

    test('the manifests wanted are the few the composition can use', () {
      final wanted = dailySessionManifestsWanted(
        listings: dailyListings,
        completion: _midPath,
        ledger: TodayLedger(localDay: _day, reviewKeys: {reviewKeyOf('a2', 1)}),
        dayKey: _day,
      );
      expect(wanted, containsAll(['b2', 'a3', 'a1', 'a2']));
      expect(wanted, isNot(contains('c1')));
      expect(wanted, isNot(contains('c2')));
    });
  });
}
