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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/rust/api/musicxml.dart' as musicxml_api;
import '../src/rust/api/musicxml.dart' show ScoreDocument, System;

part 'notation_engine.g.dart';

/// Production notation-engine provider. Override in tests with a fake so the
/// notation notifier/widgets can be exercised without the native library.
@riverpod
NotationEngine notationEngine(Ref ref) => const FrbNotationEngine();

/// Seam over the Rust MusicXML engine.
///
/// The notation notifier depends on this interface instead of the generated
/// flutter_rust_bridge functions directly, so a fake can supply hand-built
/// documents/systems in unit/widget tests (mirrors [MidiService]/[ScoreSource]).
abstract class NotationEngine {
  /// Parses uncompressed MusicXML bytes into a structured document.
  Future<ScoreDocument> parse(Uint8List bytes);

  /// Lays the document's measures out into systems for [availableWidth].
  List<System> layout(ScoreDocument document, double availableWidth);
}

/// Production [NotationEngine] backed by the generated flutter_rust_bridge API.
class FrbNotationEngine implements NotationEngine {
  const FrbNotationEngine();

  @override
  Future<ScoreDocument> parse(Uint8List bytes) =>
      musicxml_api.parseMusicxml(bytes: bytes);

  @override
  List<System> layout(ScoreDocument document, double availableWidth) =>
      musicxml_api.layoutSystems(doc: document, availableWidth: availableWidth);
}
