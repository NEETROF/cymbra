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

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../painters/staff_painter.dart';
import '../services/audio_service.dart';
import '../state/performance_scoring_core.dart';
import '../state/player_data.dart';
import '../state/session_summary.dart';
import '../theme/cymbra_theme.dart';

/// How a judged note is classified for the replay highlight.
enum ReplayMark { correct, mistimed, shortSustain, missed, wrong }

/// Classifies a [NoteJudgment]: correct notes are left un-marked, everything
/// else is highlighted by the kind of mistake.
ReplayMark markFor(NoteJudgment j) {
  if (j.wrong) return ReplayMark.wrong;
  if (j.verdict == TimingVerdict.missed) return ReplayMark.missed;
  if (j.verdict == TimingVerdict.early || j.verdict == TimingVerdict.late) {
    return ReplayMark.mistimed;
  }
  if (j.sustainRatio < 0.5) return ReplayMark.shortSustain;
  return ReplayMark.correct;
}

/// The mistake colours, matching the summary/replay legend.
Color colorForMark(ReplayMark m) => switch (m) {
  ReplayMark.correct => CymbraColors.tertiary,
  ReplayMark.mistimed => CymbraColors.handLeft,
  ReplayMark.shortSustain => CymbraColors.primary,
  ReplayMark.missed => CymbraColors.error,
  ReplayMark.wrong => CymbraColors.error,
};

/// The score context the replay needs to render the real horizontal staff,
/// captured from the player when the run finished.
class ReplayScore {
  final List<TimedNote> notes;
  final int bpm;
  final double songEndMs;
  final int keyFifths;
  final List<int> measureKeyFifths;
  final int beats;
  final int beatType;
  final List<int> measureStartMs;

  const ReplayScore({
    required this.notes,
    required this.bpm,
    required this.songEndMs,
    required this.keyFifths,
    this.measureKeyFifths = const [],
    required this.beats,
    required this.beatType,
    required this.measureStartMs,
  });

  /// Builds the replay context from the current player state (same piece).
  factory ReplayScore.fromPlayer(PlayerData d) => ReplayScore(
    notes: d.visibleNotes,
    bpm: d.bpm,
    songEndMs: d.songEndMs,
    keyFifths: d.keyFifths,
    measureKeyFifths: d.measureKeyFifths,
    beats: d.beats,
    beatType: d.beatType,
    measureStartMs: d.measureStartMs,
  );

  /// 1-based measure number containing [startMs].
  int measureOf(int startMs) {
    var m = 1;
    for (var i = 0; i < measureStartMs.length; i++) {
      if (startMs >= measureStartMs[i]) m = i + 1;
    }
    return m;
  }
}

/// Opens the mistake replay for [result] over the real horizontal score
/// ([score]). The player watches their run scrub across the actual staff with
/// mistakes ringed in place, can play/pause/seek with synced audio, and tap a
/// mistake in the list to jump straight to it.
Future<void> showMistakeReplay(
  BuildContext context,
  ReplayScore score,
  SessionResult result,
) => showDialog<void>(
  context: context,
  builder: (context) => _ReplayDialog(score: score, result: result),
);

class _ReplayDialog extends ConsumerStatefulWidget {
  const _ReplayDialog({required this.score, required this.result});

  final ReplayScore score;
  final SessionResult result;

  @override
  ConsumerState<_ReplayDialog> createState() => _ReplayDialogState();
}

class _ReplayDialogState extends ConsumerState<_ReplayDialog>
    with SingleTickerProviderStateMixin {
  late final Ticker _ticker;
  Duration _lastTick = Duration.zero;
  final Set<int> _sounding = {};

  double _elapsed = 0;
  bool _playing = false;

  ReplayScore get _score => widget.score;

  /// Note-index → mistake colour for the notes shown on the staff.
  late final Map<int, Color> _mistakeColors = {
    for (final j in widget.result.notes)
      if (j.noteIndex >= 0 && markFor(j) != ReplayMark.correct)
        j.noteIndex: colorForMark(markFor(j)),
  };

  /// The judged mistakes, in time order, for the tappable list.
  late final List<NoteJudgment> _mistakes =
      widget.result.notes
          .where((j) => markFor(j) != ReplayMark.correct)
          .toList()
        ..sort((a, b) => a.startMs.compareTo(b.startMs));

  /// The notes the player actually played, reconstructed from the judgments —
  /// this is what the replay *sounds* (so the player hears their own
  /// performance, not the score). Each hit is placed at the time it was played
  /// (`startMs + timing offset` in free run, the onset in Wait Mode) for its
  /// held duration; wrong notes sound at their press time; missed notes are
  /// silent. The score is still what's *shown* on the staff.
  late final List<TimedNote> _playedNotes = _buildPlayedNotes();

  List<TimedNote> _buildPlayedNotes() {
    final notes = <TimedNote>[];
    for (final j in widget.result.notes) {
      if (j.wrong) {
        notes.add(
          TimedNote(pitch: j.pitch, startMs: j.startMs, durationMs: 200),
        );
        continue;
      }
      if (j.verdict == TimingVerdict.missed) continue; // not played → silent
      final start = j.waitMode
          ? j.startMs
          : (j.startMs + (j.timingOffsetMs ?? 0).round());
      final intended = (j.noteIndex >= 0 && j.noteIndex < _score.notes.length)
          ? _score.notes[j.noteIndex].durationMs
          : 300;
      final held = (intended * j.sustainRatio).round().clamp(80, intended);
      notes.add(
        TimedNote(
          pitch: j.pitch,
          startMs: start < 0 ? 0 : start,
          durationMs: held,
        ),
      );
    }
    notes.sort((a, b) => a.startMs.compareTo(b.startMs));
    return notes;
  }

  late final AudioService _audio;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick);
    // Capture the audio service so [dispose] never touches `ref` after the
    // provider scope has been torn down.
    _audio = ref.read(audioServiceProvider);
  }

  @override
  void dispose() {
    _ticker.dispose();
    _audio.allNotesOff();
    super.dispose();
  }

  void _onTick(Duration elapsed) {
    final dt = (elapsed - _lastTick).inMicroseconds / 1000.0;
    _lastTick = elapsed;
    if (!_playing || dt <= 0 || dt > 100) return;
    final next = _elapsed + dt;
    _applyAudio(_elapsed, next);
    if (next >= _score.songEndMs) {
      _stopAudio();
      setState(() {
        _elapsed = _score.songEndMs;
        _playing = false;
      });
      _ticker.stop();
    } else {
      setState(() => _elapsed = next);
    }
  }

  void _applyAudio(double from, double to) {
    // Sound the *player's* notes, not the score, so they hear their performance.
    final edges = scoreNoteEdges(
      visible: _playedNotes,
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

  void _stopAudio() {
    _audio.allNotesOff();
    _sounding.clear();
  }

  void _togglePlay() {
    if (_playing) {
      _stopAudio();
      setState(() => _playing = false);
      _ticker.stop();
      return;
    }
    if (_elapsed >= _score.songEndMs) _elapsed = 0;
    _lastTick = Duration.zero;
    setState(() => _playing = true);
    _ticker.start();
  }

  void _seek(double ms) {
    _stopAudio();
    setState(() => _elapsed = ms.clamp(0, _score.songEndMs));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Dialog.fullscreen(
      backgroundColor: CymbraColors.background,
      child: Column(
        children: [
          _header(context, l10n),
          Expanded(
            child: CustomPaint(
              size: Size.infinite,
              painter: StaffPainter(
                notes: _score.notes,
                elapsedMs: _elapsed,
                activeNotes: _sounding,
                bpm: _score.bpm,
                songEndMs: _score.songEndMs,
                keyFifths: _score.keyFifths,
                measureKeyFifths: _score.measureKeyFifths,
                beats: _score.beats,
                beatType: _score.beatType,
                measureStartMs: _score.measureStartMs,
                mistakeColors: _mistakeColors,
              ),
            ),
          ),
          _transport(),
          _mistakeList(l10n),
        ],
      ),
    );
  }

  Widget _header(BuildContext context, AppLocalizations l10n) => Padding(
    padding: const EdgeInsets.fromLTRB(16, 12, 8, 4),
    child: Row(
      children: [
        Text(
          l10n.replayTitle,
          style: const TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w500,
            color: CymbraColors.onSurface,
          ),
        ),
        const Spacer(),
        IconButton(
          icon: const Icon(Icons.close, color: CymbraColors.onSurface),
          onPressed: () => Navigator.of(context).pop(),
        ),
      ],
    ),
  );

  Widget _transport() => Padding(
    padding: const EdgeInsets.symmetric(horizontal: 12),
    child: Row(
      children: [
        IconButton(
          iconSize: 32,
          color: CymbraColors.secondary,
          icon: Icon(_playing ? Icons.pause_circle : Icons.play_circle),
          onPressed: _togglePlay,
        ),
        Expanded(
          child: Slider(
            value: _elapsed.clamp(0, _score.songEndMs),
            max: _score.songEndMs <= 0 ? 1 : _score.songEndMs,
            onChangeStart: (_) {
              if (_playing) {
                setState(() => _playing = false);
                _ticker.stop();
              }
            },
            onChanged: _seek,
          ),
        ),
      ],
    ),
  );

  Widget _mistakeList(AppLocalizations l10n) {
    if (_mistakes.isEmpty) {
      return Padding(
        padding: const EdgeInsets.all(16),
        child: Text(
          l10n.replayNoMistakes,
          style: const TextStyle(color: CymbraColors.tertiary),
        ),
      );
    }
    return SizedBox(
      height: 96,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        itemCount: _mistakes.length,
        separatorBuilder: (_, _) => const SizedBox(width: 8),
        itemBuilder: (context, i) {
          final j = _mistakes[i];
          final mark = markFor(j);
          final color = colorForMark(mark);
          return InkWell(
            onTap: () => _seek(j.startMs.toDouble()),
            borderRadius: BorderRadius.circular(10),
            child: Container(
              width: 150,
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
              decoration: BoxDecoration(
                color: CymbraColors.surfaceContainerLow,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: color, width: 1),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.replayMeasure(_score.measureOf(j.startMs)),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontSize: 12,
                      color: CymbraColors.onSurfaceVariant,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    _markLabel(l10n, j, mark),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.w500,
                      color: color,
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  /// Label for a mistake chip. A mistimed note spells out the direction and
  /// offset ("Late 85 ms" / "Early 40 ms", or the reaction time in Wait Mode)
  /// so the player sees whether they rushed or dragged, and by how much.
  String _markLabel(AppLocalizations l10n, NoteJudgment j, ReplayMark m) =>
      switch (m) {
        ReplayMark.missed => l10n.replayMissed,
        ReplayMark.mistimed => _timingDetail(l10n, j) ?? l10n.replayMistimed,
        ReplayMark.shortSustain => l10n.replayShortSustain,
        ReplayMark.wrong => l10n.replayWrong,
        ReplayMark.correct => '',
      };

  /// The signed timing detail for a note: reaction time in Wait Mode, else the
  /// early/late offset. Null when no timing was recorded.
  String? _timingDetail(AppLocalizations l10n, NoteJudgment j) {
    if (j.reactionMs != null) return l10n.replayReaction(j.reactionMs!.round());
    final off = j.timingOffsetMs;
    if (off == null) return null;
    final ms = off.abs().round();
    return off < 0 ? l10n.replayEarly(ms) : l10n.replayLate(ms);
  }
}
