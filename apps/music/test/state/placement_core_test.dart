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
import 'package:music/state/placement_core.dart';

import '../support/daily_fixtures.dart';

const _units = ['u1', 'u2', 'u3', 'u4'];

void main() {
  group('computePlacement', () {
    test('the first probe not solved first try names the starting unit', () {
      final o = computePlacement(
        stepUnits: [null, 'u1', 'u2', 'u3', 'u4', null],
        flawless: {1: true, 2: true, 3: false, 4: true},
        units: _units,
      )!;
      expect(o.startUnit, 'u3');
      expect(o.solved, 3);
      expect(o.missedStep, 3);
      expect(o.allSolved, isFalse);
    });

    test('a skipped probe is a miss', () {
      final o = computePlacement(
        stepUnits: ['u1', 'u2', 'u3'],
        flawless: {0: true, 2: true}, // step 1 never answered
        units: _units,
      )!;
      expect(o.startUnit, 'u2');
      expect(o.missedStep, 1);
    });

    test('everything solved starts after the last unit probed', () {
      final o = computePlacement(
        stepUnits: ['u1', 'u2'],
        flawless: {0: true, 1: true},
        units: _units,
      )!;
      expect(o.startUnit, 'u3');
      expect(o.allSolved, isTrue);
      expect(o.solved, 2);
    });

    test('everything solved up to the last unit stays on the last unit', () {
      final o = computePlacement(
        stepUnits: ['u1', 'u4'],
        flawless: {0: true, 1: true},
        units: _units,
      )!;
      expect(o.startUnit, 'u4');
    });

    test('a piece step counts as solved when passed, missed otherwise', () {
      expect(
        computePlacement(
          stepUnits: ['u1', 'u4'],
          flawless: {0: true, 1: false},
          units: _units,
        )!.startUnit,
        'u4',
      );
      expect(
        computePlacement(
          stepUnits: ['u1', 'u4'],
          flawless: {0: true, 1: true},
          units: _units,
        )!.startUnit,
        'u4',
      );
    });

    test('nothing probed → no outcome', () {
      expect(
        computePlacement(stepUnits: [null, null], flawless: {}, units: _units),
        isNull,
      );
    });
  });

  group('units and skipping', () {
    test('the unit order follows the catalogue', () {
      expect(unitOrder(dailyListings), ['u1', 'u2', 'u3']);
      expect(unitSlugOf(dailyListings.first), 'u1');
      expect(
        unitSlugOf(
          const CourseListing(id: 'x', schemaVersion: 2, level: 'advanced'),
        ),
        'level:advanced',
      );
    });

    test('units before the start are skipped; none without a start', () {
      expect(skippedUnitsOf(_units, 'u3'), {'u1', 'u2'});
      expect(skippedUnitsOf(_units, 'u1'), isEmpty);
      expect(skippedUnitsOf(_units, null), isEmpty);
      expect(skippedUnitsOf(_units, 'u9'), isEmpty, reason: 'moved catalogue');
    });

    test('a unit number reads off its slug, else its position', () {
      expect(unitNumberOf(_units, 'u3'), 3);
      expect(unitNumberOf(['level:beginner', 'x'], 'x'), 2);
    });

    test('the prompt of a probe is what the result names', () {
      final m = dailyManifests(['a1', 'a3']);
      expect(promptOf(m['a1']!.blocks[1]), {'en': 'Play the G'});
      expect(promptOf(m['a3']!.blocks[1]), {'en': 'Piece of a3'});
      expect(promptOf(m['a1']!.blocks[0]), isEmpty);
    });
  });
}
