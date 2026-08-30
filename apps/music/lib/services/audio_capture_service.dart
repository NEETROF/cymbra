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
import 'dart:typed_data';
import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/services.dart' show MethodCall, MethodChannel;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/rust/api/audio_input.dart' as input_api;
import '../src/rust/api/audio_input.dart'
    show InputRouteKind, InputRouteVerdict;

part 'audio_capture_service.g.dart';

/// The microphone permission as the app reasons about it (change:
/// add-acoustic-piano-input). `undetermined` means asking is still possible;
/// `denied` means the OS prompt is spent and only the system settings help.
enum InputPermissionStatus { granted, denied, undetermined }

/// A capture route: what to show, how it is connected, and whether it may
/// acquire. The verdict is computed by the engine's one rule (Bluetooth
/// refused, everything else accepted) and carried here so no UI re-derives it.
class CaptureRoute {
  const CaptureRoute({
    required this.name,
    required this.kind,
    required this.refusedBluetooth,
  });

  /// The platform's name for the input (display only — never classified from).
  final String name;

  /// How it is connected, classified from the platform's port-type token.
  final InputRouteKind kind;

  /// Whether acquisition from this route is refused (spec: Bluetooth Input
  /// Refusal — a hard rule, not a warning).
  final bool refusedBluetooth;

  @override
  bool operator ==(Object other) =>
      other is CaptureRoute &&
      other.name == name &&
      other.kind == kind &&
      other.refusedBluetooth == refusedBluetooth;

  @override
  int get hashCode => Object.hash(name, kind, refusedBluetooth);

  @override
  String toString() =>
      'CaptureRoute($name, ${kind.name}${refusedBluetooth ? ', refused' : ''})';
}

/// One measured input round trip (spec: Measured Input-Offset Calibration).
class CalibrationMeasurement {
  const CalibrationMeasurement({required this.latencyMs});

  /// The emission→capture round trip, in milliseconds.
  final double latencyMs;

  @override
  bool operator ==(Object other) =>
      other is CalibrationMeasurement && other.latencyMs == latencyMs;

  @override
  int get hashCode => latencyMs.hashCode;
}

/// Production capture provider. Override in tests with a mockito mock so
/// permission outcomes, route kinds, calibration results and lifecycle
/// transitions can all be driven with no microphone and no native library.
///
/// **keepAlive** for the same reason as the routing service: the mobile
/// implementation owns the `audio_input` method-channel handler and the
/// route-change stream for the whole process.
@Riverpod(keepAlive: true)
AudioCaptureService audioCaptureService(Ref ref) {
  final service = _resolveCaptureService();
  ref.onDispose(service.dispose);
  return service;
}

AudioCaptureService _resolveCaptureService() {
  if (kIsWeb) return const UnavailableAudioCaptureService();
  if (Platform.isIOS || Platform.isAndroid) {
    return PlatformAudioCaptureService();
  }
  return const EngineAudioCaptureService();
}

/// Seam over microphone acquisition: permission, the active capture route and
/// its verdict, the capture lifecycle, and the calibration run (spec:
/// Injectable Capture Seam). Callers never branch on the platform.
abstract class AudioCaptureService {
  /// The microphone permission right now.
  Future<InputPermissionStatus> permissionStatus();

  /// Triggers the OS permission prompt; resolves to whether it was granted.
  Future<bool> requestPermission();

  /// The input capture would use right now, or null when the platform lists
  /// none (no microphone hardware).
  Future<CaptureRoute?> activeRoute();

  /// Route changes pushed by the platform. Never emits where the platform has
  /// no notification to give (desktop).
  Stream<CaptureRoute?> routeChanges();

  /// Puts the platform audio session in capture shape (a no-op outside iOS)
  /// and starts the engine's capture stream. False when either step fails —
  /// callers degrade with guidance, never crash.
  Future<bool> beginCapture();

  /// Stops the engine's capture stream and restores the playback-only session.
  Future<void> endCapture();

  /// Runs one calibration: the engine captures, emits the reference click and
  /// measures the round trip. Null when the click was never detected — the
  /// caller shows guidance and stores nothing. Manages the capture session
  /// itself; callers do not wrap it in [beginCapture]/[endCapture].
  Future<CalibrationMeasurement?> runCalibration();

  /// The obtained capture configuration, for diagnostics (spec: the recorded
  /// configuration). Keys: `source`, `unprocessedSupported`. Null when the
  /// platform has nothing to report.
  Future<Map<String, Object?>?> captureConfig();

  /// Starts acoustic note detection over the running capture: detected notes
  /// enter the same event stream MIDI does. Nothing is emitted until
  /// [setExpectedPitches] provides a non-empty window.
  void startDetection();

  /// Stops acoustic note detection (capture keeps its own lifecycle).
  void stopDetection();

  /// Replaces the expected-pitch window the score-informed presence stage
  /// evaluates (spec: Score-Informed Presence Detection). Empty idles it.
  void setExpectedPitches(List<int> pitches);

  /// Releases any platform subscription. Idempotent.
  void dispose();
}

/// Web: no capture. Every read degrades; nothing throws.
class UnavailableAudioCaptureService implements AudioCaptureService {
  const UnavailableAudioCaptureService();

  @override
  Future<InputPermissionStatus> permissionStatus() async =>
      InputPermissionStatus.denied;

  @override
  Future<bool> requestPermission() async => false;

  @override
  Future<CaptureRoute?> activeRoute() async => null;

  @override
  Stream<CaptureRoute?> routeChanges() => const Stream<CaptureRoute?>.empty();

  @override
  Future<bool> beginCapture() async => false;

  @override
  Future<void> endCapture() async {}

  @override
  Future<CalibrationMeasurement?> runCalibration() async => null;

  @override
  Future<Map<String, Object?>?> captureConfig() async => null;

  @override
  void startDetection() {}

  @override
  void stopDetection() {}

  @override
  void setExpectedPitches(List<int> pitches) {}

  @override
  void dispose() {}
}

/// Desktop wiring: the engine owns the device; the OS gates the microphone
/// with its own prompt at first open (macOS TCC), so permission is reported
/// as granted and a refusal simply surfaces as a failed capture/calibration
/// with guidance.
class EngineAudioCaptureService implements AudioCaptureService {
  const EngineAudioCaptureService();

  @override
  Future<InputPermissionStatus> permissionStatus() async =>
      InputPermissionStatus.granted;

  @override
  Future<bool> requestPermission() async => true;

  @override
  Future<CaptureRoute?> activeRoute() async {
    try {
      final inputs = input_api.listAudioInputs();
      if (inputs.isEmpty) return null;
      // Desktop hosts expose no transport type: kind degrades to `other` and
      // stays usable (spec: unknown route degrades safely).
      return CaptureRoute(
        name: inputs.first.name,
        kind: inputs.first.kind,
        refusedBluetooth: false,
      );
    } catch (_) {
      return null;
    }
  }

  @override
  Stream<CaptureRoute?> routeChanges() => const Stream<CaptureRoute?>.empty();

  @override
  Future<bool> beginCapture() async {
    try {
      return input_api.audioInputStartCapture();
    } catch (_) {
      return false;
    }
  }

  @override
  Future<void> endCapture() async {
    try {
      input_api.audioInputStopCapture();
    } catch (_) {}
  }

  @override
  Future<CalibrationMeasurement?> runCalibration() async {
    try {
      final result = await input_api.runInputCalibration();
      final latency = result.latencyMs;
      if (!result.detected || latency == null) return null;
      return CalibrationMeasurement(latencyMs: latency);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<Map<String, Object?>?> captureConfig() async => null;

  @override
  void startDetection() => _startDetection();

  @override
  void stopDetection() => _stopDetection();

  @override
  void setExpectedPitches(List<int> pitches) => _setExpectedPitches(pitches);

  @override
  void dispose() {}
}

/// Mobile wiring: permission, session shape and the input route live behind
/// the `audio_input` method channel (`AppDelegate.swift` on iOS,
/// `AudioInput.kt` on Android); capture and calibration run in the engine.
class PlatformAudioCaptureService implements AudioCaptureService {
  PlatformAudioCaptureService({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName) {
    _channel.setMethodCallHandler(_onPlatformCall);
  }

  /// Channel implemented by `AppDelegate.swift` (iOS) and `AudioInput.kt`
  /// (Android).
  static const String channelName = 'org.cymbra.music/audio_input';

  final MethodChannel _channel;
  final StreamController<CaptureRoute?> _routes =
      StreamController<CaptureRoute?>.broadcast();

  @override
  Future<InputPermissionStatus> permissionStatus() async {
    try {
      final raw = await _channel.invokeMethod<String>('permissionStatus');
      return InputPermissionStatus.values.asNameMap()[raw] ??
          InputPermissionStatus.denied;
    } catch (_) {
      return InputPermissionStatus.denied;
    }
  }

  @override
  Future<bool> requestPermission() async {
    try {
      return await _channel.invokeMethod<bool>('requestPermission') ?? false;
    } catch (_) {
      return false;
    }
  }

  @override
  Future<CaptureRoute?> activeRoute() async {
    try {
      final raw = await _channel.invokeMapMethod<String, dynamic>(
        'activeInputRoute',
      );
      return _parse(raw);
    } catch (_) {
      return null;
    }
  }

  @override
  Stream<CaptureRoute?> routeChanges() => _routes.stream;

  @override
  Future<bool> beginCapture() async {
    try {
      final session =
          await _channel.invokeMethod<bool>('beginCaptureSession') ?? false;
      if (!session) return false;
      return input_api.audioInputStartCapture();
    } catch (_) {
      return false;
    }
  }

  @override
  Future<void> endCapture() async {
    try {
      input_api.audioInputStopCapture();
    } catch (_) {}
    try {
      await _channel.invokeMethod<void>('endCaptureSession');
    } catch (_) {}
  }

  @override
  Future<CalibrationMeasurement?> runCalibration() async {
    // The session flip must surround the engine run: `.measurement` is what
    // keeps the voice-processing chain away from the reference click too.
    try {
      final session =
          await _channel.invokeMethod<bool>('beginCaptureSession') ?? false;
      if (!session) return null;
    } catch (_) {
      return null;
    }
    try {
      final result = await input_api.runInputCalibration();
      final latency = result.latencyMs;
      if (!result.detected || latency == null) return null;
      return CalibrationMeasurement(latencyMs: latency);
    } catch (_) {
      return null;
    } finally {
      await endCapture();
    }
  }

  @override
  Future<Map<String, Object?>?> captureConfig() async {
    if (Platform.isIOS) {
      // iOS always captures under `.measurement`, which is the unprocessed
      // configuration by definition.
      return const {'source': 'MEASUREMENT', 'unprocessedSupported': true};
    }
    try {
      return await _channel.invokeMapMethod<String, Object?>('inputConfig');
    } catch (_) {
      return null;
    }
  }

  @override
  void startDetection() => _startDetection();

  @override
  void stopDetection() => _stopDetection();

  @override
  void setExpectedPitches(List<int> pitches) => _setExpectedPitches(pitches);

  @override
  void dispose() {
    _routes.close();
  }

  Future<dynamic> _onPlatformCall(MethodCall call) async {
    if (call.method == 'inputRouteChanged') {
      final args = (call.arguments as Map?)?.cast<String, dynamic>();
      if (!_routes.isClosed) _routes.add(_parse(args));
    }
    return null;
  }

  /// Builds a [CaptureRoute] from the platform's `{name, token}` report: the
  /// token is classified by the engine's one definition, and the verdict is
  /// derived from the same source.
  CaptureRoute? _parse(Map<String, dynamic>? raw) {
    final name = raw?['name'] as String?;
    final token = raw?['token'] as String?;
    if (name == null || token == null) return null;
    InputRouteKind kind;
    InputRouteVerdict verdict;
    try {
      kind = input_api.classifyInputRouteToken(token: token);
      verdict = input_api.inputRouteVerdictFor(kind: kind);
    } catch (_) {
      kind = InputRouteKind.other;
      verdict = InputRouteVerdict.accepted;
    }
    return CaptureRoute(
      name: name,
      kind: kind,
      refusedBluetooth: verdict == InputRouteVerdict.refusedBluetooth,
    );
  }
}

/// Shared FFI forwarding for the engine-backed services. Guarded: a missing
/// native library degrades to a no-op, never a throw.
void _startDetection() {
  try {
    input_api.audioInputStartDetection();
  } catch (_) {}
}

void _stopDetection() {
  try {
    input_api.audioInputStopDetection();
  } catch (_) {}
}

void _setExpectedPitches(List<int> pitches) {
  try {
    input_api.setExpectedPitches(
      pitches: Uint8List.fromList([for (final p in pitches) p.clamp(0, 127)]),
    );
  } catch (_) {}
}
