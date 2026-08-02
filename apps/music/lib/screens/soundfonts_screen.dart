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
import '../services/private_soundfont_service.dart'
    show PrivateSoundFontException;
import '../services/soundfont_importer.dart'
    show PickedSoundFont, SoundFontImportException, soundFontImporterProvider;
import '../state/imported_soundfonts.dart';
import '../state/piano_catalog.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';

/// A dedicated screen to manage the user's own SoundFonts (the private,
/// server-synced library, change: add-soundfont-moderation). Reached from the
/// home top bar. Lists the imports with remove + propose-to-catalog, and an add
/// affordance that opens a right end-drawer to pick a `.sf2` and name it — an
/// edit opens the same drawer to rename.
class SoundFontsScreen extends ConsumerStatefulWidget {
  const SoundFontsScreen({super.key});

  @override
  ConsumerState<SoundFontsScreen> createState() => _SoundFontsScreenState();
}

class _SoundFontsScreenState extends ConsumerState<SoundFontsScreen> {
  final _scaffoldKey = GlobalKey<ScaffoldState>();

  /// The font being renamed; `null` puts the drawer in add mode.
  PianoEntry? _editing;

  /// Bumped on every open so the drawer form resets its fields.
  int _openSeq = 0;

  void _openAdd() {
    setState(() {
      _editing = null;
      _openSeq++;
    });
    _scaffoldKey.currentState?.openEndDrawer();
  }

  void _openEdit(PianoEntry entry) {
    setState(() {
      _editing = entry;
      _openSeq++;
    });
    _scaffoldKey.currentState?.openEndDrawer();
  }

  Future<void> _remove(PianoEntry entry) async {
    final l10n = AppLocalizations.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        content: Text(l10n.soundfontsRemoveConfirm(entry.label)),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(l10n.proposeCancel),
          ),
          FilledButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(l10n.pianoRemove),
          ),
        ],
      ),
    );
    if (confirmed ?? false) {
      await ref.read(importedSoundFontsProvider.notifier).remove(entry.id);
    }
  }

  Future<void> _propose(PianoEntry entry) async {
    final messenger = ScaffoldMessenger.of(context);
    final l10n = AppLocalizations.of(context);
    final result = await showDialog<({String license, String attribution})>(
      context: context,
      builder: (_) => _ProposeDialog(),
    );
    if (result == null) return;
    try {
      await ref
          .read(importedSoundFontsProvider.notifier)
          .proposeToPublicCatalog(
            entry.id,
            license: result.license,
            attribution: result.attribution,
            attestation: true,
          );
      showAppSnackBar(messenger, l10n.proposeDone);
    } on PrivateSoundFontException {
      showAppSnackBar(messenger, l10n.proposeError);
    } catch (_) {
      showAppSnackBar(messenger, l10n.proposeError);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final registry = ref.watch(importedSoundFontsProvider);
    return Scaffold(
      key: _scaffoldKey,
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.soundfontsTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: l10n.soundfontsAdd,
            onPressed: _openAdd,
          ),
          const SizedBox(width: 8),
        ],
      ),
      endDrawer: _SoundFontFormDrawer(
        // Key by the target + open sequence so the form resets each open.
        key: ValueKey('${_editing?.id ?? "add"}-$_openSeq'),
        editing: _editing,
        onDone: () => _scaffoldKey.currentState?.closeEndDrawer(),
      ),
      body: SafeArea(
        child: registry.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (_, _) => Center(
            child: Text(
              l10n.soundfontsError,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ),
          data: (fonts) => fonts.isEmpty
              ? Center(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Text(
                      l10n.soundfontsEmpty,
                      textAlign: TextAlign.center,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                      ),
                    ),
                  ),
                )
              : ListView.separated(
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  itemCount: fonts.length,
                  separatorBuilder: (_, _) => const Divider(height: 1),
                  itemBuilder: (context, i) {
                    final p = fonts[i];
                    final subtitle = p.license == null
                        ? null
                        : (p.attribution == null
                              ? p.license!
                              : '${p.license} · ${p.attribution}');
                    return ListTile(
                      title: Text(
                        p.label,
                        style: const TextStyle(color: CymbraColors.onSurface),
                      ),
                      subtitle: subtitle == null
                          ? null
                          : Text(
                              subtitle,
                              style: const TextStyle(
                                color: CymbraColors.onSurfaceVariant,
                                fontSize: 12,
                              ),
                            ),
                      trailing: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          if (p.remoteId != null)
                            IconButton(
                              tooltip: l10n.pianoPropose,
                              icon: const Icon(
                                Icons.publish_outlined,
                                color: CymbraColors.onSurfaceVariant,
                              ),
                              onPressed: () => _propose(p),
                            ),
                          IconButton(
                            tooltip: l10n.soundfontsRename,
                            icon: const Icon(
                              Icons.edit_outlined,
                              color: CymbraColors.onSurfaceVariant,
                            ),
                            onPressed: () => _openEdit(p),
                          ),
                          IconButton(
                            tooltip: l10n.pianoRemove,
                            icon: const Icon(
                              Icons.delete_outline,
                              color: CymbraColors.onSurfaceVariant,
                            ),
                            onPressed: () => _remove(p),
                          ),
                        ],
                      ),
                    );
                  },
                ),
        ),
      ),
    );
  }
}

/// The add/rename end-drawer. In add mode it picks a `.sf2` and names it; in edit
/// mode ([editing] set) it renames.
class _SoundFontFormDrawer extends ConsumerStatefulWidget {
  const _SoundFontFormDrawer({super.key, required this.editing, required this.onDone});

  final PianoEntry? editing;
  final VoidCallback onDone;

  @override
  ConsumerState<_SoundFontFormDrawer> createState() => _SoundFontFormDrawerState();
}

class _SoundFontFormDrawerState extends ConsumerState<_SoundFontFormDrawer> {
  late final TextEditingController _name = TextEditingController(
    text: widget.editing?.label ?? '',
  );
  PickedSoundFont? _picked;
  bool _busy = false;

  bool get _isEdit => widget.editing != null;

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  Future<void> _choose() async {
    final messenger = ScaffoldMessenger.of(context);
    final l10n = AppLocalizations.of(context);
    try {
      final picked = await ref.read(soundFontImporterProvider).pick();
      if (picked == null) return; // cancelled
      setState(() {
        _picked = picked;
        if (_name.text.trim().isEmpty) _name.text = picked.suggestedLabel;
      });
    } on SoundFontImportException {
      showAppSnackBar(messenger, l10n.pianoImportInvalid);
    } catch (_) {
      // Transient picker error — nothing to surface.
    }
  }

  Future<void> _submit() async {
    final name = _name.text.trim();
    if (name.isEmpty) return;
    setState(() => _busy = true);
    final notifier = ref.read(importedSoundFontsProvider.notifier);
    try {
      if (_isEdit) {
        await notifier.rename(widget.editing!.id, name);
      } else {
        await notifier.addImport(_picked!.bytes, name);
      }
      widget.onDone();
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final canSubmit =
        !_busy && _name.text.trim().isNotEmpty && (_isEdit || _picked != null);
    return Drawer(
      backgroundColor: CymbraColors.surfaceContainerHigh,
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                _isEdit ? l10n.soundfontsEditTitle : l10n.soundfontsAddTitle,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 18,
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 16),
              if (!_isEdit) ...[
                OutlinedButton.icon(
                  onPressed: _busy ? null : _choose,
                  icon: const Icon(Icons.folder_open),
                  label: Text(
                    _picked == null
                        ? l10n.soundfontsChooseFile
                        : _picked!.suggestedLabel,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const SizedBox(height: 12),
              ],
              TextField(
                controller: _name,
                onChanged: (_) => setState(() {}),
                style: const TextStyle(color: CymbraColors.onSurface),
                decoration: InputDecoration(labelText: l10n.soundfontsName),
              ),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: _busy ? null : widget.onDone,
                    child: Text(l10n.proposeCancel),
                  ),
                  const SizedBox(width: 8),
                  FilledButton(
                    onPressed: canSubmit ? _submit : null,
                    child: Text(
                      _isEdit ? l10n.soundfontsRename : l10n.soundfontsAdd,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Propose dialog (licence + attribution + right-to-distribute attestation).
/// Returns `(license, attribution)` on submit; submit stays disabled until a
/// licence is entered and the attestation is checked.
class _ProposeDialog extends StatefulWidget {
  @override
  State<_ProposeDialog> createState() => _ProposeDialogState();
}

class _ProposeDialogState extends State<_ProposeDialog> {
  final _license = TextEditingController();
  final _attribution = TextEditingController();
  bool _attested = false;

  @override
  void dispose() {
    _license.dispose();
    _attribution.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final canSubmit = _attested && _license.text.trim().isNotEmpty;
    return AlertDialog(
      title: Text(l10n.proposeTitle),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(l10n.proposeIntro),
            const SizedBox(height: 12),
            TextField(
              controller: _license,
              decoration: InputDecoration(labelText: l10n.proposeLicense),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: 8),
            TextField(
              controller: _attribution,
              decoration: InputDecoration(labelText: l10n.proposeAttribution),
            ),
            const SizedBox(height: 8),
            CheckboxListTile(
              contentPadding: EdgeInsets.zero,
              value: _attested,
              onChanged: (v) => setState(() => _attested = v ?? false),
              title: Text(l10n.proposeAttest),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.proposeCancel),
        ),
        FilledButton(
          onPressed: canSubmit
              ? () => Navigator.of(context).pop((
                  license: _license.text.trim(),
                  attribution: _attribution.text.trim(),
                ))
              : null,
          child: Text(l10n.proposeSubmit),
        ),
      ],
    );
  }
}
