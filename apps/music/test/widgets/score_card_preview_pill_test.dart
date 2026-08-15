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

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/score_preview_service.dart';
import 'package:music/services/sound_clip_player.dart';
import 'package:music/services/wav_duration.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/score_preview_playback.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/usage_tracking_notifier.dart';
import 'package:music/widgets/score_card.dart';

import '../support/localized.dart';

class _NoStandings implements LeaderboardService {
  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};

  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async => Leaderboard.empty;
}

/// A 44-byte header + [ms] of silence (mono 16-bit 44.1 kHz).
Uint8List wav({int ms = 10}) {
  const sampleRate = 44100;
  final frames = sampleRate * ms ~/ 1000;
  final data = ByteData(44 + frames * 2);
  void ascii(int at, String s) {
    for (var i = 0; i < s.length; i++) {
      data.setUint8(at + i, s.codeUnitAt(i));
    }
  }

  ascii(0, 'RIFF');
  data.setUint32(4, 36 + frames * 2, Endian.little);
  ascii(8, 'WAVE');
  ascii(12, 'fmt ');
  data.setUint32(16, 16, Endian.little);
  data.setUint16(20, 1, Endian.little);
  data.setUint16(22, 1, Endian.little);
  data.setUint32(24, sampleRate, Endian.little);
  data.setUint32(28, sampleRate * 2, Endian.little);
  data.setUint16(32, 2, Endian.little);
  data.setUint16(34, 16, Endian.little);
  ascii(36, 'data');
  data.setUint32(40, frames * 2, Endian.little);
  return data.buffer.asUint8List();
}

class _FakePreview implements ScorePreviewService {
  _FakePreview({this.exists = const {}, this.error});
  final Set<String> exists;
  final Object? error;
  final List<String> fetched = [];

  @override
  Future<Uint8List?> fetchClip(String catalogId) async {
    fetched.add(catalogId);
    if (error != null) throw error!;
    return exists.contains(catalogId) ? wav() : null;
  }
}

class _FakeClipPlayer implements SoundClipPlayer {
  final List<int> played = [];
  int stops = 0;

  @override
  Future<void> play(Uint8List bytes) async => played.add(bytes.length);

  @override
  Future<void> stop() async => stops++;
}

CatalogEntry _entry({bool hasPreview = true, String id = 'x'}) => CatalogEntry(
  id: 'catalog-$id',
  title: 'Canon in D',
  composer: 'Pachelbel',
  level: PracticeLevel.beginner,
  catalogId: id,
  hasPreview: hasPreview,
);

ProviderContainer _container({_FakePreview? preview, _FakeClipPlayer? player}) {
  final c = ProviderContainer(
    overrides: [
      leaderboardServiceProvider.overrideWithValue(_NoStandings()),
      scorePreviewServiceProvider.overrideWithValue(
        preview ?? _FakePreview(exists: const {'x'}),
      ),
      soundClipPlayerProvider.overrideWithValue(player ?? _FakeClipPlayer()),
      canUseOnlineServicesProvider.overrideWithValue(false),
      usageCollectionKillSwitchProvider.overrideWithValue(false),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer c, {
  required CatalogEntry entry,
  required VoidCallback onTap,
}) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        Scaffold(
          body: Center(
            child: SizedBox(
              width: 220,
              height: 320,
              child: ScoreCard(entry: entry, onTap: onTap),
            ),
          ),
        ),
        locale: const Locale('en'),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  test('wavDuration reads the canonical header', () {
    expect(wavDuration(wav(ms: 10)), const Duration(milliseconds: 10));
    expect(wavDuration(wav(ms: 1500)), const Duration(milliseconds: 1500));
    expect(wavDuration(Uint8List(10)), isNull);
    expect(wavDuration(Uint8List.fromList(List.filled(60, 0))), isNull);
  });

  testWidgets('no pill without a teaser', (tester) async {
    final c = _container();
    await _pump(tester, c, entry: _entry(hasPreview: false), onTap: () {});
    expect(find.byKey(const Key('score-preview-x')), findsNothing);
  });

  testWidgets('the pill plays once and never opens the piece', (tester) async {
    var opened = 0;
    final preview = _FakePreview(exists: const {'x'});
    final player = _FakeClipPlayer();
    final c = _container(preview: preview, player: player);
    await _pump(tester, c, entry: _entry(), onTap: () => opened++);

    final pill = find.byKey(const Key('score-preview-x'));
    expect(pill, findsOneWidget);
    expect(find.text('Excerpt'), findsOneWidget);

    await tester.tap(pill);
    await tester.pump();
    await tester.pump();
    expect(opened, 0, reason: 'the pill is a secondary control');
    expect(preview.fetched, ['x']);
    expect(player.played, hasLength(1));
    expect(find.text('Stop'), findsOneWidget);
    expect(c.read(scorePreviewPlaybackProvider).playingId, 'x');

    // ONE pass: the 10 ms clip ends by itself.
    await tester.pump(const Duration(milliseconds: 50));
    expect(player.stops, 1);
    expect(c.read(scorePreviewPlaybackProvider).playingId, isNull);
    expect(find.text('Excerpt'), findsOneWidget);

    // Tapping the card body still opens the piece.
    await tester.tapAt(tester.getCenter(find.text('Canon in D')));
    await tester.pump();
    expect(opened, 1);
    // A second audition is served from the session cache (no re-fetch).
    await tester.tap(pill);
    await tester.pump();
    await tester.pump();
    expect(preview.fetched, ['x']);
    expect(player.played, hasLength(2));
    // Let the second pass end (no pending timer at teardown).
    await tester.pump(const Duration(milliseconds: 50));
    expect(player.stops, 2);
  });

  testWidgets('a 404 greys the pill; a failure bumps the error cue', (
    tester,
  ) async {
    final c = _container(preview: _FakePreview(exists: const {}));
    await _pump(tester, c, entry: _entry(), onTap: () {});
    await tester.tap(find.byKey(const Key('score-preview-x')));
    await tester.pump();
    await tester.pump();
    expect(c.read(scorePreviewPlaybackProvider).missing, contains('x'));
    expect(c.read(scorePreviewPlaybackProvider).playingId, isNull);

    final c2 = _container(preview: _FakePreview(error: Exception('boom')));
    await c2.read(scorePreviewPlaybackProvider.notifier).toggle('x');
    expect(c2.read(scorePreviewPlaybackProvider).errorSeq, 1);
    expect(c2.read(scorePreviewPlaybackProvider).loadingId, isNull);
  });

  test(
    'one clip at a time; stop() silences and toggle() stops the same id',
    () async {
      final preview = _FakePreview(exists: const {'a', 'b'});
      final player = _FakeClipPlayer();
      final c = _container(preview: preview, player: player);
      final n = c.read(scorePreviewPlaybackProvider.notifier);
      await n.toggle('a');
      expect(c.read(scorePreviewPlaybackProvider).playingId, 'a');
      await n.toggle('b');
      expect(c.read(scorePreviewPlaybackProvider).playingId, 'b');
      expect(player.stops, 1, reason: 'a was stopped before b started');
      await n.toggle('b');
      expect(c.read(scorePreviewPlaybackProvider).playingId, isNull);
      expect(player.stops, 2);
      await n.stop();
      expect(player.stops, 2, reason: 'nothing to stop');
    },
  );
}
