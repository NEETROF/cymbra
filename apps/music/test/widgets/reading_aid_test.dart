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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/piano_keyboard_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/widgets/reading_aid.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

/// A `Player` whose state the test drives directly, so the aid can be shown the
/// exact gate situations it must react to without running real playback.
class _StubPlayer extends Player {
  _StubPlayer(this._initial);

  final PlayerData _initial;

  @override
  PlayerData build() => _initial;

  void put(PlayerData next) => state = next;
}

// C4 and E4 (right hand) attacked together at 0, both half notes; a left-hand
// C2 quarter at the same onset.
const _c4 = TimedNote(
  pitch: 60,
  startMs: 0,
  durationMs: 1000,
  staff: 1,
  diatonic: 28,
  noteType: 'half',
);
const _e4 = TimedNote(
  pitch: 64,
  startMs: 0,
  durationMs: 1000,
  staff: 1,
  diatonic: 30,
  noteType: 'half',
);
const _c2 = TimedNote(
  pitch: 36,
  startMs: 0,
  durationMs: 500,
  staff: 2,
  diatonic: 14,
  noteType: 'quarter',
);

PlayerData _data({
  required NoteReadingAid aid,
  required bool blocked,
  List<TimedNote> notes = const [_c4],
  bool waitMode = true,
  int keyFifths = 0,
}) => PlayerData(
  notes: notes,
  songEndMs: 4000,
  bpm: 120,
  beats: 4,
  beatType: 4,
  keyFifths: keyFifths,
  waitMode: waitMode,
  blocked: blocked,
  readingAid: aid,
);

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

final _figureCard = find.byKey(const Key('reading-aid-figure'));

/// The keyboard painter currently mounted in the player.
PianoKeyboardPainter _painter(WidgetTester tester) =>
    (tester
            .widgetList<CustomPaint>(find.byType(CustomPaint))
            .firstWhere((w) => w.painter is PianoKeyboardPainter)
            .painter
        as PianoKeyboardPainter);

Future<_StubPlayer> _pumpPlayer(
  WidgetTester tester,
  PlayerData data, {
  Locale? locale,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  late _StubPlayer stub;
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      playerProvider.overrideWith(() {
        stub = _StubPlayer(data);
        return stub;
      }),
    ],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        const PlayerScreen(),
        locale: locale ?? const Locale('en'),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 50));
  return stub;
}

void main() {
  group('names on the keys', () {
    testWidgets('no labels when the aid is off', (tester) async {
      await _pumpPlayer(tester, _data(aid: NoteReadingAid.off, blocked: true));
      expect(_painter(tester).noteLabels, isEmpty);
    });

    testWidgets('no labels while the gate is not blocking', (tester) async {
      await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.name, blocked: false),
      );
      expect(_painter(tester).noteLabels, isEmpty);
    });

    testWidgets('no labels outside Wait Mode', (tester) async {
      await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.name, blocked: true, waitMode: false),
      );
      expect(_painter(tester).noteLabels, isEmpty);
    });

    testWidgets('the awaited key carries its name', (tester) async {
      await _pumpPlayer(tester, _data(aid: NoteReadingAid.name, blocked: true));
      expect(_painter(tester).noteLabels, {60: 'C'});
    });

    testWidgets('labels are withdrawn when the playhead resumes', (
      tester,
    ) async {
      final stub = await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.name, blocked: true),
      );
      expect(_painter(tester).noteLabels, isNotEmpty);

      stub.put(_data(aid: NoteReadingAid.name, blocked: false));
      await tester.pump();
      expect(_painter(tester).noteLabels, isEmpty);
    });

    testWidgets('every note of a chord is named on its own key', (
      tester,
    ) async {
      await _pumpPlayer(
        tester,
        _data(
          aid: NoteReadingAid.name,
          blocked: true,
          notes: const [_c4, _e4, _c2],
        ),
      );
      expect(_painter(tester).noteLabels, {60: 'C', 64: 'E', 36: 'C'});
    });

    testWidgets('a large chord is not truncated — each key carries its own', (
      tester,
    ) async {
      final notes = [
        for (var i = 0; i < 6; i++)
          TimedNote(
            pitch: 60 + i,
            startMs: 0,
            durationMs: 500,
            diatonic: 28 + i,
          ),
      ];
      await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.name, blocked: true, notes: notes),
      );
      expect(_painter(tester).noteLabels.length, 6);
    });

    testWidgets('only the selected hand is named', (tester) async {
      await _pumpPlayer(
        tester,
        _data(
          aid: NoteReadingAid.name,
          blocked: true,
          notes: const [_c4, _c2],
        ).copyWith(selectedHands: Hand.left),
      );
      expect(_painter(tester).noteLabels, {36: 'C'});
    });

    testWidgets('French names the keys in solfège', (tester) async {
      await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.name, blocked: true),
        locale: const Locale('fr'),
      );
      final p = _painter(tester);
      expect(p.noteLabels, {60: 'Do'});
      expect(p.solfege, isTrue);
      expect(p.frenchRe, isTrue);
    });
  });

  group('rhythm card', () {
    testWidgets('absent at the name-only level', (tester) async {
      await _pumpPlayer(tester, _data(aid: NoteReadingAid.name, blocked: true));
      expect(_figureCard, findsNothing);
    });

    testWidgets('names and quantifies the figure at the rhythm level', (
      tester,
    ) async {
      await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.nameAndRhythm, blocked: true),
      );
      expect(_figureCard, findsOneWidget);
      expect(find.text('half note'), findsOneWidget);
      expect(find.text('hold 2 beats'), findsOneWidget);
    });

    testWidgets('withdrawn as soon as the playhead resumes', (tester) async {
      final stub = await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.nameAndRhythm, blocked: true),
      );
      expect(_figureCard, findsOneWidget);
      stub.put(_data(aid: NoteReadingAid.nameAndRhythm, blocked: false));
      await tester.pump();
      expect(_figureCard, findsNothing);
    });

    testWidgets('mixed figures show no figure', (tester) async {
      // C4 is a half note, C2 a quarter: naming one figure would be a lie.
      await _pumpPlayer(
        tester,
        _data(
          aid: NoteReadingAid.nameAndRhythm,
          blocked: true,
          notes: const [_c4, _c2],
        ),
      );
      expect(_figureCard, findsNothing);
      // …but the names are still on the keys.
      expect(_painter(tester).noteLabels.length, 2);
    });

    testWidgets('a shared figure is shown once for the chord', (tester) async {
      await _pumpPlayer(
        tester,
        _data(
          aid: NoteReadingAid.nameAndRhythm,
          blocked: true,
          notes: const [_c4, _e4],
        ),
      );
      expect(find.text('half note'), findsOneWidget);
    });

    testWidgets('French names the figure in French', (tester) async {
      await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.nameAndRhythm, blocked: true),
        locale: const Locale('fr'),
      );
      expect(find.text('blanche'), findsOneWidget);
    });
  });

  group('costs the score no layout space', () {
    testWidgets('the render area is identical with the aid off and on', (
      tester,
    ) async {
      final stub = await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.off, blocked: false),
      );
      final renderArea = find.byKey(const Key('render-area'));
      final withoutAid = tester.getRect(renderArea);

      // Turning the aid on takes nothing from the score.
      stub.put(_data(aid: NoteReadingAid.nameAndRhythm, blocked: false));
      await tester.pump();
      expect(tester.getRect(renderArea), withoutAid);

      // …and it does not move when the gate blocks and the aid fills in.
      stub.put(_data(aid: NoteReadingAid.nameAndRhythm, blocked: true));
      await tester.pump();
      expect(_figureCard, findsOneWidget);
      expect(tester.getRect(renderArea), withoutAid);

      stub.put(_data(aid: NoteReadingAid.nameAndRhythm, blocked: false));
      await tester.pump();
      expect(tester.getRect(renderArea), withoutAid);
    });

    testWidgets('the keyboard keeps its size too', (tester) async {
      final stub = await _pumpPlayer(
        tester,
        _data(aid: NoteReadingAid.name, blocked: false),
      );
      final keyboard = find.byKey(const Key('onscreen-keyboard'));
      final before = tester.getRect(keyboard);
      stub.put(_data(aid: NoteReadingAid.name, blocked: true));
      await tester.pump();
      expect(tester.getRect(keyboard), before);
    });
  });

  group('readingAidViewOf', () {
    test('names a key-signature alteration with no engraved accidental', () {
      // F♯4 written on the F degree under one sharp, carried by the key alone.
      const f = TimedNote(
        pitch: 66,
        startMs: 0,
        durationMs: 500,
        diatonic: 31,
        noteType: 'quarter',
      );
      final view = readingAidViewOf(
        _data(aid: NoteReadingAid.name, blocked: true, notes: const [f]),
        solfege: false,
        frenchRe: false,
      );
      expect(view.names, {66: 'F♯'});
    });

    test('uses the key signature of the measure the note falls in', () {
      // Two measures, the second modulating to two flats: a B in it is a B♭.
      const b = TimedNote(pitch: 70, startMs: 2000, durationMs: 500);
      final data = PlayerData(
        notes: const [b],
        measureStartMs: const [0, 2000],
        measureKeyFifths: const [0, -2],
        songEndMs: 4000,
        blocked: true,
        readingAid: NoteReadingAid.name,
        elapsedMs: 2000,
      );
      expect(readingAidViewOf(data, solfege: false, frenchRe: false).names, {
        70: 'B♭',
      });
    });

    test('is hidden when there is nothing to await', () {
      const empty = PlayerData(
        blocked: true,
        readingAid: NoteReadingAid.nameAndRhythm,
      );
      expect(
        readingAidViewOf(empty, solfege: false, frenchRe: false).show,
        isFalse,
      );
    });

    test('rebuild guard: the view is stable while nothing displayed changes', () {
      final a = _data(aid: NoteReadingAid.nameAndRhythm, blocked: true);
      // A different playhead that still resolves to the same onset must produce
      // an equal view, so `select` does not rebuild the aid every frame.
      final b = a.copyWith(elapsedMs: 0.5);
      final va = readingAidViewOf(a, solfege: false, frenchRe: false);
      final vb = readingAidViewOf(b, solfege: false, frenchRe: false);
      expect(va, vb);
      expect(va.hashCode, vb.hashCode);
    });

    test('views differing only by a name are not equal', () {
      final en = readingAidViewOf(
        _data(aid: NoteReadingAid.name, blocked: true),
        solfege: false,
        frenchRe: false,
      );
      final fr = readingAidViewOf(
        _data(aid: NoteReadingAid.name, blocked: true),
        solfege: true,
        frenchRe: true,
      );
      expect(en, isNot(fr));
    });
  });
}
