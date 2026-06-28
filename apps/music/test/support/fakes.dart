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

import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/src/rust/api/midi.dart';
import 'package:music/src/rust/api/score.dart';

/// In-memory [MidiService] for tests: the test scripts ports, connection state,
/// and pushes MIDI events through [emit] — no native library required.
class FakeMidiService implements MidiService {
  final StreamController<MidiEvent> _controller =
      StreamController<MidiEvent>.broadcast();

  List<String> ports;
  String? connected;
  final List<String?> selectPortCalls = <String?>[];

  FakeMidiService({this.ports = const [], this.connected});

  void emit(MidiEvent event) => _controller.add(event);

  @override
  Stream<MidiEvent> events() => _controller.stream;

  @override
  List<String> listPorts() => ports;

  @override
  String? connectedPort() => connected;

  @override
  void selectPort(String? name) {
    selectPortCalls.add(name);
    // Emulate the engine connecting to the chosen port.
    if (name != null) connected = name;
  }

  Future<void> close() => _controller.close();
}

/// Recording [AudioService] for tests: captures every call so a test can assert
/// the player drives the synth, without loading the native audio library.
///
/// Set [failInit] to emulate a missing device / SoundFont — [init] then records
/// the attempt but the service stays usable (its other calls are still recorded,
/// mirroring the production no-op-on-failure behaviour at the player's level).
class RecordingAudioService implements AudioService {
  final List<({int pitch, int velocity})> noteOns = [];
  final List<int> noteOffs = [];
  int allNotesOffCount = 0;
  int initCount = 0;

  /// Accent flag of every metronome click, in order (true = accented downbeat).
  final List<bool> metronomeClicks = [];

  final bool failInit;

  RecordingAudioService({this.failInit = false});

  /// Flat log of calls in order, for sequencing assertions.
  final List<String> calls = [];

  @override
  Future<void> init() async {
    initCount++;
    calls.add(failInit ? 'init:fail' : 'init');
  }

  @override
  void noteOn(int pitch, {int velocity = AudioService.defaultVelocity}) {
    noteOns.add((pitch: pitch, velocity: velocity));
    calls.add('on:$pitch');
  }

  @override
  void noteOff(int pitch) {
    noteOffs.add(pitch);
    calls.add('off:$pitch');
  }

  @override
  void allNotesOff() {
    allNotesOffCount++;
    calls.add('allOff');
  }

  @override
  void metronomeClick({required bool accent}) {
    metronomeClicks.add(accent);
    calls.add(accent ? 'click:accent' : 'click:beat');
  }
}

/// [ScoreSource] returning a fixed, tiny score for deterministic tests.
class FakeScoreSource implements ScoreSource {
  final Score score;
  FakeScoreSource([Score? score]) : score = score ?? defaultScore();

  @override
  Future<Score> demoScore() async => score;

  /// Two adjacent notes: C4 [0,500), D4 [500,1000). Song ends at 1000ms.
  static Score defaultScore() => Score(
    bpm: 80,
    measures: [
      Measure(
        index: 0,
        notes: [
          Note(pitch: 60, startMs: BigInt.zero, durationMs: BigInt.from(500)),
          Note(
            pitch: 62,
            startMs: BigInt.from(500),
            durationMs: BigInt.from(500),
          ),
        ],
      ),
    ],
  );
}

/// Convenience constructor for a NoteOn event.
MidiEvent noteOnEvent(int pitch, {int velocity = 100}) => MidiEvent(
  kind: MidiEventKind.noteOn,
  pitch: pitch,
  velocity: velocity,
  timestampMs: BigInt.zero,
);

/// Convenience constructor for a NoteOff event.
MidiEvent noteOffEvent(int pitch) => MidiEvent(
  kind: MidiEventKind.noteOff,
  pitch: pitch,
  velocity: 0,
  timestampMs: BigInt.zero,
);
