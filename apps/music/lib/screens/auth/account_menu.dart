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
import '../../state/curator_profile_notifier.dart';
import '../../state/session_notifier.dart';
import '../../state/session_state.dart';
import '../../state/usage_consent.dart';
import '../../theme/cymbra_theme.dart';
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
        // The curator standing (level) + unseen-award dot are merged onto the
        // account icon (change: add-curation-rewards) — the rewards themselves
        // live in the profile, reachable from this menu's "profile" entry.
        icon: const _AccountRewardIcon(),
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
            case 'usage-consent':
              // Toggle first-party usage-analytics consent (change: add-feature-
              // usage-analytics). UI calls the notifier, never the service.
              ref
                  .read(usageConsentProvider.notifier)
                  .set(!ref.read(usageConsentProvider));
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
          CheckedPopupMenuItem<String>(
            key: const Key('account-usage-consent'),
            value: 'usage-consent',
            checked: ref.watch(usageConsentProvider),
            child: Text(l10n.usageAnalyticsSetting),
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

/// The account icon with the curator standing merged in (change: add-curation-
/// rewards): the person glyph, a small level badge, and a dot when deferred
/// honesty awards have landed since the profile was last opened. Reads the reward
/// providers directly (a value read), so it stays live as level/points change.
class _AccountRewardIcon extends ConsumerWidget {
  const _AccountRewardIcon();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final level = ref.watch(curatorProfileProvider).valueOrNull?.level;
    final hasUnseen = ref.watch(curatorHasUnseenAwardsProvider);
    return Stack(
      clipBehavior: Clip.none,
      children: [
        const Icon(Icons.account_circle),
        if (level != null && level > 0)
          Positioned(
            right: -7,
            bottom: -5,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
              decoration: BoxDecoration(
                color: CymbraColors.primary,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: CymbraColors.surfaceContainerLowest,
                  width: 1.5,
                ),
              ),
              child: Text(
                '$level',
                style: const TextStyle(
                  color: CymbraColors.surfaceContainerLowest,
                  fontSize: 10,
                  fontWeight: FontWeight.w800,
                  height: 1,
                ),
              ),
            ),
          ),
        if (hasUnseen)
          Positioned(
            right: -1,
            top: -1,
            child: Container(
              key: const Key('account-award-dot'),
              width: 9,
              height: 9,
              decoration: BoxDecoration(
                color: CymbraColors.error,
                shape: BoxShape.circle,
                border: Border.all(
                  color: CymbraColors.surfaceContainerLowest,
                  width: 1.5,
                ),
              ),
            ),
          ),
      ],
    );
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
