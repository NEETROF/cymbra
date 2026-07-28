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

import '../services/play_sync_service.dart';
import 'play_activity.dart';

part 'play_activity_notifier.g.dart';

/// The per-day play activity for [userId] — the heatmap's data source (change:
/// add-play-activity-profile). Reads through the injectable [PlaySyncService]
/// seam, so the heatmap is driven by state that a test can override with a fake
/// aggregate (no native library, no live backend).
@riverpod
Future<PlayActivity> playActivity(Ref ref, String userId) =>
    ref.watch(playSyncServiceProvider).getPlayActivity(userId);
