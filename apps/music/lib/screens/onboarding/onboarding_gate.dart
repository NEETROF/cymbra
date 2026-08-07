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

import '../../state/onboarding_notifier.dart';
import '../../state/session_notifier.dart';
import '../../state/session_state.dart';
import '../auth/session_gate.dart';
import 'language_step_screen.dart';
import 'welcome_screen.dart';

/// The app's home, ahead of the account [SessionGate]: it runs the first-run
/// sequence **language → welcome** and only then hands over to the session
/// routing (optional sign-in → handle gate → app).
///
/// Neither first-run step requires an account. The welcome is shown only while
/// no session exists — a returning (signed-in or guest) user goes straight to
/// the app, and skipping the welcome lands on the entry screen, which still
/// offers continuing without an account.
class OnboardingGate extends ConsumerWidget {
  const OnboardingGate({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final onboarding = ref.watch(onboardingProvider);
    // Flags unknown yet: show the same neutral splash as the session gate rather
    // than flashing the language step at a returning user.
    if (!onboarding.loaded) return const SplashLoader();
    if (onboarding.needsLanguage) return const LanguageStepScreen();
    if (onboarding.needsWelcome) {
      final session = ref.watch(sessionNotifierProvider);
      return switch (session) {
        SessionUnknown() => const SplashLoader(),
        SessionUnauthenticated() => const WelcomeScreen(),
        _ => const SessionGate(),
      };
    }
    return const SessionGate();
  }
}
