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
import '../screens/learning_path_screen.dart';
import '../screens/lesson_player_screen.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The home-screen "Courses" section (change: add-notation-courses): one
/// **continue card** carrying the learner's next lesson — title, unit, unit
/// progress — plus the entry into the full learning path. Pinned above the
/// favorites; **omits itself entirely** when there are no courses (loading,
/// error, or empty), so it never blocks or crowds the favorites below.
///
/// With a 40+ lesson curriculum a flat tile rail stopped scaling; the section
/// now answers the only home-screen question that matters — "where was I?" —
/// in one tap, and defers browsing to the path screen.
class CoursesSection extends ConsumerWidget {
  const CoursesSection({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final courses = ref.watch(coursesProvider).valueOrNull ?? const [];
    if (courses.isEmpty) return const SizedBox.shrink();

    final l10n = AppLocalizations.of(context);
    final completed = ref.watch(
      courseCompletionProvider.select((s) => s.completed),
    );
    final lang = Localizations.localeOf(context).languageCode;

    // The next uncompleted lesson in catalogue order — or the first lesson
    // again when everything is done (courses stay replayable).
    final next = courses.firstWhere(
      (c) => !completed.contains(c.id),
      orElse: () => courses.first,
    );
    final unitLessons = courses
        .where((c) => c.unit == next.unit && c.level == next.level)
        .toList();
    final unitDone = unitLessons.where((c) => completed.contains(c.id)).length;
    final unitTitle = resolveInline(next.unitTitle, lang);

    return Column(
      key: const Key('courses-section'),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 8, 4),
          child: Row(
            children: [
              Text(
                l10n.coursesSectionTitle,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 18,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const Spacer(),
              TextButton(
                key: const Key('courses-see-path'),
                onPressed: () => openLearningPath(context),
                child: Text(
                  l10n.coursesSeeAll,
                  style: const TextStyle(color: CymbraColors.secondary),
                ),
              ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Semantics(
            button: true,
            label: resolveInline(next.title, lang),
            child: InkWell(
              key: const Key('courses-continue-card'),
              borderRadius: BorderRadius.circular(14),
              onTap: () => openLessonPlayer(context, next.id),
              child: Container(
                width: double.infinity,
                padding: const EdgeInsets.all(14),
                decoration: BoxDecoration(
                  color: CymbraColors.surfaceContainerHigh,
                  borderRadius: BorderRadius.circular(14),
                  border: Border.all(
                    color: CymbraColors.primary.withValues(alpha: 0.35),
                  ),
                ),
                child: Row(
                  children: [
                    Container(
                      width: 46,
                      height: 46,
                      decoration: BoxDecoration(
                        color: CymbraColors.primary.withValues(alpha: 0.15),
                        shape: BoxShape.circle,
                      ),
                      child: const Icon(
                        Icons.play_arrow,
                        color: CymbraColors.primary,
                        size: 26,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          if (unitTitle.isNotEmpty)
                            Text(
                              unitTitle,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: const TextStyle(
                                color: CymbraColors.onSurfaceVariant,
                                fontSize: 12,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          Text(
                            resolveInline(next.title, lang),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(
                              color: CymbraColors.onSurface,
                              fontSize: 15,
                              fontWeight: FontWeight.w700,
                              height: 1.2,
                            ),
                          ),
                          if (unitLessons.length > 1) ...[
                            const SizedBox(height: 8),
                            ClipRRect(
                              borderRadius: BorderRadius.circular(6),
                              child: LinearProgressIndicator(
                                value: unitDone / unitLessons.length,
                                minHeight: 6,
                                backgroundColor: CymbraColors.background,
                                color: CymbraColors.primary,
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
