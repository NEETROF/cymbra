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
import '../services/audio_capture_service.dart';
import '../src/rust/api/audio_input.dart' show InputRouteKind;
import '../state/input_calibration_notifier.dart';

/// "Microphone calibration" (change: add-acoustic-piano-input): the active
/// capture route, the stored measurement for it, and the run button. Every
/// failure state maps to its own localized guidance — never a raw technical
/// string.
class InputCalibrationSection extends ConsumerWidget {
  const InputCalibrationSection({super.key, this.title});

  /// Section heading builder used by the host so the section matches the ones
  /// around it. Falls back to a plain label.
  final Widget Function(String label)? title;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final calibration = ref.watch(inputCalibrationProvider);
    final running = calibration.status == CalibrationStatus.running;
    final measured = calibration.measuredMs;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        title?.call(l10n.inputCalibrationTitle) ??
            Padding(
              padding: const EdgeInsets.only(top: 12, bottom: 2),
              child: Text(
                l10n.inputCalibrationTitle,
                style: Theme.of(context).textTheme.titleSmall,
              ),
            ),
        Padding(
          padding: const EdgeInsets.only(bottom: 8),
          child: Text(
            l10n.inputCalibrationIntro,
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ),
        Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(_routeLabel(l10n, calibration.route)),
                  Text(
                    measured == null
                        ? l10n.inputCalibrationNotMeasured
                        : l10n.inputCalibrationMeasured(measured.round()),
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
            FilledButton(
              onPressed: running
                  ? null
                  : () => ref
                        .read(inputCalibrationProvider.notifier)
                        .runCalibration(),
              child: Text(l10n.inputCalibrationRun),
            ),
          ],
        ),
        if (_statusMessage(l10n, calibration.status) case final message?)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              message,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: running ? null : Theme.of(context).colorScheme.error,
              ),
            ),
          ),
      ],
    );
  }

  /// The active route as "kind label" (or the kind + name where the name adds
  /// information, e.g. a USB device). Display only — never classification.
  String _routeLabel(AppLocalizations l10n, CaptureRoute? route) {
    final kindLabel = switch (route?.kind) {
      InputRouteKind.builtin || null => l10n.inputRouteKindBuiltin,
      InputRouteKind.wired => l10n.inputRouteKindWired,
      InputRouteKind.usb => l10n.inputRouteKindUsb,
      InputRouteKind.bluetooth => l10n.inputRouteKindBluetooth,
      InputRouteKind.other => l10n.inputRouteKindOther,
    };
    final name = route?.name;
    if (name == null || name.isEmpty || route?.kind == InputRouteKind.builtin) {
      return kindLabel;
    }
    return '$kindLabel — $name';
  }

  String? _statusMessage(
    AppLocalizations l10n,
    CalibrationStatus status,
  ) => switch (status) {
    CalibrationStatus.idle || CalibrationStatus.done => null,
    CalibrationStatus.running => l10n.inputCalibrationRunning,
    CalibrationStatus.notDetected => l10n.inputCalibrationNotDetected,
    CalibrationStatus.permissionDenied => l10n.inputCalibrationPermissionDenied,
    CalibrationStatus.refusedBluetooth => l10n.inputCalibrationBluetoothRefused,
  };
}
