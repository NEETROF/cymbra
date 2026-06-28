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

import '../../state/session_notifier.dart';
import '../../state/session_state.dart';
import '../../theme/cymbra_theme.dart';
import '../library_screen.dart';
import 'entry_screen.dart';
import 'handle_onboarding_screen.dart';

/// The app's home: routes on the resolved [SessionState] (design D2). A returning
/// user (or guest) skips the entry screen; a signed-in user without a handle is
/// sent through onboarding before the library.
class SessionGate extends ConsumerWidget {
  const SessionGate({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final session = ref.watch(sessionNotifierProvider);
    return switch (session) {
      SessionUnknown() => const _SplashLoader(),
      SessionUnauthenticated() => const EntryScreen(),
      SessionGuest() => const LibraryScreen(),
      SessionAuthenticated() =>
        session.needsHandle
            ? const HandleOnboardingScreen()
            : const LibraryScreen(),
    };
  }
}

/// Shown while the session is being hydrated from secure storage.
class _SplashLoader extends StatelessWidget {
  const _SplashLoader();

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      backgroundColor: CymbraColors.background,
      body: Center(child: CircularProgressIndicator()),
    );
  }
}
