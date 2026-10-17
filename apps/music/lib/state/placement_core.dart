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

import 'dart:math' as math;

import 'package:flutter/foundation.dart';

import '../courses/course_manifest.dart';

/// The unit a listing belongs to on the path: its unit slug, or a per-level
/// fallback for a pre-unit catalogue — the one grouping rule, shared by the
/// path, the continue card and the daily session.
String unitSlugOf(CourseListing l) =>
    l.unit.isNotEmpty ? l.unit : 'level:${l.level}';

/// The path's units in catalogue order.
List<String> unitOrder(List<CourseListing> listings) {
  final seen = <String>{};
  return [
    for (final l in listings)
      if (seen.add(unitSlugOf(l))) unitSlugOf(l),
  ];
}

/// The units before [startUnit] — skipped by placement (change:
/// add-music-placement-test, D3). Empty without a start, or when the start is
/// not a unit of this path (the catalogue moved under the pointer).
Set<String> skippedUnitsOf(List<String> units, String? startUnit) {
  if (startUnit == null) return const {};
  final i = units.indexOf(startUnit);
  if (i <= 0) return const {};
  return units.take(i).toSet();
}

/// A unit's number for display, from its `u<n>` slug; its position otherwise.
int unitNumberOf(List<String> units, String unit) {
  final n = int.tryParse(unit.replaceFirst('u', ''));
  if (n != null && unit.startsWith('u')) return n;
  final i = units.indexOf(unit);
  return i < 0 ? 1 : i + 1;
}

/// What a finished placement run says (change: add-music-placement-test,
/// D2).
@immutable
class PlacementOutcome {
  const PlacementOutcome({
    required this.startUnit,
    required this.solved,
    required this.missedStep,
  });

  /// The proposed starting unit.
  final String startUnit;

  /// Exercises solved at first try.
  final int solved;

  /// The step index of the first miss, or null when everything was solved.
  final int? missedStep;

  bool get allSolved => missedStep == null;
}

/// The starting unit from a placement run: the unit of the first probing step
/// not solved first try — a skipped step is a miss, a piece counts as solved
/// when passed — or, when every probe was solved, the unit after the last one
/// probed (the last unit itself when nothing follows). Null when no step
/// probed any unit.
///
/// [stepUnits] is the probed unit of each step the player showed (null for an
/// untagged step), [flawless] the player's per-step first-try results (a step
/// absent from it was skipped), [units] the path's units in order.
PlacementOutcome? computePlacement({
  required List<String?> stepUnits,
  required Map<int, bool> flawless,
  required List<String> units,
}) {
  var solved = 0;
  int? missedStep;
  String? missedUnit;
  String? lastProbed;
  for (var i = 0; i < stepUnits.length; i++) {
    final unit = stepUnits[i];
    if (unit == null) continue;
    lastProbed = unit;
    if (flawless[i] == true) {
      solved++;
    } else if (missedStep == null) {
      missedStep = i;
      missedUnit = unit;
    }
  }
  if (lastProbed == null) return null;
  if (missedUnit != null) {
    return PlacementOutcome(
      startUnit: missedUnit,
      solved: solved,
      missedStep: missedStep,
    );
  }
  final i = units.indexOf(lastProbed);
  final next = i < 0 ? lastProbed : units[math.min(i + 1, units.length - 1)];
  return PlacementOutcome(startUnit: next, solved: solved, missedStep: null);
}

/// The prompt a probing step shows — what the result names as "missed at".
InlineText promptOf(CourseBlock block) => switch (block) {
  ReadPlayBlock(:final prompt) ||
  NameNoteBlock(:final prompt) ||
  PlaceNoteBlock(:final prompt) ||
  RhythmTapBlock(:final prompt) ||
  EarChoiceBlock(:final prompt) ||
  BuildChordBlock(:final prompt) ||
  QuestionBlock(:final prompt) => prompt,
  PieceBlock(:final title) => title,
  _ => const <String, String>{},
};
