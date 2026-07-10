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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/app_language.dart';

void main() {
  group('AppLanguage', () {
    test('supports exactly en/fr/it/es with English first (the fallback)', () {
      expect(AppLanguage.values.map((l) => l.code), ['en', 'fr', 'it', 'es']);
      expect(AppLanguage.values.first, AppLanguage.en);
    });

    test('fromCode maps supported codes and rejects everything else', () {
      expect(AppLanguage.fromCode('fr'), AppLanguage.fr);
      expect(AppLanguage.fromCode('de'), isNull);
      expect(AppLanguage.fromCode(null), isNull);
    });

    test('exposes each language as a supported Locale', () {
      expect(AppLanguage.supportedLocales, const [
        Locale('en'),
        Locale('fr'),
        Locale('it'),
        Locale('es'),
      ]);
    });

    test('every language carries a flag emoji', () {
      for (final language in AppLanguage.values) {
        expect(language.flag, isNotEmpty);
      }
    });
  });

  group('resolveLanguage', () {
    test('a supported persisted choice wins over the device locale', () {
      expect(
        resolveLanguage(persistedCode: 'es', deviceLocale: const Locale('fr')),
        AppLanguage.es,
      );
    });

    test('with no persisted choice, a supported device locale is used', () {
      expect(resolveLanguage(deviceLocale: const Locale('it')), AppLanguage.it);
    });

    test('an unsupported device locale falls back to English', () {
      expect(resolveLanguage(deviceLocale: const Locale('de')), AppLanguage.en);
    });

    test('an unsupported persisted choice falls back to the device locale', () {
      expect(
        resolveLanguage(persistedCode: 'de', deviceLocale: const Locale('fr')),
        AppLanguage.fr,
      );
    });

    test(
      'an unsupported persisted choice and device locale fall back to English',
      () {
        expect(
          resolveLanguage(
            persistedCode: 'de',
            deviceLocale: const Locale('pt'),
          ),
          AppLanguage.en,
        );
      },
    );
  });
}
