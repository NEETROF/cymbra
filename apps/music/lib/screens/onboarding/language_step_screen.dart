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

import '../../l10n/gen/app_localizations.dart';
import '../../state/app_language.dart';
import '../../state/app_locale.dart';
import '../../state/onboarding_notifier.dart';
import '../../theme/cymbra_theme.dart';
import '../../widgets/language_selector.dart' show languageName;

/// The very first launch screen (D7): choose the app language, **before** the
/// welcome, with the device locale pre-selected when it is supported. No account
/// is involved, and the choice applies immediately — tapping a language switches
/// the UI on the spot, so the welcome that follows is already in the user's
/// language. It remains changeable later from the account menu.
class LanguageStepScreen extends ConsumerWidget {
  const LanguageStepScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    // The active locale is seeded from the device locale, so the matching
    // language starts selected without any extra defaulting logic here.
    final active =
        AppLanguage.fromCode(ref.watch(appLocaleProvider).languageCode) ??
        AppLanguage.en;

    return Scaffold(
      backgroundColor: CymbraColors.background,
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 460),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(
                    Icons.language,
                    size: 48,
                    color: CymbraColors.primary,
                  ),
                  const SizedBox(height: 12),
                  Text(
                    l10n.onboardingLanguageTitle,
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 20,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                  const SizedBox(height: 20),
                  Wrap(
                    alignment: WrapAlignment.center,
                    spacing: 12,
                    runSpacing: 12,
                    children: [
                      for (final language in AppLanguage.values)
                        _LanguageTile(
                          language: language,
                          selected: language == active,
                          label: languageName(l10n, language),
                          // Applying on tap (rather than on Continue) is what
                          // makes the choice visible immediately.
                          onTap: () => ref
                              .read(appLocaleProvider.notifier)
                              .select(language),
                        ),
                    ],
                  ),
                  const SizedBox(height: 24),
                  SizedBox(
                    width: double.infinity,
                    child: FilledButton(
                      key: const Key('onboarding-language-continue'),
                      onPressed: () => ref
                          .read(onboardingProvider.notifier)
                          .chooseLanguage(active),
                      child: Text(l10n.onboardingLanguageContinue),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// One language choice: its flag with the language name underneath (the name is
/// also the screen-reader label, and the tile is announced as selected).
class _LanguageTile extends StatelessWidget {
  const _LanguageTile({
    required this.language,
    required this.selected,
    required this.label,
    required this.onTap,
  });

  final AppLanguage language;
  final bool selected;
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: label,
      selected: selected,
      button: true,
      child: InkWell(
        key: Key('onboarding-language-${language.code}'),
        borderRadius: BorderRadius.circular(12),
        onTap: onTap,
        child: Container(
          width: 104,
          padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: selected
                  ? CymbraColors.tertiary
                  : CymbraColors.surfaceContainerHighest,
              width: selected ? 2 : 1,
            ),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(language.flag, style: const TextStyle(fontSize: 28)),
              const SizedBox(height: 6),
              Text(
                label,
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: selected
                      ? CymbraColors.onSurface
                      : CymbraColors.onSurfaceVariant,
                  fontSize: 12.5,
                  fontWeight: selected ? FontWeight.w700 : FontWeight.w500,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
