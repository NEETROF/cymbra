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
import '../state/home_door.dart';
import '../state/learn_door_access.dart';
import '../theme/cymbra_theme.dart';

/// The Play / Learn door switcher in the home header (change:
/// add-music-learn-play-home), modelled on the instrument switcher: sticky,
/// always in view, one tap to change. Absent when the two-door home is switched
/// off by its flag. Under the drums context it offers **Play only** — the Learn
/// door is keyboard-only — so the control stays where the eye expects it
/// without promising a path that does not exist for drums yet.
///
/// The segments carry their words — "Play", "Learn" — wherever the header has
/// the width: the Learn door is the way into the path, and an unlabelled
/// school icon was not read as one (found while dogfooding on a landscape
/// phone, which has plenty of room). Only a genuinely narrow header (under
/// 600 px) drops to icon-only segments, the labels kept for assistive tech.
class HomeDoorSwitcher extends ConsumerWidget {
  const HomeDoorSwitcher({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (!ref.watch(learnDoorEnabledProvider)) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    final learnAvailable = ref.watch(learnDoorAvailableProvider);
    final current = ref.watch(effectiveHomeDoorProvider);
    final compact = MediaQuery.sizeOf(context).width < 600;

    // Compact: the icon alone, carrying the label for assistive tech (no
    // tooltip — Material pre-builds a tooltip's text in the tree, which would
    // defeat the point of hiding the label).
    ButtonSegment<HomeDoor> segment(
      HomeDoor door,
      String label,
      IconData icon,
    ) => ButtonSegment(
      value: door,
      icon: Icon(icon, size: 18, semanticLabel: compact ? label : null),
      label: compact ? null : Text(label),
    );

    return SegmentedButton<HomeDoor>(
      key: const Key('home-door-switcher'),
      style: ButtonStyle(
        visualDensity: VisualDensity.compact,
        backgroundColor: WidgetStateProperty.resolveWith(
          (s) => s.contains(WidgetState.selected)
              ? CymbraColors.primaryContainer
              : CymbraColors.surfaceContainerHigh,
        ),
        // Explicit: the surrounding MaterialApp theme is light, so the default
        // label colour disappears on these dark surfaces.
        foregroundColor: WidgetStateProperty.resolveWith(
          (s) => s.contains(WidgetState.selected)
              ? CymbraColors.onSurface
              : CymbraColors.onSurfaceVariant,
        ),
      ),
      segments: [
        segment(HomeDoor.play, l10n.homeDoorPlay, Icons.play_circle_outline),
        if (learnAvailable)
          segment(HomeDoor.learn, l10n.homeDoorLearn, Icons.school_outlined),
      ],
      selected: {current},
      onSelectionChanged: (s) =>
          ref.read(homeDoorChoiceProvider.notifier).select(s.first),
      showSelectedIcon: false,
    );
  }
}
