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

import 'dart:convert';
import 'dart:io' show Directory, File;
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show listEquals;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import '../services/private_soundfont_service.dart';
import '../services/soundfont_importer.dart';
import '../services/soundfont_storage.dart';
import 'piano_catalog.dart';

part 'imported_soundfonts.g.dart';

/// The user's imported SoundFonts (all [PianoKind.user]), persisted so they
/// survive relaunches. An [AsyncNotifier] so its `.future` is a first-class
/// restore signal — the selection notifier awaits it before validating a
/// persisted selection, so a selected import is never mistaken for an unknown id
/// during the startup window (and no public getter is needed).
///
/// The catalog is the union of the built-ins and this list; the picker's
/// "Add SoundFont…" and remove affordances drive it. When a currently-selected
/// import is removed, the selection notifier reacts to the catalog change and
/// falls back to the default — this notifier never pokes it directly.
@Riverpod(keepAlive: true)
class ImportedSoundFonts extends _$ImportedSoundFonts {
  /// Preferences key under which the JSON-encoded registry lives.
  static const String prefsKey = 'imported_soundfonts';

  @override
  Future<List<PianoEntry>> build() async {
    final local = await _loadLocal();
    // Best-effort server sync (change: add-soundfont-moderation): imports are now
    // server-backed and follow the user across devices. Offline / unauthenticated
    // → the local registry stands, and syncs on a later build.
    try {
      return await _sync(local);
    } catch (_) {
      return local;
    }
  }

  /// Re-runs the server sync against the current registry — e.g. when the sound
  /// hub is shown — so a moderation decision made elsewhere (a proposal going
  /// `pending` → `accepted`/`rejected`) is reflected without a relaunch. Keeps the
  /// current data visible (no loading flash) and is best-effort: an offline/error
  /// sync leaves the current state untouched.
  Future<void> refresh() async {
    final current = state.valueOrNull;
    if (current == null) return; // still loading; build() will sync
    try {
      state = AsyncData(await _sync(current));
    } catch (_) {
      // Keep the current state on a sync failure.
    }
  }

  Future<List<PianoEntry>> _loadLocal() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      return _decode(raw) ?? const [];
    } catch (_) {
      return const []; // storage unavailable / corrupt → empty
    }
  }

  /// Reconciles the local registry with the private server library: migrates any
  /// local-only import up (idempotent by content), then pulls the server library
  /// down, caching each font's bytes to a local file the engine can load.
  Future<List<PianoEntry>> _sync(List<PianoEntry> local) async {
    final service = ref.read(privateSoundFontServiceProvider);
    final migrated = await _migrateLocalImports(local, service);
    final remoteList = await service.list();
    final result = await _pullServerLibrary(migrated, remoteList, service);
    // Only persist when the sync actually changed the registry, so a no-op sync
    // never writes. `result` carries no proposal status, so the comparison stays
    // structural.
    if (!listEquals(local, result)) await _persist(result);
    return _withProposalStatus(result, remoteList);
  }

  /// Upload any local-only import (no server id yet) to the private library. Guards
  /// with a synchronous existence check so a missing file never spawns real file
  /// I/O (which would otherwise leave `build()` pending in a widget test).
  Future<List<PianoEntry>> _migrateLocalImports(
    List<PianoEntry> local,
    PrivateSoundFontService service,
  ) async {
    final migrated = <PianoEntry>[];
    for (final e in local) {
      if (e.remoteId != null || !File(e.source).existsSync()) {
        migrated.add(e);
        continue;
      }
      try {
        final bytes = await File(e.source).readAsBytes();
        final remote = await service.import(bytes, e.label);
        migrated.add(e.copyWith(remoteId: remote.id));
      } catch (_) {
        migrated.add(e); // keep as a local-only import; retries next build
      }
    }
    return migrated;
  }

  /// Add any server font not already held, downloading its bytes to a cache file.
  Future<List<PianoEntry>> _pullServerLibrary(
    List<PianoEntry> migrated,
    List<RemoteSoundFont> remoteList,
    PrivateSoundFontService service,
  ) async {
    final held = {
      for (final e in migrated)
        if (e.remoteId != null) e.remoteId!,
    };
    final toFetch = remoteList.where((r) => !held.contains(r.id)).toList();
    if (toFetch.isEmpty) return migrated;
    // Only resolve the (platform-backed) storage dir when there's a font to cache,
    // so a no-op sync never depends on path_provider.
    final dir = await ref.read(soundFontStorageDirProvider.future);
    final result = <PianoEntry>[...migrated];
    for (final r in toFetch) {
      final entry = await _cacheServerFont(r, dir, service);
      if (entry != null) result.add(entry);
    }
    return result;
  }

  /// Download + cache one server font's bytes to a local file the FFI can load;
  /// `null` when the bytes couldn't be fetched (it stays server-side).
  Future<PianoEntry?> _cacheServerFont(
    RemoteSoundFont r,
    Directory dir,
    PrivateSoundFontService service,
  ) async {
    try {
      final file = File('${dir.path}/remote-${r.id}.sf2');
      if (!file.existsSync()) {
        await file.writeAsBytes(await service.download(r.id), flush: true);
      }
      return PianoEntry(
        id: r.id,
        label: r.label,
        kind: PianoKind.user,
        source: file.path,
        remoteId: r.id,
      );
    } catch (_) {
      return null;
    }
  }

  /// Attach each font's (server-derived, non-persisted) proposal moderation status
  /// so the UI can show a tag + hide the propose action once submitted.
  List<PianoEntry> _withProposalStatus(
    List<PianoEntry> result,
    List<RemoteSoundFont> remoteList,
  ) {
    final statusByRemote = {for (final r in remoteList) r.id: r.proposalStatus};
    return [
      for (final e in result)
        (e.remoteId != null && statusByRemote[e.remoteId] != null)
            ? e.copyWith(proposalStatus: statusByRemote[e.remoteId])
            : e,
    ];
  }

  /// Runs the import flow (pick → validate → copy), uploads the font to the
  /// user's private server library so it syncs across devices, and appends it to
  /// the registry. Returns the imported entry, or `null` when the user cancels.
  /// Rethrows [SoundFontImportException] for an invalid file (registry
  /// unchanged). A failed **upload** is non-fatal: the import stays local-only and
  /// syncs on a later build.
  Future<PianoEntry?> importSoundFont() async {
    final entry = await ref.read(soundFontImporterProvider).importSoundFont();
    if (entry == null) return null;
    var synced = entry;
    try {
      // `existsSync` guard: only touch the filesystem for a real copied file, so a
      // fake importer (tests) doesn't spawn real I/O the widget harness can't drain.
      if (File(entry.source).existsSync()) {
        final bytes = await File(entry.source).readAsBytes();
        final remote = await ref
            .read(privateSoundFontServiceProvider)
            .import(bytes, entry.label);
        synced = entry.copyWith(remoteId: remote.id);
      }
    } catch (_) {
      // Offline / server error: keep the local import; it migrates on next sync.
    }
    final next = <PianoEntry>[...?state.valueOrNull, synced];
    state = AsyncData(next);
    await _persist(next);
    return synced;
  }

  /// Saves a picked `.sf2` under [label] into the private library (copy locally +
  /// upload to the server) and appends it. Used by the dedicated management
  /// screen's add drawer, where the user sets the label before committing. A
  /// failed upload is non-fatal (kept local, syncs later).
  Future<PianoEntry> addImport(Uint8List bytes, String label) async {
    final entry = await ref.read(soundFontImporterProvider).save(bytes, label);
    var synced = entry;
    try {
      final remote = await ref
          .read(privateSoundFontServiceProvider)
          .import(bytes, entry.label);
      synced = entry.copyWith(remoteId: remote.id);
    } catch (_) {
      // Offline / server error: keep the local import; it migrates on next sync.
    }
    final next = <PianoEntry>[...?state.valueOrNull, synced];
    state = AsyncData(next);
    await _persist(next);
    return synced;
  }

  /// Renames an imported font. Local-only + persisted: the sync keeps entries it
  /// already holds, so a rename survives re-syncs without a server round-trip.
  Future<void> rename(String id, String label) async {
    final current = state.valueOrNull ?? const <PianoEntry>[];
    final trimmed = label.trim();
    if (trimmed.isEmpty) return;
    final next = <PianoEntry>[
      for (final e in current)
        if (e.id == id) e.copyWith(label: trimmed) else e,
    ];
    state = AsyncData(next);
    await _persist(next);
  }

  /// Proposes an imported font to the public catalog with a mandatory licence
  /// declaration + right-to-distribute [attestation] (change:
  /// add-soundfont-moderation). Throws [PrivateSoundFontException] if the font has
  /// not synced to the server yet or the server refuses the proposal.
  Future<void> proposeToPublicCatalog(
    String id, {
    required String license,
    String attribution = '',
    required bool attestation,
  }) async {
    final current = state.valueOrNull ?? const <PianoEntry>[];
    final matches = current.where((e) => e.id == id);
    if (matches.isEmpty) return;
    final remoteId = matches.first.remoteId;
    if (remoteId == null) {
      throw const PrivateSoundFontException('font not synced yet');
    }
    await ref
        .read(privateSoundFontServiceProvider)
        .propose(
          remoteId,
          license: license,
          attribution: attribution,
          attestation: attestation,
        );
    // Optimistically reflect the pending submission so the UI updates immediately
    // and survives a relaunch (persisted); the next sync upgrades it to
    // accepted/rejected from the server.
    final next = <PianoEntry>[
      for (final e in current)
        if (e.id == id) e.copyWith(proposalStatus: 'pending') else e,
    ];
    state = AsyncData(next);
    await _persist(next);
  }

  /// Removes an imported piano: deletes its private server copy (so it stops
  /// syncing to other devices), deletes the local cached file, and drops it from
  /// the registry. A no-op for an unknown id. If the removed piano was selected,
  /// the selection notifier (watching the catalog) falls back to the default.
  Future<void> remove(String id) async {
    final current = state.valueOrNull ?? const <PianoEntry>[];
    final matches = current.where((e) => e.id == id);
    if (matches.isEmpty) return;
    final entry = matches.first;
    // Remove server-side first (best-effort) so it doesn't re-sync back.
    final remoteId = entry.remoteId;
    if (remoteId != null) {
      try {
        await ref.read(privateSoundFontServiceProvider).delete(remoteId);
      } catch (_) {
        // Best-effort: a failed server delete must not block local removal.
      }
    }
    await ref.read(soundFontImporterProvider).deleteImport(entry);
    final next = <PianoEntry>[
      for (final e in current)
        if (e.id != id) e,
    ];
    state = AsyncData(next);
    await _persist(next);
  }

  Future<void> _persist(List<PianoEntry> entries) async {
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, _encode(entries));
    } catch (_) {
      // Best-effort: the in-memory registry still applies this session.
    }
  }

  static String _encode(List<PianoEntry> entries) =>
      jsonEncode([for (final e in entries) e.toJson()]);

  static List<PianoEntry>? _decode(String? raw) {
    if (raw == null) return null;
    try {
      final list = jsonDecode(raw) as List<dynamic>;
      return [
        for (final item in list)
          PianoEntry.fromJson(item as Map<String, dynamic>),
      ].where((e) => e.kind == PianoKind.user).toList();
    } catch (_) {
      return null; // corrupt value → keep defaults
    }
  }
}
