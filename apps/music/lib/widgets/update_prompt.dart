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
import '../services/update/desktop_update_service.dart';
import '../services/update/update_manifest.dart';
import '../state/update_notifier.dart';
import '../state/update_state.dart';

/// Human-readable byte size. Deliberately coarse: the prompt exists to let
/// someone on a metered connection decide, and "48 MB" answers that — a byte
/// count does not.
String formatUpdateSize(int bytes) {
  const mb = 1024 * 1024;
  if (bytes >= mb) {
    final value = bytes / mb;
    return '${value < 10 ? value.toStringAsFixed(1) : value.round()} MB';
  }
  final kb = (bytes / 1024).ceil();
  return '$kb KB';
}

/// Maps a failure cause to a localized message.
///
/// Never a status code, an exception or a transport string: the cause is logged,
/// the user reads something they can act on (memory:
/// no-raw-technical-errors-in-ui).
String updateFailureMessage(AppLocalizations l10n, UpdateFailureCause cause) =>
    switch (cause) {
      UpdateFailureCause.network => l10n.updateErrorNetwork,
      UpdateFailureCause.verification => l10n.updateErrorVerification,
      UpdateFailureCause.integrity => l10n.updateErrorIntegrity,
      UpdateFailureCause.storage => l10n.updateErrorStorage,
    };

/// The update prompt (change: add-desktop-auto-update, tasks 8.7/8.8).
///
/// One dialog for the whole sequence — offer, progress, install, failure — so
/// the user never watches a dialog close and another open. It renders from the
/// state union and calls notifier methods; it never awaits an action's return.
class UpdatePromptDialog extends ConsumerWidget {
  const UpdatePromptDialog({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(updateProvider);
    return switch (state) {
      UpdateAvailableState(
        :final manifest,
        :final target,
        :final canSelfInstall,
      ) =>
        _offer(context, ref, l10n, manifest, target, canSelfInstall),
      UpdateDownloading(:final received, :final total) => _progress(
        context,
        l10n,
        received,
        total,
      ),
      UpdateReady() || UpdateInstalling() => _installing(l10n),
      UpdateFailed(:final cause) => _failure(context, ref, l10n, cause),
      // Every other state means the dialog should already be closing; render an
      // empty shell rather than throwing during the pop frame.
      _ => const SizedBox.shrink(),
    };
  }

  Widget _offer(
    BuildContext context,
    WidgetRef ref,
    AppLocalizations l10n,
    UpdateManifest manifest,
    UpdateTarget target,
    bool canSelfInstall,
  ) {
    final notes = manifest.notesUrl;
    return AlertDialog(
      key: const Key('update-prompt'),
      title: Text(l10n.updateAvailableTitle),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.updateAvailableBody(
              manifest.version.display,
              formatUpdateSize(target.size),
            ),
          ),
          if (!canSelfInstall) ...[
            const SizedBox(height: 12),
            Text(l10n.updateNotSelfInstallableBody),
          ],
          if (notes != null) ...[
            const SizedBox(height: 8),
            TextButton(
              key: const Key('update-release-notes'),
              onPressed: () => _open(ref, notes),
              child: Text(l10n.updateReleaseNotes),
            ),
          ],
        ],
      ),
      actions: [
        TextButton(
          key: const Key('update-skip'),
          onPressed: () => ref.read(updateProvider.notifier).skipCurrentOffer(),
          child: Text(l10n.updateActionSkip),
        ),
        TextButton(
          key: const Key('update-later'),
          onPressed: () => ref.read(updateProvider.notifier).dismiss(),
          child: Text(l10n.updateActionLater),
        ),
        FilledButton(
          key: const Key('update-confirm'),
          onPressed: canSelfInstall
              // Fire the action; the dialog re-renders from the resulting state.
              ? () => ref.read(updateProvider.notifier).download()
              : () {
                  _open(ref, notes ?? target.url);
                  ref.read(updateProvider.notifier).dismiss();
                },
          child: Text(
            canSelfInstall
                ? l10n.updateActionUpdate
                : l10n.updateActionDownloadPage,
          ),
        ),
      ],
    );
  }

  Widget _progress(
    BuildContext context,
    AppLocalizations l10n,
    int received,
    int total,
  ) => AlertDialog(
    key: const Key('update-progress'),
    title: Text(l10n.updateDownloadingTitle),
    content: Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LinearProgressIndicator(
          value: total > 0 ? (received / total).clamp(0.0, 1.0) : null,
        ),
        const SizedBox(height: 12),
        Text(
          l10n.updateDownloadingProgress(
            formatUpdateSize(received),
            formatUpdateSize(total),
          ),
        ),
      ],
    ),
  );

  Widget _installing(AppLocalizations l10n) => AlertDialog(
    key: const Key('update-installing'),
    title: Text(l10n.updateDownloadingTitle),
    content: Text(l10n.updateInstallingBody),
  );

  Widget _failure(
    BuildContext context,
    WidgetRef ref,
    AppLocalizations l10n,
    UpdateFailureCause cause,
  ) => AlertDialog(
    key: const Key('update-failure'),
    title: Text(l10n.updateAvailableTitle),
    content: Text(updateFailureMessage(l10n, cause)),
    actions: [
      TextButton(
        key: const Key('update-failure-ok'),
        onPressed: () => ref.read(updateProvider.notifier).acknowledgeFailure(),
        child: Text(MaterialLocalizations.of(context).okButtonLabel),
      ),
    ],
  );

  /// Opens an external page through the app's single URL seam (the same one the
  /// legal links use), so no widget touches `url_launcher` directly.
  void _open(WidgetRef ref, String url) {
    final uri = Uri.tryParse(url);
    if (uri == null) return;
    ref.read(legalLinkLauncherProvider).open(uri);
  }
}
