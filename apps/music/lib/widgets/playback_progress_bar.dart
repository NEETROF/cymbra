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
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';

/// A thin, full-width playback progress bar sitting directly above the
/// on-screen keyboard (or at the bottom of the render area when the keyboard
/// is hidden): the track spans the piece's duration and the fill tracks the
/// playhead. Purely informative — wrapped in an [IgnorePointer] so it never
/// intercepts keyboard or gesture input — and hidden when no timed score is
/// loaded. It freezes with the playhead (pause, Wait Mode) since it reads the
/// same position.
class PlaybackProgressBar extends ConsumerWidget {
  const PlaybackProgressBar({super.key});

  static const double _height = 5;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final songEndMs = ref.watch(playerProvider.select((d) => d.songEndMs));
    if (songEndMs <= 0) return const SizedBox.shrink();
    // The *heard* position, not the emission clock: on a delayed output the bar
    // tracks the sound like every other playhead readout (change:
    // add-audio-output-routing). Identical to `elapsedMs` at the default offset.
    final heardMs = ref.watch(playerProvider.select((d) => d.referenceMs));
    final fraction = (heardMs / songEndMs).clamp(0.0, 1.0);

    return IgnorePointer(
      child: Semantics(
        label: AppLocalizations.of(
          context,
        ).playbackProgressLabel((fraction * 100).round()),
        child: SizedBox(
          key: const Key('playback-progress'),
          height: _height,
          width: double.infinity,
          child: ColoredBox(
            color: CymbraColors.surfaceContainerHigh,
            child: FractionallySizedBox(
              alignment: Alignment.centerLeft,
              widthFactor: fraction,
              child: const ColoredBox(color: CymbraColors.secondary),
            ),
          ),
        ),
      ),
    );
  }
}
