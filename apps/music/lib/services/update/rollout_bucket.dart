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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../preferences_service.dart';

part 'rollout_bucket.g.dart';

/// Where the install's rollout bucket is remembered.
const kRolloutBucketPrefKey = 'desktop_update_rollout_bucket';

/// The install's staged-rollout bucket (change: add-desktop-auto-update,
/// design D3).
///
/// Drawn **once** and persisted, then compared locally against the manifest's
/// `rollout_percent`. Drawing once is the point: a bucket redrawn on every check
/// would make each launch an independent coin flip, so a 10 % rollout would
/// eventually reach everybody instead of ten percent of installs.
///
/// The bucket never leaves the device — the update check sends no identifier at
/// all, which is what keeps the feed one cacheable document and unusable for
/// counting installs. A client that ignored its bucket would only get a
/// legitimately signed update early, so there is nothing here to enforce.
class RolloutBucket {
  RolloutBucket(this._prefs, {Random? random})
    : _random = random ?? Random.secure();

  final PreferencesService _prefs;
  final Random _random;

  /// This install's bucket in `0..99`, drawing and persisting one on first use.
  Future<int> bucket() async {
    final stored = await _prefs.getString(kRolloutBucketPrefKey);
    final parsed = stored == null ? null : int.tryParse(stored);
    if (parsed != null && parsed >= 0 && parsed < 100) return parsed;
    final drawn = _random.nextInt(100);
    await _prefs.setString(kRolloutBucketPrefKey, '$drawn');
    return drawn;
  }

  /// Whether this install is inside a `rolloutPercent` rollout.
  ///
  /// `0` is the kill-switch and includes nobody; `100` includes everybody.
  Future<bool> isIncluded(int rolloutPercent) async {
    if (rolloutPercent <= 0) return false;
    if (rolloutPercent >= 100) return true;
    return await bucket() < rolloutPercent;
  }
}

/// The rollout bucket, behind a provider so tests inject a seeded [Random] and a
/// fake preferences store.
@Riverpod(keepAlive: true)
RolloutBucket rolloutBucket(Ref ref) =>
    RolloutBucket(ref.watch(preferencesServiceProvider));
