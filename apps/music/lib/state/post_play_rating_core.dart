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

/// Pure decision logic for the post-play rating prompt (change:
/// add-post-play-rating-prompt). No Flutter, no Riverpod, no I/O — every rule the
/// prompt obeys is a function of plain values here, so the whole gate is
/// host-testable and the notifier stays a thin shell around it.
library;

import 'player_data.dart';

/// The share of the piece's notes the playhead must have passed before the prompt
/// is offered on an early exit (25%).
///
/// Counted in **notes**, not in elapsed time or position: time is tempo-dependent
/// (practising at half speed would double it for the same music) and a position
/// fraction over-counts a piece that opens on long held notes or rests. A note
/// count measures how much music actually went by.
const double kRatingPromptMinPlayedFraction = 0.25;

/// How many offered scores the device remembers. The memory only exists to stop a
/// second prompt for the same piece, so an old entry falling off is harmless — the
/// cap just keeps the persisted value bounded.
const int kRatingPromptMemoryMax = 200;

/// What is known about the caller's existing rating of the score being played.
enum RatedState {
  /// Confirmed un-rated by the server → the prompt may be offered.
  notRated,

  /// Already rated (on this or any other device) → never prompt again.
  rated,

  /// Not resolved: still loading, offline, or the read failed. Fail-closed — an
  /// unknown state suppresses the prompt rather than risking a doomed submission
  /// or a second prompt for a score the user already rated elsewhere.
  unknown,
}

/// The share (0..1) of [notes] the playhead has passed, given the furthest point
/// reached in milliseconds.
///
/// [notes] is the WHOLE score (`PlayerData.notes`), never the selected hand's
/// subset: muting a hand must not make a run look twice as complete as it is.
/// The list is already sorted by [TimedNote.startMs], so this is a binary search
/// for the first note starting after [furthestElapsedMs]. Each note of a chord
/// counts individually — that is what the list holds, and a chordal texture still
/// crosses the threshold at roughly the same point in the music.
double playedNoteFraction(List<TimedNote> notes, double furthestElapsedMs) {
  if (notes.isEmpty) return 0;
  // Count of notes with startMs <= furthestElapsedMs, via lower-bound search on
  // the first note starting strictly after the playhead's high-water mark.
  var low = 0;
  var high = notes.length;
  while (low < high) {
    final mid = (low + high) >> 1;
    if (notes[mid].startMs <= furthestElapsedMs) {
      low = mid + 1;
    } else {
      high = mid;
    }
  }
  return low / notes.length;
}

/// Whether to offer the rating prompt for the score just played.
///
/// Every term must hold, and an unresolved [rated] suppresses. [catalogId] is null
/// for a bundled or user-contributed score — neither is in the public catalog, so
/// neither is rateable. [reachedEnd] short-circuits the playback term: finishing a
/// scored run is engagement whatever the note count says (a range-practice loop
/// that ends early does not set it).
bool shouldPromptRating({
  required bool signedIn,
  required String? catalogId,
  required RatedState rated,
  required bool alreadyOffered,
  required double playedFraction,
  required bool reachedEnd,
}) {
  if (!signedIn) return false;
  if (catalogId == null) return false;
  if (rated != RatedState.notRated) return false;
  if (alreadyOffered) return false;
  return reachedEnd || playedFraction >= kRatingPromptMinPlayedFraction;
}

/// [offered] with [catalogId] appended as the most recent entry, de-duplicated and
/// trimmed to [max] (oldest first out).
///
/// Returns a new list; the input is not mutated. Re-offering an id already present
/// moves it to the end rather than duplicating it, so the cap counts distinct
/// scores.
List<String> rememberOffered(
  List<String> offered,
  String catalogId, {
  int max = kRatingPromptMemoryMax,
}) {
  final next = [
    for (final id in offered)
      if (id != catalogId) id,
    catalogId,
  ];
  if (next.length <= max) return next;
  return next.sublist(next.length - max);
}
