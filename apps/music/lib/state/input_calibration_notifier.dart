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
import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart' show Ref;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/audio_capture_service.dart';
import '../services/preferences_service.dart';
import 'performance_scoring_core.dart';
import 'selected_audio_input.dart';
import 'usage_tracking_notifier.dart';

part 'input_calibration_notifier.freezed.dart';
part 'input_calibration_notifier.g.dart';

/// Where a calibration attempt stands, for the UI (change:
/// add-acoustic-piano-input). Failure reasons are separate states — each has
/// its own localized guidance, never a raw technical string.
enum CalibrationStatus {
  /// No run in progress.
  idle,

  /// A run is in progress (including a pending permission prompt).
  running,

  /// The last run measured a round trip; it is stored for the active route.
  done,

  /// The last run never heard the reference click — volume, distance, noise.
  notDetected,

  /// The microphone permission is denied; only the system settings help.
  permissionDenied,

  /// The active route is a Bluetooth microphone — refused for acquisition.
  refusedBluetooth,
}

/// The calibration record the UI reads: the active route and its verdict, the
/// per-route stored measurements, and the state of the current attempt.
@freezed
abstract class InputCalibrationState with _$InputCalibrationState {
  const factory InputCalibrationState({
    /// The input capture would use right now (null = none reported yet).
    CaptureRoute? route,

    /// Stored measurements, keyed by route name — a route with no entry has
    /// simply never been calibrated (spec: route change invalidates by
    /// construction, because the new route has no stored value).
    @Default(<String, double>{}) Map<String, double> stored,

    @Default(CalibrationStatus.idle) CalibrationStatus status,

    /// Runtime-only: whether the persisted store has been read back.
    @Default(false) bool hydrated,
  }) = _InputCalibrationState;

  const InputCalibrationState._();

  /// The measured round trip for the active route, or null when uncalibrated.
  double? get measuredMs => route == null ? null : stored[route!.name];
}

/// Owns the calibration flow and the per-route measurement store (spec:
/// Measured Input-Offset Calibration). The measurement is closed-loop — the
/// engine emits the click and times its return — so the stored value is always
/// a measured one, never a guess.
@Riverpod(keepAlive: true)
class InputCalibration extends _$InputCalibration {
  static const String prefsKey = 'input_calibration';

  StreamSubscription<CaptureRoute?>? _routeSub;

  @override
  InputCalibrationState build() {
    final service = ref.watch(audioCaptureServiceProvider);
    _routeSub?.cancel();
    _routeSub = service.routeChanges().listen(_onRouteChanged);
    // A desktop device selection changes what capture would open, and the
    // platform pushes nothing there — react to the selection provider instead
    // (rule: listen and update self, never poke a sibling).
    ref.listen(selectedAudioInputProvider, (_, _) => _refreshRoute());
    ref.onDispose(() {
      _routeSub?.cancel();
      _routeSub = null;
    });
    Future<void>.microtask(_restore);
    return const InputCalibrationState();
  }

  Future<void> _restore() async {
    var stored = const <String, double>{};
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      if (raw != null) {
        stored = (jsonDecode(raw) as Map<String, dynamic>).map(
          (k, v) => MapEntry(k, (v as num).toDouble()),
        );
      }
    } catch (_) {
      // Storage unavailable / corrupt → no stored measurements.
    }
    CaptureRoute? route;
    try {
      route = await ref.read(audioCaptureServiceProvider).activeRoute();
    } catch (_) {}
    state = state.copyWith(stored: stored, route: route, hydrated: true);
  }

  Future<void> _refreshRoute() async {
    try {
      _onRouteChanged(
        await ref.read(audioCaptureServiceProvider).activeRoute(),
      );
    } catch (_) {}
  }

  void _onRouteChanged(CaptureRoute? route) {
    // The stored map is keyed by route, so switching routes needs no
    // invalidation: the new route reads its own (possibly absent) entry. A
    // standing refusal lifts here too (spec: refusal lifts when the route
    // changes).
    state = state.copyWith(
      route: route,
      status: state.status == CalibrationStatus.refusedBluetooth
          ? CalibrationStatus.idle
          : state.status,
    );
  }

  /// Runs one calibration against the active route: permission, verdict,
  /// measurement, persistence — every exit sets a UI-consumable status.
  Future<void> runCalibration() async {
    if (state.status == CalibrationStatus.running) return;
    final service = ref.read(audioCaptureServiceProvider);
    state = state.copyWith(status: CalibrationStatus.running);

    var permission = await service.permissionStatus();
    if (permission == InputPermissionStatus.undetermined) {
      permission = await service.requestPermission()
          ? InputPermissionStatus.granted
          : InputPermissionStatus.denied;
    }
    if (permission != InputPermissionStatus.granted) {
      state = state.copyWith(status: CalibrationStatus.permissionDenied);
      _ship('permission_denied');
      return;
    }

    final route = await service.activeRoute();
    state = state.copyWith(route: route);
    if (route != null && route.refusedBluetooth) {
      state = state.copyWith(status: CalibrationStatus.refusedBluetooth);
      _ship('refused_bluetooth');
      return;
    }

    final measurement = await service.runCalibration();
    if (measurement == null) {
      state = state.copyWith(status: CalibrationStatus.notDetected);
      _ship('not_detected');
      return;
    }

    final stored = Map<String, double>.from(state.stored);
    if (route != null) stored[route.name] = measurement.latencyMs;
    state = state.copyWith(stored: stored, status: CalibrationStatus.done);
    _persist(stored);
    _ship(_latencyBand(measurement.latencyMs));
  }

  Future<void> _persist(Map<String, double> stored) async {
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, jsonEncode(stored));
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }

  /// The D7 fleet probe: ship the outcome as a low-cardinality bucket through
  /// the existing usage path — never the raw measurement.
  void _ship(String bucket) {
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(UsageActions.inputCalibration, variant: bucket),
    );
  }

  static String _latencyBand(double ms) {
    if (ms < 80) return 'lt80';
    if (ms < 160) return 'lt160';
    return 'gte160';
  }
}

/// The free-run availability verdict for the microphone source (spec:
/// Free-Run Gated On Measured Latency).
enum MicFreeRunGate {
  /// The measured chain fits the free-run windows: scored free-run available.
  ok,

  /// No stored measurement for the active route: calibrate first.
  needsCalibration,

  /// The measured chain exceeds what the free-run windows tolerate: honest
  /// devices stay on Wait Mode.
  latencyTooHigh,
}

/// Whether scored free-run play may open with the microphone source: only on
/// a **measured** round trip whose full chain (measurement + the detector's
/// confirmation window) still fits inside the binding window — beyond that,
/// compensating the mean leaves too much spread to bind attacks honestly.
@riverpod
MicFreeRunGate micFreeRunGate(Ref ref) {
  final measured = ref.watch(measuredInputOffsetMsProvider);
  if (measured == null) return MicFreeRunGate.needsCalibration;
  return measured + kDetectionConfirmMs <= ScoringWindows.freeBindMs
      ? MicFreeRunGate.ok
      : MicFreeRunGate.latencyTooHigh;
}

/// The measured input offset for the active capture route, in milliseconds —
/// null while uncalibrated. What audio-sourced scoring subtracts (delta spec:
/// Measured Input Offset Applied To Audio-Sourced Attacks) and what the
/// free-run gate compares against the windows.
@riverpod
double? measuredInputOffsetMs(Ref ref) =>
    ref.watch(inputCalibrationProvider.select((s) => s.measuredMs));
