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

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../state/screen_wake.dart';

/// Holds the device's screen awake for as long as [child] is mounted (change:
/// keep-play-surfaces-awake).
///
/// Wrapped around the body of each **play surface** — the player, the lesson
/// player, the drum-input calibration and the MIDI monitor — i.e. the screens
/// someone reads while their hands are on an instrument, producing no touch
/// events for the OS to count as activity.
///
/// A wrapper rather than an `initState`/`dispose` pair in each screen because
/// two of the four are `ConsumerWidget`s with no `State` to hang one off, and
/// converting them just to own a lifecycle hook is the worse change. It also
/// keeps the layering rule: this talks to the notifier, never to the service.
class KeepScreenAwake extends ConsumerStatefulWidget {
  const KeepScreenAwake({required this.child, super.key});

  final Widget child;

  @override
  ConsumerState<KeepScreenAwake> createState() => _KeepScreenAwakeState();
}

class _KeepScreenAwakeState extends ConsumerState<KeepScreenAwake> {
  /// Captured on mount, not read in [dispose]: reading a provider while the
  /// element tree is being torn down is how you get a "tried to use a disposed
  /// provider" crash on app exit.
  late final ScreenWake _screenWake;

  @override
  void initState() {
    super.initState();
    _screenWake = ref.read(screenWakeProvider.notifier);
    _defer(_screenWake.acquire);
  }

  @override
  void dispose() {
    _defer(_screenWake.release);
    super.dispose();
  }

  /// Riverpod forbids modifying a provider from a widget life-cycle
  /// (`initState`, `dispose`, `build`), because two widgets listening to the
  /// same provider could otherwise observe different states within one frame.
  /// A microtask runs once the frame's synchronous work is done — still
  /// promptly, and in FIFO order, so a route replacing another applies its
  /// release and acquire in the order they happened.
  void _defer(void Function() change) => scheduleMicrotask(change);

  @override
  Widget build(BuildContext context) => widget.child;
}
