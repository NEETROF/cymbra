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
