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
import '../../state/onboarding_notifier.dart';
import '../../state/performance_scoring.dart';
import '../../state/score_catalog.dart';
import '../../theme/cymbra_theme.dart';
import '../../widgets/language_selector.dart';
import '../open_score.dart';
import 'sign_in_invitation.dart';

/// The first-run welcome (D1): two value screens and a first action, shown
/// **before any sign-in** and **without an account**.
///
/// It is always skippable (the Skip action is on every page) and never presents
/// sign-in as a wall — from the last page the user can try the app immediately,
/// sign in, or simply continue without an account.
class WelcomeScreen extends ConsumerStatefulWidget {
  const WelcomeScreen({super.key});

  @override
  ConsumerState<WelcomeScreen> createState() => _WelcomeScreenState();
}

class _WelcomeScreenState extends ConsumerState<WelcomeScreen> {
  final PageController _pages = PageController();
  int _page = 0;

  @override
  void dispose() {
    _pages.dispose();
    super.dispose();
  }

  /// Leaves the welcome for good — used by Skip, Sign in and Continue alike, so
  /// it never reappears whichever way the user left it.
  void _finish() => ref.read(onboardingProvider.notifier).completeWelcome();

  void _next() => _pages.nextPage(
    duration: const Duration(milliseconds: 220),
    curve: Curves.easeOut,
  );

  /// The no-account try (D2): play one of the bundled scores through the real
  /// player and scoring, with no sign-in and without opening the authenticated
  /// hub. When the run produced an end-of-session summary, sign-in is *offered*
  /// afterwards — declining simply returns to the welcome.
  Future<void> _tryNow() async {
    final entry = await _pickScore();
    if (entry == null || !mounted) return;
    ref.read(onboardingProvider.notifier).clearTryRun();
    await openScore(context, ref, entry);
    if (!mounted) return;
    if (!ref.read(onboardingProvider).tryRunFinished) return;
    ref.read(onboardingProvider.notifier).clearTryRun();
    final signedIn = await inviteSignIn(
      context,
      ref,
      SignInBenefit.keepProgress,
    );
    // Signing in ends the first run; declining leaves the user on the welcome,
    // free to try again, sign in later, or continue without an account.
    if (signedIn) _finish();
  }

  Future<CatalogEntry?> _pickScore() {
    final l10n = AppLocalizations.of(context);
    final catalog = ref.read(scoreCatalogProvider);
    return showDialog<CatalogEntry>(
      context: context,
      builder: (dialogContext) => SimpleDialog(
        key: const Key('welcome-try-picker'),
        backgroundColor: CymbraColors.surfaceContainerLow,
        title: Text(l10n.welcomeTryPickTitle),
        children: [
          for (final entry in catalog)
            SimpleDialogOption(
              key: Key('welcome-try-${entry.id}'),
              onPressed: () => Navigator.of(dialogContext).pop(entry),
              child: ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(
                  entry.title,
                  style: const TextStyle(color: CymbraColors.onSurface),
                ),
                subtitle: Text(
                  '${entry.composer} · ${entry.level.localizedLabel(l10n)}',
                  style: const TextStyle(color: CymbraColors.onSurfaceVariant),
                ),
              ),
            ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final pages = <_WelcomePage>[
      _WelcomePage(
        icon: Icons.piano,
        title: l10n.welcomeValueTitle,
        body: l10n.welcomeValueBody,
      ),
      _WelcomePage(
        icon: Icons.speed,
        title: l10n.welcomeLoopTitle,
        body: l10n.welcomeLoopBody,
      ),
      _WelcomePage(
        icon: Icons.play_circle_outline,
        title: l10n.welcomeStartTitle,
        body: l10n.welcomeStartBody,
      ),
    ];
    final last = _page == pages.length - 1;

    return _TryRunListener(
      child: Scaffold(
        backgroundColor: CymbraColors.background,
        body: SafeArea(
          child: Stack(
            children: [
              // Language stays reachable here too, so a mis-tap on the first
              // step is not a dead end.
              const Align(
                alignment: Alignment.topLeft,
                child: Padding(
                  padding: EdgeInsets.all(4),
                  child: LanguageSelectorButton(),
                ),
              ),
              Align(
                alignment: Alignment.topRight,
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: TextButton(
                    key: const Key('welcome-skip'),
                    onPressed: _finish,
                    child: Text(l10n.welcomeSkip),
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(24, 44, 24, 12),
                child: Column(
                  children: [
                    Expanded(
                      child: PageView(
                        key: const Key('welcome-pages'),
                        controller: _pages,
                        onPageChanged: (i) => setState(() => _page = i),
                        children: pages,
                      ),
                    ),
                    _Dots(count: pages.length, active: _page),
                    const SizedBox(height: 10),
                    if (last) ..._finalActions(l10n) else _nextButton(l10n),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _nextButton(AppLocalizations l10n) => SizedBox(
    width: double.infinity,
    child: FilledButton(
      key: const Key('welcome-next'),
      onPressed: _next,
      child: Text(l10n.welcomeNext),
    ),
  );

  List<Widget> _finalActions(AppLocalizations l10n) => [
    SizedBox(
      width: double.infinity,
      child: FilledButton.icon(
        key: const Key('welcome-try'),
        onPressed: _tryNow,
        icon: const Icon(Icons.play_arrow),
        label: Text(l10n.welcomeTryNow),
      ),
    ),
    const SizedBox(height: 4),
    Wrap(
      alignment: WrapAlignment.center,
      children: [
        TextButton(
          key: const Key('welcome-sign-in'),
          // Sign-in is an option, never a wall: it just leaves the welcome for
          // the entry screen, which still offers continuing without an account.
          onPressed: _finish,
          child: Text(l10n.signIn),
        ),
        TextButton(
          key: const Key('welcome-continue'),
          onPressed: _finish,
          child: Text(l10n.welcomeContinueWithoutAccount),
        ),
      ],
    ),
  ];
}

/// Dedicated listener (CLAUDE.md rule): a scored run finished during the
/// no-account try — record it on the onboarding notifier so the welcome can
/// offer sign-in once the player is left. It stays mounted under the pushed
/// player, so the summary is observed even though the welcome is not visible.
class _TryRunListener extends ConsumerWidget {
  const _TryRunListener({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(performanceScorerProvider.select((s) => s.lastResult), (
      previous,
      next,
    ) {
      if (next != null && !identical(next, previous)) {
        ref.read(onboardingProvider.notifier).markTryRunFinished();
      }
    });
    return child;
  }
}

/// One welcome page: icon, title, body — centred and scrollable so it holds up
/// on the short landscape viewport the app is locked to.
class _WelcomePage extends StatelessWidget {
  const _WelcomePage({
    required this.icon,
    required this.title,
    required this.body,
  });

  final IconData icon;
  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 52, color: CymbraColors.primary),
              const SizedBox(height: 14),
              Text(
                title,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 20,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const SizedBox(height: 10),
              Text(
                body,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 14.5,
                  height: 1.35,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Page indicator dots (decorative — the pages themselves carry the content).
class _Dots extends StatelessWidget {
  const _Dots({required this.count, required this.active});

  final int count;
  final int active;

  @override
  Widget build(BuildContext context) {
    return ExcludeSemantics(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          for (var i = 0; i < count; i++)
            Container(
              width: 7,
              height: 7,
              margin: const EdgeInsets.symmetric(horizontal: 3),
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: i == active
                    ? CymbraColors.primary
                    : CymbraColors.outline,
              ),
            ),
        ],
      ),
    );
  }
}
