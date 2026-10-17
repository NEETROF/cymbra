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
import '../widgets/learning_path_body.dart';

export '../widgets/learning_path_body.dart' show PathUnit, pathUnitsOf;

/// Opens the full learning path (change: add-notation-courses).
void openLearningPath(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const LearningPathScreen()));

/// The full-screen learning path: the shared [LearningPathBody] under its own
/// app bar. Since add-music-learn-play-home the Learn door shows the same body
/// on the home; this route remains the deep-link target (the continue card's
/// "see the path" in the single-home fallback, the showcase, the captures).
class LearningPathScreen extends StatelessWidget {
  const LearningPathScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      key: const Key('path-screen'),
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(l10n.pathTitle),
      ),
      body: const SafeArea(
        child: SingleChildScrollView(
          padding: EdgeInsets.fromLTRB(16, 12, 16, 24),
          child: LearningPathBody(),
        ),
      ),
    );
  }
}
