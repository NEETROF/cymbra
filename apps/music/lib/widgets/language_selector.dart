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
import '../state/app_language.dart';
import '../state/app_locale.dart';
import '../theme/cymbra_theme.dart';

/// Localized display name for a language — the accessible label announced behind
/// its flag. Shared by the settings drawer picker and [LanguageSelectorButton].
String languageName(AppLocalizations l10n, AppLanguage language) =>
    switch (language) {
      AppLanguage.en => l10n.languageEnglish,
      AppLanguage.fr => l10n.languageFrench,
      AppLanguage.it => l10n.languageItalian,
      AppLanguage.es => l10n.languageSpanish,
    };

/// A compact language switcher: the current language's flag as a button that
/// opens a dialog of flags. Meant for screens without the settings drawer (the
/// login/entry screen and the score library). A dialog (a modal route) is used
/// instead of a popup menu, which flickers on iPad.
class LanguageSelectorButton extends ConsumerWidget {
  const LanguageSelectorButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final active =
        AppLanguage.fromCode(ref.watch(appLocaleProvider).languageCode) ??
        AppLanguage.en;
    return IconButton(
      tooltip: l10n.settingsCategoryLanguage,
      icon: Text(active.flag, style: const TextStyle(fontSize: 20)),
      onPressed: () => showLanguageDialog(context, ref),
    );
  }
}

/// Shows the language picker as a dialog of flags (with accessible labels),
/// marking the active language; selecting one switches the UI and closes.
Future<void> showLanguageDialog(BuildContext context, WidgetRef ref) {
  final l10n = AppLocalizations.of(context);
  final active =
      AppLanguage.fromCode(ref.read(appLocaleProvider).languageCode) ??
      AppLanguage.en;
  return showDialog<void>(
    context: context,
    builder: (dialogContext) => SimpleDialog(
      title: Text(l10n.settingsCategoryLanguage),
      children: [
        for (final language in AppLanguage.values)
          Semantics(
            label: languageName(l10n, language),
            selected: language == active,
            button: true,
            child: SimpleDialogOption(
              onPressed: () {
                ref.read(appLocaleProvider.notifier).select(language);
                Navigator.of(dialogContext).pop();
              },
              child: Row(
                children: [
                  Icon(
                    language == active
                        ? Icons.check_circle
                        : Icons.radio_button_unchecked,
                    size: 20,
                    color: language == active
                        ? CymbraColors.tertiary
                        : CymbraColors.onSurfaceVariant,
                  ),
                  const SizedBox(width: 16),
                  Text(language.flag, style: const TextStyle(fontSize: 24)),
                ],
              ),
            ),
          ),
      ],
    ),
  );
}
