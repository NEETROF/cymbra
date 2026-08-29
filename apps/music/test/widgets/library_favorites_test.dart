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

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/screens/library_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/screens/score_hub_screen.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/instrument_context.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/courses_section.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

class _FakeCatalog implements CatalogService {
  @override
  Future<Uint8List> getOfflineCacheKey() async => Uint8List(0);
  @override
  Future<CatalogAccessState?> dailyAccess() async => null;
  @override
  Future<CatalogAccessState?> unlockForToday(String catalogId) async => null;
  _FakeCatalog(this.saved, {this.failure});
  final List<CatalogHit> saved;
  final List<String> removed = [];

  /// When set, `listSaved` throws it — the favorites list in error, which the
  /// home has to render as something other than a bare failure.
  final Object? failure;

  @override
  Future<List<CatalogHit>> listSaved() async {
    if (failure != null) throw failure!;
    return saved.where((h) => !removed.contains(h.id)).toList();
  }

  @override
  Future<void> remove(String catalogId) async => removed.add(catalogId);
  @override
  Future<void> save(String catalogId) async {}
  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    CatalogFilters filters = const CatalogFilters(),
    int limit = 20,
    int offset = 0,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0, total: 0);
  @override
  Future<ScoreBytesResult> fetchScoreBytes(
    String catalogId, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
  @override
  Future<Uint8List> ratingPreviewBytes(String catalogId) async => Uint8List(0);
  @override
  Future<CatalogSearchPage> ratingDeck({
    int limit = 20,
    int offset = 0,
    ScoreInstrument? instrument,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0, total: 0);
}

class _FakeUpload implements ScoreUploadService {
  @override
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async {}

  _FakeUpload(this.mine);
  final List<ContributedScore> mine;
  final List<String> favoritedOff = [];

  @override
  Future<List<ContributedScore>> listMyScores() async => mine;

  @override
  Future<List<ContributedScore>> listMyScoresInCollection(String collectionId) async =>
      const [];

  @override
  Future<List<ScoreCollection>> listCollections() async => const [];

  @override
  Future<ScoreCollection> createCollection(String name) async =>
      ScoreCollection(id: 'c1', name: name, createdAt: DateTime.utc(2026));

  @override
  Future<void> renameCollection(String id, String name) async {}

  @override
  Future<void> deleteCollection(String id) async {}

  @override
  Future<void> addToCollection(String collectionId, String scoreId) async {}

  @override
  Future<void> removeFromCollection(String collectionId, String scoreId) async {}

  @override
  Future<UploadAllowance> uploadAllowance() async => const UploadAllowance(
    remaining: 100,
    max: 100,
    windowDays: 7,
    upgradeRaisesLimit: false,
  );
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async {
    if (!favorite) favoritedOff.add(id);
  }

  @override
  Future<ScoreBytesResult> fetchScoreBytes(
    String id, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) async => throw UnimplementedError();
}

/// One-lesson course catalogue, enough for the home's continue card.
class _FakeCourses implements CourseCatalogService {
  @override
  Future<List<CourseListing>> listCourses() async => const [
    CourseListing(
      id: 'sol-u1-01',
      schemaVersion: 2,
      track: 'solfege',
      level: 'beginner',
      unit: 'u1',
      unitTitle: {'en': 'First notes'},
      sortOrder: 101,
      title: {'en': 'Reading the staff'},
    ),
  ];

  @override
  Future<String?> getCourseManifestJson(String id) async =>
      '{"schemaVersion":1,"id":"sol-u1-01","blocks":['
      '{"type":"text","text":{"en":"hello"}}]}';
}

CatalogHit _saved(String id, String title, {ScoreInstrument? instrument}) =>
    CatalogHit(
      id: id,
      title: title,
      composer: 'Composer',
      level: PracticeLevel.beginner,
      license: 'CC-BY-4.0',
      source: 'pdmx',
      instrument: instrument,
    );

ContributedScore _upload(String id, String title, {bool favorite = true}) =>
    ContributedScore(
      id: id,
      level: PracticeLevel.intermediate,
      createdAt: DateTime.utc(2026, 5, 1),
      measureCount: 8,
      timeSig: '4/4',
      keyFifths: 0,
      title: title,
      composer: 'Me',
      favorite: favorite,
    );

const _bundled = [
  CatalogEntry(
    id: 'b1',
    title: 'Bundled Piece',
    composer: 'X',
    assetPath: 'assets/scores/beginner/b1.musicxml',
    level: PracticeLevel.beginner,
  ),
];

class _FakeConnectivity extends Fake implements ConnectivityService {
  _FakeConnectivity(this.online);
  bool online;
  final _status = StreamController<bool>.broadcast();
  @override
  Stream<void> get onOnline => const Stream.empty();
  @override
  Stream<bool> get onlineStatus => _status.stream;
  @override
  Future<bool> isOnline() async => online;
  @override
  Future<bool> isDefinitelyOffline() async => !(online);

  /// Flip connectivity live (as `connectivity_plus` would on a Wi-Fi change).
  void setOnline(bool value) {
    online = value;
    _status.add(value);
  }

  void dispose() => _status.close();
}

ProviderContainer _container(
  _FakeCatalog catalog,
  _FakeUpload upload, {
  bool signedIn = true,
  bool withCourses = false,
  bool online = true,
  bool drumsVisible = false,
  List<CatalogEntry> bundled = _bundled,
  FakePreferencesService? prefs,
  OfflineScoreCache? cache,
  _FakeConnectivity? connectivity,
}) {
  final c = ProviderContainer(
    overrides: [
      if (withCourses)
        courseCatalogServiceProvider.overrideWithValue(_FakeCourses()),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      drumsEnabledProvider.overrideWithValue(drumsVisible),
      scoreCatalogProvider.overrideWithValue(bundled),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      catalogServiceProvider.overrideWithValue(catalog),
      scoreUploadServiceProvider.overrideWithValue(upload),
      canUseOnlineServicesProvider.overrideWithValue(signedIn),
      connectivityServiceProvider.overrideWithValue(
        connectivity ?? _FakeConnectivity(online),
      ),
      // In-memory offline cache so eviction on remove/delete is instant (the real
      // impl touches path_provider, which isn't available in a widget test).
      offlineScoreCacheProvider.overrideWithValue(
        cache ?? InMemoryOfflineScoreCache(),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer c, {
  Size size = const Size(1400, 900),
}) async {
  await tester.binding.setSurfaceSize(size);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(const LibraryScreen(), locale: const Locale('en')),
    ),
  );
  for (var i = 0; i < 8; i++) {
    await tester.pump(const Duration(milliseconds: 20));
  }
}

Future<void> _teardown(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
}

void main() {
  testWidgets('signed in: favorites by level, no bundled, no non-favorites', (
    tester,
  ) async {
    final c = _container(
      _FakeCatalog([_saved('c1', 'Saved Piece')]),
      _FakeUpload([
        _upload('u1', 'Fav Upload'),
        _upload('u2', 'Hidden Upload', favorite: false),
      ]),
    );
    await _pump(tester, c);

    expect(find.text('Saved Piece'), findsOneWidget); // saved catalog
    expect(find.text('Fav Upload'), findsOneWidget); // favorited upload
    expect(find.text('Hidden Upload'), findsNothing); // not favorited
    expect(
      find.text('Bundled Piece'),
      findsNothing,
    ); // no bundled when signed in
    // Level headers (beginner for the saved, intermediate for the upload).
    expect(find.text('Beginner'), findsOneWidget);
    expect(find.text('Intermediate'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('signed in with no favorites shows the hub call-to-action', (
    tester,
  ) async {
    final c = _container(_FakeCatalog(const []), _FakeUpload(const []));
    await _pump(tester, c);

    expect(find.text('No favorites yet'), findsOneWidget);
    expect(
      find.widgetWithText(FilledButton, 'Browse the Score Hub'),
      findsOneWidget,
    );
    await _teardown(tester);
  });

  testWidgets('phone: courses and favorites scroll as one block', (
    tester,
  ) async {
    final c = _container(
      _FakeCatalog([
        for (var i = 0; i < 6; i++) _saved('c$i', 'Saved Piece $i'),
      ]),
      _FakeUpload([
        for (var i = 0; i < 4; i++) _upload('u$i', 'Fav Upload $i'),
      ]),
      withCourses: true,
    );
    // A phone viewport: the courses card alone would leave the favorites a
    // sliver of screen if each had its own scroll view.
    await _pump(tester, c, size: const Size(390, 700));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    expect(find.byKey(const Key('courses-continue-card')), findsOneWidget);

    // One drag on the page carries the courses section away and brings the
    // lower level section up — proof of a single vertical scroll.
    expect(find.text('Intermediate'), findsNothing); // still below the fold
    await tester.drag(find.byType(CustomScrollView), const Offset(0, -900));
    await tester.pump();

    expect(find.byKey(const Key('courses-continue-card')), findsNothing);
    expect(find.text('Intermediate'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('signed out shows the bundled demo catalog', (tester) async {
    final c = _container(
      _FakeCatalog(const []),
      _FakeUpload(const []),
      signedIn: false,
    );
    await _pump(tester, c);
    expect(find.text('Bundled Piece'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('heart un-favorites an upload (keeps it)', (tester) async {
    final upload = _FakeUpload([_upload('u1', 'Fav Upload')]);
    final c = _container(_FakeCatalog(const []), upload);
    await _pump(tester, c);
    expect(find.text('Fav Upload'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.favorite));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(upload.favoritedOff, ['u1']);
    await _teardown(tester);
  });

  testWidgets('heart removes a saved catalog score from the library', (
    tester,
  ) async {
    final catalog = _FakeCatalog([_saved('c1', 'Saved Piece')]);
    final c = _container(catalog, _FakeUpload(const []));
    await _pump(tester, c);
    expect(find.text('Saved Piece'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.favorite));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(catalog.removed, ['c1']);
    expect(find.text('Saved Piece'), findsNothing);
    await _teardown(tester);
  });

  testWidgets('offline: uncached favorites are marked "not available offline'
      '", cached ones are not', (tester) async {
    // c1's bytes are cached (playable offline); c2's are not.
    final cache = InMemoryOfflineScoreCache();
    await cache.write('catalog:c1', Uint8List.fromList(const [1]), etag: 'e');
    final c = _container(
      _FakeCatalog([
        _saved('c1', 'Cached Piece'),
        _saved('c2', 'Uncached Piece'),
      ]),
      _FakeUpload(const []),
      online: false,
      cache: cache,
    );
    await _pump(tester, c);

    // Both favorites are listed offline (from the live fetch here; the snapshot
    // fallback is covered in favorite_scores_test).
    expect(find.text('Cached Piece'), findsOneWidget);
    expect(find.text('Uncached Piece'), findsOneWidget);
    // Only the uncached one carries the "not available offline" badge.
    expect(find.text('Not available offline'), findsOneWidget);
    expect(find.byIcon(Icons.cloud_off), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('home re-marks favorites live when Wi-Fi drops (no reload)', (
    tester,
  ) async {
    final conn = _FakeConnectivity(true); // start online
    addTearDown(conn.dispose);
    final c = _container(
      _FakeCatalog([_saved('c1', 'Uncached Piece')]),
      _FakeUpload(const []),
      connectivity: conn,
    );
    await _pump(tester, c);
    // Online: nothing is flagged offline.
    expect(find.text('Not available offline'), findsNothing);

    // Wi-Fi drops → the connectivity stream emits; the home re-marks the
    // uncached favorite without any hot reload / manual refresh.
    conn.setOnline(false);
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(find.text('Not available offline'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('signed in under the drums context: favorites filter to '
      'percussion, the bundled grooves stay OUT, courses step aside', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: jsonEncode({
        'context': 'drums',
        'choiceOffered': true,
      }),
    });
    const drumEntry = CatalogEntry(
      id: 'groove-1',
      title: 'Bundled Groove',
      composer: 'Cymbra',
      assetPath: 'assets/scores/beginner/groove1.musicxml',
      level: PracticeLevel.beginner,
      instrument: ScoreInstrument.percussion,
    );
    final c = _container(
      _FakeCatalog([
        _saved('c1', 'Piano Favorite'),
        _saved('c2', 'Drum Favorite', instrument: ScoreInstrument.percussion),
      ]),
      _FakeUpload(const []),
      withCourses: true,
      drumsVisible: true,
      prefs: prefs,
      bundled: [..._bundled, drumEntry],
    );
    await _pump(tester, c);

    // The drum favorite alone. The piano favorite, the bundled piano piece and
    // the (keyboard-only) courses card step aside — and so does the bundled
    // groove, which is in the catalog passed to this container: signed in, the
    // home shows what the account holds, never a demo score.
    expect(find.text('Drum Favorite'), findsOneWidget);
    expect(find.text('Bundled Groove'), findsNothing);
    expect(find.text('Piano Favorite'), findsNothing);
    expect(find.text('Bundled Piece'), findsNothing);
    expect(find.byType(CoursesSection), findsNothing);

    // Switching back re-seeds the signed-in home the other way.
    await tester.tap(
      find.descendant(
        of: find.byKey(const Key('instrument-switcher')),
        matching: find.text('Piano'),
      ),
    );
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(find.text('Piano Favorite'), findsOneWidget);
    expect(find.byType(CoursesSection), findsOneWidget);
    expect(find.text('Drum Favorite'), findsNothing);
    await _teardown(tester);
  });

  testWidgets('signed in, drums context, nothing to show: the explicit '
      'invitation replaces the bare empty state', (tester) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: jsonEncode({
        'context': 'drums',
        'choiceOffered': true,
      }),
    });
    final c = _container(
      _FakeCatalog([_saved('c1', 'Piano Favorite')]),
      _FakeUpload(const []),
      drumsVisible: true,
      prefs: prefs,
      bundled: _bundled, // no percussion anywhere
    );
    await _pump(tester, c);

    expect(find.byKey(const Key('drums-empty-switch')), findsOneWidget);
    expect(find.text('Piano Favorite'), findsNothing);
    await _teardown(tester);
  });

  testWidgets('the drums invitation leads to the catalog, not only back to '
      'the keyboard', (tester) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: jsonEncode({
        'context': 'drums',
        'choiceOffered': true,
      }),
    });
    final c = _container(
      _FakeCatalog([_saved('c1', 'Piano Favorite')]),
      _FakeUpload(const []),
      drumsVisible: true,
      prefs: prefs,
      bundled: _bundled, // no percussion anywhere
    );
    await _pump(tester, c);

    // This state is where a signed-in drummer with nothing saved lands, so its
    // primary action has to be a way forward. Tapping it opens the hub; the
    // context is untouched, which is what separates it from the switch below.
    await tester.tap(find.byKey(const Key('drums-empty-browse')));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(find.byType(ScoreHubScreen), findsOneWidget);
    expect(prefs.store[InstrumentContext.prefsKey], contains('drums'));
    await _teardown(tester);
  });

  testWidgets('favorites failing under drums: the invitation stands in for the '
      'error, and its way forward still works', (tester) async {
    final c = _container(
      _FakeCatalog(const [], failure: StateError('offline')),
      _FakeUpload(const []),
      drumsVisible: true,
      prefs: FakePreferencesService({
        InstrumentContext.prefsKey: jsonEncode({
          'context': 'drums',
          'choiceOffered': true,
        }),
      }),
      bundled: _bundled,
    );
    await _pump(tester, c);

    // The list could not be read at all, yet the drummer gets the same way
    // forward as when it is merely empty — which is what makes the failure
    // unremarkable rather than a wall.
    expect(find.byKey(const Key('drums-empty-switch')), findsOneWidget);
    await tester.tap(find.byKey(const Key('drums-empty-browse')));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    // A real destination, not a button drawn on an error: the catalog does not
    // depend on the favorites read that just failed.
    expect(find.byType(ScoreHubScreen), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('favorites failing under the keyboard: that context keeps its '
      'own empty state, never the drums invitation', (tester) async {
    final c = _container(
      _FakeCatalog(const [], failure: StateError('offline')),
      _FakeUpload(const []),
      drumsVisible: true,
      bundled: _bundled,
    );
    await _pump(tester, c);

    expect(find.byKey(const Key('drums-empty-browse')), findsNothing);
    expect(find.byKey(const Key('drums-empty-switch')), findsNothing);
    await _teardown(tester);
  });
}
