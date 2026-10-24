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

import 'dart:io' show File, FileSystemEntity, Platform;

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../app_platform.dart';
import 'process_runner.dart';
import 'update_manifest.dart';

part 'update_installer.g.dart';

/// The file the Inno installer (and the Linux package) drops beside the
/// executable to mark a managed install (design D4/D5). The portable zip and the
/// tarball have no marker, so they take the notify-only path.
const kWindowsInstallMarker = 'install_method.txt';

/// What happened when the updater tried to install a verified artifact.
sealed class InstallOutcome {
  const InstallOutcome();
}

/// The installer was launched (or the swap completed) and the app is about to
/// exit. Nothing after this should assume the process is still alive.
class InstallStarted extends InstallOutcome {
  const InstallStarted();
}

/// This install layout cannot update itself — a portable zip, a tarball, a
/// read-only directory. Not a failure: the user is pointed at the download page
/// instead. The updater NEVER escalates privileges to work around this.
class InstallNotSelfInstallable extends InstallOutcome {
  const InstallNotSelfInstallable(this.reason);

  /// For the log only. The UI shows a localized "download the new version"
  /// message; a raw reason never reaches the screen.
  final String reason;
}

/// The user dismissed the system password dialog. Not a failure: nothing was
/// installed and the offer stays available.
class InstallCancelled extends InstallOutcome {
  const InstallCancelled();
}

/// The install was attempted and failed.
class InstallFailed extends InstallOutcome {
  const InstallFailed(this.reason);
  final String reason;
}

/// Applies a verified update (change: add-desktop-auto-update, design D4/D5).
/// An abstract seam so state and widgets are testable without spawning anything.
abstract class UpdateInstaller {
  /// Whether this install layout can replace itself at all. The UI asks first,
  /// so the prompt offers "Update" or "Download" rather than offering to update
  /// and then failing.
  Future<bool> canSelfInstall();

  /// Installs [artifact] — a file that has already been signature-verified,
  /// size-checked and hash-checked against [manifest]. On success the app exits.
  Future<InstallOutcome> install(File artifact, UpdateManifest manifest);
}

/// Windows: hand the verified installer the job and get out of the way
/// (design D4).
///
/// A running `.exe` cannot be overwritten on Windows, which is why the installer
/// — not the app — performs the swap, and why this exits immediately after
/// spawning it. The per-user install location is what makes the silent run
/// possible with **no UAC prompt**; a Program Files install would elevate on
/// every update, which in practice means nobody updates.
class WindowsUpdateInstaller implements UpdateInstaller {
  WindowsUpdateInstaller(this._runner, {String? executableDir})
    : _executableDir =
          executableDir ?? File(Platform.resolvedExecutable).parent.path;

  final ProcessRunner _runner;
  final String _executableDir;

  /// `/VERYSILENT` (no UI at all) + `/SUPPRESSMSGBOXES` (nothing can block on a
  /// dialog nobody will see) + `/NORESTART` (never reboot the machine behind the
  /// user) + `/CLOSEAPPLICATIONS` and `/RESTARTAPPLICATIONS` (Restart Manager
  /// handles the files this process still holds). The relaunch itself is the
  /// installer's `[Run]` entry, flagged `postinstall nowait` WITHOUT
  /// `skipifsilent` — without that flag combination a silent update leaves the
  /// user staring at a closed app.
  static const List<String> silentArguments = [
    '/VERYSILENT',
    '/SUPPRESSMSGBOXES',
    '/NORESTART',
    '/CLOSEAPPLICATIONS',
    '/RESTARTAPPLICATIONS',
  ];

  @override
  Future<bool> canSelfInstall() async =>
      FileSystemEntity.isFileSync('$_executableDir/$kWindowsInstallMarker');

  @override
  Future<InstallOutcome> install(File artifact, UpdateManifest manifest) async {
    if (!await canSelfInstall()) {
      return const InstallNotSelfInstallable(
        'no install marker beside the executable (portable zip)',
      );
    }
    try {
      await _runner.startDetached(artifact.path, silentArguments);
    } on Object catch (e) {
      debugPrint('desktop update: could not start the installer ($e)');
      return InstallFailed('spawn failed: $e');
    }
    _runner.quit();
    return const InstallStarted();
  }
}

/// The privileged helper the `.deb` installs, root-owned (design D5).
const kLinuxUpdateHelper = '/usr/lib/cymbra-music/cymbra-update-helper';

/// The system tool that asks for the password and runs the helper as root.
const kPkexec = '/usr/bin/pkexec';

/// `pkexec`'s exit code when the user dismissed the authentication dialog.
const kPkexecDismissed = 126;

/// Linux: a `.deb` upgraded through the system password dialog (design D5).
///
/// The package leaves [kWindowsInstallMarker] beside the executable, like the
/// Windows installer, and installs [kLinuxUpdateHelper]. The app writes the
/// signed envelope beside the verified package and runs
/// `pkexec <helper> <envelope> <package>`. The helper trusts neither file: it
/// copies the package somewhere only root can write, re-verifies the envelope
/// with its own compiled-in keys and the copy against the manifest, and only
/// then installs it — so a process of the user swapping the file while the
/// password dialog is up changes nothing.
///
/// No marker (the tarball, a dev run), no helper or no `pkexec` ⇒ notify-only.
/// The only escalation is that one helper, through the system's own dialog.
class LinuxPackageUpdateInstaller implements UpdateInstaller {
  LinuxPackageUpdateInstaller(
    this._runner, {
    String? executable,
    this.helperPath = kLinuxUpdateHelper,
    this.pkexecPath = kPkexec,
  }) : _executable = executable ?? Platform.resolvedExecutable;

  final ProcessRunner _runner;
  final String _executable;
  final String helperPath;
  final String pkexecPath;

  @override
  Future<bool> canSelfInstall() async =>
      FileSystemEntity.isFileSync(
        '${File(_executable).parent.path}/$kWindowsInstallMarker',
      ) &&
      FileSystemEntity.isFileSync(helperPath) &&
      FileSystemEntity.isFileSync(pkexecPath);

  @override
  Future<InstallOutcome> install(File artifact, UpdateManifest manifest) async {
    if (!await canSelfInstall()) {
      return const InstallNotSelfInstallable(
        'no package marker, helper or pkexec (tarball install or a dev run)',
      );
    }
    if (manifest.signedEnvelope.isEmpty) {
      return const InstallFailed('no signed envelope to hand to the helper');
    }
    final int code;
    try {
      // Beside the package, in the download's private directory: the helper
      // reads both and works on its own copy from then on.
      final envelope = File('${artifact.parent.path}/envelope.json');
      await envelope.writeAsString(manifest.signedEnvelope, flush: true);
      code = await _runner.run(pkexecPath, [
        helperPath,
        envelope.path,
        artifact.path,
      ]);
    } on Object catch (e) {
      debugPrint('desktop update: could not run the update helper ($e)');
      return InstallFailed('helper spawn failed: $e');
    }
    if (code == kPkexecDismissed) return const InstallCancelled();
    if (code != 0) {
      // 2 refused, 3 apt failed, 127 no polkit agent — the log says which.
      debugPrint('desktop update: the update helper exited with $code');
      return InstallFailed('helper exited with $code');
    }
    try {
      await _runner.startDetached(_executable, const []);
    } on Object catch (e) {
      // The new package IS installed; only the relaunch failed. The next
      // manual launch is the new version.
      debugPrint('desktop update: relaunch failed after the upgrade ($e)');
      return InstallFailed('relaunch failed: $e');
    }
    _runner.quit();
    return const InstallStarted();
  }
}

/// An installer for platforms that never self-update (store-managed, web) — and
/// the safe default when the platform is unknown.
class NoopUpdateInstaller implements UpdateInstaller {
  const NoopUpdateInstaller();

  @override
  Future<bool> canSelfInstall() async => false;

  @override
  Future<InstallOutcome> install(
    File artifact,
    UpdateManifest manifest,
  ) async =>
      const InstallNotSelfInstallable('this platform does not self-update');
}

/// The per-platform installer, behind a provider so tests override it.
@Riverpod(keepAlive: true)
UpdateInstaller updateInstaller(Ref ref) {
  final runner = ref.watch(processRunnerProvider);
  return switch (ref.watch(appPlatformProvider)) {
    AppPlatform.windows => WindowsUpdateInstaller(runner),
    AppPlatform.linux => LinuxPackageUpdateInstaller(runner),
    _ => const NoopUpdateInstaller(),
  };
}
