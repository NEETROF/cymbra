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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/state/note_density_core.dart';
import 'package:music/state/player_data.dart';

/// A run of [count] onsets [stepMs] apart on [staff], starting at [fromMs].
List<TimedNote> _run(
  int count, {
  required int stepMs,
  int fromMs = 0,
  int staff = 1,
}) => List.generate(
  count,
  (i) => TimedNote(
    pitch: 60,
    startMs: fromMs + i * stepMs,
    durationMs: stepMs,
    staff: staff,
  ),
);

void main() {
  group('onsetGapMs', () {
    test('measures the score\'s regular fastest motion', () {
      // Sixteenths at ♩=120 → 125 ms apart.
      expect(onsetGapMs(_run(40, stepMs: 125)), 125.0);
    });

    test('nothing to measure yields null', () {
      expect(onsetGapMs(const []), isNull);
      expect(
        onsetGapMs(const [TimedNote(pitch: 60, startMs: 0, durationMs: 500)]),
        isNull,
      );
      // Two notes, one onset: a chord is a single column, so no gap.
      expect(
        onsetGapMs(const [
          TimedNote(pitch: 60, startMs: 0, durationMs: 500),
          TimedNote(pitch: 64, startMs: 0, durationMs: 500),
        ]),
        isNull,
      );
    });

    test('a chord counts as one column, not a zero gap', () {
      final notes = [
        ..._run(20, stepMs: 250),
        // Thicken every onset into a triad — the spacing is unchanged.
        ..._run(20, stepMs: 250).map(
          (n) => TimedNote(
            pitch: n.pitch + 4,
            startMs: n.startMs,
            durationMs: n.durationMs,
          ),
        ),
      ];
      expect(onsetGapMs(notes), 250.0);
    });

    test('the two hands are measured apart, not against each other', () {
      // Quarter notes in each hand, the left offset by a sixteenth. The hands
      // are engraved on separate staves, so that 125 ms offset costs no
      // horizontal room and must not read as a tight gap.
      final notes = [
        ..._run(20, stepMs: 500),
        ..._run(20, stepMs: 500, fromMs: 125, staff: 2),
      ];
      expect(onsetGapMs(notes), 500.0);
    });

    test('an ornament does not size the whole piece', () {
      // A long run of sixteenths with one 4-note turn in 32nds: the turn is
      // under the 5% percentile, so the piece still measures as sixteenths.
      final notes = [
        ..._run(60, stepMs: 125),
        ..._run(4, stepMs: 62, fromMs: 20000),
      ];
      expect(onsetGapMs(notes), 125.0);
    });

    test('sustained fast motion does size the piece', () {
      // Same 32nds, but now they are half the piece — no longer an outlier.
      final notes = [
        ..._run(30, stepMs: 125),
        ..._run(30, stepMs: 62, fromMs: 20000),
      ];
      expect(onsetGapMs(notes), 62.0);
    });

    test('is independent of the note order', () {
      final ordered = _run(20, stepMs: 125);
      final shuffled = ordered.reversed.toList();
      expect(onsetGapMs(shuffled), onsetGapMs(ordered));
    });
  });

  group('densityCappedLookAheadMs', () {
    // A typical landscape viewport: 702 px right of the playhead, engraved at
    // the Partition's 12 px staff space.
    const trackPx = 702.0;
    const lineGap = 12.0;
    const minSpaces = StaffPainter.minOnsetSpaces;

    double cap(double gapMs, {double requestedMs = 4000}) =>
        densityCappedLookAheadMs(
          requestedMs: requestedMs,
          trackPx: trackPx,
          lineGap: lineGap,
          minSpaces: minSpaces,
          gapMs: gapMs,
        );

    test('a sparse score keeps the caller\'s window untouched', () {
      // Hymn 135: 4/2 at ♩=108 — one measure is 4444 ms, and its quarter-note
      // motion is already far wider than the readability floor.
      expect(cap(1000), 4000.0);
    });

    test('an unmeasurable score keeps the caller\'s window untouched', () {
      expect(
        densityCappedLookAheadMs(
          requestedMs: 4000,
          trackPx: trackPx,
          lineGap: lineGap,
          minSpaces: minSpaces,
        ),
        4000.0,
      );
    });

    test('a dense score is narrowed until its glyphs fit', () {
      // Für Elise: 3/8 at ♩=120, sixteenth motion (125 ms). At the full 4 s
      // window a sixteenth advances 702·125/4000 = 21.9 px, well under the
      // 2.68·12 = 32.2 px the notation needs — so seven measures were crammed
      // into a width that holds about four.
      final window = cap(125);
      expect(window, lessThan(4000));
      expect(window, closeTo(2728.7, 0.5));
      final pxPerMs = trackPx / window;
      expect(pxPerMs * 125, closeTo(minSpaces * lineGap, 0.001));
      // ~3.6 measures of 750 ms instead of 5.3 — the same notes, readable.
      expect(window / 750, closeTo(3.6, 0.1));
    });

    test('the resulting spacing clears the glyph budget', () {
      for (final gapMs in [125.0, 250.0, 375.0, 1000.0]) {
        final pxPerMs = trackPx / cap(gapMs);
        expect(
          pxPerMs * gapMs,
          greaterThanOrEqualTo(minSpaces * lineGap - 0.001),
          reason: 'a gap of $gapMs ms engraves too tight',
        );
      }
    });

    test('never narrows past the playable floor', () {
      // 32nds at ♩=120 (62.5 ms) would want ~1360 ms; the floor keeps enough
      // anticipation to place a hand, at the cost of the spacing guarantee.
      expect(cap(62.5), kMinLookAheadMs);
      // Pathologically dense input cannot collapse the window either.
      expect(cap(1), kMinLookAheadMs);
    });

    test('only ever narrows — a small window is never widened', () {
      // The caller already asked for less than the floor (a large score size on
      // a narrow viewport): the density cap must not push it back up.
      expect(cap(1, requestedMs: 900), 900.0);
      expect(cap(125, requestedMs: 900), 900.0);
    });

    test('follows the viewport and the score-size setting', () {
      // Twice the width at the same staff space fits twice the time.
      final narrow = densityCappedLookAheadMs(
        requestedMs: 4000,
        trackPx: 400,
        lineGap: lineGap,
        minSpaces: minSpaces,
        gapMs: 125,
      );
      final wide = densityCappedLookAheadMs(
        requestedMs: 4000,
        trackPx: 800,
        lineGap: lineGap,
        minSpaces: minSpaces,
        gapMs: 125,
      );
      expect(wide, closeTo(narrow * 2, 0.001));
      // Bigger notation (score size L) needs proportionally more room per gap,
      // so the window narrows further.
      final large = densityCappedLookAheadMs(
        requestedMs: 4000,
        trackPx: trackPx,
        lineGap: lineGap * 1.2,
        minSpaces: minSpaces,
        gapMs: 200,
      );
      final medium = densityCappedLookAheadMs(
        requestedMs: 4000,
        trackPx: trackPx,
        lineGap: lineGap,
        minSpaces: minSpaces,
        gapMs: 200,
      );
      expect(large, lessThan(medium));
    });

    test('a degenerate layout is left alone rather than dividing by zero', () {
      for (final args in [
        (trackPx: 0.0, lineGap: lineGap, minSpaces: minSpaces),
        (trackPx: trackPx, lineGap: 0.0, minSpaces: minSpaces),
        (trackPx: trackPx, lineGap: lineGap, minSpaces: 0.0),
      ]) {
        expect(
          densityCappedLookAheadMs(
            requestedMs: 4000,
            trackPx: args.trackPx,
            lineGap: args.lineGap,
            minSpaces: args.minSpaces,
            gapMs: 125,
          ),
          4000.0,
        );
      }
    });
  });

  group('medianMeasureMs', () {
    test('takes the typical measure, not the shortest', () {
      // A pickup bar, then full 750 ms measures. Sizing on the pickup would
      // shrink the window for the whole piece.
      expect(
        medianMeasureMs(const [0, 250, 1000, 1750, 2500], songEndMs: 3250),
        750,
      );
    });

    test('closes the last measure with the end of the piece', () {
      expect(medianMeasureMs(const [0, 1000], songEndMs: 2000), 1000);
    });

    test('a score with no measure table yields null', () {
      expect(medianMeasureMs(const [], songEndMs: 5000), isNull);
      expect(medianMeasureMs(const [0], songEndMs: 0), isNull);
    });

    test('survives an unsorted or duplicated table', () {
      expect(medianMeasureMs(const [1500, 0, 750, 750], songEndMs: 2250), 750);
    });
  });

  group('measure cap', () {
    const trackPx = 702.0;
    const lineGap = 12.0;
    const minSpaces = StaffPainter.minOnsetSpaces;

    double cap({double? gapMs, double? measureMs, double requestedMs = 4000}) =>
        densityCappedLookAheadMs(
          requestedMs: requestedMs,
          trackPx: trackPx,
          lineGap: lineGap,
          minSpaces: minSpaces,
          gapMs: gapMs,
          measureMs: measureMs,
        );

    test('bounds how many measures face the reader', () {
      // Für Elise: 3/8 at ♩=120 → a 750 ms measure. Spacing alone left ~4.3 on
      // screen; two is what a reader tracks.
      expect(cap(measureMs: 750), kMaxVisibleMeasures * 750);
      expect(cap(measureMs: 750) / 750, kMaxVisibleMeasures);
    });

    test('a slow score is untouched by the measure bound', () {
      // Hymn 135: 4/2 at ♩=108 → a 4444 ms measure, so two of them already
      // exceed the window.
      expect(cap(measureMs: 4444), 4000.0);
    });

    test('the tighter of the two bounds wins', () {
      // Dense but long measures → spacing governs.
      expect(cap(gapMs: 125, measureMs: 4000), closeTo(2728.7, 0.5));
      // Sparse but short measures → the measure count governs.
      expect(cap(gapMs: 1000, measureMs: 900), 1800.0);
      // Both bite: the tighter one is the answer.
      expect(cap(gapMs: 125, measureMs: 1000), 2000.0);
    });

    test('the playable floor still outranks both', () {
      // 2/4 at ♩=160 → a 750 ms… no: a 400 ms measure. Two of them is under the
      // floor, so anticipation wins and more than two measures show.
      expect(cap(measureMs: 400), kMinLookAheadMs);
      expect(cap(gapMs: 62.5, measureMs: 400), kMinLookAheadMs);
    });

    test('an unmeasured score is not capped by measures', () {
      expect(cap(), 4000.0);
      expect(cap(measureMs: 0), 4000.0);
    });

    test('each bound works without the other', () {
      expect(cap(gapMs: 125), closeTo(2728.7, 0.5));
      expect(cap(measureMs: 750), 1500.0);
    });
  });

  group('cachedOnsetGapMs', () {
    test('matches the uncached measurement', () {
      final notes = _run(40, stepMs: 125);
      expect(cachedOnsetGapMs(notes), onsetGapMs(notes));
    });

    test('re-measures when the score changes', () {
      expect(cachedOnsetGapMs(_run(40, stepMs: 125)), 125.0);
      expect(cachedOnsetGapMs(_run(40, stepMs: 250)), 250.0);
      // And back, so a stale memo would show up here too.
      expect(cachedOnsetGapMs(_run(40, stepMs: 125)), 125.0);
    });
  });

  group('cachedMedianMeasureMs', () {
    test('matches the uncached measurement', () {
      const starts = [0, 750, 1500];
      expect(
        cachedMedianMeasureMs(starts, songEndMs: 2250),
        medianMeasureMs(starts, songEndMs: 2250),
      );
    });

    test('re-measures when the table or the end moves', () {
      expect(cachedMedianMeasureMs(const [0, 750], songEndMs: 1500), 750);
      expect(cachedMedianMeasureMs(const [0, 1000], songEndMs: 2000), 1000);
      // A changed songEndMs alone must invalidate too — same list identity, so
      // a memo keyed on the table only would return the stale 750.
      final starts = [0, 750];
      expect(cachedMedianMeasureMs(starts, songEndMs: 1500), 750);
      expect(cachedMedianMeasureMs(starts, songEndMs: 2750), 2000);
    });
  });
}
