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
import '../../services/account_service.dart';
import '../../services/auth_service.dart';
import '../../services/oidc_token_source.dart';
import '../../state/auth_flow.dart';
import '../../state/pending_social_link.dart';
import '../../widgets/app_snackbar.dart';
import '../auth/auth_messages.dart';
import '../auth/auth_scaffold.dart';

/// Collision resolution screen (change: add-account-identity-linking, D7): the
/// user proves ownership of a pre-existing account with a method they've used
/// before, and the app links the just-created social identity to it (deleting the
/// orphan first). The screen never claims an account exists nor discloses its
/// method — the user supplies the method by choosing it. The provider that minted
/// the orphan is hidden (re-authenticating with it would just re-enter the orphan).
class SignInToLinkScreen extends ConsumerStatefulWidget {
  const SignInToLinkScreen({required this.pending, super.key});

  final PendingSocialLink pending;

  @override
  ConsumerState<SignInToLinkScreen> createState() => _SignInToLinkScreenState();
}

class _SignInToLinkScreenState extends ConsumerState<SignInToLinkScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  bool _busy = false;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  /// Run [authenticateExisting] → delete orphan → adopt existing → link, then land
  /// the user on their existing account. A failure keeps the orphan intact and
  /// shows a friendly, context-appropriate message (never the raw error).
  Future<void> _link(Future<void> Function() run) async {
    final l10n = AppLocalizations.of(context);
    final messenger = ScaffoldMessenger.of(context);
    setState(() => _busy = true);
    try {
      await run();
      if (!mounted) return;
      showAppSnackBar(messenger, l10n.signInToLinkSuccess);
      Navigator.of(context).popUntil((r) => r.isFirst);
    } on AuthException catch (e) {
      if (mounted) showAuthError(context, e, fallback: l10n.authErrLinkFailed);
    } catch (_) {
      // A desktop OAuth / platform failure is not an AuthException.
      if (mounted) showAppSnackBar(messenger, l10n.authErrLinkFailed);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _linkWithEmail() {
    final email = _email.text.trim();
    final password = _password.text;
    if (email.isEmpty || password.isEmpty) return;
    _link(
      () => ref
          .read(authFlowProvider)
          .linkExisting(
            pending: widget.pending,
            authenticateExisting: () => ref
                .read(authFlowProvider)
                .reauthEmail(email: email, password: password),
          ),
    );
  }

  void _linkWithProvider(String provider) => _link(
    () => ref
        .read(authFlowProvider)
        .linkExisting(
          pending: widget.pending,
          authenticateExisting: () =>
              ref.read(authFlowProvider).reauthOidc(provider),
        ),
  );

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final oidc = ref.watch(oidcTokenSourceProvider);
    // Offer an existing method the user may have used — but never the provider
    // that created this orphan (re-authenticating with it re-enters the orphan).
    final offerGoogle =
        oidc.googleAvailable &&
        widget.pending.provider != LinkedIdentity.providerGoogle;
    final offerApple =
        oidc.appleAvailable &&
        widget.pending.provider != LinkedIdentity.providerApple;

    return AuthScaffold(
      title: l10n.signInToLinkTitle,
      children: [
        Text(l10n.signInToLinkIntro, textAlign: TextAlign.center),
        const SizedBox(height: 24),
        TextField(
          key: const Key('link-email'),
          controller: _email,
          keyboardType: TextInputType.emailAddress,
          autocorrect: false,
          decoration: InputDecoration(labelText: l10n.fieldEmail),
        ),
        const SizedBox(height: 16),
        TextField(
          key: const Key('link-password'),
          controller: _password,
          obscureText: true,
          decoration: InputDecoration(labelText: l10n.fieldPassword),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('link-with-email'),
          onPressed: _busy ? null : _linkWithEmail,
          child: _busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(l10n.signIn),
        ),
        if (offerGoogle) ...[
          const SizedBox(height: 8),
          OutlinedButton.icon(
            key: const Key('link-with-google'),
            onPressed: _busy
                ? null
                : () => _linkWithProvider(LinkedIdentity.providerGoogle),
            icon: const Icon(Icons.account_circle),
            label: Text(l10n.providerGoogle),
          ),
        ],
        if (offerApple) ...[
          const SizedBox(height: 8),
          OutlinedButton.icon(
            key: const Key('link-with-apple'),
            onPressed: _busy
                ? null
                : () => _linkWithProvider(LinkedIdentity.providerApple),
            icon: const Icon(Icons.apple),
            label: Text(l10n.providerApple),
          ),
        ],
      ],
    );
  }
}
