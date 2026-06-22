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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/midi_service.dart';
import '../src/rust/api/midi.dart';
import 'player_data.dart';

part 'player_notifier.g.dart';

/// Central player notifier: pressed keys, score, rendering mode, playhead and
/// Wait Mode logic. Listens to the real-time MIDI stream and also receives notes
/// from the computer-keyboard fallback (via [noteOn]/[noteOff]).
@riverpod
class Player extends _$Player {
  Timer? _statusTimer;
  StreamSubscription<MidiEvent>? _sub;

  @override
  PlayerData build() {
    final midi = ref.watch(midiServiceProvider);
    _sub = midi.events().listen(_onMidi, onError: (Object _) {});
    // Poll the MIDI connection state every second (handles hot-plug).
    _statusTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _refreshMidiStatus(),
    );
    ref.onDispose(() {
      _statusTimer?.cancel();
      _sub?.cancel();
    });
    _loadScore();
    // Initial MIDI status, read directly (cannot touch `state` during build).
    List<String> ports;
    String? device;
    try {
      ports = midi.listPorts();
      device = midi.connectedPort();
    } catch (_) {
      ports = const [];
      device = null;
    }
    return PlayerData(midiPorts: ports, connectedDevice: device);
  }

  MidiService get _midi => ref.read(midiServiceProvider);

  Future<void> _loadScore() async {
    final score = await ref.read(scoreSourceProvider).demoScore();
    final all = <TimedNote>[];
    for (final m in score.measures) {
      for (final n in m.notes) {
        all.add(
          TimedNote(
            pitch: n.pitch,
            startMs: n.startMs.toInt(),
            durationMs: n.durationMs.toInt(),
          ),
        );
      }
    }
    all.sort((a, b) => a.startMs.compareTo(b.startMs));
    final end = all.isEmpty
        ? 0.0
        : all
              .map((n) => n.startMs + n.durationMs)
              .reduce((a, b) => a > b ? a : b)
              .toDouble();
    state = state.copyWith(score: score, notes: all, songEndMs: end);
  }

  void _onMidi(MidiEvent event) {
    switch (event.kind) {
      case MidiEventKind.noteOn:
        noteOn(event.pitch);
      case MidiEventKind.noteOff:
        noteOff(event.pitch);
    }
  }

  void _refreshMidiStatus() {
    try {
      final ports = _midi.listPorts();
      final device = _midi.connectedPort();
      if (ports.length != state.midiPorts.length ||
          !ports.every(state.midiPorts.contains) ||
          device != state.connectedDevice) {
        state = state.copyWith(midiPorts: ports, connectedDevice: device);
      }
    } catch (_) {
      // MIDI status unavailable; keep the previous state.
    }
  }

  /// Chooses the MIDI device to listen to (null = auto: 1st non-virtual port).
  void selectMidiPort(String? name) {
    try {
      _midi.selectPort(name);
    } catch (_) {}
    _refreshMidiStatus();
  }

  // --- Input (real MIDI or keyboard fallback) ---------------------------

  void noteOn(int pitch) {
    if (!state.activeNotes.contains(pitch)) {
      state = state.copyWith(activeNotes: {...state.activeNotes, pitch});
    }
  }

  void noteOff(int pitch) {
    if (state.activeNotes.contains(pitch)) {
      state = state.copyWith(
        activeNotes: {...state.activeNotes}..remove(pitch),
      );
    }
  }

  // --- Playback controls ------------------------------------------------

  void togglePlay() => state = state.copyWith(isPlaying: !state.isPlaying);
  void setMode(RenderMode m) => state = state.copyWith(mode: m);
  void toggleWaitMode() => state = state.copyWith(waitMode: !state.waitMode);
  void setSpeed(double s) => state = state.copyWith(speed: s.clamp(0.25, 2.0));
  void setKeyboardRange(KeyboardRangeMode m) =>
      state = state.copyWith(keyboardRange: m);
  void restart() => state = state.copyWith(elapsedMs: 0);

  // --- Time advance (called by the screen's Ticker) ---------------------

  /// Advances the playhead by [dtMs] ms (already multiplied by the speed).
  /// Handles Wait Mode freezing and the simple loop at end of song.
  void advance(double dtMs) {
    final s = state;
    if (!s.isPlaying || s.notes.isEmpty) return;

    final required = s.requiredNotesAt(s.elapsedMs);
    if (s.waitMode &&
        required.isNotEmpty &&
        !s.activeNotes.containsAll(required)) {
      // The correct note isn't held: freeze the cascade.
      if (!s.blocked) state = s.copyWith(blocked: true);
      return;
    }

    var next = s.elapsedMs + dtMs;

    // In Wait Mode, don't go past the next note start until it's validated.
    if (s.waitMode) {
      final ns = _nextNoteStart(s.elapsedMs);
      if (ns != null && next > ns) next = ns.toDouble();
    }

    if (s.songEndMs > 0 && next >= s.songEndMs) {
      next = 0; // simple loop
    }

    state = s.copyWith(elapsedMs: next, blocked: false);
  }

  /// Next note start strictly after [t], or null if there are no more.
  int? _nextNoteStart(double t) {
    for (final n in state.notes) {
      if (n.startMs > t + 1) return n.startMs;
    }
    return null;
  }
}
