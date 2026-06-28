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

/// Email-verification step (spec: "Email verification by code"). The user types
/// the emailed OTP; on success — when the password is known (came from sign-up or
/// a sign-in that hit `FAILED_PRECONDITION`) — the app signs in directly,
/// otherwise it returns to sign-in. Offers a resend wired to `ResendVerification`.
class OtpVerifyScreen extends ConsumerStatefulWidget {
  const OtpVerifyScreen({super.key, required this.email, this.password});

  final String email;

  /// When present, sign in automatically after a successful verification.
  final String? password;

  @override
  ConsumerState<OtpVerifyScreen> createState() => _OtpVerifyScreenState();
}

class _OtpVerifyScreenState extends ConsumerState<OtpVerifyScreen> {
  final _code = TextEditingController();
  bool _busy = false;

  @override
  void dispose() {
    _code.dispose();
    super.dispose();
  }

  Future<void> _verify() async {
    final code = _code.text.trim();
    if (code.isEmpty) return;
    setState(() => _busy = true);
    try {
      await ref.read(authFlowProvider).verifyEmail(code);
      if (widget.password != null) {
        await ref
            .read(authFlowProvider)
            .signInEmail(email: widget.email, password: widget.password!);
        if (mounted) Navigator.of(context).popUntil((r) => r.isFirst);
      } else {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Email verified — please sign in.')),
          );
          Navigator.of(context).pop();
        }
      }
    } on AuthException catch (e) {
      if (mounted) {
        showAuthError(context, e, fallback: 'That code is invalid or expired.');
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _resend() async {
    try {
      await ref.read(authFlowProvider).resendVerification(widget.email);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('A new code was sent to your email.')),
        );
      }
    } on AuthException catch (e) {
      if (mounted) showAuthError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AuthScaffold(
      title: 'Verify your email',
      children: [
        Text(
          'Enter the code we sent to ${widget.email}.',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        TextField(
          key: const Key('otp-code'),
          controller: _code,
          keyboardType: TextInputType.number,
          decoration: const InputDecoration(labelText: 'Verification code'),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('otp-verify'),
          onPressed: _busy ? null : _verify,
          child: _busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Verify'),
        ),
        TextButton(
          key: const Key('otp-resend'),
          onPressed: _busy ? null : _resend,
          child: const Text('Resend code'),
        ),
      ],
    );
  }
}
