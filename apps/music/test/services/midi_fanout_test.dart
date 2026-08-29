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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEvent, MidiEventKind;

// The MIDI fan-out, asserted directly (change: add-drum-input-calibration,
// task 1.3).
//
// The engine holds ONE Flutter sink: `midi_event_stream` stores the sink it is
// handed in a global and the input callback reads that global, so a second
// `events()` would *replace* the first subscriber rather than join it —
// silently, with which consumer loses decided by build order. Everything added
// since depends on that not being true: the monitor is a second reader by
// definition, the calibration pass a third, and in the player the consumer they
// would starve is the player.
//
// Until now that was only observable through two consumers of a fake service
// that was already broadcast for its own reasons — which asserts the fake, not
// the adapter. The opener seam makes the real adapter testable on the VM.

MidiEvent _event(int pitch) => MidiEvent(
  kind: MidiEventKind.noteOn,
  pitch: pitch,
  velocity: 100,
  channel: 9,
  timestampMs: BigInt.zero,
);

void main() {
  late StreamController<MidiEvent> engine;
  late int opens;

  setUp(() {
    engine = StreamController<MidiEvent>();
    opens = 0;
    FrbMidiService.opener = () {
      opens++;
      return engine.stream;
    };
    FrbMidiService.resetSharedStream();
  });

  tearDown(() {
    FrbMidiService.resetSharedStream();
    FrbMidiService.opener = FrbMidiService.defaultOpener;
    // NOT awaited: this controller is single-subscription, exactly like the
    // engine stream it stands in for, and `close()` on one nobody listened to
    // never completes.
    unawaited(engine.close());
  });

  test('the engine stream is opened once, however many callers ask', () {
    const service = FrbMidiService();
    final a = service.events();
    final b = service.events();
    final c = const FrbMidiService().events();

    expect(opens, 1, reason: 'a second call must join, never re-open');
    expect(identical(a, b), isTrue);
    expect(identical(a, c), isTrue, reason: 'the cache is process-wide');
  });

  test('what it hands out is a broadcast stream', () {
    // A single-subscription stream would throw on the second listen — which is
    // exactly the failure this fan-out exists to prevent.
    expect(const FrbMidiService().events().isBroadcast, isTrue);
  });

  test('two listeners both receive every event', () async {
    const service = FrbMidiService();
    final first = <int>[];
    final second = <int>[];
    final subA = service.events().listen((e) => first.add(e.pitch));
    final subB = service.events().listen((e) => second.add(e.pitch));

    engine
      ..add(_event(38))
      ..add(_event(42));
    await pumpEventQueue();

    expect(first, [38, 42]);
    expect(second, [38, 42], reason: 'the second reader is not starved');
    await subA.cancel();
    await subB.cancel();
  });

  test('one listener leaving does not disturb the other', () async {
    // The monitor and the calibration pass are surfaces you open and leave; the
    // player underneath must not lose its input when they do.
    const service = FrbMidiService();
    final staying = <int>[];
    final leaving = <int>[];
    final subA = service.events().listen((e) => staying.add(e.pitch));
    final subB = service.events().listen((e) => leaving.add(e.pitch));

    engine.add(_event(38));
    await pumpEventQueue();
    await subB.cancel();
    engine.add(_event(42));
    await pumpEventQueue();

    expect(staying, [38, 42], reason: 'the survivor keeps receiving');
    expect(leaving, [38]);
    await subA.cancel();
  });

  test('the last listener leaving does not close the shared stream', () async {
    // `onCancel` is deliberately left alone: the engine's watcher thread runs
    // for the process lifetime either way, and tearing the sink down when the
    // last screen leaves would only mean re-registering it on the next one.
    const service = FrbMidiService();
    final sub = service.events().listen((_) {});
    await sub.cancel();

    final after = <int>[];
    final again = service.events().listen((e) => after.add(e.pitch));
    engine.add(_event(38));
    await pumpEventQueue();

    expect(opens, 1, reason: 'still the same engine subscription');
    expect(after, [38], reason: 're-listening works after the last cancel');
    await again.cancel();
  });

  test(
    'an engine error reaches every listener without killing the fan-out',
    () async {
      const service = FrbMidiService();
      final errors = <Object>[];
      final events = <int>[];
      final sub = service.events().listen(
        (e) => events.add(e.pitch),
        onError: errors.add,
      );

      engine.addError(StateError('device unplugged'));
      await pumpEventQueue();
      engine.add(_event(38));
      await pumpEventQueue();

      expect(errors, hasLength(1));
      expect(events, [
        38,
      ], reason: 'a hot-plug error is not the end of the stream');
      await sub.cancel();
    },
  );
}
