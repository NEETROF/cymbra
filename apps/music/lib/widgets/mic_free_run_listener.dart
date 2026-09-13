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
import '../state/input_calibration_notifier.dart';
import '../state/player_notifier.dart';
import 'app_snackbar.dart';

/// Isolates the free-run steering side effect (change:
/// add-acoustic-piano-input, spec: Free-Run Gated On Measured Latency): when
/// starting scored free play with the microphone was redirected to Wait Mode,
/// say so — which of the two reasons applies decides the copy — then
/// acknowledge the flag. A dedicated listener widget, per architecture rule 4.
class MicFreeRunListener extends ConsumerWidget {
  const MicFreeRunListener({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(playerProvider.select((d) => d.micSteeredToWaitMode), (
      _,
      steered,
    ) {
      if (!steered) return;
      final l10n = AppLocalizations.of(context);
      final gate = ref.read(micFreeRunGateProvider);
      showAppToast(
        Overlay.of(context, rootOverlay: true),
        gate == MicFreeRunGate.latencyTooHigh
            ? l10n.micFreeRunLatencyTooHigh
            : l10n.micFreeRunNeedsCalibration,
      );
      ref.read(playerProvider.notifier).acknowledgeMicSteer();
    });
    return child;
  }
}
