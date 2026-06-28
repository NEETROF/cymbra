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
import '../../services/grpc_client.dart';
import '../../services/oidc_token_source.dart';
import '../../state/auth_flow.dart';
import '../../state/session_notifier.dart';
import '../../theme/cymbra_theme.dart';
import 'auth_messages.dart';
import 'auth_scaffold.dart';

/// Account deletion gated behind **fresh re-authentication** + an explicit
/// irreversible confirmation (spec: "Account deletion", design D8). The user
/// re-enters email + password (verified via `SignInLocal`) or re-runs Google/
/// Apple; only then is the confirmation enabled and `DeleteAccount` called.
class DeleteAccountScreen extends ConsumerStatefulWidget {
  const DeleteAccountScreen({super.key});

  @override
  ConsumerState<DeleteAccountScreen> createState() =>
      _DeleteAccountScreenState();
}

class _DeleteAccountScreenState extends ConsumerState<DeleteAccountScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  bool _busy = false;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  /// Run a re-authentication [reauth] that returns true on success (false = the
  /// user cancelled an OIDC sheet); on success, confirm and delete.
  Future<void> _reauthThenDelete(Future<bool> Function() reauth) async {
    setState(() => _busy = true);
    try {
      final ok = await reauth();
      if (!ok) return; // cancelled — no-op
      if (!mounted) return;
      final confirmed = await _confirmIrreversible();
      if (confirmed != true) return; // explicit confirmation required
      await ref.read(accountServiceProvider).deleteAccount();
      await ref.read(sessionNotifierProvider.notifier).onAccountDeleted();
      if (mounted) Navigator.of(context).popUntil((r) => r.isFirst);
    } on AuthException catch (e) {
      if (mounted) {
        showAuthError(
          context,
          e,
          fallback: 'Authentication failed — your account was not deleted.',
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _deleteWithPassword() {
    final email = _email.text.trim();
    final password = _password.text;
    if (email.isEmpty || password.isEmpty) return Future.value();
    return _reauthThenDelete(() async {
      await ref
          .read(authFlowProvider)
          .signInEmail(email: email, password: password);
      return true;
    });
  }

  Future<void> _deleteWithGoogle() =>
      _reauthThenDelete(() => ref.read(authFlowProvider).signInWithGoogle());

  Future<void> _deleteWithApple() =>
      _reauthThenDelete(() => ref.read(authFlowProvider).signInWithApple());

  Future<bool?> _confirmIrreversible() => showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      title: const Text('Delete account?'),
      content: const Text(
        'This permanently deletes your Cymbra account and cannot be undone.',
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(false),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('delete-confirm'),
          style: FilledButton.styleFrom(backgroundColor: CymbraColors.error),
          onPressed: () => Navigator.of(ctx).pop(true),
          child: const Text('Delete'),
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final oidc = ref.watch(oidcTokenSourceProvider);
    final googleAvailable = oidc.googleAvailable;
    final appleAvailable = oidc.appleAvailable;

    return AuthScaffold(
      title: 'Delete account',
      children: [
        const Text(
          'For your security, confirm it’s you before deleting. This action is '
          'permanent.',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        TextField(
          key: const Key('delete-email'),
          controller: _email,
          keyboardType: TextInputType.emailAddress,
          autocorrect: false,
          decoration: const InputDecoration(labelText: 'Email'),
        ),
        const SizedBox(height: 16),
        TextField(
          key: const Key('delete-password'),
          controller: _password,
          obscureText: true,
          decoration: const InputDecoration(labelText: 'Password'),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('delete-with-password'),
          style: FilledButton.styleFrom(backgroundColor: CymbraColors.error),
          onPressed: _busy ? null : _deleteWithPassword,
          child: _busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Confirm with password & delete'),
        ),
        if (googleAvailable || appleAvailable) ...[
          const SizedBox(height: 16),
          const Text('or re-authenticate with', textAlign: TextAlign.center),
        ],
        if (googleAvailable) ...[
          const SizedBox(height: 8),
          OutlinedButton.icon(
            key: const Key('delete-with-google'),
            onPressed: _busy ? null : _deleteWithGoogle,
            icon: const Icon(Icons.account_circle),
            label: const Text('Google'),
          ),
        ],
        if (appleAvailable) ...[
          const SizedBox(height: 8),
          OutlinedButton.icon(
            key: const Key('delete-with-apple'),
            onPressed: _busy ? null : _deleteWithApple,
            icon: const Icon(Icons.apple),
            label: const Text('Apple'),
          ),
        ],
      ],
    );
  }
}
