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

/// The store-listing capture scenario (change: add-store-screenshot-harness).
///
/// Drives the REAL app — real bundled scores, parsed and laid out by the real
/// Rust bridge — through the listing surfaces, and reports one image per
/// surface to `test_driver/integration_test.dart`, which writes them into
/// `store/`. Not a behaviour test: it asserts only that the seeded state took
/// effect, so a silently-failed override fails the run instead of shipping a
/// bad image (task 2.7).
///
/// Run it through `melos run screenshots` (see `melos.yaml`) rather than
/// directly — it needs `flutter drive`, the target and the locale:
///
/// ```bash
/// melos run screenshots --  ios/iphone_6.9 fr
/// ```
///
/// Captures are rendered from a [RepaintBoundary] wrapped around the app at the
/// target's declared size, NOT from `binding.takeScreenshot`: the
/// `integration_test` plugin implements screenshots on Android and iOS only
/// (its macOS plugin answers `FlutterMethodNotImplemented`), and rendering the
/// layer tree ourselves also fixes the output dimensions by construction
/// instead of inheriting whatever the capture device happens to be.
@Tags(['capture'])
library;

import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:music/main.dart';
import 'package:music/src/rust/frb_generated.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/favorite_scores.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/widgets/score_card.dart';

import '../tool/store_manifest.dart';
import 'support/capture_fixtures.dart';

/// The target to capture, as `<platform>/<sizeClass>` (see `store/manifest.json`).
const String _targetId = String.fromEnvironment(
  'CAPTURE_TARGET',
  defaultValue: 'macos/desktop_1440x900',
);

/// The locale this run captures.
const String _locale = String.fromEnvironment(
  'CAPTURE_LOCALE',
  defaultValue: 'en',
);

/// The bundled drum groove the percussion surfaces are captured on: two voices,
/// an open hi-hat and a crash, so the lanes, the kick bar and the engraved
/// percussion all have something to show (change: add-instrument-context 6.1).
const String _drumScoreId = 'groove-ouvert';

/// The score the player surfaces are captured on: a visually dense bundled
/// piece, so the falling notes and the staff have something to show.
const String _scoreId = 'arabesque-l-66-no-1-in-e-major';

/// Root boundary the captures are rendered from.
const Key _rootKey = Key('capture-root');

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => RustLib.init());

  testWidgets('captures the store listing set', (tester) async {
    final target = captureTargetById(_targetId);
    expect(
      target,
      isNotNull,
      reason:
          '"$_targetId" is not a declared capture target. Declared: '
          '${kCaptureTargets.map((t) => t.id).join(', ')}.',
    );
    expect(
      kCaptureLocales,
      contains(_locale),
      reason: '"$_locale" is not a shipping locale.',
    );

    // The target's declared pixel size and density drive the layout, so the run
    // reproduces the device class the store asks about — and the capture comes
    // out at exactly the declared dimensions whatever the host device is.
    tester.view.physicalSize = Size(
      target!.widthPx.toDouble(),
      target.heightPx.toDouble(),
    );
    tester.view.devicePixelRatio = target.devicePixelRatio;
    addTearDown(tester.view.reset);

    final midi = CaptureMidiService();
    addTearDown(midi.dispose);

    await tester.pumpWidget(
      RepaintBoundary(
        key: _rootKey,
        child: ProviderScope(
          overrides: captureOverrides(locale: _locale, midi: midi),
          child: const CymbraApp(),
        ),
      ),
    );
    await _settle(tester, frames: 20);

    final container = ProviderScope.containerOf(
      tester.element(find.byType(CymbraApp)),
    );

    // --- The seeded state actually took effect (task 2.7) --------------------
    expect(
      container.read(favoriteScoresProvider).valueOrNull,
      isNotEmpty,
      reason: 'the library would render its empty state',
    );
    expect(
      container.read(courseCompletionProvider).completed,
      isNotEmpty,
      reason: 'the learning path would show an untouched curriculum',
    );

    // --- 01 library ---------------------------------------------------------
    await _shot(tester, binding, target, 'library');

    // --- Open a real bundled score -----------------------------------------
    final card = find.byWidgetPredicate(
      (w) => w is ScoreCard && w.entry.id == _scoreId,
    );
    // In the phone viewport the library scrolls and the card is built lazily.
    await _bringIntoView(tester, card, delta: 200);
    expect(card, findsOneWidget, reason: 'the capture score left the catalog');
    await tester.tap(card);
    // Navigation + asset load + the real bridge parse/layout.
    await _settle(tester, frames: 30);

    // The pre-play setup modal opens over the player; start the session.
    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _settle(tester, frames: 10);

    expect(
      container.read(playerProvider).midiConnected,
      isTrue,
      reason: 'the player would show its "no MIDI device" chip',
    );

    // --- Perform the opening bars ------------------------------------------
    // Wait Mode (the default) freezes the playhead on each onset until the
    // expected keys are pressed, so the harness plays the piece exactly instead
    // of racing a clock: the gauge earns a real, high percentage and the run
    // stops wherever it is told to, never reaching the end-of-session summary
    // that covered the July captures.
    container.read(playerProvider.notifier).setPlaying(true);
    await _settle(tester, frames: 10);
    await _perform(tester, container, midi, onsets: 12);

    expect(
      container.read(performanceScorerProvider).syncPercent,
      greaterThan(50),
      reason: 'the score gauge would advertise a failed performance',
    );

    // --- 02 synthesia / 03 staff -------------------------------------------
    container.read(playerProvider.notifier).setMode(RenderMode.synthesia);
    await _settle(tester, frames: 10);
    await _shot(tester, binding, target, 'synthesia');

    container.read(playerProvider.notifier).setMode(RenderMode.staff);
    await _settle(tester, frames: 10);
    await _perform(tester, container, midi, onsets: 4);
    await _shot(tester, binding, target, 'staff');

    // Playback must not be left running behind the remaining surfaces.
    container.read(playerProvider.notifier).setPlaying(false);
    await _settle(tester, frames: 6);

    // --- 05 measure selection ----------------------------------------------
    // Long-press the transport's rewind control — the in-game entry point.
    await tester.longPress(find.byKey(const Key('transport-rewind')));
    await _settle(tester, frames: 15);
    expect(
      find.byKey(const Key('measure-select-canvas')),
      findsOneWidget,
      reason: 'measure selection did not open',
    );
    await _shot(tester, binding, target, 'measures');

    // Leave measure selection and the player, back to the library.
    final navigator = tester.state<NavigatorState>(
      find.byType(Navigator).first,
    );
    navigator.pop();
    await _settle(tester, frames: 10);
    navigator.pop();
    await _settle(tester, frames: 15);

    // --- 04 courses ---------------------------------------------------------
    // Reaching the capture score may have scrolled the courses section off a
    // phone viewport; scroll back up to it.
    final coursesEntry = find.byKey(const Key('courses-see-path'));
    await _bringIntoView(tester, coursesEntry, delta: -200);
    await tester.tap(coursesEntry);
    await _settle(tester, frames: 15);
    expect(
      find.byKey(const Key('path-screen')),
      findsOneWidget,
      reason: 'the learning path did not open',
    );
    await _shot(tester, binding, target, 'courses');

    // --- 06 drums / 07 drums_staff -----------------------------------------
    // Back to the library, then switch the instrument context the way a user
    // does — through the header switcher — which re-seeds the home on the
    // bundled drum grooves (change: add-instrument-context).
    navigator.pop();
    await _settle(tester, frames: 15);
    final switcher = find.byKey(const Key('instrument-switcher'));
    await _bringIntoView(tester, switcher, delta: -200);
    expect(
      switcher,
      findsOneWidget,
      reason: 'the drum feature is not visible — the capture override is gone',
    );
    await tester.tap(
      find.descendant(
        of: switcher,
        matching: find.byIcon(Icons.album_outlined),
      ),
    );
    await _settle(tester, frames: 15);

    final drumCard = find.byWidgetPredicate(
      (w) => w is ScoreCard && w.entry.id == _drumScoreId,
    );
    await _bringIntoView(tester, drumCard, delta: 200);
    expect(
      drumCard,
      findsOneWidget,
      reason: 'the capture drum score left the bundled catalog',
    );
    await tester.tap(drumCard);
    await _settle(tester, frames: 30);

    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _settle(tester, frames: 10);
    expect(
      container.read(playerProvider).isPercussion,
      isTrue,
      reason: 'the drum score did not route to the percussion presentation',
    );

    // Strokes arrive exactly as an electronic kit sends them — the same
    // performed-not-raced approach as the piano leg (changes:
    // add-drum-input-mapping, add-drum-scoring).
    container.read(playerProvider.notifier).setPlaying(true);
    await _settle(tester, frames: 10);
    await _perform(tester, container, midi, onsets: 10);
    expect(
      container.read(performanceScorerProvider).syncPercent,
      greaterThan(50),
      reason: 'the drum gauge would advertise a failed performance',
    );
    await _shot(tester, binding, target, 'drums');

    container.read(playerProvider.notifier).setMode(RenderMode.staff);
    await _settle(tester, frames: 10);
    await _perform(tester, container, midi, onsets: 4);
    await _shot(tester, binding, target, 'drums_staff');
    container.read(playerProvider.notifier).setPlaying(false);
    await _settle(tester, frames: 6);
  });
}

/// Plays the next [onsets] expected onsets, exactly as a player following the
/// score would: read the keys the app is waiting for, press them, release them.
Future<void> _perform(
  WidgetTester tester,
  ProviderContainer container,
  CaptureMidiService midi, {
  required int onsets,
}) async {
  for (var i = 0; i < onsets; i++) {
    final expected = container.read(playerProvider).expectedKeys;
    if (expected.isEmpty) return;
    for (final pitch in expected) {
      midi.press(pitch);
    }
    await _settle(tester, frames: 3);
    for (final pitch in expected) {
      midi.release(pitch);
    }
    await _settle(tester, frames: 3);
  }
}

/// Scrolls the library until [target] is built and on screen — a no-op when it
/// already is. [delta] is the scroll step: positive scrolls down the list.
///
/// The per-level card grids are themselves (non-scrolling) scrollables, so the
/// gesture is aimed at the library's own [CustomScrollView] rather than at
/// whichever scrollable happens to be first or last in the tree.
Future<void> _bringIntoView(
  WidgetTester tester,
  Finder target, {
  required double delta,
}) async {
  if (target.evaluate().isNotEmpty) return;
  await tester.scrollUntilVisible(
    target,
    delta,
    scrollable: find
        .descendant(
          of: find.byType(CustomScrollView),
          matching: find.byType(Scrollable),
        )
        .first,
  );
  await _settle(tester, frames: 6);
}

/// Pumps a fixed number of frames. The player animates continuously (falling
/// notes, the gauge), so `pumpAndSettle` would never return.
Future<void> _settle(WidgetTester tester, {int frames = 10}) async {
  for (var i = 0; i < frames; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// Renders the current frame at the target's declared size and reports it to the
/// driver under its manifest path.
///
/// Fails the run on a dimension mismatch rather than emitting an off-size image
/// the store would reject days later (task 5.1).
Future<void> _shot(
  WidgetTester tester,
  IntegrationTestWidgetsFlutterBinding binding,
  CaptureTarget target,
  String surface,
) async {
  final index = kCaptureSurfaces.indexOf(surface);
  expect(index, isNonNegative, reason: '"$surface" is not a declared surface');

  final boundary = tester.renderObject<RenderRepaintBoundary>(
    find.byKey(_rootKey),
  );
  final bytes = await tester.runAsync(() async {
    final image = await boundary.toImage(pixelRatio: target.devicePixelRatio);
    expect(
      [image.width, image.height],
      [target.widthPx, target.heightPx],
      reason: '$surface came out off-size for ${target.id}',
    );
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    return data!.buffer.asUint8List();
  });

  final report = binding.reportData ??= <String, dynamic>{};
  final screenshots = report['screenshots'] ??= <dynamic>[];
  (screenshots as List<dynamic>).add(<String, dynamic>{
    'screenshotName': captureRelativeName(target, _locale, index),
    'bytes': bytes!.toList(),
  });
}
