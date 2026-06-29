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

// End-to-end test driving the REAL app: it builds and loads the native Rust
// library (cargokit) and exercises the genuine flutter_rust_bridge path
// (RustLib.init, parse_musicxml, layout_systems, midiEventStream). No MIDI
// hardware is required — the computer-keyboard fallback covers the input path.
// Run locally with `flutter test integration_test -d macos`; in CI it runs on
// the Linux desktop engine under Xvfb.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:music/main.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/token_store.dart';
import 'package:music/src/rust/frb_generated.dart';
import 'package:music/state/score_catalog.dart';

import 'support/fixture_score.dart';

/// Optional pause (ms) held between steps so a *visible* `flutter drive` run is
/// watchable. Zero by default — CI and `flutter test` pass no `--dart-define`,
/// so the gate stays fast — and set by `melos run integration`. Override with
/// `--dart-define=WATCH_MS=1500`.
const int _watchMs = int.fromEnvironment('WATCH_MS');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  /// Real-time pause so the current screen is visible (no-op when [_watchMs]==0).
  Future<void> watch(WidgetTester tester) => _watchMs > 0
      ? tester.pump(Duration(milliseconds: _watchMs))
      : Future.value();

  testWidgets('library → score → plays, keyboard input, render modes', (
    tester,
  ) async {
    // The desktop/tablet-first UI is laid out for a realistic viewport; pin a
    // desktop size so the headless CI window (defaults to ~800x600) doesn't
    // overflow the dense top/transport bars.
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    // Drive a test-owned score fixture (not the app's shipping assets, which
    // change independently) — still parsed/laid out by the real Rust bridge.
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          scoreCatalogProvider.overrideWithValue(const [kFixtureCatalogEntry]),
          scoreAssetSourceProvider.overrideWithValue(
            const FixtureScoreAssetSource(),
          ),
          // Boot straight into the library: a guest session skips the entry
          // screen, and the in-memory store keeps the test off platform secure
          // storage (no Keychain/libsecret keyring in headless CI).
          tokenStoreProvider.overrideWithValue(const _GuestTokenStore()),
        ],
        child: const CymbraApp(),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    // Boots into the score library; pick the fixture score.
    expect(find.text('Cymbra — Score Library'), findsOneWidget);
    await watch(tester);
    final entry = find.text('Ode to Joy (theme)');
    expect(entry, findsOneWidget);
    await tester.tap(entry);

    // Let navigation + asset load + the real bridge parse/layout settle.
    for (var i = 0; i < 25; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    // Player chrome for the loaded score (parsed over the bridge).
    expect(find.text('Cymbra Music'), findsWidgets);
    expect(find.textContaining('Ode to Joy'), findsWidgets);
    await watch(tester);

    // Transport: play.
    expect(find.byIcon(Icons.play_arrow), findsOneWidget);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    expect(find.byIcon(Icons.pause), findsOneWidget);
    await watch(tester);

    // Computer-keyboard fallback: press and release C4.
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyA);
    await tester.pump();
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyA);
    await tester.pump();

    // Cycle the three rendering modes: Synthesia → Staff → Partition → Synthesia.
    await tester.tap(find.text('Staff'));
    await tester.pump();
    await watch(tester);
    await tester.tap(find.text('Partition'));
    await tester.pump(const Duration(milliseconds: 100));
    await watch(tester);
    await tester.tap(find.text('Synthesia'));
    await tester.pump();
    await watch(tester);
  });
}

/// In-memory [TokenStore] reporting a persisted guest choice, so [SessionGate]
/// routes straight to the library without touching platform secure storage.
class _GuestTokenStore implements TokenStore {
  const _GuestTokenStore();

  @override
  Future<bool> isGuest() async => true;

  @override
  Future<StoredTokens?> readTokens() async => null;

  @override
  Future<void> writeTokens(StoredTokens tokens) async {}

  @override
  Future<void> setGuest() async {}

  @override
  Future<void> clear() async {}
}
