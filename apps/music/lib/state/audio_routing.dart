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

import '../services/audio_routing_service.dart';
import '../services/audio_service.dart';
import 'player_preferences.dart';

export '../services/audio_routing_service.dart' show AudioRoute, AudioRouteKind;

part 'audio_routing.freezed.dart';
part 'audio_routing.g.dart';

/// Where the app's audio goes, as the sound-output section sees it (change:
/// add-audio-output-routing).
@freezed
abstract class AudioRoutingState with _$AudioRoutingState {
  const factory AudioRoutingState({
    /// Outputs the user may choose from. Always empty on mobile, where the OS
    /// owns the route.
    @Default(<AudioRoute>[]) List<AudioRoute> outputs,

    /// The route audio is **actually** going through — never the one that was
    /// requested, so a fallback is visible rather than hidden.
    AudioRoute? active,

    /// Whether the app can pick an output itself (desktop) rather than only
    /// presenting the system's picker (mobile).
    @Default(false) bool canSelectDevice,

    /// Set when the last requested device could not be opened: the previously
    /// working audio kept running and the user is told, once.
    @Default(false) bool selectionFailed,
  }) = _AudioRoutingState;

  const AudioRoutingState._();

  /// Whether the active route delays the sound enough to matter when playing.
  bool get isWireless => active?.kind.isWireless ?? false;
}

/// The audio routing section's notifier: reads the platform's outputs and active
/// route, applies a selection, and re-reads reality afterwards.
@Riverpod(keepAlive: true)
class AudioRouting extends _$AudioRouting {
  /// How long to wait for the engine to actually be on the requested device
  /// before calling the selection failed. The audio thread rebuilds its stream
  /// off the calling thread, so the change is never instantaneous — but it is
  /// fast, and a device that will not open never gets there at all.
  static const Duration _settleStep = Duration(milliseconds: 120);
  static const int _settleAttempts = 8;

  AudioRoutingService get _service => ref.read(audioRoutingServiceProvider);

  @override
  AudioRoutingState build() {
    unawaited(_restore());
    return const AudioRoutingState();
  }

  /// Loads the platform's outputs, re-applies the remembered device and starts
  /// following route changes. Never throws: the seam already degrades to "no
  /// devices / unknown route".
  Future<void> _restore() async {
    // The engine publishes its command channel as part of starting up, so an
    // output remembered from the last session can only be applied once audio
    // init has been asked for. It is idempotent.
    await ref.read(audioServiceProvider).init();

    final service = _service;
    final subscription = service.routeChanges().listen((route) {
      if (route != null) state = state.copyWith(active: route);
    }, onError: (Object _) {});
    ref.onDispose(subscription.cancel);

    final remembered = ref.read(playerPreferencesProvider).audioOutput;
    if (service.supportsDeviceSelection && remembered != null) {
      await service.selectOutput(remembered);
    }
    state = state.copyWith(
      canSelectDevice: service.supportsDeviceSelection,
      outputs: await service.listOutputs(),
      active: await service.activeRoute(),
    );
  }

  /// Re-reads the platform's outputs and active route — after the OS picker is
  /// dismissed, or when the section is opened.
  Future<void> refresh() async {
    final service = _service;
    state = state.copyWith(
      canSelectDevice: service.supportsDeviceSelection,
      outputs: await service.listOutputs(),
      active: await service.activeRoute(),
    );
  }

  /// Sends all app audio to [name] (null = follow the system default) and
  /// remembers it. The state always ends up describing the device **actually**
  /// in use: a device that will not open leaves the working audio alone and
  /// raises [AudioRoutingState.selectionFailed].
  Future<void> selectOutput(String? name) async {
    final service = _service;
    if (!service.supportsDeviceSelection) return;
    state = state.copyWith(selectionFailed: false);
    await service.selectOutput(name);
    ref.read(playerPreferencesProvider.notifier).setAudioOutput(name);
    final active = await _settledRoute(name);
    state = state.copyWith(
      active: active,
      selectionFailed: name != null && active?.name != name,
    );
  }

  /// Shows the operating system's route picker, then re-reads the route it left
  /// active (the OS gives no result back).
  Future<void> presentRoutePicker() async {
    await _service.presentRoutePicker();
    await refresh();
  }

  /// Clears the failure flag once it has been shown, so the same message is not
  /// raised again on the next rebuild.
  void acknowledgeFailure() {
    if (state.selectionFailed) state = state.copyWith(selectionFailed: false);
  }

  /// Polls the engine until it reports [requested] as active, giving the audio
  /// thread time to rebuild its stream. Returns whatever is active when the
  /// budget runs out — which is exactly what should be displayed.
  Future<AudioRoute?> _settledRoute(String? requested) async {
    var active = await _service.activeRoute();
    if (requested == null) return active;
    for (var attempt = 0; attempt < _settleAttempts; attempt++) {
      if (active?.name == requested) return active;
      await Future<void>.delayed(_settleStep);
      active = await _service.activeRoute();
    }
    return active;
  }
}
