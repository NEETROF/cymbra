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

import '../services/audio_service.dart';
import '../services/midi_service.dart';
import '../src/rust/api/midi.dart';
import '../src/rust/api/musicxml.dart';
import 'notation_data.dart';
import 'notation_notifier.dart';
import 'player_data.dart';
import 'notation_playback.dart';
import 'score_catalog.dart';

part 'player_notifier.g.dart';

/// Central player notifier: pressed keys, score, rendering mode, playhead and
/// Wait Mode logic. Listens to the real-time MIDI stream and also receives notes
/// from the computer-keyboard fallback (via [noteOn]/[noteOff]).
///
/// Content comes from one of two sources: when a library score is selected, the
/// player shows that parsed MusicXML (with playback timing derived from it);
/// otherwise it falls back to the built-in demo score.
@riverpod
class Player extends _$Player {
  Timer? _statusTimer;
  StreamSubscription<MidiEvent>? _sub;
  ScoreDocument? _loadedDocument;

  /// Pitches the score is currently sounding (auto-play), tracked so each note
  /// is released when the playhead passes its end. Audio-only and ephemeral, so
  /// it lives here rather than in [PlayerData].
  final Set<int> _sounding = <int>{};

  @override
  PlayerData build() {
    final midi = ref.watch(midiServiceProvider);
    _sub = midi.events().listen(_onMidi, onError: (Object _) {});
    // Start the piano synth (loads the SoundFont). Fire-and-forget: it is
    // idempotent and degrades to a silent no-op on any failure.
    final audio = ref.read(audioServiceProvider);
    unawaited(audio.init());
    // Poll the MIDI connection state every second (handles hot-plug).
    _statusTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _refreshMidiStatus(),
    );
    // React to the selected score's notation loading / changing without
    // rebuilding (which would reset the playhead and pressed keys).
    ref.listen(notationProvider, (_, next) => _applyNotation(next));
    ref.onDispose(() {
      _statusTimer?.cancel();
      _sub?.cancel();
      // Flush any held/sounding voices so leaving the screen doesn't leave a
      // note ringing in the audio pipeline. Use the captured reference (not
      // ref.read) since the container is disposing.
      audio.allNotesOff();
    });
    _loadInitial();
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
  AudioService get _audio => ref.read(audioServiceProvider);

  /// Releases every sounding score voice (stop / restart / loop / hand switch),
  /// so no note is left hanging.
  void _silenceAll() {
    _audio.allNotesOff();
    _sounding.clear();
  }

  /// Sounds/releases score notes as the playhead travels from [from] to [to].
  void _applyScoreAudio(PlayerData s, double from, double to) {
    final edges = scoreNoteEdges(
      visible: s.visibleNotes,
      from: from,
      to: to,
      sounding: _sounding,
    );
    for (final p in edges.stops) {
      _audio.noteOff(p);
      _sounding.remove(p);
    }
    for (final p in edges.starts) {
      _audio.noteOn(p);
      _sounding.add(p);
    }
  }

  /// Loads the initial content: the selected score's notation if it is already
  /// available, otherwise the demo score (when nothing is selected).
  Future<void> _loadInitial() async {
    final notation = ref.read(notationProvider);
    if (notation.document != null) {
      _applyNotation(notation);
    } else if (ref.read(selectedScoreProvider) == null) {
      await _loadDemo();
    }
    // If a score is selected but not parsed yet, the notation listener applies
    // it once it resolves.
  }

  /// Applies a freshly-parsed MusicXML document: derives the playback timeline
  /// and resets the playhead. Ignored when the document is unchanged (e.g. a
  /// width-driven re-layout) so playback is not disturbed.
  void _applyNotation(NotationData notation) {
    final document = notation.document;
    if (document == null || identical(document, _loadedDocument)) return;
    _loadedDocument = document;
    final derived = notationToTimedNotes(document);
    state = state.copyWith(
      score: null,
      title: document.meta.title,
      bpm: derived.bpm,
      keyFifths: document.attributes.keyFifths,
      beats: document.attributes.time.beats,
      beatType: document.attributes.time.beatType,
      notes: derived.notes,
      songEndMs: derived.songEndMs,
      measureStartMs: derived.measureStartMs,
      elapsedMs: 0,
      isPlaying: false,
    );
  }

  Future<void> _loadDemo() async {
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
    state = state.copyWith(
      score: score,
      title: 'Demo — C Major Scale',
      bpm: score.bpm,
      notes: all,
      songEndMs: end,
    );
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
    // Every input source converges here, so a single hook sounds the piano for
    // the on-screen keyboard, the computer keyboard, and MIDI alike — during
    // playback and while stopped.
    _audio.noteOn(pitch);
    if (!state.activeNotes.contains(pitch)) {
      state = state.copyWith(activeNotes: {...state.activeNotes, pitch});
    }
    // Wait Mode validates by attack: if this note is part of the onset the
    // playhead is sitting on, latch it so it still counts once released.
    if (state.onsetPitchesAt(state.elapsedMs).contains(pitch) &&
        !state.gateSatisfied.contains(pitch)) {
      state = state.copyWith(gateSatisfied: {...state.gateSatisfied, pitch});
    }
  }

  void noteOff(int pitch) {
    _audio.noteOff(pitch);
    if (state.activeNotes.contains(pitch)) {
      state = state.copyWith(
        activeNotes: {...state.activeNotes}..remove(pitch),
      );
    }
  }

  // --- Playback controls ------------------------------------------------

  void togglePlay() => setPlaying(!state.isPlaying);
  // Set the play/pause state explicitly (used to pause while the settings drawer
  // is open and restore the prior state when it closes).
  void setPlaying(bool playing) {
    // Stopping silences any voices the score was sounding.
    if (!playing) _silenceAll();
    state = state.copyWith(isPlaying: playing);
  }

  void setMode(RenderMode m) => state = state.copyWith(mode: m);
  // Re-arm the onset gate at the current playhead when toggling Wait Mode on,
  // and silence any in-flight score voices so none hang across the switch.
  void toggleWaitMode() {
    _silenceAll();
    state = state.copyWith(waitMode: !state.waitMode, gateSatisfied: const {});
  }

  /// Toggles the metronome on/off (driven by the header Tempo chip). A plain
  /// preference flip: it persists across pause/stop and across score changes, and
  /// ticks resume on the next beat boundary once playback runs again.
  void toggleMetronome() =>
      state = state.copyWith(metronomeEnabled: !state.metronomeEnabled);

  void setSpeed(double s) => state = state.copyWith(speed: s.clamp(0.25, 2.0));
  void setKeyboardRange(KeyboardRangeMode m) =>
      state = state.copyWith(keyboardRange: m);
  // Re-arm the onset gate so a hand switch can't leave the cascade frozen on an
  // onset that is now hidden (or pre-satisfied from the previous selection), and
  // silence voices so a now-hidden hand's notes don't keep sounding.
  void setSelectedHands(Hand hand) {
    _silenceAll();
    state = state.copyWith(selectedHands: hand, gateSatisfied: const {});
  }

  void restart() {
    _silenceAll();
    state = state.copyWith(elapsedMs: 0, gateSatisfied: const {});
  }

  // --- Time advance (called by the screen's Ticker) ---------------------

  /// Advances the playhead by [dtMs] ms (already multiplied by the speed).
  ///
  /// Wait Mode gates on note *onsets*: the cascade freezes at each onset until
  /// every note starting there has been pressed (latched in [PlayerData.gateSatisfied]),
  /// then advances to the next onset — notes do not need to be held for their
  /// duration. A simple loop restarts at the end of the song.
  void advance(double dtMs) {
    final s = state;
    if (!s.isPlaying || s.notes.isEmpty) return;

    final onset = s.onsetPitchesAt(s.elapsedMs);
    if (s.waitMode && onset.isNotEmpty && !s.gateSatisfied.containsAll(onset)) {
      // The onset's notes haven't all been attacked: freeze the cascade.
      if (!s.blocked) state = s.copyWith(blocked: true);
      return;
    }

    var next = s.elapsedMs + dtMs;

    // In Wait Mode, don't go past the next onset until it's validated.
    if (s.waitMode) {
      final ns = s.nextOnsetAfter(s.elapsedMs);
      if (ns != null && next > ns) next = ns;
    }

    var loop = false;
    if (s.songEndMs > 0 && next >= s.songEndMs) {
      next = 0; // simple loop
      loop = true;
    }

    // Score audio: sound onsets the playhead crosses and release notes whose end
    // it passes. The half-open span means a frozen Wait Mode onset (next ==
    // elapsedMs) does not pre-sound — it sounds only once time advances past it.
    // A loop wrap silences everything instead of sounding across the seam.
    if (loop) {
      _silenceAll();
    } else {
      _applyScoreAudio(s, s.elapsedMs, next);
    }

    // Metronome: click + pulse on each beat boundary the playhead crosses. Skipped
    // on a loop wrap (no tick across the seam) and naturally silent while paused
    // (the early return above) or frozen in Wait Mode (next == elapsedMs yields no
    // beats). Beats are positional — derived from the score timing each frame — so
    // a seek/restart simply resumes on the next real boundary with no extra tick.
    var beatCount = s.beatCount;
    var lastBeatAccent = s.lastBeatAccent;
    if (!loop && s.metronomeEnabled) {
      final crossed = metronomeBeatsCrossed(
        measureStartMs: s.measureStartMs,
        beats: s.beats,
        bpm: s.bpm,
        songEndMs: s.songEndMs,
        from: s.elapsedMs,
        to: next,
      );
      for (final beat in crossed) {
        _audio.metronomeClick(accent: beat.accent);
        beatCount++;
        lastBeatAccent = beat.accent;
      }
    }

    // Leaving the satisfied onset (or looping) re-arms the gate for the next one.
    final leftOnset = onset.isNotEmpty && next != s.elapsedMs;
    state = s.copyWith(
      elapsedMs: next,
      blocked: false,
      gateSatisfied: (leftOnset || loop) ? const {} : s.gateSatisfied,
      beatCount: beatCount,
      lastBeatAccent: lastBeatAccent,
    );
  }
}
