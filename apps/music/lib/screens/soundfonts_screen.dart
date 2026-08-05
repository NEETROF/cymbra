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
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../services/audio_service.dart';
import '../services/curator_rewards_service.dart' show RewardShopItemView;
import '../services/private_soundfont_service.dart'
    show PrivateSoundFontException;
import '../services/soundfont_catalog_service.dart'
    show serverSoundFontsProvider;
import '../services/soundfont_importer.dart'
    show PickedSoundFont, SoundFontImportException, soundFontImporterProvider;
import '../services/soundfont_source.dart' show soundFontSourceProvider;
import '../state/imported_soundfonts.dart';
import '../state/piano_catalog.dart';
import '../state/player_data.dart' show scoreNoteEdges;
import '../state/reward_shop_notifier.dart';
import '../state/selected_piano.dart';
import '../state/sound_preview_sample.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/reward_celebration.dart';

/// The instrument-sound **hub** (change: add-soundfont-moderation), reached from
/// the home top bar. Modelled on the score hub: a search field over the sounds,
/// with "My instrument sounds" (the user's private, server-synced imports) first,
/// then the "Catalog" (built-in + downloadable). Tapping a sound auditions it by
/// playing a short bundled sample with that SoundFont loaded. Adding/renaming a
/// user sound opens a right end-drawer (the back-office-style flow).
class SoundFontsScreen extends ConsumerStatefulWidget {
  const SoundFontsScreen({super.key});

  @override
  ConsumerState<SoundFontsScreen> createState() => _SoundFontsScreenState();
}

class _SoundFontsScreenState extends ConsumerState<SoundFontsScreen>
    with SingleTickerProviderStateMixin {
  final _scaffoldKey = GlobalKey<ScaffoldState>();

  /// The font being renamed; `null` puts the add/edit drawer in add mode.
  PianoEntry? _editing;

  /// Bumped on every drawer open so the form resets its fields.
  int _openSeq = 0;

  /// Lowercased search query over sound labels.
  String _query = '';

  // --- Audition (sample playback) ------------------------------------------
  late final AudioService _audio;
  late final Ticker _ticker;

  /// The selected piano's font path, captured on entry so we can restore the
  /// synth when leaving (auditioning swaps the active font globally).
  String? _restorePath;

  /// The id of the sound currently being auditioned, or `null`.
  String? _previewingId;

  bool _seeded = false;
  Duration _lastTick = Duration.zero;
  double _elapsedMs = 0;
  final Set<int> _sounding = <int>{};

  @override
  void initState() {
    super.initState();
    _audio = ref.read(audioServiceProvider);
    unawaited(_audio.init());
    _ticker = createTicker(_onTick);
    // Pre-warm the (kept-alive) audition sample so the first tap plays without a
    // parse delay.
    ref.read(soundPreviewSampleProvider);
    unawaited(_captureRestore());
    // Refresh server-sourced state each time the hub is opened: the downloadable
    // catalog, and the private library (so a proposal accepted/rejected elsewhere
    // updates its status tag without a relaunch).
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      ref.invalidate(serverSoundFontsProvider);
      ref.read(importedSoundFontsProvider.notifier).refresh();
    });
  }

  @override
  void dispose() {
    _ticker.dispose();
    _audio.allNotesOff();
    // Best-effort restore of the selected sound's font (captured reference — no
    // provider read during dispose).
    final path = _restorePath;
    if (path != null) unawaited(_audio.loadSoundFont(path));
    super.dispose();
  }

  Future<void> _captureRestore() async {
    try {
      final id = ref.read(selectedPianoProvider);
      final entry = ref
          .read(pianoCatalogProvider)
          .firstWhere((e) => e.id == id, orElse: () => defaultPiano);
      _restorePath = await ref.read(soundFontSourceProvider).resolve(entry);
    } catch (_) {
      // Non-fatal: without a restore path we just leave the last-loaded font.
    }
  }

  // --- Drawer (add / rename) -----------------------------------------------

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
      if (_previewingId == entry.id) await _stopPreview();
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
    } on PrivateSoundFontException catch (e) {
      // 409 = already in the catalog (already proposed, or identical content).
      showAppSnackBar(
        messenger,
        e.statusCode == 409 ? l10n.proposeAlreadyDone : l10n.proposeError,
      );
    } catch (_) {
      showAppSnackBar(messenger, l10n.proposeError);
    }
  }

  // --- Reward unlock -------------------------------------------------------

  /// Redeem the reward keyed by a locked catalog font's id (change: add-curation-
  /// rewards). Fire-and-observe: the shop notifier persists then refreshes, and a
  /// listener (wired in [build]) shows the celebration / error.
  void _redeemReward(String key) =>
      ref.read(rewardShopProvider.notifier).redeem(key);

  // --- Audition ------------------------------------------------------------

  Future<void> _togglePreview(PianoEntry entry) async {
    if (_previewingId == entry.id) {
      await _stopPreview();
      return;
    }
    final messenger = ScaffoldMessenger.of(context);
    final l10n = AppLocalizations.of(context);
    try {
      final path = await ref.read(soundFontSourceProvider).resolve(entry);
      await _audio.loadSoundFont(path);
    } catch (_) {
      showAppSnackBar(messenger, l10n.soundfontsPreviewError);
      return;
    }
    if (!mounted) return;
    // Kick the sample load (parsed lazily; the ticker waits for it).
    ref.read(soundPreviewSampleProvider);
    _ticker.stop();
    _audio.allNotesOff();
    _sounding.clear();
    _seeded = false;
    _lastTick = Duration.zero;
    _elapsedMs = 0;
    setState(() => _previewingId = entry.id);
    _ticker.start();
  }

  Future<void> _stopPreview() async {
    _ticker.stop();
    _audio.allNotesOff();
    _sounding.clear();
    if (mounted) setState(() => _previewingId = null);
    final path = _restorePath;
    if (path != null) {
      try {
        await _audio.loadSoundFont(path);
      } catch (_) {}
    }
  }

  void _onTick(Duration elapsed) {
    final sample = ref.read(soundPreviewSampleProvider).valueOrNull;
    if (sample == null || sample.isEmpty) {
      _lastTick = elapsed;
      return;
    }
    if (!_seeded) {
      _elapsedMs = sample.startMs;
      _seeded = true;
      _lastTick = elapsed;
      return;
    }
    final dtMs = (elapsed - _lastTick).inMicroseconds / 1000.0;
    _lastTick = elapsed;
    if (dtMs <= 0 || dtMs > 100) return; // skip a stalled/huge frame

    var next = _elapsedMs + dtMs;
    if (sample.songEndMs > 0 && next >= sample.songEndMs) {
      // Loop the audition until the user stops it.
      _audio.allNotesOff();
      _sounding.clear();
      next = sample.startMs;
    } else {
      final edges = scoreNoteEdges(
        visible: sample.notes,
        from: _elapsedMs,
        to: next,
        sounding: _sounding,
      );
      for (final p in edges.stops) {
        _audio.noteOff(p);
        _sounding.remove(p);
      }
      for (final p in edges.starts) {
        _audio.noteOn(p);
        _sounding.add(p);
      }
    }
    _elapsedMs = next;
  }

  // --- Build ---------------------------------------------------------------

  bool _matchesQuery(PianoEntry p) =>
      _query.isEmpty || p.label.toLowerCase().contains(_query);

  /// Build a catalog sound card, gating it behind a reward unlock when [item] is a
  /// costed, redeemable, not-yet-owned reward matching this font (id == key).
  Widget _buildCatalogCard(PianoEntry p, RewardShopItemView? item) {
    final locked =
        item != null && item.pointCost > 0 && item.redeemable && !item.owned;
    return _SoundCard(
      entry: p,
      playing: _previewingId == p.id,
      // A locked reward font is not selectable/auditionable until redeemed.
      onTap: locked ? null : () => _togglePreview(p),
      locked: locked,
      lockCost: locked ? item.pointCost : null,
      onRedeem: locked ? () => _redeemReward(p.id) : null,
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final catalog = ref.watch(pianoCatalogProvider).where(_matchesQuery);
    final mine = catalog.where((p) => p.kind == PianoKind.user).toList();
    final cat = catalog
        .where((p) => p.kind != PianoKind.user)
        .toList(growable: false);
    // Cross-reference catalog fonts with reward-shop items (piano id == item key)
    // so a costed, unowned reward font shows a lock + cost + redeem affordance.
    final shopByKey = ref.watch(rewardShopItemsByKeyProvider);

    // Show a celebration (or error) exactly once per redeem, driven off the shop
    // notifier's transient cue — a side effect kept out of the card widgets.
    ref.listen(rewardShopProvider.select((s) => s.valueOrNull?.redeemSeq), (
      prev,
      next,
    ) {
      if (next == null || next == prev) return;
      final state = ref.read(rewardShopProvider).valueOrNull;
      if (state == null) return;
      if (state.redeemError) {
        showAppSnackBar(
          ScaffoldMessenger.of(context),
          l10n.rewardShopRedeemError,
        );
      } else if (state.lastRedeemedLabel != null) {
        showRewardCelebration(
          context,
          title: l10n.rewardCelebrationRedeemedTitle,
          message: l10n.rewardShopRedeemed(state.lastRedeemedLabel!),
          icon: Icons.music_note,
        );
      }
    });

    return Scaffold(
      key: _scaffoldKey,
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.soundfontsTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
        actions: [
          IconButton(
            icon: const Icon(Icons.library_add_outlined),
            tooltip: l10n.soundfontsAdd,
            onPressed: _openAdd,
          ),
          const SizedBox(width: 8),
        ],
      ),
      endDrawer: _SoundFontFormDrawer(
        key: ValueKey('${_editing?.id ?? "add"}-$_openSeq'),
        editing: _editing,
        onDone: () => _scaffoldKey.currentState?.closeEndDrawer(),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
              child: TextField(
                onChanged: (v) =>
                    setState(() => _query = v.trim().toLowerCase()),
                style: const TextStyle(color: CymbraColors.onSurface),
                decoration: InputDecoration(
                  hintText: l10n.soundfontsSearchHint,
                  prefixIcon: const Icon(
                    Icons.search,
                    color: CymbraColors.outline,
                  ),
                  filled: true,
                  fillColor: CymbraColors.surfaceContainerHigh,
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(12),
                    borderSide: BorderSide.none,
                  ),
                ),
              ),
            ),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.only(bottom: 24),
                children: [
                  _SectionHeader(l10n.soundfontsSectionMine),
                  if (mine.isEmpty)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(20, 4, 20, 12),
                      child: Text(
                        l10n.soundfontsEmpty,
                        style: const TextStyle(
                          color: CymbraColors.onSurfaceVariant,
                        ),
                      ),
                    )
                  else
                    for (final p in mine)
                      _SoundCard(
                        entry: p,
                        playing: _previewingId == p.id,
                        onTap: () => _togglePreview(p),
                        onRename: () => _openEdit(p),
                        onRemove: () => _remove(p),
                        // Propose only when synced AND not already proposed; once
                        // proposed the card shows a status tag instead.
                        onPropose:
                            (p.remoteId != null && p.proposalStatus == null)
                            ? () => _propose(p)
                            : null,
                      ),
                  _SectionHeader(l10n.soundfontsSectionCatalog),
                  for (final p in cat) _buildCatalogCard(p, shopByKey[p.id]),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  const _SectionHeader(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
    child: Text(
      text,
      style: const TextStyle(
        color: CymbraColors.onSurface,
        fontSize: 16,
        fontWeight: FontWeight.w700,
      ),
    ),
  );
}

/// One sound row/card: a play/stop leading control (audition), the label + its
/// licence, and (for a user sound) rename / remove / propose actions. The whole
/// card is tappable to toggle the audition.
class _SoundCard extends StatelessWidget {
  const _SoundCard({
    required this.entry,
    required this.playing,
    required this.onTap,
    this.onRename,
    this.onRemove,
    this.onPropose,
    this.locked = false,
    this.lockCost,
    this.onRedeem,
  });

  final PianoEntry entry;
  final bool playing;

  /// Tap toggles the audition; `null` disables it (a locked reward font).
  final VoidCallback? onTap;
  final VoidCallback? onRename;
  final VoidCallback? onRemove;
  final VoidCallback? onPropose;

  /// This catalog font is a costed reward the caller has not yet unlocked.
  final bool locked;

  /// The reward's point cost (when [locked]).
  final int? lockCost;

  /// Redeem this reward (when [locked]).
  final VoidCallback? onRedeem;

  String? get _subtitle {
    final license = entry.license;
    if (license == null) return null;
    final attribution = entry.attribution;
    return attribution == null ? license : '$license · $attribution';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final subtitle = _subtitle;
    return Card(
      color: CymbraColors.surfaceContainerHigh,
      margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: ListTile(
        onTap: onTap,
        leading: Icon(
          locked
              ? Icons.lock_outline
              : (playing ? Icons.stop_circle : Icons.play_circle_outline),
          color: locked
              ? CymbraColors.outline
              : (playing
                    ? CymbraColors.primary
                    : CymbraColors.onSurfaceVariant),
          semanticLabel: locked
              ? l10n.soundfontsLocked
              : (playing ? l10n.soundfontsStop : l10n.soundfontsPlay),
        ),
        title: Text(
          entry.label,
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
        trailing: locked
            ? _RewardLock(cost: lockCost ?? 0, onRedeem: onRedeem)
            : (onRename == null &&
                  onRemove == null &&
                  onPropose == null &&
                  entry.proposalStatus == null)
            ? null
            : Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (entry.proposalStatus != null)
                    _ProposalTag(status: entry.proposalStatus!),
                  if (onPropose != null)
                    IconButton(
                      tooltip: l10n.pianoPropose,
                      icon: const Icon(
                        Icons.publish_outlined,
                        color: CymbraColors.onSurfaceVariant,
                      ),
                      onPressed: onPropose,
                    ),
                  if (onRename != null)
                    IconButton(
                      tooltip: l10n.soundfontsRename,
                      icon: const Icon(
                        Icons.edit_outlined,
                        color: CymbraColors.onSurfaceVariant,
                      ),
                      onPressed: onRename,
                    ),
                  if (onRemove != null)
                    IconButton(
                      tooltip: l10n.pianoRemove,
                      icon: const Icon(
                        Icons.delete_outline,
                        color: CymbraColors.onSurfaceVariant,
                      ),
                      onPressed: onRemove,
                    ),
                ],
              ),
      ),
    );
  }
}

/// The trailing affordance for a locked reward font: its point cost + an Unlock
/// (redeem) button.
class _RewardLock extends StatelessWidget {
  const _RewardLock({required this.cost, required this.onRedeem});

  final int cost;
  final VoidCallback? onRedeem;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          l10n.rewardShopCost(cost),
          style: const TextStyle(
            color: CymbraColors.primary,
            fontSize: 12.5,
            fontWeight: FontWeight.w700,
          ),
        ),
        const SizedBox(width: 8),
        FilledButton(
          key: const Key('soundfont-unlock'),
          onPressed: onRedeem,
          child: Text(l10n.soundfontsUnlock),
        ),
      ],
    );
  }
}

/// A small pill showing a proposed sound's moderation status.
class _ProposalTag extends StatelessWidget {
  const _ProposalTag({required this.status});

  /// `pending` / `accepted` / `rejected`.
  final String status;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final (label, color) = switch (status) {
      'accepted' => (l10n.proposalStatusAccepted, CymbraColors.primary),
      'rejected' => (l10n.proposalStatusRejected, CymbraColors.error),
      _ => (l10n.proposalStatusPending, CymbraColors.onSurfaceVariant),
    };
    return Container(
      margin: const EdgeInsets.only(right: 4),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}

/// The add/rename end-drawer. In add mode it picks a `.sf2` and names it; in edit
/// mode ([editing] set) it renames.
class _SoundFontFormDrawer extends ConsumerStatefulWidget {
  const _SoundFontFormDrawer({
    super.key,
    required this.editing,
    required this.onDone,
  });

  final PianoEntry? editing;
  final VoidCallback onDone;

  @override
  ConsumerState<_SoundFontFormDrawer> createState() =>
      _SoundFontFormDrawerState();
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

/// The predefined libre licences a user may declare when proposing a sound (only
/// CC0 / CC-BY family — the catalog rejects anything else).
const List<String> _proposeLicenses = [
  'CC0-1.0',
  'CC-BY 3.0',
  'CC-BY 4.0',
  'CC-BY-SA 4.0',
];

/// Propose dialog: a licence **combobox** (predefined choices) with a short
/// description of the selection, plus attribution + a right-to-distribute
/// attestation. Returns `(license, attribution)` on submit.
class _ProposeDialog extends StatefulWidget {
  @override
  State<_ProposeDialog> createState() => _ProposeDialogState();
}

class _ProposeDialogState extends State<_ProposeDialog> {
  String _license = _proposeLicenses.first;
  final _attribution = TextEditingController();
  bool _attested = false;

  @override
  void dispose() {
    _attribution.dispose();
    super.dispose();
  }

  String _licenseDescription(AppLocalizations l10n) {
    if (_license.startsWith('CC0')) return l10n.licenseDescCc0;
    if (_license.startsWith('CC-BY-SA')) return l10n.licenseDescCcbysa;
    return l10n.licenseDescCcby;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final canSubmit = _attested; // a licence is always selected
    return AlertDialog(
      title: Text(l10n.proposeTitle),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(l10n.proposeIntro),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              initialValue: _license,
              decoration: InputDecoration(labelText: l10n.proposeLicense),
              items: [
                for (final l in _proposeLicenses)
                  DropdownMenuItem<String>(value: l, child: Text(l)),
              ],
              onChanged: (v) => setState(() => _license = v ?? _license),
            ),
            Padding(
              padding: const EdgeInsets.only(top: 6),
              child: Text(
                _licenseDescription(l10n),
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                ),
              ),
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
                  license: _license,
                  attribution: _attribution.text.trim(),
                ))
              : null,
          child: Text(l10n.proposeSubmit),
        ),
      ],
    );
  }
}
