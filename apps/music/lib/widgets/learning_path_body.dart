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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../painters/path_curve_painter.dart';
import '../painters/smufl.dart';
import '../screens/lesson_player_screen.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../state/placement_core.dart';
import '../state/piece_stars_core.dart';
import '../state/placement_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../theme/motion.dart';
import 'lesson_hero.dart';
import 'star_row.dart';

part 'learning_path_body.g.dart';

/// The units' Bravura emblems, rotating by unit position — free iconography
/// the app already ships.
const List<String> _unitGlyphs = [
  Smufl.gClef,
  Smufl.noteQuarterUp,
  Smufl.fClef,
  Smufl.note8thUp,
  Smufl.accidentalSharp,
  Smufl.noteHalfUp,
  Smufl.cClef,
];

/// Fixed height of a lesson node row, so the curve behind the nodes is drawn
/// from the very indents the rows use (change:
/// add-music-motion-and-celebrations).
const double kPathNodeRowHeight = 56;

/// One unit of the path: its listings in catalogue order, sharing a slug and
/// an inline-i18n title.
typedef PathUnit = ({
  String slug,
  InlineText title,
  List<CourseListing> lessons,
});

/// Groups [listings] (already in catalogue order) into units, preserving
/// order. Listings without a unit fall back to grouping by level, so a
/// pre-unit catalogue still renders a sensible path.
List<PathUnit> pathUnitsOf(List<CourseListing> listings) {
  final units = <String, List<CourseListing>>{};
  for (final l in listings) {
    (units[unitSlugOf(l)] ??= []).add(l);
  }
  return [
    for (final e in units.entries)
      (slug: e.key, title: e.value.first.unitTitle, lessons: e.value),
  ];
}

/// A gentle meander keeps the path playful; the curve is drawn through the
/// same centres.
double pathNodeIndent(int i) =>
    12.0 + 28.0 * ((i % 4 < 2) ? (i % 4) : (3 - i % 4));

/// Units whose header already lit up this app session (change:
/// add-music-motion-and-celebrations): a finished unit lights once, not on
/// every rebuild of the path.
@Riverpod(keepAlive: true)
class LitUnits extends _$LitUnits {
  @override
  Set<String> build() => const {};

  void mark(String slug) => state = {...state, slug};
}

/// The learning path's body (change: add-notation-courses, shared by the Learn
/// door since add-music-learn-play-home): units in catalogue order, each with
/// its progress, and one lesson node per course — completed nodes are checked,
/// the first uncompleted node across the whole path breathes as "next up", and
/// every node stays tappable (the order guides, it never locks). A curve runs
/// behind the nodes, filled up to the last completed one (change:
/// add-music-motion-and-celebrations).
///
/// A non-scrolling column, so it drops into the path screen's list and into the
/// Learn door's scroll view alike.
class LearningPathBody extends ConsumerWidget {
  const LearningPathBody({super.key, this.heroes = true});

  /// Whether the nodes carry the lesson's shared-element tag. Off on the Learn
  /// door, where the continue card already carries it for the same lesson.
  final bool heroes;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final lang = Localizations.localeOf(context).languageCode;
    final listings = ref.watch(coursesProvider).valueOrNull ?? const [];
    final completion = ref.watch(courseCompletionProvider);
    final skipped = ref.watch(skippedUnitsProvider);
    final units = pathUnitsOf(listings);
    // Next up: the first course neither completed nor passed (change:
    // add-music-piece-lessons — a passed piece step counts as done), outside
    // the units the placement skipped (change: add-music-placement-test).
    final nextId = listings
        .where(
          (l) => !completion.isDone(l.id) && !skipped.contains(unitSlugOf(l)),
        )
        .map((l) => l.id)
        .firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (var i = 0; i < units.length; i++)
          _UnitSection(
            unit: units[i],
            glyph: _unitGlyphs[i % _unitGlyphs.length],
            completion: completion,
            nextId: nextId,
            lang: lang,
            heroes: heroes,
            skipped: skipped.contains(units[i].slug),
          ),
      ],
    );
  }
}

class _UnitSection extends ConsumerStatefulWidget {
  const _UnitSection({
    required this.unit,
    required this.glyph,
    required this.completion,
    required this.nextId,
    required this.lang,
    required this.heroes,
    this.skipped = false,
  });

  final PathUnit unit;
  final String glyph;
  final CourseCompletionState completion;
  final String? nextId;
  final String lang;
  final bool heroes;

  /// Skipped by placement (change: add-music-placement-test): collapsed to
  /// its header until opened; its lessons keep their real states underneath.
  final bool skipped;

  @override
  ConsumerState<_UnitSection> createState() => _UnitSectionState();
}

class _UnitSectionState extends ConsumerState<_UnitSection>
    with TickerProviderStateMixin {
  /// The segment filling right now (its start node's index), or -1.
  int _fillingIndex = -1;

  /// A skipped unit the learner opened anyway.
  bool _expanded = false;
  late final AnimationController _fill = AnimationController(
    vsync: this,
    duration: Motion.standard,
  );
  late final AnimationController _glow = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 700),
  );
  late List<bool> _wasDone = _doneFlags();

  List<bool> _doneFlags() => [
    for (final l in widget.unit.lessons) widget.completion.isDone(l.id),
  ];

  @override
  void initState() {
    super.initState();
    _fill.addListener(_tick);
    _glow.addListener(_tick);
  }

  void _tick() => setState(() {});

  @override
  void didUpdateWidget(_UnitSection old) {
    super.didUpdateWidget(old);
    final now = _doneFlags();
    // A node that just became done: its segment to the next node fills.
    for (var i = 0; i < now.length; i++) {
      final was = i < _wasDone.length && _wasDone[i];
      if (now[i] && !was && i < now.length - 1) {
        _fillingIndex = i;
        _fill
          ..duration = Motion.of(context, Motion.standard)
          ..forward(from: 0);
        break;
      }
    }
    // The unit just finished: its header lights, once per app session. The
    // mark is a provider write, so it waits for the frame to end (a provider
    // is never modified while the tree builds).
    final wasAll = _wasDone.isNotEmpty && _wasDone.every((d) => d);
    final isAll = now.isNotEmpty && now.every((d) => d);
    if (isAll &&
        !wasAll &&
        !ref.read(litUnitsProvider).contains(widget.unit.slug)) {
      final glowFor = Motion.of(context, const Duration(milliseconds: 700));
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        final lit = ref.read(litUnitsProvider);
        if (lit.contains(widget.unit.slug)) return;
        ref.read(litUnitsProvider.notifier).mark(widget.unit.slug);
        _glow
          ..duration = glowFor
          ..forward(from: 0);
      });
    }
    _wasDone = now;
  }

  @override
  void dispose() {
    _fill.dispose();
    _glow.dispose();
    super.dispose();
  }

  /// Passed, or completed without any piece run → done. Completed with a
  /// piece run that never passed → visited (the third state, change:
  /// add-music-piece-lessons). Otherwise next up or later.
  _NodeState _stateOf(String id) {
    if (widget.completion.isVisited(id)) return _NodeState.visited;
    if (widget.completion.isDone(id)) return _NodeState.done;
    return id == widget.nextId ? _NodeState.next : _NodeState.later;
  }

  @override
  Widget build(BuildContext context) {
    final unit = widget.unit;
    final completion = widget.completion;
    final done = unit.lessons.where((l) => completion.isDone(l.id)).length;
    final total = unit.lessons.length;
    final title = resolveInline(unit.title, widget.lang);
    if (widget.skipped && !_expanded) {
      final l10n = AppLocalizations.of(context);
      return Container(
        key: Key('path-unit-${unit.slug}'),
        margin: const EdgeInsets.only(top: 12, bottom: 4),
        padding: const EdgeInsets.fromLTRB(14, 10, 6, 10),
        decoration: BoxDecoration(
          color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.6),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: CymbraColors.onSurfaceVariant.withValues(alpha: 0.15),
          ),
        ),
        child: Row(
          children: [
            Text(
              widget.glyph,
              style: const TextStyle(
                fontFamily: Smufl.fontFamily,
                fontSize: 22,
                height: 1,
                color: CymbraColors.onSurfaceVariant,
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title.isEmpty ? unit.slug : title,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 14,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  Text(
                    '${l10n.placementSkippedUnit} · $done/$total',
                    key: Key('path-unit-skipped-${unit.slug}'),
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ),
            TextButton(
              key: Key('path-unit-open-${unit.slug}'),
              onPressed: () => setState(() => _expanded = true),
              child: Text(
                l10n.placementOpenUnit,
                style: const TextStyle(color: CymbraColors.secondary),
              ),
            ),
          ],
        ),
      );
    }
    // The glow rises then fades over its run: 0 → 1 → 0.
    final glow = _glow.value > 0 && _glow.value < 1
        ? (1 - (2 * _glow.value - 1).abs()).clamp(0.0, 1.0)
        : 0.0;
    final fills = [
      for (var i = 0; i < total; i++)
        i == _fillingIndex && _fill.isAnimating
            ? _fill.value
            : (completion.isDone(unit.lessons[i].id) ? 1.0 : 0.0),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          key: Key('path-unit-${unit.slug}'),
          margin: const EdgeInsets.only(top: 12, bottom: 8),
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: done == total
                  ? CymbraColors.tertiary.withValues(alpha: 0.6)
                  : CymbraColors.onSurfaceVariant.withValues(alpha: 0.15),
            ),
            boxShadow: glow > 0
                ? [
                    BoxShadow(
                      color: CymbraColors.tertiary.withValues(
                        alpha: 0.5 * glow,
                      ),
                      blurRadius: 18 * glow,
                      spreadRadius: 2 * glow,
                    ),
                  ]
                : null,
          ),
          child: Row(
            children: [
              Text(
                widget.glyph,
                style: const TextStyle(
                  fontFamily: Smufl.fontFamily,
                  fontSize: 26,
                  height: 1,
                  color: CymbraColors.primary,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title.isEmpty ? unit.slug : title,
                      style: const TextStyle(
                        color: CymbraColors.onSurface,
                        fontSize: 15,
                        fontWeight: FontWeight.w800,
                      ),
                    ),
                    const SizedBox(height: 6),
                    ClipRRect(
                      borderRadius: BorderRadius.circular(6),
                      child: LinearProgressIndicator(
                        value: total == 0 ? 0 : done / total,
                        minHeight: 9,
                        backgroundColor: CymbraColors.background,
                        color: done == total
                            ? CymbraColors.tertiary
                            : CymbraColors.primary,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 12),
              Text(
                '$done/$total',
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ],
          ),
        ),
        // The curve runs behind the nodes, from the very centres the rows put
        // their circles at: filled up to the last completed node, dashed
        // beyond.
        Stack(
          children: [
            Positioned.fill(
              child: IgnorePointer(
                child: CustomPaint(
                  key: Key('path-curve-${unit.slug}'),
                  painter: PathCurvePainter(
                    centers: [
                      for (var i = 0; i < total; i++)
                        Offset(
                          pathNodeIndent(i) + 20,
                          i * kPathNodeRowHeight + kPathNodeRowHeight / 2,
                        ),
                    ],
                    fills: fills,
                  ),
                ),
              ),
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                for (var i = 0; i < total; i++)
                  _LessonNode(
                    listing: unit.lessons[i],
                    state: _stateOf(unit.lessons[i].id),
                    indent: pathNodeIndent(i),
                    lang: widget.lang,
                    hero: widget.heroes,
                    // A piece lesson's stars, from its best run (change:
                    // add-music-piece-stars); none before any run.
                    bestPct: completion.bestPct[unit.lessons[i].id],
                  ),
              ],
            ),
          ],
        ),
      ],
    );
  }
}

enum _NodeState { done, visited, next, later }

class _LessonNode extends StatefulWidget {
  const _LessonNode({
    required this.listing,
    required this.state,
    required this.indent,
    required this.lang,
    required this.hero,
    this.bestPct,
  });

  final CourseListing listing;
  final _NodeState state;
  final double indent;
  final String lang;
  final bool hero;

  /// The best overall synchronisation of this lesson's piece step, when run.
  final double? bestPct;

  @override
  State<_LessonNode> createState() => _LessonNodeState();
}

class _LessonNodeState extends State<_LessonNode>
    with TickerProviderStateMixin {
  // Eager, like ScoreChip's controllers: a lazy late-final controller crashes
  // in dispose when the node was never built visible.
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1400),
  );

  /// The check popping in when the node becomes done (change:
  /// add-music-motion-and-celebrations); already settled for a node that was
  /// done when first built.
  late final AnimationController _pop = AnimationController(
    vsync: this,
    duration: Motion.quick,
    value: 1,
  );

  @override
  void initState() {
    super.initState();
    if (widget.state == _NodeState.next) _pulse.repeat(reverse: true);
  }

  @override
  void didUpdateWidget(_LessonNode old) {
    super.didUpdateWidget(old);
    if (widget.state == _NodeState.next && !_pulse.isAnimating) {
      _pulse.repeat(reverse: true);
    } else if (widget.state != _NodeState.next && _pulse.isAnimating) {
      _pulse.stop();
    }
    if (widget.state == _NodeState.done && old.state != _NodeState.done) {
      _pop
        ..duration = Motion.of(context, Motion.quick)
        ..forward(from: 0);
    }
  }

  @override
  void dispose() {
    _pulse.dispose();
    _pop.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final title = resolveInline(widget.listing.title, widget.lang);
    final isNext = widget.state == _NodeState.next;
    final isDone = widget.state == _NodeState.done;
    final isVisited = widget.state == _NodeState.visited;
    final circleColor = switch (widget.state) {
      _NodeState.done => CymbraColors.tertiary,
      _NodeState.visited => CymbraColors.secondaryContainer,
      _NodeState.next => CymbraColors.primary,
      _NodeState.later => CymbraColors.surfaceContainerHigh,
    };
    // The row has a fixed height: with the stars under it, the title keeps to
    // one line (two overflowed it — found while dogfooding).
    final titleText = Text(
      title,
      maxLines: widget.bestPct == null ? 2 : 1,
      overflow: TextOverflow.ellipsis,
      style: TextStyle(
        color: widget.state == _NodeState.later
            ? CymbraColors.onSurfaceVariant
            : CymbraColors.onSurface,
        fontSize: 14,
        fontWeight: isNext ? FontWeight.w800 : FontWeight.w600,
      ),
    );
    return SizedBox(
      height: kPathNodeRowHeight,
      child: Padding(
        padding: EdgeInsets.only(left: widget.indent, top: 8, bottom: 8),
        child: Semantics(
          button: true,
          label: title,
          child: InkWell(
            key: Key('path-node-${widget.listing.id}'),
            borderRadius: BorderRadius.circular(24),
            onTap: () => openLessonPlayer(context, widget.listing.id),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                AnimatedBuilder(
                  animation: _pulse,
                  builder: (context, child) => Container(
                    width: 40,
                    height: 40,
                    decoration: BoxDecoration(
                      color: circleColor,
                      shape: BoxShape.circle,
                      boxShadow: isNext
                          ? [
                              BoxShadow(
                                color: CymbraColors.primary.withValues(
                                  alpha: 0.25 + 0.3 * _pulse.value,
                                ),
                                blurRadius: 10 + 6 * _pulse.value,
                              ),
                            ]
                          : null,
                    ),
                    child: ScaleTransition(
                      scale: CurvedAnimation(parent: _pop, curve: Motion.pop),
                      child: Icon(
                        isDone
                            ? Icons.check
                            : isVisited
                            ? Icons.replay
                            : (isNext ? Icons.play_arrow : Icons.music_note),
                        key: isVisited
                            ? Key('path-visited-${widget.listing.id}')
                            : null,
                        size: 20,
                        color: isDone || isNext || isVisited
                            ? CymbraColors.surfaceContainerLowest
                            : CymbraColors.onSurfaceVariant.withValues(
                                alpha: 0.6,
                              ),
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 230),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      widget.hero
                          ? Hero(
                              tag: lessonHeroTag(widget.listing.id),
                              flightShuttleBuilder: heroTextFlight,
                              child: titleText,
                            )
                          : titleText,
                      if (widget.bestPct case final pct?)
                        Padding(
                          padding: const EdgeInsets.only(top: 2),
                          child: StarRow(
                            key: Key('path-stars-${widget.listing.id}'),
                            stars: starsFor(pct),
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
