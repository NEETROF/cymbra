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

/// Star thresholds on the best overall synchronisation (change:
/// add-music-piece-stars, D1): one star from 60 %, two from 80 % — the piece
/// step's default pass threshold, so "passed" and "two stars" are one thing —
/// three from 95 %. App constants, stated in the spec.
const List<int> kStarThresholds = [60, 80, 95];

/// Stars for a best overall synchronisation, 0 to 3; 0 for no scored run.
int starsFor(double? bestPct) {
  if (bestPct == null) return 0;
  var stars = 0;
  for (final t in kStarThresholds) {
    if (bestPct >= t) stars++;
  }
  return stars;
}
