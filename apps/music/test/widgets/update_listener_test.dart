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
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/update/app_version.dart';
import 'package:music/services/update/desktop_update_service.dart';
import 'package:music/services/update/update_manifest.dart';
import 'package:music/state/update_listener.dart';
import 'package:music/state/update_notifier.dart';
import 'package:music/state/update_state.dart';
import 'package:music/widgets/desktop_update_tile.dart';

AppVersion v(String raw) => AppVersion.tryParse(raw)!;

const _target = UpdateTarget(
  kind: 'inno-setup',
  url: 'https://example.invalid/setup.exe',
  size: 1024,
  sha256: 'ab',
);

UpdateManifest _manifest() => UpdateManifest(
  schema: kUpdateSchemaVersion,
  product: 'music',
  channel: 'stable',
  version: v('1.25.0+34'),
  releasedAt: '2026-08-21T10:00:00Z',
  minSupportedVersion: null,
  notesUrl: null,
  targets: const {'windows-x64': _target},
);

/// A notifier whose state the test drives directly, so the listener is exercised
/// without a service, an installer or a network.
class _ScriptedUpdate extends Update {
  _ScriptedUpdate(this._initial);
  final UpdateState _initial;

  @override
  UpdateState build() => _initial;

  void emit(UpdateState next) => state = next;

  // The listener's launch check must not reach a service in a widget test.
  @override
  Future<void> checkOnLaunch() async {}
}

void main() {
  late _ScriptedUpdate updater;

  Future<ProviderContainer> pump(
    WidgetTester tester, {
    bool sessionActive = false,
    UpdateState initial = const UpdateState.idle(),
  }) async {
    updater = _ScriptedUpdate(initial);
    final container = ProviderContainer(
      overrides: [
        updateProvider.overrideWith(() => updater),
        updateSessionActiveProvider.overrideWithValue(sessionActive),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: const UpdateListener(child: Text('app')),
        ),
      ),
    );
    await tester.pump();
    return container;
  }

  testWidgets('an offer opens the prompt over the app', (tester) async {
    await pump(tester);
    expect(find.text('app'), findsOne);
    updater.emit(
      UpdateState.available(
        manifest: _manifest(),
        target: _target,
        canSelfInstall: true,
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('update-prompt')), findsOne);
  });

  testWidgets('the prompt closes when the offer is dismissed', (tester) async {
    await pump(tester);
    updater.emit(
      UpdateState.available(
        manifest: _manifest(),
        target: _target,
        canSelfInstall: true,
      ),
    );
    await tester.pumpAndSettle();
    updater.emit(const UpdateState.idle());
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('update-prompt')), findsNothing);
  });

  testWidgets('the prompt is deferred while a session is running, and shown '
      'when it ends', (tester) async {
    // The session flag is a provider the app overrides with the player, so the
    // test flips it the same way a piece starting and ending would.
    final active = StateProvider<bool>((ref) => true);
    updater = _ScriptedUpdate(const UpdateState.idle());
    final container = ProviderContainer(
      overrides: [
        updateProvider.overrideWith(() => updater),
        updateSessionActiveProvider.overrideWith((ref) => ref.watch(active)),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: const UpdateListener(child: Text('app')),
        ),
      ),
    );
    await tester.pump();

    updater.emit(
      UpdateState.available(
        manifest: _manifest(),
        target: _target,
        canSelfInstall: true,
      ),
    );
    await tester.pumpAndSettle();
    // Interrupting someone mid-piece is the fastest way to make the prompt
    // something people dismiss without reading.
    expect(find.byKey(const Key('update-prompt')), findsNothing);

    container.read(active.notifier).state = false;
    await tester.pumpAndSettle();
    // The offer waited; it did not vanish.
    expect(find.byKey(const Key('update-prompt')), findsOne);
  });

  testWidgets('a forced update replaces the app and keeps blocking through the '
      'download', (tester) async {
    await pump(tester);
    updater.emit(
      UpdateState.updateRequired(
        manifest: _manifest(),
        target: _target,
        canSelfInstall: true,
        current: v('1.19.0+20'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('update-required')), findsOne);
    expect(find.text('app'), findsNothing);

    // The notifier moves on as the user acts; without the latch the blocking
    // screen would hand the app back to a client that cannot reach the backend.
    updater.emit(
      UpdateState.downloading(manifest: _manifest(), received: 1, total: 1024),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('update-required')), findsOne);
    expect(find.text('app'), findsNothing);
  });

  group('DesktopUpdateTile', () {
    Future<void> pumpTile(
      WidgetTester tester, {
      required AppPlatform platform,
      bool enabled = true,
      UpdateState state = const UpdateState.idle(),
    }) async {
      updater = _ScriptedUpdate(state);
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            updateProvider.overrideWith(() => updater),
            appPlatformProvider.overrideWithValue(platform),
            desktopUpdateEnabledProvider.overrideWithValue(enabled),
          ],
          child: MaterialApp(
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: const Scaffold(body: DesktopUpdateTile()),
          ),
        ),
      );
      await tester.pump();
    }

    testWidgets('renders on Windows and Linux', (tester) async {
      for (final p in [AppPlatform.windows, AppPlatform.linux]) {
        await pumpTile(tester, platform: p);
        expect(find.byKey(const Key('profile-check-updates')), findsOne);
      }
    });

    testWidgets('renders nothing on store-managed platforms', (tester) async {
      for (final p in [
        AppPlatform.ios,
        AppPlatform.android,
        AppPlatform.macos,
        AppPlatform.web,
      ]) {
        await pumpTile(tester, platform: p);
        expect(find.byKey(const Key('profile-check-updates')), findsNothing);
      }
    });

    testWidgets('renders nothing while the flag is off', (tester) async {
      await pumpTile(tester, platform: AppPlatform.windows, enabled: false);
      expect(find.byKey(const Key('profile-check-updates')), findsNothing);
    });

    testWidgets('reports an up-to-date result', (tester) async {
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      await pumpTile(
        tester,
        platform: AppPlatform.windows,
        state: const UpdateState.upToDate(),
      );
      expect(find.text(l10n.updateUpToDateStatus), findsOne);
    });

    testWidgets(
      'while checking, the row is busy and cannot start a second one',
      (tester) async {
        final l10n = await AppLocalizations.delegate.load(const Locale('en'));
        await pumpTile(
          tester,
          platform: AppPlatform.windows,
          state: const UpdateState.checking(),
        );
        expect(find.text(l10n.updateCheckingStatus), findsOne);
        expect(tester.widget<ListTile>(find.byType(ListTile)).onTap, isNull);
      },
    );

    testWidgets('a failure reads as a localized message, never a raw cause', (
      tester,
    ) async {
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      await pumpTile(
        tester,
        platform: AppPlatform.windows,
        state: const UpdateState.failed(UpdateFailureCause.network),
      );
      expect(find.text(l10n.updateErrorNetwork), findsOne);
      expect(find.textContaining('UpdateFailureCause'), findsNothing);
    });
  });
}
