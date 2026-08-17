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
import '../../services/oidc_token_source.dart';
import '../../state/auth_flow.dart';
import '../../state/session_notifier.dart';
import '../../theme/cymbra_theme.dart';
import '../../widgets/app_snackbar.dart';
import '../auth/auth_messages.dart';
import '../auth/auth_scaffold.dart';
import '../auth/email_sign_up_screen.dart';

/// What the user gains by signing in, for the action they just reached. The
/// invitation always names one of these — never a bare "sign in" wall (D3).
enum SignInBenefit {
  /// Saving scores to a personal library.
  saveLibrary,

  /// Earning points by rating scores.
  earnPoints,

  /// Appearing on the leaderboards.
  leaderboards,

  /// Making a curator profile public.
  goPublic,

  /// Keeping progress and library after the no-account try.
  keepProgress,
}

/// The localized benefit line shown in the invitation.
String signInBenefitMessage(AppLocalizations l10n, SignInBenefit benefit) =>
    switch (benefit) {
      SignInBenefit.saveLibrary => l10n.signInInviteBenefitLibrary,
      SignInBenefit.earnPoints => l10n.signInInviteBenefitPoints,
      SignInBenefit.leaderboards => l10n.signInInviteBenefitLeaderboards,
      SignInBenefit.goPublic => l10n.signInInviteBenefitPublic,
      SignInBenefit.keepProgress => l10n.signInInviteBenefitProgress,
    };

/// Route name of the invitation screen, so the sign-in listener pops exactly
/// this route (and any sub-screen pushed above it) when the session resolves.
const String signInInvitationRouteName = 'sign-in-invitation';

/// Invites sign-in for an account-gated [benefit] and reports whether the caller
/// may now proceed (D3).
///
/// - Already signed in → `true` immediately, no interruption.
/// - Declined → `false`; the caller does nothing and the user stays exactly
///   where they were (never a dead end).
/// - Accepted → the sign-in surface is pushed; once the session resolves the
///   route pops and this returns `true`, so the caller **resumes** the action
///   the user originally intended.
Future<bool> inviteSignIn(
  BuildContext context,
  WidgetRef ref,
  SignInBenefit benefit,
) async {
  if (ref.read(canUseOnlineServicesProvider)) return true;
  final l10n = AppLocalizations.of(context);
  final accepted = await showDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      key: const Key('sign-in-invitation'),
      backgroundColor: CymbraColors.surfaceContainerLow,
      title: Text(l10n.signInInviteTitle),
      content: Text(signInBenefitMessage(l10n, benefit)),
      actions: [
        TextButton(
          key: const Key('sign-in-invitation-decline'),
          onPressed: () => Navigator.of(dialogContext).pop(false),
          child: Text(l10n.signInInviteDecline),
        ),
        FilledButton(
          key: const Key('sign-in-invitation-accept'),
          onPressed: () => Navigator.of(dialogContext).pop(true),
          child: Text(l10n.signInInviteAccept),
        ),
      ],
    ),
  );
  if (accepted != true || !context.mounted) return false;
  await Navigator.of(context).push(
    MaterialPageRoute<void>(
      settings: const RouteSettings(name: signInInvitationRouteName),
      builder: (_) => SignInInvitationScreen(benefit: benefit),
    ),
  );
  return ref.read(canUseOnlineServicesProvider);
}

/// The sign-in surface reached from a contextual invitation. Unlike the launch
/// entry screen it is a **pushed** route: completing sign-in pops back to the
/// action the user was trying to perform instead of restarting at the app root.
class SignInInvitationScreen extends ConsumerStatefulWidget {
  const SignInInvitationScreen({required this.benefit, super.key});

  final SignInBenefit benefit;

  @override
  ConsumerState<SignInInvitationScreen> createState() =>
      _SignInInvitationScreenState();
}

class _SignInInvitationScreenState
    extends ConsumerState<SignInInvitationScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  bool _busy = false;

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    super.dispose();
  }

  Future<void> _run(Future<void> Function() action) async {
    final l10n = AppLocalizations.of(context);
    final messenger = ScaffoldMessenger.of(context);
    setState(() => _busy = true);
    try {
      await action();
    } on AuthException catch (e) {
      if (mounted) {
        showAuthError(context, e, fallback: l10n.authErrUnauthenticated);
      }
    } catch (e) {
      // A native SDK / platform failure must not break the invitation — but it
      // must be traceable (release builds only surface the generic message).
      debugPrint('sign-in: platform failure: $e');
      if (mounted) showAppSnackBar(messenger, l10n.entryError);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _signInWithEmail() {
    final email = _email.text.trim();
    final password = _password.text;
    if (email.isEmpty || password.isEmpty) return;
    _run(
      () => ref
          .read(authFlowProvider)
          .signInEmail(email: email, password: password),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final oidc = ref.watch(oidcTokenSourceProvider);
    return _ResumeOnSignedIn(
      child: AuthScaffold(
        title: l10n.signIn,
        children: [
          Text(
            signInBenefitMessage(l10n, widget.benefit),
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.onSurfaceVariant),
          ),
          const SizedBox(height: 24),
          if (oidc.googleAvailable) ...[
            OutlinedButton.icon(
              key: const Key('invite-google'),
              onPressed: _busy
                  ? null
                  : () => _run(
                      () => ref.read(authFlowProvider).signInWithGoogle(),
                    ),
              icon: const Icon(Icons.account_circle),
              label: Text(l10n.entryContinueGoogle),
            ),
            const SizedBox(height: 8),
          ],
          if (oidc.appleAvailable) ...[
            OutlinedButton.icon(
              key: const Key('invite-apple'),
              onPressed: _busy
                  ? null
                  : () => _run(
                      () => ref.read(authFlowProvider).signInWithApple(),
                    ),
              icon: const Icon(Icons.apple),
              label: Text(l10n.entryContinueApple),
            ),
            const SizedBox(height: 8),
          ],
          TextField(
            key: const Key('invite-email'),
            controller: _email,
            keyboardType: TextInputType.emailAddress,
            autocorrect: false,
            decoration: InputDecoration(labelText: l10n.fieldEmail),
          ),
          const SizedBox(height: 16),
          TextField(
            key: const Key('invite-password'),
            controller: _password,
            obscureText: true,
            decoration: InputDecoration(labelText: l10n.fieldPassword),
          ),
          const SizedBox(height: 24),
          FilledButton(
            key: const Key('invite-submit'),
            onPressed: _busy ? null : _signInWithEmail,
            child: _busy
                ? const SizedBox.square(
                    dimension: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(l10n.signIn),
          ),
          TextButton(
            key: const Key('invite-create'),
            onPressed: _busy
                ? null
                : () => Navigator.of(context).push(
                    MaterialPageRoute<void>(
                      builder: (_) => const EmailSignUpScreen(),
                    ),
                  ),
            child: Text(l10n.signInCreateAccount),
          ),
        ],
      ),
    );
  }
}

/// Dedicated listener (CLAUDE.md rule): when the session becomes usable online,
/// unwind back to the screen that asked for sign-in so the intended action can
/// resume. Popping is scoped to [signInInvitationRouteName], so any sub-screen
/// pushed above the invitation (verification, sign-up) is unwound with it.
class _ResumeOnSignedIn extends ConsumerWidget {
  const _ResumeOnSignedIn({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(canUseOnlineServicesProvider, (_, signedIn) {
      if (!signedIn) return;
      final navigator = Navigator.of(context);
      navigator.popUntil(
        (route) =>
            route.settings.name == signInInvitationRouteName || route.isFirst,
      );
      if (navigator.canPop()) navigator.pop();
    });
    return child;
  }
}
