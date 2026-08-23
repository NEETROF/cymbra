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
ReplayMark markFor(NoteJudgment j) {
  if (j.wrong) return ReplayMark.wrong;
  if (j.verdict == TimingVerdict.missed) return ReplayMark.missed;
  if (j.verdict == TimingVerdict.early || j.verdict == TimingVerdict.late) {
    return ReplayMark.mistimed;
  }
  if (j.sustainRatio < 0.5) return ReplayMark.shortSustain;
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

  /// Scrolls the bottom mistake bar so it tracks the staff.
  final ScrollController _mistakeScroll = ScrollController();

  /// Index of the mistake the bar is currently centred on, so the follow only
  /// animates when the playhead actually moves to another mistake (not on
  /// every tick).
  int _followed = -1;

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
  void _setElapsed(double ms, {bool? playing}) {
    setState(() {
      _elapsed = ms;
      if (playing != null) _playing = playing;
    });
    _followMistakes();
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

  /// Scrolls the mistake bar so the mistake under the playhead is centred,
  /// mirroring the staff scrolling underneath it. Only fires when the playhead
  /// crosses into another mistake, so a manual scroll of the bar survives until
  /// the run actually reaches the next one.
  void _followMistakes() {
    if (_mistakes.isEmpty) return;
    final index = _nearestMistake();
    if (index == _followed) return;
    _followed = index;
    if (!_mistakeScroll.hasClients) return;
    final position = _mistakeScroll.position;
    final target =
        (_kChipPadding +
                index * _kChipStride +
                _kChipWidth / 2 -
                position.viewportDimension / 2)
            .clamp(position.minScrollExtent, position.maxScrollExtent);
    _mistakeScroll.animateTo(
      target,
      duration: const Duration(milliseconds: 240),
      curve: Curves.easeOut,
    );
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
    _setElapsed(ms.clamp(0, _score.songEndMs));
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
          // The chip under the playhead is ringed thicker and lifted, so the
          // bar reads as "here is where you are" while the staff scrolls.
          final active = i == current;
          return InkWell(
            onTap: () => _seek(j.startMs.toDouble()),
            borderRadius: BorderRadius.circular(10),
            child: Container(
              width: _kChipWidth,
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
              decoration: BoxDecoration(
                color: active
                    ? CymbraColors.surfaceContainerHigh
                    : CymbraColors.surfaceContainerLow,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: color, width: active ? 2 : 1),
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
