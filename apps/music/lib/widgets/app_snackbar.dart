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

/// Shows [message] as a snackbar, **replacing** any current or queued one first
/// (so messages never stack up) and with a **close button** so the user can
/// dismiss it manually.
///
/// Takes a [ScaffoldMessengerState] rather than a `BuildContext` so it is safe to
/// call after an `await` (capture `ScaffoldMessenger.of(context)` before the gap).
void showAppSnackBar(ScaffoldMessengerState messenger, String message) {
  messenger
    ..clearSnackBars()
    ..showSnackBar(SnackBar(content: Text(message), showCloseIcon: true));
}
