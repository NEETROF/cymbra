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

import '../../services/auth_service.dart';
import '../../state/auth_flow.dart';
import 'auth_messages.dart';
import 'auth_scaffold.dart';
import 'email_sign_up_screen.dart';
import 'forgot_password_screen.dart';
import 'otp_verify_screen.dart';

/// Email sign-in (spec: "Email sign-in"). Distinguishes wrong-credential,
/// lockout, and unverified — the latter routes to the verification step rather
/// than failing. On success the [SessionNotifier] takes over (handle onboarding
/// if needed) and we pop back to the gate.
class EmailSignInScreen extends ConsumerStatefulWidget {
  const EmailSignInScreen({super.key});

  @override
  ConsumerState<EmailSignInScreen> createState() => _EmailSignInScreenState();
}

class _EmailSignInScreenState extends ConsumerState<EmailSignInScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  bool _busy = false;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final email = _email.text.trim();
    final password = _password.text;
    if (email.isEmpty || password.isEmpty) return;

    setState(() => _busy = true);
    try {
      await ref
          .read(authFlowProvider)
          .signInEmail(email: email, password: password);
      if (mounted) Navigator.of(context).popUntil((r) => r.isFirst);
    } on AuthException catch (e) {
      if (!mounted) return;
      if (e.error == AuthError.failedPrecondition) {
        // Email not verified yet — route to verification, carrying the password
        // so a successful verify signs the user straight in.
        Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => OtpVerifyScreen(email: email, password: password),
          ),
        );
        return;
      }
      showAuthError(
        context,
        e,
        fallback: e.error == AuthError.rateLimited
            ? 'Too many attempts — please try again later.'
            : 'Incorrect email or password.',
      );
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _goTo(Widget screen) => Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => screen));

  @override
  Widget build(BuildContext context) {
    return AuthScaffold(
      title: 'Sign in',
      children: [
        TextField(
          key: const Key('signin-email'),
          controller: _email,
          keyboardType: TextInputType.emailAddress,
          autocorrect: false,
          decoration: const InputDecoration(labelText: 'Email'),
        ),
        const SizedBox(height: 16),
        TextField(
          key: const Key('signin-password'),
          controller: _password,
          obscureText: true,
          decoration: const InputDecoration(labelText: 'Password'),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('signin-submit'),
          onPressed: _busy ? null : _submit,
          child: _busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Sign in'),
        ),
        const SizedBox(height: 8),
        TextButton(
          key: const Key('signin-forgot'),
          onPressed: _busy ? null : () => _goTo(const ForgotPasswordScreen()),
          child: const Text('Forgot password?'),
        ),
        TextButton(
          key: const Key('signin-create'),
          onPressed: _busy ? null : () => _goTo(const EmailSignUpScreen()),
          child: const Text('Create an account'),
        ),
      ],
    );
  }
}
