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

import 'dart:math' as math;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../painters/notation_palette.dart';
import '../painters/staff_painter.dart';
import '../services/audio_service.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The score as a **horizontal ribbon**, embedded in the practice pickers so the
/// range is chosen on the music itself — tap the first bar, then the last one —
/// instead of on blind bar numbers (change: add-measure-range-practice, D5).
///
/// A musician does not know that the hard passage is "bars 17–20"; they know it
/// by how it looks. The steppers underneath stay as fine-tuning (and as the only
/// control when the piece carries no measure table), but this is the primary
/// picker.
///
/// **Why horizontal.** The engraved Partition stacks systems vertically, so it
/// needs ~320px to show three lines and hides the rest behind a scroll. The
/// range being picked is a *time* range, and the Portée already maps time to x —
/// so the whole piece fits in one band about a third as tall, with every bar
/// reachable without scrolling. It renders the piece **whole**: `lookAheadMs` is
/// the song's own duration and the density caps are disabled, which turns the
/// playback window into a static overview.
class PracticeScoreStrip extends ConsumerWidget {
  const PracticeScoreStrip({
    super.key,
    required this.fromMeasure,
    required this.toMeasure,
    required this.onRangeChanged,
    this.height = 132,
  });

  /// The draft range (0-based, inclusive) currently tinted on the ribbon.
  final int fromMeasure;
  final int toMeasure;

  /// Called with a normalized `(start, end)` once the second bar is tapped.
  final void Function(int start, int end) onRangeChanged;

  /// Preferred height of the ribbon. Yields on a short viewport (phone landscape
  /// is ~320pt tall and this sits in a dialog that must keep its action button
  /// reachable).
  final double height;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final data = ref.watch(playerProvider);
    // Without a measure table (the demo score) there is nothing to pick on:
    // the steppers below remain the whole control.
    if (data.measureStartMs.isEmpty || data.songEndMs <= 0) {
      return const SizedBox.shrink();
    }
    return _Ribbon(
      key: const Key('practice-score-strip-host'),
      height: math.min(height, MediaQuery.of(context).size.height * 0.30),
      fromMeasure: fromMeasure,
      toMeasure: toMeasure,
      onRangeChanged: onRangeChanged,
    );
  }
}

/// Stateful half: it owns the measure rects the painter fills and the
/// first-tap-in-progress marker.
class _Ribbon extends ConsumerStatefulWidget {
  const _Ribbon({
    super.key,
    required this.height,
    required this.fromMeasure,
    required this.toMeasure,
    required this.onRangeChanged,
  });

  final double height;
  final int fromMeasure;
  final int toMeasure;
  final void Function(int start, int end) onRangeChanged;

  @override
  ConsumerState<_Ribbon> createState() => _RibbonState();
}

/// Horizontal room given to each bar. Enough for the note heads of a busy bar
/// not to collide, which is the whole reason this scrolls instead of fitting.
const double _pxPerMeasure = 96;

class _RibbonState extends ConsumerState<_Ribbon> {
  /// Measure rects of the last painted frame, refilled by the painter, so a tap
  /// resolves to the bar under the finger.
  final List<({int measure, Rect rect})> _hits = [];

  /// First bar of a selection in progress (null = the next tap starts one).
  int? _pendingStart;

  final ScrollController _scroll = ScrollController();

  /// Audition of the chosen passage: a timer walks the range once, sounding the
  /// notes through the same synth the player uses, so picking bars is something
  /// you HEAR, not only see. Restarted from scratch whenever the range changes.
  ///
  /// A plain [Timer] rather than a [Ticker]: this lives inside a dialog route,
  /// and a ticker is silently muted whenever its subtree's [TickerMode] is off —
  /// a failure mode that looks exactly like "the audio never starts".
  Timer? _preview;

  /// Playhead of the audition, and the bounds/notes of the pass currently
  /// running. They are FIELDS, not closure captures: capturing them once meant
  /// every later range change kept auditioning the first passage.
  double _previewMs = 0;
  double _previewEndMs = 0;
  List<TimedNote> _previewNotes = const [];
  final Set<int> _previewSounding = <int>{};
  ({int start, int end})? _previewedRange;

  /// Captured up front: [dispose] silences the audition, and by then `ref` is
  /// no longer usable.
  late final AudioService _audio;

  @override
  void initState() {
    super.initState();
    _audio = ref.read(audioServiceProvider);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      // Audition the passage as soon as the picker opens — waiting for a first
      // edit meant simply opening it was silent.
      _auditionRange(widget.fromMeasure, widget.toMeasure);
      // And SHOW it: the picker opens already holding the saved range, so no
      // bound ever "changes" and the ribbon would otherwise sit at bar 1 while
      // the passage is off-screen.
      _scrollTo(widget.fromMeasure, ref.read(playerProvider));
    });
  }

  @override
  void dispose() {
    _stopPreview();
    _scroll.dispose();
    super.dispose();
  }

  /// Silences the audition and forgets what it was playing.
  void _stopPreview() {
    _preview?.cancel();
    _preview = null;
    for (final p in _previewSounding) {
      _audio.noteOff(p);
    }
    _previewSounding.clear();
  }

  /// Plays bars [from]…[to] once, cutting off whatever was playing. A no-op when
  /// the same range is already the one being auditioned.
  void _auditionRange(int from, int to) {
    if (_previewedRange?.start == from && _previewedRange?.end == to) return;
    final data = ref.read(playerProvider);
    if (data.measureStartMs.isEmpty || from >= data.measureStartMs.length) {
      return;
    }
    _previewedRange = (start: from, end: to);
    _stopPreview();
    _previewMs = data.measureStartMs[from].toDouble();
    _previewEndMs = data.measureEndMs(to);
    _previewNotes = data.visibleNotes;
    if (_previewEndMs <= _previewMs) return;
    const step = Duration(milliseconds: 16);
    _preview = Timer.periodic(step, (_) {
      final next = _previewMs + step.inMilliseconds;
      if (next >= _previewEndMs) {
        _stopPreview(); // one pass only — this is an audition, not a loop
        return;
      }
      final edges = scoreNoteEdges(
        visible: _previewNotes,
        from: _previewMs,
        to: next,
        sounding: _previewSounding,
      );
      for (final p in edges.stops) {
        _audio.noteOff(p);
        _previewSounding.remove(p);
      }
      for (final p in edges.starts) {
        _audio.noteOn(p);
        _previewSounding.add(p);
      }
      _previewMs = next;
    });
  }

  @override
  void didUpdateWidget(covariant _Ribbon old) {
    super.didUpdateWidget(old);
    final from = widget.fromMeasure;
    final to = widget.toMeasure;
    if (old.fromMeasure == from && old.toMeasure == to) return;
    final data = ref.read(playerProvider);
    if (data.measureStartMs.isEmpty) return;
    // Follow the bound that actually moved, so a +/- on "to" does not scroll
    // back to "from".
    _scrollTo(old.toMeasure != to ? to : from, data);
    // Re-audition the NEW passage; the previous pass is cut off rather than
    // left ringing over it.
    _auditionRange(from, to);
  }

  /// Brings [measure] into view on the ribbon, so moving a bound with the +/−
  /// steppers always shows the bar being moved instead of leaving it off-screen.
  void _scrollTo(int measure, PlayerData data) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_scroll.hasClients) return;
      final hit = _hits.where((h) => h.measure == measure).firstOrNull;
      if (hit == null) return;
      final viewport = _scroll.position.viewportDimension;
      // The painter records rects in CANVAS coordinates — absolute within the
      // scrollable content — so centring is `centre - viewport/2`. Adding the
      // current offset (as this once did) double-counts it and the ribbon drifts
      // instead of landing on the bar.
      final target = (hit.rect.center.dx - viewport / 2).clamp(
        0.0,
        _scroll.position.maxScrollExtent,
      );
      if ((target - _scroll.offset).abs() < 4) return;
      _scroll.animateTo(
        target,
        duration: const Duration(milliseconds: 250),
        curve: Curves.easeOut,
      );
    });
  }

  void _onTapUp(TapUpDetails details) {
    int? measure;
    for (final hit in _hits) {
      // Only the horizontal span matters: the ribbon is one system tall, and a
      // tap slightly above or below the staff still means that bar.
      if (details.localPosition.dx >= hit.rect.left &&
          details.localPosition.dx < hit.rect.right) {
        measure = hit.measure;
        break;
      }
    }
    if (measure == null) return;
    final start = _pendingStart;
    if (start == null) {
      // First tap: show it alone so the pick reads back immediately.
      setState(() => _pendingStart = measure);
      return;
    }
    setState(() => _pendingStart = null);
    // Normalized here so tapping right-to-left works exactly like left-to-right.
    widget.onRangeChanged(math.min(start, measure), math.max(start, measure));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final data = ref.watch(playerProvider);
    final pending = _pendingStart;
    final range = pending != null
        ? (start: pending, end: pending)
        : (start: widget.fromMeasure, end: widget.toMeasure);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 6),
          child: Text(
            pending != null
                ? l10n.practicePickEndBar
                : l10n.practicePickStartBar,
            style: TextStyle(
              color: pending != null
                  ? CymbraColors.tertiary
                  : CymbraColors.onSurfaceVariant,
              fontSize: 11,
              fontWeight: pending != null ? FontWeight.w700 : FontWeight.w400,
            ),
          ),
        ),
        Container(
          height: widget.height,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: CymbraColors.outlineVariant),
          ),
          clipBehavior: Clip.antiAlias,
          // Give every bar a readable slice and SCROLL, rather than squeezing the
          // whole piece into the visible width — at ~15px per bar the note heads
          // simply pile on top of each other and nothing is pickable. The canvas
          // is sized from the bar count; the painter's fixed head occupies the
          // first quarter, hence the /0.75.
          child: LayoutBuilder(
            builder: (context, constraints) => SingleChildScrollView(
              controller: _scroll,
              scrollDirection: Axis.horizontal,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTapUp: _onTapUp,
                child: CustomPaint(
                  key: const Key('practice-score-strip'),
                  // Never narrower than the viewport (a short piece must not
                  // leave a gap), never tighter than a readable bar.
                  size: Size(
                    math.max(
                      data.measureStartMs.length * _pxPerMeasure,
                      constraints.maxWidth,
                    ),
                    widget.height,
                  ),
                  painter: StaffPainter(
                    notes: data.visibleNotes,
                    rests: data.visibleRests,
                    // A STATIC overview, not playback: time zero at the left, the
                    // whole song as the window, and the density caps disabled so
                    // nothing narrows it back down to a playback-sized slice.
                    elapsedMs: 0,
                    lookAheadMs: data.songEndMs,
                    onsetGapMs: null,
                    measureMs: null,
                    showPlayhead: false,
                    // Just past the clef/armature, so the bars get the width
                    // instead of a quarter of the canvas sitting empty.
                    timeOriginFraction: math.min(
                      0.10,
                      110 /
                          math.max(
                            data.measureStartMs.length * _pxPerMeasure,
                            constraints.maxWidth,
                          ),
                    ),
                    activeNotes: const {},
                    bpm: data.bpm,
                    songEndMs: data.songEndMs,
                    keyFifths: data.keyFifths,
                    measureKeyFifths: data.measureKeyFifths,
                    beats: data.beats,
                    beatType: data.beatType,
                    measureStartMs: data.measureStartMs,
                    palette: NotationPalette.paper,
                    practiceRange: range,
                    measureHits: _hits,
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
