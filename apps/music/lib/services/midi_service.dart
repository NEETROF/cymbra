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

import '../src/rust/api/midi.dart' as midi_api;
import '../src/rust/api/score.dart' as score_api;
import '../src/rust/api/midi.dart' show MidiEvent;
import '../src/rust/api/score.dart' show Score;

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
}

/// Production [ScoreSource] backed by the generated flutter_rust_bridge API.
class FrbScoreSource implements ScoreSource {
  const FrbScoreSource();

  @override
  Future<Score> demoScore() => score_api.demoScore();
}
