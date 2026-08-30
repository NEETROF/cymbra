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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../screens/update_required_screen.dart';
import '../widgets/update_prompt.dart';
import 'update_notifier.dart';
import 'update_state.dart';

/// Turns updater state into effects (change: add-desktop-auto-update, task 8.5).
///
/// The one place `ref.listen` drives navigation and dialogs for this feature —
/// a dedicated listener widget near the top of the subtree, per the repo's
/// Riverpod rules. No build method anywhere else opens a dialog, and the UI
/// never awaits a notifier action's return.
///
/// It also owns the launch check, once per app start, after the first frame:
/// running it during `initState` would put a network call on the critical path
/// of the first paint.
class UpdateListener extends ConsumerStatefulWidget {
  const UpdateListener({required this.child, super.key});

  final Widget child;

  @override
  ConsumerState<UpdateListener> createState() => _UpdateListenerState();
}

class _UpdateListenerState extends ConsumerState<UpdateListener> {
  bool _promptOpen = false;

  /// Latched once a forced update is seen, and never cleared.
  ///
  /// The notifier moves on to `downloading`/`installing` as the user acts, so
  /// without a latch the blocking screen would disappear the moment the download
  /// started — handing the app back to someone whose client cannot talk to the
  /// backend.
  UpdateRequired? _forced;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      unawaited(ref.read(updateProvider.notifier).checkOnLaunch());
    });
  }

  @override
  Widget build(BuildContext context) {
    ref.listen<UpdateState>(updateProvider, (previous, next) {
      if (next is UpdateRequired) {
        // setState so the blocking screen replaces the subtree on this frame.
        setState(() => _forced = next);
        return;
      }
      _syncPrompt(next);
    });

    // A prompt deferred during a play or practice session is presented when the
    // session ends — the offer is still in the notifier's state, it was simply
    // not shown.
    ref.listen<bool>(updateSessionActiveProvider, (previous, active) {
      if (!active) _syncPrompt(ref.read(updateProvider));
    });

    final forced = _forced;
    if (forced != null) return UpdateRequiredScreen(state: forced);
    return widget.child;
  }

  void _syncPrompt(UpdateState state) {
    final wantsPrompt = switch (state) {
      UpdateAvailableState() ||
      UpdateDownloading() ||
      UpdateReady() ||
      UpdateInstalling() ||
      UpdateFailed() => true,
      _ => false,
    };
    // Never interrupt someone mid-piece. The offer waits; it does not vanish.
    if (wantsPrompt && ref.read(updateSessionActiveProvider)) return;
    if (wantsPrompt && !_promptOpen) {
      _promptOpen = true;
      showDialog<void>(
        context: context,
        // The sequence continues inside the dialog (download → install), so a
        // stray tap outside must not abandon it half-way.
        barrierDismissible: false,
        builder: (_) => const UpdatePromptDialog(),
      ).whenComplete(() => _promptOpen = false);
      return;
    }
    if (!wantsPrompt && _promptOpen) {
      _promptOpen = false;
      Navigator.of(context, rootNavigator: true).pop();
    }
  }
}
