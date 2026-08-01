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

import '../services/leaderboard_service.dart';
import 'leaderboard.dart';

part 'leaderboard_notifier.g.dart';

/// One board `(scoreId, mode)` — the leaderboard view's data source (change:
/// add-play-leaderboards). Reads through the injectable [LeaderboardService]
/// seam, so the view is driven by state a test can override with a fake board
/// (no native library, no live backend). Keyed by the piece and mode, so the
/// tempo/reaction toggle simply watches a different instance.
@riverpod
Future<Leaderboard> leaderboard(
  Ref ref,
  String scoreId,
  LeaderboardMode mode,
) => ref
    .watch(leaderboardServiceProvider)
    .getLeaderboard(scoreId: scoreId, mode: mode);
