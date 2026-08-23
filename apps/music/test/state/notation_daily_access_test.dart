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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/state/catalog_daily_access_notifier.dart';
import 'package:music/state/curator_profile_notifier.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/saved_catalog_scores.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/usage_tracking_notifier.dart';

import '../support/notation_fakes.dart';

/// The freemium daily-access gate seen from the app (change:
/// add-score-daily-access-rewards): the catalog bytes fetch answers with an
/// access state — served, or LOCKED (no bytes, not "unchanged").
class _FakeCatalog extends Fake implements CatalogService {
  _FakeCatalog(
    this.bytes, {
    this.locked = false,
    this.serverEtag = 'e',
    this.error,
    this.unlockError,
    this.saved = const [],
  });

  final Uint8List bytes;
  final bool locked;
  final String serverEtag;
  final Object? error;
  final Object? unlockError;
  final List<CatalogHit> saved;
  int fetchCalls = 0;
  String? lastIfNoneMatch;
  final List<String> unlockCalls = [];

  CatalogAccessState _state({required bool locked}) => CatalogAccessState(
    enabled: true,
    locked: locked,
    freeQuota: 3,
    freeUsed: 3,
    resetsAtMs: DateTime.utc(2026, 8, 16).millisecondsSinceEpoch,
    daySlotCost: 20,
    spendableBalance: 25,
    subscriber: false,
    upsell: true,
    openedToday: const ['a', 'b', 'c'],
  );

  @override
  Future<ScoreBytesResult> fetchScoreBytes(
    String id, {
    String? ifNoneMatch,
  }) async {
    fetchCalls++;
    lastIfNoneMatch = ifNoneMatch;
    if (error != null) throw error!;
    if (locked) {
      return ScoreBytesResult(
        data: null,
        etag: serverEtag,
        unchanged: false,
        access: _state(locked: true),
      );
    }
    if (ifNoneMatch != null && ifNoneMatch == serverEtag) {
      return ScoreBytesResult(
        data: null,
        etag: serverEtag,
        unchanged: true,
        access: _state(locked: false),
      );
    }
    return ScoreBytesResult(
      data: bytes,
      etag: serverEtag,
      unchanged: false,
      access: _state(locked: false),
    );
  }

  @override
  Future<List<CatalogHit>> listSaved() async => saved;

  @override
  Future<CatalogAccessState?> dailyAccess() async => _state(locked: false);

  @override
  Future<CatalogAccessState?> unlockForToday(String catalogId) async {
    unlockCalls.add(catalogId);
    if (unlockError != null) throw unlockError!;
    return CatalogAccessState(
      enabled: true,
      locked: false,
      freeQuota: 3,
      freeUsed: 3,
      resetsAtMs: DateTime.utc(2026, 8, 16).millisecondsSinceEpoch,
      daySlotCost: 20,
      spendableBalance: 5,
      subscriber: false,
      upsell: true,
      openedToday: const ['a', 'b', 'c', 'x'],
      paidToday: const ['x'],
    );
  }
}

class _FakeConnectivity extends Fake implements ConnectivityService {
  _FakeConnectivity(this.online);
  final bool online;
  @override
  Stream<void> get onOnline => const Stream.empty();
  @override
  Stream<bool> get onlineStatus => const Stream.empty();
  @override
  Future<bool> isOnline() async => online;
  @override
  Future<bool> isDefinitelyOffline() async => !(online);
}

Future<void> _flush() async {
  for (var i = 0; i < 8; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

const _hit = CatalogHit(id: 'x', license: 'CC0', source: 'pdmx', title: 'X');

CatalogEntry _catalog() => catalogEntryFromHit(_hit);

void main() {
  Uint8List bytes() => Uint8List.fromList(const [1, 2, 3, 4]);

  ProviderContainer build({
    required InMemoryOfflineScoreCache cache,
    required _FakeCatalog catalog,
    bool online = true,
  }) {
    final c = ProviderContainer(
      overrides: [
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
        offlineScoreCacheProvider.overrideWithValue(cache),
        catalogServiceProvider.overrideWithValue(catalog),
        connectivityServiceProvider.overrideWithValue(
          _FakeConnectivity(online),
        ),
        // Signed in for the daily-access read; usage telemetry stays off (its
        // enablement gate reads the same provider through the consent chain).
        canUseOnlineServicesProvider.overrideWithValue(true),
        usageCollectionKillSwitchProvider.overrideWithValue(false),
      ],
    );
    addTearDown(c.dispose);
    c.listen(notationProvider, (_, _) {}, fireImmediately: true);
    return c;
  }

  test('a locked open is a typed failure and never plays', () async {
    final cache = InMemoryOfflineScoreCache();
    final catalog = _FakeCatalog(bytes(), locked: true);
    final c = build(cache: cache, catalog: catalog);

    c.read(selectedScoreProvider.notifier).select(_catalog());
    await _flush();

    final n = c.read(notationProvider);
    expect(n.failure, ScoreLoadFailure.locked);
    expect(n.hasDocument, isFalse);
    // Nothing cached from a refused open.
    expect(await cache.has('catalog:x'), isFalse);
    // The numbers went to the daily-access provider (locked bit stripped).
    final access = c.read(catalogDailyAccessProvider).valueOrNull;
    expect(access, isNotNull);
    expect(access!.locked, isFalse);
    expect(access.freeLeft, 0);
    expect(access.daySlotCost, 20);
  });

  test('online, a cached favourite is decided by the server first — locked '
      'does not play the cached copy (kept)', () async {
    final cache = InMemoryOfflineScoreCache();
    await cache.write('catalog:x', bytes(), etag: 'e');
    final catalog = _FakeCatalog(bytes(), locked: true, saved: const [_hit]);
    final c = build(cache: cache, catalog: catalog, online: true);

    c.read(selectedScoreProvider.notifier).select(_catalog());
    await _flush();

    expect(catalog.fetchCalls, 1, reason: 'the conditional fetch ran first');
    expect(catalog.lastIfNoneMatch, 'e');
    expect(c.read(notationProvider).failure, ScoreLoadFailure.locked);
    expect(c.read(notationProvider).hasDocument, isFalse);
    expect(await cache.has('catalog:x'), isTrue, reason: 'access is per-day');
  });

  test('online, a cached favourite within quota plays from cache after the '
      'server said unchanged', () async {
    final cache = InMemoryOfflineScoreCache();
    await cache.write('catalog:x', bytes(), etag: 'e');
    final catalog = _FakeCatalog(bytes(), serverEtag: 'e', saved: const [_hit]);
    final c = build(cache: cache, catalog: catalog, online: true);

    c.read(selectedScoreProvider.notifier).select(_catalog());
    await _flush();

    expect(catalog.fetchCalls, 1);
    expect(c.read(notationProvider).hasDocument, isTrue);
    expect((await cache.read('catalog:x'))!.etag, 'e');
  });

  test('offline, a cached favourite plays with no network request', () async {
    final cache = InMemoryOfflineScoreCache();
    await cache.write('catalog:x', bytes(), etag: 'e');
    final catalog = _FakeCatalog(bytes(), locked: true, saved: const [_hit]);
    final c = build(cache: cache, catalog: catalog, online: false);

    c.read(selectedScoreProvider.notifier).select(_catalog());
    await _flush();

    expect(
      catalog.fetchCalls,
      0,
      reason: 'offline grace: no server round-trip',
    );
    expect(c.read(notationProvider).hasDocument, isTrue);
  });

  test('online but unreachable, a cached favourite still plays', () async {
    final cache = InMemoryOfflineScoreCache();
    await cache.write('catalog:x', bytes(), etag: 'e');
    final catalog = _FakeCatalog(
      bytes(),
      error: AuthException(AuthError.unavailable),
      saved: const [_hit],
    );
    final c = build(cache: cache, catalog: catalog, online: true);

    c.read(selectedScoreProvider.notifier).select(_catalog());
    await _flush();

    expect(catalog.fetchCalls, 1);
    expect(c.read(notationProvider).hasDocument, isTrue);
  });

  test('a served open is cached (favourite) and reports the state', () async {
    final cache = InMemoryOfflineScoreCache();
    final catalog = _FakeCatalog(bytes(), saved: const [_hit]);
    final c = build(cache: cache, catalog: catalog);
    // The favourite gate reads the loaded saved list (kept alive here, as the
    // library screen keeps it alive in the app).
    c.listen(savedCatalogScoresProvider, (_, _) {});
    await c.read(savedCatalogScoresProvider.future);

    c.read(selectedScoreProvider.notifier).select(_catalog());
    await _flush();

    expect(c.read(notationProvider).hasDocument, isTrue);
    expect(await cache.has('catalog:x'), isTrue);
    expect(c.read(catalogDailyAccessProvider).valueOrNull?.enabled, isTrue);
  });

  group('CatalogUnlock', () {
    test('success reports the state, bumps the reward revision, names the '
        'entry', () async {
      final catalog = _FakeCatalog(bytes());
      final c = build(cache: InMemoryOfflineScoreCache(), catalog: catalog);
      final revBefore = c.read(rewardRevisionProvider);

      await c.read(catalogUnlockProvider.notifier).unlock(_catalog());

      expect(catalog.unlockCalls, ['x']);
      final s = c.read(catalogUnlockProvider);
      expect(s.unlocked, _catalog());
      expect(s.seq, 1);
      expect(s.error, isFalse);
      expect(s.insufficient, isFalse);
      expect(c.read(rewardRevisionProvider), isNot(revBefore));
      final access = c.read(catalogDailyAccessProvider).valueOrNull!;
      expect(access.spendableBalance, 5);
      expect(access.paidToday, ['x']);
      expect(access.isOpenToday('x'), isTrue);
    });

    test('insufficient points is a typed refusal', () async {
      final catalog = _FakeCatalog(
        bytes(),
        unlockError: AuthException(AuthError.failedPrecondition),
      );
      final c = build(cache: InMemoryOfflineScoreCache(), catalog: catalog);

      await c.read(catalogUnlockProvider.notifier).unlock(_catalog());

      final s = c.read(catalogUnlockProvider);
      expect(s.unlocked, isNull);
      expect(s.insufficient, isTrue);
      expect(s.error, isFalse);
      expect(s.seq, 1);
    });

    test('any other failure is a generic error state', () async {
      final catalog = _FakeCatalog(
        bytes(),
        unlockError: AuthException(AuthError.unavailable),
      );
      final c = build(cache: InMemoryOfflineScoreCache(), catalog: catalog);

      await c.read(catalogUnlockProvider.notifier).unlock(_catalog());

      final s = c.read(catalogUnlockProvider);
      expect(s.error, isTrue);
      expect(s.insufficient, isFalse);
    });

    test('a bundled entry (no catalog id) is a no-op', () async {
      final catalog = _FakeCatalog(bytes());
      final c = build(cache: InMemoryOfflineScoreCache(), catalog: catalog);
      await c
          .read(catalogUnlockProvider.notifier)
          .unlock(
            const CatalogEntry(
              id: 'ode',
              title: 'Ode',
              composer: '',
              level: PracticeLevel.beginner,
              assetPath: 'assets/ode.xml',
            ),
          );
      expect(catalog.unlockCalls, isEmpty);
      expect(c.read(catalogUnlockProvider).seq, 0);
    });
  });

  test('CatalogAccessState helpers', () {
    final s = CatalogAccessState(
      enabled: true,
      locked: false,
      freeQuota: 3,
      freeUsed: 1,
      resetsAtMs: DateTime.utc(2026, 8, 16).millisecondsSinceEpoch,
      daySlotCost: 20,
      spendableBalance: 19,
      subscriber: false,
      upsell: true,
      openedToday: const ['a'],
    );
    expect(s.freeLeft, 2);
    expect(s.canAffordDaySlot, isFalse);
    expect(s.isOpenToday('a'), isTrue);
    expect(s.isOpenToday('b'), isFalse);
    expect(
      s.untilReset(DateTime.utc(2026, 8, 15, 22)),
      const Duration(hours: 2),
    );
    expect(s.untilReset(DateTime.utc(2026, 8, 17)), Duration.zero);
    expect(s.copyWith(locked: true).locked, isTrue);
    expect(s.copyWith(), s);
  });
}
