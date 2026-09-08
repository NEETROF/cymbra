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
import 'package:music/services/account_service.dart';
import 'package:music/state/session_state.dart';

void main() {
  group(
    'SessionState.accountUnresolved (change: fix-session-account-retry)',
    () {
      test('is true for an authenticated session with no account', () {
        expect(const SessionState.authenticated().accountUnresolved, isTrue);
      });

      test('is false once the account is resolved', () {
        const resolved = SessionState.authenticated(
          account: Account(userId: 'user-1', version: 1, handle: 'ada'),
        );
        expect(resolved.accountUnresolved, isFalse);
      });

      test('is false for an account still needing a handle', () {
        // A handle-less account is resolved — it routes to onboarding, it does
        // NOT retry. The two conditions must not be conflated.
        const brandNew = SessionState.authenticated(
          account: Account(userId: 'user-1', version: 1),
        );
        expect(brandNew.accountUnresolved, isFalse);
        expect(brandNew.needsHandle, isTrue);
      });

      test('is false for every non-authenticated state', () {
        expect(const SessionState.unknown().accountUnresolved, isFalse);
        expect(const SessionState.guest().accountUnresolved, isFalse);
        expect(const SessionState.unauthenticated().accountUnresolved, isFalse);
      });
    },
  );
}
