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

import 'catalog_access_state.dart';

/// The result of a (possibly conditional) score-bytes fetch (change:
/// add-offline-score-cache). Carries the server content hash (ETag) so the offline
/// cache can store it and, on a later online open, ask the backend "still this
/// hash?" — when [unchanged] the payload is omitted and the cached copy is reused,
/// saving the re-download and re-encrypt.
class ScoreBytesResult {
  /// The canonical MusicXML bytes, or `null` when [unchanged] (the caller keeps
  /// its cached copy).
  final Uint8List? data;

  /// The stored content hash of these bytes (opaque ETag), echoed back so the
  /// client can cache it and send it as the next `ifNoneMatch`.
  final String etag;

  /// `true` when the supplied `ifNoneMatch` matched the stored hash: [data] is
  /// `null` and the caller reuses its cached bytes.
  final bool unchanged;

  /// The caller's daily-access state (change: add-score-daily-access-rewards),
  /// attached by the CATALOG bytes fetch when the gate is wired; `null` for a
  /// user upload or when no gate exists. When [locked], [data] is `null` AND
  /// [unchanged] is `false`: the piece was refused by the daily quota — a state,
  /// not an error, and never "play your cached copy".
  final CatalogAccessState? access;

  const ScoreBytesResult({
    this.data,
    required this.etag,
    this.unchanged = false,
    this.access,
  });

  /// The piece was refused by the daily quota.
  bool get locked => access?.locked ?? false;
}
