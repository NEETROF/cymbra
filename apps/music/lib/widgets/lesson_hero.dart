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

/// Shared-element tags (change: add-music-learn-play-home): the card or node
/// the player taps and the destination's title carry the same tag, so the
/// title flies into place. One surface MUST carry a given tag at most once —
/// the Learn door puts the lesson tag on the continue card and NOT on the
/// path nodes.
library;

/// The tag of a lesson's title, by course id.
String lessonHeroTag(String courseId) => 'lesson-hero-$courseId';

/// The tag of a score's title, by catalog entry id.
String scoreHeroTag(String entryId) => 'score-hero-$entryId';
