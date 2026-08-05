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

import '../l10n/gen/app_localizations.dart';
import '../theme/cymbra_theme.dart';

/// The predefined licence choices offered in the propose dialog — the same set the
/// SoundFont propose dialog uses, so the two flows stay consistent.
const List<String> _scoreProposeLicenses = [
  'CC0-1.0',
  'CC-BY 3.0',
  'CC-BY 4.0',
  'CC-BY-SA 4.0',
];

/// What a successful propose dialog returns: the chosen licence, the optional
/// free-text attribution, and — for a re-proposal of a rejected score — the mandatory
/// justification (change: add-score-catalog-proposal). The right-to-distribute
/// attestation is enforced by the dialog (submit disabled until ticked), so a `null`
/// result = the user cancelled.
typedef ScoreProposeResult = ({
  String license,
  String attribution,
  String? justification,
});

/// Show the propose-a-score dialog. When [rejected] is true, the moderator's
/// [rejectionReason] is shown and a non-empty justification is required before the
/// re-propose can be submitted (it reopens the rejected catalog entry).
Future<ScoreProposeResult?> showScoreProposeDialog(
  BuildContext context, {
  bool rejected = false,
  String? rejectionReason,
}) {
  return showDialog<ScoreProposeResult>(
    context: context,
    builder: (_) => _ScoreProposeDialog(
      rejected: rejected,
      rejectionReason: rejectionReason,
    ),
  );
}

class _ScoreProposeDialog extends StatefulWidget {
  const _ScoreProposeDialog({required this.rejected, this.rejectionReason});

  final bool rejected;
  final String? rejectionReason;

  @override
  State<_ScoreProposeDialog> createState() => _ScoreProposeDialogState();
}

class _ScoreProposeDialogState extends State<_ScoreProposeDialog> {
  String _license = _scoreProposeLicenses.first;
  final _attribution = TextEditingController();
  final _justification = TextEditingController();
  bool _attested = false;

  @override
  void dispose() {
    _attribution.dispose();
    _justification.dispose();
    super.dispose();
  }

  // A licence is always selected, so submit is gated on the attestation (and, when
  // re-proposing a rejected score, on a non-empty justification).
  bool get _canSubmit =>
      _attested && (!widget.rejected || _justification.text.trim().isNotEmpty);

  String _licenseDescription(AppLocalizations l10n) {
    if (_license.startsWith('CC0')) return l10n.licenseDescCc0;
    if (_license.startsWith('CC-BY-SA')) return l10n.licenseDescCcbysa;
    return l10n.licenseDescCcby;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return AlertDialog(
      title: Text(l10n.scoreProposeTitle),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(l10n.scoreProposeIntro),
            const SizedBox(height: 12),
            if (widget.rejected &&
                (widget.rejectionReason?.isNotEmpty ?? false))
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: Text(
                  l10n.scoreProposeRejectedReason(widget.rejectionReason!),
                  style: TextStyle(
                    color: CymbraColors.error,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            // Licence combobox (predefined choices) + a short description of the
            // selection, mirroring the SoundFont propose dialog.
            DropdownButtonFormField<String>(
              initialValue: _license,
              decoration: InputDecoration(labelText: l10n.proposeLicense),
              items: [
                for (final l in _scoreProposeLicenses)
                  DropdownMenuItem<String>(value: l, child: Text(l)),
              ],
              onChanged: (v) => setState(() => _license = v ?? _license),
            ),
            Padding(
              padding: const EdgeInsets.only(top: 6),
              child: Text(
                _licenseDescription(l10n),
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                ),
              ),
            ),
            const SizedBox(height: 8),
            TextField(
              controller: _attribution,
              decoration: InputDecoration(labelText: l10n.proposeAttribution),
            ),
            if (widget.rejected) ...[
              const SizedBox(height: 12),
              TextField(
                controller: _justification,
                minLines: 2,
                maxLines: 4,
                decoration: InputDecoration(
                  labelText: l10n.scoreProposeJustification,
                ),
                onChanged: (_) => setState(() {}),
              ),
            ],
            const SizedBox(height: 8),
            CheckboxListTile(
              contentPadding: EdgeInsets.zero,
              controlAffinity: ListTileControlAffinity.leading,
              value: _attested,
              onChanged: (v) => setState(() => _attested = v ?? false),
              title: Text(l10n.scoreProposeAttest),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.proposeCancel),
        ),
        FilledButton(
          onPressed: _canSubmit
              ? () => Navigator.of(context).pop((
                  license: _license,
                  attribution: _attribution.text.trim(),
                  justification: widget.rejected
                      ? _justification.text.trim()
                      : null,
                ))
              : null,
          child: Text(l10n.proposeSubmit),
        ),
      ],
    );
  }
}

/// A small pill showing a contributed score's public-catalog proposal status.
class ScoreProposalTag extends StatelessWidget {
  const ScoreProposalTag({super.key, required this.status});

  /// `pending` / `accepted` / `rejected`.
  final String status;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    // Short labels (the card is small; the long "En attente de vérification" would push
    // the pill too wide) — change: add-score-catalog-proposal.
    final (label, color) = switch (status) {
      'accepted' => (l10n.scoreTagAccepted, CymbraColors.primary),
      'rejected' => (l10n.scoreTagRejected, CymbraColors.error),
      _ => (l10n.scoreTagPending, CymbraColors.onSurfaceVariant),
    };
    // A dark scrim (like the card's other cover overlays) keeps the pill legible over
    // the cover art, with the status colour carried by the text + a subtle border.
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.7)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}
