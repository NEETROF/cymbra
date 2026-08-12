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
import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/services.dart' show MethodCall, MethodChannel;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/rust/api/audio.dart' as audio_api;
import '../src/rust/api/audio.dart' show AudioOutputInfo;

part 'audio_routing_service.g.dart';

/// How an audio output is connected, as the app reasons about it (change:
/// add-audio-output-routing).
///
/// The **kind** — never the route's name — is what tells the UI a route is
/// wireless, so a platform reporting a connection this build does not know
/// degrades to [AudioRouteKind.other] instead of breaking the section.
enum AudioRouteKind {
  /// The device's own speakers / integrated audio.
  builtin,

  /// Wired headphones or a wired headset.
  headphones,

  /// A wireless (Bluetooth) route.
  bluetooth,

  /// A USB audio device (interface, USB-audio piano, USB headset).
  usb,

  /// Anything the platform does not describe well enough to classify.
  other;

  /// Whether this route carries the audio wirelessly, and therefore late enough
  /// to matter when playing along. The single predicate behind the warning and
  /// the offset control.
  bool get isWireless => this == AudioRouteKind.bluetooth;

  /// Parses the wire value used by the platform channels; anything unknown maps
  /// to [AudioRouteKind.other].
  static AudioRouteKind parse(String? name) =>
      AudioRouteKind.values.asNameMap()[name] ?? AudioRouteKind.other;
}

/// Where the app's audio is going: a name to show and the kind of connection.
class AudioRoute {
  const AudioRoute({required this.name, required this.kind});

  /// The platform's name for the output (a device name on desktop, a port name
  /// on mobile). Also the handle a desktop selection is persisted under.
  final String name;

  /// How it is connected.
  final AudioRouteKind kind;

  @override
  bool operator ==(Object other) =>
      other is AudioRoute && other.name == name && other.kind == kind;

  @override
  int get hashCode => Object.hash(name, kind);

  @override
  String toString() => 'AudioRoute($name, ${kind.name})';
}

/// Production audio-routing provider. Override in tests with a mock so device
/// lists, active routes, route kinds and selection failures can be driven
/// deterministically, with no audio hardware and no native library.
@riverpod
AudioRoutingService audioRoutingService(Ref ref) {
  final service = _resolveRoutingService();
  ref.onDispose(service.dispose);
  return service;
}

AudioRoutingService _resolveRoutingService() {
  if (kIsWeb) return const UnavailableAudioRoutingService();
  if (Platform.isIOS || Platform.isAndroid) {
    return PlatformAudioRoutingService();
  }
  return const EngineAudioRoutingService();
}

/// Seam over "where does the app's audio go".
///
/// Two platform realities behind one shape: on desktop the engine enumerates
/// real output devices and can open a chosen one; on mobile there is no device
/// to pick — the OS owns the route, so the app presents the system picker and
/// reports what is active. Callers (the routing notifier) never branch on the
/// platform: they read [supportsDeviceSelection] and use the operations it
/// allows.
abstract class AudioRoutingService {
  /// Outputs the user may choose from. Empty when the platform does not let the
  /// app choose (mobile) or the host reports none.
  Future<List<AudioRoute>> listOutputs();

  /// Sends all app audio to [name], or follows the system default when null. A
  /// no-op where selection is unsupported.
  Future<void> selectOutput(String? name);

  /// The route audio is going through right now, or null when unknown.
  Future<AudioRoute?> activeRoute();

  /// Shows the operating system's route picker. A no-op where there is none.
  Future<void> presentRoutePicker();

  /// Whether the app can pick an output itself (desktop) as opposed to only
  /// reporting the system's route and offering its picker (mobile).
  bool get supportsDeviceSelection;

  /// Routes reported by the platform after the user changes them. Never emits
  /// where the platform has no notification to give.
  Stream<AudioRoute?> routeChanges();

  /// Releases any platform subscription. Idempotent.
  void dispose();
}

/// Desktop wiring: the Rust engine owns the output device, so listing,
/// selecting and reporting all go through the bridge (change:
/// add-audio-output-routing).
///
/// Every call is guarded: a native hiccup degrades to "no devices / unknown
/// route" rather than throwing into the settings section.
class EngineAudioRoutingService implements AudioRoutingService {
  const EngineAudioRoutingService();

  @override
  bool get supportsDeviceSelection => true;

  @override
  Future<List<AudioRoute>> listOutputs() async {
    try {
      return audio_api.listAudioOutputs().map(_toRoute).toList();
    } catch (_) {
      return const [];
    }
  }

  @override
  Future<void> selectOutput(String? name) async {
    try {
      audio_api.setAudioOutput(name: name);
    } catch (_) {}
  }

  @override
  Future<AudioRoute?> activeRoute() async {
    try {
      final active = audio_api.activeAudioOutput();
      return active == null ? null : _toRoute(active);
    } catch (_) {
      return null;
    }
  }

  /// Desktop has no OS route picker to present — the in-app device list *is* the
  /// picker.
  @override
  Future<void> presentRoutePicker() async {}

  @override
  Stream<AudioRoute?> routeChanges() => const Stream<AudioRoute?>.empty();

  @override
  void dispose() {}

  static AudioRoute _toRoute(AudioOutputInfo info) => AudioRoute(
    name: info.name,
    kind: switch (info.kind) {
      audio_api.AudioRouteKind.builtin => AudioRouteKind.builtin,
      audio_api.AudioRouteKind.headphones => AudioRouteKind.headphones,
      audio_api.AudioRouteKind.bluetooth => AudioRouteKind.bluetooth,
      audio_api.AudioRouteKind.usb => AudioRouteKind.usb,
      audio_api.AudioRouteKind.other => AudioRouteKind.other,
    },
  );
}

/// Mobile wiring: iOS and Android own the route, so the app reads it and
/// presents the system picker over a method channel. Selecting a device is
/// deliberately unsupported rather than faked.
class PlatformAudioRoutingService implements AudioRoutingService {
  PlatformAudioRoutingService({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName) {
    _channel.setMethodCallHandler(_onPlatformCall);
  }

  /// Channel implemented by `AppDelegate.swift` (iOS) and `MainActivity.kt`
  /// (Android).
  static const String channelName = 'org.cymbra.music/audio_routing';

  final MethodChannel _channel;
  final StreamController<AudioRoute?> _routes =
      StreamController<AudioRoute?>.broadcast();

  @override
  bool get supportsDeviceSelection => false;

  /// The system owns the route: there is no per-device list to offer, and
  /// showing one the app cannot honor would be a lie.
  @override
  Future<List<AudioRoute>> listOutputs() async => const [];

  @override
  Future<void> selectOutput(String? name) async {}

  @override
  Future<AudioRoute?> activeRoute() async {
    try {
      final raw = await _channel.invokeMapMethod<String, dynamic>(
        'activeRoute',
      );
      return _parse(raw);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<void> presentRoutePicker() async {
    try {
      await _channel.invokeMethod<void>('presentRoutePicker');
    } catch (_) {}
  }

  @override
  Stream<AudioRoute?> routeChanges() => _routes.stream;

  @override
  void dispose() {
    _channel.setMethodCallHandler(null);
    unawaited(_routes.close());
  }

  Future<void> _onPlatformCall(MethodCall call) async {
    if (call.method != 'routeChanged' || _routes.isClosed) return;
    final raw = call.arguments;
    _routes.add(raw is Map ? _parse(raw.cast<String, dynamic>()) : null);
  }

  static AudioRoute? _parse(Map<String, dynamic>? raw) {
    final name = raw?['name'];
    if (name is! String || name.isEmpty) return null;
    return AudioRoute(
      name: name,
      kind: AudioRouteKind.parse(raw?['kind'] as String?),
    );
  }
}

/// Platforms with neither an engine device list nor a system route picker (the
/// web build). Reports nothing and accepts nothing, so the settings section
/// simply has no sound-output row to show.
class UnavailableAudioRoutingService implements AudioRoutingService {
  const UnavailableAudioRoutingService();

  @override
  bool get supportsDeviceSelection => false;

  @override
  Future<List<AudioRoute>> listOutputs() async => const [];

  @override
  Future<void> selectOutput(String? name) async {}

  @override
  Future<AudioRoute?> activeRoute() async => null;

  @override
  Future<void> presentRoutePicker() async {}

  @override
  Stream<AudioRoute?> routeChanges() => const Stream<AudioRoute?>.empty();

  @override
  void dispose() {}
}
