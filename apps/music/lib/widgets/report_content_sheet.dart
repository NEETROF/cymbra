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
import '../services/content_report_service.dart';
import '../state/content_report_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'app_snackbar.dart';

/// Open the content-report sheet for [targetId] (change: add-content-reporting).
///
/// Play's UGC policy requires an in-app way to report objectionable content, so
/// this is reachable from every surface that shows content someone else
/// contributed. The sheet owns nothing: it fires [ContentReport.submit] and lets
/// a listener close it.
Future<void> showReportContentSheet(
  BuildContext context, {
  required ReportTarget target,
  required String targetId,
}) => showModalBottomSheet<void>(
  context: context,
  isScrollControlled: true,
  backgroundColor: CymbraColors.surfaceContainerLowest,
  builder: (_) => _ReportContentSheet(target: target, targetId: targetId),
);

class _ReportContentSheet extends ConsumerStatefulWidget {
  const _ReportContentSheet({required this.target, required this.targetId});

  final ReportTarget target;
  final String targetId;

  @override
  ConsumerState<_ReportContentSheet> createState() =>
      _ReportContentSheetState();
}

class _ReportContentSheetState extends ConsumerState<_ReportContentSheet> {
  ReportReason _reason = ReportReason.copyright;
  final _note = TextEditingController();

  @override
  void dispose() {
    _note.dispose();
    super.dispose();
  }

  String _reasonLabel(AppLocalizations l10n, ReportReason r) => switch (r) {
    ReportReason.copyright => l10n.reportReasonCopyright,
    ReportReason.inappropriate => l10n.reportReasonInappropriate,
    ReportReason.wrongContent => l10n.reportReasonWrongContent,
    ReportReason.other => l10n.reportReasonOther,
  };

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final sending = ref.watch(contentReportProvider.select((s) => s.sending));
    return _ReportOutcomeListener(
      child: Padding(
        // Lift the sheet above the keyboard while the note field has focus.
        padding: EdgeInsets.only(
          left: 20,
          right: 20,
          top: 20,
          bottom: 20 + MediaQuery.of(context).viewInsets.bottom,
        ),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                l10n.reportSheetTitle,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              Text(
                l10n.reportSheetIntro,
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 12),
              // RadioGroup rather than per-tile groupValue/onChanged: those two
              // are deprecated since Flutter 3.32.
              RadioGroup<ReportReason>(
                groupValue: _reason,
                // RadioGroup requires a non-null callback, so the in-flight guard
                // lives in the body rather than in a disabled handler.
                onChanged: (v) {
                  if (sending || v == null) return;
                  setState(() => _reason = v);
                },
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    for (final r in ReportReason.values)
                      RadioListTile<ReportReason>(
                        key: Key('report-reason-${r.wire}'),
                        value: r,
                        title: Text(_reasonLabel(l10n, r)),
                        contentPadding: EdgeInsets.zero,
                        dense: true,
                      ),
                  ],
                ),
              ),
              const SizedBox(height: 8),
              TextField(
                key: const Key('report-note'),
                controller: _note,
                enabled: !sending,
                maxLength: kReportNoteMaxLength,
                maxLines: 3,
                decoration: InputDecoration(
                  labelText: l10n.reportNoteLabel,
                  // The counter only matters near the cap; 2000 characters of
                  // running count is noise on a three-line field.
                  counterText: '',
                ),
              ),
              const SizedBox(height: 12),
              FilledButton(
                key: const Key('report-submit'),
                onPressed: sending
                    ? null
                    : () => ref
                          .read(contentReportProvider.notifier)
                          .submit(
                            target: widget.target,
                            targetId: widget.targetId,
                            reason: _reason,
                            note: _note.text,
                          ),
                child: Text(l10n.reportSubmit),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The sheet's only side-effect site (architecture rule 4): closes the sheet and
/// acknowledges on success, and reports a failure with a localized message —
/// never a raw transport string.
class _ReportOutcomeListener extends ConsumerWidget {
  const _ReportOutcomeListener({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    ref.listen(contentReportProvider.select((s) => s.sentSeq), (prev, next) {
      if (prev == null || next <= prev) return;
      // Capture the overlay BEFORE popping: the sheet's own route is going away,
      // and a toast has to outlive it (a snackbar would be painted under it).
      final overlay = Overlay.of(context, rootOverlay: true);
      Navigator.of(context).maybePop();
      showAppToast(overlay, l10n.reportSent);
    });
    ref.listen(contentReportProvider.select((s) => s.errorSeq), (prev, next) {
      if (prev == null || next <= prev) return;
      showAppToast(Overlay.of(context, rootOverlay: true), l10n.reportFailed);
    });
    return child;
  }
}
