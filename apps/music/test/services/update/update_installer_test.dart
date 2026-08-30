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
import 'package:music/services/update/update_installer.dart';

/// Records what the installer would have spawned. **No test ever starts a real
/// process or exits the test runner** — that is the entire reason the seam
/// exists (task 7.7).
class RecordingProcessRunner implements ProcessRunner {
  RecordingProcessRunner({this.chmodExitCode = 0});

  final List<({String executable, List<String> arguments})> started = [];
  final List<({String executable, List<String> arguments})> ran = [];
  int quits = 0;
  int chmodExitCode;

  /// Set to make the spawn throw, modelling a missing or locked executable.
  Object? startError;

  @override
  Future<void> startDetached(String executable, List<String> arguments) async {
    if (startError != null) throw startError!;
    started.add((executable: executable, arguments: arguments));
  }

  @override
  Future<int> run(String executable, List<String> arguments) async {
    ran.add((executable: executable, arguments: arguments));
    return chmodExitCode;
  }

  @override
  void quit() => quits++;
}

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

  group('WindowsUpdateInstaller', () {
    Directory installDir({required bool marker}) {
      final dir = Directory('${temp.path}/install')..createSync();
      if (marker) {
        File(
          '${dir.path}/$kWindowsInstallMarker',
        ).writeAsStringSync('inno-setup');
      }
      return dir;
    }

    test(
      'spawns the installer with the exact silent arguments, then exits',
      () async {
        final dir = installDir(marker: true);
        final installer = WindowsUpdateInstaller(
          runner,
          executableDir: dir.path,
        );
        final setup = artifact('cymbra-setup.exe');

        expect(await installer.canSelfInstall(), isTrue);
        expect(await installer.install(setup), isA<InstallStarted>());

        expect(runner.started, hasLength(1));
        expect(runner.started.single.executable, setup.path);
        // Every flag matters: /VERYSILENT + /SUPPRESSMSGBOXES so nothing blocks on
        // a dialog nobody sees, /NORESTART so the machine is never rebooted behind
        // the user, /CLOSEAPPLICATIONS + /RESTARTAPPLICATIONS for the files this
        // process still holds.
        expect(runner.started.single.arguments, [
          '/VERYSILENT',
          '/SUPPRESSMSGBOXES',
          '/NORESTART',
          '/CLOSEAPPLICATIONS',
          '/RESTARTAPPLICATIONS',
        ]);
        // A running .exe cannot be overwritten on Windows: the app MUST exit.
        expect(runner.quits, 1);
      },
    );

    test(
      'no marker (the portable zip) is notify-only, and spawns nothing',
      () async {
        final dir = installDir(marker: false);
        final installer = WindowsUpdateInstaller(
          runner,
          executableDir: dir.path,
        );
        expect(await installer.canSelfInstall(), isFalse);
        expect(
          await installer.install(artifact('cymbra-setup.exe')),
          isA<InstallNotSelfInstallable>(),
        );
        expect(runner.started, isEmpty);
        expect(runner.quits, 0);
      },
    );

    test('a failed spawn is a failure, and the app does NOT exit', () async {
      final dir = installDir(marker: true);
      runner.startError = const ProcessException('setup.exe', [], 'boom');
      final installer = WindowsUpdateInstaller(runner, executableDir: dir.path);
      expect(
        await installer.install(artifact('cymbra-setup.exe')),
        isA<InstallFailed>(),
      );
      // Quitting here would close the app with nothing installed.
      expect(runner.quits, 0);
    });
  });

  group('LinuxUpdateInstaller', () {
    test(
      'chmods then renames over the running file, relaunches, and exits',
      () async {
        final running = artifact('Cymbra.AppImage', 'old version');
        final installer = LinuxUpdateInstaller(
          runner,
          environment: {'APPIMAGE': running.path},
        );
        final downloaded = artifact('downloaded.AppImage', 'new version');

        expect(await installer.canSelfInstall(), isTrue);
        expect(await installer.install(downloaded), isA<InstallStarted>());

        // The running path now holds the new bytes — replacing a running
        // executable by rename() is safe on Linux (the process keeps the old
        // inode); overwriting in place is what fails with ETXTBSY.
        expect(running.readAsStringSync(), 'new version');
        // chmod is AWAITED before the rename: a non-executable file renamed into
        // place is an app that no longer starts.
        expect(runner.ran, hasLength(1));
        expect(runner.ran.single.executable, 'chmod');
        expect(runner.ran.single.arguments.first, '0755');
        expect(runner.started.single.executable, running.path);
        expect(runner.quits, 1);
        // The staging file is gone: it WAS the rename source.
        expect(
          temp.listSync().map((e) => e.path.split('/').last),
          isNot(contains(startsWith('.Cymbra.AppImage'))),
        );
      },
    );

    test(
      r'$APPIMAGE unset (tarball install or a dev run) is notify-only',
      () async {
        final installer = LinuxUpdateInstaller(runner, environment: const {});
        expect(await installer.canSelfInstall(), isFalse);
        expect(
          await installer.install(artifact('downloaded.AppImage')),
          isA<InstallNotSelfInstallable>(),
        );
        expect(runner.started, isEmpty);
        expect(runner.quits, 0);
      },
    );

    test(r'an empty $APPIMAGE is treated as unset', () async {
      final installer = LinuxUpdateInstaller(
        runner,
        environment: const {'APPIMAGE': ''},
      );
      expect(await installer.canSelfInstall(), isFalse);
    });

    test(
      'a read-only directory is notify-only — it never escalates',
      () async {
        final readOnly = Directory('${temp.path}/opt')..createSync();
        final running = File('${readOnly.path}/Cymbra.AppImage')
          ..writeAsStringSync('old');
        // 0555: the probe write fails, which is exactly what a root-owned /opt
        // looks like to a normal user.
        Process.runSync('chmod', ['0555', readOnly.path]);
        addTearDown(() => Process.runSync('chmod', ['0755', readOnly.path]));

        final installer = LinuxUpdateInstaller(
          runner,
          environment: {'APPIMAGE': running.path},
        );
        expect(await installer.canSelfInstall(), isFalse);
        expect(
          await installer.install(artifact('downloaded.AppImage')),
          isA<InstallNotSelfInstallable>(),
        );
        expect(runner.started, isEmpty);
        expect(runner.quits, 0);
        // POSIX modes only; the Linux installer never runs on Windows anyway.
      },
      skip: Platform.isWindows ? 'POSIX permissions' : null,
    );

    test(
      'a failed chmod aborts before the rename, leaving the old file',
      () async {
        runner.chmodExitCode = 1;
        final running = artifact('Cymbra.AppImage', 'old version');
        final installer = LinuxUpdateInstaller(
          runner,
          environment: {'APPIMAGE': running.path},
        );
        expect(
          await installer.install(
            artifact('downloaded.AppImage', 'new version'),
          ),
          isA<InstallFailed>(),
        );
        expect(running.readAsStringSync(), 'old version');
        expect(runner.quits, 0);
      },
    );

    test(
      'a failed relaunch is reported, but the swap already happened',
      () async {
        final running = artifact('Cymbra.AppImage', 'old version');
        final installer = LinuxUpdateInstaller(
          runner,
          environment: {'APPIMAGE': running.path},
        );
        runner.startError = const ProcessException('AppImage', [], 'boom');
        final outcome = await installer.install(
          artifact('downloaded.AppImage', 'new version'),
        );
        expect(outcome, isA<InstallFailed>());
        // Saying "the update failed" would be a lie: the next manual launch is new.
        expect(running.readAsStringSync(), 'new version');
        expect(runner.quits, 0);
      },
    );

    test('a stale staging file from a killed attempt is replaced', () async {
      final running = artifact('Cymbra.AppImage', 'old version');
      File('${temp.path}/.Cymbra.AppImage.new').writeAsStringSync('stale');
      final installer = LinuxUpdateInstaller(
        runner,
        environment: {'APPIMAGE': running.path},
      );
      expect(
        await installer.install(artifact('downloaded.AppImage', 'new version')),
        isA<InstallStarted>(),
      );
      expect(running.readAsStringSync(), 'new version');
    });
  });

  test('NoopUpdateInstaller never self-installs', () async {
    const installer = NoopUpdateInstaller();
    expect(await installer.canSelfInstall(), isFalse);
    expect(
      await installer.install(artifact('anything')),
      isA<InstallNotSelfInstallable>(),
    );
  });
}
