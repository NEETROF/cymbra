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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import '../services/soundfont_importer.dart';
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
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      return _decode(raw) ?? const [];
    } catch (_) {
      return const []; // storage unavailable / corrupt → empty
    }
  }

  /// Runs the import flow (pick → validate → copy) and, on success, appends the
  /// new piano to the registry and persists it. Returns the imported entry, or
  /// `null` when the user cancels. Rethrows [SoundFontImportException] for an
  /// invalid file so the caller can show a non-fatal message; the registry is
  /// left unchanged in that case.
  Future<PianoEntry?> importSoundFont() async {
    final entry = await ref.read(soundFontImporterProvider).importSoundFont();
    if (entry == null) return null;
    final next = <PianoEntry>[...?state.valueOrNull, entry];
    state = AsyncData(next);
    await _persist(next);
    return entry;
  }

  /// Removes an imported piano: deletes its copied file and drops it from the
  /// registry. A no-op for an unknown id. If the removed piano was selected, the
  /// selection notifier (watching the catalog) falls back to the default.
  Future<void> remove(String id) async {
    final current = state.valueOrNull ?? const <PianoEntry>[];
    final matches = current.where((e) => e.id == id);
    if (matches.isEmpty) return;
    await ref.read(soundFontImporterProvider).deleteImport(matches.first);
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
