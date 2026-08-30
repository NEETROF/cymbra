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

import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/update/app_version.dart';
import 'package:music/services/update/desktop_update_service.dart';
import 'package:music/services/update/rollout_bucket.dart';
import 'package:music/services/update/update_installer.dart';
import 'package:music/services/update/update_manifest.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/update_notifier.dart';
import 'package:music/state/update_state.dart';

import '../support/prefs_fakes.dart';
import 'update_notifier_test.mocks.dart';

@GenerateNiceMocks([
  MockSpec<DesktopUpdateService>(),
  MockSpec<UpdateInstaller>(),
])
AppVersion v(String raw) => AppVersion.tryParse(raw)!;

const _target = UpdateTarget(
  kind: 'inno-setup',
  url: 'https://example.invalid/setup.exe',
  size: 1024,
  sha256: 'ab',
);

UpdateManifest manifest({String version = '1.25.0+34', String? min}) =>
    UpdateManifest(
      schema: kUpdateSchemaVersion,
      product: 'music',
      channel: 'stable',
      version: v(version),
      releasedAt: '2026-08-21T10:00:00Z',
      minSupportedVersion: min == null ? null : v(min),
      notesUrl: 'https://example.invalid/notes',
      targets: const {'windows-x64': _target},
    );

void main() {
  // Mockito cannot invent a value for a sealed return type, even when every
  // call is stubbed.
  provideDummy<UpdateCheckOutcome>(const UpdateUpToDate());
  provideDummy<UpdateDownloadOutcome>(
    const UpdateDownloadFailed(UpdateFailureCause.network),
  );
  provideDummy<InstallOutcome>(const InstallNotSelfInstallable('dummy'));

  late MockDesktopUpdateService service;
  late MockUpdateInstaller installer;
  late FakePreferencesService prefs;
  late DateTime now;

  ProviderContainer build({
    bool enabled = true,
    AppPlatform platform = AppPlatform.windows,
    Map<String, String>? stored,
    String current = '1.24.0+32',
    bool sessionActive = false,
    bool canSelfInstall = true,
  }) {
    service = MockDesktopUpdateService();
    installer = MockUpdateInstaller();
    prefs = FakePreferencesService(stored);
    now = DateTime.utc(2026, 8, 30, 12);
    when(installer.canSelfInstall()).thenAnswer((_) async => canSelfInstall);
    final container = ProviderContainer(
      overrides: [
        desktopUpdateEnabledProvider.overrideWithValue(enabled),
        updateSessionActiveProvider.overrideWithValue(sessionActive),
        appPlatformProvider.overrideWithValue(platform),
        preferencesServiceProvider.overrideWithValue(prefs),
        nowFnProvider.overrideWithValue(() => now),
        desktopUpdateServiceProvider.overrideWithValue(service),
        updateInstallerProvider.overrideWithValue(installer),
        currentAppVersionProvider.overrideWith(
          (ref) async => AppVersion.tryParse(current),
        ),
      ],
    );
    addTearDown(container.dispose);
    return container;
  }

  void answers(UpdateCheckOutcome outcome) {
    when(service.check(any)).thenAnswer((_) async => outcome);
  }

  group('gates', () {
    test(
      'the flag off means no check at all — not even a network call',
      () async {
        final c = build(enabled: false);
        await c.read(updateProvider.notifier).checkOnLaunch();
        await c.read(updateProvider.notifier).checkNow();
        verifyNever(service.check(any));
        expect(c.read(updateProvider), isA<UpdateIdle>());
      },
    );

    test('a store-managed platform never consults the feed', () async {
      for (final p in [
        AppPlatform.ios,
        AppPlatform.android,
        AppPlatform.macos,
        AppPlatform.web,
      ]) {
        final c = build(platform: p);
        await c.read(updateProvider.notifier).checkOnLaunch();
        await c.read(updateProvider.notifier).checkNow();
        verifyNever(service.check(any));
      }
    });

    test('a launch check is skipped while a session is running', () async {
      final c = build(sessionActive: true);
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkOnLaunch();
      verifyNever(service.check(any));
    });

    test('a manual check still runs during a session', () async {
      final c = build(sessionActive: true);
      answers(const UpdateUpToDate());
      await c.read(updateProvider.notifier).checkNow();
      verify(service.check(any)).called(1);
    });

    test('an unparseable running version disables the updater', () async {
      final c = build(current: 'not-a-version');
      answers(const UpdateUpToDate());
      await c.read(updateProvider.notifier).checkNow();
      verifyNever(service.check(any));
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });
  });

  group('throttle', () {
    test('a launch check runs when nothing was ever recorded', () async {
      final c = build();
      answers(const UpdateUpToDate());
      await c.read(updateProvider.notifier).checkOnLaunch();
      verify(service.check(any)).called(1);
      expect(prefs.store[kUpdateLastCheckPrefKey], isNotNull);
    });

    test('a launch check inside 24 h is suppressed', () async {
      final c = build(
        stored: {
          kUpdateLastCheckPrefKey: DateTime.utc(
            2026,
            8,
            30,
            1,
          ).toIso8601String(),
        },
      );
      answers(const UpdateUpToDate());
      await c.read(updateProvider.notifier).checkOnLaunch();
      verifyNever(service.check(any));
    });

    test('a launch check past 24 h runs', () async {
      final c = build(
        stored: {
          kUpdateLastCheckPrefKey: DateTime.utc(
            2026,
            8,
            29,
            11,
          ).toIso8601String(),
        },
      );
      answers(const UpdateUpToDate());
      await c.read(updateProvider.notifier).checkOnLaunch();
      verify(service.check(any)).called(1);
    });

    test('a manual check ignores the throttle', () async {
      final c = build(
        stored: {
          kUpdateLastCheckPrefKey: DateTime.utc(
            2026,
            8,
            30,
            11,
          ).toIso8601String(),
        },
      );
      answers(const UpdateUpToDate());
      await c.read(updateProvider.notifier).checkNow();
      verify(service.check(any)).called(1);
    });

    test(
      'a corrupt or future timestamp does not suppress checks forever',
      () async {
        for (final stored in ['nonsense', '', '2099-01-01T00:00:00Z']) {
          final c = build(stored: {kUpdateLastCheckPrefKey: stored});
          answers(const UpdateUpToDate());
          await c.read(updateProvider.notifier).checkOnLaunch();
          verify(service.check(any)).called(1);
        }
      },
    );
  });

  group('offering', () {
    test('offers a newer release inside the rollout', () async {
      final c = build(stored: {kRolloutBucketPrefKey: '10'});
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 25,
        ),
      );
      await c.read(updateProvider.notifier).checkOnLaunch();
      final state = c.read(updateProvider);
      expect(state, isA<UpdateAvailableState>());
      expect((state as UpdateAvailableState).canSelfInstall, isTrue);
    });

    test('stays quiet outside the rollout on an automatic check', () async {
      final c = build(stored: {kRolloutBucketPrefKey: '90'});
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 25,
        ),
      );
      await c.read(updateProvider.notifier).checkOnLaunch();
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });

    test('rollout 0 is the kill-switch', () async {
      final c = build(stored: {kRolloutBucketPrefKey: '0'});
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 0,
        ),
      );
      await c.read(updateProvider.notifier).checkOnLaunch();
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });

    test('a manual check bypasses the rollout bucket', () async {
      final c = build(stored: {kRolloutBucketPrefKey: '90'});
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 1,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      expect(c.read(updateProvider), isA<UpdateAvailableState>());
    });

    test('a skipped version is not re-offered automatically', () async {
      final c = build(
        stored: {
          kRolloutBucketPrefKey: '0',
          kUpdateSkippedVersionPrefKey: '1.25.0+34',
        },
      );
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkOnLaunch();
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });

    test('a NEWER version is still offered after a skip', () async {
      final c = build(
        stored: {
          kRolloutBucketPrefKey: '0',
          kUpdateSkippedVersionPrefKey: '1.25.0+34',
        },
      );
      answers(
        UpdateAvailable(
          manifest: manifest(version: '1.26.0+40'),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkOnLaunch();
      expect(c.read(updateProvider), isA<UpdateAvailableState>());
    });

    test('a manual check surfaces a skipped version again', () async {
      final c = build(stored: {kUpdateSkippedVersionPrefKey: '1.25.0+34'});
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      expect(c.read(updateProvider), isA<UpdateAvailableState>());
    });

    test('a portable install is offered notify-only', () async {
      final c = build(canSelfInstall: false);
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      expect(
        (c.read(updateProvider) as UpdateAvailableState).canSelfInstall,
        isFalse,
      );
    });

    test(
      'below min_supported_version it forces, ignoring rollout and skip',
      () async {
        final c = build(
          stored: {
            kRolloutBucketPrefKey: '99',
            kUpdateSkippedVersionPrefKey: '1.25.0+34',
          },
        );
        answers(
          UpdateAvailable(
            manifest: manifest(min: '1.25.0+34'),
            target: _target,
            rolloutPercent: 1,
          ),
        );
        await c.read(updateProvider.notifier).checkOnLaunch();
        // A client that cannot talk to the backend must never be left stranded by
        // a staged rollout or a previous "skip".
        expect(c.read(updateProvider), isA<UpdateRequired>());
      },
    );

    test('at or above min_supported_version it is a normal offer', () async {
      final c = build(
        current: '1.25.0+34',
        stored: {kRolloutBucketPrefKey: '0'},
      );
      answers(
        UpdateAvailable(
          manifest: manifest(version: '1.26.0+40', min: '1.25.0+34'),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      expect(c.read(updateProvider), isA<UpdateAvailableState>());
    });
  });

  group('failures', () {
    test('an automatic check that fails is a silent no-op', () async {
      final c = build();
      answers(const UpdateCheckFailed(UpdateFailureCause.network));
      await c.read(updateProvider.notifier).checkOnLaunch();
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });

    test('a manual check that fails is reported', () async {
      final c = build();
      answers(const UpdateCheckFailed(UpdateFailureCause.network));
      await c.read(updateProvider.notifier).checkNow();
      expect(
        (c.read(updateProvider) as UpdateFailed).cause,
        UpdateFailureCause.network,
      );
      c.read(updateProvider.notifier).acknowledgeFailure();
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });

    test(
      'up-to-date is distinct from idle, so a manual check can say so',
      () async {
        final c = build();
        answers(const UpdateUpToDate());
        await c.read(updateProvider.notifier).checkNow();
        expect(c.read(updateProvider), isA<UpdateUpToDateState>());
      },
    );
  });

  group('download and install', () {
    Future<ProviderContainer> offered({bool canSelfInstall = true}) async {
      final c = build(canSelfInstall: canSelfInstall);
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      return c;
    }

    test('reports progress and hands the artifact to the installer', () async {
      final c = await offered();
      final file = File('${Directory.systemTemp.path}/cymbra-setup.exe');
      final seen = <int>[];
      when(
        service.download(any, onProgress: anyNamed('onProgress')),
      ).thenAnswer((invocation) async {
        final onProgress =
            invocation.namedArguments[#onProgress] as void Function(int, int)?;
        onProgress?.call(512, 1024);
        return UpdateDownloaded(file);
      });
      when(installer.install(any)).thenAnswer((_) async {
        return const InstallStarted();
      });
      c.listen(updateProvider, (_, next) {
        if (next is UpdateDownloading) seen.add(next.received);
      });

      await c.read(updateProvider.notifier).download();
      expect(seen, contains(512));
      verify(installer.install(file)).called(1);
      // The app is exiting: nothing repaints into a misleading "done".
      expect(c.read(updateProvider), isA<UpdateInstalling>());
    });

    test('a download failure is reported and nothing is installed', () async {
      final c = await offered();
      when(
        service.download(any, onProgress: anyNamed('onProgress')),
      ).thenAnswer(
        (_) async => const UpdateDownloadFailed(UpdateFailureCause.integrity),
      );
      await c.read(updateProvider.notifier).download();
      expect(
        (c.read(updateProvider) as UpdateFailed).cause,
        UpdateFailureCause.integrity,
      );
      verifyNever(installer.install(any));
    });

    test('an installer that refuses falls back to notify-only', () async {
      final c = await offered();
      when(
        service.download(any, onProgress: anyNamed('onProgress')),
      ).thenAnswer(
        (_) async => UpdateDownloaded(
          File('${Directory.systemTemp.path}/cymbra-setup.exe'),
        ),
      );
      when(
        installer.install(any),
      ).thenAnswer((_) async => const InstallNotSelfInstallable('portable'));
      await c.read(updateProvider.notifier).download();
      final state = c.read(updateProvider);
      expect(state, isA<UpdateAvailableState>());
      expect((state as UpdateAvailableState).canSelfInstall, isFalse);
    });

    test('download does nothing when there is no offer on the table', () async {
      final c = build();
      await c.read(updateProvider.notifier).download();
      verifyNever(service.download(any, onProgress: anyNamed('onProgress')));
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });
  });

  group('dismissal', () {
    test('later clears the offer without remembering anything', () async {
      final c = build();
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      c.read(updateProvider.notifier).dismiss();
      expect(c.read(updateProvider), isA<UpdateIdle>());
      expect(prefs.store[kUpdateSkippedVersionPrefKey], isNull);
    });

    test('skip remembers the exact version', () async {
      final c = build();
      answers(
        UpdateAvailable(
          manifest: manifest(),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      await c.read(updateProvider.notifier).skipCurrentOffer();
      expect(prefs.store[kUpdateSkippedVersionPrefKey], '1.25.0+34');
      expect(c.read(updateProvider), isA<UpdateIdle>());
    });

    test('a forced update can be neither dismissed nor skipped', () async {
      final c = build();
      answers(
        UpdateAvailable(
          manifest: manifest(min: '1.25.0+34'),
          target: _target,
          rolloutPercent: 100,
        ),
      );
      await c.read(updateProvider.notifier).checkNow();
      c.read(updateProvider.notifier).dismiss();
      await c.read(updateProvider.notifier).skipCurrentOffer();
      expect(c.read(updateProvider), isA<UpdateRequired>());
      expect(prefs.store[kUpdateSkippedVersionPrefKey], isNull);
    });
  });
}
