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

/// Widest a toast gets; beyond this it would read as a banner rather than a
/// transient confirmation.
const double _maxToastWidth = 420;

/// Shows [message] as a **toast**: a floating card pinned to the bottom-RIGHT,
/// auto-dismissing, **replacing** any current or queued one first (so messages
/// never stack up) and carrying a close button for manual dismissal.
///
/// Bottom-right rather than the Material default (fixed, full-width, centred):
/// the app is landscape-locked, so a full-width bar across the bottom covers the
/// keyboard and the transport controls — exactly the surfaces the user is looking
/// at when the message arrives.
///
/// Takes a [ScaffoldMessengerState] rather than a `BuildContext` so it is safe to
/// call after an `await` (capture `ScaffoldMessenger.of(context)` before the gap).
void showAppSnackBar(ScaffoldMessengerState messenger, String message) {
  // The messenger carries its own context, so the toast can size itself against
  // the real viewport even when the caller's context is already gone.
  final width = MediaQuery.maybeSizeOf(messenger.context)?.width ?? 0;
  final left = (width - _maxToastWidth - 16).clamp(16.0, double.infinity);
  messenger
    ..clearSnackBars()
    ..showSnackBar(
      SnackBar(
        content: Text(message),
        showCloseIcon: true,
        behavior: SnackBarBehavior.floating,
        margin: EdgeInsets.only(left: left, right: 16, bottom: 16),
      ),
    );
}
