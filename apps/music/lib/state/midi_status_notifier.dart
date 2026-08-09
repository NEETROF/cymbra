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

import 'package:flutter/foundation.dart' show listEquals;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/midi_service.dart';

part 'midi_status_notifier.g.dart';

/// The MIDI instrument picture a lesson shows (change: add-notation-courses):
/// the available ports and the connected one, over the same [MidiService] seam
/// the game uses — without instantiating the game's whole player state.
///
/// Deliberately no periodic polling: the state refreshes when the picker opens
/// (always fresh when it matters) and whenever a MIDI event arrives (a playing
/// instrument proves itself connected), which also keeps widget tests free of
/// never-settling timers.
@riverpod
class MidiStatus extends _$MidiStatus {
  @override
  ({List<String> ports, String? connected}) build() {
    final midi = ref.watch(midiServiceProvider);
    final sub = midi.events().listen((_) => refresh());
    ref.onDispose(sub.cancel);
    return (ports: midi.listPorts(), connected: midi.connectedPort());
  }

  /// Re-reads ports and connection, updating only on a real change.
  void refresh() {
    final midi = ref.read(midiServiceProvider);
    final ports = midi.listPorts();
    final connected = midi.connectedPort();
    if (!listEquals(ports, state.ports) || connected != state.connected) {
      state = (ports: ports, connected: connected);
    }
  }

  /// Connects to [port] (the picker's choice) and reflects the result.
  void select(String? port) {
    ref.read(midiServiceProvider).selectPort(port);
    refresh();
  }
}
