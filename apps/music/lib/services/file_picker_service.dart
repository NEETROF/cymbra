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

import 'dart:io' show Platform;
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'file_picker_service.g.dart';

/// A file the user picked to contribute: its display [name] and raw [bytes].
class PickedScoreFile {
  final String name;
  final Uint8List bytes;

  const PickedScoreFile({required this.name, required this.bytes});
}

/// Seam over the platform file picker, so the contribution flow can be driven by
/// an in-memory fake in tests (no native picker). Mirrors [MidiService] and
/// [ScoreAssetSource].
abstract class FilePickerService {
  /// Prompt the user to pick a single MusicXML file (`.musicxml` / `.xml` /
  /// `.mxl`). Resolves to `null` when the user cancels or picks nothing.
  Future<PickedScoreFile?> pickScore();

  /// Prompt the user to pick one **or several** MusicXML files (change:
  /// add-private-score-catalog). Resolves to an empty list when the user cancels
  /// or picks nothing; a single-file selection yields a one-element list, and the
  /// caller decides whether that means the wizard or the batch flow.
  Future<List<PickedScoreFile>> pickScores();
}

/// Production [FilePickerService] over the `file_picker` package.
class FilePickerServiceImpl implements FilePickerService {
  const FilePickerServiceImpl();

  @override
  Future<PickedScoreFile?> pickScore() async {
    // Mobile pickers key the extension filter off system-registered types:
    // iOS maps `allowedExtensions` to UTIs and Android to MIME types, and
    // `.mxl`/`.musicxml` are registered as neither — so a `custom` filter greys
    // them out. Fall back to any-file on mobile (our FFI validation is the real
    // gate anyway); keep the extension filter on desktop, where native dialogs
    // match on the extension itself and it gives nicer UX.
    final filtered = !Platform.isIOS && !Platform.isAndroid;
    final result = await FilePicker.platform.pickFiles(
      type: filtered ? FileType.custom : FileType.any,
      allowedExtensions: filtered ? const ['musicxml', 'xml', 'mxl'] : null,
      // Read the bytes in-memory: works uniformly across mobile/desktop and feeds
      // the FFI validator directly, no filesystem path juggling.
      withData: true,
    );
    if (result == null || result.files.isEmpty) return null;
    final file = result.files.first;
    final bytes = file.bytes;
    if (bytes == null) return null;
    return PickedScoreFile(name: file.name, bytes: bytes);
  }

  @override
  Future<List<PickedScoreFile>> pickScores() async {
    // Same platform reasoning as `pickScore` (see above), with multi-selection on.
    final filtered = !Platform.isIOS && !Platform.isAndroid;
    final result = await FilePicker.platform.pickFiles(
      type: filtered ? FileType.custom : FileType.any,
      allowedExtensions: filtered ? const ['musicxml', 'xml', 'mxl'] : null,
      withData: true,
      allowMultiple: true,
    );
    if (result == null) return const [];
    // A file whose bytes the platform could not read is dropped here rather than
    // failing the whole selection: the batch reports per file, not all-or-nothing.
    return [
      for (final f in result.files)
        if (f.bytes case final bytes?)
          PickedScoreFile(name: f.name, bytes: bytes),
    ];
  }
}

/// Production file-picker provider. Override in tests with a fake that returns
/// canned [PickedScoreFile]s.
@riverpod
FilePickerService filePicker(Ref ref) => const FilePickerServiceImpl();
