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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/rust/api/midi.dart' as midi_api;
import '../src/rust/api/score.dart' as score_api;
import '../src/rust/api/midi.dart' show MidiEcho, MidiEvent;
import '../src/rust/api/score.dart' show Score;

part 'midi_service.g.dart';

/// Production MIDI engine provider. Override in tests with a fake.
@riverpod
MidiService midiService(Ref ref) => const FrbMidiService();

/// Production score-source provider. Override in tests with a fake.
@riverpod
ScoreSource scoreSource(Ref ref) => const FrbScoreSource();

/// Seam over the real-time MIDI engine.
///
/// [PlayerState] depends on this interface instead of the generated
/// flutter_rust_bridge functions directly, so it can be driven by a fake in
/// unit/widget tests (which run on the Dart VM with no native library loaded).
/// The production wiring is [FrbMidiService], which forwards to the bridge.
abstract class MidiService {
  /// Real-time stream of NoteOn/NoteOff events.
  Stream<MidiEvent> events();

  /// Names of available MIDI input ports (virtual ports last).
  List<String> listPorts();

  /// Name of the currently connected port, or null.
  String? connectedPort();

  /// Choose the device to listen to (null = auto: first real port).
  void selectPort(String? name);

  /// Chooses what the **engine** sounds for a live MIDI event, from its own
  /// callback (change: add-drum-input-mapping — beta fix for input latency).
  ///
  /// The app keeps the policy and pushes the answer here; whenever this is not
  /// [MidiEcho.off], the notifier stops synthesizing MIDI-sourced notes itself,
  /// so a note is sounded by exactly one side. Everything else a live note
  /// drives — the gate, the scorer, the surfaces — is unaffected: it still runs
  /// on the event the engine also streams over the bridge.
  void setEcho(MidiEcho mode);
}

/// Source of the score to play. Separated from [MidiService] because it has a
/// different lifecycle (one-shot load vs. continuous stream).
abstract class ScoreSource {
  Future<Score> demoScore();
}

/// Production [MidiService] backed by the generated flutter_rust_bridge API.
class FrbMidiService implements MidiService {
  const FrbMidiService();

  @override
  Stream<MidiEvent> events() => midi_api.midiEventStream();

  @override
  List<String> listPorts() => midi_api.listMidiPorts();

  @override
  String? connectedPort() => midi_api.connectedPort();

  @override
  void selectPort(String? name) => midi_api.setMidiPort(name: name);

  @override
  void setEcho(MidiEcho mode) => midi_api.setMidiEcho(mode: mode);
}

/// Production [ScoreSource] backed by the generated flutter_rust_bridge API.
class FrbScoreSource implements ScoreSource {
  const FrbScoreSource();

  @override
  Future<Score> demoScore() => score_api.demoScore();
}
