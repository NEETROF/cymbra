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
import 'delete_account_screen.dart';

/// App-bar account control. For a guest it offers to sign in / create an account
/// (leaving guest mode → entry screen). For a signed-in user it exposes sign-out
/// and account deletion. Account deletion is never shown to guests.
class AccountMenu extends ConsumerWidget {
  const AccountMenu({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final session = ref.watch(sessionNotifierProvider);
    return switch (session) {
      SessionGuest() => TextButton.icon(
        key: const Key('account-signin'),
        onPressed: () =>
            ref.read(sessionNotifierProvider.notifier).leaveGuest(),
        icon: const Icon(Icons.login),
        label: const Text('Sign in'),
      ),
      SessionAuthenticated(:final account) => PopupMenuButton<String>(
        key: const Key('account-menu'),
        icon: const Icon(Icons.account_circle),
        onSelected: (value) {
          switch (value) {
            case 'signout':
              ref.read(sessionNotifierProvider.notifier).signOut();
            case 'delete':
              Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const DeleteAccountScreen(),
                ),
              );
          }
        },
        itemBuilder: (context) => [
          if (account?.handle != null)
            PopupMenuItem<String>(
              enabled: false,
              child: Text('@${account!.handle}'),
            ),
          const PopupMenuItem<String>(
            value: 'signout',
            child: Text('Sign out'),
          ),
          const PopupMenuItem<String>(
            value: 'delete',
            child: Text('Delete account'),
          ),
        ],
      ),
      _ => const SizedBox.shrink(),
    };
  }
}
