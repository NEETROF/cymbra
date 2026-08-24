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
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/services/token_store.dart';
import 'package:music/src/rust/frb_generated.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/onboarding_notifier.dart';
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
///
/// [firstRunDone] seeds the first-run flags (change: add-welcome-onboarding) so
/// a test whose subject is *not* onboarding boots straight into the app; pass
/// `false` to exercise the real first-launch sequence.
List<Override> _englishLocaleOverrides({bool firstRunDone = true}) => [
  deviceLocaleProvider.overrideWithValue(const Locale('en')),
  preferencesServiceProvider.overrideWithValue(
    _MemoryPrefs(
      firstRunDone
          ? {
              Onboarding.languagePrefsKey: 'true',
              Onboarding.welcomePrefsKey: 'true',
            }
          : null,
    ),
  ),
];

/// An in-memory [PreferencesService], so [AppLocale] resolves to the pinned
/// device locale instead of any language stored by the real app, and the
/// first-run flow can record its own steps without touching device storage.
class _MemoryPrefs implements PreferencesService {
  _MemoryPrefs([Map<String, String>? seed]) : _store = {...?seed};

  final Map<String, String> _store;

  @override
  Future<String?> getString(String key) async => _store[key];
  @override
  Future<void> setString(String key, String value) async => _store[key] = value;
  @override
  Future<void> remove(String key) async => _store.remove(key);
}

/// A connectivity seam that never touches `connectivity_plus`. On the headless
/// Linux CI the real plugin talks to NetworkManager over D-Bus, which is absent,
/// so its listener throws a late async error that fails the test *after* it
/// completed. This fake (always online, no streams) keeps the run deterministic —
/// connectivity is not what these tests exercise.
class _FakeConnectivity implements ConnectivityService {
  const _FakeConnectivity();
  @override
  Stream<void> get onOnline => const Stream<void>.empty();
  @override
  Stream<bool> get onlineStatus => const Stream<bool>.empty();
  @override
  Future<bool> isOnline() async => true;

  @override
  Future<bool> isDefinitelyOffline() async => false;
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
          // Avoid the real connectivity_plus plugin (no D-Bus on headless CI).
          connectivityServiceProvider.overrideWithValue(
            const _FakeConnectivity(),
          ),
          // Pin English + in-memory prefs so localized strings are deterministic
          // regardless of the host device locale or any persisted language. This
          // test starts from a genuine first launch (nothing recorded yet).
          ..._englishLocaleOverrides(firstRunDone: false),
        ],
        child: const CymbraApp(),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    // First launch opens on the language step, before anything else (change:
    // add-welcome-onboarding). English is pre-selected from the pinned device
    // locale, so continuing is one tap.
    expect(find.text('Choose your language'), findsOneWidget);
    await watch(tester);
    await tester.tap(find.byKey(const Key('onboarding-language-continue')));
    for (var i = 0; i < 5; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    // The welcome is for a user with no session at all — this one resumed a
    // guest session, so it lands straight in the score library. Pick the
    // fixture score.
    expect(find.text('Cymbra — Score Library'), findsOneWidget);
    await watch(tester);
    final entry = find.text('Ode to Joy (theme)');
    expect(entry, findsOneWidget);
    await tester.tap(entry);

    // Let navigation + asset load + the real bridge parse/layout settle.
    for (var i = 0; i < 25; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    // The pre-play setup modal opens over the player — validate it to start.
    final startSetup = find.widgetWithText(FilledButton, 'Play');
    expect(startSetup, findsOneWidget);
    await watch(tester);
    await tester.tap(startSetup);
    for (var i = 0; i < 5; i++) {
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

    // Cycle the three rendering modes: Cascade → Staff → Partition → Cascade.
    await tester.tap(find.text('Staff'));
    await tester.pump();
    await watch(tester);
    await tester.tap(find.text('Partition'));
    await tester.pump(const Duration(milliseconds: 100));
    await watch(tester);
    await tester.tap(find.text('Cascade'));
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
          // Avoid the real connectivity_plus plugin (no D-Bus on headless CI).
          connectivityServiceProvider.overrideWithValue(
            const _FakeConnectivity(),
          ),
          // Pin English so the localized difficulty labels are deterministic
          // regardless of the host device locale or persisted language, and mark
          // the first run as done — the wizard, not onboarding, is the subject.
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
    // the hub AppBar (the contribute action now lives in the hub). Tooltips are
    // localized and the app is pinned to English here, so match the EN strings.
    await tester.tap(find.byTooltip('Score Hub'));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    await watch(tester);
    await tester.tap(find.byTooltip('Contribute a score'));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    await watch(tester);

    // Import: pick the fixture; the real bridge validates it.
    await tester.tap(find.text('Choose a file'));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.textContaining('is valid'), findsOneWidget);
    await watch(tester);

    // Attest, then advance to the real engraved preview.
    await tester.tap(find.text('I am the author'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Verify'));
    for (var i = 0; i < 20; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    // Server-parity metadata derived by the real parser (Ode to Joy / Beethoven).
    expect(find.text('Detected information (read-only)'), findsOneWidget);
    expect(find.textContaining('Ode to Joy'), findsWidgets);
    await watch(tester);

    // Confirm: choose a difficulty and submit.
    await tester.tap(find.text('Continue'));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.text('Difficulty level'), findsOneWidget);
    await tester.tap(find.text('Intermediate'));
    await tester.pump();
    await tester.tap(find.text('Submit'));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    // Submitted the real bytes + chosen inputs; success screen shown.
    expect(upload.uploads, hasLength(1));
    expect(upload.uploads.single.level, PracticeLevel.intermediate);
    expect(upload.uploads.single.basis, RightsBasis.author);
    expect(find.text('Score added to your contributions.'), findsOneWidget);
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
  @override
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async {}

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
  Future<ScoreBytesResult> fetchScoreBytes(
    String id, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
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
