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

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/audio_routing_service.dart';

/// The mobile half of the routing seam (change: add-audio-output-routing),
/// driven over a fake platform channel: no device, no OS picker, no hardware.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel(PlatformAudioRoutingService.channelName);
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  late List<MethodCall> calls;
  late Object? Function(MethodCall call) respond;

  setUp(() {
    calls = [];
    respond = (_) => null;
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      return respond(call);
    });
  });

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  /// Pushes a platform-initiated `routeChanged` through the channel, the way
  /// `AppDelegate.swift` / `MainActivity.kt` do.
  Future<void> pushRouteChange(Object? arguments) =>
      messenger.handlePlatformMessage(
        PlatformAudioRoutingService.channelName,
        const StandardMethodCodec().encodeMethodCall(
          MethodCall('routeChanged', arguments),
        ),
        (_) {},
      );

  test('reads the active route the platform reports', () async {
    respond = (_) => {'name': 'AirPods Pro', 'kind': 'bluetooth'};
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);

    final route = await service.activeRoute();

    expect(route?.name, 'AirPods Pro');
    expect(route?.kind, AudioRouteKind.bluetooth);
    expect(route!.kind.isWireless, isTrue);
  });

  test('an unrecognized kind degrades to other', () async {
    respond = (_) => {'name': 'Car', 'kind': 'carplay-quantum-link'};
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);

    expect((await service.activeRoute())?.kind, AudioRouteKind.other);
  });

  test('an unusable payload reads as no route rather than an error', () async {
    respond = (_) => null;
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);

    expect(await service.activeRoute(), isNull);
  });

  test('a platform failure degrades to no route', () async {
    respond = (_) => throw PlatformException(code: 'boom');
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);

    expect(await service.activeRoute(), isNull);
  });

  test('presenting the picker calls through to the platform', () async {
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);

    await service.presentRoutePicker();

    expect(calls.map((c) => c.method), contains('presentRoutePicker'));
  });

  test('offers no device list and never fakes a selection', () async {
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);

    expect(service.supportsDeviceSelection, isFalse);
    expect(await service.listOutputs(), isEmpty);
    await service.selectOutput('Anything');

    expect(calls.map((c) => c.method), isNot(contains('selectOutput')));
  });

  test('route changes pushed by the platform reach the stream', () async {
    final service = PlatformAudioRoutingService();
    addTearDown(service.dispose);
    final seen = <AudioRoute?>[];
    final sub = service.routeChanges().listen(seen.add);
    addTearDown(sub.cancel);

    await pushRouteChange({'name': 'Speaker', 'kind': 'builtin'});
    await pushRouteChange(null);

    expect(seen, [
      const AudioRoute(name: 'Speaker', kind: AudioRouteKind.builtin),
      null,
    ]);
  });

  test('a route change after dispose is ignored', () async {
    PlatformAudioRoutingService().dispose();

    await expectLater(
      pushRouteChange({'name': 'Speaker', 'kind': 'builtin'}),
      completes,
    );
  });

  group('Android device selection', () {
    /// A platform whose only USB output is [name] under [id] — ids change on
    /// every replug, names do not.
    Object? Function(MethodCall) platformWith({
      required String name,
      required int id,
    }) =>
        (call) => switch (call.method) {
          'allOutputs' => [
            {'id': '$id', 'name': name, 'kind': 'usb'},
            {'id': '2', 'name': 'Speakers', 'kind': 'builtin'},
          ],
          _ => null,
        };

    test('lists the platform outputs with their kinds', () async {
      respond = platformWith(name: 'P-145', id: 21);
      final service = AndroidAudioRoutingService();
      addTearDown(service.dispose);

      final outputs = await service.listOutputs();

      expect(outputs, const [
        AudioRoute(name: 'P-145', kind: AudioRouteKind.usb),
        AudioRoute(name: 'Speakers', kind: AudioRouteKind.builtin),
      ]);
      expect(service.supportsDeviceSelection, isTrue);
    });

    test('resolves a selection to a fresh id on every call', () async {
      // Listed once under id 21, then replugged: the same name now lives under
      // id 42. Pinning with the cached 21 would silently un-pin.
      respond = platformWith(name: 'P-145', id: 21);
      final service = AndroidAudioRoutingService();
      addTearDown(service.dispose);
      await service.listOutputs();
      respond = platformWith(name: 'P-145', id: 42);

      await service.selectOutput('P-145');

      final select = calls.lastWhere((c) => c.method == 'selectOutput');
      expect(select.arguments, {'deviceId': 42});
    });

    test('a selection whose device is gone leaves the audio alone', () async {
      respond = platformWith(name: 'P-145', id: 21);
      final service = AndroidAudioRoutingService();
      addTearDown(service.dispose);
      await service.listOutputs();
      respond = platformWith(name: 'Other Piano', id: 9);

      await service.selectOutput('P-145');

      // No pin request at all: sending -1 would move the sound to the system
      // default, which is not what the user asked for.
      expect(calls.map((c) => c.method), isNot(contains('selectOutput')));
    });

    test('duplicate names collapse to one route', () async {
      // A USB instrument can expose several outputs under one product name; a
      // dropdown with two identical values throws, so the seam de-duplicates
      // (first wins — the platform sends the list priority-sorted).
      respond = (call) => switch (call.method) {
        'allOutputs' => [
          {'id': '21', 'name': 'P-145', 'kind': 'usb'},
          {'id': '22', 'name': 'P-145', 'kind': 'usb'},
          {'id': '2', 'name': 'Speakers', 'kind': 'builtin'},
        ],
        _ => null,
      };
      final service = AndroidAudioRoutingService();
      addTearDown(service.dispose);

      final outputs = await service.listOutputs();

      expect(outputs, const [
        AudioRoute(name: 'P-145', kind: AudioRouteKind.usb),
        AudioRoute(name: 'Speakers', kind: AudioRouteKind.builtin),
      ]);
      await service.selectOutput('P-145');
      final select = calls.lastWhere((c) => c.method == 'selectOutput');
      expect(select.arguments, {'deviceId': 21});
    });

    test('selecting the system default sends -1', () async {
      final service = AndroidAudioRoutingService();
      addTearDown(service.dispose);

      await service.selectOutput(null);

      final select = calls.lastWhere((c) => c.method == 'selectOutput');
      expect(select.arguments, {'deviceId': -1});
    });
  });

  group('unavailable platforms', () {
    test('report nothing and accept nothing', () async {
      const service = UnavailableAudioRoutingService();

      expect(service.supportsDeviceSelection, isFalse);
      expect(await service.listOutputs(), isEmpty);
      expect(await service.activeRoute(), isNull);
      await service.selectOutput('X');
      await service.presentRoutePicker();
      expect(await service.routeChanges().toList(), isEmpty);
      service.dispose();
    });
  });
}
