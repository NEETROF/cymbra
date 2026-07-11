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

part 'clock_service.g.dart';

/// Injectable monotonic wall-clock seam.
///
/// The scorer needs real elapsed time for Wait-Mode **reaction** timing (where
/// the score playhead is frozen at the gate) and for the finish timestamp. State
/// depends on this interface rather than `DateTime`/`Stopwatch` directly so tests
/// drive time deterministically with a fake — no flaky sleeps.
abstract class Clock {
  /// Monotonic milliseconds since an arbitrary epoch (only differences matter).
  int nowMs();
}

/// Production [Clock] backed by a monotonic [Stopwatch].
class SystemClock implements Clock {
  final Stopwatch _sw = Stopwatch()..start();

  @override
  int nowMs() => _sw.elapsedMilliseconds;
}

@Riverpod(keepAlive: true)
Clock clock(Ref ref) => SystemClock();
