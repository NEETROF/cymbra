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
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../screens/lesson_player_screen.dart';
import '../screens/open_score.dart';
import '../state/daily_session_core.dart';
import '../state/daily_session_notifier.dart';
import '../state/rating_activity_notifier.dart' show nowFnProvider;
import '../state/session_notifier.dart';
import '../state/streak_notifier.dart';
import '../state/today_ledger.dart';
import '../theme/cymbra_theme.dart';
import '../state/course_completion_notifier.dart';
import '../state/piece_stars_core.dart';
import 'animated_counter.dart';
import 'piece_run.dart';
import 'star_row.dart';

/// Today's session at the top of the Learn door (change:
/// add-music-daily-session): the items, the ring over the goal, the streak as
/// the server reports it, and the goal choice. Hidden while the catalogue
/// loads and when there are no courses. Owns the midnight timer that
/// refreshes the local day, so a door left open overnight recomposes.
class DailySessionBlock extends ConsumerStatefulWidget {
  const DailySessionBlock({super.key});

  @override
  ConsumerState<DailySessionBlock> createState() => _DailySessionBlockState();
}

class _DailySessionBlockState extends ConsumerState<DailySessionBlock> {
  Timer? _midnight;

  @override
  void initState() {
    super.initState();
    _armMidnight();
  }

  void _armMidnight() {
    final now = ref.read(nowFnProvider)();
    final wait =
        nextLocalMidnight(now).difference(now) + const Duration(seconds: 1);
    _midnight = Timer(wait, () {
      if (!mounted) return;
      ref.read(localDayProvider.notifier).refresh();
      _armMidnight();
    });
  }

  @override
  void dispose() {
    _midnight?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final session = ref.watch(dailySessionProvider);
    return switch (session) {
      AsyncData(:final value) when value.items.isNotEmpty => _Card(
        session: value,
      ),
      _ => const SizedBox.shrink(),
    };
  }
}

class _Card extends ConsumerWidget {
  const _Card({required this.session});

  final DailySession session;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    return Container(
      key: const Key('daily-session-block'),
      margin: const EdgeInsets.fromLTRB(16, 12, 16, 4),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: session.completedToday
              ? CymbraColors.tertiary.withValues(alpha: 0.6)
              : CymbraColors.onSurfaceVariant.withValues(alpha: 0.15),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.dailySessionTitle.toUpperCase(),
                      style: const TextStyle(
                        color: CymbraColors.primary,
                        fontSize: 13,
                        fontWeight: FontWeight.w800,
                        letterSpacing: 0.8,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      l10n.dailySessionAbout,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                        fontSize: 13,
                      ),
                    ),
                    const SizedBox(height: 6),
                    const _StreakLine(),
                  ],
                ),
              ),
              const SizedBox(width: 12),
              _Ring(done: session.doneCount, goal: session.goal),
            ],
          ),
          const SizedBox(height: 12),
          for (final item in session.items)
            _ItemRow(item: item, lang: lang, l10n: l10n),
          const SizedBox(height: 8),
          _GoalRow(goal: session.goal, l10n: l10n),
        ],
      ),
    );
  }
}

/// The streak as the server reports it — read, never computed here (D4);
/// absent signed out.
class _StreakLine extends ConsumerWidget {
  const _StreakLine();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return const SizedBox.shrink();
    }
    final streak = ref.watch(streakProvider).valueOrNull;
    if (streak == null) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    final text = streak.current == 0
        ? l10n.dailyStreakNone
        : '${l10n.dailyStreakDays(streak.current)} · '
              '${streak.playedToday ? l10n.dailyStreakCovered : l10n.dailyStreakNotYet}';
    return Row(
      key: const Key('daily-streak'),
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          Icons.local_fire_department,
          size: 16,
          color: streak.playedToday
              ? CymbraColors.tertiary
              : CymbraColors.onSurfaceVariant,
        ),
        const SizedBox(width: 4),
        Flexible(
          child: Text(
            text,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 12.5,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ],
    );
  }
}

/// Items done today over the goal; the count counts up (change:
/// add-music-motion-and-celebrations).
class _Ring extends StatelessWidget {
  const _Ring({required this.done, required this.goal});

  final int done;
  final DailyGoal goal;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final progress = (done / goal.count).clamp(0.0, 1.0);
    return Column(
      key: const Key('daily-ring'),
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: 60,
          height: 60,
          child: TweenAnimationBuilder<double>(
            tween: Tween(end: progress),
            duration: const Duration(milliseconds: 280),
            curve: Curves.easeOutCubic,
            builder: (context, p, child) => CustomPaint(
              painter: _RingPainter(
                progress: p,
                color: done >= goal.count
                    ? CymbraColors.tertiary
                    : CymbraColors.primary,
                track: CymbraColors.background,
              ),
              child: child,
            ),
            child: Center(
              child: AnimatedCounter(
                value: done,
                format: (v) => '$v/${goal.count}',
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 15,
                  fontWeight: FontWeight.w800,
                ),
              ),
            ),
          ),
        ),
        const SizedBox(height: 2),
        Text(
          l10n.dailySessionToday,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 11,
          ),
        ),
      ],
    );
  }
}

class _RingPainter extends CustomPainter {
  const _RingPainter({
    required this.progress,
    required this.color,
    required this.track,
  });

  final double progress;
  final Color color;
  final Color track;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    final r = rect.deflate(4);
    final base = Paint()
      ..color = track
      ..strokeWidth = 6
      ..style = PaintingStyle.stroke;
    canvas.drawArc(r, 0, 2 * math.pi, false, base);
    if (progress > 0) {
      final fill = Paint()
        ..color = color
        ..strokeWidth = 6
        ..strokeCap = StrokeCap.round
        ..style = PaintingStyle.stroke;
      canvas.drawArc(r, -math.pi / 2, 2 * math.pi * progress, false, fill);
    }
  }

  @override
  bool shouldRepaint(_RingPainter old) =>
      old.progress != progress || old.color != color || old.track != track;
}

class _ItemRow extends ConsumerWidget {
  const _ItemRow({required this.item, required this.lang, required this.l10n});

  final DailyItem item;
  final String lang;
  final AppLocalizations l10n;

  Future<void> _open(BuildContext context, WidgetRef ref) async {
    switch (item.kind) {
      case DailyItemKind.lesson || DailyItemKind.replay:
        openLessonPlayer(context, item.courseId);
      case DailyItemKind.piece:
        final block = item.piece;
        if (block == null) {
          openLessonPlayer(context, item.courseId);
          return;
        }
        final outcome = await runPieceStep(
          context,
          ref,
          courseId: item.courseId,
          lessonTitle: resolveInline(item.subtitle, lang),
          block: block,
        );
        // Never traps: a piece that cannot open sends the learner to the
        // lesson that holds it instead.
        if (outcome != PieceOpenOutcome.opened && context.mounted) {
          openLessonPlayer(context, item.courseId);
        }
      case DailyItemKind.review:
        final block = item.block;
        if (block == null) {
          openLessonPlayer(context, item.courseId);
          return;
        }
        openReviewLesson(
          context,
          ReviewRun(
            key: item.key,
            manifest: CourseManifest(
              schemaVersion: kCourseSchemaVersion,
              id: item.key,
              title: item.subtitle,
              blocks: [block],
            ),
          ),
        );
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final kindLabel = switch (item.kind) {
      DailyItemKind.lesson => l10n.dailyItemLesson,
      DailyItemKind.piece => l10n.dailyItemPiece,
      DailyItemKind.review => l10n.dailyItemReview,
      DailyItemKind.replay => l10n.dailyItemReplay,
    };
    var title = resolveInline(item.title, lang);
    if (title.isEmpty) {
      title = item.kind == DailyItemKind.review
          ? l10n.dailyReviewFallback
          : resolveInline(item.subtitle, lang);
    }
    final subtitle = resolveInline(item.subtitle, lang);
    // A piece's stars from the lesson's best run (change:
    // add-music-piece-stars); empty before any run.
    final bestPct = item.kind == DailyItemKind.piece
        ? ref.watch(
            courseCompletionProvider.select((s) => s.bestPct[item.courseId]),
          )
        : null;
    return InkWell(
      key: Key('daily-item-${item.kind.name}'),
      borderRadius: BorderRadius.circular(10),
      onTap: () => _open(context, ref),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 6),
        child: Row(
          children: [
            SizedBox(
              width: 76,
              child: Text(
                kindLabel.toUpperCase(),
                style: TextStyle(
                  color: item.done
                      ? CymbraColors.tertiary
                      : CymbraColors.onSurfaceVariant,
                  fontSize: 11,
                  fontWeight: FontWeight.w800,
                  letterSpacing: 0.6,
                ),
              ),
            ),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 14,
                      fontWeight: FontWeight.w700,
                      decoration: item.done
                          ? TextDecoration.lineThrough
                          : TextDecoration.none,
                    ),
                  ),
                  if (subtitle.isNotEmpty && subtitle != title)
                    Text(
                      subtitle,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                        fontSize: 12,
                      ),
                    ),
                  if (item.kind == DailyItemKind.piece)
                    Padding(
                      padding: const EdgeInsets.only(top: 2),
                      child: StarRow(
                        key: const Key('daily-piece-stars'),
                        stars: starsFor(bestPct),
                      ),
                    ),
                ],
              ),
            ),
            const SizedBox(width: 8),
            if (item.done)
              Row(
                key: Key('daily-done-${item.kind.name}'),
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    l10n.dailyItemDone,
                    style: const TextStyle(
                      color: CymbraColors.tertiary,
                      fontSize: 12,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  const SizedBox(width: 4),
                  const Icon(
                    Icons.check_circle,
                    size: 18,
                    color: CymbraColors.tertiary,
                  ),
                ],
              )
            else
              const Icon(
                Icons.chevron_right,
                size: 20,
                color: CymbraColors.onSurfaceVariant,
              ),
          ],
        ),
      ),
    );
  }
}

/// The goal choice, right in the block (D3): light, normal, full.
class _GoalRow extends ConsumerWidget {
  const _GoalRow({required this.goal, required this.l10n});

  final DailyGoal goal;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    String label(DailyGoal g) => switch (g) {
      DailyGoal.light => l10n.dailyGoalLight,
      DailyGoal.normal => l10n.dailyGoalNormal,
      DailyGoal.full => l10n.dailyGoalFull,
    };
    return Wrap(
      key: const Key('daily-goal'),
      crossAxisAlignment: WrapCrossAlignment.center,
      spacing: 6,
      runSpacing: 4,
      children: [
        Text(
          '${l10n.dailyGoalLabel} :',
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 12,
            fontWeight: FontWeight.w600,
          ),
        ),
        for (final g in DailyGoal.values)
          ChoiceChip(
            key: Key('daily-goal-${g.name}'),
            label: Text(label(g)),
            selected: g == goal,
            visualDensity: VisualDensity.compact,
            labelStyle: const TextStyle(fontSize: 12),
            onSelected: (_) =>
                ref.read(dailyGoalSettingProvider.notifier).set(g),
          ),
      ],
    );
  }
}
