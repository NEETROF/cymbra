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

import 'foreground_notification.dart';

/// Wires foreground push messages into the in-app banner (change:
/// add-foreground-notifications, design D5).
///
/// A dedicated listener widget near the top of the app subtree — the
/// `PushRegistrationListener` shape — isolating both `ref.listen` side effects:
/// delivered messages are handed to the [ForegroundNotification] notifier
/// (which alone decides whether anything surfaces), and a tapped banner's
/// routing payload is resolved and navigated here. It sits **inside** the
/// navigator (the banner layer paints above it), so `Navigator.of(context)`
/// pushes onto the root navigator. The UI never touches the push service.
class ForegroundNotificationListener extends ConsumerWidget {
  const ForegroundNotificationListener({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(foregroundMessagesProvider, (previous, next) {
      final message = next.valueOrNull;
      if (message == null) return;
      ref.read(foregroundNotificationProvider.notifier).show(message);
    });
    ref.listen(foregroundNotificationProvider, (previous, next) {
      final route = next.tappedRoute;
      if (route == null || route == previous?.tappedRoute) return;
      // Consume before navigating so a re-entrant state change cannot replay
      // the push; an unknown route (an older build receiving a newer category)
      // has already dismissed the banner and simply goes nowhere.
      ref.read(foregroundNotificationProvider.notifier).routeHandled();
      final builder = ref.read(pushRouteBuildersProvider)[route];
      if (builder != null) {
        Navigator.of(context).push(MaterialPageRoute<void>(builder: builder));
      }
    });
    return child;
  }
}
