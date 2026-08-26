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

  /// Every echo mode the app has pushed, in order (change:
  /// add-drum-input-mapping — the engine sounds live notes itself).
  final List<MidiEcho> echoModes = <MidiEcho>[];

  /// What the engine is currently sounding on its own.
  MidiEcho echo = MidiEcho.off;

  /// The audio seam the *engine* plays through when the echo is armed. Set it
  /// to the test's [RecordingAudioService] and this fake behaves like the real
  /// engine: an emitted MIDI event is sounded in the callback, before it ever
  /// reaches the app — which is precisely what the app then declines to do
  /// again.
  AudioService? echoTo;

  FakeMidiService({this.ports = const [], this.connected, this.echoTo});

  void emit(MidiEvent event) {
    final sink = echoTo;
    final on = event.kind == MidiEventKind.noteOn;
    switch (echo) {
      case MidiEcho.off:
        break;
      case MidiEcho.melodic:
        on ? sink?.noteOn(event.pitch) : sink?.noteOff(event.pitch);
      case MidiEcho.drum:
        if (on) sink?.drumOn(event.pitch);
    }
    _controller.add(event);
  }

  @override
  void setEcho(MidiEcho mode) {
    echoModes.add(mode);
    echo = mode;
  }

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

  /// Percussion strokes (change: add-drum-audio-channel), recorded separately
  /// from the melodic pair so routing tests can assert the channel split.
  final List<({int key, int velocity})> drumOns = [];
  final List<int> drumOffs = [];
  int allNotesOffCount = 0;
  int initCount = 0;

  /// Accent flag of every metronome click, in order (true = accented downbeat).
  final List<bool> metronomeClicks = [];

  final bool failInit;

  RecordingAudioService({this.failInit = false});

  /// Flat log of calls in order, for sequencing assertions.
  final List<String> calls = [];

  /// Every SoundFont path handed to [loadSoundFont], in order — lets a test
  /// assert which piano was swapped in.
  final List<String> loadedSoundFonts = [];

  @override
  Future<void> init() async {
    initCount++;
    calls.add(failInit ? 'init:fail' : 'init');
  }

  @override
  Future<void> loadSoundFont(String sf2Path) async {
    loadedSoundFonts.add(sf2Path);
    calls.add('load:$sf2Path');
  }

  /// Every path handed to the **awaited** swap, in order (change:
  /// add-drum-audio-channel) — the percussion-readiness seam.
  final List<String> awaitedLoads = [];

  /// The awaited swap's outcome when [pendingAwaitedLoad] is not set.
  bool awaitedLoadResult = true;

  /// When set, [loadSoundFontAwaited] resolves with this completer's future,
  /// so a test can hold the swap in flight and observe the visual-only window
  /// before completing it.
  Completer<bool>? pendingAwaitedLoad;

  @override
  Future<bool> loadSoundFontAwaited(String sf2Path) async {
    awaitedLoads.add(sf2Path);
    calls.add('loadAwaited:$sf2Path');
    final pending = pendingAwaitedLoad;
    if (pending != null) return pending.future;
    return awaitedLoadResult;
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
  void drumOn(int key, {int velocity = AudioService.defaultVelocity}) {
    drumOns.add((key: key, velocity: velocity));
    calls.add('drumOn:$key');
  }

  @override
  void drumOff(int key) {
    drumOffs.add(key);
    calls.add('drumOff:$key');
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
