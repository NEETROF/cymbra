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
import '../../state/connected_accounts_notifier.dart';
import '../../state/connected_accounts_state.dart';
import '../auth/auth_scaffold.dart';
import '../auth/otp_verify_screen.dart';

/// "Set a password" sub-flow (change: add-account-identity-linking): collects an
/// email + password for the current account. The action runs on
/// [ConnectedAccountsNotifier]; this screen fires it and, on success, goes
/// straight to the code-entry screen ([OtpVerifyScreen]) so the user verifies the
/// email in place — the credential is bound only once the code is confirmed
/// (change: verify-before-local-credential-link). The user stays signed in via
/// their existing session. Only offered when the account has no `local` identity yet.
class SetPasswordScreen extends ConsumerStatefulWidget {
  const SetPasswordScreen({super.key});

  @override
  ConsumerState<SetPasswordScreen> createState() => _SetPasswordScreenState();
}

class _SetPasswordScreenState extends ConsumerState<SetPasswordScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();

  /// The `actionSeq` at submit time, so we only react to *our* action completing.
  int? _pendingSeq;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  void _submit() {
    final email = _email.text.trim();
    final password = _password.text;
    if (email.isEmpty || password.isEmpty) return;
    _pendingSeq = ref.read(connectedAccountsNotifierProvider).actionSeq;
    ref
        .read(connectedAccountsNotifierProvider.notifier)
        .linkEmailPassword(
          email: email,
          password: password,
          locale: AppLocalizations.of(context).localeName,
        );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final busy = ref.watch(
      connectedAccountsNotifierProvider.select((s) => s.busy),
    );

    // React to our action completing: on success, go straight to the code-entry
    // screen so the user verifies the email in place (the credential binds only on
    // verification); stay on failure (the parent surfaces the error).
    ref.listen(connectedAccountsNotifierProvider.select((s) => s.actionSeq), (
      previous,
      next,
    ) {
      if (_pendingSeq == null || next == _pendingSeq) return;
      final state = ref.read(connectedAccountsNotifierProvider);
      if (state.lastAction != ConnectedAccountsAction.setPassword) return;
      _pendingSeq = null;
      if (state.actionError == null && mounted) {
        // Push the code screen (do NOT replace this route): replacing would
        // complete the parent's `await push(SetPasswordScreen)` immediately, firing
        // its list-refresh before verification. Instead, wait for the code screen to
        // return, then pop ourselves — so the parent refreshes only after the
        // identity is actually bound (at verify time).
        final email = _email.text.trim();
        final navigator = Navigator.of(context);
        navigator
            .push(
              MaterialPageRoute<void>(
                builder: (_) => OtpVerifyScreen(email: email),
              ),
            )
            .then((_) {
              if (mounted) navigator.pop();
            });
      }
    });

    return AuthScaffold(
      title: l10n.setPasswordTitle,
      children: [
        Text(l10n.setPasswordIntro, textAlign: TextAlign.center),
        const SizedBox(height: 24),
        TextField(
          key: const Key('set-password-email'),
          controller: _email,
          keyboardType: TextInputType.emailAddress,
          autocorrect: false,
          decoration: InputDecoration(labelText: l10n.fieldEmail),
        ),
        const SizedBox(height: 16),
        TextField(
          key: const Key('set-password-password'),
          controller: _password,
          obscureText: true,
          decoration: InputDecoration(labelText: l10n.fieldPassword),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('set-password-submit'),
          onPressed: busy ? null : _submit,
          child: busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(l10n.setPasswordButton),
        ),
      ],
    );
  }
}
