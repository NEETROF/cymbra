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
import 'package:music/painters/staff_painter.dart';

void main() {
  group('StaffPainter.staffLineGap', () {
    test('keeps the proportional band on comfortable heights', () {
      // Tall enough that the 18 px ceiling binds before the fit cap.
      expect(StaffPainter.staffLineGap(height: 400, twoStaff: true), 18.0);
      expect(StaffPainter.staffLineGap(height: 250, twoStaff: false), 18.0);
    });

    test('caps the gap so a short window cannot collide the two staves', () {
      // 120 px used to keep the 8 px floor (needs ~154 px) and overlap; the
      // fit cap now shrinks the gap to fit: 120/19.2 = 6.25.
      final gap = StaffPainter.staffLineGap(height: 120, twoStaff: true);
      expect(gap, closeTo(120 / 19.2, 0.001));
      // The grand-staff budget fits the height with air between the hands.
      expect(19.2 * gap, lessThanOrEqualTo(120.0001));
      // A lone staff also fits its budget on a short height (no clipped stems).
      final solo = StaffPainter.staffLineGap(height: 120, twoStaff: false);
      expect(solo, closeTo(120 / 12.0, 0.001));
    });

    test('never shrinks below the 3 px readability floor', () {
      expect(StaffPainter.staffLineGap(height: 40, twoStaff: true), 3.0);
    });

    test('noteScale applies after the clamps (in-card preview)', () {
      final base = StaffPainter.staffLineGap(height: 300, twoStaff: true);
      expect(
        StaffPainter.staffLineGap(height: 300, twoStaff: true, noteScale: 0.5),
        closeTo(base * 0.5, 0.001),
      );
    });
  });
}
