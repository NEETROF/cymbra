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
import 'package:music/state/countdown.dart';

void main() {
  group('countdownLabel', () {
    test('starts at 3 and counts down one digit per second', () {
      expect(countdownLabel(kCountdownStartMs), '3');
      expect(countdownLabel(2701), '3');
      expect(countdownLabel(2700), '2');
      expect(countdownLabel(1701), '2');
      expect(countdownLabel(1700), '1');
    });

    test('shows GO for the final stretch, then nothing at zero', () {
      expect(countdownLabel(700), 'GO');
      expect(countdownLabel(1), 'GO');
      expect(countdownLabel(0), isNull);
      expect(countdownLabel(-10), isNull);
    });

    test('isCountdownGo marks only the final GO window', () {
      expect(isCountdownGo(kCountdownStartMs), isFalse);
      expect(isCountdownGo(700), isTrue);
      expect(isCountdownGo(0), isFalse);
    });
  });
}
