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

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../painters/smufl.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'lesson_player_screen.dart';

/// Opens the full learning path (change: add-notation-courses).
void openLearningPath(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const LearningPathScreen()));

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
    final slug = l.unit.isNotEmpty ? l.unit : 'level:${l.level}';
    (units[slug] ??= []).add(l);
  }
  return [
    for (final e in units.entries)
      (slug: e.key, title: e.value.first.unitTitle, lessons: e.value),
  ];
}

/// The full-screen learning path: units in catalogue order, each with its
/// progress, and one lesson node per course — completed nodes are checked, the
/// first uncompleted node across the whole path breathes as "next up", and
/// every node stays tappable (the order guides, it never locks).
class LearningPathScreen extends ConsumerWidget {
  const LearningPathScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final listings = ref.watch(coursesProvider).valueOrNull ?? const [];
    final completed = ref.watch(
      courseCompletionProvider.select((s) => s.completed),
    );
    final units = pathUnitsOf(listings);
    final nextId = listings
        .where((l) => !completed.contains(l.id))
        .map((l) => l.id)
        .firstOrNull;

    return Scaffold(
      key: const Key('path-screen'),
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(l10n.pathTitle),
      ),
      body: SafeArea(
        child: ListView.builder(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
          itemCount: units.length,
          itemBuilder: (context, i) => _UnitSection(
            unit: units[i],
            glyph: _unitGlyphs[i % _unitGlyphs.length],
            completed: completed,
            nextId: nextId,
            lang: lang,
          ),
        ),
      ),
    );
  }
}

class _UnitSection extends StatelessWidget {
  const _UnitSection({
    required this.unit,
    required this.glyph,
    required this.completed,
    required this.nextId,
    required this.lang,
  });

  final PathUnit unit;
  final String glyph;
  final Set<String> completed;
  final String? nextId;
  final String lang;

  @override
  Widget build(BuildContext context) {
    final done = unit.lessons.where((l) => completed.contains(l.id)).length;
    final total = unit.lessons.length;
    final title = resolveInline(unit.title, lang);
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
          ),
          child: Row(
            children: [
              Text(
                glyph,
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
        for (var i = 0; i < unit.lessons.length; i++)
          _LessonNode(
            listing: unit.lessons[i],
            state: completed.contains(unit.lessons[i].id)
                ? _NodeState.done
                : (unit.lessons[i].id == nextId
                      ? _NodeState.next
                      : _NodeState.later),
            // A gentle meander keeps the path playful without a custom canvas.
            indent: 12.0 + 28.0 * ((i % 4 < 2) ? (i % 4) : (3 - i % 4)),
            lang: lang,
          ),
      ],
    );
  }
}

enum _NodeState { done, next, later }

class _LessonNode extends StatefulWidget {
  const _LessonNode({
    required this.listing,
    required this.state,
    required this.indent,
    required this.lang,
  });

  final CourseListing listing;
  final _NodeState state;
  final double indent;
  final String lang;

  @override
  State<_LessonNode> createState() => _LessonNodeState();
}

class _LessonNodeState extends State<_LessonNode>
    with SingleTickerProviderStateMixin {
  // Eager, like ScoreChip's controllers: a lazy late-final controller crashes
  // in dispose when the node was never built visible.
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1400),
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
  }

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final title = resolveInline(widget.listing.title, widget.lang);
    final isNext = widget.state == _NodeState.next;
    final isDone = widget.state == _NodeState.done;
    final circleColor = switch (widget.state) {
      _NodeState.done => CymbraColors.tertiary,
      _NodeState.next => CymbraColors.primary,
      _NodeState.later => CymbraColors.surfaceContainerHigh,
    };
    return Padding(
      padding: EdgeInsets.only(left: widget.indent, top: 6, bottom: 6),
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
                  child: Icon(
                    isDone
                        ? Icons.check
                        : (isNext ? Icons.play_arrow : Icons.music_note),
                    size: 20,
                    color: isDone || isNext
                        ? CymbraColors.surfaceContainerLowest
                        : CymbraColors.onSurfaceVariant.withValues(alpha: 0.6),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 230),
                child: Text(
                  title,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    color: widget.state == _NodeState.later
                        ? CymbraColors.onSurfaceVariant
                        : CymbraColors.onSurface,
                    fontSize: 14,
                    fontWeight: isNext ? FontWeight.w800 : FontWeight.w600,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
