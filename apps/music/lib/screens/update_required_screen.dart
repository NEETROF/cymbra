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
import '../services/legal_links.dart';
import '../state/update_notifier.dart';
import '../state/update_state.dart';
import '../widgets/update_prompt.dart';

/// The blocking forced-update screen (change: add-desktop-auto-update, task 8.9).
///
/// Shown when the running build is below the manifest's `min_supported_version`.
/// Blocking on purpose: the alternative is a client that keeps failing against
/// the backend with errors the user cannot act on. There is no dismiss — the
/// only way out is installing, or closing the app.
///
/// It replaces the app subtree rather than pushing a route, so no navigation
/// (a deep link, a back gesture, a dialog already on screen) can get behind it.
class UpdateRequiredScreen extends ConsumerWidget {
  const UpdateRequiredScreen({required this.state, super.key});

  final UpdateRequired state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final live = ref.watch(updateProvider);
    final notes = state.manifest.notesUrl;
    return Scaffold(
      key: const Key('update-required'),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: Padding(
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.system_update,
                  size: 48,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(height: 16),
                Text(
                  l10n.updateRequiredTitle,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 12),
                Text(l10n.updateRequiredBody(state.manifest.version.display)),
                if (!state.canSelfInstall) ...[
                  const SizedBox(height: 12),
                  Text(l10n.updateNotSelfInstallableBody),
                ],
                const SizedBox(height: 24),
                ..._progressOrAction(context, ref, l10n, live, notes),
              ],
            ),
          ),
        ),
      ),
    );
  }

  List<Widget> _progressOrAction(
    BuildContext context,
    WidgetRef ref,
    AppLocalizations l10n,
    UpdateState live,
    String? notes,
  ) => switch (live) {
    UpdateDownloading(:final received, :final total) => [
      LinearProgressIndicator(
        value: total > 0 ? (received / total).clamp(0.0, 1.0) : null,
      ),
      const SizedBox(height: 8),
      Text(
        l10n.updateDownloadingProgress(
          formatUpdateSize(received),
          formatUpdateSize(total),
        ),
      ),
    ],
    UpdateReady() || UpdateInstalling() => [
      const LinearProgressIndicator(),
      const SizedBox(height: 8),
      Text(l10n.updateInstallingBody),
    ],
    // A failure here is not dismissible either: the action stays on screen so
    // the user can retry, with the reason above it.
    UpdateFailed(:final cause) => [
      Text(
        updateFailureMessage(l10n, cause),
        style: TextStyle(color: Theme.of(context).colorScheme.error),
      ),
      const SizedBox(height: 12),
      ..._action(ref, l10n, notes),
    ],
    _ => _action(ref, l10n, notes),
  };

  List<Widget> _action(WidgetRef ref, AppLocalizations l10n, String? notes) => [
    FilledButton(
      key: const Key('update-required-action'),
      onPressed: state.canSelfInstall
          ? () => ref.read(updateProvider.notifier).download()
          : () {
              final url = Uri.tryParse(notes ?? state.target.url);
              if (url != null) ref.read(legalLinkLauncherProvider).open(url);
            },
      child: Text(
        state.canSelfInstall
            ? l10n.updateActionUpdate
            : l10n.updateActionDownloadPage,
      ),
    ),
  ];
}
