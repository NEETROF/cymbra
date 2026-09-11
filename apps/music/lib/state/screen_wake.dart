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

import 'dart:async';

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/screen_wake_service.dart';

part 'screen_wake.freezed.dart';
part 'screen_wake.g.dart';

/// Whether the device's screen must be held awake, and why (change:
/// keep-play-surfaces-awake).
@freezed
abstract class ScreenWakeState with _$ScreenWakeState {
  const factory ScreenWakeState({
    /// How many play surfaces are currently mounted.
    ///
    /// A count, not a flag: play surfaces stack — the measure selector opens
    /// over the player, a lesson can host a player — and with a flag the inner
    /// screen's dismissal would drop a hold the screen still underneath it
    /// depends on.
    @Default(0) int holders,

    /// Whether Cymbra is the foreground app.
    ///
    /// Part of the same state rather than a separate owner because the answer
    /// to "hold the screen?" is a function of both inputs; splitting them is
    /// how you get a hold re-taken on resume for a screen that has since been
    /// popped.
    @Default(true) bool foreground,
  }) = _ScreenWakeState;

  const ScreenWakeState._();

  /// The only thing the platform is ever told.
  bool get shouldHold => holders > 0 && foreground;
}

/// Owns the screen-awake hold for the whole app.
///
/// Play surfaces — screens whose normal use is reading them while the player's
/// hands are on an instrument, so they produce no touch events for minutes at a
/// time — [acquire] on mount and [release] on dispose, via the `KeepScreenAwake`
/// wrapper. The lifecycle observer reports the foreground.
///
/// The hold deliberately spans the whole visit rather than tracking playback:
/// Wait Mode *stops* the playhead while the player works out an onset, and a
/// stopped playhead is not an idle player but a struggling one.
@Riverpod(keepAlive: true)
class ScreenWake extends _$ScreenWake {
  /// The wrapper widget defers its acquire/release off the widget life-cycle
  /// (Riverpod forbids mutating a provider there), so a call can land after the
  /// container is gone — app exit, or a test's tear-down between frames.
  bool _disposed = false;

  @override
  ScreenWakeState build() {
    ref.onDispose(() => _disposed = true);
    return const ScreenWakeState();
  }

  /// One more play surface is on screen.
  void acquire() => _update(state.copyWith(holders: state.holders + 1));

  /// One fewer play surface. Clamped at zero so an unbalanced release (a widget
  /// disposed twice, a hot reload) cannot make a later [acquire] a no-op.
  void release() {
    if (_disposed || state.holders == 0) return;
    _update(state.copyWith(holders: state.holders - 1));
  }

  /// Whether Cymbra is the foreground app.
  void setForeground(bool foreground) =>
      _update(state.copyWith(foreground: foreground));

  /// Applies [next], telling the platform **only when the answer flips** — so a
  /// stack of five surfaces is one call, not five.
  void _update(ScreenWakeState next) {
    if (_disposed) return;
    final was = state.shouldHold;
    state = next;
    if (next.shouldHold == was) return;
    unawaited(
      ref.read(screenWakeServiceProvider).setEnabled(enabled: next.shouldHold),
    );
  }
}
