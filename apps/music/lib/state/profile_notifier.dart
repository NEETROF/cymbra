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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/profile_service.dart';

part 'profile_notifier.g.dart';

/// A player's public profile (change: add-play-activity-profile). For the owner
/// (viewing their own id) the server returns the full profile incl. visibility;
/// for another player it returns the allow-listed public fields, or fails
/// `notFound` when the target is private/ineligible. Read through the injectable
/// [ProfileService] seam so screens are testable without the backend.
@riverpod
Future<PlayerProfile> playerProfile(Ref ref, String userId) =>
    ref.watch(profileServiceProvider).getPlayerProfile(userId);

/// Drives the visibility control + neutral age gate (change: add-play-activity-
/// profile). The action's outcome is exposed as an [AsyncValue] the UI reacts to
/// (a success carries the new visibility; a failure carries the error, which the
/// listener maps to a friendly message — the raw error is never shown).
@riverpod
class ProfileVisibilityController extends _$ProfileVisibilityController {
  @override
  FutureOr<String?> build() => null;

  /// Set the caller's visibility. Going `public` sends the age-gate [dateOfBirth]
  /// once (used to derive eligibility server-side, then discarded). Fires the
  /// action; the UI reacts to the resulting state rather than awaiting a return.
  Future<void> setVisibility(String visibility, {DateTime? dateOfBirth}) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(
      () => ref
          .read(profileServiceProvider)
          .setProfileVisibility(visibility, dateOfBirth: dateOfBirth),
    );
  }
}
