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

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:music/main.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/services/token_store.dart';
import 'package:music/src/rust/frb_generated.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import 'support/fixture_score.dart';

/// Optional pause (ms) held between steps so a *visible* `flutter drive` run is
/// watchable. Zero by default — CI and `flutter test` pass no `--dart-define`,
/// so the gate stays fast — and set by `melos run integration`. Override with
/// `--dart-define=WATCH_MS=1500`.
const int _watchMs = int.fromEnvironment('WATCH_MS');

/// Pins the app to English with no persisted language, so localized strings are
/// deterministic on any host (the dev machine may report a non-English locale or
/// carry a persisted language choice from the real app).
List<Override> _englishLocaleOverrides() => [
  deviceLocaleProvider.overrideWithValue(const Locale('en')),
  preferencesServiceProvider.overrideWithValue(const _EmptyPrefs()),
];

/// An empty [PreferencesService] (nothing persisted), so [AppLocale] resolves to
/// the pinned device locale instead of any language stored by the real app.
class _EmptyPrefs implements PreferencesService {
  const _EmptyPrefs();
  @override
  Future<String?> getString(String key) async => null;
  @override
  Future<void> setString(String key, String value) async {}
  @override
  Future<void> remove(String key) async {}
}

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
          // Pin English + empty prefs so localized strings are deterministic
          // regardless of the host device locale or any persisted language.
          ..._englishLocaleOverrides(),
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

  testWidgets('contribution wizard: import → verify → confirm → submit', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    final upload = _RecordingUpload();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          scoreCatalogProvider.overrideWithValue(const []),
          tokenStoreProvider.overrideWithValue(const _GuestTokenStore()),
          // Pin English + empty prefs so the localized difficulty labels are
          // deterministic regardless of the host device locale or persisted language.
          ..._englishLocaleOverrides(),
          // Signed-in-equivalent so the contribution entry point is shown; the
          // picker and backend upload are faked, but validation/preview run on
          // the REAL Rust bridge (the point of an integration test).
          canUseOnlineServicesProvider.overrideWithValue(true),
          filePickerProvider.overrideWithValue(const _FixturePicker()),
          scoreUploadServiceProvider.overrideWithValue(upload),
        ],
        child: const CymbraApp(),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    // Open the Score Hub from the library AppBar, then the upload wizard from
    // the hub AppBar (the contribute action now lives in the hub).
    await tester.tap(find.byTooltip('Hub de partitions'));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    await watch(tester);
    await tester.tap(find.byTooltip('Contribuer une partition'));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    await watch(tester);

    // Import: pick the fixture; the real bridge validates it.
    await tester.tap(find.text('Choisir un fichier'));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.textContaining('est valide'), findsOneWidget);
    await watch(tester);

    // Attest, then advance to the real engraved preview.
    await tester.tap(find.text('J\'en suis l\'auteur'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Vérifier'));
    for (var i = 0; i < 20; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    // Server-parity metadata derived by the real parser (Ode to Joy / Beethoven).
    expect(find.text('Informations détectées (lecture seule)'), findsOneWidget);
    expect(find.textContaining('Ode to Joy'), findsWidgets);
    await watch(tester);

    // Confirm: choose a difficulty and submit.
    await tester.tap(find.text('Continuer'));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.text('Niveau de difficulté'), findsOneWidget);
    await tester.tap(find.text('Intermediate'));
    await tester.pump();
    await tester.tap(find.text('Envoyer'));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    // Submitted the real bytes + chosen inputs; success screen shown.
    expect(upload.uploads, hasLength(1));
    expect(upload.uploads.single.level, PracticeLevel.intermediate);
    expect(upload.uploads.single.basis, RightsBasis.author);
    expect(find.text('Partition ajoutée à vos contributions.'), findsOneWidget);
    await watch(tester);
  });
}

/// A [FilePickerService] that returns the MusicXML fixture bytes (no native
/// picker), so the wizard's real FFI validation runs against known content.
class _FixturePicker implements FilePickerService {
  const _FixturePicker();

  @override
  Future<PickedScoreFile?> pickScore() async => PickedScoreFile(
    name: 'ode-to-joy.musicxml',
    bytes: Uint8List.fromList(utf8.encode(kFixtureScoreXml)),
  );
}

/// A [ScoreUploadService] that records the submit inputs instead of hitting gRPC.
class _RecordingUpload implements ScoreUploadService {
  final List<({PracticeLevel level, RightsBasis basis, int len})> uploads = [];

  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) async {
    uploads.add((level: level, basis: rightsBasis, len: data.length));
    return ContributedScore(
      id: 'contrib-1',
      level: level,
      createdAt: DateTime.utc(2026),
      measureCount: 2,
      timeSig: '4/4',
      keyFifths: 0,
      title: 'Ode to Joy',
    );
  }

  @override
  Future<List<ContributedScore>> listMyScores() async => const [];
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async {}

  @override
  Future<Uint8List> fetchBytes(String id) async => Uint8List(0);
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
