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
import 'drum_calibration.dart';
import 'drum_input_mapping_notifier.dart';
import 'drum_kit.dart';
import 'midi_status_notifier.dart';
import 'player_notifier.dart';

part 'drum_calibration_notifier.g.dart';

/// Drives a calibration pass from the live MIDI stream (change:
/// add-drum-input-calibration).
///
/// A **third reader** of the event stream, alongside the player and the
/// monitor — possible because the production adapter fans one engine
/// subscription out to every listener. It observes only: the player keeps
/// sounding what the instrument sends throughout the pass, which is the point
/// of the "strokes stay audible" requirement. A player who hears nothing while
/// calibrating cannot tell a mis-mapped pad from a disconnected kit.
///
/// The state machine is pure ([CalibrationState]); this owns the subscription,
/// the timestamps, and the one write to storage — **on completion only**, so
/// abandoning leaves the previous mapping exactly as it was.
///
/// Auto-dispose: leaving the screen ends the pass.
@riverpod
class DrumCalibration extends _$DrumCalibration {
  @override
  CalibrationState build() {
    // Keeps the player alive for the length of the pass, so strokes stay
    // audible: the player is the one thing that sounds what the instrument
    // sends, and a player who hears nothing while calibrating cannot tell a
    // mis-mapped pad from a disconnected kit.
    //
    // `listen` with an empty callback rather than `watch`: the dependency is
    // what holds the player open, and watching it would rebuild this notifier —
    // discarding the pass — on every frame of playback.
    ref.listen(playerProvider, (_, _) {});
    final sub = ref
        .read(midiServiceProvider)
        .events()
        .listen(_onEvent, onError: (Object _) {});
    ref.onDispose(sub.cancel);
    return CalibrationState(pieces: _targets());
  }

  /// What this pass asks for: the loaded score's own kit (design D10).
  ///
  /// `read`, not `watch`, for the reason the keep-alive above is a `listen`:
  /// watching would rebuild this notifier — discarding the pass — on every frame
  /// of playback. Re-read at [start] rather than only at build, so a pass begun
  /// on a score that finished loading after this surface opened still asks for
  /// that score's pieces.
  ///
  /// Falls back to the standard kit when there is no percussion score to read —
  /// nothing loaded yet, or the surface reached with a keyboard score, where the
  /// numbers are pitches and would name pieces the player never struck.
  List<String> _targets() {
    final data = ref.read(playerProvider);
    final targets = data.calibrationTargets;
    return targets.isEmpty ? kCalibrationPieceOrder : targets;
  }

  /// The last timestamp the engine stamped, so a step arms against the events
  /// that have actually arrived rather than against a wall clock the stream
  /// knows nothing about. Every transition re-arms at this value, which is what
  /// makes "the next stroke" mean the next one *the player plays*.
  int _lastSeenMs = 0;

  void _onEvent(MidiEvent event) {
    // Note-ons only: a kit sends its release within milliseconds of the attack,
    // and recording that as the answer would learn the pad twice.
    if (event.kind != MidiEventKind.noteOn) return;
    final at = event.timestampMs.toInt();
    if (at > _lastSeenMs) _lastSeenMs = at;
    // The RAW number, deliberately: the pass exists to learn what this device
    // sends, so applying the mapping it is building would be circular.
    _apply(state.afterStroke(event.pitch, atMs: at));
  }

  void skip() => _apply(state.skip(atMs: _lastSeenMs));
  void back() => _apply(state.back(atMs: _lastSeenMs));
  void strikeAgain() => _apply(state.strikeAgain(atMs: _lastSeenMs));
  void reassign() => _apply(state.reassign(atMs: _lastSeenMs));

  /// Leaves the pass. Nothing is written — the stored mapping stands.
  void abandon() => state = state.abandon();

  /// Ends the pass here, keeping what it learned. Goes through [_apply], so it
  /// stores exactly like reaching the last step does.
  void finish() => _apply(state.finish());

  /// Begins a pass (or begins one again), discarding anything an earlier pass
  /// in this session had learned but not stored. Armed against the events seen
  /// so far, so the stroke that opened the screen cannot answer the first step,
  /// and asking for the score that is loaded **now**.
  void start() =>
      state = CalibrationState(pieces: _targets()).start(atMs: _lastSeenMs);

  void _apply(CalibrationState next) {
    if (identical(next, state)) return;
    state = next;
    // The single write, and only here: a pass that ends any other way must
    // leave the device exactly as it found it (design D4).
    if (next.outcome != CalibrationOutcome.completed) return;
    final port = ref.read(midiStatusProvider).connected;
    if (port == null) return;
    ref.read(drumInputMappingStoreProvider.notifier).save(port, next.mapping);
  }
}
