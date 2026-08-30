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
import '../services/score_upload_service.dart';
import '../state/batch_import_notifier.dart';
import '../state/score_catalog.dart' show PracticeLevel;
import '../theme/cymbra_theme.dart';

/// Batch import (change: add-private-score-catalog): one attestation and one
/// difficulty for the whole selection, then a sequential run whose per-file
/// outcomes land in a result board. A file that fails never stops the others.
class BatchImportScreen extends ConsumerStatefulWidget {
  const BatchImportScreen({super.key});

  @override
  ConsumerState<BatchImportScreen> createState() => _BatchImportScreenState();
}

class _BatchImportScreenState extends ConsumerState<BatchImportScreen> {
  @override
  void initState() {
    super.initState();
    // Read the remaining allowance so the warning can be shown BEFORE any
    // upload runs. Failure is silent (see the notifier): not knowing must not
    // block an import the server may accept.
    Future.microtask(
      () => ref.read(batchImportNotifierProvider.notifier).loadAllowance(),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(batchImportNotifierProvider);
    final notifier = ref.read(batchImportNotifierProvider.notifier);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.batchImportTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            l10n.batchImportSelected(state.files.length),
            style: const TextStyle(fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 16),
          if (state.done)
            _ResultBoard(state: state)
          else if (state.running)
            _Progress(state: state)
          else
            ..._setup(context, l10n, state, notifier),
        ],
      ),
    );
  }

  /// The pre-run form: quota warning, one attestation, one difficulty, start.
  List<Widget> _setup(
    BuildContext context,
    AppLocalizations l10n,
    BatchImportState state,
    BatchImportNotifier notifier,
  ) => [
    if (state.exceedsAllowance) ...[
      _QuotaWarning(state: state),
      const SizedBox(height: 16),
    ],
    Text(l10n.uploadRightsQuestion),
    Text(
      l10n.batchImportRightsNote,
      style: const TextStyle(fontSize: 12, fontStyle: FontStyle.italic),
    ),
    RadioGroup<RightsBasis>(
      groupValue: state.rightsBasis,
      onChanged: (v) {
        if (v != null) notifier.setRightsBasis(v);
      },
      child: Column(
        children: [
          RadioListTile<RightsBasis>(
            value: RightsBasis.author,
            title: Text(l10n.uploadRightsAuthor),
          ),
          RadioListTile<RightsBasis>(
            value: RightsBasis.publicDomain,
            title: Text(l10n.uploadRightsPublicDomain),
          ),
          RadioListTile<RightsBasis>(
            value: RightsBasis.privateUse,
            title: Text(l10n.uploadRightsPrivateUse),
            subtitle: Text(
              l10n.uploadRightsPrivateUseHint,
              style: const TextStyle(fontSize: 12),
            ),
          ),
        ],
      ),
    ),
    CheckboxListTile(
      value: state.rightsAck,
      onChanged: (v) => notifier.setRightsAck(v ?? false),
      controlAffinity: ListTileControlAffinity.leading,
      title: Text(l10n.uploadRightsAck, style: const TextStyle(fontSize: 13)),
    ),
    const SizedBox(height: 16),
    Text(l10n.batchImportLevelQuestion),
    RadioGroup<PracticeLevel>(
      groupValue: state.level,
      onChanged: (v) {
        if (v != null) notifier.setLevel(v);
      },
      child: Column(
        children: [
          for (final level in PracticeLevel.values)
            RadioListTile<PracticeLevel>(
              value: level,
              title: Text(_levelLabel(l10n, level)),
            ),
        ],
      ),
    ),
    const SizedBox(height: 24),
    FilledButton.icon(
      // Fire and react to state — never await the action here.
      onPressed: state.canStart ? () => notifier.run() : null,
      icon: const Icon(Icons.library_add_outlined),
      label: Text(l10n.batchImportStart),
    ),
  ];
}

String _levelLabel(AppLocalizations l10n, PracticeLevel level) =>
    switch (level) {
      PracticeLevel.beginner => l10n.levelBeginner,
      PracticeLevel.intermediate => l10n.levelIntermediate,
      PracticeLevel.advanced => l10n.levelAdvanced,
    };

/// Says, before anything is uploaded, how much of the selection cannot land.
class _QuotaWarning extends StatelessWidget {
  const _QuotaWarning({required this.state});

  final BatchImportState state;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final allowance = state.allowance!;
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerLowest,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: CymbraColors.error),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.batchImportQuotaWarning(
              allowance.remaining,
              state.overAllowanceCount,
            ),
          ),
          if (allowance.upgradeRaisesLimit)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                l10n.batchImportQuotaWarningUpgrade,
                style: const TextStyle(fontSize: 12),
              ),
            ),
        ],
      ),
    );
  }
}

class _Progress extends StatelessWidget {
  const _Progress({required this.state});

  final BatchImportState state;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final total = state.files.length;
    return Column(
      children: [
        LinearProgressIndicator(
          value: total == 0 ? null : state.results.length / total,
        ),
        const SizedBox(height: 12),
        Text(l10n.batchImportRunning(state.currentIndex + 1, total)),
      ],
    );
  }
}

/// Per-file outcomes, in localized, non-technical wording.
class _ResultBoard extends StatelessWidget {
  const _ResultBoard({required this.state});

  final BatchImportState state;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.batchImportDone(state.importedCount, state.files.length),
          style: const TextStyle(fontWeight: FontWeight.w600, fontSize: 16),
        ),
        const SizedBox(height: 8),
        for (final r in state.results)
          ListTile(
            dense: true,
            leading: Icon(
              _icon(r.outcome),
              color: r.outcome == BatchOutcome.imported
                  ? CymbraColors.tertiary
                  : CymbraColors.error,
            ),
            title: Text(r.name),
            subtitle: Text(_outcomeLabel(l10n, r.outcome)),
          ),
        const SizedBox(height: 16),
        FilledButton(
          onPressed: () => Navigator.of(context).maybePop(),
          child: Text(l10n.batchImportClose),
        ),
      ],
    );
  }

  static IconData _icon(BatchOutcome o) => switch (o) {
    BatchOutcome.imported => Icons.check_circle_outline,
    BatchOutcome.duplicate => Icons.library_add_check_outlined,
    BatchOutcome.invalid => Icons.error_outline,
    BatchOutcome.quotaExceeded => Icons.lock_outline,
    BatchOutcome.failed => Icons.refresh,
  };
}

String _outcomeLabel(AppLocalizations l10n, BatchOutcome o) => switch (o) {
  BatchOutcome.imported => l10n.batchImportOutcomeImported,
  BatchOutcome.duplicate => l10n.batchImportOutcomeDuplicate,
  BatchOutcome.invalid => l10n.batchImportOutcomeInvalid,
  BatchOutcome.quotaExceeded => l10n.batchImportOutcomeQuota,
  BatchOutcome.failed => l10n.batchImportOutcomeFailed,
};
