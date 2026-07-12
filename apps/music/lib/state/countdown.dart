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

/// Pre-start countdown timing (pure, host-testable).
///
/// A race-game style 5…4…3…2…1…GO shown before playback begins so the player has
/// time to get ready. The digits run for [_digitsMs] (one second each) and GO
/// holds for [_goMs]; the total is [kCountdownStartMs]. GO ends exactly as the
/// countdown reaches 0, so it disappears before the first note.
const double _goMs = 700;
const double _digitsMs = 5000; // 5 → 1, one second each
const double kCountdownStartMs = _digitsMs + _goMs;

/// The label to show for a countdown with [remainingMs] left: `"5"`…`"1"`,
/// `"GO"`, or null when the countdown is over (nothing to show).
String? countdownLabel(double remainingMs) {
  if (remainingMs <= 0) return null;
  if (remainingMs <= _goMs) return 'GO';
  return ((remainingMs - _goMs) / 1000).ceil().toString();
}

/// Whether [remainingMs] represents the final GO step (used to style it).
bool isCountdownGo(double remainingMs) =>
    remainingMs > 0 && remainingMs <= _goMs;
