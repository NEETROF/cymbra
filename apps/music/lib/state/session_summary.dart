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

import 'performance_scoring_core.dart';

/// The judgment recorded for one event in a scored run.
///
/// Two shapes share this record (see design D6/D10):
/// - an **onset judgment** ([wrong] == false): an expected score note, carrying
///   its [verdict], its [sustainRatio] (absent on a percussion stroke), and —
///   depending on the mode active when it was judged — a [timingOffsetMs]
///   (Wait Mode off) or a [reactionMs] (Wait Mode on).
/// - an **extra/wrong note** ([wrong] == true): a press that bound to no open
///   onset. [noteIndex] is -1, [verdict] is [TimingVerdict.missed], and the
///   timing/sustain fields are unused.
///
/// Plain immutable class with manual JSON (no json_serializable dependency),
/// matching the codebase's `TimedNote`/`Account` style.
class NoteJudgment {
  /// Index into the scored piece's note list, or -1 for an extra/wrong note.
  final int noteIndex;
  final int pitch;
  final int startMs;

  /// Wait-Mode state active at the moment this onset was judged. Stamped
  /// per-onset because Wait Mode can toggle mid-run.
  final bool waitMode;

  final TimingVerdict verdict;

  /// Signed score-clock offset from the onset (Wait Mode off). Null otherwise.
  final double? timingOffsetMs;

  /// Reaction time from gate-open to attack (Wait Mode on). Null otherwise.
  final double? reactionMs;

  /// Sustain ratio in [0, 1] for a landed onset; 0 for a miss/wrong note.
  ///
  /// **Absent (null), never zero, on a percussion stroke** (change:
  /// add-drum-scoring): a stroke's release is an artefact of the hardware, not
  /// of the player, so no ratio is derived at all — the same absence an
  /// unplayed mode's sub-score carries, and the serialization preserves it.
  final double? sustainRatio;

  /// True for an extra/wrong note that bound to no onset.
  final bool wrong;

  const NoteJudgment({
    required this.noteIndex,
    required this.pitch,
    required this.startMs,
    required this.waitMode,
    required this.verdict,
    this.timingOffsetMs,
    this.reactionMs,
    this.sustainRatio = 0,
    this.wrong = false,
  });

  /// Whether this is a landed (non-missed, non-wrong) onset.
  bool get isHit => !wrong && verdict != TimingVerdict.missed;

  Map<String, dynamic> toJson() => {
    'noteIndex': noteIndex,
    'pitch': pitch,
    'startMs': startMs,
    'waitMode': waitMode,
    'verdict': verdict.name,
    if (timingOffsetMs != null) 'timingOffsetMs': timingOffsetMs,
    if (reactionMs != null) 'reactionMs': reactionMs,
    // Omitted — not zeroed — for a percussion stroke, so the absence survives
    // the round trip exactly as an absent mode sub-score does.
    if (sustainRatio != null) 'sustainRatio': sustainRatio,
    'wrong': wrong,
  };

  factory NoteJudgment.fromJson(Map<String, dynamic> json) => NoteJudgment(
    noteIndex: (json['noteIndex'] as num).toInt(),
    pitch: (json['pitch'] as num).toInt(),
    startMs: (json['startMs'] as num).toInt(),
    waitMode: json['waitMode'] as bool,
    verdict: TimingVerdict.values.byName(json['verdict'] as String),
    timingOffsetMs: (json['timingOffsetMs'] as num?)?.toDouble(),
    reactionMs: (json['reactionMs'] as num?)?.toDouble(),
    // Absent ⇒ absent: a percussion stroke never carried one, and every
    // keyboard record ever written carries it.
    sustainRatio: (json['sustainRatio'] as num?)?.toDouble(),
    wrong: json['wrong'] as bool? ?? false,
  );
}

/// Immutable, serializable result of one scored run: the overall
/// synchronization percentage, the per-mode sub-scores that feed the two
/// separate leaderboards, the run classification, per-dimension aggregates, the
/// per-verdict counts, best combo, piece identity, and the per-note judgments
/// that drive the mistake replay.
///
/// Serializable so the deferred server-sync change can upload it unchanged and
/// route it to the correct leaderboard(s).
class SessionResult {
  final String pieceId;
  final String title;

  /// Which hand(s) were played, as a [Hand]-like name (`left`/`right`/`both`).
  final String hands;

  final double overallSyncPct;
  final RunMode runMode;

  /// Sub-score over the free-run onsets, or null when there were none.
  final double? freeSyncPct;

  /// Sub-score over the Wait-Mode onsets, or null when there were none.
  final double? waitSyncPct;

  final int freeOnsetCount;
  final int waitOnsetCount;

  /// Mean signed timing offset (ms) over the free-run hit onsets — negative =
  /// tends early (rushes), positive = tends late (drags). Null when there were
  /// no free-run hits.
  final double? avgFreeOffsetMs;

  /// Mean reaction time (ms) over the Wait-Mode hit onsets. Null when there were
  /// no Wait-Mode hits.
  final double? avgReactionMs;

  /// Per-dimension aggregates in [0, 1] for the summary breakdown.
  final double timing;
  final double correctness;

  /// The sustain aggregate — **absent (null) for a percussion run** (change:
  /// add-drum-scoring), which has no sustain dimension at all. Absent like a
  /// mode sub-score with no onsets: never zero, never a constant full credit,
  /// and the summary drops the row rather than rendering it empty.
  final double? sustain;

  /// Count of judged onsets by verdict (excludes extra/wrong notes).
  final Map<TimingVerdict, int> verdictCounts;

  /// Number of extra/wrong notes recorded.
  final int wrongNotes;

  final int bestCombo;
  final List<NoteJudgment> notes;

  /// Wall-clock epoch (ms) when the run finished; stamped by the notifier.
  final int playedAtMs;

  /// Playback speed multiplier the run was played at.
  final double speed;

  /// Which input produced the run — `midi` or `microphone` (delta spec: Input
  /// Source Stamped On The Run Record). Stamped so later policy (leaderboard
  /// eligibility, analytics, support) can distinguish audio-sourced runs
  /// without re-deriving anything.
  final String inputSource;

  const SessionResult({
    required this.pieceId,
    required this.title,
    required this.hands,
    required this.overallSyncPct,
    required this.runMode,
    required this.freeSyncPct,
    required this.waitSyncPct,
    required this.freeOnsetCount,
    required this.waitOnsetCount,
    required this.avgFreeOffsetMs,
    required this.avgReactionMs,
    required this.timing,
    required this.correctness,
    required this.sustain,
    required this.verdictCounts,
    required this.wrongNotes,
    required this.bestCombo,
    required this.notes,
    required this.playedAtMs,
    required this.speed,
    this.inputSource = 'midi',
  });

  /// Derives a full result from the per-note [judgments] and run metadata,
  /// computing the overall and per-mode sub-scores via the scoring core so all
  /// derivation lives in one tested place.
  ///
  /// [percussion] selects the two-dimension blend (change: add-drum-scoring):
  /// the sustain aggregate is then absent and every sub-score is renormalized
  /// over timing and correctness alone. The keyboard default is untouched.
  factory SessionResult.fromJudgments({
    required String pieceId,
    required String title,
    required String hands,
    required List<NoteJudgment> judgments,
    required int bestCombo,
    required int playedAtMs,
    required double speed,
    bool percussion = false,
    bool acousticInput = false,
  }) {
    // Both attack-only regimes share one blend (delta spec: Sustain Judgment,
    // audio-sourced runs): percussion has no sustain to judge, and acoustic
    // input cannot observe releases (the damper pedal masks them).
    final sustainless = percussion || acousticInput;
    double? subScore(bool waitMode) {
      final onsets = judgments
          .where((j) => !j.wrong && j.waitMode == waitMode)
          .toList(growable: false);
      if (onsets.isEmpty) return null;
      final wrong = judgments
          .where((j) => j.wrong && j.waitMode == waitMode)
          .length;
      if (sustainless) {
        return percussionSyncPercent(
          onsetVerdicts: onsets.map((j) => j.verdict),
          wrongNotes: wrong,
        );
      }
      return syncPercent(
        onsetVerdicts: onsets.map((j) => j.verdict),
        sustainRatios: onsets.where((j) => j.isHit).map((j) => j.sustainRatio!),
        wrongNotes: wrong,
      );
    }

    final onsets = judgments.where((j) => !j.wrong).toList(growable: false);
    final wrong = judgments.where((j) => j.wrong).length;
    final freeCount = onsets.where((j) => !j.waitMode).length;
    final waitCount = onsets.where((j) => j.waitMode).length;

    final verdicts = onsets.map((j) => j.verdict).toList(growable: false);
    // Empty and unread on the percussion path — every branch that consumes it
    // is guarded by [percussion] below.
    final ratios = onsets
        .where((j) => j.isHit && j.sustainRatio != null)
        .map((j) => j.sustainRatio!);

    double? mean(Iterable<double> xs) {
      var sum = 0.0;
      var n = 0;
      for (final x in xs) {
        sum += x;
        n++;
      }
      return n == 0 ? null : sum / n;
    }

    final avgFreeOffset = mean(
      onsets
          .where((j) => !j.waitMode && j.isHit && j.timingOffsetMs != null)
          .map((j) => j.timingOffsetMs!),
    );
    final avgReaction = mean(
      onsets
          .where((j) => j.waitMode && j.isHit && j.reactionMs != null)
          .map((j) => j.reactionMs!),
    );

    final counts = <TimingVerdict, int>{};
    for (final j in onsets) {
      counts[j.verdict] = (counts[j.verdict] ?? 0) + 1;
    }

    return SessionResult(
      pieceId: pieceId,
      title: title,
      hands: hands,
      overallSyncPct: sustainless
          ? percussionSyncPercent(onsetVerdicts: verdicts, wrongNotes: wrong)
          : syncPercent(
              onsetVerdicts: verdicts,
              sustainRatios: ratios,
              wrongNotes: wrong,
            ),
      runMode: classifyRun(freeOnsets: freeCount, waitOnsets: waitCount),
      freeSyncPct: subScore(false),
      waitSyncPct: subScore(true),
      freeOnsetCount: freeCount,
      waitOnsetCount: waitCount,
      avgFreeOffsetMs: avgFreeOffset,
      avgReactionMs: avgReaction,
      timing: timingScore(verdicts),
      correctness: correctnessScore(verdicts, wrong),
      sustain: sustainless
          ? null
          : sustainScore(ratios, anyOnsetJudged: verdicts.isNotEmpty),
      verdictCounts: counts,
      wrongNotes: wrong,
      bestCombo: bestCombo,
      notes: judgments,
      playedAtMs: playedAtMs,
      speed: speed,
      inputSource: acousticInput ? 'microphone' : 'midi',
    );
  }

  Map<String, dynamic> toJson() => {
    'pieceId': pieceId,
    'title': title,
    'hands': hands,
    'overallSyncPct': overallSyncPct,
    'runMode': runMode.name,
    if (freeSyncPct != null) 'freeSyncPct': freeSyncPct,
    if (waitSyncPct != null) 'waitSyncPct': waitSyncPct,
    'freeOnsetCount': freeOnsetCount,
    'waitOnsetCount': waitOnsetCount,
    if (avgFreeOffsetMs != null) 'avgFreeOffsetMs': avgFreeOffsetMs,
    if (avgReactionMs != null) 'avgReactionMs': avgReactionMs,
    'timing': timing,
    'correctness': correctness,
    // Omitted for a percussion run: the dimension does not exist there, and an
    // absent key restores as an absent aggregate.
    if (sustain != null) 'sustain': sustain,
    'verdictCounts': {
      for (final e in verdictCounts.entries) e.key.name: e.value,
    },
    'wrongNotes': wrongNotes,
    'bestCombo': bestCombo,
    'notes': notes.map((n) => n.toJson()).toList(),
    'playedAtMs': playedAtMs,
    'speed': speed,
    'inputSource': inputSource,
  };

  factory SessionResult.fromJson(Map<String, dynamic> json) => SessionResult(
    pieceId: json['pieceId'] as String,
    title: json['title'] as String,
    hands: json['hands'] as String,
    overallSyncPct: (json['overallSyncPct'] as num).toDouble(),
    runMode: RunMode.values.byName(json['runMode'] as String),
    freeSyncPct: (json['freeSyncPct'] as num?)?.toDouble(),
    waitSyncPct: (json['waitSyncPct'] as num?)?.toDouble(),
    freeOnsetCount: (json['freeOnsetCount'] as num).toInt(),
    waitOnsetCount: (json['waitOnsetCount'] as num).toInt(),
    avgFreeOffsetMs: (json['avgFreeOffsetMs'] as num?)?.toDouble(),
    avgReactionMs: (json['avgReactionMs'] as num?)?.toDouble(),
    timing: (json['timing'] as num).toDouble(),
    correctness: (json['correctness'] as num).toDouble(),
    sustain: (json['sustain'] as num?)?.toDouble(),
    verdictCounts: {
      for (final e in (json['verdictCounts'] as Map<String, dynamic>).entries)
        TimingVerdict.values.byName(e.key): (e.value as num).toInt(),
    },
    wrongNotes: (json['wrongNotes'] as num?)?.toInt() ?? 0,
    bestCombo: (json['bestCombo'] as num).toInt(),
    notes: (json['notes'] as List<dynamic>)
        .map((e) => NoteJudgment.fromJson(e as Map<String, dynamic>))
        .toList(),
    playedAtMs: (json['playedAtMs'] as num).toInt(),
    speed: (json['speed'] as num).toDouble(),
    // Absent on records stored before the stamp existed: those were MIDI.
    inputSource: json['inputSource'] as String? ?? 'midi',
  );
}
