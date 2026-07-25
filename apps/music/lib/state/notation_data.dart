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

import 'package:freezed_annotation/freezed_annotation.dart';

import '../src/rust/api/musicxml.dart';

part 'notation_data.freezed.dart';

/// Why a score failed to load — a typed cause so the UI can show a *specific*
/// localized message (never the raw exception/gRPC text).
enum ScoreLoadFailure {
  /// The score no longer exists (its catalog row is gone).
  notFound,

  /// The score exists but its bytes aren't available yet (e.g. not synced to the
  /// serving store yet, or gated pending review).
  notAvailableYet,

  /// The backend is unreachable / offline.
  unavailable,

  /// Any other failure (parse error, unexpected).
  generic,
}

/// Immutable Partition-mode state: the parsed document, its laid-out systems for
/// the current [availableWidth], and a typed [failure] when loading/parsing
/// failed. Held by the `Notation` notifier and consumed by `PartitionPainter`.
@freezed
abstract class NotationData with _$NotationData {
  const NotationData._();

  const factory NotationData({
    /// The parsed score, or null before a score is loaded / on error.
    ScoreDocument? document,

    /// Systems laid out for [availableWidth] (empty until a document loads).
    @Default(<System>[]) List<System> systems,

    /// The viewport width last used to lay out [systems].
    @Default(0.0) double availableWidth,

    /// The typed cause when loading/parsing failed; null on success.
    ScoreLoadFailure? failure,
  }) = _NotationData;

  /// True once a score has been parsed without error.
  bool get hasDocument => document != null && failure == null;

  /// True when the last load failed.
  bool get hasFailure => failure != null;
}
