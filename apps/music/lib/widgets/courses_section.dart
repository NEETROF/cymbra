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
import '../screens/lesson_player_screen.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The home-screen "Courses" section (change: add-notation-courses): a compact
/// row of course tiles pinned **above** the favorites, each with a completion
/// indicator, opening the lesson player on tap. It **omits itself entirely**
/// when there are no courses (loading, error, or empty), so it never blocks or
/// crowds the favorites below.
///
/// Courses arrive from [coursesProvider] already ordered by track then level, so
/// the row is grouped in that order; a per-tile chip shows the level.
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

    return Column(
      key: const Key('courses-section'),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
          child: Text(
            l10n.coursesSectionTitle,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 18,
              fontWeight: FontWeight.w800,
            ),
          ),
        ),
        SizedBox(
          height: 116,
          child: ListView.separated(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            itemCount: courses.length,
            separatorBuilder: (_, _) => const SizedBox(width: 12),
            itemBuilder: (context, i) => _CourseTile(
              listing: courses[i],
              completed: completed.contains(courses[i].id),
              lang: lang,
              l10n: l10n,
            ),
          ),
        ),
      ],
    );
  }
}

class _CourseTile extends StatelessWidget {
  const _CourseTile({
    required this.listing,
    required this.completed,
    required this.lang,
    required this.l10n,
  });

  final CourseListing listing;
  final bool completed;
  final String lang;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final title = resolveInline(listing.title, lang);
    return Semantics(
      button: true,
      label: completed ? '$title, ${l10n.courseCompletedLabel}' : title,
      child: InkWell(
        key: Key('course-tile-${listing.id}'),
        borderRadius: BorderRadius.circular(12),
        onTap: () => openLessonPlayer(context, listing.id),
        child: Container(
          width: 156,
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: completed
                  ? CymbraColors.tertiary.withValues(alpha: 0.6)
                  : CymbraColors.onSurfaceVariant.withValues(alpha: 0.15),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(
                    Icons.school_outlined,
                    size: 18,
                    color: CymbraColors.secondary,
                  ),
                  const Spacer(),
                  if (completed)
                    const Icon(
                      Icons.check_circle,
                      size: 18,
                      color: CymbraColors.tertiary,
                    ),
                ],
              ),
              const Spacer(),
              Text(
                title,
                maxLines: 3,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  height: 1.2,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
