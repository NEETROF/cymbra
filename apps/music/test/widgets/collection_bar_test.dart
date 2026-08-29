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
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_collections.dart';
import 'package:music/widgets/collection_bar.dart';

@GenerateNiceMocks([MockSpec<ScoreUploadService>()])
import 'collection_bar_test.mocks.dart';

ScoreCollection _c(String id, String name) =>
    ScoreCollection(id: id, name: name, createdAt: DateTime.utc(2026));

Future<ProviderContainer> _pump(
  WidgetTester tester,
  MockScoreUploadService service,
) async {
  final container = ProviderContainer(
    overrides: [scoreUploadServiceProvider.overrideWithValue(service)],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: const MaterialApp(
        locale: Locale('en'),
        localizationsDelegates: [
          AppLocalizations.delegate,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        supportedLocales: AppLocalizations.supportedLocales,
        home: Scaffold(body: CollectionBar()),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return container;
}

/// Opens the "add to a collection" sheet over a trivial host screen.
Future<void> _pumpAddSheet(
  WidgetTester tester,
  MockScoreUploadService service,
) async {
  final container = ProviderContainer(
    overrides: [scoreUploadServiceProvider.overrideWithValue(service)],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        locale: const Locale('en'),
        localizationsDelegates: const [
          AppLocalizations.delegate,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        supportedLocales: AppLocalizations.supportedLocales,
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              onPressed: () => showAddToCollection(context, 's1'),
              child: const Text('open'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('open'));
  await tester.pumpAndSettle();
}

late final AppLocalizations _enStrings;

void main() {
  setUpAll(() async {
    _enStrings = await AppLocalizations.delegate.load(const Locale('en'));
  });

  testWidgets('selecting a collection filters, clearing it restores all', (
    tester,
  ) async {
    final service = MockScoreUploadService();
    when(
      service.listCollections(),
    ).thenAnswer((_) async => [_c('c1', 'Chopin'), _c('c2', 'Etudes')]);
    final c = await _pump(tester, service);

    // "All my scores" is the default; both collections are offered.
    expect(c.read(collectionFilterProvider), isNull);
    expect(find.widgetWithText(FilterChip, 'Chopin'), findsOneWidget);
    expect(find.widgetWithText(FilterChip, 'Etudes'), findsOneWidget);

    await tester.tap(find.widgetWithText(FilterChip, 'Chopin'));
    await tester.pumpAndSettle();
    expect(c.read(collectionFilterProvider), 'c1');

    await tester.tap(find.widgetWithText(FilterChip, 'All my scores'));
    await tester.pumpAndSettle();
    expect(c.read(collectionFilterProvider), isNull);
  });

  testWidgets('a failed collection list leaves the library usable', (
    tester,
  ) async {
    final service = MockScoreUploadService();
    when(service.listCollections()).thenThrow(StateError('offline'));
    await _pump(tester, service);
    // No crash, and the unfiltered view is still selectable.
    expect(find.widgetWithText(FilterChip, 'All my scores'), findsOneWidget);
  });

  testWidgets('a duplicate name shows a localized message, not an exception', (
    tester,
  ) async {
    final service = MockScoreUploadService();
    when(
      service.listCollections(),
    ).thenAnswer((_) async => [_c('c1', 'Chopin')]);
    when(
      service.createCollection(any),
    ).thenThrow(AuthException(AuthError.alreadyExists));
    await _pump(tester, service);

    await tester.tap(find.widgetWithText(ActionChip, 'Manage collections'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'New collection'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'chopin');
    await tester.tap(find.widgetWithText(FilledButton, 'Save'));
    await tester.pumpAndSettle();

    expect(
      find.text('You already have a collection with that name.'),
      findsOneWidget,
    );
    expect(find.textContaining('AuthException'), findsNothing);
  });

  test('every failure reason has its own localized wording', () {
    // Guards the mapping itself: a new CollectionError must get a message.
    final messages = {
      for (final e in CollectionError.values)
        e: collectionErrorMessage(_enStrings, e),
    };
    expect(messages.values.toSet(), hasLength(CollectionError.values.length));
    expect(messages[CollectionError.nameTaken], contains('already have'));
    expect(messages[CollectionError.invalidName], contains('name'));
  });

  testWidgets('renaming goes through the store with the typed name', (
    tester,
  ) async {
    final service = MockScoreUploadService();
    when(
      service.listCollections(),
    ).thenAnswer((_) async => [_c('c1', 'Chopin')]);
    await _pump(tester, service);

    await tester.tap(find.widgetWithText(ActionChip, 'Manage collections'));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.edit_outlined));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Nocturnes');
    await tester.tap(find.widgetWithText(FilledButton, 'Save'));
    await tester.pumpAndSettle();

    verify(service.renameCollection('c1', 'Nocturnes')).called(1);
  });

  testWidgets('cancelling the name dialog changes nothing', (tester) async {
    final service = MockScoreUploadService();
    when(service.listCollections()).thenAnswer((_) async => const []);
    await _pump(tester, service);

    await tester.tap(find.widgetWithText(ActionChip, 'Manage collections'));
    await tester.pumpAndSettle();
    expect(find.text('No collections yet.'), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'New collection'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'Discarded');
    await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
    await tester.pumpAndSettle();

    verifyNever(service.createCollection(any));
  });

  group('add to a collection', () {
    testWidgets('picking one adds the score and confirms it', (tester) async {
      final service = MockScoreUploadService();
      when(
        service.listCollections(),
      ).thenAnswer((_) async => [_c('c1', 'Chopin')]);
      await _pumpAddSheet(tester, service);

      await tester.tap(find.text('Chopin'));
      await tester.pumpAndSettle();

      verify(service.addToCollection('c1', 's1')).called(1);
      expect(find.text('Added to Chopin'), findsOneWidget);
    });

    testWidgets('a failure shows a localized message, not an exception', (
      tester,
    ) async {
      final service = MockScoreUploadService();
      when(
        service.listCollections(),
      ).thenAnswer((_) async => [_c('c1', 'Chopin')]);
      when(service.addToCollection(any, any)).thenThrow(StateError('offline'));
      await _pumpAddSheet(tester, service);

      await tester.tap(find.text('Chopin'));
      await tester.pumpAndSettle();

      expect(find.text('Something went wrong. Try again.'), findsOneWidget);
      expect(find.textContaining('StateError'), findsNothing);
    });

    testWidgets('with no collections it says so instead of an empty sheet', (
      tester,
    ) async {
      final service = MockScoreUploadService();
      when(service.listCollections()).thenAnswer((_) async => const []);
      await _pumpAddSheet(tester, service);
      expect(find.text('No collections yet.'), findsOneWidget);
    });
  });

  testWidgets('deleting a collection asks first and says scores are kept', (
    tester,
  ) async {
    final service = MockScoreUploadService();
    when(
      service.listCollections(),
    ).thenAnswer((_) async => [_c('c1', 'Chopin')]);
    await _pump(tester, service);

    await tester.tap(find.widgetWithText(ActionChip, 'Manage collections'));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pumpAndSettle();
    expect(
      find.text('Delete this collection? Your scores are kept.'),
      findsOneWidget,
    );

    // Cancelling deletes nothing.
    await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
    await tester.pumpAndSettle();
    verifyNever(service.deleteCollection(any));

    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Delete'));
    await tester.pumpAndSettle();
    verify(service.deleteCollection('c1')).called(1);
  });
}
