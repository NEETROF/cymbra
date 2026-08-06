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
import '../state/coaching_notifier.dart';
import 'coach_copy.dart';
import 'coach_mark.dart';

/// Renders the **guided player sequence** (D8) above every route.
///
/// It is mounted from `MaterialApp.builder`, so the spotlight can point at
/// controls that live inside the pre-play setup dialog. It draws nothing unless
/// the coaching controller has an active step, and it never blocks: the
/// highlighted control stays tappable through the cut-out and Skip ends the
/// sequence at any point.
class CoachLayer extends ConsumerStatefulWidget {
  const CoachLayer({super.key});

  @override
  ConsumerState<CoachLayer> createState() => _CoachLayerState();
}

class _CoachLayerState extends ConsumerState<CoachLayer> {
  /// The spotlighted control's rect for the current step; `null` until it is
  /// resolved after layout (or when the control is not on screen at all, in
  /// which case the bubble is shown centred instead).
  Rect? _hole;
  PlayerCoachStep? _resolvedFor;

  /// Rects only exist post-layout, so resolve the target after the frame the
  /// step became active — and once more on the following frame, since the
  /// surface holding the control may still be animating in.
  void _resolve(PlayerCoachStep step, {int retries = 6}) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || ref.read(coachingProvider).step != step) return;
      final rect = ref.read(coachTargetRegistryProvider).rectFor(step.anchor);
      if (rect == null && retries > 0) {
        _resolve(step, retries: retries - 1);
        return;
      }
      if (rect != _hole) setState(() => _hole = rect);
    });
  }

  @override
  Widget build(BuildContext context) {
    final step = ref.watch(coachingProvider.select((s) => s.step));
    if (step == null) {
      _resolvedFor = null;
      return const SizedBox.shrink();
    }
    if (step != _resolvedFor) {
      _resolvedFor = step;
      _hole = null;
      _resolve(step);
    }

    final l10n = AppLocalizations.of(context);
    final coaching = ref.read(coachingProvider.notifier);
    final copy = playerCoachCopy(l10n, step);
    final last = step.next == null;
    return Positioned.fill(
      child: CoachMarkOverlay(
        key: Key('coach-step-${step.name}'),
        hole: _hole,
        title: copy.title,
        body: copy.body,
        stepLabel: l10n.coachStepOf(
          step.index + 1,
          PlayerCoachStep.values.length,
        ),
        nextLabel: last ? l10n.coachDone : l10n.coachNext,
        skipLabel: last ? null : l10n.coachSkip,
        onNext: coaching.nextStep,
        onSkip: coaching.skipTour,
        // "Do it now": the control under the cut-out stays live, so the user can
        // change the setting while the hint is up — it guides, it never gates.
        passThrough: true,
      ),
    );
  }
}
