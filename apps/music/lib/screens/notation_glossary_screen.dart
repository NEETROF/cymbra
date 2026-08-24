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
import '../notation/notation_help_content.dart';
import '../state/instrument_context.dart';
import '../theme/cymbra_theme.dart';

/// Opens the browsable notation glossary from a stable entry point (the help/tips
/// surface).
void openNotationGlossary(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const NotationGlossaryScreen()));

/// The browsable "reading the staff" glossary (change: add-notation-help): the
/// same explanations shown as one-time long-press bubbles, listed so a user can
/// look a symbol up away from a score, or re-read a bubble they already
/// dismissed. It is fed by [notationGlossarySamples] through the very same
/// [notationHelpFor] lookup the bubbles use, so the two can never diverge.
class NotationGlossaryScreen extends ConsumerWidget {
  const NotationGlossaryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final style = notationNameStyle(Localizations.localeOf(context));
    // The glossary is opened away from any score, so it follows the instrument
    // the app is CURRENTLY presenting — the same context the hub's filter is
    // seeded from. A drummer looking a symbol up gets the drum staff's list.
    final percussion =
        ref.watch(effectiveInstrumentContextProvider) == AppInstrument.drums;

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.notationHelpGlossaryTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              l10n.notationHelpHintBody,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 14,
                height: 1.35,
              ),
            ),
            const SizedBox(height: 18),
            for (final sample in notationGlossarySamplesFor(
              percussion: percussion,
            ))
              _GlossaryRow(
                help: notationHelpFor(
                  l10n,
                  sample,
                  solfege: style.solfege,
                  frenchRe: style.frenchRe,
                  // No kit is loaded here, so a stroke is named generically —
                  // the glossary explains the SYMBOL, not this score's piece.
                  drumPieceName: percussion
                      ? (_) => l10n.notationHelpGlossaryStroke
                      : null,
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _GlossaryRow extends StatelessWidget {
  const _GlossaryRow({required this.help});

  final NotationHelp help;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            help.title,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 15,
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 3),
          Text(
            help.body,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 13.5,
              height: 1.35,
            ),
          ),
        ],
      ),
    );
  }
}
