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

import 'package:flutter/widgets.dart';

/// The app's motion tokens (change: add-music-motion-and-celebrations): three
/// durations, two curves, one rule. Widgets take their timings from here and
/// never read the platform's reduced-motion flag themselves — [of] applies it,
/// so a platform that asks for reduced motion gets every animation's final
/// frame at once.
class Motion {
  Motion._();

  /// A state change the eye should barely notice: a check appearing, a chip
  /// toggling.
  static const Duration quick = Duration(milliseconds: 160);

  /// A transition between two surfaces or states: a door switch, a path
  /// segment filling.
  static const Duration standard = Duration(milliseconds: 280);

  /// A celebration sequence: icon in, burst, title, counter.
  static const Duration celebrate = Duration(milliseconds: 900);

  /// Entering and settling.
  static const Curve enter = Curves.easeOutCubic;

  /// Popping in with a little overshoot — reward moments only.
  static const Curve pop = Curves.easeOutBack;

  /// Whether the platform asked for reduced motion.
  static bool reduced(BuildContext context) =>
      MediaQuery.disableAnimationsOf(context);

  /// [duration], or zero under reduced motion.
  static Duration of(BuildContext context, Duration duration) =>
      reduced(context) ? Duration.zero : duration;
}
