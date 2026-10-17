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

import '../l10n/gen/app_localizations.dart';
import '../services/course_catalog_service.dart';
import '../theme/cymbra_theme.dart';
import 'courses_section.dart';
import 'daily_session_block.dart';
import 'learning_path_body.dart';
import 'placement_offer_card.dart';

/// The Learn door's body (change: add-music-learn-play-home): today's session
/// (change: add-music-daily-session), the continue card,
/// then the whole learning path by unit — the path is the home here, not a
/// route the learner has to find. Works signed out exactly like the courses
/// card does (completion local until sign-in).
class LearnDoorBody extends ConsumerWidget {
  const LearnDoorBody({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final courses = ref.watch(coursesProvider);
    return CustomScrollView(
      key: const Key('learn-door-body'),
      slivers: [
        // The placement offer, at the first visit (change:
        // add-music-placement-test) — first, because "where do I start?" is
        // the question of a first visit, and today's session would otherwise
        // fill a landscape phone and push the offer below the fold (found
        // while dogfooding).
        const SliverToBoxAdapter(child: PlacementOfferCard()),
        // Today's session (change: add-music-daily-session).
        const SliverToBoxAdapter(
          child: KeyedSubtree(
            key: Key('daily-session-slot'),
            child: DailySessionBlock(),
          ),
        ),
        switch (courses) {
          AsyncData(:final value) when value.isEmpty => SliverFillRemaining(
            hasScrollBody: false,
            child: _Empty(l10n: l10n),
          ),
          AsyncData() => SliverList.list(
            children: [
              // The path sits right below, so the card drops its own link to it.
              const CoursesSection(showPathLink: false),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 20, 8, 0),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.pathTitle,
                        style: const TextStyle(
                          color: CymbraColors.primary,
                          fontSize: 15,
                          fontWeight: FontWeight.w800,
                          letterSpacing: 0.5,
                        ),
                      ),
                    ),
                    // The placement stays reachable from the path header.
                    const PlacementEntryButton(),
                  ],
                ),
              ),
              const Padding(
                padding: EdgeInsets.fromLTRB(16, 0, 16, 24),
                // The continue card above carries the lesson's hero; the nodes
                // must not carry the same tag on the same surface.
                child: LearningPathBody(heroes: false),
              ),
            ],
          ),
          AsyncError() => SliverFillRemaining(
            hasScrollBody: false,
            child: _Empty(l10n: l10n),
          ),
          _ => const SliverFillRemaining(
            hasScrollBody: false,
            child: Center(child: CircularProgressIndicator()),
          ),
        },
      ],
    );
  }
}

/// No courses to show (none published, or unreachable with no cache): a quiet
/// statement, never an error — the Play door is one tap away.
class _Empty extends StatelessWidget {
  const _Empty({required this.l10n});

  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    return Center(
      key: const Key('learn-door-empty'),
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(
              Icons.school_outlined,
              size: 56,
              color: CymbraColors.outline,
            ),
            const SizedBox(height: 16),
            Text(
              l10n.learnDoorEmpty,
              textAlign: TextAlign.center,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ],
        ),
      ),
    );
  }
}
