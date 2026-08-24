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
import '../state/note_density_core.dart';
import '../state/performance_scoring_core.dart';
import '../state/player_data.dart';
import '../state/session_summary.dart';
import '../theme/cymbra_theme.dart';

/// How a judged note is classified for the replay highlight.
enum ReplayMark { correct, mistimed, shortSustain, missed, wrong }

/// Classifies a [NoteJudgment]: correct notes are left un-marked, everything
/// else is highlighted by the kind of mistake.
///
/// [ReplayMark.shortSustain] is a **keyboard-only** category: a percussion
/// stroke carries no sustain ratio at all (change: add-drum-scoring), so no
/// stroke is ever flagged for it — the absence decides, not a threshold on a
/// zero.
ReplayMark markFor(NoteJudgment j) {
  if (j.wrong) return ReplayMark.wrong;
  if (j.verdict == TimingVerdict.missed) return ReplayMark.missed;
  if (j.verdict == TimingVerdict.early || j.verdict == TimingVerdict.late) {
    return ReplayMark.mistimed;
  }
  final sustain = j.sustainRatio;
  if (sustain != null && sustain < 0.5) return ReplayMark.shortSustain;
  return ReplayMark.correct;
}

/// Geometry of a mistake chip in the bottom bar. The list is a fixed-extent
/// row, so the replay can compute where a chip sits without measuring it.
const double _kChipWidth = 150;
const double _kChipGap = 8;
const double _kChipPadding = 12;
const double _kChipStride = _kChipWidth + _kChipGap;

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

  /// Render-only tie continuations of the piece (`PlayerData.tieContinuations`
  /// hand-filtered), so the replay staff engraves tied notation the same way
  /// the live one does.
  final List<TimedNote> tieContinuations;
  final int bpm;
  final double songEndMs;
  final int keyFifths;
  final List<int> measureKeyFifths;
  final int beats;
  final int beatType;
  final List<int> measureStartMs;

  /// Whether the replayed piece is percussion (change: add-drum-scoring). The
  /// staff engraves itself from the notes' own percussion clef, so this decides
  /// only what the transport **sounds**: a run's own strokes must come out of
  /// the drum channel, never the piano preset.
  final bool isPercussion;

  const ReplayScore({
    required this.notes,
    this.tieContinuations = const [],
    required this.bpm,
    required this.songEndMs,
    required this.keyFifths,
    this.measureKeyFifths = const [],
    required this.beats,
    required this.beatType,
    required this.measureStartMs,
    this.isPercussion = false,
  });

  /// Builds the replay context from the current player state (same piece).
  factory ReplayScore.fromPlayer(PlayerData d) => ReplayScore(
    notes: d.visibleNotes,
    tieContinuations: d.visibleTieContinuations,
    bpm: d.bpm,
    songEndMs: d.songEndMs,
    keyFifths: d.keyFifths,
    measureKeyFifths: d.measureKeyFifths,
    beats: d.beats,
    beatType: d.beatType,
    measureStartMs: d.measureStartMs,
    isPercussion: d.isPercussion,
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

  /// Scrolls the bottom mistake bar so it tracks the staff.
  final ScrollController _mistakeScroll = ScrollController();

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
      // A percussion stroke carries no sustain ratio: it is a one-shot, so it
      // is replayed for the note's own written length and left to its natural
      // end by the drum channel.
      final held = (intended * (j.sustainRatio ?? 1)).round().clamp(
        80,
        intended,
      );
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
    _mistakeScroll.dispose();
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
      _setElapsed(_score.songEndMs, playing: false);
      _ticker.stop();
    } else {
      _setElapsed(next);
    }
  }

  /// Moves the playhead and keeps the bottom mistake bar aligned with it.
  /// [animate] is for a discrete jump (tapping a mistake chip); the ticker and
  /// the slider follow continuously instead, so the bar glides rather than
  /// snapping from one chip to the next.
  void _setElapsed(double ms, {bool? playing, bool animate = false}) {
    setState(() {
      _elapsed = ms;
      if (playing != null) _playing = playing;
    });
    _followMistakes(animate: animate);
  }

  /// Index of the mistake sitting closest to the playhead — the one the staff
  /// is showing right now. Ties (and the stretch before the first mistake)
  /// resolve to the earlier mistake, so the bar never jumps ahead of the staff.
  int _nearestMistake() {
    var best = 0;
    var bestGap = double.infinity;
    for (var i = 0; i < _mistakes.length; i++) {
      final gap = (_mistakes[i].startMs - _elapsed).abs();
      if (gap < bestGap) {
        bestGap = gap;
        best = i;
      }
    }
    return best;
  }

  /// Centre of the chip for mistake [i] in scroll coordinates.
  double _chipCenter(int i) =>
      _kChipPadding + i * _kChipStride + _kChipWidth / 2;

  /// Where the bar should sit for the current playhead, in scroll coordinates:
  /// the chip centres interpolated *between* consecutive mistakes, so the bar
  /// creeps forward with the run at the same pace the staff scrolls instead of
  /// jumping a whole chip at a time. Before the first mistake and after the
  /// last one it simply rests on that chip.
  double _followCenter() {
    if (_elapsed <= _mistakes.first.startMs) return _chipCenter(0);
    final last = _mistakes.length - 1;
    if (_elapsed >= _mistakes[last].startMs) return _chipCenter(last);
    for (var i = 0; i < last; i++) {
      final from = _mistakes[i].startMs.toDouble();
      final to = _mistakes[i + 1].startMs.toDouble();
      if (_elapsed < to) {
        final span = to - from;
        final t = span <= 0 ? 0.0 : (_elapsed - from) / span;
        return _chipCenter(i) + (_chipCenter(i + 1) - _chipCenter(i)) * t;
      }
    }
    return _chipCenter(last);
  }

  /// Scrolls the mistake bar so the run's position is centred, mirroring the
  /// staff scrolling underneath it. Driven every tick, which is what makes the
  /// motion continuous; only an explicit jump (tapping a chip) animates.
  void _followMistakes({bool animate = false}) {
    if (_mistakes.isEmpty || !_mistakeScroll.hasClients) return;
    final position = _mistakeScroll.position;
    final target = (_followCenter() - position.viewportDimension / 2).clamp(
      position.minScrollExtent,
      position.maxScrollExtent,
    );
    if ((target - position.pixels).abs() < 0.5) return;
    if (animate) {
      _mistakeScroll.animateTo(
        target,
        duration: const Duration(milliseconds: 320),
        curve: Curves.easeOutCubic,
      );
      return;
    }
    // A drag of the bar itself owns the scroll while it lasts — don't fight it.
    if (position.isScrollingNotifier.value) return;
    _mistakeScroll.jumpTo(target);
  }

  void _applyAudio(double from, double to) {
    // Sound the *player's* notes, not the score, so they hear their performance.
    // A percussion run goes through the drum entry points (change:
    // add-drum-audio-channel), never the melodic pair: a snare number sent to
    // the piano preset comes out as a piano note.
    final edges = scoreNoteEdges(
      visible: _playedNotes,
      from: from,
      to: to,
      sounding: _sounding,
    );
    for (final p in edges.stops) {
      if (_score.isPercussion) {
        _audio.drumOff(p);
      } else {
        _audio.noteOff(p);
      }
      _sounding.remove(p);
    }
    for (final p in edges.starts) {
      if (_score.isPercussion) {
        _audio.drumOn(p);
      } else {
        _audio.noteOn(p);
      }
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

  void _seek(double ms, {bool animate = false}) {
    _stopAudio();
    _setElapsed(ms.clamp(0, _score.songEndMs), animate: animate);
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
                tieContinuations: _score.tieContinuations,
                elapsedMs: _elapsed,
                activeNotes: _sounding,
                bpm: _score.bpm,
                songEndMs: _score.songEndMs,
                keyFifths: _score.keyFifths,
                measureKeyFifths: _score.measureKeyFifths,
                beats: _score.beats,
                beatType: _score.beatType,
                measureStartMs: _score.measureStartMs,
                // Same readability cap as the live Portée: the replay scrolls
                // the same score at the same window, so a dense piece would be
                // just as cramped here.
                onsetGapMs: cachedOnsetGapMs(_score.notes),
                measureMs: medianMeasureMs(
                  _score.measureStartMs,
                  songEndMs: _score.songEndMs,
                ),
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
    final current = _nearestMistake();
    return SizedBox(
      height: 96,
      child: ListView.separated(
        key: const ValueKey('replay-mistake-list'),
        controller: _mistakeScroll,
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(
          horizontal: _kChipPadding,
          vertical: 8,
        ),
        itemCount: _mistakes.length,
        separatorBuilder: (_, _) => const SizedBox(width: _kChipGap),
        itemBuilder: (context, i) {
          final j = _mistakes[i];
          final mark = markFor(j);
          final color = colorForMark(mark);
          // The chip under the playhead is ringed thick, tinted in its own
          // mistake colour and haloed, so the bar reads at a glance as "here is
          // where you are" while the staff scrolls. The others stay muted.
          final active = i == current;
          return InkWell(
            onTap: () => _seek(j.startMs.toDouble(), animate: true),
            borderRadius: BorderRadius.circular(10),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 220),
              curve: Curves.easeOut,
              width: _kChipWidth,
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
              decoration: BoxDecoration(
                color: active
                    ? Color.alphaBlend(
                        color.withValues(alpha: 0.22),
                        CymbraColors.surfaceContainerHigh,
                      )
                    : CymbraColors.surfaceContainerLow,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(
                  color: active ? color : color.withValues(alpha: 0.45),
                  width: active ? 3 : 1,
                ),
                boxShadow: active
                    ? [
                        BoxShadow(
                          color: color.withValues(alpha: 0.45),
                          blurRadius: 12,
                          spreadRadius: 1,
                        ),
                      ]
                    : null,
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
