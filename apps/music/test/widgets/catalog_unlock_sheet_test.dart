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
import 'package:music/services/catalog_service.dart';
import 'package:music/services/score_preview_service.dart';
import 'package:music/state/catalog_daily_access_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/state/usage_tracking_notifier.dart';
import 'package:music/widgets/catalog_access_widgets.dart';
import 'package:music/widgets/catalog_unlock_listener.dart';
import 'package:music/widgets/catalog_unlock_sheet.dart';

import '../support/localized.dart';

class _FakeCatalog extends Fake implements CatalogService {
  _FakeCatalog(this.state, {this.unlockError});
  final CatalogAccessState? state;
  final Object? unlockError;
  final List<String> unlockCalls = [];

  @override
  Future<CatalogAccessState?> dailyAccess() async => state;

  @override
  Future<CatalogAccessState?> unlockForToday(String catalogId) async {
    unlockCalls.add(catalogId);
    if (unlockError != null) throw unlockError!;
    return CatalogAccessState(
      enabled: true,
      locked: false,
      freeQuota: 3,
      freeUsed: 3,
      resetsAtMs: _reset,
      daySlotCost: 20,
      spendableBalance: 5,
      subscriber: false,
      upsell: true,
      openedToday: const ['x'],
      paidToday: const ['x'],
    );
  }
}

class _FakePreview implements ScorePreviewService {
  final List<String> auditioned = [];
  int stops = 0;

  @override
  Future<bool> audition(String catalogId) async {
    auditioned.add(catalogId);
    return true;
  }

  @override
  Future<void> stop() async => stops++;
}

final _reset = DateTime.utc(2026, 8, 16).millisecondsSinceEpoch;

CatalogAccessState _state({int balance = 25, int freeUsed = 3}) =>
    CatalogAccessState(
      enabled: true,
      locked: false,
      freeQuota: 3,
      freeUsed: freeUsed,
      resetsAtMs: _reset,
      daySlotCost: 20,
      spendableBalance: balance,
      subscriber: false,
      upsell: true,
      openedToday: const ['a', 'b', 'c'],
    );

const _entry = CatalogEntry(
  id: 'catalog-x',
  title: 'Clair de Lune',
  composer: 'Debussy',
  level: PracticeLevel.advanced,
  catalogId: 'x',
  hasPreview: true,
);

ProviderContainer _container(
  _FakeCatalog catalog, {
  ScorePreviewService? preview,
}) {
  final c = ProviderContainer(
    overrides: [
      catalogServiceProvider.overrideWithValue(catalog),
      scorePreviewServiceProvider.overrideWithValue(preview ?? _FakePreview()),
      canUseOnlineServicesProvider.overrideWithValue(true),
      usageCollectionKillSwitchProvider.overrideWithValue(false),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

/// A host page with a button that opens the sheet, wrapped in the unlock
/// listener (as the hub/library are).
class _Host extends ConsumerWidget {
  const _Host({required this.entry});
  final CatalogEntry entry;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Two nested listeners, as in the app (hub pushed over the library): the
    // outcome must still be surfaced once.
    return CatalogUnlockListener(
      child: CatalogUnlockListener(
        child: Scaffold(
          body: Column(
            children: [
              const CatalogAccessChip(),
              Center(
                child: TextButton(
                  key: const Key('open-sheet'),
                  onPressed: () => showCatalogUnlockSheet(context, ref, entry),
                  child: const Text('open'),
                ),
              ),
              SizedBox(
                width: 200,
                height: 40,
                child: CatalogAccessMark(catalogId: entry.catalogId!),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer c, {
  CatalogEntry entry = _entry,
}) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(_Host(entry: entry), locale: const Locale('en')),
    ),
  );
  await tester.pumpAndSettle();
}

Future<void> _openSheet(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('open-sheet')));
  await tester.pumpAndSettle();
  expect(find.byKey(const Key('catalog-unlock-sheet')), findsOneWidget);
}

void main() {
  testWidgets('chip shows free opens left and reset time', (tester) async {
    final c = _container(_FakeCatalog(_state(freeUsed: 1)));
    await _pump(tester, c);
    expect(find.byKey(const Key('catalog-access-chip')), findsOneWidget);
    expect(find.textContaining('2 free opens left'), findsOneWidget);
    expect(find.textContaining('resets in'), findsOneWidget);
    // Piece 'x' is not open today and the quota is not exhausted: no mark.
    expect(find.byKey(const Key('catalog-access-open-today')), findsNothing);
    expect(find.byKey(const Key('catalog-access-locked')), findsNothing);
  });

  testWidgets('chip hidden when the gate is off; marks follow the state', (
    tester,
  ) async {
    final c = _container(_FakeCatalog(null));
    await _pump(tester, c);
    expect(find.byKey(const Key('catalog-access-chip')), findsNothing);
    // Quota exhausted → the card carries the lock + cost hint.
    c.read(catalogDailyAccessProvider.notifier).report(_state());
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('catalog-access-chip')), findsOneWidget);
    expect(find.byKey(const Key('catalog-access-locked')), findsOneWidget);
    expect(find.text('20 pts'), findsOneWidget);
    // Once open today → the check mark instead.
    c
        .read(catalogDailyAccessProvider.notifier)
        .report(
          CatalogAccessState(
            enabled: true,
            locked: false,
            freeQuota: 3,
            freeUsed: 3,
            resetsAtMs: _reset,
            daySlotCost: 20,
            spendableBalance: 25,
            subscriber: false,
            upsell: true,
            openedToday: const ['x'],
          ),
        );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('catalog-access-open-today')), findsOneWidget);
    expect(find.byKey(const Key('catalog-access-locked')), findsNothing);
  });

  testWidgets('sheet: audition plays the teaser, never the MusicXML', (
    tester,
  ) async {
    final preview = _FakePreview();
    final catalog = _FakeCatalog(_state());
    final c = _container(catalog, preview: preview);
    await _pump(tester, c);
    await _openSheet(tester);

    expect(find.text('Clair de Lune'), findsOneWidget);
    expect(find.byKey(const Key('catalog-unlock-upsell')), findsOneWidget);
    expect(find.text('Your balance: 25 pts'), findsOneWidget);
    await tester.tap(find.byKey(const Key('catalog-unlock-listen')));
    await tester.pumpAndSettle();
    expect(preview.auditioned, ['x']);
    expect(find.text('Stop'), findsOneWidget);
    // Dismiss stops playback and spends nothing.
    await tester.tap(find.byKey(const Key('catalog-unlock-dismiss')));
    await tester.pumpAndSettle();
    expect(preview.stops, greaterThanOrEqualTo(1));
    expect(catalog.unlockCalls, isEmpty);
  });

  testWidgets('sheet: no teaser greys the listen control', (tester) async {
    final c = _container(_FakeCatalog(_state()));
    await _pump(
      tester,
      c,
      entry: const CatalogEntry(
        id: 'catalog-x',
        title: 'Clair de Lune',
        composer: 'Debussy',
        level: PracticeLevel.advanced,
        catalogId: 'x',
      ),
    );
    await _openSheet(tester);
    final btn = tester.widget<OutlinedButton>(
      find.byKey(const Key('catalog-unlock-listen')),
    );
    expect(btn.onPressed, isNull);
    expect(find.text('No excerpt available'), findsOneWidget);
  });

  testWidgets('sheet: unaffordable → confirm disabled with the shortfall', (
    tester,
  ) async {
    final catalog = _FakeCatalog(_state(balance: 12));
    final c = _container(catalog);
    await _pump(tester, c);
    await _openSheet(tester);
    final btn = tester.widget<FilledButton>(
      find.byKey(const Key('catalog-unlock-confirm')),
    );
    expect(btn.onPressed, isNull);
    expect(find.text('You need 8 more points'), findsOneWidget);
    expect(find.text('Unlock for 20 pts today'), findsOneWidget);
  });

  testWidgets('sheet: confirm fires the unlock; the listener surfaces success', (
    tester,
  ) async {
    final catalog = _FakeCatalog(_state());
    final c = _container(catalog);
    await _pump(tester, c);
    await _openSheet(tester);
    await tester.tap(find.byKey(const Key('catalog-unlock-confirm')));
    // A few frames: the sheet closes, the unlock resolves, the listener fires
    // (its re-open pre-flight is still spinning at this point).
    for (var i = 0; i < 5; i++) {
      await tester.pump(const Duration(milliseconds: 80));
    }
    expect(catalog.unlockCalls, ['x']);
    expect(find.byKey(const Key('catalog-unlock-sheet')), findsNothing);
    // Exactly one snackbar (and one re-open) despite the two stacked listeners.
    expect(find.text('Unlocked for today!'), findsOneWidget);
    expect(find.text('Loading score…'), findsOneWidget);
    // The state now reflects the paid slot: the piece is open today.
    expect(
      c.read(catalogDailyAccessProvider).valueOrNull!.isOpenToday('x'),
      isTrue,
    );
    // The listener re-opens the piece; with no engine wired here the pre-flight
    // fails on the notation load (a load-error snackbar), which is outside this
    // widget's scope — just drain it.
    await tester.pump(const Duration(seconds: 1));
    await tester.pumpAndSettle();
  });

  testWidgets('listener surfaces a typed refusal, nothing spent', (
    tester,
  ) async {
    final catalog = _FakeCatalog(
      _state(),
      unlockError: Exception('FAILED_PRECONDITION'),
    );
    final c = _container(catalog);
    await _pump(tester, c);
    await _openSheet(tester);
    await tester.tap(find.byKey(const Key('catalog-unlock-confirm')));
    await tester.pumpAndSettle();
    expect(find.text('This piece could not be unlocked'), findsOneWidget);
    expect(find.text('Unlocked for today!'), findsNothing);
  });
}
