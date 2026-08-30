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

import 'dart:io' show File, FileSystemEntity, Directory, Platform;

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../app_platform.dart';
import 'process_runner.dart';

part 'update_installer.g.dart';

/// The file the Inno installer drops in `{app}` to mark an installer-managed
/// install (design D4). The portable zip has no marker, so it takes the
/// notify-only path — it must never try to overwrite its own files.
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
  /// size-checked and hash-checked. On success the app exits.
  Future<InstallOutcome> install(File artifact);
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
  Future<InstallOutcome> install(File artifact) async {
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

/// Linux: an AppImage replaces itself (design D5).
///
/// The AppImage runtime exports `$APPIMAGE` — the absolute path of the running
/// file. The new one is downloaded, moved **into the same directory** (same
/// filesystem, so the swap is a single atomic `rename()`), made executable, and
/// renamed over the running file.
///
/// Replacing a running executable by `rename()` is safe on Linux: the running
/// process keeps the old inode open. Overwriting the file *in place* is what
/// fails, with `ETXTBSY` — which is why the order below is copy-then-rename and
/// never write-through.
///
/// `$APPIMAGE` unset (a tarball install, or `flutter run`) or a directory that
/// is not writable (`/opt`, root-owned) ⇒ notify-only. It never escalates.
class LinuxUpdateInstaller implements UpdateInstaller {
  LinuxUpdateInstaller(this._runner, {Map<String, String>? environment})
    : _environment = environment ?? Platform.environment;

  final ProcessRunner _runner;
  final Map<String, String> _environment;

  /// The running AppImage's path, or `null` when this is not an AppImage.
  String? get appImagePath {
    final value = _environment['APPIMAGE'];
    if (value == null || value.isEmpty) return null;
    return value;
  }

  @override
  Future<bool> canSelfInstall() async {
    final path = appImagePath;
    if (path == null) return false;
    return _directoryIsWritable(File(path).parent);
  }

  @override
  Future<InstallOutcome> install(File artifact) async {
    final path = appImagePath;
    if (path == null) {
      return const InstallNotSelfInstallable(
        r'$APPIMAGE is unset (tarball install or a dev run)',
      );
    }
    final running = File(path);
    final dir = running.parent;
    if (!_directoryIsWritable(dir)) {
      return InstallNotSelfInstallable('${dir.path} is not writable');
    }
    try {
      // Stage beside the target so the final step is a rename WITHIN one
      // filesystem — an atomic swap. Copying across filesystems first and
      // renaming after is the whole point; a cross-device rename would fall back
      // to a non-atomic copy and a crash mid-way would leave a broken install.
      final staged = File('${dir.path}/.${_basename(path)}.new');
      if (await staged.exists()) await staged.delete();
      await artifact.copy(staged.path);
      await _chmodExecutable(staged);
      await staged.rename(running.path);
    } on Object catch (e) {
      debugPrint('desktop update: AppImage swap failed ($e)');
      return InstallFailed('swap failed: $e');
    }
    try {
      await _runner.startDetached(running.path, const []);
    } on Object catch (e) {
      // The new binary IS in place; only the relaunch failed. Say so rather than
      // implying the update did not happen — the next manual launch is new.
      debugPrint(
        'desktop update: relaunch failed after a successful swap ($e)',
      );
      return InstallFailed('relaunch failed: $e');
    }
    _runner.quit();
    return const InstallStarted();
  }

  static String _basename(String path) => path.split('/').last;

  static bool _directoryIsWritable(Directory dir) {
    // Ask the filesystem instead of reasoning about ownership and modes: a probe
    // file is the only answer that accounts for ACLs, read-only mounts and
    // containers.
    try {
      final probe = File('${dir.path}/.cymbra-update-probe');
      probe.writeAsStringSync('');
      probe.deleteSync();
      return true;
    } on Object {
      return false;
    }
  }

  Future<void> _chmodExecutable(File file) async {
    // 0755: the AppImage must be executable by the user who launches it. Awaited
    // rather than fired and forgotten — the rename is the very next step, and a
    // non-executable file renamed into place is an app that no longer starts.
    final code = await _runner.run('chmod', ['0755', file.path]);
    if (code != 0) throw StateError('chmod exited with $code');
  }
}

/// An installer for platforms that never self-update (store-managed, web) — and
/// the safe default when the platform is unknown.
class NoopUpdateInstaller implements UpdateInstaller {
  const NoopUpdateInstaller();

  @override
  Future<bool> canSelfInstall() async => false;

  @override
  Future<InstallOutcome> install(File artifact) async =>
      const InstallNotSelfInstallable('this platform does not self-update');
}

/// The per-platform installer, behind a provider so tests override it.
@Riverpod(keepAlive: true)
UpdateInstaller updateInstaller(Ref ref) {
  final runner = ref.watch(processRunnerProvider);
  return switch (ref.watch(appPlatformProvider)) {
    AppPlatform.windows => WindowsUpdateInstaller(runner),
    AppPlatform.linux => LinuxUpdateInstaller(runner),
    _ => const NoopUpdateInstaller(),
  };
}
