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

import 'package:freezed_annotation/freezed_annotation.dart';

import '../services/update/app_version.dart';
import '../services/update/desktop_update_service.dart';
import '../services/update/update_manifest.dart';

part 'update_state.freezed.dart';

/// The desktop updater's state (change: add-desktop-auto-update, design D8).
///
/// One union, so every screen renders from an exhaustive `switch` and there is
/// no combination of scattered `loading`/`error`/`available` flags that can
/// contradict itself. Side effects (banner, dialog, blocking screen) live in the
/// dedicated listener widget, never in a build method.
@freezed
sealed class UpdateState with _$UpdateState {
  /// Nothing in flight and nothing to offer — including "the feature is off",
  /// "not a desktop build" and "the check failed silently".
  const factory UpdateState.idle() = UpdateIdle;

  /// A check is running. Only a *manual* check is worth showing.
  const factory UpdateState.checking() = UpdateChecking;

  /// A check finished and this build is current. Distinct from [UpdateIdle] so a
  /// manual check can say "you're up to date" — an automatic one stays quiet.
  const factory UpdateState.upToDate() = UpdateUpToDateState;

  /// A verified, strictly newer release is on offer.
  const factory UpdateState.available({
    required UpdateManifest manifest,
    required UpdateTarget target,

    /// False for a portable zip / tarball / read-only directory: the prompt then
    /// offers the download page instead of an in-app update.
    required bool canSelfInstall,
  }) = UpdateAvailableState;

  /// Downloading. [total] is the manifest's declared size, so progress is known
  /// from the first byte.
  const factory UpdateState.downloading({
    required UpdateManifest manifest,
    required int received,
    required int total,
  }) = UpdateDownloading;

  /// Verified on disk and ready to install.
  const factory UpdateState.ready({required UpdateManifest manifest}) =
      UpdateReady;

  /// The installer has been handed the job; the app is about to exit.
  const factory UpdateState.installing({required UpdateManifest manifest}) =
      UpdateInstalling;

  /// A user-visible failure. Only ever reached from an action the user started —
  /// an automatic check failing is a silent [UpdateIdle] with a logged cause.
  const factory UpdateState.failed(UpdateFailureCause cause) = UpdateFailed;

  /// The running build is below the manifest's `min_supported_version`: a
  /// blocking, localized screen, dismissible only by updating.
  const factory UpdateState.updateRequired({
    required UpdateManifest manifest,
    required UpdateTarget target,
    required bool canSelfInstall,
    required AppVersion current,
  }) = UpdateRequired;

  const UpdateState._();

  /// The version being offered, when there is one.
  AppVersion? get offeredVersion => switch (this) {
    UpdateAvailableState(:final manifest) ||
    UpdateDownloading(:final manifest) ||
    UpdateReady(:final manifest) ||
    UpdateInstalling(:final manifest) ||
    UpdateRequired(:final manifest) => manifest.version,
    _ => null,
  };

  /// Whether an operation is in flight — the notifier refuses to start a second
  /// one, so a double tap cannot download twice.
  bool get isBusy => switch (this) {
    UpdateChecking() || UpdateDownloading() || UpdateInstalling() => true,
    _ => false,
  };
}
