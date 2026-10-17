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

import '../theme/motion.dart';

/// A number that counts from its previous value to the new one when it grows
/// (change: add-music-motion-and-celebrations). A decrease shows at once: a
/// debit counted down would read as a show, and honesty wins.
///
/// [format] turns the interpolated integer into text (the localized "+12" or
/// "Lv 4 · 252 pts"); the default is the bare number.
class AnimatedCounter extends StatefulWidget {
  const AnimatedCounter({
    super.key,
    required this.value,
    this.duration = Motion.celebrate,
    this.format,
    this.style,
    this.textAlign,

    /// Count up from here on first build (a "+N" cue counts from zero).
    this.initialFrom,
  });

  final int value;
  final Duration duration;
  final String Function(int value)? format;
  final TextStyle? style;
  final TextAlign? textAlign;
  final int? initialFrom;

  @override
  State<AnimatedCounter> createState() => _AnimatedCounterState();
}

class _AnimatedCounterState extends State<AnimatedCounter> {
  late int _from = widget.initialFrom ?? widget.value;

  @override
  void didUpdateWidget(AnimatedCounter old) {
    super.didUpdateWidget(old);
    if (old.value != widget.value) _from = old.value;
  }

  @override
  Widget build(BuildContext context) {
    final increase = widget.value > _from;
    return TweenAnimationBuilder<double>(
      key: ValueKey(widget.value),
      tween: Tween(
        begin: increase ? _from.toDouble() : widget.value.toDouble(),
        end: widget.value.toDouble(),
      ),
      duration: increase ? Motion.of(context, widget.duration) : Duration.zero,
      curve: Motion.enter,
      builder: (context, v, _) {
        final shown = v.round();
        return Text(
          widget.format?.call(shown) ?? '$shown',
          style: widget.style,
          textAlign: widget.textAlign,
        );
      },
    );
  }
}
