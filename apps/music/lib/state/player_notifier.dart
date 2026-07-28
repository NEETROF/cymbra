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
import 'countdown.dart';
import 'notation_data.dart';
import 'notation_notifier.dart';
import 'performance_scoring.dart';
import 'player_data.dart';
import 'notation_playback.dart';
import 'player_preferences.dart';
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
    // Seed from the device-persisted play preferences so hands / speed /
    // metronome / MIDI device are remembered across scores and restarts.
    final prefs = ref.read(playerPreferencesProvider);
    // Initial MIDI status, read directly (cannot touch `state` during build).
    List<String> ports;
    String? device;
    try {
      // Re-apply a remembered device before reading the connection. Null = auto,
      // which is already the service default, so only a specific port is applied.
      if (prefs.midiPort != null) midi.selectPort(prefs.midiPort);
      ports = midi.listPorts();
      device = midi.connectedPort();
    } catch (_) {
      ports = const [];
      device = null;
    }
    return PlayerData(
      midiPorts: ports,
      connectedDevice: device,
      selectedHands: prefs.hands,
      speed: prefs.speed,
      metronomeEnabled: prefs.metronome,
    );
  }

  MidiService get _midi => ref.read(midiServiceProvider);
  AudioService get _audio => ref.read(audioServiceProvider);
  PerformanceScorer get _scorer => ref.read(performanceScorerProvider.notifier);

  /// Whether the playhead sits at the fresh-start position (at or before the
  /// effective start) rather than resumed mid-piece. Centralises the "starting
  /// from the top" check for the countdown and scored-run guards, so both honour
  /// the trimmed, possibly non-zero start; a paused playhead is `> startMs`.
  bool _atStart(PlayerData s) => s.elapsedMs <= s.startMs;

  /// Begins a scored run for the current piece if playback is starting cleanly
  /// from the top. Every render mode is scored (Synthesia, the scrolling staff,
  /// and the engraved Partition). Idempotent: a run already active is left alone
  /// (the scorer resets its own state on [PerformanceScorer.startRun]).
  void _maybeStartRun() {
    final s = state;
    // Read-only preview never opens a scored run, so every `PerformanceScorer`
    // method stays an inert no-op and no scoring/performance events fire (change:
    // add-app-score-rating).
    if (s.preview) return;
    if (s.visibleNotes.isEmpty || !_atStart(s)) return;
    _scorer.startRun(
      pieceId: s.title ?? 'demo',
      title: s.title ?? 'Demo',
      hands: s.selectedHands.name,
      speed: s.speed,
      notes: s.visibleNotes,
    );
  }

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
      // The score was pre-loaded before this screen mounted (the hub/library
      // guard). `build()` has not returned yet, so `state` is not initialized —
      // defer the apply to a microtask so it runs after build returns.
      Future.microtask(() => _applyNotation(notation));
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
    final updated = state.copyWith(
      score: null,
      title: document.meta.title,
      bpm: derived.bpm,
      keyFifths: document.attributes.keyFifths,
      beats: document.attributes.time.beats,
      beatType: document.attributes.time.beatType,
      notes: derived.notes,
      rests: derived.rests,
      songEndMs: derived.songEndMs,
      measureStartMs: derived.measureStartMs,
      isPlaying: false,
    );
    // Start a short lead-in before the first note, skipping leading rests/empty
    // measures — computed from the newly-loaded notes and current hand selection.
    state = updated.copyWith(elapsedMs: updated.startMs);
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
    final updated = state.copyWith(
      score: score,
      title: 'Demo — C Major Scale',
      bpm: score.bpm,
      notes: all,
      rests: const [], // the demo score has no rests; clear any prior score's
      songEndMs: end,
    );
    // Seed the playhead at the effective start (0 for the demo, which opens on a
    // note) so every load path shares the same trim-leading-silence rule.
    state = updated.copyWith(elapsedMs: updated.startMs);
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
  /// Persisted so the device is remembered next time.
  void selectMidiPort(String? name) {
    try {
      _midi.selectPort(name);
    } catch (_) {}
    ref.read(playerPreferencesProvider.notifier).setMidiPort(name);
    _refreshMidiStatus();
  }

  // --- Input (real MIDI or keyboard fallback) ---------------------------

  void noteOn(int pitch) {
    // Read-only preview: no user interaction on the played score — ignore every
    // input source so nothing sounds from a press, no gate is touched, and the
    // scorer is never fed (change: add-app-score-rating).
    if (state.preview) return;
    // Every input source converges here, so a single hook sounds the piano for
    // the on-screen keyboard, the computer keyboard, and MIDI alike — during
    // playback and while stopped.
    _audio.noteOn(pitch);
    // A fresh attack starts a new, uncounted hold: drop any prior "consumed"
    // mark so this press can satisfy the onset it lands on (and only that one).
    final active = state.activeNotes.contains(pitch)
        ? state.activeNotes
        : {...state.activeNotes, pitch};
    final consumed = state.consumedHeld.contains(pitch)
        ? ({...state.consumedHeld}..remove(pitch))
        : state.consumedHeld;
    // Wait Mode validates by attack: if this note is part of the onset the
    // playhead is sitting on, latch it so it still counts once released, and
    // consume the hold so a later repeat of this pitch needs a fresh attack.
    final atOnset =
        state.onsetPitchesAt(state.elapsedMs).contains(pitch) &&
        !state.gateSatisfied.contains(pitch);
    state = state.copyWith(
      activeNotes: active,
      gateSatisfied: atOnset
          ? {...state.gateSatisfied, pitch}
          : state.gateSatisfied,
      consumedHeld: atOnset ? {...consumed, pitch} : consumed,
    );
    // Feed the scorer the attack at the current playhead; it binds to a pending
    // onset or records an extra note (a no-op when no run is active). Presses
    // made during the pre-start countdown are warm-ups and are not scored.
    if (state.countdownMs <= 0) {
      _scorer.noteOn(pitch, state.elapsedMs, waitMode: state.waitMode);
    }
  }

  void noteOff(int pitch) {
    // Preview ignores input (see [noteOn]): no release, no scorer note-off.
    if (state.preview) return;
    _audio.noteOff(pitch);
    // The hold ended: drop it from the held set and clear its consumed mark so a
    // re-press starts fresh.
    if (state.activeNotes.contains(pitch) ||
        state.consumedHeld.contains(pitch)) {
      state = state.copyWith(
        activeNotes: {...state.activeNotes}..remove(pitch),
        consumedHeld: {...state.consumedHeld}..remove(pitch),
      );
    }
    _scorer.noteOff(pitch, state.elapsedMs);
  }

  // --- Playback controls ------------------------------------------------

  void togglePlay() => setPlaying(!state.isPlaying);
  // Set the play/pause state explicitly (used to pause while the settings drawer
  // is open and restore the prior state when it closes).
  void setPlaying(bool playing) {
    // Stopping silences any voices and cancels any pending countdown.
    if (!playing) _silenceAll();
    state = state.copyWith(
      isPlaying: playing,
      countdownMs: playing ? state.countdownMs : 0,
    );
    // Starting cleanly from the top opens a scored run.
    if (playing) _maybeStartRun();
  }

  /// Starts playback from the transport, arming a get-ready countdown (5…1…GO)
  /// when starting a **free-run** piece from the top, so the player has time to
  /// ready their hands before the notes start moving. In Wait Mode the cascade
  /// already freezes at the first onset (unlimited ready time), so no countdown
  /// is needed. Resuming mid-piece plays immediately. Plain [setPlaying] stays
  /// countdown-free (used internally and in tests).
  void startPlayback() {
    if (!state.waitMode && _atStart(state) && state.countdownMs == 0) {
      state = state.copyWith(countdownMs: kCountdownStartMs);
    }
    setPlaying(true);
  }

  // Every mode is scored and the scored note set is mode-independent, so
  // switching the render mode keeps the in-flight run (and its gauge/effects)
  // rather than discarding it.
  void setMode(RenderMode m) => state = state.copyWith(mode: m);

  /// Enters (or leaves) read-only preview mode (change: add-app-score-rating):
  /// forces Wait Mode off (with input disabled nothing could satisfy its gate)
  /// and cancels any scored run, so the render plays through read-only. Called by
  /// the deck's in-card preview before autostarting playback.
  void setPreview(bool preview) {
    if (preview) {
      _scorer.cancelRun();
      state = state.copyWith(preview: true, waitMode: false, blocked: false);
    } else {
      state = state.copyWith(preview: false);
    }
  }

  // Re-arm the onset gate at the current playhead when toggling Wait Mode on,
  // and silence any in-flight score voices so none hang across the switch.
  void toggleWaitMode() {
    // Inert in read-only preview (Wait Mode would freeze playback waiting on
    // input that is disabled).
    if (state.preview) return;
    _silenceAll();
    state = state.copyWith(
      waitMode: !state.waitMode,
      gateSatisfied: const {},
      consumedHeld: const {},
    );
  }

  /// Toggles the metronome on/off (driven by the header Tempo chip). Persisted
  /// via [playerPreferencesProvider] so the choice survives pause/stop, switching
  /// pieces and app restarts; ticks resume on the next beat boundary once
  /// playback runs again.
  void toggleMetronome() {
    final next = !state.metronomeEnabled;
    ref.read(playerPreferencesProvider.notifier).setMetronome(enabled: next);
    state = state.copyWith(metronomeEnabled: next);
  }

  /// Sets the playback speed (0.25×–2×) and remembers it across scores/restarts.
  void setSpeed(double s) {
    final clamped = s.clamp(0.25, 2.0);
    ref.read(playerPreferencesProvider.notifier).setSpeed(clamped);
    state = state.copyWith(speed: clamped);
  }

  void setKeyboardRange(KeyboardRangeMode m) =>
      state = state.copyWith(keyboardRange: m);
  void setKeyboardVisible(bool visible) =>
      state = state.copyWith(keyboardVisible: visible);
  // Re-arm the onset gate so a hand switch can't leave the cascade frozen on an
  // onset that is now hidden (or pre-satisfied from the previous selection), and
  // silence voices so a now-hidden hand's notes don't keep sounding.
  void setSelectedHands(Hand hand) {
    ref.read(playerPreferencesProvider.notifier).setHands(hand);
    _silenceAll();
    // Changing the played hand(s) changes which notes are scored, so the piece
    // restarts from the top with a fresh scored run for the new selection: the
    // score stays coherent over the whole piece, the gauge/effects keep working,
    // and the run still finishes into a summary at the end (rather than the
    // cancelled-run case, which would loop with no scoring).
    _scorer.cancelRun();
    final updated = state.copyWith(
      selectedHands: hand,
      gateSatisfied: const {},
      consumedHeld: const {},
    );
    // The effective start depends on the selection, so recompute it for the new
    // hand(s) — a hand that enters later starts trimmed to its own first note.
    state = updated.copyWith(elapsedMs: updated.startMs);
    if (state.isPlaying) _maybeStartRun();
  }

  void restart() {
    _silenceAll();
    // Discard any in-flight run; a fresh one opens when playback next starts
    // from the top in a scored view.
    _scorer.cancelRun();
    state = state.copyWith(
      elapsedMs: state.startMs,
      countdownMs: 0,
      gateSatisfied: const {},
      consumedHeld: const {},
    );
    if (state.isPlaying) _maybeStartRun();
  }

  /// Transport "restart": jump back to the top and start playing again, with the
  /// get-ready countdown (in free run). Used by the restart button and Retry.
  void restartFromTop() {
    restart();
    startPlayback();
  }

  // --- Time advance (called by the screen's Ticker) ---------------------

  /// Advances the playhead by [dtMs] ms (already multiplied by the speed).
  ///
  /// Wait Mode gates on note *onsets*: the cascade freezes at each onset until
  /// every note starting there has been pressed (latched in [PlayerData.gateSatisfied]),
  /// then advances to the next onset — notes do not need to be held for their
  /// duration. A simple loop restarts at the end of the song.
  void advance(double dtMs) {
    var s = state;
    if (!s.isPlaying || s.notes.isEmpty) return;

    // Pre-start countdown: freeze the playhead (and audio/scoring) while the
    // 5…1…GO ticks down in real time, then playback proceeds normally.
    if (s.countdownMs > 0) {
      final remaining = s.countdownMs - dtMs;
      state = s.copyWith(countdownMs: remaining > 0 ? remaining : 0);
      return;
    }

    final onset = s.onsetPitchesAt(s.elapsedMs);

    // Drive the scorer's time-based bookkeeping (gate-open stamping in Wait Mode,
    // miss detection in free run, sustain finalization). Runs before the Wait
    // Mode blocked early-return below so the gate-open time is stamped even while
    // the cascade is frozen. A no-op when no run is active.
    _scorer.tick(s.elapsedMs, waitMode: s.waitMode);

    // Wait Mode tolerance: a key already held (and not already consumed by an
    // earlier onset) when the playhead reaches this onset counts as attacked —
    // a sustained/tied note need not be re-pressed. Consuming the hold keeps a
    // repeated pitch honest: it must be re-attacked to satisfy the next onset.
    if (s.waitMode && onset.isNotEmpty) {
      final heldDue = <int>{
        for (final p in onset)
          if (s.activeNotes.contains(p) && !s.consumedHeld.contains(p)) p,
      };
      if (heldDue.isNotEmpty) {
        s = s.copyWith(
          gateSatisfied: {...s.gateSatisfied, ...heldDue},
          consumedHeld: {...s.consumedHeld, ...heldDue},
        );
        // A sustained/tied note carried into its onset satisfies the gate with
        // no fresh attack — credit the scorer for it (reaction ≈ 0) so it is not
        // later marked missed.
        for (final p in heldDue) {
          _scorer.noteOn(p, s.elapsedMs, waitMode: true);
        }
      }
    }

    if (s.waitMode && onset.isNotEmpty && !s.gateSatisfied.containsAll(onset)) {
      // The onset's notes haven't all been attacked: freeze the cascade.
      // Persist any seeding done above (s no longer identical) even when
      // already blocked; otherwise just latch the blocked flag once.
      if (!identical(s, state) || !s.blocked) state = s.copyWith(blocked: true);
      return;
    }

    var next = s.elapsedMs + dtMs;

    // In Wait Mode, don't go past the next onset until it's validated.
    if (s.waitMode) {
      final ns = s.nextOnsetAfter(s.elapsedMs);
      if (ns != null && next > ns) next = ns;
    }

    var loop = false;
    var finishScoredRun = false;
    var previewEnded = false;
    if (s.songEndMs > 0 && next >= s.songEndMs) {
      if (ref.read(performanceScorerProvider).active) {
        // A scored run ends the piece (produces the summary) instead of looping.
        next = s.songEndMs;
        finishScoredRun = true;
      } else if (s.preview) {
        // Read-only preview plays through once and stops at the end (no loop),
        // so it doesn't run forever behind the deck card.
        next = s.songEndMs;
        previewEnded = true;
      } else {
        next = s.startMs; // simple loop — wrap to the trimmed start, not 0
        loop = true;
      }
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
    // A held pitch stays *consumed* across a normal onset advance (so it can't
    // walk through a repeat), but a loop wrap re-arms from scratch.
    final leftOnset = onset.isNotEmpty && next != s.elapsedMs;
    state = s.copyWith(
      elapsedMs: next,
      blocked: false,
      gateSatisfied: (leftOnset || loop) ? const {} : s.gateSatisfied,
      consumedHeld: loop ? const {} : s.consumedHeld,
      beatCount: beatCount,
      lastBeatAccent: lastBeatAccent,
    );

    // End of a scored run: finalize the result (drives the summary modal) and
    // pause at the last position rather than looping.
    if (finishScoredRun) {
      _silenceAll();
      _scorer.finishRun(next, waitMode: s.waitMode);
      state = state.copyWith(isPlaying: false);
    } else if (previewEnded) {
      // Read-only preview reached the end: silence and stop (no summary, no loop).
      _silenceAll();
      state = state.copyWith(isPlaying: false);
    }
  }
}
