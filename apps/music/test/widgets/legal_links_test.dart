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
import 'package:music/screens/auth/account_menu.dart';
import 'package:music/screens/auth/entry_screen.dart';
import 'package:music/services/legal_links.dart';
import 'package:music/services/token_store.dart';
import 'package:music/state/app_locale.dart';
import 'package:music/state/session_notifier.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';
import '../support/localized.dart';

/// Records every URL asked to open, without touching the native browser.
class _RecordingLauncher implements LegalLinkLauncher {
  final List<Uri> opened = [];

  @override
  Future<void> open(Uri url) async => opened.add(url);
}

void main() {
  group('legalLinksFor', () {
    test('French locale resolves the French pages', () {
      final links = legalLinksFor('fr');
      expect(links.terms, Uri.parse('https://cymbra.app/cgu/'));
      expect(links.privacy, Uri.parse('https://cymbra.app/confidentialite/'));
    });

    test('non-French locales fall back to the English pages', () {
      for (final code in ['en', 'es', 'it', 'de']) {
        final links = legalLinksFor(code);
        expect(links.terms, Uri.parse('https://cymbra.app/en/terms/'));
        expect(links.privacy, Uri.parse('https://cymbra.app/en/privacy/'));
      }
    });
  });

  group('entry consent notice', () {
    Future<_RecordingLauncher> pumpEntry(
      WidgetTester tester, {
      required Locale locale,
    }) async {
      final launcher = _RecordingLauncher();
      final container = ProviderContainer(
        overrides: [
          ...authOverrides(oidc: FakeOidcTokenSource(appleAvailable: true)),
          deviceLocaleProvider.overrideWithValue(locale),
          legalLinkLauncherProvider.overrideWithValue(launcher),
        ],
      );
      addTearDown(container.dispose);
      await tester.binding.setSurfaceSize(const Size(1200, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const EntryScreen(), locale: locale),
        ),
      );
      await tester.pumpAndSettle();
      return launcher;
    }

    testWidgets('renders the two tappable references', (tester) async {
      await pumpEntry(tester, locale: const Locale('en'));
      expect(find.byKey(const Key('entry-legal-terms')), findsOneWidget);
      expect(find.byKey(const Key('entry-legal-privacy')), findsOneWidget);
    });

    testWidgets('tapping a reference opens the English page', (tester) async {
      final launcher = await pumpEntry(tester, locale: const Locale('en'));

      await tester.tap(find.byKey(const Key('entry-legal-terms')));
      await tester.tap(find.byKey(const Key('entry-legal-privacy')));
      await tester.pump();

      expect(launcher.opened, [
        Uri.parse('https://cymbra.app/en/terms/'),
        Uri.parse('https://cymbra.app/en/privacy/'),
      ]);
    });

    testWidgets('French locale opens the French pages', (tester) async {
      final launcher = await pumpEntry(tester, locale: const Locale('fr'));

      await tester.tap(find.byKey(const Key('entry-legal-terms')));
      await tester.tap(find.byKey(const Key('entry-legal-privacy')));
      await tester.pump();

      expect(launcher.opened, [
        Uri.parse('https://cymbra.app/cgu/'),
        Uri.parse('https://cymbra.app/confidentialite/'),
      ]);
    });
  });

  group('account menu legal links', () {
    const tokens = StoredTokens(accessToken: 'a', refreshToken: 'r');

    Future<_RecordingLauncher> pumpMenu(
      WidgetTester tester, {
      required Locale locale,
    }) async {
      final launcher = _RecordingLauncher();
      final container = ProviderContainer(
        overrides: [
          ...authOverrides(
            store: FakeTokenStore(tokens: tokens),
            account: FakeAccountService(account: fakeAccount(handle: 'bob')),
          ),
          deviceLocaleProvider.overrideWithValue(locale),
          legalLinkLauncherProvider.overrideWithValue(launcher),
        ],
      );
      addTearDown(container.dispose);
      container.read(sessionNotifierProvider);
      await tester.runAsync(() => pumpEventQueue());
      await tester.binding.setSurfaceSize(const Size(1200, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(
            const Scaffold(body: Center(child: AccountMenu())),
            locale: locale,
          ),
        ),
      );
      await tester.pump();
      return launcher;
    }

    testWidgets('terms and privacy open the locale-resolved pages', (
      tester,
    ) async {
      final launcher = await pumpMenu(tester, locale: const Locale('en'));

      await tester.tap(find.byKey(const Key('account-menu')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('account-legal-terms')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('account-menu')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('account-legal-privacy')));
      await tester.pumpAndSettle();

      expect(launcher.opened, [
        Uri.parse('https://cymbra.app/en/terms/'),
        Uri.parse('https://cymbra.app/en/privacy/'),
      ]);
    });

    testWidgets('French locale opens the French pages', (tester) async {
      final launcher = await pumpMenu(tester, locale: const Locale('fr'));

      await tester.tap(find.byKey(const Key('account-menu')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('account-legal-terms')));
      await tester.pumpAndSettle();

      expect(launcher.opened, [Uri.parse('https://cymbra.app/cgu/')]);
    });

    testWidgets(
      'tapping the licenses entry opens the in-app license page, not an external browser',
      (tester) async {
        final launcher = await pumpMenu(tester, locale: const Locale('en'));

        await tester.tap(find.byKey(const Key('account-menu')));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('account-legal-licenses')));
        await tester.pumpAndSettle();

        expect(find.byType(LicensePage), findsOneWidget);
        expect(launcher.opened, isEmpty);
      },
    );

    testWidgets('the account menu hosts the language picker', (tester) async {
      await pumpMenu(tester, locale: const Locale('en'));

      await tester.tap(find.byKey(const Key('account-menu')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('account-language')));
      await tester.pumpAndSettle();

      // The language picker dialog opened, listing the languages (by flag).
      expect(find.byType(SimpleDialog), findsOneWidget);
      expect(find.text('🇫🇷'), findsOneWidget);
    });
  });
}
