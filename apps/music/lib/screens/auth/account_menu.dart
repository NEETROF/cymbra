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
import '../../services/legal_links.dart';
import '../../state/app_language.dart';
import '../../state/app_locale.dart';
import '../../state/session_notifier.dart';
import '../../state/session_state.dart';
import '../../widgets/language_selector.dart' show showLanguageDialog;
import '../account/connected_accounts_screen.dart';
import '../profile_screen.dart';
import 'delete_account_screen.dart';

/// App-bar account control. For a guest it offers to sign in / create an account
/// (leaving guest mode → entry screen). For a signed-in user it exposes sign-out
/// and account deletion. Account deletion is never shown to guests.
class AccountMenu extends ConsumerWidget {
  const AccountMenu({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final session = ref.watch(sessionNotifierProvider);
    final l10n = AppLocalizations.of(context);
    // Legal pages follow the active language (fr → French pages, else English)
    // and open in an external browser through the injectable launcher seam.
    final links = legalLinksFor(ref.watch(appLocaleProvider).languageCode);
    final launcher = ref.read(legalLinkLauncherProvider);
    // Language now lives in this menu (moved off the home top bar): its row shows
    // the active language's flag.
    final activeLanguage =
        AppLanguage.fromCode(ref.watch(appLocaleProvider).languageCode) ??
        AppLanguage.en;
    return switch (session) {
      SessionGuest() => TextButton.icon(
        key: const Key('account-signin'),
        onPressed: () =>
            ref.read(sessionNotifierProvider.notifier).leaveGuest(),
        icon: const Icon(Icons.login),
        label: Text(l10n.signIn),
      ),
      SessionAuthenticated(:final account) => PopupMenuButton<String>(
        key: const Key('account-menu'),
        icon: const Icon(Icons.account_circle),
        onSelected: (value) {
          switch (value) {
            case 'profile':
              Navigator.of(context).push(
                MaterialPageRoute<void>(builder: (_) => const ProfileScreen()),
              );
            case 'connected':
              Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const ConnectedAccountsScreen(),
                ),
              );
            case 'language':
              showLanguageDialog(context, ref);
            case 'signout':
              ref.read(sessionNotifierProvider.notifier).signOut();
            case 'signout-all':
              _confirmSignOutEverywhere(context, ref, l10n);
            case 'delete':
              Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const DeleteAccountScreen(),
                ),
              );
            case 'terms':
              launcher.open(links.terms);
            case 'privacy':
              launcher.open(links.privacy);
          }
        },
        itemBuilder: (context) => [
          if (account?.handle != null)
            PopupMenuItem<String>(
              enabled: false,
              child: Text('@${account!.handle}'),
            ),
          PopupMenuItem<String>(
            key: const Key('account-profile'),
            value: 'profile',
            child: Text(l10n.accountProfile),
          ),
          PopupMenuItem<String>(
            key: const Key('account-connected'),
            value: 'connected',
            child: Text(l10n.connectedAccountsManage),
          ),
          PopupMenuItem<String>(
            key: const Key('account-language'),
            value: 'language',
            child: Row(
              children: [
                Text(activeLanguage.flag, style: const TextStyle(fontSize: 18)),
                const SizedBox(width: 12),
                Expanded(child: Text(l10n.settingsCategoryLanguage)),
              ],
            ),
          ),
          PopupMenuItem<String>(
            value: 'signout',
            child: Text(l10n.accountSignOut),
          ),
          PopupMenuItem<String>(
            key: const Key('account-signout-all'),
            value: 'signout-all',
            child: Text(l10n.accountSignOutAll),
          ),
          PopupMenuItem<String>(
            value: 'delete',
            child: Text(l10n.accountDelete),
          ),
          const PopupMenuDivider(),
          PopupMenuItem<String>(
            key: const Key('account-legal-terms'),
            value: 'terms',
            child: _LegalMenuRow(
              icon: Icons.description_outlined,
              label: l10n.legalTerms,
            ),
          ),
          PopupMenuItem<String>(
            key: const Key('account-legal-privacy'),
            value: 'privacy',
            child: _LegalMenuRow(
              icon: Icons.privacy_tip_outlined,
              label: l10n.legalPrivacy,
            ),
          ),
        ],
      ),
      _ => const SizedBox.shrink(),
    };
  }

  /// Confirm, then revoke every session. On success the notifier tears down the
  /// local session (routing back to entry); on failure the session is kept and a
  /// generic, localized error is shown — the raw exception is never surfaced.
  Future<void> _confirmSignOutEverywhere(
    BuildContext context,
    WidgetRef ref,
    AppLocalizations l10n,
  ) async {
    // Capture the messenger before any `await` so we don't touch `context`
    // across an async gap.
    final messenger = ScaffoldMessenger.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(l10n.accountSignOutAllTitle),
        content: Text(l10n.accountSignOutAllBody),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: Text(l10n.cancel),
          ),
          TextButton(
            key: const Key('account-signout-all-confirm'),
            onPressed: () => Navigator.of(dialogContext).pop(true),
            child: Text(l10n.accountSignOutAllConfirm),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      await ref.read(sessionNotifierProvider.notifier).signOutEverywhere();
    } catch (_) {
      // Session kept (the revoke failed): tell the user, never the raw error.
      messenger.showSnackBar(
        SnackBar(content: Text(l10n.accountSignOutAllError)),
      );
    }
  }
}

/// A legal-link row in the account menu: an icon, the label, and a trailing
/// "opens externally" hint so it reads as leaving the app rather than an
/// in-app action like sign-out or delete.
class _LegalMenuRow extends StatelessWidget {
  const _LegalMenuRow({required this.icon, required this.label});

  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 20),
        const SizedBox(width: 12),
        Expanded(child: Text(label)),
        const SizedBox(width: 12),
        const Icon(Icons.open_in_new, size: 16),
      ],
    );
  }
}
