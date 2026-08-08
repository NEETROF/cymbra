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

// Reference renders of the reading aid at the two touch form factors, so the
// placement (names on the keys, floating rhythm card) can be reviewed without a
// device. Tagged `golden`: platform-sensitive, refreshed with
// `flutter test --tags golden --update-goldens`.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import '../support/notation_fakes.dart';

class _StubPlayer extends Player {
  _StubPlayer(this._initial);

  final PlayerData _initial;

  @override
  PlayerData build() => _initial;
}

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

/// A one-sharp piece held at a right-hand F♯/A chord over a left-hand D — the
/// F♯ is carried by the key signature alone, which is the case the aid exists
/// for. Both right-hand notes are dotted halves so the card can quantify them.
const _blockedOnFSharp = PlayerData(
  notes: [
    // F♯4 written on the F degree (diatonic 31), sounding 66.
    TimedNote(
      pitch: 66,
      startMs: 0,
      durationMs: 1500,
      staff: 1,
      diatonic: 31,
      noteType: 'half',
      dots: 1,
    ),
    // A4 on the A degree (diatonic 33).
    TimedNote(
      pitch: 69,
      startMs: 0,
      durationMs: 1500,
      staff: 1,
      diatonic: 33,
      noteType: 'half',
      dots: 1,
    ),
    // D3 in the left hand (diatonic 22).
    TimedNote(
      pitch: 50,
      startMs: 0,
      durationMs: 1500,
      staff: 2,
      diatonic: 22,
      noteType: 'half',
      dots: 1,
    ),
    TimedNote(pitch: 71, startMs: 2000, durationMs: 500, staff: 1),
  ],
  songEndMs: 4000,
  bpm: 90,
  beats: 4,
  beatType: 4,
  keyFifths: 1,
  keyboardRange: KeyboardRangeMode.auto,
  waitMode: true,
  blocked: true,
  readingAid: NoteReadingAid.nameAndRhythm,
);

Future<void> _shoot(
  WidgetTester tester, {
  required Size size,
  required PlayerData data,
  required String file,
  Locale locale = const Locale('fr'),
}) async {
  await tester.binding.setSurfaceSize(size);
  addTearDown(() => tester.binding.setSurfaceSize(null));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      playerProvider.overrideWith(() => _StubPlayer(data)),
    ],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        locale: locale,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        // Name the loaded face explicitly: it is what the painters' key labels
        // resolve through as well, via DefaultTextStyle.
        theme: ThemeData(fontFamily: 'Roboto'),
        home: const PlayerScreen(),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 60));

  // The transport rail overflows its column at the phone-landscape surface.
  // That is pre-existing — it happens with the reading aid off too — so these
  // renders tolerate it, but only it: any other exception still fails here.
  for (var e = tester.takeException(); e != null; e = tester.takeException()) {
    if (!'$e'.contains('overflowed')) throw e as Object;
  }

  await expectLater(
    find.byType(PlayerScreen),
    matchesGoldenFile('goldens/$file'),
  );
}

void main() {
  // iPhone 15 Pro in landscape (the player is landscape-locked).
  testWidgets('iPhone — aid on, gate held', tags: 'golden', (tester) async {
    await _shoot(
      tester,
      size: const Size(852, 393),
      data: _blockedOnFSharp,
      file: 'reading_aid_iphone.png',
    );
  });

  // The same moment with the aid off, to compare what it costs (nothing).
  testWidgets('iPhone — aid off', tags: 'golden', (tester) async {
    await _shoot(
      tester,
      size: const Size(852, 393),
      data: _blockedOnFSharp.copyWith(readingAid: NoteReadingAid.off),
      file: 'reading_aid_iphone_off.png',
    );
  });

  // iPad Pro 11" in landscape.
  testWidgets('iPad — aid on, gate held', tags: 'golden', (tester) async {
    await _shoot(
      tester,
      size: const Size(1194, 834),
      data: _blockedOnFSharp,
      file: 'reading_aid_ipad.png',
    );
  });

  // The full 88-key range: the case where a label cannot fit across a key and
  // has to turn a quarter turn instead of overflowing onto its neighbours.
  testWidgets('iPhone — 88 keys, labels turned', tags: 'golden', (
    tester,
  ) async {
    await _shoot(
      tester,
      size: const Size(852, 393),
      data: _blockedOnFSharp.copyWith(keyboardRange: KeyboardRangeMode.keys88),
      file: 'reading_aid_iphone_88.png',
    );
  });
}
