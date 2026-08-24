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

import '../state/score_font.dart';

/// Dedicated listener widget for the font-follows-score reaction (change:
/// add-drum-audio-channel; architecture rule 4 — `ref.listen` side effects
/// live in one place near the top of the player subtree). It renders [child]
/// and only reports the loaded score's family to the [ScoreFont] controller,
/// which owns every font swap: opening a percussion score installs the
/// remembered kit, and leaving the player (or loading a keyboard score)
/// restores the remembered piano. The widget never touches a service.
class ScoreFontListener extends ConsumerStatefulWidget {
  const ScoreFontListener({
    required this.percussion,
    required this.child,
    super.key,
  });

  /// Whether the score this subtree is sounding is percussion. Passed in
  /// rather than read from the player: the upload and rating previews sound
  /// their own locally-parsed score, and a listener that could only read the
  /// player left them playing drum numbers through the piano font.
  final bool percussion;

  final Widget child;

  @override
  ConsumerState<ScoreFontListener> createState() => _ScoreFontListenerState();
}

class _ScoreFontListenerState extends ConsumerState<ScoreFontListener> {
  /// Captured in [initState] so [dispose] never touches `ref` after teardown.
  late final ScoreFont _scoreFont;

  @override
  void initState() {
    super.initState();
    _scoreFont = ref.read(scoreFontProvider.notifier);
    // The score is usually pre-loaded before this subtree mounts (the hub
    // guard), so the listener below would never fire for it: report the
    // already-loaded family once, after the first frame.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      unawaited(_scoreFont.setScoreFamily(percussion: widget.percussion));
    });
  }

  @override
  void dispose() {
    // Leaving the player is returning to a keyboard surface (home, free play):
    // restore the remembered piano. Deferred — a provider must not be written
    // while the tree is being finalized; the controller itself swallows a
    // container that is fully tearing down.
    final scoreFont = _scoreFont;
    scheduleMicrotask(() {
      unawaited(scoreFont.setScoreFamily(percussion: false));
    });
    super.dispose();
  }

  @override
  void didUpdateWidget(ScoreFontListener old) {
    super.didUpdateWidget(old);
    // The family changes when a different score loads under the same subtree
    // (the player keeps its route across score changes).
    if (old.percussion != widget.percussion) {
      // Deferred to after the frame, like the initial report: the swap awaits
      // provider futures, and starting that chain inside the build phase
      // leaves it hanging when those providers are themselves initialising.
      final wanted = widget.percussion;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        unawaited(_scoreFont.setScoreFamily(percussion: wanted));
      });
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
