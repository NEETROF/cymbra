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

import '../state/countdown.dart';
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The race-game style pre-start countdown (5…4…3…2…1…GO) shown centred over the
/// player while playback is armed but the playhead is frozen. Each step fades in
/// and scales up, then fades out as the next appears; the GO step disappears when
/// the countdown reaches 0 (before the first note). Renders nothing when there is
/// no active countdown.
class CountdownOverlay extends ConsumerWidget {
  const CountdownOverlay({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final remaining = ref.watch(playerProvider.select((d) => d.countdownMs));
    final label = countdownLabel(remaining);
    if (label == null) return const SizedBox.shrink();

    final isGo = isCountdownGo(remaining);
    final color = isGo ? CymbraColors.tertiary : CymbraColors.primary;

    return IgnorePointer(
      child: Center(
        child: AnimatedSwitcher(
          duration: const Duration(milliseconds: 350),
          transitionBuilder: (child, animation) {
            final scale = Tween<double>(begin: 0.4, end: 1).animate(
              CurvedAnimation(parent: animation, curve: Curves.easeOutBack),
            );
            return FadeTransition(
              opacity: animation,
              child: ScaleTransition(scale: scale, child: child),
            );
          },
          child: Text(
            label,
            // Key by label so each digit/GO animates in as a new child.
            key: ValueKey(label),
            style: TextStyle(
              fontSize: isGo ? 96 : 120,
              fontWeight: FontWeight.w500,
              color: color,
              shadows: [
                Shadow(color: color.withValues(alpha: 0.6), blurRadius: 24),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
