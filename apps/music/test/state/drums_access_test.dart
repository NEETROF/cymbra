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
import 'package:music/state/drums_access.dart';
import 'package:flutter_test/flutter_test.dart';

FlagSnapshot _with(Map<String, FlagEntry> entries) =>
    FlagSnapshot(app: 'music', identity: 'u1', version: 'v1', entries: entries);

void main() {
  // change: graduate-drums-from-beta — the drums are open to every player and
  // `drums.enabled` is the kill-switch.
  group('drumsEnabledIn', () {
    test('shows the drums before the first flag fetch', () {
      expect(drumsEnabledIn(FlagSnapshot.empty('music', null)), isTrue);
    });

    test('follows the server when it sends the flag', () {
      expect(
        drumsEnabledIn(
          _with({kDrumsEnabledFlag: const FlagEntry(FlagKind.bool_, true)}),
        ),
        isTrue,
      );
      expect(
        drumsEnabledIn(
          _with({kDrumsEnabledFlag: const FlagEntry(FlagKind.bool_, false)}),
        ),
        isFalse,
        reason: 'the kill-switch hides the drums',
      );
    });

    test('ignores a value of the wrong type', () {
      expect(
        drumsEnabledIn(
          _with({kDrumsEnabledFlag: const FlagEntry(FlagKind.string, 'off')}),
        ),
        isTrue,
      );
    });
  });
}
