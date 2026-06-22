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

import 'package:flutter/foundation.dart';

import '../services/midi_service.dart';
import '../src/rust/api/midi.dart';
import '../src/rust/api/score.dart';

/// The two score rendering modes.
enum RenderMode { staff, synthesia }

/// Central player state: pressed keys, score, rendering mode, time position
/// and "Wait Mode" logic.
///
/// Listens to the real-time MIDI stream from Rust and also receives notes from
/// the computer keyboard fallback. It is a [ChangeNotifier]: painters subscribe
/// to it via a [ListenableBuilder].
class PlayerState extends ChangeNotifier {
  /// MIDI engine and score source. Default to the production (FFI) wiring;
  /// tests inject fakes so the state can run without a native library.
  final MidiService _midi;
  final ScoreSource _scores;

  PlayerState({MidiService? midi, ScoreSource? scores})
    : _midi = midi ?? const FrbMidiService(),
      _scores = scores ?? const FrbScoreSource();

  // --- Real-time input --------------------------------------------------

  /// MIDI notes currently pressed (real MIDI keyboard + keyboard fallback).
  final Set<int> activeNotes = <int>{};

  StreamSubscription<MidiEvent>? _midiSub;

  // --- MIDI connection state (on-screen indicator) ----------------------

  /// Detected MIDI devices.
  List<String> midiPorts = const [];

  /// Currently connected port (null if none plugged in).
  String? connectedDevice;

  bool get midiConnected => connectedDevice != null;

  Timer? _statusTimer;

  // --- Score ------------------------------------------------------------

  Score? score;

  /// All notes of the score, flattened and sorted by start.
  /// Pairs each note with its pitch and its bounds in milliseconds (int).
  List<TimedNote> _notes = const [];
  List<TimedNote> get notes => _notes;

  double _songEndMs = 0;
  double get songEndMs => _songEndMs;

  // --- Playback ---------------------------------------------------------

  RenderMode mode = RenderMode.synthesia;
  bool waitMode = true;
  bool isPlaying = false;

  /// Playback position (playhead), in milliseconds.
  double elapsedMs = 0;

  /// Speed multiplier (1.0 = 100%).
  double speed = 1.0;

  /// True when Wait Mode is currently blocking progression.
  bool blocked = false;

  // --- Initialization ---------------------------------------------------

  Future<void> init() async {
    score = await _scores.demoScore();
    _flatten();
    _listenMidi();
    _refreshMidiStatus();
    // Poll the MIDI connection state every second (handles hot-plug).
    _statusTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _refreshMidiStatus(),
    );
    notifyListeners();
  }

  /// Chooses the MIDI device to listen to (null = auto: 1st non-virtual port).
  void selectMidiPort(String? name) {
    try {
      _midi.selectPort(name);
    } catch (e) {
      debugPrint('MIDI port selection failed: $e');
    }
    _refreshMidiStatus();
  }

  void _refreshMidiStatus() {
    try {
      final ports = _midi.listPorts();
      final device = _midi.connectedPort();
      if (ports.length != midiPorts.length ||
          !ports.every(midiPorts.contains) ||
          device != connectedDevice) {
        midiPorts = ports;
        connectedDevice = device;
        notifyListeners();
      }
    } catch (e) {
      debugPrint('MIDI status unavailable: $e');
    }
  }

  void _flatten() {
    final s = score;
    if (s == null) return;
    final all = <TimedNote>[];
    for (final m in s.measures) {
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
    _notes = all;
    _songEndMs = all.isEmpty
        ? 0
        : all
              .map((n) => (n.startMs + n.durationMs))
              .reduce((a, b) => a > b ? a : b)
              .toDouble();
  }

  void _listenMidi() {
    _midiSub = _midi.events().listen((event) {
      switch (event.kind) {
        case MidiEventKind.noteOn:
          noteOn(event.pitch);
        case MidiEventKind.noteOff:
          noteOff(event.pitch);
      }
    }, onError: (Object e) => debugPrint('MIDI stream error: $e'));
  }

  // --- Input (real MIDI or keyboard fallback) ---------------------------

  void noteOn(int pitch) {
    if (activeNotes.add(pitch)) notifyListeners();
  }

  void noteOff(int pitch) {
    if (activeNotes.remove(pitch)) notifyListeners();
  }

  // --- Playback controls ------------------------------------------------

  void togglePlay() {
    isPlaying = !isPlaying;
    notifyListeners();
  }

  void setMode(RenderMode m) {
    mode = m;
    notifyListeners();
  }

  void toggleWaitMode() {
    waitMode = !waitMode;
    notifyListeners();
  }

  void setSpeed(double s) {
    speed = s.clamp(0.25, 2.0);
    notifyListeners();
  }

  void restart() {
    elapsedMs = 0;
    notifyListeners();
  }

  // --- Time advance (called by the screen's Ticker) ---------------------

  /// Advances the playhead by [dtMs] milliseconds (already multiplied by the
  /// speed). Handles Wait Mode freezing and the simple loop at end of song.
  void advance(double dtMs) {
    if (!isPlaying || _notes.isEmpty) return;

    final required = requiredNotesAt(elapsedMs);

    if (waitMode && required.isNotEmpty && !activeNotes.containsAll(required)) {
      // The correct note isn't held: we freeze the cascade.
      if (!blocked) {
        blocked = true;
        notifyListeners();
      }
      return;
    }

    if (blocked) blocked = false;

    var next = elapsedMs + dtMs;

    // In Wait Mode, we don't go past the next note start until it's been
    // validated: we stop right on the bar to re-evaluate the freeze.
    if (waitMode) {
      final ns = _nextNoteStart(elapsedMs);
      if (ns != null && next > ns) next = ns.toDouble();
    }

    if (_songEndMs > 0 && next >= _songEndMs) {
      next = 0; // simple loop
    }

    elapsedMs = next;
    notifyListeners();
  }

  /// Notes that should be held at instant [t] (playhead within the
  /// window [start, start+duration]). Acts as the "gate" for Wait Mode.
  Set<int> requiredNotesAt(double t) {
    final result = <int>{};
    for (final n in _notes) {
      if (n.startMs <= t + 1 && t < n.startMs + n.durationMs) {
        result.add(n.pitch);
      }
    }
    return result;
  }

  /// Next note start strictly after [t], or null if there are no more.
  int? _nextNoteStart(double t) {
    for (final n in _notes) {
      if (n.startMs > t + 1) return n.startMs;
    }
    return null;
  }

  @override
  void dispose() {
    _statusTimer?.cancel();
    _midiSub?.cancel();
    super.dispose();
  }
}

/// A score note with its time bounds in milliseconds (int), more convenient to
/// handle on the Dart side than the bridge's `BigInt`.
class TimedNote {
  final int pitch;
  final int startMs;
  final int durationMs;

  const TimedNote({
    required this.pitch,
    required this.startMs,
    required this.durationMs,
  });
}
