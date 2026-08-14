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
///
/// **keepAlive**: the mobile services own the `audio_routing` method-channel
/// handler and the route-change stream for the whole process. Auto-disposing
/// this (it is only ever `ref.read`, never watched) unregistered that handler
/// moments after boot, so every `routeChanged` the platform pushed was silently
/// dropped and a replugged device could never be re-pinned.
@Riverpod(keepAlive: true)
AudioRoutingService audioRoutingService(Ref ref) {
  final service = _resolveRoutingService();
  ref.onDispose(service.dispose);
  return service;
}

AudioRoutingService _resolveRoutingService() {
  if (kIsWeb) return const UnavailableAudioRoutingService();
  // iOS genuinely cannot choose: `AVAudioSession` owns the route, and the app
  // may only report it and offer the system picker.
  if (Platform.isIOS) return PlatformAudioRoutingService();
  // Android can — for its **own** output. The engine enumerates the platform's
  // devices and opens a chosen one, so the app gets a real device picker even
  // though it still cannot move the *system* route.
  if (Platform.isAndroid) return AndroidAudioRoutingService();
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

  /// Whether USB outputs on this platform are an experimental choice the UI
  /// must label as such. True only on Android, where the platform's USB-audio
  /// path proved broken below the app on every device tested (random clock on
  /// one, all-app crackle on another) — selectable, but with eyes open. The
  /// engine never *defaults* onto USB there regardless of this flag.
  bool get marksUsbExperimental => false;

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
  bool get marksUsbExperimental => false;

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

  @override
  bool get marksUsbExperimental => false;

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

/// Android wiring: the **platform** both lists the outputs and plays.
///
/// On Android the engine does not own the stream — `EngineOutput.kt` does, pulling
/// rendered samples from Rust (see the engine's `android_output` module for why:
/// `cpal`'s AAudio path there cannot enumerate the platform's outputs and does not
/// deliver usable audio to a USB-audio instrument). So everything about *where*
/// the sound goes is asked of the platform, which is also the only side that can
/// name the devices.
class AndroidAudioRoutingService implements AudioRoutingService {
  AndroidAudioRoutingService({MethodChannel? channel})
    : _channel =
          channel ??
          const MethodChannel(PlatformAudioRoutingService.channelName) {
    _channel.setMethodCallHandler(_onPlatformCall);
  }

  final MethodChannel _channel;
  final StreamController<AudioRoute?> _routes =
      StreamController<AudioRoute?>.broadcast();

  /// Outputs by name, so a choice can be resolved to the id the platform needs.
  /// Ids change when a device is replugged; names do not, so the name is what is
  /// persisted and the id is looked up fresh.
  final Map<String, int> _idsByName = {};

  @override
  bool get supportsDeviceSelection => true;

  @override
  bool get marksUsbExperimental => true;

  @override
  Future<List<AudioRoute>> listOutputs() async {
    try {
      final raw = await _channel.invokeListMethod<Object?>('allOutputs');
      final outputs = <AudioRoute>[];
      _idsByName.clear();
      for (final entry in raw ?? const <Object?>[]) {
        if (entry is! Map) continue;
        final map = entry.cast<String, dynamic>();
        final route = _parse(map);
        if (route == null) continue;
        // One route per name: the selection travels by name, and a duplicate
        // name breaks the picker (a dropdown cannot hold two items with one
        // value). The platform already dedupes; this keeps the invariant even
        // if it ever stops. First wins — the list arrives priority-sorted.
        if (_idsByName.containsKey(route.name)) continue;
        final id = int.tryParse(map['id'] as String? ?? '');
        if (id != null) _idsByName[route.name] = id;
        outputs.add(route);
      }
      return outputs;
    } catch (_) {
      return const [];
    }
  }

  @override
  Future<void> selectOutput(String? name) async {
    try {
      if (name == null) {
        await _channel.invokeMethod<bool>('selectOutput', {'deviceId': -1});
        return;
      }
      // Resolve the name to a *fresh* id on every selection: ids die on replug
      // while names survive, so a cached id may belong to a device that no
      // longer exists — and pinning to a dead id degrades to the system default.
      await listOutputs();
      final id = _idsByName[name];
      // Gone entirely: leave the working audio alone rather than sending -1,
      // which would un-pin. The caller sees the mismatch via the active route.
      if (id == null) return;
      await _channel.invokeMethod<bool>('selectOutput', {'deviceId': id});
    } catch (_) {}
  }

  /// The route the **platform** reports for media. The engine follows it unless a
  /// device was pinned, in which case the pinned one is what plays.
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

  /// Nothing to present: this platform exposes no per-app output picker, and the
  /// settings intent lands on a screen that cannot choose one. The in-app list is
  /// the picker.
  @override
  Future<void> presentRoutePicker() async {}

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
  bool get marksUsbExperimental => false;

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
