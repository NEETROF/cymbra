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

import '../l10n/gen/app_localizations.dart';
import '../theme/cymbra_theme.dart';

/// INTERIM (change: add-drums-access, removed by `add-drum-kit-view`): the
/// "drums not playable yet" state shown instead of the player for a percussion
/// score. Without it the waterfall would draw the drum part's GM key numbers
/// as falling **piano** notes and the piano synth would sound them — which a
/// tester would read as broken, not unbuilt. Deliberately NOT styled as an
/// error: the file is fine, the presentation is not built yet.
///
/// `add-drum-kit-view` deletes this screen and the single predicate that
/// routes here (see `openScore` in `open_score.dart`).
class DrumsNotPlayableScreen extends StatelessWidget {
  const DrumsNotPlayableScreen({super.key, required this.title});

  /// The score's display title, so the user knows what they opened.
  final String title;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(title),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(
                Icons.music_note,
                size: 56,
                color: CymbraColors.primary,
              ),
              const SizedBox(height: 16),
              Text(
                l10n.drumsNotPlayableTitle,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 18,
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                l10n.drumsNotPlayableBody,
                textAlign: TextAlign.center,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
