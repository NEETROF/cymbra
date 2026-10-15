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

/// The showcase video scenario (change: add-site-music-showcase, D3/D6).
///
/// Drives the REAL app — a real bundled score, parsed and laid out by the real
/// Rust bridge, a faked keyboard that plays it in Wait Mode — and draws the video
/// one frame at a time on a [VirtualClock]: every frame is rendered from the
/// app's own layer tree at an exact instant, and every sound command is logged
/// at that same instant. Nothing is filmed, so nothing can drift.
///
/// Run it through `tool/record_showcase_video.sh <locale>` (or
/// `melos run showcase-video -- <locale>`): the driver collects the frames and
/// the sound log, the script renders the sound and assembles the MP4.
@Tags(['capture'])
library;

import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:music/main.dart';
import 'package:music/services/clock_service.dart';
import 'package:music/src/rust/frb_generated.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/widgets/score_card.dart';

import '../tool/store_manifest.dart';
import 'support/capture_fixtures.dart';
import 'support/showcase_fixtures.dart';

/// The locale this run records.
const String _locale = String.fromEnvironment(
  'CAPTURE_LOCALE',
  defaultValue: 'en',
);

/// The layout the video is drawn in: the iPad 13" class, the device a piano's
/// music stand holds (design D2).
const String _targetId = 'ios/ipad_13';

/// Output width in pixels; the height follows the target's 4:3.
const double _videoWidth = 1280;

const int _fps = 30;
const double _frameMs = 1000 / _fps;

/// Length of the video. The site's budget is 30 s (spec: site-music-page).
const double _durationMs = 26000;

/// When the reading switches from the engraved staff to the falling notes, so
/// the one video shows both ways to read.
const double _switchToCascadeMs = 13000;

/// How long the first frame holds before the first key, so the loop has a
/// visible start.
const double _leadInMs = 800;

/// The piece: the same flowing, dense bundled score the store captures use.
const String _scoreId = 'arabesque-l-66-no-1-in-e-major';

const Key _rootKey = Key('showcase-root');

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => RustLib.init());

  testWidgets('records the showcase video', (tester) async {
    expect(kCaptureLocales, contains(_locale), reason: 'not a shipping locale');
    final target = captureTargetById(_targetId)!;
    tester.view.physicalSize = Size(
      target.widthPx.toDouble(),
      target.heightPx.toDouble(),
    );
    tester.view.devicePixelRatio = target.devicePixelRatio;
    addTearDown(tester.view.reset);
    final pixelRatio = _videoWidth / target.logicalWidth;

    final clock = VirtualClock();
    final audio = RecordingAudioService(clock);
    final midi = CaptureMidiService();
    addTearDown(midi.dispose);
    // Tickers run until the player is on screen (route transitions need them),
    // then stop: from there on, time moves only when the scenario moves it.
    final tickers = ValueNotifier<bool>(true);

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          ...captureOverrides(locale: _locale, midi: midi, audio: audio),
          clockProvider.overrideWithValue(clock),
        ],
        child: RepaintBoundary(
          key: _rootKey,
          child: ValueListenableBuilder<bool>(
            valueListenable: tickers,
            builder: (_, enabled, child) =>
                TickerMode(enabled: enabled, child: child!),
            child: const CymbraApp(),
          ),
        ),
      ),
    );
    await _settle(tester, frames: 20);
    final container = ProviderScope.containerOf(
      tester.element(find.byType(CymbraApp)),
    );

    // --- Open the piece -----------------------------------------------------
    final card = find.byWidgetPredicate(
      (w) => w is ScoreCard && w.entry.id == _scoreId,
    );
    if (card.evaluate().isEmpty) {
      await tester.scrollUntilVisible(
        card,
        200,
        scrollable: find
            .descendant(
              of: find.byType(CustomScrollView),
              matching: find.byType(Scrollable),
            )
            .first,
      );
    }
    await tester.ensureVisible(card);
    await _settle(tester, frames: 6);
    expect(card, findsOneWidget, reason: 'the showcase score left the catalog');
    await tester.tap(card);
    await _settle(tester, frames: 30);
    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _settle(tester, frames: 10);

    final player = container.read(playerProvider.notifier);
    expect(
      container.read(playerProvider).midiConnected,
      isTrue,
      reason: 'the player would show its "no MIDI device" chip',
    );
    expect(
      container.read(playerProvider).waitMode,
      isTrue,
      reason: 'the faked keyboard follows the score in Wait Mode',
    );
    player.setMode(RenderMode.staff);
    await _settle(tester, frames: 6);

    // --- Record ---------------------------------------------------------------
    tickers.value = false;
    await tester.pump();
    final frames = await Directory.systemTemp.createTemp('cymbra_showcase_');
    final held = <int>{};
    var switched = false;
    var playing = false;
    final total = (_durationMs / _frameMs).round();

    for (var i = 0; i < total; i++) {
      clock.ms = i * _frameMs;

      if (!playing && clock.ms >= _leadInMs) {
        player.setPlaying(true);
        playing = true;
      }
      if (!switched && clock.ms >= _switchToCascadeMs) {
        player.setMode(RenderMode.synthesia);
        switched = true;
      }
      if (playing) {
        if (i > 0) player.advance(_frameMs);
        // A pianist reading the score: when the playhead waits on an onset,
        // lift the previous keys and play the ones it asks for.
        final s = container.read(playerProvider);
        if (s.blocked && s.expectedKeys.isNotEmpty) {
          for (final pitch in held) {
            midi.release(pitch);
          }
          held
            ..clear()
            ..addAll(s.expectedKeys);
          for (final pitch in held) {
            midi.press(pitch, velocity: 80 + (pitch % 5) * 6);
          }
          // Let the keyboard's events reach the player before the frame.
          await tester.runAsync(() => Future<void>.delayed(Duration.zero));
        }
      }

      await tester.pump();
      await _writeFrame(tester, frames, i, pixelRatio);
    }
    for (final pitch in held) {
      midi.release(pitch);
    }
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    clock.ms = total * _frameMs;
    audio.end();

    // --- The run reached its expected state ---------------------------------
    expect(
      container.read(performanceScorerProvider).syncPercent,
      greaterThan(50),
      reason: 'the video would show a failed performance',
    );
    expect(
      audio.lines.where((l) => l.contains(' on ')).length,
      greaterThan(20),
      reason: 'the video would be (nearly) silent',
    );

    final report = binding.reportData ??= <String, dynamic>{};
    report['showcase'] = <String, dynamic>{
      'locale': _locale,
      'fps': _fps,
      'frames': total,
      'framesDir': frames.path,
      'soundLog': audio.lines,
    };
  });
}

/// Draws the current frame from the app's layer tree and writes it as
/// `frame_NNNNN.png` in [dir].
Future<void> _writeFrame(
  WidgetTester tester,
  Directory dir,
  int index,
  double pixelRatio,
) async {
  final boundary = tester.renderObject<RenderRepaintBoundary>(
    find.byKey(_rootKey),
  );
  await tester.runAsync(() async {
    final image = await boundary.toImage(pixelRatio: pixelRatio);
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    final name = 'frame_${index.toString().padLeft(5, '0')}.png';
    await File('${dir.path}/$name').writeAsBytes(data!.buffer.asUint8List());
  });
}

/// Pumps a fixed number of frames (the player animates continuously, so
/// `pumpAndSettle` would never return).
Future<void> _settle(WidgetTester tester, {int frames = 10}) async {
  for (var i = 0; i < frames; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}
