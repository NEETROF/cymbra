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

import 'package:cymbra_flags/cymbra_flags.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/learn_door_access.dart';

FlagSnapshot _with(Map<String, FlagEntry> entries) =>
    FlagSnapshot(app: 'music', identity: 'u1', version: 'v1', entries: entries);

void main() {
  // change: add-music-learn-play-home — `music.learn_door.enabled` is a
  // kill-switch: absent means on, only an explicit false folds the home.
  group('learnDoorEnabledIn', () {
    test('shows the two doors before the first flag fetch', () {
      expect(learnDoorEnabledIn(FlagSnapshot.empty('music', null)), isTrue);
    });

    test('follows the server when it sends the flag', () {
      expect(
        learnDoorEnabledIn(
          _with({kLearnDoorEnabledFlag: const FlagEntry(FlagKind.bool_, true)}),
        ),
        isTrue,
      );
      expect(
        learnDoorEnabledIn(
          _with({
            kLearnDoorEnabledFlag: const FlagEntry(FlagKind.bool_, false),
          }),
        ),
        isFalse,
        reason: 'the kill-switch folds the home back to one body',
      );
    });

    test('ignores a value of the wrong type', () {
      expect(
        learnDoorEnabledIn(
          _with({
            kLearnDoorEnabledFlag: const FlagEntry(FlagKind.string, 'off'),
          }),
        ),
        isTrue,
      );
    });
  });
}
