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
import 'package:music/state/piece_stars_core.dart';

void main() {
  group('starsFor (change: add-music-piece-stars)', () {
    test('the three thresholds, each edge inclusive', () {
      expect(starsFor(59), 0);
      expect(starsFor(60), 1);
      expect(starsFor(79), 1);
      expect(starsFor(80), 2);
      expect(starsFor(94), 2);
      expect(starsFor(95), 3);
      expect(starsFor(100), 3);
      expect(starsFor(0), 0);
    });

    test('no scored run → no star', () {
      expect(starsFor(null), 0);
    });

    test('two stars is the piece step\'s default pass', () {
      expect(kStarThresholds[1], 80);
    });
  });
}
