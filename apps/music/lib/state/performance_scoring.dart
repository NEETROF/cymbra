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

import '../services/clock_service.dart';
import 'performance_scoring_core.dart';
import 'player_data.dart';
import 'session_summary.dart';

part 'performance_scoring.g.dart';

/// A transient per-note hit effect for the Guitar-Hero–style feedback layer.
/// Carries just enough for the effects painter to place and fade a spark; the UI
/// discards it after a short time (nothing here persists).
class HitEffect {
  final int pitch;
  final TimingVerdict verdict;
  final bool wrong;

  /// Wall-clock ms the effect was created (for fade timing in the UI).
  final int atMs;

  const HitEffect({
    required this.pitch,
    required this.verdict,
    required this.wrong,
    required this.atMs,
  });
}

/// Immutable live scoring state watched by the gauge and effects widgets.
class ScoringData {
  /// Whether a scored run is active (drives gauge/effects visibility).
  final bool active;

  /// Live synchronization percentage in [0, 100] (100 before any judgment).
  final double syncPercent;

  final int combo;
  final int bestCombo;

  /// Most-recent hit effects (bounded, transient).
  final List<HitEffect> recentHits;

  /// The finished run's result, set at song end (drives the summary modal).
  final SessionResult? lastResult;

  const ScoringData({
    this.active = false,
    this.syncPercent = 100,
    this.combo = 0,
    this.bestCombo = 0,
    this.recentHits = const [],
    this.lastResult,
  });

  /// Feedback tier 0–4 derived purely from [syncPercent].
  int get tier => feedbackTier(syncPercent);

  ScoringData copyWith({
    bool? active,
    double? syncPercent,
    int? combo,
    int? bestCombo,
    List<HitEffect>? recentHits,
    SessionResult? lastResult,
    bool clearLastResult = false,
  }) => ScoringData(
    active: active ?? this.active,
    syncPercent: syncPercent ?? this.syncPercent,
    combo: combo ?? this.combo,
    bestCombo: bestCombo ?? this.bestCombo,
    recentHits: recentHits ?? this.recentHits,
    lastResult: clearLastResult ? null : (lastResult ?? this.lastResult),
  );
}

/// Mutable per-note tracking used by the matcher (kept off the immutable state).
class _Tracked {
  final int index;
  final TimedNote note;
  TimingVerdict? verdict;
  bool waitMode = false;
  double? timingOffsetMs;
  double? reactionMs;
  int? gateOpenWallMs;
  double sustainRatio = 0;
  bool sustainFinal = false;
  bool resolved = false; // hit or missed

  _Tracked(this.index, this.note);

  bool get isHit => resolved && verdict != null && verdict != TimingVerdict.missed;
}

const int _maxRecentHits = 8;

/// Evaluates a live performance against the scored notes and exposes the rolling
/// synchronization score, combo, and per-note judgments. Driven by the [Player]
/// via [startRun]/[noteOn]/[noteOff]/[tick]/[finishRun]; it never drives
/// playback. See design D1/D2/D10.
@riverpod
class PerformanceScorer extends _$PerformanceScorer {
  final List<_Tracked> _tracked = [];
  final Map<int, int> _heldBound = {}; // pitch -> tracked index
  final List<NoteJudgment> _wrong = [];

  // Captured run metadata.
  String _pieceId = '';
  String _title = '';
  String _hands = 'both';
  double _speed = 1;

  int _combo = 0;
  int _bestCombo = 0;
  final List<HitEffect> _recent = [];

  @override
  ScoringData build() => const ScoringData();

  Clock get _clock => ref.read(clockProvider);

  /// Begins a scored run over [notes] (the visible scored notes at start).
  void startRun({
    required String pieceId,
    required String title,
    required String hands,
    required double speed,
    required List<TimedNote> notes,
  }) {
    _tracked
      ..clear()
      ..addAll([
        for (var i = 0; i < notes.length; i++) _Tracked(i, notes[i]),
      ]);
    _heldBound.clear();
    _wrong.clear();
    _combo = 0;
    _bestCombo = 0;
    _recent.clear();
    _pieceId = pieceId;
    _title = title;
    _hands = hands;
    _speed = speed;
    state = const ScoringData(active: true, syncPercent: 100);
  }

  /// Discards the current run without producing a result (e.g. the render mode
  /// left the scored views mid-run).
  void cancelRun() {
    if (!state.active) return;
    _tracked.clear();
    _heldBound.clear();
    _wrong.clear();
    state = state.copyWith(active: false);
  }

  /// Records a key press at score-clock [playheadMs] under [waitMode]. Binds to
  /// the best matching pending onset or records an extra/wrong note.
  void noteOn(int pitch, double playheadMs, {required bool waitMode}) {
    if (!state.active) return;
    final match = _matchOnset(pitch, playheadMs, waitMode: waitMode);
    if (match == null) {
      // In Wait Mode a press only counts as wrong when a gate is actually open
      // (the frozen playhead sits on an onset); presses made while exploring
      // ahead of the gate are ignored, not penalized.
      if (waitMode && !_gateActiveAt(playheadMs)) return;
      _wrong.add(NoteJudgment(
        noteIndex: -1,
        pitch: pitch,
        startMs: playheadMs.round(),
        waitMode: waitMode,
        verdict: TimingVerdict.missed,
        wrong: true,
      ));
      _combo = 0;
      _pushEffect(pitch, TimingVerdict.missed, wrong: true);
      _recompute();
      return;
    }

    final t = match;
    t.waitMode = waitMode;
    if (waitMode) {
      final opened = t.gateOpenWallMs ?? _clock.nowMs();
      final reaction = (_clock.nowMs() - opened).toDouble();
      t.reactionMs = reaction;
      t.verdict = verdictForReactionMs(reaction);
    } else {
      final offset = playheadMs - t.note.startMs;
      t.timingOffsetMs = offset;
      t.verdict = verdictForOffsetMs(offset);
    }
    t.resolved = true;
    _heldBound[pitch] = t.index;
    _combo++;
    if (_combo > _bestCombo) _bestCombo = _combo;
    _pushEffect(pitch, t.verdict!, wrong: false);
    _recompute();
  }

  /// Finalizes the sustain of the bound note for [pitch] released at [playheadMs].
  void noteOff(int pitch, double playheadMs) {
    if (!state.active) return;
    final idx = _heldBound.remove(pitch);
    if (idx == null) return;
    final t = _tracked[idx];
    if (t.sustainFinal) return;
    final held = (playheadMs - t.note.startMs).clamp(0.0, double.infinity);
    t.sustainRatio = sustainRatioFor(held, t.note.durationMs.toDouble());
    t.sustainFinal = true;
    _recompute();
  }

  /// Advances time-based bookkeeping to [playheadMs] under [waitMode]: stamps the
  /// gate-open time for onsets the frozen playhead has reached (Wait Mode), marks
  /// unplayed onsets missed once their bind window has passed (free run only), and
  /// auto-finalizes sustain for notes held past their end.
  void tick(double playheadMs, {required bool waitMode}) {
    if (!state.active) return;
    var changed = false;
    for (final t in _tracked) {
      if (waitMode &&
          !t.resolved &&
          t.gateOpenWallMs == null &&
          (t.note.startMs - playheadMs).abs() <= 1) {
        t.gateOpenWallMs = _clock.nowMs();
      }
      if (!waitMode &&
          !t.resolved &&
          t.note.startMs + ScoringWindows.freeBindMs < playheadMs) {
        t.verdict = TimingVerdict.missed;
        t.waitMode = false;
        t.resolved = true;
        _combo = 0;
        _pushEffect(t.note.pitch, TimingVerdict.missed, wrong: false);
        changed = true;
      }
      if (t.isHit &&
          !t.sustainFinal &&
          !_heldBound.containsValue(t.index) &&
          playheadMs >= t.note.startMs + t.note.durationMs) {
        t.sustainRatio = sustainRatioFor(
          (playheadMs - t.note.startMs),
          t.note.durationMs.toDouble(),
        );
        t.sustainFinal = true;
        changed = true;
      }
    }
    if (changed) _recompute();
  }

  /// Ends the run at [playheadMs], producing and storing the [SessionResult].
  void finishRun(double playheadMs, {required bool waitMode}) {
    if (!state.active) return;
    // Resolve stragglers: unplayed onsets miss; still-held notes get their
    // sustain from how far the playhead reached.
    for (final t in _tracked) {
      if (!t.resolved) {
        t.verdict = TimingVerdict.missed;
        t.waitMode = waitMode;
        t.resolved = true;
      }
      if (t.isHit && !t.sustainFinal) {
        t.sustainRatio = sustainRatioFor(
          (playheadMs - t.note.startMs).clamp(0.0, double.infinity),
          t.note.durationMs.toDouble(),
        );
        t.sustainFinal = true;
      }
    }

    final result = SessionResult.fromJudgments(
      pieceId: _pieceId,
      title: _title,
      hands: _hands,
      judgments: _buildJudgments(),
      bestCombo: _bestCombo,
      playedAtMs: _clock.nowMs(),
      speed: _speed,
    );
    state = state.copyWith(
      active: false,
      syncPercent: result.overallSyncPct,
      combo: _combo,
      bestCombo: _bestCombo,
      lastResult: result,
    );
  }

  /// Clears the last result (e.g. after the summary modal is dismissed).
  void clearLastResult() =>
      state = state.copyWith(clearLastResult: true);

  // --- internals --------------------------------------------------------

  /// Whether any pending onset sits at the (frozen) playhead — i.e. a Wait-Mode
  /// gate is open.
  bool _gateActiveAt(double playheadMs) => _tracked.any(
        (t) => !t.resolved && (t.note.startMs - playheadMs).abs() <= 1,
      );

  _Tracked? _matchOnset(int pitch, double playheadMs, {required bool waitMode}) {
    _Tracked? best;
    var bestDelta = double.infinity;
    for (final t in _tracked) {
      if (t.resolved || t.note.pitch != pitch) continue;
      final delta = (t.note.startMs - playheadMs).abs();
      if (waitMode) {
        // The active gate sits exactly at the frozen playhead.
        if (delta <= 1 && delta < bestDelta) {
          best = t;
          bestDelta = delta;
        }
      } else {
        if (delta <= ScoringWindows.freeBindMs && delta < bestDelta) {
          best = t;
          bestDelta = delta;
        }
      }
    }
    return best;
  }

  List<NoteJudgment> _buildJudgments() {
    final list = <NoteJudgment>[
      for (final t in _tracked)
        if (t.resolved)
          NoteJudgment(
            noteIndex: t.index,
            pitch: t.note.pitch,
            startMs: t.note.startMs,
            waitMode: t.waitMode,
            verdict: t.verdict!,
            timingOffsetMs: t.timingOffsetMs,
            reactionMs: t.reactionMs,
            sustainRatio: t.isHit ? t.sustainRatio : 0,
          ),
      ..._wrong,
    ]..sort((a, b) => a.startMs.compareTo(b.startMs));
    return list;
  }

  /// Recomputes the live synchronization percentage from resolved onsets, using
  /// each hit's provisional-or-final sustain (unfinalized ⇒ full credit so the
  /// gauge doesn't dip while a note is still being held correctly).
  void _recompute() {
    final resolved = _tracked.where((t) => t.resolved);
    final verdicts = resolved.map((t) => t.verdict!);
    final sustains = resolved
        .where((t) => t.isHit)
        .map((t) => t.sustainFinal ? t.sustainRatio : 1.0);
    final pct = syncPercent(
      onsetVerdicts: verdicts,
      sustainRatios: sustains,
      wrongNotes: _wrong.length,
    );
    state = state.copyWith(
      syncPercent: pct,
      combo: _combo,
      bestCombo: _bestCombo,
      recentHits: List.unmodifiable(_recent),
    );
  }

  void _pushEffect(int pitch, TimingVerdict verdict, {required bool wrong}) {
    _recent.add(HitEffect(
      pitch: pitch,
      verdict: verdict,
      wrong: wrong,
      atMs: _clock.nowMs(),
    ));
    if (_recent.length > _maxRecentHits) {
      _recent.removeRange(0, _recent.length - _maxRecentHits);
    }
  }
}
