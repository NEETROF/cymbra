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

import 'dart:convert';

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../painters/keyboard_range.dart';
import '../painters/notation_palette.dart';
import '../services/preferences_service.dart';
import 'player_data.dart' show Hand, NoteReadingAid, RenderMode;

export '../painters/notation_palette.dart' show NotationTheme;

part 'player_preferences.freezed.dart';
part 'player_preferences.g.dart';

/// User-selectable notation size, applied to both notation views: it scales the
/// engraved Partition (staff space + line re-wrap) and the horizontal Portée
/// (staff/glyph scale + look-ahead window) together.
enum ScoreSize { small, medium, large }

extension ScoreSizeFactor on ScoreSize {
  /// Scale factor applied to the notation. Medium is the historical 1.0.
  double get factor => switch (this) {
    ScoreSize.small => 0.85,
    ScoreSize.medium => 1.0,
    ScoreSize.large => 1.2,
  };
}

/// Effective score size for a device: the user's stored choice when there is
/// one, otherwise a form-factor default — **small on phones** (the short
/// landscape viewport earns its legibility back through the narrower
/// look-ahead window, not bigger glyphs), medium on tablets/desktops.
ScoreSize resolveScoreSize(ScoreSize? stored, {required bool isPhone}) =>
    stored ?? (isPhone ? ScoreSize.small : ScoreSize.medium);

/// The user's play settings, remembered across scores and app restarts. The
/// pre-play setup modal and the in-game settings drawer both read and write
/// these (via the `Player` notifier), and each score is seeded from them.
@freezed
abstract class PlayerPrefs with _$PlayerPrefs {
  const factory PlayerPrefs({
    @Default(Hand.both) Hand hands,
    @Default(1.0) double speed,
    @Default(false) bool metronome,

    /// On-screen keyboard range mode; defaults to auto-fit.
    @Default(KeyboardRangeMode.auto) KeyboardRangeMode keyboardRange,

    /// How much reading help to show at a held onset. Names the note by default:
    /// a beginner who does not know the notes is exactly the player who will not
    /// go looking for this in the settings. Turning it off is one tap away.
    @Default(NoteReadingAid.name) NoteReadingAid readingAid,

    /// Notation size for both notation views (Partition + Portée). Null means
    /// "not chosen yet": the effective default is resolved per form factor by
    /// [resolveScoreSize] (small on phones, medium elsewhere).
    ScoreSize? scoreSize,

    /// Notation rendering theme (dark surface or paper) for both notation
    /// views.
    @Default(NotationTheme.dark) NotationTheme notationTheme,

    /// Preferred MIDI input port name; null = auto (first real device).
    String? midiPort,

    /// Whether the connected MIDI instrument makes its own sound, so the app
    /// must not synthesize the notes it already played (change:
    /// add-audio-output-routing). Off by default — today's behaviour.
    @Default(false) bool instrumentSoundsItself,

    /// Whether the app stops sounding the **written score** (change:
    /// add-practice-focus-controls). The exact counterpart of
    /// [instrumentSoundsItself], on the other side of the exercise: that one
    /// silences the notes the player *plays*, this one the notes the app
    /// *asks for*. All four combinations are meaningful.
    ///
    /// Everything else about the session is untouched — the playhead, the
    /// drawing, the Wait Mode gate, the scorer and the metronome. Only the
    /// score's own audio stops. On a kit that is what makes an exercise
    /// audible at all: the click, the written part and the player's strokes
    /// otherwise mask each other on the same percussion timbres.
    @Default(false) bool scoreAudioMuted,

    /// Preferred audio **output device name**; null = follow the system default.
    /// A name is the only stable-ish handle the audio host offers, so a
    /// remembered device that is absent at startup simply falls back to the
    /// default (the *active* device is what the UI shows).
    String? audioOutput,

    /// Output latency compensation in milliseconds (change:
    /// add-audio-output-routing). Shifts the scoring reference and the visual
    /// playhead back by the delay a route adds, so a player following delayed
    /// audio is not judged late. 0 = today's behaviour.
    @Default(0) int outputOffsetMs,

    /// The render mode last chosen, remembered **per instrument family**: a
    /// player who reads drums on the stage and piano on the staff finds each
    /// where they left it, instead of one choice overwriting the other. Null
    /// means "never chosen" — the family's own default then stands (the
    /// cascade), which is what a first score must show.
    ///
    /// Stored, never inferred: nothing in a score says how its owner likes to
    /// read it.
    RenderMode? keyboardMode,
    RenderMode? percussionMode,

    /// The inverted-kit layout (change: add-drum-kit-view): reverses the
    /// percussion cascade's lane order and the pad strip together. Describes
    /// the KIT's setup, never the player's handedness — many left-handed
    /// drummers play a standard kit. Never inferred; off by default.
    @Default(false) bool invertedKit,
  }) = _PlayerPrefs;
}

/// Device-persisted [PlayerPrefs]. Seeded synchronously with defaults so the
/// first frame never blocks, then reconciled against the stored value; every
/// setter writes the whole record back (best-effort — a storage failure keeps
/// the in-memory value rather than throwing).
@Riverpod(keepAlive: true)
class PlayerPreferences extends _$PlayerPreferences {
  /// Preferences key under which the JSON-encoded [PlayerPrefs] lives.
  static const String prefsKey = 'player_prefs';

  // A completion *signal*, not state — putting it in [PlayerPrefs] would force
  // every copyWith site to thread it through for the benefit of one boot-time
  // awaiter, hence the lint exemption.
  // ignore: avoid_public_notifier_properties
  /// Completes when the stored record has been read and applied (or found
  /// absent/corrupt — defaults then stand). [build] seeds defaults synchronously
  /// so the first frame never blocks, which means an early reader sees `null`
  /// where storage holds a value; a caller acting on a persisted choice at
  /// startup (the audio-routing restore) must await this first.
  Future<void> get restored => _restored;
  Future<void> _restored = Future.value();

  @override
  PlayerPrefs build() {
    _restored = _restore();
    return const PlayerPrefs();
  }

  Future<void> _restore() async {
    String? raw;
    try {
      raw = await ref.read(preferencesServiceProvider).getString(prefsKey);
    } catch (_) {
      return; // storage unavailable → keep defaults
    }
    if (raw == null) return;
    final restored = _decode(raw);
    if (restored != null) state = restored;
  }

  void setHands(Hand hands) => _update(state.copyWith(hands: hands));
  void setSpeed(double speed) =>
      _update(state.copyWith(speed: speed.clamp(0.25, 2.0)));
  void setMetronome({required bool enabled}) =>
      _update(state.copyWith(metronome: enabled));
  void setKeyboardRange(KeyboardRangeMode mode) =>
      _update(state.copyWith(keyboardRange: mode));
  void setMidiPort(String? port) => _update(state.copyWith(midiPort: port));
  void setNoteReadingAid(NoteReadingAid aid) =>
      _update(state.copyWith(readingAid: aid));
  void setScoreSize(ScoreSize size) => _update(state.copyWith(scoreSize: size));
  void setNotationTheme(NotationTheme theme) =>
      _update(state.copyWith(notationTheme: theme));
  void setInstrumentSoundsItself({required bool enabled}) =>
      _update(state.copyWith(instrumentSoundsItself: enabled));
  void setScoreAudioMuted({required bool muted}) =>
      _update(state.copyWith(scoreAudioMuted: muted));
  void setAudioOutput(String? name) =>
      _update(state.copyWith(audioOutput: name));

  /// Sets the output latency compensation, clamped to a sane range: negative
  /// values would judge attacks *ahead* of the clock, and beyond a couple of
  /// seconds the reference no longer describes any real transport.
  void setOutputOffsetMs(int ms) =>
      _update(state.copyWith(outputOffsetMs: ms.clamp(0, 2000)));

  /// Remembers the render mode for the family that is playing. The stage is
  /// percussion-only, so it is never stored against the keyboard — a record
  /// that could hold it would hand a keyboard score a mode it cannot render.
  void setLastMode(RenderMode mode, {required bool percussion}) {
    if (percussion) {
      _update(state.copyWith(percussionMode: mode));
    } else if (mode != RenderMode.stage) {
      _update(state.copyWith(keyboardMode: mode));
    }
  }

  void setInvertedKit({required bool enabled}) =>
      _update(state.copyWith(invertedKit: enabled));

  void _update(PlayerPrefs next) {
    if (next == state) return;
    state = next;
    _persist(next);
  }

  Future<void> _persist(PlayerPrefs prefs) async {
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, _encode(prefs));
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }

  static String _encode(PlayerPrefs p) => jsonEncode({
    'hands': p.hands.name,
    'speed': p.speed,
    'metronome': p.metronome,
    'keyboardRange': p.keyboardRange.name,
    'readingAid': p.readingAid.name,
    'scoreSize': p.scoreSize?.name,
    'notationTheme': p.notationTheme.name,
    'midiPort': p.midiPort,
    'instrumentSoundsItself': p.instrumentSoundsItself,
    'scoreAudioMuted': p.scoreAudioMuted,
    'audioOutput': p.audioOutput,
    'outputOffsetMs': p.outputOffsetMs,
    'invertedKit': p.invertedKit,
    'keyboardMode': p.keyboardMode?.name,
    'percussionMode': p.percussionMode?.name,
  });

  static PlayerPrefs? _decode(String raw) {
    try {
      final m = jsonDecode(raw) as Map<String, dynamic>;
      final handName = m['hands'] as String?;
      // A record written by an earlier version carries no reading-aid level, and
      // a future one may carry a level this build does not know: either falls
      // back to the default rather than discarding the whole record.
      final aidName = m['readingAid'] as String?;
      return PlayerPrefs(
        hands: Hand.values.asNameMap()[handName] ?? Hand.both,
        speed: (m['speed'] as num?)?.toDouble().clamp(0.25, 2.0) ?? 1.0,
        metronome: m['metronome'] as bool? ?? false,
        keyboardRange:
            KeyboardRangeMode.values.asNameMap()[m['keyboardRange']
                as String?] ??
            KeyboardRangeMode.auto,
        readingAid:
            NoteReadingAid.values.asNameMap()[aidName] ?? NoteReadingAid.name,
        scoreSize: ScoreSize.values.asNameMap()[m['scoreSize'] as String?],
        notationTheme:
            NotationTheme.values.asNameMap()[m['notationTheme'] as String?] ??
            NotationTheme.dark,
        midiPort: m['midiPort'] as String?,
        instrumentSoundsItself: m['instrumentSoundsItself'] as bool? ?? false,
        scoreAudioMuted: m['scoreAudioMuted'] as bool? ?? false,
        audioOutput: m['audioOutput'] as String?,
        outputOffsetMs: ((m['outputOffsetMs'] as num?)?.toInt() ?? 0).clamp(
          0,
          2000,
        ),
        invertedKit: m['invertedKit'] as bool? ?? false,
        // An unknown mode name (a record written by a build that had one this
        // one does not) falls back to "never chosen" rather than discarding
        // the whole record.
        keyboardMode: RenderMode.values
            .asNameMap()[m['keyboardMode'] as String?],
        percussionMode: RenderMode.values
            .asNameMap()[m['percussionMode'] as String?],
      );
    } catch (_) {
      return null; // corrupt value → keep defaults
    }
  }
}
