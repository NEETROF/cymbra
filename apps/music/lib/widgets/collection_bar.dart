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
import '../services/score_upload_service.dart';
import '../state/score_collections.dart';
import '../theme/cymbra_theme.dart';

/// Localized wording for a collection failure — the raw error never shows.
String collectionErrorMessage(AppLocalizations l10n, CollectionError e) =>
    switch (e) {
      CollectionError.nameTaken => l10n.collectionsErrorNameTaken,
      CollectionError.invalidName => l10n.collectionsErrorInvalidName,
      CollectionError.failed => l10n.collectionsErrorFailed,
    };

/// The private library's collection filter (change: add-private-score-catalog):
/// "all my scores" plus one chip per collection, and a way in to managing them.
/// Shown only in the private-library view.
class CollectionBar extends ConsumerWidget {
  const CollectionBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final collections = ref.watch(scoreCollectionsProvider);
    final selected = ref.watch(collectionFilterProvider);
    final filter = ref.read(collectionFilterProvider.notifier);

    // While the list loads (or fails) the library simply shows unfiltered — a
    // collection list is an aid, never a gate on seeing one's own scores.
    final items = collections.valueOrNull ?? const <ScoreCollection>[];

    return SizedBox(
      height: 40,
      child: ListView(
        scrollDirection: Axis.horizontal,
        children: [
          FilterChip(
            label: Text(l10n.collectionsAll),
            selected: selected == null,
            showCheckmark: false,
            onSelected: (_) => filter.select(null),
          ),
          for (final c in items) ...[
            const SizedBox(width: 6),
            FilterChip(
              label: Text(c.name),
              selected: selected == c.id,
              showCheckmark: false,
              onSelected: (on) => filter.select(on ? c.id : null),
            ),
          ],
          const SizedBox(width: 6),
          ActionChip(
            avatar: const Icon(Icons.settings_outlined, size: 18),
            label: Text(l10n.collectionsManage),
            onPressed: () => showCollectionManager(context),
          ),
        ],
      ),
    );
  }
}

/// Manage collections: create, rename, delete. Deleting says plainly that the
/// scores are kept — the destructive-sounding action is not destructive.
Future<void> showCollectionManager(BuildContext context) =>
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      builder: (_) => const _CollectionManagerSheet(),
    );

class _CollectionManagerSheet extends ConsumerWidget {
  const _CollectionManagerSheet();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final collections = ref.watch(scoreCollectionsProvider).valueOrNull ?? [];
    final notifier = ref.read(scoreCollectionsProvider.notifier);

    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.collectionsManage,
              style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            if (collections.isEmpty)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 12),
                child: Text(l10n.collectionsEmpty),
              ),
            for (final c in collections)
              ListTile(
                dense: true,
                title: Text(c.name),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      icon: const Icon(Icons.edit_outlined),
                      tooltip: l10n.collectionsRename,
                      onPressed: () async {
                        final name = await promptCollectionName(
                          context,
                          initial: c.name,
                        );
                        if (name == null || !context.mounted) return;
                        final err = await notifier.rename(c.id, name);
                        if (!context.mounted) return;
                        _report(context, l10n, err);
                      },
                    ),
                    IconButton(
                      icon: const Icon(Icons.delete_outline),
                      tooltip: l10n.collectionsDelete,
                      onPressed: () async {
                        final ok = await _confirmDelete(context, l10n);
                        if (!ok || !context.mounted) return;
                        final err = await notifier.remove(c.id);
                        if (!context.mounted) return;
                        _report(context, l10n, err);
                      },
                    ),
                  ],
                ),
              ),
            const SizedBox(height: 8),
            FilledButton.icon(
              icon: const Icon(Icons.add),
              label: Text(l10n.collectionsNew),
              onPressed: () async {
                final name = await promptCollectionName(context);
                if (name == null || !context.mounted) return;
                final err = await notifier.create(name);
                if (!context.mounted) return;
                _report(context, l10n, err);
              },
            ),
          ],
        ),
      ),
    );
  }
}

Future<bool> _confirmDelete(BuildContext context, AppLocalizations l10n) async {
  final ok = await showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      content: Text(l10n.collectionsDeleteConfirm),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(false),
          child: Text(l10n.commonCancel),
        ),
        FilledButton(
          onPressed: () => Navigator.of(ctx).pop(true),
          child: Text(l10n.collectionsDelete),
        ),
      ],
    ),
  );
  return ok ?? false;
}

/// Ask for a collection name. Returns `null` when cancelled.
Future<String?> promptCollectionName(
  BuildContext context, {
  String initial = '',
}) {
  final l10n = AppLocalizations.of(context);
  final controller = TextEditingController(text: initial);
  return showDialog<String>(
    context: context,
    builder: (ctx) => AlertDialog(
      content: TextField(
        controller: controller,
        autofocus: true,
        decoration: InputDecoration(labelText: l10n.collectionsNameLabel),
        onSubmitted: (v) => Navigator.of(ctx).pop(v),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(),
          child: Text(l10n.commonCancel),
        ),
        FilledButton(
          onPressed: () => Navigator.of(ctx).pop(controller.text),
          child: Text(l10n.commonSave),
        ),
      ],
    ),
  );
}

/// Put one of the caller's scores into a collection.
Future<void> showAddToCollection(BuildContext context, String scoreId) =>
    showModalBottomSheet(
      context: context,
      builder: (_) => _AddToCollectionSheet(scoreId: scoreId),
    );

class _AddToCollectionSheet extends ConsumerWidget {
  const _AddToCollectionSheet({required this.scoreId});

  final String scoreId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final collections = ref.watch(scoreCollectionsProvider).valueOrNull ?? [];
    final notifier = ref.read(scoreCollectionsProvider.notifier);

    return SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(
              l10n.collectionsAddTo,
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
            ),
          ),
          if (collections.isEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(l10n.collectionsEmpty),
            ),
          for (final c in collections)
            ListTile(
              title: Text(c.name),
              onTap: () async {
                final err = await notifier.addScore(c.id, scoreId);
                if (!context.mounted) return;
                Navigator.of(context).maybePop();
                if (err != null) {
                  _report(context, l10n, err);
                } else {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.collectionsAdded(c.name))),
                  );
                }
              },
            ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}

/// Show a localized failure, or nothing when the action worked.
void _report(BuildContext context, AppLocalizations l10n, CollectionError? e) {
  if (e == null) return;
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(
      backgroundColor: CymbraColors.error,
      content: Text(collectionErrorMessage(l10n, e)),
    ),
  );
}
