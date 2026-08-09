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

import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_preferences.dart';

import '../support/prefs_fakes.dart';

ProviderContainer _container(FakePreferencesService prefs) {
  final c = ProviderContainer(
    overrides: [preferencesServiceProvider.overrideWithValue(prefs)],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('defaults when nothing is stored', () {
    final c = _container(FakePreferencesService());
    final p = c.read(playerPreferencesProvider);
    expect(p.hands, Hand.both);
    expect(p.speed, 1.0);
    expect(p.metronome, isFalse);
    expect(p.keyboardRange, KeyboardRangeMode.auto);
    expect(p.readingAid, NoteReadingAid.name);
    expect(p.scoreSize, isNull); // resolved per form factor at use sites
    expect(p.notationTheme, NotationTheme.dark);
    expect(p.midiPort, isNull);
  });

  test('restores the persisted prefs (previous launch)', () async {
    final store = {
      PlayerPreferences.prefsKey: jsonEncode({
        'hands': 'left',
        'speed': 1.5,
        'metronome': true,
        'keyboardRange': 'keys61',
        'scoreSize': 'large',
        'notationTheme': 'paper',
        'midiPort': 'Synth',
      }),
    };
    final c = _container(FakePreferencesService(store));
    c.read(playerPreferencesProvider); // trigger build + async restore
    await Future<void>.delayed(Duration.zero);

    final p = c.read(playerPreferencesProvider);
    expect(p.hands, Hand.left);
    expect(p.speed, 1.5);
    expect(p.metronome, isTrue);
    expect(p.keyboardRange, KeyboardRangeMode.keys61);
    expect(p.scoreSize, ScoreSize.large);
    expect(p.notationTheme, NotationTheme.paper);
    expect(p.midiPort, 'Synth');
  });

  test('a record without scoreSize (older launch) stays unchosen', () async {
    final store = {
      PlayerPreferences.prefsKey: jsonEncode({
        'hands': 'both',
        'speed': 1.0,
        'metronome': false,
        'midiPort': null,
      }),
    };
    final c = _container(FakePreferencesService(store));
    c.read(playerPreferencesProvider);
    await Future<void>.delayed(Duration.zero);
    expect(c.read(playerPreferencesProvider).scoreSize, isNull);
  });

  test('the unchosen size resolves small on phones, medium elsewhere', () {
    expect(resolveScoreSize(null, isPhone: true), ScoreSize.small);
    expect(resolveScoreSize(null, isPhone: false), ScoreSize.medium);
    // A stored choice always wins over the form-factor default.
    expect(resolveScoreSize(ScoreSize.large, isPhone: true), ScoreSize.large);
  });

  test('each setter persists the whole record to the device', () async {
    final fake = FakePreferencesService();
    final c = _container(fake);
    final notifier = c.read(playerPreferencesProvider.notifier);

    notifier.setHands(Hand.right);
    notifier.setSpeed(1.25);
    notifier.setMetronome(enabled: true);
    notifier.setKeyboardRange(KeyboardRangeMode.keys25);
    notifier.setMidiPort('Piano');
    notifier.setScoreSize(ScoreSize.small);
    notifier.setNotationTheme(NotationTheme.paper);
    await Future<void>.delayed(Duration.zero);

    final saved =
        jsonDecode(fake.store[PlayerPreferences.prefsKey]!)
            as Map<String, dynamic>;
    expect(saved['hands'], 'right');
    expect(saved['speed'], 1.25);
    expect(saved['metronome'], isTrue);
    expect(saved['keyboardRange'], 'keys25');
    expect(saved['midiPort'], 'Piano');
    expect(saved['scoreSize'], 'small');
    expect(saved['notationTheme'], 'paper');
  });

  test('score sizes map to their notation scale factors', () {
    expect(ScoreSize.small.factor, 0.85);
    expect(ScoreSize.medium.factor, 1.0);
    expect(ScoreSize.large.factor, 1.2);
  });

  test('the reading-aid level round-trips', () async {
    final fake = FakePreferencesService();
    final c = _container(fake);
    c
        .read(playerPreferencesProvider.notifier)
        .setNoteReadingAid(NoteReadingAid.nameAndRhythm);
    await Future<void>.delayed(Duration.zero);

    final saved =
        jsonDecode(fake.store[PlayerPreferences.prefsKey]!)
            as Map<String, dynamic>;
    expect(saved['readingAid'], 'nameAndRhythm');

    // A fresh container restores it from that same record.
    final restored = _container(FakePreferencesService(fake.store));
    restored.read(playerPreferencesProvider);
    await Future<void>.delayed(Duration.zero);
    expect(
      restored.read(playerPreferencesProvider).readingAid,
      NoteReadingAid.nameAndRhythm,
    );
  });

  test('a record written before the aid existed restores the rest', () async {
    // No `readingAid` key at all — the aid takes its default, the rest loads.
    final c = _container(
      FakePreferencesService({
        PlayerPreferences.prefsKey: jsonEncode({
          'hands': 'right',
          'speed': 0.75,
          'metronome': true,
        }),
      }),
    );
    c.read(playerPreferencesProvider);
    await Future<void>.delayed(Duration.zero);

    final p = c.read(playerPreferencesProvider);
    expect(p.readingAid, NoteReadingAid.name);
    expect(p.hands, Hand.right);
    expect(p.speed, 0.75);
    expect(p.metronome, isTrue);
  });

  test('an unrecognized level falls back without losing the record', () async {
    final c = _container(
      FakePreferencesService({
        PlayerPreferences.prefsKey: jsonEncode({
          'hands': 'left',
          'speed': 1.5,
          'readingAid': 'holographicStaff',
        }),
      }),
    );
    c.read(playerPreferencesProvider);
    await Future<void>.delayed(Duration.zero);

    final p = c.read(playerPreferencesProvider);
    expect(p.readingAid, NoteReadingAid.name);
    expect(p.hands, Hand.left);
    expect(p.speed, 1.5);
  });

  test(
    'an explicit off survives — the default never overrides a choice',
    () async {
      final c = _container(
        FakePreferencesService({
          PlayerPreferences.prefsKey: jsonEncode({
            'hands': 'both',
            'readingAid': 'off',
          }),
        }),
      );
      c.read(playerPreferencesProvider);
      await Future<void>.delayed(Duration.zero);
      expect(c.read(playerPreferencesProvider).readingAid, NoteReadingAid.off);
    },
  );

  test('a corrupt stored value falls back to defaults', () async {
    final c = _container(
      FakePreferencesService({PlayerPreferences.prefsKey: 'not json'}),
    );
    c.read(playerPreferencesProvider);
    await Future<void>.delayed(Duration.zero);
    expect(c.read(playerPreferencesProvider).hands, Hand.both);
  });
}
