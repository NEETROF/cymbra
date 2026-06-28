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

import '../../services/auth_policy.dart';
import '../../services/auth_service.dart';
import '../../state/auth_flow.dart';
import 'auth_messages.dart';
import 'auth_scaffold.dart';
import 'otp_verify_screen.dart';

/// Email sign-up (spec: "Email sign-up"). Validates the password against the
/// backend policy before submitting, calls `SignUpLocal`, and advances to OTP
/// verification. `ALREADY_EXISTS` becomes an "email already in use" message.
class EmailSignUpScreen extends ConsumerStatefulWidget {
  const EmailSignUpScreen({super.key});

  @override
  ConsumerState<EmailSignUpScreen> createState() => _EmailSignUpScreenState();
}

class _EmailSignUpScreenState extends ConsumerState<EmailSignUpScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  String? _passwordError;
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
    final policyError = passwordPolicyError(password);
    setState(() => _passwordError = policyError);
    if (email.isEmpty || policyError != null) return;

    setState(() => _busy = true);
    try {
      await ref.read(authFlowProvider).signUp(email: email, password: password);
      if (mounted) {
        Navigator.of(context).pushReplacement(
          MaterialPageRoute<void>(
            builder: (_) => OtpVerifyScreen(email: email, password: password),
          ),
        );
      }
    } on AuthException catch (e) {
      if (mounted) {
        showAuthError(
          context,
          e,
          fallback: 'Could not create the account. Please try again.',
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AuthScaffold(
      title: 'Create your account',
      children: [
        TextField(
          key: const Key('signup-email'),
          controller: _email,
          keyboardType: TextInputType.emailAddress,
          autocorrect: false,
          decoration: const InputDecoration(labelText: 'Email'),
        ),
        const SizedBox(height: 16),
        TextField(
          key: const Key('signup-password'),
          controller: _password,
          obscureText: true,
          decoration: InputDecoration(
            labelText: 'Password',
            helperText: 'At least $kPasswordMinLength characters',
            errorText: _passwordError,
          ),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('signup-submit'),
          onPressed: _busy ? null : _submit,
          child: _busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Create account'),
        ),
      ],
    );
  }
}
