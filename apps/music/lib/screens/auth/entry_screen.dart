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
import '../../services/auth_service.dart';
import '../../services/legal_links.dart';
import '../../services/oidc_token_source.dart';
import '../../state/app_locale.dart';
import '../../state/auth_flow.dart';
import '../../state/session_notifier.dart';
import '../../theme/cymbra_theme.dart';
import '../../widgets/language_selector.dart';
import 'auth_messages.dart';
import 'email_sign_in_screen.dart';

/// The launch entry experience (spec: "Account entry is the launch experience").
/// Offers exactly four mutually-exclusive choices — Google, Apple, email, and
/// guest — on the Cymbra dark theme. Shown only when the session is
/// `unauthenticated`.
class EntryScreen extends ConsumerStatefulWidget {
  const EntryScreen({super.key});

  @override
  ConsumerState<EntryScreen> createState() => _EntryScreenState();
}

class _EntryScreenState extends ConsumerState<EntryScreen> {
  bool _busy = false;

  Future<void> _run(Future<void> Function() action) async {
    setState(() => _busy = true);
    try {
      await action();
    } on AuthException catch (e) {
      if (mounted) showAuthError(context, e);
    } catch (e) {
      // A native SDK / platform failure must not crash the entry screen.
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(AppLocalizations.of(context).entryError)),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _continueWithGoogle() =>
      _run(() => ref.read(authFlowProvider).signInWithGoogle());

  void _continueWithApple() =>
      _run(() => ref.read(authFlowProvider).signInWithApple());

  void _continueWithEmail() {
    Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => const EmailSignInScreen()));
  }

  void _continueAsGuest() =>
      _run(() => ref.read(sessionNotifierProvider.notifier).continueAsGuest());

  @override
  Widget build(BuildContext context) {
    final oidc = ref.watch(oidcTokenSourceProvider);
    final googleAvailable = oidc.googleAvailable;
    final appleAvailable = oidc.appleAvailable;
    final l10n = AppLocalizations.of(context);

    return Scaffold(
      backgroundColor: CymbraColors.background,
      body: SafeArea(
        child: Stack(
          children: [
            // Language switcher reachable before signing in (top-right corner).
            const Align(
              alignment: Alignment.topRight,
              child: Padding(
                padding: EdgeInsets.all(4),
                child: LanguageSelectorButton(),
              ),
            ),
            Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.symmetric(
                  horizontal: 32,
                  vertical: 24,
                ),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 420),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(
                        Icons.piano,
                        size: 72,
                        color: CymbraColors.primary,
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'Cymbra',
                        style: Theme.of(context).textTheme.headlineMedium
                            ?.copyWith(
                              color: CymbraColors.onSurface,
                              fontWeight: FontWeight.w800,
                            ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        l10n.entrySubtitle,
                        textAlign: TextAlign.center,
                        style: const TextStyle(
                          color: CymbraColors.onSurfaceVariant,
                        ),
                      ),
                      const SizedBox(height: 40),
                      if (googleAvailable) ...[
                        _EntryButton(
                          key: const Key('entry-google'),
                          icon: Icons.account_circle,
                          label: l10n.entryContinueGoogle,
                          onPressed: _busy ? null : _continueWithGoogle,
                        ),
                        const SizedBox(height: 12),
                      ],
                      if (appleAvailable) ...[
                        _EntryButton(
                          key: const Key('entry-apple'),
                          icon: Icons.apple,
                          label: l10n.entryContinueApple,
                          onPressed: _busy ? null : _continueWithApple,
                        ),
                        const SizedBox(height: 12),
                      ],
                      _EntryButton(
                        key: const Key('entry-email'),
                        icon: Icons.mail_outline,
                        label: l10n.entryContinueEmail,
                        onPressed: _busy ? null : _continueWithEmail,
                      ),
                      const SizedBox(height: 24),
                      TextButton(
                        key: const Key('entry-guest'),
                        onPressed: _busy ? null : _continueAsGuest,
                        child: Text(l10n.entryContinueGuest),
                      ),
                      const SizedBox(height: 16),
                      const _LegalConsent(),
                      if (_busy) ...[
                        const SizedBox(height: 16),
                        const CircularProgressIndicator(),
                      ],
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// A full-width branded entry option button.
class _EntryButton extends StatelessWidget {
  const _EntryButton({
    super.key,
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  final IconData icon;
  final String label;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: FilledButton.icon(
        onPressed: onPressed,
        icon: Icon(icon),
        label: Text(label),
        style: FilledButton.styleFrom(
          backgroundColor: CymbraColors.surfaceContainerHigh,
          foregroundColor: CymbraColors.onSurface,
          padding: const EdgeInsets.symmetric(vertical: 16),
          alignment: Alignment.center,
        ),
      ),
    );
  }
}

/// Consent notice under the entry options: "By continuing, you accept our
/// Terms and Privacy Policy." The two labels open the locale-resolved legal pages
/// in an external browser via the injected launcher (RGPD
/// acceptance-by-continuing at account entry). URLs follow the active locale, so
/// `fr` opens the French pages and everything else the English ones.
class _LegalConsent extends ConsumerWidget {
  const _LegalConsent();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final links = legalLinksFor(ref.watch(appLocaleProvider).languageCode);
    final launcher = ref.read(legalLinkLauncherProvider);

    const bodyStyle = TextStyle(
      color: CymbraColors.onSurfaceVariant,
      fontSize: 12,
      height: 1.4,
    );
    const linkStyle = TextStyle(
      color: CymbraColors.primary,
      fontSize: 12,
      height: 1.4,
      decoration: TextDecoration.underline,
      decorationColor: CymbraColors.primary,
    );

    Widget link(Key key, String label, VoidCallback onTap) => GestureDetector(
      key: key,
      onTap: onTap,
      child: Text(label, style: linkStyle),
    );

    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 360),
      child: Text.rich(
        TextSpan(
          children: [
            TextSpan(text: l10n.entryLegalPrefix),
            WidgetSpan(
              alignment: PlaceholderAlignment.middle,
              child: link(
                const Key('entry-legal-terms'),
                l10n.legalTerms,
                () => launcher.open(links.terms),
              ),
            ),
            TextSpan(text: l10n.entryLegalSeparator),
            WidgetSpan(
              alignment: PlaceholderAlignment.middle,
              child: link(
                const Key('entry-legal-privacy'),
                l10n.legalPrivacy,
                () => launcher.open(links.privacy),
              ),
            ),
            TextSpan(text: l10n.entryLegalSuffix),
          ],
        ),
        textAlign: TextAlign.center,
        style: bodyStyle,
      ),
    );
  }
}
