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

import 'dart:ui' show Offset, Size;

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../painters/staff_hit_index.dart';

part 'notation_help_notifier.freezed.dart';
part 'notation_help_notifier.g.dart';

/// The help bubble currently shown for a long-pressed staff symbol (change:
/// add-notation-help): the resolved [descriptor], the [anchor] the user pressed
/// (in the staff area's local coordinates) and the [areaSize] so the bubble can
/// clamp itself on screen. `null` when no bubble is open.
@freezed
sealed class NotationHelpBubbleState with _$NotationHelpBubbleState {
  const factory NotationHelpBubbleState({
    required SymbolDescriptor descriptor,
    required Offset anchor,
    required Size areaSize,
  }) = _NotationHelpBubbleState;
}

/// Holds the open notation-help bubble, if any. A single bubble is shown at a
/// time (only one staff area is interactive at once — the player's staff or the
/// engraved partition), so one controller suffices. The overlay widget reacts to
/// this state; it never resolves symbols itself.
@riverpod
class NotationHelpBubbleController extends _$NotationHelpBubbleController {
  @override
  NotationHelpBubbleState? build() => null;

  /// Opens the bubble for [descriptor], anchored at [anchor] within an area of
  /// [areaSize].
  void show(SymbolDescriptor descriptor, Offset anchor, Size areaSize) {
    state = NotationHelpBubbleState(
      descriptor: descriptor,
      anchor: anchor,
      areaSize: areaSize,
    );
  }

  /// Closes the bubble (no-op when none is open).
  void dismiss() {
    if (state != null) state = null;
  }
}
