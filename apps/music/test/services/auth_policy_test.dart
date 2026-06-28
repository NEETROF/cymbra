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
import 'package:music/services/auth_policy.dart';

void main() {
  group('passwordPolicyError', () {
    test('accepts a policy-compliant password', () {
      expect(passwordPolicyError('correct horse battery'), isNull);
      expect(passwordPolicyError('a' * kPasswordMinLength), isNull);
    });

    test('rejects a too-short password', () {
      expect(passwordPolicyError('a' * (kPasswordMinLength - 1)), isNotNull);
      expect(passwordPolicyError(''), isNotNull);
    });
  });

  group('isValidHandle (mirrors backend handle_core)', () {
    test('accepts 1–15 letters and numbers', () {
      expect(isValidHandle('alice'), isTrue);
      expect(isValidHandle('Alice99'), isTrue);
      expect(isValidHandle('a'), isTrue);
      expect(isValidHandle('123456789012345'), isTrue); // 15
      expect(isValidHandle('café'), isTrue); // Unicode letters
    });

    test('rejects empty, too long, or non-alphanumeric handles', () {
      expect(isValidHandle(''), isFalse);
      expect(isValidHandle('1234567890123456'), isFalse); // 16
      for (final bad in [
        'has space',
        'dash-y',
        'under_score',
        'dot.',
        '@bob',
      ]) {
        expect(isValidHandle(bad), isFalse, reason: bad);
      }
    });
  });
}
