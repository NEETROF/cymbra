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

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../courses/course_manifest.dart' show RhythmTapBlock, resolveInline;
import '../courses/lesson_rhythm.dart';
import '../courses/lesson_sounder.dart';
import '../l10n/gen/app_localizations.dart';
import '../painters/smufl.dart';
import '../services/audio_service.dart';
import '../state/note_label.dart' show FigureToken, NoteFigure;
import '../state/player_data.dart' show metronomeBeatsCrossed;
import '../theme/cymbra_theme.dart';

/// The pitch every tap sounds on — one fixed woodblock-like piano note, so the
/// learner hears *their* rhythm against the click, never a melody.
const int _tapPitch = 76;

/// A course `rhythmTap` step (change: add-notation-courses, schema v2): the
/// written rhythm shown as SMuFL glyphs, tapped on a big pad against the
/// metronome — one bar of count-in, then the pattern window plus half a beat
/// of grace.
///
/// The widget only schedules and displays; every timed truth is the pure core:
/// [rhythmOnsets] says where the attacks fall, [metronomeBeatsCrossed] (steady
/// grid, shifted so count-in and pattern share one grid) says which clicks a
/// frame crosses, and [gradeRhythmTaps] says how the taps compare. A single
/// [Ticker] is the only clock — no periodic timers — so stopping it stops the
/// cadence outright, and [dispose] silences everything.
class RhythmTapView extends ConsumerStatefulWidget {
  const RhythmTapView({
    super.key,
    required this.block,
    required this.onCompleted,
  });

  final RhythmTapBlock block;

  /// Called exactly once, when a pass succeeds. [flawless] only on the very
  /// first pass with every onset hit and no stray tap.
  final void Function({required bool flawless}) onCompleted;

  @override
  ConsumerState<RhythmTapView> createState() => _RhythmTapViewState();
}

enum _Phase { intro, countIn, tapping, result }

class _RhythmTapViewState extends ConsumerState<RhythmTapView>
    with SingleTickerProviderStateMixin {
  late final Ticker _ticker;
  late final AudioService _audio;
  late final LessonSounder _sounder;
  final Set<Timer> _timers = {};

  late final ({List<double> onsetsMs, double totalMs}) _resolved = rhythmOnsets(
    pattern: widget.block.pattern,
    bpm: widget.block.bpm,
    beatType: widget.block.beatType,
  );

  /// When each pattern element (note *or* rest) starts, for the progress tint.
  late final List<double> _elementStartMs = () {
    final starts = <double>[];
    var t = 0.0;
    for (final f in widget.block.pattern) {
      starts.add(t);
      t += f.beats(widget.block.beatType) * _beatMs;
    }
    return starts;
  }();

  double get _beatMs => 60000.0 / widget.block.bpm;
  double get _barMs => widget.block.beats * _beatMs;

  _Phase _phase = _Phase.intro;

  /// A demo replay is running (intro only) — driven by the same ticker.
  bool _demo = false;

  /// Run clock in ms: the demo counts from 0, the exercise from −[_barMs]
  /// (count-in) through the pattern window.
  double _clockMs = 0;
  int _nextDemoOnset = 0;
  final List<double> _taps = [];
  List<bool> _hit = const [];
  bool _passed = false;
  bool _everFailed = false;
  bool _completed = false;
  bool _padFlash = false;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick);
    // Capture the audio service so [dispose] never touches `ref` after the
    // provider scope has been torn down (the sanctioned audition pattern).
    _audio = ref.read(audioServiceProvider);
    _sounder = LessonSounder(_audio);
  }

  @override
  void dispose() {
    _ticker.dispose();
    for (final t in _timers) {
      t.cancel();
    }
    _sounder.dispose();
    super.dispose();
  }

  /// A cancellable delay that never leaks a timer past [dispose].
  void _after(Duration d, VoidCallback fn) {
    late final Timer t;
    t = Timer(d, () {
      _timers.remove(t);
      if (mounted) fn();
    });
    _timers.add(t);
  }

  void _onTick(Duration elapsed) {
    final ms = elapsed.inMicroseconds / 1000.0;
    if (_demo) {
      _tickDemo(ms);
    } else {
      _tickExercise(ms - _barMs);
    }
  }

  void _tickDemo(double now) {
    final from = _clockMs;
    _clockMs = now;
    for (final b in _beatsCrossed(from, now)) {
      _audio.metronomeClick(accent: b.accent);
    }
    while (_nextDemoOnset < _resolved.onsetsMs.length &&
        _resolved.onsetsMs[_nextDemoOnset] <= now) {
      _sounder.tap(_tapPitch, durationMs: 150);
      _nextDemoOnset++;
    }
    if (now >= _resolved.totalMs + _beatMs / 2) {
      _ticker.stop();
      setState(() => _demo = false);
    } else {
      setState(() {});
    }
  }

  void _tickExercise(double now) {
    final from = _clockMs;
    _clockMs = now;
    // Shifted by one bar so the count-in [−bar, 0) and the pattern sit on one
    // continuous grid: the first pattern beat is just the next accent.
    for (final b in _beatsCrossed(from + _barMs, now + _barMs)) {
      _audio.metronomeClick(accent: b.accent);
    }
    if (_phase == _Phase.countIn && now >= 0) _phase = _Phase.tapping;
    if (now >= _resolved.totalMs + _beatMs / 2) {
      _ticker.stop();
      _finish();
    } else {
      setState(() {});
    }
  }

  List<({double timeMs, bool accent})> _beatsCrossed(double from, double to) =>
      metronomeBeatsCrossed(
        measureStartMs: const [],
        beats: widget.block.beats,
        bpm: widget.block.bpm,
        songEndMs: 1e12,
        from: from,
        to: to,
      );

  void _playDemo() {
    _ticker.stop();
    _sounder.stopAll();
    _demo = true;
    _clockMs = 0;
    _nextDemoOnset = 0;
    setState(() {});
    _ticker.start();
  }

  void _start() {
    _ticker.stop();
    _sounder.stopAll();
    _demo = false;
    _taps.clear();
    _hit = const [];
    _passed = false;
    _clockMs = -_barMs;
    setState(() => _phase = _Phase.countIn);
    _ticker.start();
  }

  void _retry() {
    setState(() => _phase = _Phase.intro);
    _playDemo();
  }

  void _finish() {
    final windowMs = (0.25 * _beatMs).clamp(120.0, 260.0);
    final grade = gradeRhythmTaps(
      onsetsMs: _resolved.onsetsMs,
      tapsMs: _taps,
      windowMs: windowMs,
    );
    final hits = grade.hit.where((h) => h).length;
    final passed =
        hits / _resolved.onsetsMs.length >= widget.block.passRatio &&
        grade.extras <= 1;
    final flawless =
        passed &&
        !_everFailed &&
        hits == _resolved.onsetsMs.length &&
        grade.extras == 0;
    if (!passed) _everFailed = true;
    setState(() {
      _hit = grade.hit;
      _passed = passed;
      _phase = _Phase.result;
    });
    if (!passed) return;
    _sounder.chime();
    _after(const Duration(milliseconds: 450), () {
      if (_completed) return;
      _completed = true;
      widget.onCompleted(flawless: flawless);
    });
  }

  void _padTap() {
    if (_phase != _Phase.tapping) return;
    _taps.add(_clockMs);
    _sounder.tap(_tapPitch, durationMs: 90);
    setState(() => _padFlash = true);
    _after(const Duration(milliseconds: 90), () {
      setState(() => _padFlash = false);
    });
  }

  /// The complete SMuFL glyph for one pattern element: full note (head, stem,
  /// flag) via [FigureToken], or the rest glyph — dots appended either way.
  String _glyphFor(RhythmFigure f) {
    if (!f.rest) return FigureToken(f.figure, f.dots, null).glyphWithDots;
    final rest = switch (f.figure) {
      NoteFigure.whole => Smufl.restWhole,
      NoteFigure.half => Smufl.restHalf,
      NoteFigure.eighth => Smufl.rest8th,
      NoteFigure.sixteenth => Smufl.rest16th,
      _ => Smufl.restQuarter,
    };
    return rest + Smufl.augmentationDot * f.dots;
  }

  Color _glyphColor(int index, RhythmFigure f, int onsetIndex) {
    if (_phase == _Phase.result && !f.rest) {
      // Verdict discipline: a miss is never success-coloured; rests neutral.
      return onsetIndex < _hit.length && _hit[onsetIndex]
          ? CymbraColors.tertiary
          : CymbraColors.error;
    }
    if ((_phase == _Phase.tapping || _demo) &&
        _elementStartMs[index] <= _clockMs) {
      return CymbraColors.primary; // the subtle progress cursor
    }
    return CymbraColors.onSurface;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final prompt = resolveInline(widget.block.prompt, lang);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                prompt.isEmpty ? l10n.lessonYourTurn : prompt,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            if (_passed)
              const Icon(Icons.check_circle, color: CymbraColors.tertiary),
          ],
        ),
        const SizedBox(height: 16),
        _glyphRow(),
        const SizedBox(height: 16),
        switch (_phase) {
          _Phase.intro => _introControls(l10n),
          _Phase.countIn => _countIn(),
          _Phase.tapping => _pad(),
          _Phase.result => _result(l10n),
        },
      ],
    );
  }

  Widget _glyphRow() {
    final children = <Widget>[];
    var onset = 0;
    for (var i = 0; i < widget.block.pattern.length; i++) {
      final f = widget.block.pattern[i];
      children.add(
        Padding(
          key: Key('rhythm-glyph-$i'),
          padding: const EdgeInsets.only(right: 16),
          child: Text(
            _glyphFor(f),
            style: TextStyle(
              fontFamily: Smufl.fontFamily,
              fontSize: 30,
              height: 1,
              color: _glyphColor(i, f, onset),
            ),
          ),
        ),
      );
      if (!f.rest) onset++;
    }
    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Wrap(runSpacing: 12, children: children),
    );
  }

  Widget _introControls(AppLocalizations l10n) {
    return Row(
      children: [
        OutlinedButton.icon(
          key: const Key('rhythm-listen'),
          onPressed: _playDemo,
          icon: const Icon(Icons.volume_up),
          label: Text(l10n.lessonListen),
        ),
        const SizedBox(width: 12),
        FilledButton.icon(
          key: const Key('rhythm-start'),
          onPressed: _start,
          icon: const Icon(Icons.play_arrow),
          label: Text(l10n.lessonStart),
        ),
      ],
    );
  }

  Widget _countIn() {
    final remaining = (-_clockMs / _beatMs).ceil().clamp(1, widget.block.beats);
    return SizedBox(
      height: 110,
      width: double.infinity,
      child: Center(
        child: AnimatedSwitcher(
          duration: const Duration(milliseconds: 250),
          transitionBuilder: (child, animation) {
            final scale = Tween<double>(begin: 0.5, end: 1).animate(
              CurvedAnimation(parent: animation, curve: Curves.easeOutBack),
            );
            return FadeTransition(
              opacity: animation,
              child: ScaleTransition(scale: scale, child: child),
            );
          },
          child: Text(
            '$remaining',
            key: ValueKey(remaining),
            style: const TextStyle(
              fontSize: 64,
              fontWeight: FontWeight.w500,
              color: CymbraColors.primary,
            ),
          ),
        ),
      ),
    );
  }

  Widget _pad() {
    return GestureDetector(
      key: const Key('rhythm-pad'),
      behavior: HitTestBehavior.opaque,
      onTapDown: (_) => _padTap(),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 90),
        height: 110,
        width: double.infinity,
        decoration: BoxDecoration(
          color: _padFlash
              ? CymbraColors.surfaceContainerHighest
              : CymbraColors.surfaceContainerHigh,
          borderRadius: BorderRadius.circular(16),
          border: Border.all(
            color: _padFlash
                ? CymbraColors.primary
                : CymbraColors.outlineVariant,
          ),
        ),
        child: Icon(
          Icons.touch_app,
          size: 36,
          color: _padFlash
              ? CymbraColors.primary
              : CymbraColors.onSurfaceVariant,
        ),
      ),
    );
  }

  Widget _result(AppLocalizations l10n) {
    if (_passed) {
      return Text(
        l10n.lessonWellDone,
        style: const TextStyle(
          color: CymbraColors.tertiary,
          fontSize: 18,
          fontWeight: FontWeight.w700,
        ),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.lessonAlmost,
          style: const TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
        const SizedBox(height: 12),
        FilledButton.tonalIcon(
          key: const Key('rhythm-retry'),
          onPressed: _retry,
          icon: const Icon(Icons.replay),
          label: Text(l10n.lessonTryAgain),
        ),
      ],
    );
  }
}
