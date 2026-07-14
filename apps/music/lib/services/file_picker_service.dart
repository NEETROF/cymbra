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
}

/// Production [FilePickerService] over the `file_picker` package.
class FilePickerServiceImpl implements FilePickerService {
  const FilePickerServiceImpl();

  @override
  Future<PickedScoreFile?> pickScore() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['musicxml', 'xml', 'mxl'],
      // Read the bytes in-memory: works uniformly across mobile/desktop/web and
      // feeds the FFI validator directly, no filesystem path juggling.
      withData: true,
    );
    if (result == null || result.files.isEmpty) return null;
    final file = result.files.first;
    final bytes = file.bytes;
    if (bytes == null) return null;
    return PickedScoreFile(name: file.name, bytes: bytes);
  }
}

/// Production file-picker provider. Override in tests with a fake that returns
/// canned [PickedScoreFile]s.
@riverpod
FilePickerService filePicker(Ref ref) => const FilePickerServiceImpl();
