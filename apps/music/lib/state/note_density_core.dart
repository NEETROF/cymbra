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

/// How tight the scrolling Portée is allowed to engrave, derived from the
/// score's own note density.
///
/// The Portée maps time to pixels through a **constant** window
/// (`StaffPainter.lookAheadMs`): the width right of the playhead always shows
/// the same number of milliseconds. That is what keeps the scroll speed
/// constant — and therefore readable — but it means the *visual* density is
/// whatever the piece happens to be. A slow 4/2 hymn at ♩=108 spends 4.4 s per
/// measure and fills the window with a single bar; Für Elise in 3/8 at ♩=120
/// spends 750 ms per measure and packs five of them, sixteenths and all, into
/// the same width.
///
/// So the window is capped by the *tightest note spacing the piece actually
/// contains*, not by its metre: [onsetGapMs] measures the score, and
/// [densityCappedLookAheadMs] narrows the window until that spacing clears the
/// engraver's readability floor. The cap is a function of the score alone, so
/// it is computed once and the scroll stays linear for the whole run — a
/// window recomputed per frame would rubber-band the notation in and out as
/// dense and sparse passages went by.
library;

import 'dart:math' as math;

import 'player_data.dart';

/// Floor the look-ahead may never narrow below, whatever the density.
///
/// The Portée is played from, not just read: below roughly a second and a half
/// of anticipation there is no time to place a hand, and a perfectly spaced
/// window nobody can play in is worse than a cramped one. When this floor binds
/// the [kMinColumnSpaces] guarantee is deliberately given up.
const double kMinLookAheadMs = 1500;

/// Fraction of the tightest onset gaps ignored when measuring a score.
///
/// Grace notes, turns and other ornaments produce a handful of gaps far shorter
/// than anything else in the piece; sizing the whole score on them would zoom
/// every page to fit one figure. Dropping the tightest 5% keeps the measurement
/// on the piece's ordinary fastest motion.
const double kOnsetGapPercentile = 0.05;

/// The score's characteristic tightest onset spacing, in **score** ms — the
/// [kOnsetGapPercentile]-th percentile of the gaps between consecutive distinct
/// onsets.
///
/// Gaps are measured **per staff** and then pooled: the treble and the bass are
/// engraved on separate systems, so a left-hand note landing 30 ms after a
/// right-hand one costs no horizontal room and must not count as a tight gap.
/// Notes sharing an onset (a chord) are one column, hence the distinct onsets.
///
/// Independent of the hand filter and of the playback speed: it is a property
/// of the engraving, and muting a hand or slowing down must not rescale the
/// notation. Returns `null` when the score has nothing to measure (fewer than
/// two distinct onsets on every staff) — the caller then leaves the window
/// alone.
double? onsetGapMs(
  List<TimedNote> notes, {
  double percentile = kOnsetGapPercentile,
}) {
  if (notes.length < 2) return null;
  final byStaff = <int, Set<int>>{};
  for (final n in notes) {
    (byStaff[n.staff] ??= <int>{}).add(n.startMs);
  }
  final gaps = <int>[];
  for (final onsets in byStaff.values) {
    if (onsets.length < 2) continue;
    final sorted = onsets.toList()..sort();
    for (var i = 1; i < sorted.length; i++) {
      final gap = sorted[i] - sorted[i - 1];
      if (gap > 0) gaps.add(gap);
    }
  }
  if (gaps.isEmpty) return null;
  gaps.sort();
  final index = ((gaps.length - 1) * percentile.clamp(0.0, 1.0)).round();
  return gaps[index].toDouble();
}

/// [requestedMs] narrowed until the score's tightest spacing ([gapMs], from
/// [onsetGapMs]) is engraved at least [minSpaces] staff spaces wide.
///
/// [minSpaces] is the caller's glyph budget (`StaffPainter.minOnsetSpaces`) —
/// this module knows about time and density, the painter knows how wide its
/// notation draws. [trackPx] is the width right of the playhead the window is
/// spread over and [lineGap] the staff line gap everything is drawn from, so
/// the result follows the viewport and the score-size setting. The window is
/// only ever **narrowed** — a score already sparse enough keeps the caller's
/// window untouched — and never past [floorMs] (see [kMinLookAheadMs]), nor
/// past [requestedMs] when that is already the tighter of the two.
double densityCappedLookAheadMs({
  required double requestedMs,
  required double trackPx,
  required double lineGap,
  required double minSpaces,
  double? gapMs,
  double floorMs = kMinLookAheadMs,
}) {
  if (gapMs == null || gapMs <= 0) return requestedMs;
  if (trackPx <= 0 || lineGap <= 0 || minSpaces <= 0) return requestedMs;
  final needed = trackPx * gapMs / (minSpaces * lineGap);
  if (needed >= requestedMs) return requestedMs;
  return math.max(needed, math.min(requestedMs, floorMs));
}

/// One-entry memo over [onsetGapMs], keyed on the **identity** of the note
/// list.
///
/// The player rebuilds every frame while the playhead moves, but `copyWith`
/// carries the same immutable `notes` list across those rebuilds, so this hits
/// on every frame after the first of a score and the measurement is paid once
/// per load instead of once per frame.
double? cachedOnsetGapMs(List<TimedNote> notes) {
  if (identical(notes, _memoNotes)) return _memoGap;
  final gap = onsetGapMs(notes);
  _memoNotes = notes;
  _memoGap = gap;
  return gap;
}

List<TimedNote>? _memoNotes;
double? _memoGap;
