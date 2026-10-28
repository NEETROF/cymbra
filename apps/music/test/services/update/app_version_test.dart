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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/update/app_version.dart';

AppVersion v(String raw) => AppVersion.tryParse(raw)!;

void main() {
  group('AppVersion.tryParse', () {
    test('parses the app format, with and without a build', () {
      expect(v('1.24.0+32').toString(), '1.24.0+32');
      expect(v('1.24.0').toString(), '1.24.0+0');
      expect(v('  1.24.0+32  ').toString(), '1.24.0+32');
    });

    test('rejects anything that is not major.minor.patch(+build)', () {
      for (final bad in [
        '',
        '   ',
        '1',
        '1.2',
        '1.2.3.4',
        'v1.2.3',
        '1.2.x',
        '1.2.3+',
        '1.2.3+abc',
        '-1.2.3',
        '1.2.3-beta',
        '+1.2.3',
        '1.2.3++4',
      ]) {
        expect(AppVersion.tryParse(bad), isNull, reason: 'accepted "$bad"');
      }
    });
  });

  group('ordering', () {
    test('is numeric, not lexical', () {
      // The bug a text sort produces: 1.10.0 must be ABOVE 1.9.0.
      expect(v('1.10.0+40') > v('1.9.0+39'), isTrue);
      expect(v('2.0.0+1') > v('1.99.99+9999'), isTrue);
      expect(v('1.24.1+1') > v('1.24.0+99'), isTrue);
    });

    test('the build number is a real tiebreaker, unlike strict semver', () {
      // Strict semver ignores build metadata in precedence; here two releases
      // can share a triple, so +33 IS newer than +32.
      expect(v('1.24.0+33') > v('1.24.0+32'), isTrue);
      expect(v('1.24.0+32') < v('1.24.0+33'), isTrue);
    });

    test('a missing build reads as +0', () {
      expect(v('1.24.0') == v('1.24.0+0'), isTrue);
      expect(v('1.24.0+1') > v('1.24.0'), isTrue);
    });

    test('equality and hashing agree with the ordering', () {
      expect(v('1.2.3+4') == v('1.2.3+4'), isTrue);
      expect(v('1.2.3+4').hashCode, v('1.2.3+4').hashCode);
      expect(v('1.2.3+4') == v('1.2.3+5'), isFalse);
      expect(v('1.2.3+4') >= v('1.2.3+4'), isTrue);
      expect(v('1.2.3+4') <= v('1.2.3+4'), isTrue);
    });

    test('sorts a list into release order', () {
      final list = [v('1.9.0+9'), v('1.10.0+10'), v('1.10.0+11'), v('0.9.9+1')]
        ..sort();
      expect(list.map((e) => e.toString()), [
        '0.9.9+1',
        '1.9.0+9',
        '1.10.0+10',
        '1.10.0+11',
      ]);
    });
  });

  test('display drops the build number, which means nothing to a user', () {
    expect(v('1.24.0+32').display, '1.24.0');
  });
}
