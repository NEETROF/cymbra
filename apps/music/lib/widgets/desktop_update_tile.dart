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
import '../services/app_platform.dart';
import '../services/update/desktop_update_service.dart';
import '../state/update_notifier.dart';
import '../state/update_state.dart';
import 'update_prompt.dart';

/// The manual "check for updates" entry (change: add-desktop-auto-update,
/// task 8.10).
///
/// Desktop-only: on iOS, Android and macOS the store owns updates, and an
/// in-app check there would be both wrong and, on the App Store, a rules
/// problem. Renders nothing at all on those platforms — a greyed-out row would
/// only invite the question.
///
/// A manual check ignores the 24-hour throttle, the rollout bucket and any
/// skipped version: the user asked, so the answer is what the feed offers.
class DesktopUpdateTile extends ConsumerWidget {
  const DesktopUpdateTile({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final platform = ref.watch(appPlatformProvider);
    if (platform != AppPlatform.windows && platform != AppPlatform.linux) {
      return const SizedBox.shrink();
    }
    if (!ref.watch(desktopUpdateEnabledProvider)) {
      return const SizedBox.shrink();
    }
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(updateProvider);
    final version = ref.watch(currentAppVersionProvider).valueOrNull;
    return ListTile(
      key: const Key('profile-check-updates'),
      contentPadding: EdgeInsets.zero,
      leading: const Icon(Icons.system_update_alt),
      title: Text(l10n.updateCheckAction),
      subtitle: Text(_subtitle(l10n, state, version?.display)),
      trailing: state.isBusy
          ? const SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            )
          : const Icon(Icons.chevron_right),
      // Fire the action; the tile and the prompt both react to the state.
      onTap: state.isBusy
          ? null
          : () => ref.read(updateProvider.notifier).checkNow(),
    );
  }

  String _subtitle(AppLocalizations l10n, UpdateState state, String? version) =>
      switch (state) {
        UpdateChecking() => l10n.updateCheckingStatus,
        UpdateUpToDateState() => l10n.updateUpToDateStatus,
        UpdateFailed(:final cause) => updateFailureMessage(l10n, cause),
        // The running build is the useful default: it is what a user is asked
        // for when they report a problem.
        _ => l10n.updateCurrentVersion(version ?? '—'),
      };
}
