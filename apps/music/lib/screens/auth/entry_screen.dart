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
import '../../services/oidc_token_source.dart';
import '../../state/auth_flow.dart';
import '../../state/session_notifier.dart';
import '../../theme/cymbra_theme.dart';
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
          const SnackBar(
            content: Text('Something went wrong. Please try again.'),
          ),
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
    final appleAvailable = ref.watch(oidcTokenSourceProvider).appleAvailable;

    return Scaffold(
      backgroundColor: CymbraColors.background,
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
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
                    style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      color: CymbraColors.onSurface,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Sign in to sync and share, or jump straight in.',
                    textAlign: TextAlign.center,
                    style: TextStyle(color: CymbraColors.onSurfaceVariant),
                  ),
                  const SizedBox(height: 40),
                  _EntryButton(
                    key: const Key('entry-google'),
                    icon: Icons.account_circle,
                    label: 'Continue with Google',
                    onPressed: _busy ? null : _continueWithGoogle,
                  ),
                  if (appleAvailable) ...[
                    const SizedBox(height: 12),
                    _EntryButton(
                      key: const Key('entry-apple'),
                      icon: Icons.apple,
                      label: 'Continue with Apple',
                      onPressed: _busy ? null : _continueWithApple,
                    ),
                  ],
                  const SizedBox(height: 12),
                  _EntryButton(
                    key: const Key('entry-email'),
                    icon: Icons.mail_outline,
                    label: 'Continue with email',
                    onPressed: _busy ? null : _continueWithEmail,
                  ),
                  const SizedBox(height: 24),
                  TextButton(
                    key: const Key('entry-guest'),
                    onPressed: _busy ? null : _continueAsGuest,
                    child: const Text('Continue without an account'),
                  ),
                  if (_busy) ...[
                    const SizedBox(height: 16),
                    const CircularProgressIndicator(),
                  ],
                ],
              ),
            ),
          ),
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
