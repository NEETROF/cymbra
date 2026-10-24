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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/update/process_runner.dart';
import 'package:music/services/update/app_version.dart';
import 'package:music/services/update/update_installer.dart';
import 'package:music/services/update/update_manifest.dart';

/// Records what the installer would have spawned. **No test ever starts a real
/// process or exits the test runner** — that is the entire reason the seam
/// exists (task 7.7).
class RecordingProcessRunner implements ProcessRunner {
  RecordingProcessRunner({this.runExitCode = 0});

  final List<({String executable, List<String> arguments})> started = [];
  final List<({String executable, List<String> arguments})> ran = [];
  int quits = 0;
  int runExitCode;

  /// Set to make [run] throw, modelling a missing executable.
  Object? runError;

  /// Set to make the spawn throw, modelling a missing or locked executable.
  Object? startError;

  @override
  Future<void> startDetached(String executable, List<String> arguments) async {
    if (startError != null) throw startError!;
    started.add((executable: executable, arguments: arguments));
  }

  @override
  Future<int> run(String executable, List<String> arguments) async {
    if (runError != null) throw runError!;
    ran.add((executable: executable, arguments: arguments));
    return runExitCode;
  }

  @override
  void quit() => quits++;
}

const _envelope =
    '{"manifest":"…","signature":"…","key_id":"k","rollout_percent":0}';

UpdateManifest _manifest({String envelope = _envelope}) => UpdateManifest(
  schema: kUpdateSchemaVersion,
  product: 'music',
  channel: 'stable',
  version: AppVersion.tryParse('1.40.0+50')!,
  releasedAt: '2026-10-24T00:00:00Z',
  minSupportedVersion: null,
  notesUrl: null,
  targets: const {},
  signedEnvelope: envelope,
);

void main() {
  late Directory temp;
  late RecordingProcessRunner runner;

  setUp(() {
    temp = Directory.systemTemp.createTempSync('cymbra-installer-test');
    runner = RecordingProcessRunner();
  });
  tearDown(() {
    if (temp.existsSync()) temp.deleteSync(recursive: true);
  });

  File artifact(String name, [String body = 'payload']) =>
      File('${temp.path}/$name')..writeAsStringSync(body);

  group('LinuxPackageUpdateInstaller', () {
    late File executable;
    late File helper;
    late File pkexec;

    /// A package install: the marker beside the executable, the root-owned
    /// helper and pkexec — each a plain file here, nothing is ever run.
    LinuxPackageUpdateInstaller packaged({
      bool marker = true,
      bool withHelper = true,
      bool withPkexec = true,
    }) {
      final opt = Directory('${temp.path}/opt/cymbra-music')
        ..createSync(recursive: true);
      executable = File('${opt.path}/music')..writeAsStringSync('binary');
      if (marker) {
        File('${opt.path}/$kInstallMarker').writeAsStringSync('deb');
      }
      helper = File('${temp.path}/cymbra-update-helper');
      withHelper
          ? helper.writeAsStringSync('helper')
          : (helper.existsSync() ? helper.deleteSync() : null);
      pkexec = File('${temp.path}/pkexec');
      withPkexec
          ? pkexec.writeAsStringSync('pkexec')
          : (pkexec.existsSync() ? pkexec.deleteSync() : null);
      return LinuxPackageUpdateInstaller(
        runner,
        executable: executable.path,
        helperPath: helper.path,
        pkexecPath: pkexec.path,
      );
    }

    File download() {
      final dir = Directory('${temp.path}/download')..createSync();
      return File('${dir.path}/cymbra-music_1.40.0_amd64.deb')
        ..writeAsStringSync('package');
    }

    test(
      'runs pkexec on the helper with the envelope and the package, relaunches, exits',
      () async {
        final installer = packaged();
        final deb = download();

        expect(await installer.canSelfInstall(), isTrue);
        expect(
          await installer.install(deb, _manifest()),
          isA<InstallStarted>(),
        );

        // Exactly one escalation, and only for the packaged helper.
        expect(runner.ran, hasLength(1));
        final call = runner.ran.single;
        expect(call.executable, pkexec.path);
        final envelopePath = '${deb.parent.path}/envelope.json';
        expect(call.arguments, [helper.path, envelopePath, deb.path]);
        // The helper gets the exact feed response, to re-verify with its keys.
        expect(File(envelopePath).readAsStringSync(), _envelope);
        expect(runner.started.single.executable, executable.path);
        expect(runner.quits, 1);
      },
    );

    test(
      'a dismissed password dialog is a cancellation, not an error',
      () async {
        runner.runExitCode = kPkexecDismissed;
        final installer = packaged();
        expect(
          await installer.install(download(), _manifest()),
          isA<InstallCancelled>(),
        );
        expect(runner.started, isEmpty);
        expect(runner.quits, 0);
      },
    );

    for (final code in [2, 3, 127]) {
      test('helper exit $code is a failure and the app stays open', () async {
        runner.runExitCode = code;
        final installer = packaged();
        expect(
          await installer.install(download(), _manifest()),
          isA<InstallFailed>(),
        );
        expect(runner.started, isEmpty);
        expect(runner.quits, 0);
      });
    }

    test('no marker (the tarball, a dev run) is notify-only', () async {
      final installer = packaged(marker: false);
      expect(await installer.canSelfInstall(), isFalse);
      expect(
        await installer.install(download(), _manifest()),
        isA<InstallNotSelfInstallable>(),
      );
      expect(runner.ran, isEmpty);
    });

    test(
      'a missing helper or pkexec is notify-only — never another route',
      () async {
        expect(await packaged(withHelper: false).canSelfInstall(), isFalse);
        expect(await packaged(withPkexec: false).canSelfInstall(), isFalse);
        expect(runner.ran, isEmpty);
      },
    );

    test('no signed envelope means nothing is escalated', () async {
      final installer = packaged();
      expect(
        await installer.install(download(), _manifest(envelope: '')),
        isA<InstallFailed>(),
      );
      expect(runner.ran, isEmpty);
    });

    test('a pkexec that cannot be run is a failure', () async {
      runner.runError = const ProcessException('pkexec', [], 'boom');
      expect(
        await packaged().install(download(), _manifest()),
        isA<InstallFailed>(),
      );
      expect(runner.quits, 0);
    });

    test(
      'a failed relaunch is reported, the upgrade having happened',
      () async {
        runner.startError = const ProcessException('music', [], 'boom');
        expect(
          await packaged().install(download(), _manifest()),
          isA<InstallFailed>(),
        );
        expect(runner.quits, 0);
      },
    );
  });

  test('NoopUpdateInstaller never self-installs', () async {
    const installer = NoopUpdateInstaller();
    expect(await installer.canSelfInstall(), isFalse);
    expect(
      await installer.install(artifact('anything'), _manifest()),
      isA<InstallNotSelfInstallable>(),
    );
  });
}
