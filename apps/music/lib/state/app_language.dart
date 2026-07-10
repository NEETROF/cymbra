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

import 'dart:ui' show Locale;

/// The UI languages the app supports.
///
/// Each carries its `Locale` [code] and the flag shown in the language picker
/// (a Unicode regional-indicator emoji — the picker uses the flag rather than a
/// language name, with an accessible label supplied separately). Order matters:
/// [en] is first so it is the ultimate fallback when the device locale is not
/// supported (Flutter resolves an unknown locale to the first supported one).
enum AppLanguage {
  en('en', '🇬🇧'),
  fr('fr', '🇫🇷'),
  it('it', '🇮🇹'),
  es('es', '🇪🇸');

  const AppLanguage(this.code, this.flag);

  /// ISO-639 language code, used as the `Locale` language and the persisted key.
  final String code;

  /// Flag emoji shown in the picker (regional-indicator pair).
  final String flag;

  /// The `Locale` for this language.
  Locale get locale => Locale(code);

  /// The supported language whose [code] equals [code], or `null` if none
  /// (including when [code] is null or a language we do not translate).
  static AppLanguage? fromCode(String? code) {
    for (final language in values) {
      if (language.code == code) return language;
    }
    return null;
  }

  /// Every supported `Locale`, in declaration order (English first).
  static List<Locale> get supportedLocales => [
    for (final language in values) language.locale,
  ];
}

/// Resolves the effective UI language.
///
/// Precedence (see the `app-localization` spec):
/// 1. a supported [persistedCode] the user previously chose,
/// 2. otherwise the [deviceLocale]'s language if it is supported,
/// 3. otherwise English.
///
/// Pure and host-testable: it takes the device locale as data rather than
/// reading the platform, so every branch is exercised without a real device.
AppLanguage resolveLanguage({
  String? persistedCode,
  required Locale deviceLocale,
}) {
  return AppLanguage.fromCode(persistedCode) ??
      AppLanguage.fromCode(deviceLocale.languageCode) ??
      AppLanguage.en;
}
