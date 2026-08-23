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

import '../l10n/gen/app_localizations.dart';
import '../state/player_data.dart';
import '../theme/cymbra_theme.dart';

/// A radio-style value row with a leading "selected" check. Shared by the player
/// settings drawer and the pre-play setup modal so the two never diverge.
class SettingOptionRow extends StatelessWidget {
  const SettingOptionRow({
    required this.selected,
    required this.label,
    required this.onTap,
    super.key,
  });

  final bool selected;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => ListTile(
    leading: Icon(
      selected ? Icons.check_circle : Icons.radio_button_unchecked,
      size: 20,
      color: selected ? CymbraColors.tertiary : CymbraColors.onSurfaceVariant,
    ),
    title: Text(label, style: const TextStyle(color: CymbraColors.onSurface)),
    onTap: onTap,
  );
}

/// The localized label for a [Hand] selection.
String handLabel(
  AppLocalizations l10n,
  Hand hand, {
  bool percussion = false,
}) => switch (hand) {
  // For a percussion score the same three-valued state reads hands / feet /
  // both (change: add-drum-kit-view): the split a drummer practises is hands
  // against feet, keyed to the voice convention — Hand.right selects the
  // hands and Hand.left the feet.
  Hand.left => percussion ? l10n.handFeet : l10n.handLeft,
  Hand.right => percussion ? l10n.handHands : l10n.handRight,
  Hand.both => l10n.handBoth,
};
