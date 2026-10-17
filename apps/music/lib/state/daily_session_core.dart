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

import 'package:flutter/foundation.dart';

import '../courses/course_manifest.dart';
import 'course_completion_notifier.dart';
import 'today_ledger.dart';

/// How many items a day the learner signed up for (change:
/// add-music-daily-session, D3). The item list always shows every item; the
/// goal is only the ring's denominator.
enum DailyGoal {
  light(1),
  normal(2),
  full(3);

  const DailyGoal(this.count);

  /// Items a day.
  final int count;

  /// The persisted name, back to a goal — normal when unknown.
  static DailyGoal parse(String? raw) => DailyGoal.values.firstWhere(
    (g) => g.name == raw,
    orElse: () => DailyGoal.normal,
  );
}

enum DailyItemKind { lesson, piece, review, replay }

/// One item of today's session: what it names, whether it was done today,
/// and what opening it needs.
@immutable
class DailyItem {
  const DailyItem({
    required this.kind,
    required this.courseId,
    required this.key,
    required this.title,
    required this.subtitle,
    required this.done,
    this.piece,
    this.block,
  });

  final DailyItemKind kind;
  final String courseId;

  /// The ledger key: the course id for a lesson/replay, the course id for a
  /// piece step, `review:<course>#<block>` for a review.
  final String key;
  final InlineText title;
  final InlineText subtitle;
  final bool done;

  /// The piece step to run, when the manifest was available.
  final PieceBlock? piece;

  /// The drill block the review runs, when the manifest was available.
  final CourseBlock? block;
}

/// Today's session: its items, the goal, and whether the path is finished.
@immutable
class DailySession {
  const DailySession({
    required this.items,
    required this.goal,
    required this.pathFinished,
  });

  final List<DailyItem> items;
  final DailyGoal goal;

  /// Every course completed or passed: the session is a piece, a review and a
  /// replay suggestion, with no lesson item.
  final bool pathFinished;

  int get doneCount => items.where((i) => i.done).length;

  /// Done ≥ goal — the completion moment's condition (D5).
  bool get completedToday => items.isNotEmpty && doneCount >= goal.count;
}

/// The device's local calendar day of [now], `yyyy-mm-dd` — the one
/// definition of "today" the session uses (the streak's day is the server's
/// reading of the same local clock).
String localDayKey(DateTime now) {
  final l = now.toLocal();
  String two(int v) => v.toString().padLeft(2, '0');
  return '${l.year}-${two(l.month)}-${two(l.day)}';
}

/// Days since the epoch for a [localDayKey] — the round-robin's clock.
int dayOrdinal(String dayKey) {
  final parts = dayKey.split('-');
  if (parts.length != 3) return 0;
  final y = int.tryParse(parts[0]);
  final m = int.tryParse(parts[1]);
  final d = int.tryParse(parts[2]);
  if (y == null || m == null || d == null) return 0;
  return DateTime.utc(y, m, d).millisecondsSinceEpoch ~/
      Duration.millisecondsPerDay;
}

/// The next local midnight after [now] — when the session recomposes.
DateTime nextLocalMidnight(DateTime now) {
  final l = now.toLocal();
  return DateTime(l.year, l.month, l.day + 1);
}

String reviewKeyOf(String courseId, int blockIndex) =>
    'review:$courseId#$blockIndex';

/// `review:<course>#<index>` back to its parts, or null when malformed.
({String courseId, int index})? parseReviewKey(String key) {
  if (!key.startsWith('review:')) return null;
  final hash = key.lastIndexOf('#');
  if (hash < 0) return null;
  final index = int.tryParse(key.substring(hash + 1));
  if (index == null) return null;
  return (courseId: key.substring('review:'.length, hash), index: index);
}

/// The blocks a review can be made of: short drills with their own gate.
bool isDrillBlock(CourseBlock block) =>
    block is ReadPlayBlock ||
    block is RhythmTapBlock ||
    block is EarChoiceBlock;

/// Units (by slug, catalogue order) → their lessons.
Map<String, List<CourseListing>> _units(List<CourseListing> listings) {
  final units = <String, List<CourseListing>>{};
  for (final l in listings) {
    final slug = l.unit.isNotEmpty ? l.unit : 'level:${l.level}';
    (units[slug] ??= []).add(l);
  }
  return units;
}

/// The done lessons the review may draw from, as unit pools in preference
/// order: the fully completed units, latest first — "the most recently
/// completed unit" — then the units merely started, latest first. Two pools
/// at most: the latest unit may close on a piece alone and hold no drill.
List<List<CourseListing>> _reviewPools(
  List<CourseListing> listings,
  CourseCompletionState completion,
) {
  final full = <List<CourseListing>>[];
  final partial = <List<CourseListing>>[];
  for (final lessons in _units(listings).values) {
    final done = lessons.where((l) => completion.isDone(l.id)).toList();
    if (done.isEmpty) continue;
    (done.length == lessons.length ? full : partial).add(done);
  }
  return [...full.reversed, ...partial.reversed].take(2).toList();
}

int _pick(int count, String dayKey) => dayOrdinal(dayKey) % count;

/// Today's ledger as it applies to [dayKey]: yesterday's is nothing.
TodayLedger _effective(TodayLedger ledger, String dayKey) =>
    ledger.localDay == dayKey ? ledger : const TodayLedger();

/// The lesson item's course, before any manifest is known: the lesson done
/// today if one is, else the first course neither completed nor passed.
CourseListing? _lessonOf(
  List<CourseListing> listings,
  CourseCompletionState completion,
  TodayLedger today,
) {
  final byId = {for (final l in listings) l.id: l};
  for (final id in today.lessonIds) {
    final l = byId[id];
    if (l != null) return l;
  }
  return listings.where((l) => !completion.isDone(l.id)).firstOrNull;
}

/// Which course manifests [composeDailySession] wants loaded for [dayKey]:
/// the lesson's (its piece step), the passed courses' (a piece to replay), the
/// review pool's (a drill), and whatever today's ledger already names.
Set<String> dailySessionManifestsWanted({
  required List<CourseListing> listings,
  required CourseCompletionState completion,
  required TodayLedger ledger,
  required String dayKey,
}) {
  final today = _effective(ledger, dayKey);
  final byId = {for (final l in listings) l.id: l};
  final wanted = <String>{};
  final lesson = _lessonOf(listings, completion, today);
  if (lesson != null) wanted.add(lesson.id);
  wanted.addAll(today.pieceStepKeys.where(byId.containsKey));
  for (final key in today.reviewKeys) {
    final parsed = parseReviewKey(key);
    if (parsed != null && byId.containsKey(parsed.courseId)) {
      wanted.add(parsed.courseId);
    }
  }
  wanted.addAll(
    listings.where((l) => completion.isPassed(l.id)).map((l) => l.id),
  );
  for (final pool in _reviewPools(listings, completion)) {
    wanted.addAll(pool.map((l) => l.id));
  }
  return wanted;
}

/// Today's session from the learner's position (change:
/// add-music-daily-session, D1) — a pure function of existing state:
///
/// - **lesson**: the lesson completed today if any (shown done), else the
///   first course neither completed nor passed; none when the path is
///   finished;
/// - **piece**: the piece step run today if any (done), else the lesson's own
///   piece step when it has one not yet passed, else a passed piece to replay,
///   round-robin by day;
/// - **review**: the review finished today if any (done), else a drill drawn
///   from the latest unit with done lessons that has one, round-robin by day;
/// - **replay** (path finished only): the lesson replayed today if any, else a
///   lesson picked round-robin by day.
///
/// [manifests] holds whatever course manifests were available; an item whose
/// manifest is missing is still listed (by its lesson's title) and opens the
/// lesson instead.
DailySession composeDailySession({
  required List<CourseListing> listings,
  required CourseCompletionState completion,
  required Map<String, CourseManifest> manifests,
  required TodayLedger ledger,
  required DailyGoal goal,
  required String dayKey,
}) {
  if (listings.isEmpty) {
    return DailySession(items: const [], goal: goal, pathFinished: false);
  }
  final today = _effective(ledger, dayKey);
  final byId = {for (final l in listings) l.id: l};
  final pathFinished = listings.every((l) => completion.isDone(l.id));
  final items = <DailyItem>[];

  // Lesson.
  CourseListing? lesson;
  if (!pathFinished) {
    lesson = _lessonOf(listings, completion, today);
    if (lesson != null) {
      items.add(
        DailyItem(
          kind: DailyItemKind.lesson,
          courseId: lesson.id,
          key: lesson.id,
          title: lesson.title,
          subtitle: lesson.unitTitle,
          done: today.lessonIds.contains(lesson.id),
        ),
      );
    }
  }

  // Piece.
  DailyItem pieceItem(CourseListing l, {required bool done}) {
    final block = manifests[l.id]?.blocks.whereType<PieceBlock>().firstOrNull;
    return DailyItem(
      kind: DailyItemKind.piece,
      courseId: l.id,
      key: l.id,
      title: block != null && block.title.isNotEmpty ? block.title : l.title,
      subtitle: l.title,
      done: done,
      piece: block,
    );
  }

  final pieceToday = today.pieceStepKeys
      .map((id) => byId[id])
      .whereType<CourseListing>()
      .firstOrNull;
  if (pieceToday != null) {
    items.add(pieceItem(pieceToday, done: true));
  } else {
    final ownPiece = lesson == null
        ? null
        : manifests[lesson.id]?.blocks.whereType<PieceBlock>().firstOrNull;
    if (lesson != null && ownPiece != null && !completion.isPassed(lesson.id)) {
      items.add(pieceItem(lesson, done: false));
    } else {
      final replayable = [
        for (final l in listings)
          if (completion.isPassed(l.id) &&
              (manifests[l.id]?.blocks.whereType<PieceBlock>().isNotEmpty ??
                  false))
            l,
      ];
      if (replayable.isNotEmpty) {
        items.add(
          pieceItem(replayable[_pick(replayable.length, dayKey)], done: false),
        );
      }
    }
  }

  // Review.
  DailyItem reviewItem(
    CourseListing l,
    int index,
    CourseBlock? block, {
    required bool done,
  }) {
    final prompt = switch (block) {
      ReadPlayBlock(:final prompt) ||
      RhythmTapBlock(:final prompt) ||
      EarChoiceBlock(:final prompt) => prompt,
      _ => const <String, String>{},
    };
    return DailyItem(
      kind: DailyItemKind.review,
      courseId: l.id,
      key: reviewKeyOf(l.id, index),
      title: prompt,
      subtitle: l.title,
      done: done,
      block: block,
    );
  }

  final reviewToday = today.reviewKeys
      .map(parseReviewKey)
      .where((p) => p != null && byId.containsKey(p.courseId))
      .firstOrNull;
  if (reviewToday != null) {
    final l = byId[reviewToday.courseId]!;
    final blocks = manifests[l.id]?.blocks;
    final block = blocks != null && reviewToday.index < blocks.length
        ? blocks[reviewToday.index]
        : null;
    items.add(reviewItem(l, reviewToday.index, block, done: true));
  } else {
    // The latest unit with a drill to offer wins.
    for (final pool in _reviewPools(listings, completion)) {
      final candidates = <(CourseListing, int, CourseBlock)>[
        for (final l in pool)
          if (manifests[l.id] case final m?)
            for (var i = 0; i < m.blocks.length; i++)
              if (isDrillBlock(m.blocks[i])) (l, i, m.blocks[i]),
      ];
      if (candidates.isEmpty) continue;
      final (l, i, b) = candidates[_pick(candidates.length, dayKey)];
      items.add(reviewItem(l, i, b, done: false));
      break;
    }
  }

  // Replay suggestion, once the path is finished.
  if (pathFinished) {
    final replayedToday = today.lessonIds
        .map((id) => byId[id])
        .whereType<CourseListing>()
        .firstOrNull;
    final l = replayedToday ?? listings[_pick(listings.length, dayKey)];
    items.add(
      DailyItem(
        kind: DailyItemKind.replay,
        courseId: l.id,
        key: l.id,
        title: l.title,
        subtitle: l.unitTitle,
        done: replayedToday != null,
      ),
    );
  }

  return DailySession(items: items, goal: goal, pathFinished: pathFinished);
}
