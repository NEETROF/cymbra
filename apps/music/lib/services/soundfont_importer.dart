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

import 'dart:io' show File, Platform;
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:uuid/uuid.dart';

import '../state/piano_catalog.dart';
import 'soundfont_storage.dart';

part 'soundfont_importer.g.dart';

/// Thrown when the file the user picked is not a loadable SoundFont, so the
/// import flow can reject it non-fatally (a message, no crash, catalog
/// unchanged).
class SoundFontImportException implements Exception {
  const SoundFontImportException();
}

/// Whether [bytes] look like a loadable SoundFont (`.sf2`): a RIFF container
/// tagged `sfbk`. Mirrors the engine's `is_valid_soundfont` pre-check so a bad
/// file is rejected before it is ever copied or handed to the synth.
bool isValidSoundFont(Uint8List bytes) =>
    bytes.length >= 12 &&
    bytes[0] == 0x52 && // 'R'
    bytes[1] == 0x49 && // 'I'
    bytes[2] == 0x46 && // 'F'
    bytes[3] == 0x46 && // 'F'
    bytes[8] == 0x73 && // 's'
    bytes[9] == 0x66 && // 'f'
    bytes[10] == 0x62 && // 'b'
    bytes[11] == 0x6b; // 'k'

/// A `.sf2` the user picked (and that passed validation) but hasn't been saved
/// yet — its bytes plus a suggested label from the filename. The management
/// drawer lets the user edit the label before committing via [SoundFontImporter.save].
class PickedSoundFont {
  const PickedSoundFont({required this.bytes, required this.suggestedLabel});
  final Uint8List bytes;
  final String suggestedLabel;
}

/// Seam over the SoundFont import flow (pick → validate → copy into app
/// storage), so the settings drawer can be driven by an in-memory fake in tests
/// (no native picker, no filesystem). Mirrors [FilePickerService].
abstract class SoundFontImporter {
  /// Prompts the user to pick a `.sf2` and validates it is a loadable SoundFont,
  /// **without** copying it yet. Resolves to `null` when the user cancels. Throws
  /// [SoundFontImportException] when the picked file is not a valid SoundFont.
  Future<PickedSoundFont?> pick();

  /// Copies [bytes] into durable app storage under the given [label] and returns
  /// a `user`-kind catalog entry. Called after the user confirms a [pick].
  Future<PianoEntry> save(Uint8List bytes, String label);

  /// One-shot pick + save with the filename as label (the picker dropdown's
  /// "Add" affordance). `null` on cancel; throws [SoundFontImportException] for an
  /// invalid file.
  Future<PianoEntry?> importSoundFont();

  /// Deletes the copied file backing an imported [entry] (best-effort; a missing
  /// file is not an error). Called when the user removes an imported piano.
  Future<void> deleteImport(PianoEntry entry);
}

/// Production [SoundFontImporter] over `file_picker` + app storage.
class SoundFontImporterImpl implements SoundFontImporter {
  SoundFontImporterImpl(this._ref);

  final Ref _ref;

  @override
  Future<PickedSoundFont?> pick() async {
    // Desktop dialogs filter by extension nicely; mobile pickers key off
    // system-registered UTIs/MIME types, and `.sf2` is registered as neither —
    // a `custom` filter would grey the file out. So filter on desktop only and
    // let the header check be the real gate on mobile (mirrors the score picker).
    final filtered = !Platform.isIOS && !Platform.isAndroid;
    final result = await FilePicker.platform.pickFiles(
      type: filtered ? FileType.custom : FileType.any,
      allowedExtensions: filtered ? const ['sf2'] : null,
      withData: true,
    );
    if (result == null || result.files.isEmpty) return null;
    final picked = result.files.first;
    final bytes = picked.bytes;
    if (bytes == null || !isValidSoundFont(bytes)) {
      throw const SoundFontImportException();
    }
    // Suggested label = filename without the .sf2 extension.
    final name = picked.name;
    final label = name.toLowerCase().endsWith('.sf2')
        ? name.substring(0, name.length - 4)
        : name;
    return PickedSoundFont(
      bytes: bytes,
      suggestedLabel: label.isEmpty ? name : label,
    );
  }

  @override
  Future<PianoEntry> save(Uint8List bytes, String label) async {
    final id = const Uuid().v4();
    final dir = await _ref.read(soundFontStorageDirProvider.future);
    final dest = File('${dir.path}/$id.sf2');
    await dest.writeAsBytes(bytes, flush: true);
    return PianoEntry(
      id: id,
      label: label.trim().isEmpty ? id : label.trim(),
      kind: PianoKind.user,
      source: dest.path,
    );
  }

  @override
  Future<PianoEntry?> importSoundFont() async {
    final picked = await pick();
    if (picked == null) return null;
    return save(picked.bytes, picked.suggestedLabel);
  }

  @override
  Future<void> deleteImport(PianoEntry entry) async {
    try {
      final file = File(entry.source);
      if (await file.exists()) await file.delete();
    } catch (_) {
      // Best-effort: a missing/locked file must not block removing the entry.
    }
  }
}

/// Production importer provider. Override in tests with a fake that returns a
/// canned [PianoEntry] (or throws [SoundFontImportException]).
@riverpod
SoundFontImporter soundFontImporter(Ref ref) => SoundFontImporterImpl(ref);
