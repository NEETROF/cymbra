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
import 'package:music/state/player_data.dart';
import 'package:music/state/post_play_rating_core.dart';

/// `n` quarter notes, one every 500 ms, alternating staves so a test can prove the
/// fraction ignores hand selection.
List<TimedNote> _notes(int n) => [
  for (var i = 0; i < n; i++)
    TimedNote(
      pitch: 60 + (i % 12),
      startMs: i * 500,
      durationMs: 500,
      staff: i.isEven ? 1 : 2,
    ),
];

void main() {
  group('playedNoteFraction', () {
    test('is 0 for an empty score and before the first note', () {
      expect(playedNoteFraction(const [], 10_000), 0);
      // The first note starts at 0, so a playhead that never moved has passed it;
      // a negative high-water mark (never started) has passed nothing.
      expect(playedNoteFraction(_notes(4), -1), 0);
    });

    test('counts the notes the playhead has passed', () {
      final notes = _notes(4); // starts at 0, 500, 1000, 1500
      expect(playedNoteFraction(notes, 0), 0.25);
      expect(playedNoteFraction(notes, 999), 0.5);
      expect(playedNoteFraction(notes, 1500), 1.0);
      // Past the end stays at 1.0 (never above).
      expect(playedNoteFraction(notes, 999_999), 1.0);
    });

    test('a note exactly at the playhead counts as passed', () {
      final notes = _notes(4);
      expect(playedNoteFraction(notes, 500), 0.5);
    });

    test('counts each note of a chord individually', () {
      // Three notes sharing one startMs, then a fourth later.
      final chord = [
        const TimedNote(pitch: 60, startMs: 0, durationMs: 500, staff: 1),
        const TimedNote(pitch: 64, startMs: 0, durationMs: 500, staff: 1),
        const TimedNote(pitch: 67, startMs: 0, durationMs: 500, staff: 1),
        const TimedNote(pitch: 72, startMs: 500, durationMs: 500, staff: 1),
      ];
      expect(playedNoteFraction(chord, 0), 0.75);
      expect(playedNoteFraction(chord, 500), 1.0);
    });

    test('reads the whole score, not one hand', () {
      // A left hand dense at the opening, a right hand spread over the piece: the
      // fraction the caller must use (whole score) and the one a hand-filtered list
      // would give differ sharply, which is why the notifier passes `notes`, never
      // `visibleNotes`.
      const all = [
        TimedNote(pitch: 60, startMs: 0, durationMs: 250, staff: 1),
        TimedNote(pitch: 48, startMs: 0, durationMs: 250, staff: 2),
        TimedNote(pitch: 50, startMs: 250, durationMs: 250, staff: 2),
        TimedNote(pitch: 52, startMs: 500, durationMs: 250, staff: 2),
        TimedNote(pitch: 62, startMs: 1000, durationMs: 500, staff: 1),
        TimedNote(pitch: 64, startMs: 2000, durationMs: 500, staff: 1),
        TimedNote(pitch: 65, startMs: 3000, durationMs: 500, staff: 1),
      ];
      final rightHandOnly = all.where((n) => n.staff == 1).toList();
      // Whole score at 500 ms: 4 of 7 notes passed — well past the threshold.
      expect(playedNoteFraction(all, 500), closeTo(4 / 7, 1e-9));
      // The muted-hand view would report 1 of 4 — below it. Same music, different
      // verdict, which is the bug the whole-score rule prevents.
      expect(playedNoteFraction(rightHandOnly, 500), 0.25);
    });
  });

  group('shouldPromptRating', () {
    bool prompt({
      bool signedIn = true,
      String? catalogId = 'c1',
      RatedState rated = RatedState.notRated,
      bool declined = false,
      double playedFraction = 1.0,
      bool reachedEnd = false,
    }) => shouldPromptRating(
      signedIn: signedIn,
      catalogId: catalogId,
      rated: rated,
      declined: declined,
      playedFraction: playedFraction,
      reachedEnd: reachedEnd,
    );

    test('prompts when every term holds', () {
      expect(prompt(), isTrue);
    });

    test('each term individually suppresses', () {
      expect(prompt(signedIn: false), isFalse);
      expect(prompt(catalogId: null), isFalse); // bundled / contributed score
      expect(prompt(rated: RatedState.rated), isFalse);
      expect(prompt(declined: true), isFalse);
      expect(prompt(playedFraction: 0.24), isFalse);
    });

    test('an unknown rated state suppresses (fail-closed)', () {
      expect(prompt(rated: RatedState.unknown), isFalse);
    });

    test('the threshold is inclusive at 25%', () {
      expect(prompt(playedFraction: kRatingPromptMinPlayedFraction), isTrue);
      expect(
        prompt(playedFraction: kRatingPromptMinPlayedFraction - 0.001),
        isFalse,
      );
    });

    test('reaching the end satisfies the playback term on its own', () {
      expect(prompt(playedFraction: 0, reachedEnd: true), isTrue);
      // …but never overrides the other terms.
      expect(
        prompt(playedFraction: 0, reachedEnd: true, signedIn: false),
        isFalse,
      );
      expect(
        prompt(playedFraction: 0, reachedEnd: true, rated: RatedState.rated),
        isFalse,
      );
      expect(
        prompt(playedFraction: 0, reachedEnd: true, declined: true),
        isFalse,
      );
    });
  });

  group('rememberDeclined', () {
    test('appends in order and does not mutate the input', () {
      const start = <String>['a', 'b'];
      final next = rememberDeclined(start, 'c');
      expect(next, ['a', 'b', 'c']);
      expect(start, ['a', 'b']);
    });

    test('re-offering an id moves it to the end instead of duplicating', () {
      expect(rememberDeclined(['a', 'b', 'c'], 'a'), ['b', 'c', 'a']);
    });

    test('trims the oldest entries at the cap', () {
      final full = [for (var i = 0; i < 5; i++) 'id$i'];
      expect(rememberDeclined(full, 'new', max: 5), [
        'id1',
        'id2',
        'id3',
        'id4',
        'new',
      ]);
      // Already-present ids do not grow the list, so nothing is trimmed.
      expect(rememberDeclined(full, 'id0', max: 5).length, 5);
    });

    test('the default cap keeps the persisted value bounded', () {
      var offered = <String>[];
      for (var i = 0; i < kRatingPromptMemoryMax + 50; i++) {
        offered = rememberDeclined(offered, 'id$i');
      }
      expect(offered.length, kRatingPromptMemoryMax);
      expect(offered.last, 'id${kRatingPromptMemoryMax + 49}');
      expect(offered.first, 'id50'); // the first 50 fell off
    });
  });
}
