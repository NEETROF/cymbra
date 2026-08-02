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

import '../l10n/gen/app_localizations.dart';
import '../services/soundfont_importer.dart' show SoundFontImportException;
import '../state/imported_soundfonts.dart';
import '../state/piano_catalog.dart';
import '../theme/cymbra_theme.dart';
import 'app_snackbar.dart';

/// A **combobox** to choose the instrument sound (SoundFont) the synth plays with
/// — the same control everywhere a score is played (the pre-play setup popup and
/// the rating deck). Mirrors the back-office soundfont picker: a dropdown of the
/// catalog, plus an "add a SoundFont" entry that runs the import flow.
///
/// Controlled: the parent owns [value] (a piano id) and reacts to [onChanged], so
/// it can be a draft (applied on Validate in the popup) or immediate (live in the
/// deck). [instrument] is the score's instrument family — the catalog is filtered
/// to matching sounds (every sound is a piano today, so it's a no-op until scores
/// carry an instrument). [dense] drops the caption + manage affordance for a
/// compact form that fits an app bar.
class SoundSelectorField extends ConsumerWidget {
  const SoundSelectorField({
    super.key,
    required this.value,
    required this.onChanged,
    this.instrument = 'piano',
    this.dense = false,
  });

  /// The selected sound's catalog id.
  final String value;

  /// Called with the newly chosen sound id (after a pick or a successful import).
  final ValueChanged<String> onChanged;

  /// The score's instrument family; the catalog is filtered to it.
  final String instrument;

  /// Compact form (dropdown only) for tight spots like an app bar.
  final bool dense;

  /// Sentinel dropdown value that triggers the SoundFont import flow instead of
  /// selecting an existing sound.
  static const String _addValue = '__add_soundfont__';

  /// Whether [p] belongs to [instrument]. `PianoEntry` has no instrument field
  /// yet and every catalog sound is a piano, so this is a no-op today; filter on
  /// the entry's instrument here once scores/sounds are typed.
  bool _matches(PianoEntry p, String instrument) => true;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final catalog = ref.watch(pianoCatalogProvider);
    final sounds = catalog.where((p) => _matches(p, instrument)).toList();

    // The dropdown value must match an item; fall back to the default if the
    // current selection isn't in the (filtered) catalog.
    final selectedId = sounds.any((p) => p.id == value)
        ? value
        : (sounds.isNotEmpty ? sounds.first.id : defaultPianoId);
    final selected = sounds.firstWhere(
      (p) => p.id == selectedId,
      orElse: () => defaultPiano,
    );
    final hasImports = sounds.any((p) => p.kind == PianoKind.user);
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
          isDense: dense,
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
            DropdownMenuItem<String>(
              value: _addValue,
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(
                    Icons.add,
                    size: 18,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                  const SizedBox(width: 8),
                  Text(l10n.pianoAddSoundFont),
                ],
              ),
            ),
          ],
          onChanged: (v) => _onPicked(context, ref, v),
        ),
      ),
    );

    if (dense) return dropdown;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(child: dropdown),
            if (hasImports)
              Padding(
                padding: const EdgeInsets.only(left: 4),
                child: IconButton(
                  tooltip: l10n.pianoManage,
                  icon: const Icon(
                    Icons.tune,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                  onPressed: () => _openManager(context, ref),
                ),
              ),
          ],
        ),
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

  /// Handles a dropdown pick: the add sentinel runs the import (and selects the
  /// imported font on success); anything else selects that sound.
  Future<void> _onPicked(
    BuildContext context,
    WidgetRef ref,
    String? picked,
  ) async {
    if (picked == null) return;
    if (picked != _addValue) {
      onChanged(picked);
      return;
    }
    final messenger = ScaffoldMessenger.of(context);
    final l10n = AppLocalizations.of(context);
    try {
      final entry = await ref
          .read(importedSoundFontsProvider.notifier)
          .importSoundFont();
      if (entry != null) onChanged(entry.id);
    } on SoundFontImportException {
      showAppSnackBar(messenger, l10n.pianoImportInvalid);
    } catch (_) {
      // Cancelled or a transient picker error — nothing to surface.
    }
  }

  /// Opens the manage sheet to remove imported sounds.
  void _openManager(BuildContext context, WidgetRef ref) {
    showModalBottomSheet<void>(
      context: context,
      backgroundColor: CymbraColors.surfaceContainerHigh,
      builder: (_) => _SoundManagerSheet(
        onRemovedSelected: () {
          // If the removed sound was selected, fall back to the default.
          onChanged(defaultPianoId);
        },
        selectedId: value,
      ),
    );
  }
}

/// Bottom sheet listing the imported sounds with a delete affordance (the remove
/// that used to live in the in-game settings drawer). Selection happens in the
/// combobox; this is management only.
class _SoundManagerSheet extends ConsumerWidget {
  const _SoundManagerSheet({
    required this.onRemovedSelected,
    required this.selectedId,
  });

  /// Called when the currently-selected sound is the one removed.
  final VoidCallback onRemovedSelected;
  final String selectedId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final imported = ref
        .watch(pianoCatalogProvider)
        .where((p) => p.kind == PianoKind.user)
        .toList();
    return SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
            child: Text(
              l10n.pianoManage,
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 16,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
          if (imported.isEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 4, 20, 20),
              child: Text(
                l10n.pianoGroupImported,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            )
          else
            for (final p in imported)
              ListTile(
                title: Text(
                  p.label,
                  style: const TextStyle(color: CymbraColors.onSurface),
                ),
                trailing: IconButton(
                  tooltip: l10n.pianoRemove,
                  icon: const Icon(
                    Icons.delete_outline,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                  onPressed: () {
                    ref.read(importedSoundFontsProvider.notifier).remove(p.id);
                    if (p.id == selectedId) onRemovedSelected();
                  },
                ),
              ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}
