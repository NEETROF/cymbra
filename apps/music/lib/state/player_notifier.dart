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

import 'package:flutter/foundation.dart' show mapEquals, setEquals;

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/audio_service.dart';
import '../services/midi_service.dart';
import '../src/rust/api/midi.dart';
import '../src/rust/api/musicxml.dart';
import 'countdown.dart';
import 'notation_data.dart';
import 'notation_notifier.dart';
import 'performance_scoring.dart';
import 'play_sync_notifier.dart';
import 'practice_settings_store.dart';
import 'drum_input_mapping.dart';
import 'drum_input_mapping_notifier.dart';
import 'drum_kit.dart';
import 'player_data.dart';
import 'score_font.dart';
import 'notation_playback.dart';
import 'player_preferences.dart';
import 'score_catalog.dart';
import 'usage_tracking_notifier.dart';
import '../analytics/usage_actions.dart';

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

  /// Whether the current selective run has already been recorded as a
  /// **countable practice session** (change: add-measure-range-practice, D4): a
  /// run that has actually sounded at least one onset (the "started then quit"
  /// threshold). Recorded **once per session, never per lap**, so looping cannot
  /// inflate the day's practice count; the flag is cleared at a session boundary
  /// (range change, score change) so the next one opens a fresh record.
  bool _practiceRecorded = false;

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
    // The kit font arriving changes what a live stroke may sound (change:
    // add-drum-audio-channel), and the engine's echo has to learn it at the
    // same instant the notifier would have.
    ref.listen(scoreFontProvider, (_, _) => _applyEcho());
    // A calibration completing, an edited entry or a cleared device all change
    // what the engine must translate (change: add-drum-input-calibration).
    // Listened, not watched: this notifier must not rebuild — and reset the
    // playhead — because a mapping was saved.
    ref.listen(drumInputMappingStoreProvider, (_, _) => _applyMapping());
    ref.onDispose(() {
      _statusTimer?.cancel();
      _sub?.cancel();
      // The player screen is gone: the engine must stop sounding live notes on
      // its own behalf (see [_applyEcho]) — no other surface plays what the
      // instrument sends. Never let a missing engine break a teardown.
      try {
        midi.setEcho(MidiEcho.off);
      } catch (_) {}
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
      keyboardRange: prefs.keyboardRange,
      readingAid: prefs.readingAid,
      instrumentSoundsItself: prefs.instrumentSoundsItself,
      scoreAudioMuted: prefs.scoreAudioMuted,
      outputOffsetMs: prefs.outputOffsetMs,
      invertedKit: prefs.invertedKit,
    );
  }

  /// The echo mode last pushed to the engine (change: add-drum-input-mapping —
  /// beta fix for input latency): what the engine is sounding on its own, and
  /// therefore what this notifier must not sound a second time. Null until the
  /// first push, so the engine is always told once.
  MidiEcho? _echo;

  /// Whether a MIDI-sourced note reaching [noteOn] has ALREADY been sounded by
  /// the engine, in its own callback, before it ever crossed the bridge.
  bool get _engineEchoes => _echo != null && _echo != MidiEcho.off;

  /// Pushes the app's sounding policy to the engine so a live note is played
  /// where it is heard soonest — in the MIDI callback rather than after a trip
  /// through the Dart event loop, which is what a beta tester heard as "strong
  /// latency between the hit and the sound".
  ///
  /// The policy itself is unchanged and still lives here; only the execution
  /// moves. Both existing guards are honoured, so what sounds is exactly what
  /// sounded before:
  /// * [PlayerData.synthesizes] — an instrument that sounds its own notes is
  ///   never doubled (change: add-audio-output-routing);
  /// * the kit-readiness gate — until the kit font is installed a drum score is
  ///   visual-only, and a stroke must not come out through the piano font
  ///   (change: add-drum-audio-channel).
  ///
  /// Called on every input that can change the answer, and idempotent, so it is
  /// safe to over-call.
  void _applyEcho() {
    final mode = _echoMode();
    if (mode == _echo) return;
    _echo = mode;
    try {
      _midi.setEcho(mode);
    } catch (_) {
      // No engine (tests, a failed native load): the notifier keeps sounding
      // live notes itself, exactly as it did before this existed.
      _echo = MidiEcho.off;
    }
  }

  /// The translation table last pushed to the engine (change:
  /// add-drum-input-calibration). Null until the first push, so the engine is
  /// always told once — even when the answer is "no mapping".
  Map<int, int>? _pushedMapping;

  /// Pushes the connected device's mapping to the engine, which sounds a live
  /// stroke from its own MIDI callback and therefore has to translate it there.
  ///
  /// The same shape as [_applyEcho], for the same reason: the app owns the
  /// policy — which device is connected, what it was calibrated to — and the
  /// engine only applies it. Called on every input that can change the answer
  /// (device change, calibration completing, an edited entry, leaving the
  /// player), and idempotent, so over-calling is safe.
  void _applyMapping() {
    // `_mappingForDevice` already answers "identity on a keyboard score", so
    // the engine is pushed exactly what this side applies — the two cannot
    // drift, which is the whole point of there being one policy.
    final table = _mappingForDevice().translationTable;
    if (_pushedMapping != null && mapEquals(_pushedMapping, table)) return;
    _pushedMapping = table;
    try {
      _midi.setMapping(table);
    } catch (_) {
      // No engine (tests, a failed native load): the notifier still translates
      // on its own side, so what is scored stays correct — only the engine's
      // echo would sound the raw number, and without an engine there is none.
      _pushedMapping = const {};
    }
  }

  MidiEcho _echoMode() {
    if (!state.synthesizes(NoteSource.midiDevice)) return MidiEcho.off;
    if (!state.isPercussion) return MidiEcho.melodic;
    return ref.read(scoreFontProvider) == KitFontStatus.ready
        ? MidiEcho.drum
        : MidiEcho.off;
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
  ///
  /// A **selective run** (a practice range narrower than the whole piece) is
  /// never scored (change: add-measure-range-practice, D2): the scorer simply
  /// never arms, so no partial `SessionResult` exists to suppress downstream.
  /// The rule is instrument-agnostic and covers a percussion practice run with
  /// no carve-out.
  ///
  /// A **percussion** full run arms it like any other (change:
  /// add-drum-scoring): the scorer now judges strokes at the kit piece's grain
  /// over a two-dimension blend, so the run produces an honest session result.
  /// The interim that kept it disarmed existed only because the judge was
  /// keyboard-shaped — exact-pitch matching against numbers a drum lane
  /// deliberately collapses, sustain judgment against one-shots that have
  /// none.
  void _maybeStartRun() {
    final s = state;
    if (s.visibleNotes.isEmpty || !_atStart(s) || s.isSelectiveRun) return;
    final entry = ref.read(selectedScoreProvider);
    _scorer.startRun(
      pieceId: _pieceIdentity(),
      title: entry?.title ?? s.title ?? 'Demo',
      // hands / feet / both on a drum score — the selection the run was
      // actually judged over (change: add-drum-scoring).
      hands: s.handsSelectionName,
      speed: s.speed,
      notes: s.visibleNotes,
      percussion: s.isPercussion,
    );
  }

  /// The identity the scored-run, practice-activity and per-score-settings paths
  /// all report this piece under (see [pieceIdentityOf]).
  String _pieceIdentity() =>
      pieceIdentityOf(ref.read(selectedScoreProvider), state.title);

  /// Persists (or forgets) this score's practice settings after any change to
  /// the range (change: add-measure-range-practice, D7), so
  /// reopening the piece pre-fills what the player was last drilling. Returning
  /// to a full run forgets the selection.
  void _persistPracticeSettings() {
    final s = state;
    final store = ref.read(practiceSettingsStoreProvider);
    final id = _pieceIdentity();
    if (!s.isSelectiveRun) {
      unawaited(store.clear(id));
      return;
    }
    unawaited(
      store.save(
        id,
        PracticeSettings(
          startMeasure: s.practiceStartMeasure!,
          endMeasure: s.practiceEndMeasure!,
        ),
      ),
    );
  }

  /// Records the in-flight selective run as a **countable practice session** —
  /// the design's "at least one onset elapsed" threshold, so opening a range and
  /// quitting without playing anything never counts.
  ///
  /// The record is captured **as soon as the session becomes countable**, not
  /// when it ends. The outbox entry is durable from that moment, so a practice
  /// that never gets a clean ending — the app killed mid-loop, the device dying,
  /// a crash — still reaches the server. Waiting for the end used to drop those
  /// silently, and now that a practice day also holds the streak, a lost record
  /// costs the player a day they actually earned.
  ///
  /// Guarded by [_practiceRecorded], so a looping session is still captured
  /// exactly once, never per lap. `practiced_at_ms` is therefore stamped when the
  /// player started working the passage rather than when they stopped — the same
  /// local day in every case that is not a session straddling midnight.
  void _markPracticeProgress() {
    if (_practiceRecorded) return;
    _practiceRecorded = true;
    unawaited(
      ref
          .read(playSyncNotifierProvider.notifier)
          .capturePractice(scoreId: _pieceIdentity())
          // Best-effort: a container already tearing down makes the capture
          // fail, and nothing could be delivered then anyway.
          .catchError((Object _) {}),
    );
  }

  /// Closes the in-flight practice session so the next one opens a fresh record.
  /// The record itself was already captured by [_markPracticeProgress]; this only
  /// re-arms the guard at a session boundary (a new range, a new piece).
  void _endPracticeSession() => _practiceRecorded = false;

  /// Releases every sounding score voice (stop / restart / loop / hand switch),
  /// so no note is left hanging.
  void _silenceAll() {
    _audio.allNotesOff();
    _sounding.clear();
  }

  /// Sounds/releases score notes as the playhead travels from [from] to [to],
  /// routed by the loaded score's family (change: add-drum-audio-channel): a
  /// percussion score's General MIDI numbers go through the drum entry points,
  /// a keyboard score's pitches through the melodic pair exactly as before.
  ///
  /// [justPlayed] holds the numbers the player struck themselves at the onset
  /// this span crosses — the schedule owes them no second attack (see the call
  /// site in [advance]).
  void _applyScoreAudio(
    PlayerData s,
    double from,
    double to, {
    Set<int> justPlayed = const {},
  }) {
    // The written part is muted (change: add-practice-focus-controls). Returning
    // before the edges are computed skips the bookkeeping with the attack, so no
    // release is ever owed for a voice that was never started — the shape of the
    // double-strike bug `add-drum-audio-channel` 10.3 fixed. `_sounding` was
    // emptied when the mute went on, so there is nothing left hanging either.
    if (s.scoreAudioMuted) return;
    if (s.isPercussion) {
      // Percussion readiness gate: until the kit font's awaited install has
      // resolved (KitFontStatus.ready), playback is visual-only — the
      // schedule advances but nothing reaches the synth, so a drum part is
      // never sounded through the still-loaded keyboard font. `_sounding` is
      // untouched here, so no phantom release accumulates either.
      if (ref.read(scoreFontProvider) != KitFontStatus.ready) return;
      final edges = scoreNoteEdges(
        visible: s.visibleNotes,
        from: from,
        to: to,
        sounding: _sounding,
      );
      for (final p in edges.stops) {
        _audio.drumOff(p);
        _sounding.remove(p);
      }
      for (final p in edges.starts) {
        // The stroke the player just made IS this note: sounding it again puts
        // a flam on every gated onset — one hit, two kicks — which is what the
        // beta report heard on the bass drum. Skipped whole, `_sounding`
        // included, so no release is owed for a voice never started.
        if (justPlayed.contains(p)) continue;
        _audio.drumOn(p);
        _sounding.add(p);
      }
      return;
    }
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
  /// available, otherwise the demo score (when nothing is selected). Also the
  /// first push of the engine's echo mode, once `state` exists.
  Future<void> _loadInitial() async {
    Future.microtask(_applyEcho);
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
    // A different piece ends any practice session on the previous one.
    _endPracticeSession();
    _loadedDocument = document;
    final derived = notationToTimedNotes(document);
    // The engraved title wins — it is what the score itself says — but plenty of
    // MusicXML carries no `<work-title>` (three of the bundled scores don't), and
    // overwriting with nothing left the header reading "Now Playing: —" for a
    // piece the library had just named. Fall back to the entry that was opened,
    // the same precedence `_maybeStartRun` already applies to a scored run.
    final documentTitle = document.meta.title;
    final updated = state.copyWith(
      score: null,
      title: documentTitle == null || documentTitle.isEmpty
          ? ref.read(selectedScoreProvider)?.title
          : documentTitle,
      bpm: derived.bpm,
      keyFifths: document.attributes.keyFifths,
      beats: document.attributes.time.beats,
      beatType: document.attributes.time.beatType,
      notes: derived.notes,
      rests: derived.rests,
      tieContinuations: derived.tieContinuations,
      songEndMs: derived.songEndMs,
      measureStartMs: derived.measureStartMs,
      measureKeyFifths: derived.measureKeyFifths,
      writtenMeasureOf: derived.writtenMeasureOf,
      measureDecors: derived.measureDecors,
      writtenMeasureCount: document.measures.length,
      // Percussion routing (change: add-drum-kit-view): the lane layout is
      // derived ONCE here and consumed by both the cascade and the pad strip.
      // The cascade stays the DEFAULT presentation on load — the notation
      // modes are offered but entered only by the player's choice (change:
      // add-drum-notation-render). Wait Mode is left exactly as the player set
      // it: the pads satisfy the gate now (change: add-drum-input-mapping) and
      // the matcher judges what they satisfy (change: add-drum-scoring), so a
      // drum score no longer forces it off.
      isPercussion: derived.isPercussion,
      drumLanes: derived.isPercussion
          ? deriveDrumLanes(derived.notes)
          : const <DrumLane>[],
      mode: _modeForLoadedScore(percussion: derived.isPercussion),
      // A focus selection describes the passage being worked on, not a
      // preference (change: add-practice-focus-controls): another score's
      // selection would hand this one a kit with holes in it, so every piece is
      // in focus on load.
      mutedDrumPieces: const {},
      // The struck-flash table is keyed by controller POSITION, so another
      // score's stamps would land on this one's pads (change:
      // add-drum-input-mapping).
      struckSurfacesMs: const {},
      strokeAtMs: const {},
      isPlaying: false,
      // A range chosen for the previous score means nothing here (and its indices
      // may not even exist in this one): a freshly-loaded document always starts
      // as a full run. The per-score saved settings re-apply it if there are any.
      practiceStartMeasure: null,
      practiceEndMeasure: null,
    );
    // Start a short lead-in before the first note, skipping leading rests/empty
    // measures — computed from the newly-loaded notes and current hand selection.
    // A new score also resets how much of it has been heard.
    state = updated.copyWith(
      elapsedMs: updated.startMs,
      furthestElapsedMs: updated.startMs,
    );
    // A different family sounds a live note on a different channel.
    _applyEcho();
    // …and it changes whether the device's mapping applies at all: a kit table
    // is a statement about pieces, so it is inert on a keyboard score. This is
    // also the first push, once `state` exists to name a device.
    _applyMapping();
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
      tieContinuations: const [], // nor ties
      writtenMeasureOf: const [],
      measureDecors: const [],
      writtenMeasureCount: 0,
      songEndMs: end,
    );
    // Seed the playhead at the effective start (0 for the demo, which opens on a
    // note) so every load path shares the same trim-leading-silence rule — and
    // reset how much of the (new) piece has been heard.
    state = updated.copyWith(
      elapsedMs: updated.startMs,
      furthestElapsedMs: updated.startMs,
    );
  }

  /// The one point a live MIDI event enters the app — and therefore the one
  /// point its number is translated (change: add-drum-input-calibration,
  /// design D2).
  ///
  /// Everything downstream keeps its General MIDI vocabulary and never learns
  /// that a mapping exists: `noteOn` sounds it, `laneIndexOf` flashes it, the
  /// gate opens on it and the scorer credits it, all on the translated number.
  /// Four translations would be four chances for those answers to disagree
  /// about what was just played, which is the failure this seam exists to make
  /// impossible.
  ///
  /// The mapping is read per event rather than cached: it is a map lookup on a
  /// table with at most a dozen entries, and the alternative — a cached copy
  /// invalidated on device change, on calibration completing and on an edit —
  /// is three more chances to serve a stale table.
  void _onMidi(MidiEvent event) {
    final pitch = _mappingForDevice().translate(event.pitch);
    switch (event.kind) {
      case MidiEventKind.noteOn:
        noteOn(pitch, source: NoteSource.midiDevice);
      case MidiEventKind.noteOff:
        noteOff(pitch, source: NoteSource.midiDevice);
    }
  }

  /// The mapping to apply to a live event right now: the connected device's
  /// learned table on a **percussion** score, the identity everywhere else.
  ///
  /// Scoped to percussion deliberately. The table says "this pad is the snare",
  /// which is a statement about kit pieces; applying it to a keyboard score
  /// would bend that score's *pitches* — a drummer who calibrated their kit to
  /// send 31 for the snare would find middle D transposed on the piano, for a
  /// mapping that was never about pitch. The device may well be the same one
  /// (a module that also has keys), so "is a kit connected" cannot answer this;
  /// what the score asks for can.
  ///
  /// Read from the store directly rather than through
  /// `activeDrumMappingProvider`, which resolves the port from
  /// `midiStatusProvider` — this notifier already knows its own connected
  /// device, and depending on that provider from here would close a cycle.
  DrumInputMapping _mappingForDevice() => state.isPercussion
      ? ref
            .read(drumInputMappingStoreProvider.notifier)
            .forPort(state.connectedDevice)
      : DrumInputMapping.empty;

  void _refreshMidiStatus() {
    try {
      final ports = _midi.listPorts();
      final device = _midi.connectedPort();
      if (ports.length != state.midiPorts.length ||
          !ports.every(state.midiPorts.contains) ||
          device != state.connectedDevice) {
        final deviceChanged = device != state.connectedDevice;
        state = state.copyWith(midiPorts: ports, connectedDevice: device);
        // A different kit is a different table (change:
        // add-drum-input-calibration) — including "no kit", which is the
        // identity. Hot-plug reaches here, so this covers it too.
        if (deviceChanged) _applyMapping();
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

  /// Turns the instrument-sounds-itself rule on or off (change:
  /// add-audio-output-routing) and remembers it across restarts.
  ///
  /// Silences every sounding voice across the change: a MIDI key held while the
  /// rule turns on would otherwise never receive its release (the note-off is
  /// suppressed) and hang.
  void setInstrumentSoundsItself({required bool enabled}) {
    if (state.instrumentSoundsItself == enabled) return;
    _silenceAll();
    state = state.copyWith(instrumentSoundsItself: enabled);
    // The engine echoes on the app's behalf, so it has to learn the rule too.
    _applyEcho();
    ref
        .read(playerPreferencesProvider.notifier)
        .setInstrumentSoundsItself(enabled: enabled);
  }

  /// Silences (or restores) the app's playback of the **written score** (change:
  /// add-practice-focus-controls) and remembers it across restarts.
  ///
  /// Nothing else about the session changes: the playhead keeps advancing, the
  /// score keeps being drawn, the Wait Mode gate keeps holding and releasing, the
  /// scorer keeps judging, and the metronome keeps clicking. It is the exercise's
  /// own voice that stops — the thing that, on a kit, masks the player's strokes
  /// and the click on the same percussion timbres.
  ///
  /// Silences every sounding voice across the change, for the same reason
  /// [setInstrumentSoundsItself] does: a note the schedule started is owed a
  /// release, and muting on the way out would suppress the one it is waiting
  /// for. `_sounding` is cleared with it, so no release is ever owed for a voice
  /// that is no longer playing.
  void setScoreAudioMuted({required bool muted}) {
    if (state.scoreAudioMuted == muted) return;
    _silenceAll();
    state = state.copyWith(scoreAudioMuted: muted);
    ref
        .read(playerPreferencesProvider.notifier)
        .setScoreAudioMuted(muted: muted);
  }

  /// Sets the output latency compensation (change: add-audio-output-routing)
  /// and remembers it. Clamped by the preferences store; the player mirrors what
  /// was actually stored so the two cannot disagree.
  void setOutputOffsetMs(int ms) {
    final prefs = ref.read(playerPreferencesProvider.notifier)
      ..setOutputOffsetMs(ms);
    state = state.copyWith(outputOffsetMs: prefs.state.outputOffsetMs);
  }

  // --- Input (real MIDI or keyboard fallback) ---------------------------

  /// A live note-on from [source]. Every input source converges here, so a
  /// single hook drives sounding, key feedback, the Wait Mode gate and scoring
  /// for the on-screen keyboard, the computer keyboard and MIDI alike — during
  /// playback and while stopped.
  ///
  /// [source] is consulted for **sounding only** (change:
  /// add-audio-output-routing): a note the connected instrument already played
  /// itself is not synthesized a second time. Everything below the audio call
  /// runs identically for every source.
  ///
  /// On a **percussion** score the pitch is a General MIDI percussion number
  /// and the note is a *stroke* (change: add-drum-input-mapping): it sounds
  /// through the seam's one-shot rather than the pitched piano voice, and
  /// flashes the controller surface it resolves to. Everything else — the held
  /// set, the gate bookkeeping, the scorer feed — is the shared path, with the
  /// one substitution `add-drum-scoring` makes: what counts as "this note" is
  /// the kit-piece equivalence, not the raw number.
  void noteOn(int pitch, {NoteSource source = NoteSource.onScreen}) {
    // A note from the instrument was already sounded by the engine, in its own
    // MIDI callback, before it crossed the bridge (see [_applyEcho]) — sounding
    // it here too would double every stroke. Every other source still sounds
    // from here, and everything below this line is untouched by the echo.
    final echoed = source == NoteSource.midiDevice && _engineEchoes;
    if (state.isPercussion) {
      if (!echoed) _soundStroke(pitch, source);
    } else if (!echoed && state.synthesizes(source)) {
      _audio.noteOn(pitch);
    }
    // A fresh attack starts a new, uncounted hold: drop any prior "consumed"
    // mark so this press can satisfy the onset it lands on (and only that one).
    final active = state.activeNotes.contains(pitch)
        ? state.activeNotes
        : {...state.activeNotes, pitch};
    final consumed = state.consumedHeld.contains(pitch)
        ? ({...state.consumedHeld}..remove(pitch))
        : state.consumedHeld;
    // Wait Mode validates by attack: if this note answers the onset the
    // playhead is sitting on, latch it so it still counts once released, and
    // consume the hold so a later repeat of this pitch needs a fresh attack.
    //
    // What "answers" means goes through the shared stroke identity
    // ([PlayerData.strokeSatisfies]) rather than raw equality, so on a drum
    // score an incoming 40 latches the written 38 the gate is waiting for —
    // the same resolution the scorer binds with, never a second one. The
    // *required* numbers are latched, so the release check keeps comparing
    // against the score's own vocabulary.
    final satisfied = <int>{
      for (final required in state.onsetPitchesSatisfiedBy(
        pitch,
        state.elapsedMs,
      ))
        if (!state.gateSatisfied.contains(required)) required,
    };
    final atOnset = satisfied.isNotEmpty;
    final struck = _struckSurfacesAfter(pitch);
    state = state.copyWith(
      activeNotes: active,
      gateSatisfied: atOnset
          ? {...state.gateSatisfied, ...satisfied}
          : state.gateSatisfied,
      consumedHeld: atOnset ? {...consumed, pitch} : consumed,
      struckSurfacesMs: struck,
      // A stroke that answered the gate right here is SPENT: it is not left
      // in the table where the early-stroke tolerance could credit it to the
      // next onset as well. Any other stroke is remembered, on the playhead's
      // clock, in case the onset it was aimed at is a few milliseconds away.
      strokeAtMs: _strokeAtAfter(pitch, spent: atOnset),
    );
    // Feed the scorer the attack at the current playhead; it binds to a pending
    // onset or records an extra note (a no-op when no run is active). Presses
    // made during the pre-start countdown are warm-ups and are not scored.
    if (state.countdownMs <= 0) {
      _scorer.noteOn(pitch, state.clocks, waitMode: state.waitMode);
    }
  }

  /// Sounds a live percussion stroke as a **one-shot** (change:
  /// add-drum-input-mapping): `drumOn`, never the pitched `noteOn` a live
  /// stroke wrongly got before this change — the drum channel is where the kit
  /// font's bank-128 presets resolve, so a snare number sent to the melodic
  /// pair comes out as a piano note.
  ///
  /// Two guards, both borrowed unchanged rather than re-decided here:
  /// * [PlayerData.synthesizes] — the instrument-sounds-itself rule (change:
  ///   add-audio-output-routing): a module that sounds its own strokes is not
  ///   doubled, while on-screen taps always sound.
  /// * the kit-readiness gate the scheduled path already applies (change:
  ///   add-drum-audio-channel): until the kit font's awaited install resolves,
  ///   percussion is visual-only — a stroke must not come out through the
  ///   still-loaded piano font. The stroke still registers and still flashes;
  ///   it is *sounding* that is unavailable, and only that.
  ///
  /// Velocity is deliberately not consumed: the one-shot is filled with the
  /// schedule's own default loudness, exactly like the pitched path, which has
  /// never consumed velocity either (change: add-drum-input-mapping — dynamics
  /// is one future decision for both instruments and both directions, not a
  /// percussion side-door).
  void _soundStroke(int pitch, NoteSource source) {
    if (!state.synthesizes(source)) return;
    if (ref.read(scoreFontProvider) != KitFontStatus.ready) return;
    _audio.drumOn(pitch, velocity: AudioService.defaultVelocity);
  }

  /// The mode a freshly loaded score opens in: the one last chosen for ITS
  /// family, or the family default when the player has never chosen — the
  /// cascade, which is the designed reading surface for playing.
  ///
  /// A stored mode is still checked against the family: the stage exists only
  /// for percussion, and a record from another build could name it.
  RenderMode _modeForLoadedScore({required bool percussion}) {
    final prefs = ref.read(playerPreferencesProvider);
    if (percussion) return prefs.percussionMode ?? RenderMode.synthesia;
    final stored = prefs.keyboardMode;
    return stored == null || stored == RenderMode.stage ? state.mode : stored;
  }

  /// The struck-surface table after a stroke on [gm] (change:
  /// add-drum-input-mapping): the surface it resolves to, stamped with the
  /// wall clock. A number resolving to no surface — a piece this score does
  /// not use, a kick on a kickless score — leaves the table untouched: free
  /// play sounds, flashes nothing, and raises nothing.
  ///
  /// Keyed on the surface, so any source (pad tap, pedal tap, external kit)
  /// produces the same entry, and a lane collapsing several numbers flashes
  /// once wherever the stroke came from.
  Map<int, double> _struckSurfacesAfter(int gm) {
    if (!state.isPercussion) return state.struckSurfacesMs;
    final surface = state.struckSurfaceFor(gm);
    if (surface == null) return state.struckSurfacesMs;
    return {
      ...state.struckSurfacesMs,
      surface: DateTime.now().millisecondsSinceEpoch.toDouble(),
    };
  }

  /// [PlayerData.strokeAtMs] after a stroke on [gm]: stamped at the playhead
  /// for a stroke that answered nothing yet, dropped when it was [spent] on
  /// the onset it just satisfied.
  Map<int, double> _strokeAtAfter(int gm, {required bool spent}) {
    if (!state.isPercussion) return state.strokeAtMs;
    final surface = state.struckSurfaceFor(gm);
    if (surface == null) return state.strokeAtMs;
    final next = {...state.strokeAtMs};
    if (spent) {
      next.remove(surface);
    } else {
      next[surface] = state.elapsedMs;
    }
    return next;
  }

  /// Releases a live note from [source]. Mirrors [noteOn]: the release is only
  /// sent to the synth when the attack was, so the two stay paired.
  ///
  /// On a **percussion** score a release is *bookkeeping, never meaning*
  /// (change: add-drum-input-mapping): it clears the held entry its attack
  /// created and nothing else — no audible effect, no feedback effect (the
  /// flash runs on its own clock). It is deliberately **not** forwarded to the
  /// seam's paired `drum_off`: that release reaches the engine as a NoteOff on
  /// the drum channel (`api/audio_core.rs`), which puts the voice into the
  /// preset's release stage — with the SoundFont default release that *clips*
  /// the one-shot. E-kits send their note-off within milliseconds of the
  /// attack, so forwarding it would cut every cymbal off the moment the stick
  /// left it. The binding scenario ("an immediate release leaves the sound to
  /// its natural end"), not the pairing, is the contract; a kit voice
  /// self-terminates, so nothing hangs when the release is dropped — and a kit
  /// that never sends one costs nothing either.
  void noteOff(int pitch, {NoteSource source = NoteSource.onScreen}) {
    // Paired with [noteOn]: the engine's melodic echo releases its own voice.
    final echoed = source == NoteSource.midiDevice && _engineEchoes;
    if (!state.isPercussion && !echoed && state.synthesizes(source)) {
      _audio.noteOff(pitch);
    }
    // The hold ended: drop it from the held set and clear its consumed mark so a
    // re-press starts fresh.
    if (state.activeNotes.contains(pitch) ||
        state.consumedHeld.contains(pitch)) {
      state = state.copyWith(
        activeNotes: {...state.activeNotes}..remove(pitch),
        consumedHeld: {...state.consumedHeld}..remove(pitch),
      );
    }
    _scorer.noteOff(pitch, state.clocks);
  }

  // --- Playback controls ------------------------------------------------

  void togglePlay() => setPlaying(!state.isPlaying);
  // Set the play/pause state explicitly (used to pause while the settings drawer
  // is open and restore the prior state when it closes).
  void setPlaying(bool playing) {
    final wasPlaying = state.isPlaying;
    // Stopping silences any voices and cancels any pending countdown.
    if (!playing) _silenceAll();
    state = state.copyWith(
      isPlaying: playing,
      countdownMs: playing ? state.countdownMs : 0,
    );
    // Starting cleanly from the top opens a scored run.
    if (playing) _maybeStartRun();
    // Usage telemetry (change: add-feature-usage-analytics) on a real transition.
    // (A settings-drawer pause/restore may add minor start/stop noise — accepted.)
    if (playing && !wasPlaying) {
      _track(UsageActions.playStart, subjectId: _currentScoreId);
    } else if (!playing && wasPlaying) {
      _track(UsageActions.playStop, subjectId: _currentScoreId);
    }
  }

  /// Fire-and-forget usage tracking (only a notifier calls the tracking notifier;
  /// emission is gated on consent + kill-switch inside it).
  void _track(String action, {String? variant, String? subjectId}) {
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(action, variant: variant, subjectId: subjectId),
    );
  }

  String? get _currentScoreId => ref.read(selectedScoreProvider)?.id;

  /// Starts playback from the transport, arming a get-ready countdown (3…2…1…GO)
  /// when starting a **free-run** piece from the top, so the player has time to
  /// ready their hands before the notes start moving. In Wait Mode the cascade
  /// already freezes at the first onset (unlimited ready time), so no countdown
  /// is needed. Resuming mid-piece plays immediately. Plain [setPlaying] stays
  /// countdown-free (used internally and in tests).
  void startPlayback() {
    // A practice loop ALWAYS opens on the countdown, Wait Mode or not: the lap
    // is short and starts abruptly at the passage, so the 3…2…1 is what makes it
    // playable — and every later lap re-arms it the same way (see [advance]).
    if ((state.isSelectiveRun || !state.waitMode) &&
        _atStart(state) &&
        state.countdownMs == 0) {
      state = state.copyWith(countdownMs: kCountdownStartMs);
    }
    setPlaying(true);
  }

  // Every mode is scored and the scored note set is mode-independent, so
  // switching the render mode keeps the in-flight run (and its gauge/effects)
  // rather than discarding it.
  void setMode(RenderMode m) {
    if (m == state.mode) return;
    state = state.copyWith(mode: m);
    // Remembered per instrument family, so the next score of THAT family opens
    // the way this one was being read.
    ref
        .read(playerPreferencesProvider.notifier)
        .setLastMode(m, percussion: state.isPercussion);
    _track(UsageActions.playModeSwitch, variant: m.name);
  }

  // Re-arm the onset gate at the current playhead when toggling Wait Mode on,
  // and silence any in-flight score voices so none hang across the switch.
  void toggleWaitMode() {
    _silenceAll();
    state = state.copyWith(
      waitMode: !state.waitMode,
      gateSatisfied: const {},
      consumedHeld: const {},
      strokeAtMs: const {},
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
    _track(UsageActions.settingsChange, variant: UsageVariants.metronome);
  }

  /// Sets the playback speed (0.25×–2×) and remembers it across scores/restarts.
  void setSpeed(double s) {
    final clamped = s.clamp(0.25, 2.0);
    ref.read(playerPreferencesProvider.notifier).setSpeed(clamped);
    state = state.copyWith(speed: clamped);
    _track(UsageActions.settingsChange, variant: UsageVariants.tempo);
  }

  /// Sets the on-screen keyboard range and remembers it across scores/restarts.
  void setKeyboardRange(KeyboardRangeMode m) {
    ref.read(playerPreferencesProvider.notifier).setKeyboardRange(m);
    state = state.copyWith(keyboardRange: m);
  }

  /// Toggles the inverted-kit layout (change: add-drum-kit-view) and remembers
  /// it. Applies only to the PRESENTED lane order — the derived layout, the
  /// notation and the note interpretation are untouched by construction
  /// ([PlayerData.presentedDrumLanes] is the single application point).
  void setInvertedKit({required bool enabled}) {
    ref
        .read(playerPreferencesProvider.notifier)
        .setInvertedKit(enabled: enabled);
    // Mirroring the layout mirrors the surface indices, so any in-flight
    // struck flash would jump to the pad opposite the one actually struck
    // (change: add-drum-input-mapping). Dropping it is the honest answer.
    state = state.copyWith(
      invertedKit: enabled,
      struckSurfacesMs: const {},
      strokeAtMs: const {},
    );
  }

  /// Sets how much reading help is shown at a held onset, and remembers it
  /// across scores and restarts.
  void setReadingAid(NoteReadingAid aid) {
    if (aid == state.readingAid) return;
    ref.read(playerPreferencesProvider.notifier).setNoteReadingAid(aid);
    state = state.copyWith(readingAid: aid);
    _track(UsageActions.settingsChange, variant: UsageVariants.readingAid);
  }

  void setKeyboardVisible(bool visible) =>
      state = state.copyWith(keyboardVisible: visible);
  // Re-arm the onset gate so a hand switch can't leave the cascade frozen on an
  // onset that is now hidden (or pre-satisfied from the previous selection), and
  // silence voices so a now-hidden hand's notes don't keep sounding.
  void setSelectedHands(Hand hand) {
    ref.read(playerPreferencesProvider.notifier).setHands(hand);
    _track(UsageActions.settingsChange, variant: UsageVariants.hand);
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
      strokeAtMs: const {},
    );
    // The effective start depends on the selection, so recompute it for the new
    // hand(s) — a hand that enters later starts trimmed to its own first note.
    state = updated.copyWith(elapsedMs: updated.startMs);
    if (state.isPlaying) _maybeStartRun();
  }

  // --- Per-piece focus (change: add-practice-focus-controls) -------------

  /// Applies a new muted-piece set — the one path every focus action goes
  /// through, so mute, solo, unmute and clear all re-arm the session the same
  /// way [setSelectedHands] does for the keyboard.
  ///
  /// Same reasoning, at the new grain: changing what the session asks for
  /// changes what is scored, so any in-flight run is discarded and restarted
  /// from the top for the new selection; the onset gate is cleared so it cannot
  /// stay frozen on an onset that is now out of focus (or pre-satisfied by the
  /// previous selection); and sounding voices are released so a piece that just
  /// left the selection does not keep ringing.
  void _applyDrumFocus(Set<String> muted) {
    if (!state.isPercussion || setEquals(muted, state.mutedDrumPieces)) return;
    _silenceAll();
    _scorer.cancelRun();
    final updated = state.copyWith(
      mutedDrumPieces: muted,
      gateSatisfied: const {},
      consumedHeld: const {},
      strokeAtMs: const {},
    );
    // The effective start depends on the selection: a piece that enters later
    // starts the run trimmed to its own first note.
    state = updated.copyWith(elapsedMs: updated.startMs);
    if (state.isPlaying) _maybeStartRun();
  }

  /// Takes [pieceId] out of focus. Muting the last piece in focus restores the
  /// whole kit rather than leaving a session that asks for nothing (design D2).
  void muteDrumPiece(String pieceId) => _applyDrumFocus(
    mutedAfterMuting(state.mutedDrumPieces, pieceId, state.kitPieceIds),
  );

  /// Puts [pieceId] back in focus.
  void unmuteDrumPiece(String pieceId) =>
      _applyDrumFocus({...state.mutedDrumPieces}..remove(pieceId));

  /// Isolates [pieceId] — from the full kit it becomes the only piece asked
  /// for; from an existing selection it is **added** to it (design D2), which
  /// is what isolating a second piece means.
  void soloDrumPiece(String pieceId) => _applyDrumFocus(
    mutedAfterSoloing(state.mutedDrumPieces, pieceId, state.kitPieceIds),
  );

  /// Restores every piece of the kit.
  void clearDrumFocus() => _applyDrumFocus(const {});

  // --- Practice range (change: add-measure-range-practice) ---------------

  /// Sets the **active practice range** to measures `[start, end]`, normalized
  /// and clamped to the piece (`0 ≤ start ≤ end ≤ lastMeasure`; a single-measure
  /// range is allowed). A range narrower than the whole piece makes the run
  /// **selective** — unscored practice — so any in-flight scored run is
  /// discarded, the playhead moves to the range's first measure and held voices
  /// are silenced. A no-op on a piece with no measure table (the demo score).
  ///
  /// The single setter both range pickers go through (setup-sheet steppers and
  /// tap-on-score), per design D5.
  void setPracticeRange(int start, int end) {
    final range = normalizePracticeRange(
      start: start,
      end: end,
      measureCount: state.practiceMeasureCount,
    );
    if (range == null) return;
    // A new range is a new practice session: close the previous one first, so it
    // is counted once and the next range starts a fresh one.
    _endPracticeSession();
    _silenceAll();
    _scorer.cancelRun();
    // A selective run is deliberately **linear over the written measures** —
    // the player chose bars to drill, not a performance route — so on a piece
    // with repeats the timeline is re-derived without unrolling. (Identical
    // tables on a piece without repeats: no-op.)
    _applyTimeline(unroll: false);
    final updated = state.copyWith(
      practiceStartMeasure: range.start,
      practiceEndMeasure: range.end,
      countdownMs: 0,
      gateSatisfied: const {},
      consumedHeld: const {},
      strokeAtMs: const {},
    );
    state = updated.copyWith(elapsedMs: updated.startMs);
    _persistPracticeSettings();
  }

  /// Clears the practice range back to the whole piece (a **full run**, scored
  /// exactly as before) and returns the playhead to the piece's effective start.
  void clearPracticeRange() {
    // Returning to a full run ends the practice session.
    _endPracticeSession();
    _silenceAll();
    _scorer.cancelRun();
    // A full run follows the performance route again (repeats unrolled).
    _applyTimeline(unroll: true);
    final updated = state.copyWith(
      practiceStartMeasure: null,
      practiceEndMeasure: null,
      countdownMs: 0,
      gateSatisfied: const {},
      consumedHeld: const {},
      strokeAtMs: const {},
    );
    state = updated.copyWith(elapsedMs: updated.startMs);
    _persistPracticeSettings();
  }

  /// Re-derives the playback timeline of the loaded document — unrolled for a
  /// full run, written-linear for a selective practice run — leaving every
  /// other field untouched. A no-op for the demo score (no document) and
  /// whenever the tables are already in the requested shape.
  void _applyTimeline({required bool unroll}) {
    final document = _loadedDocument;
    if (document == null) return;
    final derived = notationToTimedNotes(document, unroll: unroll);
    if (state.measureStartMs.length == derived.measureStartMs.length) {
      return; // same shape — the piece has no repeats (or already linear)
    }
    state = state.copyWith(
      notes: derived.notes,
      rests: derived.rests,
      tieContinuations: derived.tieContinuations,
      songEndMs: derived.songEndMs,
      measureStartMs: derived.measureStartMs,
      measureKeyFifths: derived.measureKeyFifths,
      writtenMeasureOf: derived.writtenMeasureOf,
      measureDecors: derived.measureDecors,
    );
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
      strokeAtMs: const {},
    );
    if (state.isPlaying) _maybeStartRun();
  }

  /// Transport "restart": jump back to the top and start playing again, with the
  /// get-ready countdown (in free run). Used by the restart button and Retry.
  void restartFromTop() {
    restart();
    startPlayback();
  }

  /// Transport measure rewind (change: add-in-game-measure-selection): moves the
  /// playhead to the start of the measure containing it — or, when already at
  /// (or within [kRewindEpsilonMs] of) that start, the previous measure — so
  /// repeated taps stack back one measure at a time, clamped to the run's
  /// effective start. Held voices are silenced and the Wait-Mode latches
  /// cleared; any in-flight scored run is discarded, and none re-arms mid-piece
  /// (see [_maybeStartRun]), so a rewound run can never submit a score. The
  /// playing/paused state is preserved — no countdown replay, the point is a
  /// quick "again from the top of the bar". A no-op on a piece with no measure
  /// table (the demo score).
  void rewindOneMeasure() {
    final s = state;
    final target = rewindTargetMs(
      elapsedMs: s.elapsedMs,
      measureStartMs: s.measureStartMs,
      minMs: s.startMs,
    );
    if (target == null) return;
    _silenceAll();
    _scorer.cancelRun();
    state = s.copyWith(
      elapsedMs: target,
      countdownMs: 0,
      gateSatisfied: const {},
      consumedHeld: const {},
      strokeAtMs: const {},
    );
  }

  // --- Time advance (called by the screen's Ticker) ---------------------

  /// Advances the transport by [dtMs] ms of **real** (wall-clock) frame time.
  ///
  /// The playhead moves by `dtMs * speed` — the transport speed is applied here,
  /// not by the caller, so that the parts of the frame that are *not* musical
  /// time keep running at wall-clock rate. The pre-start countdown is one such
  /// part: a get-ready beat is a real-world beat, so 3…2…1…GO always lasts
  /// [kCountdownStartMs] whether the piece is played at 0.25× or 2×.
  ///
  /// Wait Mode gates on note *onsets*: the cascade freezes at each onset until
  /// every note starting there has been pressed (latched in [PlayerData.gateSatisfied]),
  /// then advances to the next onset — notes do not need to be held for their
  /// duration. A simple loop restarts at the end of the song.
  void advance(double dtMs) {
    var s = state;
    if (!s.isPlaying || s.notes.isEmpty) return;

    // Pre-start countdown: freeze the playhead (and audio/scoring) while the
    // 3…2…1…GO ticks down in REAL time — deliberately on [dtMs] and not on the
    // speed-scaled delta below, so slowing the tempo down does not stretch the
    // wait before the first note. Then playback proceeds normally.
    if (s.countdownMs > 0) {
      final remaining = s.countdownMs - dtMs;
      state = s.copyWith(countdownMs: remaining > 0 ? remaining : 0);
      return;
    }

    // Past the countdown, everything below is musical time: scale by the speed.
    final musicalDtMs = dtMs * s.speed;

    final onset = s.onsetPitchesAt(s.elapsedMs);

    // Drive the scorer's time-based bookkeeping (gate-open stamping in Wait Mode,
    // miss detection in free run, sustain finalization). Runs before the Wait
    // Mode blocked early-return below so the gate-open time is stamped even while
    // the cascade is frozen. A no-op when no run is active.
    //
    // The scorer receives BOTH clocks ([PlayerData.clocks]) and picks per call:
    // gate-open stamps, attacks and miss windows read the mode's judgment clock
    // — the heard clock in free run (change: add-audio-output-routing), the
    // emission clock in Wait Mode, where the frozen playhead has to match its
    // own onset to the millisecond — while each sustain is measured on the
    // clock that bound its note, so a hold straddling a Wait Mode toggle never
    // switches clocks. Identical to a bare playhead whenever the offset is 0.
    _scorer.tick(s.clocks, waitMode: s.waitMode);

    // Wait Mode tolerance: a key already held (and not already consumed by an
    // earlier onset) when the playhead reaches this onset counts as attacked —
    // a sustained/tied note need not be re-pressed. Consuming the hold keeps a
    // repeated pitch honest: it must be re-attacked to satisfy the next onset.
    //
    // **Keyboard only** (change: add-drum-scoring). The carve-out exists to
    // tolerate sustained and tied notes carried into the onset where they
    // first sound — a situation a kit cannot produce. A stroke is an attack;
    // "already holding it" is meaningless, so a percussion onset requires its
    // own strokes, struck while the gate is active, and an early stroke never
    // pre-satisfies it.
    if (s.waitMode && !s.isPercussion && onset.isNotEmpty) {
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
          _scorer.noteOn(p, s.clocks, waitMode: true);
        }
      }
    }

    // Wait Mode tolerance, the percussion half (this experiment): a stroke
    // played slightly EARLY still answers the onset it was aimed at, instead
    // of being dropped and demanded again. A kit is played by feel — the hand
    // leaves before the ear checks — and a gate that only accepts a stroke
    // arriving after the playhead does turns a groove into a typing test.
    //
    // The stroke is spent when credited, so one hit never validates two
    // onsets, and only strokes inside [kStrokeToleranceMs] count.
    if (s.waitMode && s.isPercussion && onset.isNotEmpty) {
      final earlyDue = <int>{};
      final strokes = {...s.strokeAtMs};
      for (final required in onset) {
        if (s.gateSatisfied.contains(required)) continue;
        final surface = s.struckSurfaceFor(required);
        if (surface == null) continue;
        final at = strokes[surface];
        // Measured on the playhead: the window is a musical one, and it is
        // the SAME number the surfaces light a stroke with — including its
        // real-time floor at speeds above normal, and its rejection of a
        // stroke left over from before a transport reset.
        if (at == null ||
            !strokeAnswersOnset(
              strokeMs: at,
              nowMs: s.elapsedMs,
              speed: s.speed,
            )) {
          continue;
        }
        earlyDue.add(required);
        strokes.remove(surface); // spent — never credited twice
      }
      if (earlyDue.isNotEmpty) {
        s = s.copyWith(
          gateSatisfied: {...s.gateSatisfied, ...earlyDue},
          strokeAtMs: strokes,
        );
        // Credit the scorer for the stroke it already saw as an extra note's
        // worth of nothing: it answered this onset, just before it opened.
        for (final p in earlyDue) {
          _scorer.noteOn(p, s.clocks, waitMode: true);
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

    var next = s.elapsedMs + musicalDtMs;

    // In Wait Mode, don't go past the next onset until it's validated.
    if (s.waitMode) {
      final ns = s.nextOnsetAfter(s.elapsedMs);
      if (ns != null && next > ns) next = ns;
    }

    // How far the playhead actually travelled this frame, BEFORE any wrap/stop
    // rewinds it — the span the practice-activity threshold is measured over.
    final travelledTo = next;

    var loop = false;
    var finishScoredRun = false;
    // Stop at the effective end (the range's last measure for a selective run,
    // else the last note's resolution) rather than the raw songEndMs, so trailing
    // rests / empty trailing measures are trimmed — the symmetric counterpart of
    // the trimmed start. endMs falls back to songEndMs when the selection has no
    // notes, so behaviour is unchanged there.
    final endMs = s.endMs;
    if (endMs > 0 && next >= endMs) {
      if (ref.read(performanceScorerProvider).active) {
        // A scored run ends the piece (produces the summary) instead of looping
        // — once the JUDGMENT clock reaches the end, not the emission clock. On
        // a delayed route the player is still hearing (and being judged on) the
        // last [outputOffsetMs] of the piece when the playhead reaches endMs,
        // so the run keeps judging through that drain tail; finalizing at endMs
        // would sweep the piece's unheard tail into `missed` (see
        // [PlayerData.scoredRunEndMs]). No-op in Wait Mode and at offset 0.
        final finishAt = s.scoredRunEndMs;
        if (next >= finishAt) {
          next = finishAt;
          finishScoredRun = true;
        }
      } else {
        // Simple loop — wrap to the effective start (the range's first measure
        // for a selective run), not 0.
        next = s.startMs;
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
      // A gated percussion onset was just played BY THE PLAYER — that is what
      // released it — so the schedule does not strike it again: the two
      // attacks land a frame apart and read as a flam (change:
      // add-drum-input-mapping, beta fix). Keyboard playback is untouched: a
      // held pitch re-sounding under the same finger is not a second attack in
      // the same way, and nothing in the reports asks for it.
      _applyScoreAudio(
        s,
        s.elapsedMs,
        next,
        justPlayed: s.waitMode && s.isPercussion ? onset : const {},
      );
    }

    // A selective run that has actually got through at least one onset is a real
    // practice session (see [_markPracticeProgress]) — measured over the span the
    // playhead travelled, so a frame that also wraps still counts. Once opened
    // the session stays open across laps, so looping never inflates the day's
    // practice count; the guard also makes this free after the first hit.
    if (!_practiceRecorded && s.isSelectiveRun && travelledTo > s.elapsedMs) {
      final crossed = scoreNoteEdges(
        visible: s.visibleNotes,
        from: s.elapsedMs,
        to: travelledTo,
        sounding: const <int>{},
      );
      if (crossed.starts.isNotEmpty) _markPracticeProgress();
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

    // A completed lap of a selective loop: consume one of the finite repetitions
    // and, when the tempo ramp is on, step the speed up for the next lap —
    // clamped to the transport maximum and never below the speed the run started
    // at (design D3). Both are no-ops for a full run or an infinite/unramped loop.
    // Leaving the satisfied onset (or looping) re-arms the gate for the next one.
    // A held pitch stays *consumed* across a normal onset advance (so it can't
    // walk through a repeat), but a loop wrap re-arms from scratch.
    // How much of the piece has now been heard (change: add-post-play-rating-
    // prompt). Monotonic within the score, so a seek back keeps it; on a loop wrap
    // `next` is the trimmed START, yet the playhead did just reach the end — credit
    // `endMs`, not the wrapped position.
    //
    // A **selective run** never credits it (change: add-measure-range-practice):
    // drilling bars 30–32 would otherwise mark every earlier note as "played" —
    // `playedNoteFraction` counts notes before the high-water mark, not notes
    // actually crossed — and a two-bar practice would look like a full
    // playthrough to the rating prompt. This extends the rule #199 already states
    // for `reachedEnd` ("a range-practice loop that ends early does not set it")
    // to the note-fraction term: practice is not a playthrough.
    final reached = s.isSelectiveRun
        ? s.furthestElapsedMs
        : (loop ? endMs : next);
    final leftOnset = onset.isNotEmpty && next != s.elapsedMs;
    state = s.copyWith(
      elapsedMs: next,
      furthestElapsedMs: reached > s.furthestElapsedMs
          ? reached
          : s.furthestElapsedMs,
      blocked: false,
      gateSatisfied: (leftOnset || loop) ? const {} : s.gateSatisfied,
      consumedHeld: loop ? const {} : s.consumedHeld,
      // A pending stroke survives leaving an onset — it may have been aimed
      // early at the NEXT one, which is the whole point of the tolerance — but
      // never a wrap: the playhead has just jumped backwards, and every stamp
      // on it now reads as impossibly early.
      strokeAtMs: loop ? const {} : s.strokeAtMs,
      beatCount: beatCount,
      lastBeatAccent: lastBeatAccent,
      // Every lap of a practice loop opens with its own 3…2…1…GO, exactly like
      // the first one: the player has just been thrown back to the start of the
      // passage and needs the same beat to get their hands in place.
      countdownMs: (loop && s.isSelectiveRun)
          ? kCountdownStartMs
          : s.countdownMs,
    );

    // End of a scored run: finalize the result (drives the summary modal) and
    // pause at the last position rather than looping.
    if (finishScoredRun) {
      _silenceAll();
      _scorer.finishRun(s.clocksAt(next), waitMode: s.waitMode);
      state = state.copyWith(isPlaying: false);
    }
  }
}
