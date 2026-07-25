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

import '../../l10n/gen/app_localizations.dart';
import '../../services/auth_policy.dart';
import '../../services/auth_service.dart';
import '../../state/auth_flow.dart';
import '../../widgets/app_snackbar.dart';
import 'auth_messages.dart';
import 'auth_scaffold.dart';

/// Forgot-password flow (spec: "Forgot password reset by code"). Step 1 requests
/// a reset for an email with identical UX whether or not the account exists (no
/// enumeration). Step 2 takes the emailed code + a new password and calls
/// `ResetPassword`, then informs the user all sessions were signed out.
class ForgotPasswordScreen extends ConsumerStatefulWidget {
  const ForgotPasswordScreen({super.key});

  @override
  ConsumerState<ForgotPasswordScreen> createState() =>
      _ForgotPasswordScreenState();
}

class _ForgotPasswordScreenState extends ConsumerState<ForgotPasswordScreen> {
  final _email = TextEditingController();
  final _code = TextEditingController();
  final _newPassword = TextEditingController();
  String? _passwordError;
  bool _requested = false;
  bool _busy = false;

  @override
  void dispose() {
    _email.dispose();
    _code.dispose();
    _newPassword.dispose();
    super.dispose();
  }

  Future<void> _request() async {
    final email = _email.text.trim();
    if (email.isEmpty) return;
    final l10n = AppLocalizations.of(context);
    setState(() => _busy = true);
    try {
      await ref.read(authFlowProvider).requestPasswordReset(email);
      // No-enumeration: the same confirmation regardless of whether it exists.
      if (mounted) {
        setState(() => _requested = true);
        showAppSnackBar(ScaffoldMessenger.of(context), l10n.resetSentSnack);
      }
    } on AuthException catch (e) {
      if (mounted) showAuthError(context, e);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _reset() async {
    final code = _code.text.trim();
    final newPassword = _newPassword.text;
    final policyError = passwordPolicyError(newPassword);
    setState(() => _passwordError = policyError);
    if (code.isEmpty || policyError != null) return;

    final l10n = AppLocalizations.of(context);
    setState(() => _busy = true);
    try {
      await ref
          .read(authFlowProvider)
          .resetPassword(code: code, newPassword: newPassword);
      if (mounted) {
        showAppSnackBar(ScaffoldMessenger.of(context), l10n.resetDoneSnack);
        Navigator.of(context).pop();
      }
    } on AuthException catch (e) {
      if (mounted) {
        showAuthError(context, e, fallback: l10n.resetInvalidCodeFallback);
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return AuthScaffold(
      title: l10n.resetTitle,
      children: [
        TextField(
          key: const Key('forgot-email'),
          controller: _email,
          enabled: !_requested,
          keyboardType: TextInputType.emailAddress,
          autocorrect: false,
          decoration: InputDecoration(labelText: l10n.fieldEmail),
        ),
        const SizedBox(height: 16),
        if (!_requested)
          FilledButton(
            key: const Key('forgot-request'),
            onPressed: _busy ? null : _request,
            child: Text(l10n.resetSendCode),
          )
        else ...[
          TextField(
            key: const Key('forgot-code'),
            controller: _code,
            keyboardType: TextInputType.number,
            decoration: InputDecoration(labelText: l10n.fieldResetCode),
          ),
          const SizedBox(height: 16),
          TextField(
            key: const Key('forgot-new-password'),
            controller: _newPassword,
            obscureText: true,
            decoration: InputDecoration(
              labelText: l10n.fieldNewPassword,
              helperText: l10n.passwordMinChars(kPasswordMinLength),
              errorText: _passwordError,
            ),
          ),
          const SizedBox(height: 24),
          FilledButton(
            key: const Key('forgot-reset'),
            onPressed: _busy ? null : _reset,
            child: Text(l10n.resetSetNewPassword),
          ),
        ],
      ],
    );
  }
}
