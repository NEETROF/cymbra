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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/midi_service.dart';
import '../src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import 'drum_kit.dart';
import 'midi_monitor.dart';
import 'player_data.dart';
import 'player_notifier.dart';

part 'midi_monitor_notifier.g.dart';

/// The live MIDI input read-out (change: add-drum-input-calibration).
///
/// A **second reader** of the event stream, alongside the player's own — which
/// is only possible because the production adapter now fans one engine
/// subscription out to every listener (`FrbMidiService.events`). Observing
/// changes nothing: the player keeps sounding, gating and scoring exactly as it
/// does with this closed, and closing it leaves the input path as it was.
///
/// Auto-dispose: the monitor is a surface you open, look at, and leave.
@riverpod
class MidiMonitor extends _$MidiMonitor {
  @override
  List<MidiMonitorEntry> build() {
    // Read the kit ONCE per event rather than watching the player here: the
    // player rebuilds on every frame of playback, and a monitor that rebuilt
    // with it would drop its own history sixty times a second.
    final sub = ref
        .read(midiServiceProvider)
        .events()
        .listen(_record, onError: (Object _) {});
    ref.onDispose(sub.cancel);
    return const [];
  }

  int _seq = 0;

  void _record(MidiEvent event) {
    final player = ref.read(playerProvider);
    final entry = readMidiEvent(
      seq: _seq++,
      pitch: event.pitch,
      velocity: event.velocity,
      channel: event.channel,
      isNoteOn: event.kind == MidiEventKind.noteOn,
      percussion: player.isPercussion,
      lanes: player.drumLanes,
      hasKick: _scoreWritesKick(player),
    );
    // Newest first — the entry a player is looking for is the one they just
    // played, and a list that grows downward puts it off screen.
    state = [entry, ...state.take(kMidiMonitorCapacity - 1)];
  }

  /// Whether the loaded score writes a kick. The kick has no lane (it is the
  /// full-width bar), so the lane layout alone cannot answer it.
  bool _scoreWritesKick(PlayerData player) =>
      player.notes.any((n) => kKickGmNumbers.contains(n.pitch));

  /// Empties the read-out — the gesture for "watch this next stroke", which is
  /// the whole workflow when a player is hunting one silent pad.
  void clear() => state = const [];
}
