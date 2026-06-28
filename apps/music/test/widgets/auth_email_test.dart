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
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/auth/email_sign_in_screen.dart';
import 'package:music/screens/auth/email_sign_up_screen.dart';
import 'package:music/screens/auth/forgot_password_screen.dart';
import 'package:music/screens/auth/otp_verify_screen.dart';
import 'package:music/services/auth_service.dart';

import '../support/auth_fakes.dart';
import '../support/auth_harness.dart';

Future<ProviderContainer> _pump(
  WidgetTester tester,
  Widget child, {
  FakeAuthService? auth,
  FakeAccountService? account,
}) async {
  final c = authContainer(auth: auth, account: account);
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: MaterialApp(home: child),
    ),
  );
  return c;
}

const _goodPassword = 'longenoughpassword';

void main() {
  group('Email sign-up (task 5.1)', () {
    testWidgets('a weak password blocks submission', (tester) async {
      final auth = FakeAuthService();
      await _pump(tester, const EmailSignUpScreen(), auth: auth);

      await tester.enterText(find.byKey(const Key('signup-email')), 'a@x.dev');
      await tester.enterText(find.byKey(const Key('signup-password')), 'short');
      await tester.tap(find.byKey(const Key('signup-submit')));
      await tester.pump();

      expect(auth.calls, isEmpty); // never reached the backend
      expect(find.textContaining('at least'), findsWidgets);
    });

    testWidgets('an existing email shows an in-use message', (tester) async {
      final auth = FakeAuthService()
        ..signUpError = const AuthException(AuthError.alreadyExists);
      await _pump(tester, const EmailSignUpScreen(), auth: auth);

      await tester.enterText(find.byKey(const Key('signup-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('signup-password')),
        _goodPassword,
      );
      await tester.tap(find.byKey(const Key('signup-submit')));
      await tester.pump();
      await tester.pump();

      expect(find.textContaining('already in use'), findsOneWidget);
    });

    testWidgets('success advances to OTP verification', (tester) async {
      await _pump(tester, const EmailSignUpScreen(), auth: FakeAuthService());

      await tester.enterText(find.byKey(const Key('signup-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('signup-password')),
        _goodPassword,
      );
      await tester.tap(find.byKey(const Key('signup-submit')));
      await tester.pumpAndSettle();

      expect(find.byType(OtpVerifyScreen), findsOneWidget);
    });
  });

  group('Email sign-in (task 5.3)', () {
    testWidgets('wrong credentials show an invalid-credentials message', (
      tester,
    ) async {
      final auth = FakeAuthService()
        ..signInError = const AuthException(AuthError.unauthenticated);
      await _pump(tester, const EmailSignInScreen(), auth: auth);

      await tester.enterText(find.byKey(const Key('signin-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('signin-password')),
        _goodPassword,
      );
      await tester.tap(find.byKey(const Key('signin-submit')));
      await tester.pump();
      await tester.pump();

      expect(
        find.textContaining('Incorrect email or password'),
        findsOneWidget,
      );
    });

    testWidgets('an unverified account routes to verification', (tester) async {
      final auth = FakeAuthService()
        ..signInError = const AuthException(AuthError.failedPrecondition);
      await _pump(tester, const EmailSignInScreen(), auth: auth);

      await tester.enterText(find.byKey(const Key('signin-email')), 'a@x.dev');
      await tester.enterText(
        find.byKey(const Key('signin-password')),
        _goodPassword,
      );
      await tester.tap(find.byKey(const Key('signin-submit')));
      await tester.pumpAndSettle();

      expect(find.byType(OtpVerifyScreen), findsOneWidget);
    });
  });

  group('OTP verify (task 5.2)', () {
    testWidgets('resend reports a new code was sent', (tester) async {
      await _pump(
        tester,
        const OtpVerifyScreen(email: 'a@x.dev'),
        auth: FakeAuthService(),
      );

      await tester.tap(find.byKey(const Key('otp-resend')));
      await tester.pump();
      await tester.pump();

      expect(find.textContaining('new code was sent'), findsOneWidget);
    });

    testWidgets('a rate-limited resend tells the user to wait', (tester) async {
      final auth = FakeAuthService()
        ..resendError = const AuthException(AuthError.rateLimited);
      await _pump(tester, const OtpVerifyScreen(email: 'a@x.dev'), auth: auth);

      await tester.tap(find.byKey(const Key('otp-resend')));
      await tester.pump();
      await tester.pump();

      expect(find.textContaining('Too many attempts'), findsOneWidget);
    });
  });

  group('Forgot password (task 5.4)', () {
    testWidgets('request then reset completes and informs of sign-out', (
      tester,
    ) async {
      final auth = FakeAuthService();
      await _pump(tester, const ForgotPasswordScreen(), auth: auth);

      await tester.enterText(find.byKey(const Key('forgot-email')), 'a@x.dev');
      await tester.tap(find.byKey(const Key('forgot-request')));
      await tester.pump();
      await tester.pump();
      expect(find.textContaining('on its way'), findsOneWidget);

      await tester.enterText(find.byKey(const Key('forgot-code')), '123456');
      await tester.enterText(
        find.byKey(const Key('forgot-new-password')),
        _goodPassword,
      );
      await tester.tap(find.byKey(const Key('forgot-reset')));
      await tester.pump();
      await tester.pump();

      expect(auth.calls, contains('resetPassword:123456'));
    });
  });
}
