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

import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'haptics_service.g.dart';

/// Seam over the device's haptics (change: add-music-motion-and-celebrations).
/// Called on a pass or a completion, never on a miss. The platform honours its
/// own system haptics setting underneath (Android's haptic-feedback toggle,
/// iOS's System Haptics), so the app does not read it.
abstract class HapticsService {
  /// One light tap.
  Future<void> light();
}

/// Production haptics over Flutter's [HapticFeedback]; a no-op where the
/// platform has no haptics (desktop), which the plugin handles itself.
class SystemHapticsService implements HapticsService {
  const SystemHapticsService();

  @override
  Future<void> light() => HapticFeedback.lightImpact();
}

/// Production haptics provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
HapticsService hapticsService(Ref ref) => const SystemHapticsService();
