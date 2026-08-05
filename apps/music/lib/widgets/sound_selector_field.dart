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

import '../services/soundfont_catalog_service.dart'
    show serverSoundFontsProvider;
import '../state/piano_catalog.dart';
import '../theme/cymbra_theme.dart';

/// A **combobox** to choose the instrument sound (SoundFont) the synth plays with
/// — the same control everywhere a score is played (the pre-play setup popup and
/// the rating deck). A pure selector: importing and managing SoundFonts lives on
/// the dedicated management screen (change: add-soundfont-moderation), reached
/// from the home top bar — not here.
///
/// Controlled: the parent owns [value] (a piano id) and reacts to [onChanged], so
/// it can be a draft (applied on Validate in the popup) or immediate (live in the
/// deck). [instrument] is the score's instrument family — the catalog is filtered
/// to matching sounds (every sound is a piano today, so it's a no-op until scores
/// carry an instrument). [dense] drops the caption for a compact form that fits an
/// app bar.
class SoundSelectorField extends ConsumerStatefulWidget {
  const SoundSelectorField({
    super.key,
    required this.value,
    required this.onChanged,
    this.instrument = 'piano',
    this.dense = false,
  });

  /// The selected sound's catalog id.
  final String value;

  /// Called with the newly chosen sound id.
  final ValueChanged<String> onChanged;

  /// The score's instrument family; the catalog is filtered to it.
  final String instrument;

  /// Compact form (dropdown only) for tight spots like an app bar.
  final bool dense;

  @override
  ConsumerState<SoundSelectorField> createState() => _SoundSelectorFieldState();
}

class _SoundSelectorFieldState extends ConsumerState<SoundSelectorField> {
  @override
  void initState() {
    super.initState();
    // Refresh the server-sourced downloadable catalog each time the picker is
    // shown, so a font accepted/added on the server appears without an app
    // restart (the provider otherwise fetches once and caches).
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) ref.invalidate(serverSoundFontsProvider);
    });
  }

  /// Whether [p] belongs to [instrument]. `PianoEntry` has no instrument field
  /// yet and every catalog sound is a piano, so this is a no-op today; filter on
  /// the entry's instrument here once scores/sounds are typed.
  bool _matches(PianoEntry p, String instrument) => true;

  @override
  Widget build(BuildContext context) {
    final catalog = ref.watch(pianoCatalogProvider);
    final sounds = catalog
        .where((p) => _matches(p, widget.instrument))
        .toList();

    // The dropdown value must match an item; fall back to the default if the
    // current selection isn't in the (filtered) catalog.
    final selectedId = sounds.any((p) => p.id == widget.value)
        ? widget.value
        : (sounds.isNotEmpty ? sounds.first.id : defaultPianoId);
    final selected = sounds.firstWhere(
      (p) => p.id == selectedId,
      orElse: () => defaultPiano,
    );
    final subtitle = _licenseLabel(selected);

    final dropdown = Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(10),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<String>(
          isExpanded: true,
          isDense: widget.dense,
          value: selectedId,
          dropdownColor: CymbraColors.surfaceContainerHigh,
          iconEnabledColor: CymbraColors.onSurfaceVariant,
          style: const TextStyle(color: CymbraColors.onSurface),
          items: [
            for (final p in sounds)
              DropdownMenuItem<String>(
                value: p.id,
                child: Text(p.label, overflow: TextOverflow.ellipsis),
              ),
          ],
          onChanged: (v) {
            if (v != null) widget.onChanged(v);
          },
        ),
      ),
    );

    if (widget.dense) return dropdown;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        dropdown,
        if (subtitle != null)
          Padding(
            padding: const EdgeInsets.only(top: 4, left: 4),
            child: Text(
              subtitle,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 12,
              ),
            ),
          ),
      ],
    );
  }

  /// The CC-BY grands' required visible license/attribution line; null for a user
  /// import or a CC0 default (nothing to surface).
  String? _licenseLabel(PianoEntry piano) {
    final license = piano.license;
    if (license == null) return null;
    final attribution = piano.attribution;
    return attribution == null ? license : '$license · $attribution';
  }
}
