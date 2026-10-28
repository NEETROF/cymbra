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

import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/update/rollout_bucket.dart';

import '../../support/prefs_fakes.dart';

/// A `Random` that hands out a fixed sequence, so "drawn once" is observable.
class _ScriptedRandom implements Random {
  _ScriptedRandom(this._values);
  final List<int> _values;
  int _i = 0;

  @override
  int nextInt(int max) => _values[_i++ % _values.length];

  @override
  bool nextBool() => throw UnimplementedError();
  @override
  double nextDouble() => throw UnimplementedError();
}

void main() {
  test('draws a bucket once and persists it', () async {
    final prefs = FakePreferencesService();
    final bucket = RolloutBucket(prefs, random: _ScriptedRandom([7, 88]));

    expect(await bucket.bucket(), 7);
    expect(prefs.store[kRolloutBucketPrefKey], '7');
    // Redrawing every check would turn a 10 % rollout into an eventual 100 %.
    expect(await bucket.bucket(), 7);
  });

  test('reuses a bucket from a previous launch', () async {
    final prefs = FakePreferencesService({kRolloutBucketPrefKey: '42'});
    final bucket = RolloutBucket(prefs, random: _ScriptedRandom([7]));
    expect(await bucket.bucket(), 42);
  });

  test('redraws when the stored value is corrupt or out of range', () async {
    for (final stored in ['', 'nope', '-1', '100', '1000']) {
      final prefs = FakePreferencesService({kRolloutBucketPrefKey: stored});
      final bucket = RolloutBucket(prefs, random: _ScriptedRandom([13]));
      expect(await bucket.bucket(), 13, reason: 'stored "$stored"');
    }
  });

  group('isIncluded', () {
    test('0 is the kill-switch and includes nobody', () async {
      final bucket = RolloutBucket(
        FakePreferencesService({kRolloutBucketPrefKey: '0'}),
        random: _ScriptedRandom([0]),
      );
      expect(await bucket.isIncluded(0), isFalse);
      expect(await bucket.isIncluded(-5), isFalse);
    });

    test('100 includes everybody, even the top bucket', () async {
      final bucket = RolloutBucket(
        FakePreferencesService({kRolloutBucketPrefKey: '99'}),
        random: _ScriptedRandom([99]),
      );
      expect(await bucket.isIncluded(100), isTrue);
    });

    test(
      'the boundary is exclusive: bucket 25 is outside a 25 % rollout',
      () async {
        final at = RolloutBucket(
          FakePreferencesService({kRolloutBucketPrefKey: '25'}),
          random: _ScriptedRandom([25]),
        );
        final below = RolloutBucket(
          FakePreferencesService({kRolloutBucketPrefKey: '24'}),
          random: _ScriptedRandom([24]),
        );
        expect(await at.isIncluded(25), isFalse);
        expect(await below.isIncluded(25), isTrue);
      },
    );
  });
}
