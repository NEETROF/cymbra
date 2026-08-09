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
import 'dart:ui' show VoidCallback;

import '../services/audio_service.dart';

/// The one way a lesson exercise makes sound (change: add-notation-courses,
/// schema v2). Every touch in a lesson is audible — a tapped key, a placed
/// note, a browsed answer chip — so exercises share this small helper instead
/// of each re-wiring [AudioService].
///
/// Follows the sanctioned audition-widget pattern (see `mistake_replay.dart`):
/// the widget captures the service in `initState`, and [dispose] cancels every
/// scheduled timer and releases every voice, so nothing can ring past the
/// widget. All playback is fire-and-forget over the non-throwing service — if
/// audio is unavailable the lesson still works, silently.
class LessonSounder {
  LessonSounder(this._audio);

  final AudioService _audio;
  final Set<Timer> _timers = {};
  final Set<int> _sounding = {};
  bool _disposed = false;

  /// Sounds [pitch] briefly — the feedback for any tap.
  void tap(int pitch, {int durationMs = 300}) {
    if (_disposed) return;
    _on(pitch);
    _after(durationMs, () => _off(pitch));
  }

  /// Plays [pitches] as a sequence (one every [gapMs], each held [noteMs]) or —
  /// when [harmonic] — together as a block chord. [onDone] fires once the last
  /// voice has been released, unless the sounder is stopped first.
  void playSequence(
    List<int> pitches, {
    int noteMs = 550,
    int gapMs = 700,
    bool harmonic = false,
    VoidCallback? onDone,
  }) {
    if (_disposed || pitches.isEmpty) return;
    var end = 0;
    for (var i = 0; i < pitches.length; i++) {
      final p = pitches[i];
      final start = harmonic ? 0 : i * gapMs;
      _after(start, () => _on(p));
      _after(start + noteMs, () => _off(p));
      if (start + noteMs > end) end = start + noteMs;
    }
    if (onDone != null) _after(end + 40, onDone);
  }

  /// A short rising arpeggio — the success flourish of a completed exercise.
  void chime({int root = 84}) {
    if (_disposed) return;
    for (var i = 0; i < 3; i++) {
      final p = root + const [0, 4, 7][i];
      _after(i * 70, () => _on(p, velocity: 70));
      _after(i * 70 + 220, () => _off(p));
    }
  }

  /// Cancels everything scheduled and silences every voice this sounder
  /// started (also called by [dispose]).
  void stopAll() {
    for (final t in _timers) {
      t.cancel();
    }
    _timers.clear();
    for (final p in _sounding.toList()) {
      _audio.noteOff(p);
    }
    _sounding.clear();
  }

  void dispose() {
    stopAll();
    _disposed = true;
  }

  void _on(int pitch, {int velocity = AudioService.defaultVelocity}) {
    _sounding.add(pitch);
    _audio.noteOn(pitch, velocity: velocity);
  }

  void _off(int pitch) {
    _sounding.remove(pitch);
    _audio.noteOff(pitch);
  }

  void _after(int ms, VoidCallback fn) {
    late final Timer t;
    t = Timer(Duration(milliseconds: ms), () {
      _timers.remove(t);
      if (!_disposed) fn();
    });
    _timers.add(t);
  }
}
