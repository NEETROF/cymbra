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
import 'package:music/screens/update_required_screen.dart';
import 'package:music/services/update/app_version.dart';
import 'package:music/services/update/desktop_update_service.dart';
import 'package:music/services/update/update_manifest.dart';
import 'package:music/state/update_notifier.dart';
import 'package:music/state/update_state.dart';
import 'package:music/widgets/update_prompt.dart';

AppVersion v(String raw) => AppVersion.tryParse(raw)!;

const _target = UpdateTarget(
  kind: 'inno-setup',
  url: 'https://example.invalid/setup.exe',
  size: 48123904,
  sha256: 'ab',
);

UpdateManifest _manifest({String? notes = 'https://example.invalid/notes'}) =>
    UpdateManifest(
      schema: kUpdateSchemaVersion,
      product: 'music',
      channel: 'stable',
      version: v('1.25.0+34'),
      releasedAt: '2026-08-21T10:00:00Z',
      minSupportedVersion: null,
      notesUrl: notes,
      targets: const {'windows-x64': _target},
    );

/// Pins the notifier to a fixed state so the widget renders one branch without
/// a service, an installer or a network.
class _FixedUpdate extends Update {
  _FixedUpdate(this._value);
  final UpdateState _value;
  @override
  UpdateState build() => _value;
}

Future<void> _pump(WidgetTester tester, UpdateState state, Widget child) =>
    tester.pumpWidget(
      ProviderScope(
        overrides: [updateProvider.overrideWith(() => _FixedUpdate(state))],
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: Scaffold(body: child),
        ),
      ),
    );

void main() {
  group('formatUpdateSize', () {
    test('reads as a size someone can decide on, not a byte count', () {
      expect(formatUpdateSize(48123904), '46 MB');
      expect(formatUpdateSize(5 * 1024 * 1024), '5.0 MB');
      expect(formatUpdateSize(2048), '2 KB');
      expect(formatUpdateSize(1), '1 KB');
    });
  });

  group('UpdatePromptDialog', () {
    testWidgets('the offer shows the version, the size and three actions', (
      tester,
    ) async {
      await _pump(
        tester,
        UpdateState.available(
          manifest: _manifest(),
          target: _target,
          canSelfInstall: true,
        ),
        const UpdatePromptDialog(),
      );
      expect(find.byKey(const Key('update-prompt')), findsOne);
      expect(find.textContaining('1.25.0'), findsOne);
      expect(find.textContaining('46 MB'), findsOne);
      expect(find.byKey(const Key('update-confirm')), findsOne);
      expect(find.byKey(const Key('update-later')), findsOne);
      expect(find.byKey(const Key('update-skip')), findsOne);
      expect(find.byKey(const Key('update-release-notes')), findsOne);
    });

    testWidgets('a portable install offers the download page, not an update', (
      tester,
    ) async {
      await _pump(
        tester,
        UpdateState.available(
          manifest: _manifest(),
          target: _target,
          canSelfInstall: false,
        ),
        const UpdatePromptDialog(),
      );
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l10n.updateActionDownloadPage), findsOne);
      expect(find.text(l10n.updateNotSelfInstallableBody), findsOne);
      expect(find.text(l10n.updateActionUpdate), findsNothing);
    });

    testWidgets('a manifest without release notes hides the link', (
      tester,
    ) async {
      await _pump(
        tester,
        UpdateState.available(
          manifest: _manifest(notes: null),
          target: _target,
          canSelfInstall: true,
        ),
        const UpdatePromptDialog(),
      );
      expect(find.byKey(const Key('update-release-notes')), findsNothing);
    });

    testWidgets('progress shows a determinate bar and the byte counts', (
      tester,
    ) async {
      await _pump(
        tester,
        UpdateState.downloading(
          manifest: _manifest(),
          received: 24061952,
          total: 48123904,
        ),
        const UpdatePromptDialog(),
      );
      expect(find.byKey(const Key('update-progress')), findsOne);
      final bar = tester.widget<LinearProgressIndicator>(
        find.byType(LinearProgressIndicator),
      );
      expect(bar.value, closeTo(0.5, 0.01));
      expect(find.textContaining('46 MB'), findsOne);
    });

    testWidgets('installing says the app will restart', (tester) async {
      await _pump(
        tester,
        UpdateState.installing(manifest: _manifest()),
        const UpdatePromptDialog(),
      );
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.byKey(const Key('update-installing')), findsOne);
      expect(find.text(l10n.updateInstallingBody), findsOne);
    });

    testWidgets('a failure shows a localized message, never a raw cause', (
      tester,
    ) async {
      await _pump(
        tester,
        const UpdateState.failed(UpdateFailureCause.integrity),
        const UpdatePromptDialog(),
      );
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l10n.updateErrorIntegrity), findsOne);
      // The enum name must never reach the screen.
      expect(find.textContaining('UpdateFailureCause'), findsNothing);
      expect(find.textContaining('integrity'), findsNothing);
    });
  });

  group('UpdateRequiredScreen', () {
    UpdateRequired required({bool canSelfInstall = true}) => UpdateRequired(
      manifest: _manifest(),
      target: _target,
      canSelfInstall: canSelfInstall,
      current: v('1.19.0+20'),
    );

    testWidgets('blocks with one action and no way to dismiss', (tester) async {
      final state = required();
      await _pump(tester, state, UpdateRequiredScreen(state: state));
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.byKey(const Key('update-required')), findsOne);
      expect(find.text(l10n.updateRequiredTitle), findsOne);
      expect(find.textContaining('1.25.0'), findsOne);
      expect(find.byKey(const Key('update-required-action')), findsOne);
      // No "Later", no "Skip": the only way out is installing.
      expect(find.text(l10n.updateActionLater), findsNothing);
      expect(find.text(l10n.updateActionSkip), findsNothing);
    });

    testWidgets('keeps blocking while the download runs', (tester) async {
      final state = required();
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            updateProvider.overrideWith(
              () => _FixedUpdate(
                UpdateState.downloading(
                  manifest: _manifest(),
                  received: 10,
                  total: 100,
                ),
              ),
            ),
          ],
          child: MaterialApp(
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: UpdateRequiredScreen(state: state),
          ),
        ),
      );
      expect(find.byKey(const Key('update-required')), findsOne);
      expect(find.byType(LinearProgressIndicator), findsOne);
      expect(find.byKey(const Key('update-required-action')), findsNothing);
    });

    testWidgets('a failure leaves the action on screen so it can be retried', (
      tester,
    ) async {
      final state = required();
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            updateProvider.overrideWith(
              () => _FixedUpdate(
                const UpdateState.failed(UpdateFailureCause.network),
              ),
            ),
          ],
          child: MaterialApp(
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: UpdateRequiredScreen(state: state),
          ),
        ),
      );
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l10n.updateErrorNetwork), findsOne);
      expect(find.byKey(const Key('update-required-action')), findsOne);
    });

    testWidgets('a portable install points at the download page', (
      tester,
    ) async {
      final state = required(canSelfInstall: false);
      await _pump(tester, state, UpdateRequiredScreen(state: state));
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l10n.updateActionDownloadPage), findsOne);
      expect(find.text(l10n.updateNotSelfInstallableBody), findsOne);
    });
  });
}
