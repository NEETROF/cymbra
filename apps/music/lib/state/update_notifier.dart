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

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/app_platform.dart';
import '../services/preferences_service.dart';
import '../services/update/app_version.dart';
import '../services/update/desktop_update_service.dart';
import '../services/update/rollout_bucket.dart';
import '../services/update/update_installer.dart';
import 'rating_activity_notifier.dart' show nowFnProvider;
import 'update_state.dart';

part 'update_notifier.g.dart';

/// The runtime flag that gates the whole feature (task 8.4).
///
/// Plain `false` by default so tests never build the flag client; `main.dart`
/// overrides it with the remote `desktop.auto_update.enabled` flag. Off, the
/// updater never checks, never prompts and never blocks — including the forced
/// update, because a flag that cannot turn the feature off completely is not a
/// kill-switch.
@Riverpod(keepAlive: true)
bool desktopUpdateEnabled(Ref ref) => false;

/// The remote flag key.
const kDesktopUpdateFlag = 'desktop.auto_update.enabled';

/// Whether a play or practice session is running right now (task 8.6).
///
/// Plain `false` by default; `main.dart` wires it to the player. Interrupting
/// someone mid-piece with an update prompt is the fastest way to make the prompt
/// something people learn to dismiss without reading.
@Riverpod(keepAlive: true)
bool updateSessionActive(Ref ref) => false;

/// How long a launch check is suppressed after the previous one.
const kUpdateCheckThrottle = Duration(hours: 24);

/// When the last check ran.
const kUpdateLastCheckPrefKey = 'desktop_update_last_check_at';

/// The version the user chose to skip.
const kUpdateSkippedVersionPrefKey = 'desktop_update_skipped_version';

/// The desktop updater (change: add-desktop-auto-update, design D8).
///
/// Owns *when* to look, *whether* to offer, and the download/install sequence.
/// Every side effect the user sees — banner, dialog, blocking screen — is the
/// listener widget's job, and the UI never awaits an action's return: it fires
/// the action and reacts to the resulting state.
@Riverpod(keepAlive: true)
class Update extends _$Update {
  @override
  UpdateState build() => const UpdateState.idle();

  PreferencesService get _prefs => ref.read(preferencesServiceProvider);

  /// Whether this build participates in the updater at all: the flag is on and
  /// the platform is one that ships outside a store.
  bool get _eligible {
    if (!ref.read(desktopUpdateEnabledProvider)) return false;
    final platform = ref.read(appPlatformProvider);
    // macOS/iOS/Android are store-managed and web has no artifact — consulting
    // the feed there would offer a Windows installer to a Mac.
    return platform == AppPlatform.windows || platform == AppPlatform.linux;
  }

  /// The throttled launch check. A no-op when the feature is off, on a
  /// store-managed platform, while something is already in flight, while a
  /// session is running, or within [kUpdateCheckThrottle] of the last check.
  Future<void> checkOnLaunch() async {
    if (!_eligible || state.isBusy) return;
    // Never spend a launch check's network + verification on someone who is
    // mid-piece; the next launch (or the settings entry) will do it.
    if (ref.read(updateSessionActiveProvider)) return;
    if (!await _throttleElapsed()) return;
    await _check(manual: false);
  }

  /// The explicit "check for updates" action. Ignores the throttle, the rollout
  /// bucket and any skipped version: the user asked, so the answer is what the
  /// feed actually offers.
  Future<void> checkNow() async {
    if (!_eligible || state.isBusy) return;
    await _check(manual: true);
  }

  Future<bool> _throttleElapsed() async {
    final raw = await _prefs.getString(kUpdateLastCheckPrefKey);
    if (raw == null) return true;
    final last = DateTime.tryParse(raw);
    if (last == null) return true;
    final now = ref.read(nowFnProvider)();
    // A stored timestamp in the future (a clock that was wound back) would
    // otherwise suppress checks indefinitely.
    if (last.isAfter(now)) return true;
    return now.difference(last) >= kUpdateCheckThrottle;
  }

  Future<void> _check({required bool manual}) async {
    state = const UpdateState.checking();
    final current = await ref.read(currentAppVersionProvider.future);
    if (current == null) {
      state = const UpdateState.idle();
      return;
    }
    final outcome = await ref.read(desktopUpdateServiceProvider).check(current);
    await _rememberCheckTime();
    switch (outcome) {
      case UpdateCheckFailed(:final cause):
        debugPrint('desktop update: check failed ($cause)');
        // An automatic check that fails is a silent no-op: the user did not ask,
        // so an error on screen would be noise they cannot act on.
        state = manual ? UpdateState.failed(cause) : const UpdateState.idle();
      case UpdateUpToDate():
        state = const UpdateState.upToDate();
      case UpdateAvailable():
        state = await _offer(outcome, current: current, manual: manual);
    }
  }

  Future<UpdateState> _offer(
    UpdateAvailable available, {
    required AppVersion current,
    required bool manual,
  }) async {
    final canSelfInstall = await ref
        .read(updateInstallerProvider)
        .canSelfInstall();
    // The floor first, and unconditionally: a client below
    // `min_supported_version` cannot talk to the backend, so neither a staged
    // rollout nor a previous "skip" may leave it stranded on errors it cannot
    // act on.
    if (available.forcesUpdate(current)) {
      return UpdateState.updateRequired(
        manifest: available.manifest,
        target: available.target,
        canSelfInstall: canSelfInstall,
        current: current,
      );
    }
    if (!manual) {
      final skipped = await _prefs.getString(kUpdateSkippedVersionPrefKey);
      if (skipped == available.manifest.version.toString()) {
        return const UpdateState.idle();
      }
      final included = await ref
          .read(rolloutBucketProvider)
          .isIncluded(available.rolloutPercent);
      if (!included) return const UpdateState.idle();
    }
    return UpdateState.available(
      manifest: available.manifest,
      target: available.target,
      canSelfInstall: canSelfInstall,
    );
  }

  Future<void> _rememberCheckTime() async {
    try {
      await _prefs.setString(
        kUpdateLastCheckPrefKey,
        ref.read(nowFnProvider)().toIso8601String(),
      );
    } on Object catch (e) {
      // A preferences failure must not turn a successful check into an error —
      // the worst case is that the next launch checks again.
      debugPrint('desktop update: could not record the check time ($e)');
    }
  }

  /// Downloads the offered artifact, then installs it. The UI fires this and
  /// reacts to the state; it never awaits the return.
  Future<void> download() async {
    final offer = switch (state) {
      UpdateAvailableState(:final manifest, :final target) => (
        manifest,
        target,
      ),
      UpdateRequired(:final manifest, :final target) => (manifest, target),
      _ => null,
    };
    if (offer == null) return;
    final (manifest, target) = offer;

    state = UpdateState.downloading(
      manifest: manifest,
      received: 0,
      total: target.size,
    );
    final outcome = await ref
        .read(desktopUpdateServiceProvider)
        .download(
          target,
          onProgress: (received, total) {
            // Ignore progress that arrives after the user cancelled or a second
            // operation took over.
            if (state is! UpdateDownloading) return;
            state = UpdateState.downloading(
              manifest: manifest,
              received: received,
              total: total,
            );
          },
        );
    switch (outcome) {
      case UpdateDownloadFailed(:final cause):
        debugPrint('desktop update: download failed ($cause)');
        state = UpdateState.failed(cause);
      case UpdateDownloaded(:final file):
        state = UpdateState.ready(manifest: manifest);
        state = UpdateState.installing(manifest: manifest);
        final result = await ref.read(updateInstallerProvider).install(file);
        switch (result) {
          case InstallStarted():
            // The app is exiting; leave the state as `installing` so nothing
            // repaints into a misleading "done".
            break;
          case InstallNotSelfInstallable(:final reason):
            debugPrint('desktop update: not self-installable ($reason)');
            state = UpdateState.available(
              manifest: manifest,
              target: target,
              canSelfInstall: false,
            );
          case InstallFailed(:final reason):
            debugPrint('desktop update: install failed ($reason)');
            state = const UpdateState.failed(UpdateFailureCause.storage);
        }
    }
  }

  /// "Later" — the offer disappears until the next check.
  void dismiss() {
    if (state is UpdateRequired) return; // a forced update is not dismissible
    state = const UpdateState.idle();
  }

  /// "Skip this version" — remembered, so the same release is not re-offered.
  /// A *newer* one still is, and a manual check still surfaces it.
  Future<void> skipCurrentOffer() async {
    final version = state.offeredVersion;
    if (version == null || state is UpdateRequired) return;
    try {
      await _prefs.setString(kUpdateSkippedVersionPrefKey, version.toString());
    } on Object catch (e) {
      debugPrint('desktop update: could not record the skipped version ($e)');
    }
    state = const UpdateState.idle();
  }

  /// Clears a user-visible failure so the surface can be closed.
  void acknowledgeFailure() {
    if (state is UpdateFailed) state = const UpdateState.idle();
  }
}
