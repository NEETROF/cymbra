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
import '../../services/oidc_token_source.dart';
import '../../state/connected_accounts_notifier.dart';
import '../../state/connected_accounts_state.dart';
import '../../widgets/app_snackbar.dart';
import '../auth/auth_messages.dart';
import 'set_password_screen.dart';

/// Connected accounts (change: add-account-identity-linking): lists the sign-in
/// identities on the current account and offers link/unlink actions. Reached from
/// the account menu; only signed-in users can open it (guests never see it).
class ConnectedAccountsScreen extends ConsumerWidget {
  const ConnectedAccountsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(title: Text(l10n.connectedAccountsTitle)),
      body: const ConnectedAccountsActionListener(child: _Body()),
    );
  }
}

/// Isolated listener for the screen's action side effects (snackbars): fires once
/// per completed action (keyed off `actionSeq`) and maps a failure to a friendly,
/// action-aware message — the raw error is never shown.
class ConnectedAccountsActionListener extends ConsumerWidget {
  const ConnectedAccountsActionListener({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(connectedAccountsNotifierProvider.select((s) => s.actionSeq), (
      previous,
      next,
    ) {
      if (previous == null || next == previous) return;
      final state = ref.read(connectedAccountsNotifierProvider);
      final action = state.lastAction;
      if (action == null) return;
      final l10n = AppLocalizations.of(context);
      final messenger = ScaffoldMessenger.of(context);
      final error = state.actionError;
      if (error != null) {
        showAppSnackBar(messenger, linkErrorMessage(l10n, error, action));
      } else {
        showAppSnackBar(
          messenger,
          action == ConnectedAccountsAction.setPassword
              ? l10n.setPasswordSuccess
              : l10n.linkSuccess,
        );
      }
    });
    return child;
  }
}

class _Body extends ConsumerWidget {
  const _Body();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(connectedAccountsNotifierProvider);
    final notifier = ref.read(connectedAccountsNotifierProvider.notifier);

    return state.identities.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => _ErrorRetry(
        message: l10n.connectedAccountsError,
        onRetry: notifier.load,
      ),
      data: (identities) => _Content(state: state),
    );
  }
}

class _Content extends ConsumerWidget {
  const _Content({required this.state});

  final ConnectedAccountsState state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final notifier = ref.read(connectedAccountsNotifierProvider.notifier);
    final oidc = ref.watch(oidcTokenSourceProvider);
    final busy = state.busy;

    return ListView(
      padding: const EdgeInsets.symmetric(vertical: 12),
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 4, 16, 12),
          child: Text(l10n.connectedAccountsIntro),
        ),
        if (state.items.isEmpty)
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(l10n.connectedAccountsEmpty),
          ),
        for (final identity in state.items)
          _IdentityRow(
            identity: identity,
            isLast: state.isLastIdentity,
            busy: busy,
            onUnlink: () => _confirmUnlink(context, notifier, identity),
          ),
        const Divider(height: 24),
        // Offer a link action only for providers not present AND available on
        // this platform (already-linked identities are still listed above).
        if (!state.hasGoogle && oidc.googleAvailable)
          _LinkAction(
            key: const Key('link-google'),
            icon: Icons.account_circle,
            label: l10n.linkGoogleAction,
            onPressed: busy ? null : notifier.linkGoogle,
          ),
        if (!state.hasApple && oidc.appleAvailable)
          _LinkAction(
            key: const Key('link-apple'),
            icon: Icons.apple,
            label: l10n.linkAppleAction,
            onPressed: busy ? null : notifier.linkApple,
          ),
        if (!state.hasLocal)
          _LinkAction(
            key: const Key('set-password'),
            icon: Icons.password,
            label: l10n.setPasswordAction,
            onPressed: busy
                ? null
                : () => Navigator.of(context).push(
                    MaterialPageRoute<void>(
                      builder: (_) => const SetPasswordScreen(),
                    ),
                  ),
          ),
      ],
    );
  }

  Future<void> _confirmUnlink(
    BuildContext context,
    ConnectedAccountsNotifier notifier,
    LinkedIdentity identity,
  ) async {
    final l10n = AppLocalizations.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.identityRemove),
        content: Text(_providerLabel(l10n, identity.provider)),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(l10n.cancel),
          ),
          TextButton(
            key: const Key('unlink-confirm'),
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(l10n.identityRemove),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    await notifier.unlink(
      provider: identity.provider,
      subject: identity.subject,
    );
  }
}

class _IdentityRow extends StatelessWidget {
  const _IdentityRow({
    required this.identity,
    required this.isLast,
    required this.busy,
    required this.onUnlink,
  });

  final LinkedIdentity identity;
  final bool isLast;
  final bool busy;
  final VoidCallback onUnlink;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final date = MaterialLocalizations.of(
      context,
    ).formatShortDate(identity.linkedAt);
    return ListTile(
      key: Key('identity-${identity.provider}'),
      leading: Icon(_providerIcon(identity.provider)),
      title: Text(_providerLabel(l10n, identity.provider)),
      subtitle: Text(l10n.identityLinkedOn(date)),
      trailing: IconButton(
        key: Key('unlink-${identity.provider}'),
        icon: const Icon(Icons.link_off),
        // The last identity can never be unlinked (anti-lockout); explain why.
        tooltip: isLast ? l10n.identityOnlyMethodNote : l10n.identityRemove,
        onPressed: (isLast || busy) ? null : onUnlink,
      ),
    );
  }
}

class _LinkAction extends StatelessWidget {
  const _LinkAction({
    required this.icon,
    required this.label,
    required this.onPressed,
    super.key,
  });

  final IconData icon;
  final String label;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: OutlinedButton.icon(
        onPressed: onPressed,
        icon: Icon(icon),
        label: Align(alignment: Alignment.centerLeft, child: Text(label)),
      ),
    );
  }
}

class _ErrorRetry extends StatelessWidget {
  const _ErrorRetry({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(message, textAlign: TextAlign.center),
          const SizedBox(height: 12),
          FilledButton(onPressed: onRetry, child: Text(l10n.retry)),
        ],
      ),
    );
  }
}

IconData _providerIcon(String provider) => switch (provider) {
  LinkedIdentity.providerGoogle => Icons.account_circle,
  LinkedIdentity.providerApple => Icons.apple,
  _ => Icons.email,
};

String _providerLabel(AppLocalizations l10n, String provider) =>
    switch (provider) {
      LinkedIdentity.providerGoogle => l10n.providerGoogle,
      LinkedIdentity.providerApple => l10n.providerApple,
      _ => l10n.providerEmailPassword,
    };
