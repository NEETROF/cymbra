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
import 'player_data.dart' show Hand, NoteReadingAid;

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

  @override
  PlayerPrefs build() {
    _restore();
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
      );
    } catch (_) {
      return null; // corrupt value → keep defaults
    }
  }
}
