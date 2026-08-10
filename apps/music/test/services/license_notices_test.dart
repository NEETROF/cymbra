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

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/license_notices.dart';

void main() {
  group('parseRustLicenseNotices', () {
    test('well-formed JSON yields one entry per license group', () {
      final entries = parseRustLicenseNotices('''
      {
        "licenses": [
          {
            "id": "MIT",
            "name": "MIT License",
            "text": "MIT license text",
            "packages": ["midir 0.11.0", "cpal 0.18.0"]
          },
          {
            "id": "Apache-2.0",
            "name": "Apache License 2.0",
            "text": "Apache license text",
            "packages": ["rustysynth 1.3.0"]
          }
        ]
      }
      ''');

      expect(entries, hasLength(2));
      final mit = entries[0] as LicenseEntryWithLineBreaks;
      expect(mit.packages, ['midir 0.11.0', 'cpal 0.18.0']);
      expect(mit.paragraphs.map((p) => p.text).join('\n'), 'MIT license text');
      final apache = entries[1] as LicenseEntryWithLineBreaks;
      expect(apache.packages, ['rustysynth 1.3.0']);
    });

    test('malformed JSON yields an empty list', () {
      expect(parseRustLicenseNotices('not json'), isEmpty);
    });

    test('missing "licenses" key yields an empty list', () {
      expect(parseRustLicenseNotices('{}'), isEmpty);
    });

    test('"licenses" not a list yields an empty list', () {
      expect(parseRustLicenseNotices('{"licenses": "oops"}'), isEmpty);
    });

    test('entries missing text or packages are skipped', () {
      final entries = parseRustLicenseNotices('''
      {
        "licenses": [
          {"id": "MIT", "name": "MIT License", "packages": ["a 1.0.0"]},
          {"id": "ISC", "name": "ISC License", "text": "ISC text"},
          {
            "id": "Zlib",
            "name": "zlib License",
            "text": "zlib text",
            "packages": []
          }
        ]
      }
      ''');

      expect(entries, isEmpty);
    });
  });

  group('registerRustLicenseNotices', () {
    testWidgets(
      'the collected entries are readable from LicenseRegistry.licenses',
      (tester) async {
        registerRustLicenseNotices();

        final entries = await tester.runAsync(
          () => LicenseRegistry.licenses.toList(),
        );

        expect(
          entries!.whereType<LicenseEntryWithLineBreaks>().any(
            (e) => e.packages.any((p) => p.startsWith('midir ')),
          ),
          isTrue,
          reason:
              'expected the bundled assets/generated/rust_licenses.json '
              'to surface a midir entry',
        );
      },
    );
  });
}
