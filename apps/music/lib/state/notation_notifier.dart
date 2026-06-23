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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/notation_engine.dart';
import '../services/score_asset_source.dart';
import 'notation_data.dart';
import 'score_catalog.dart';

part 'notation_notifier.g.dart';

/// Loads the selected score's MusicXML asset, parses it via the Rust bridge, and
/// lays it out into systems — exposing the result as immutable [NotationData].
///
/// Re-builds whenever [SelectedScore] changes. Layout is recomputed cheaply (a
/// synchronous bridge call) when the viewport width changes; the document is
/// only re-parsed when the selected score changes.
@riverpod
class Notation extends _$Notation {
  /// Width used for the first layout, before the screen reports its real size.
  static const double _initialWidth = 800;

  /// Minimum width change (px) that triggers a re-layout, to avoid thrashing.
  static const double _relayoutThreshold = 8;

  @override
  NotationData build() {
    final entry = ref.watch(selectedScoreProvider);
    if (entry != null) {
      // `_load` reads `state`, which is not set during build(); defer it to a
      // microtask so it runs after this build returns and sets state when it
      // resolves.
      Future.microtask(() => _load(entry));
    }
    return const NotationData();
  }

  ScoreAssetSource get _source => ref.read(scoreAssetSourceProvider);
  NotationEngine get _engine => ref.read(notationEngineProvider);

  Future<void> _load(CatalogEntry entry) async {
    final width = state.availableWidth > 0
        ? state.availableWidth
        : _initialWidth;
    try {
      final bytes = await _source.load(entry.assetPath);
      final document = await _engine.parse(bytes);
      // Guard against a selection change while we were loading.
      if (ref.read(selectedScoreProvider) != entry) return;
      final systems = _engine.layout(document, width);
      state = NotationData(
        document: document,
        systems: systems,
        availableWidth: width,
      );
    } catch (e) {
      if (ref.read(selectedScoreProvider) != entry) return;
      state = NotationData(error: e.toString(), availableWidth: width);
    }
  }

  /// Updates the viewport width and re-lays-out the cached document. No-op when
  /// the change is below [_relayoutThreshold] or no document is loaded yet.
  void setAvailableWidth(double width) {
    if (width <= 0) return;
    if ((width - state.availableWidth).abs() < _relayoutThreshold) return;
    final document = state.document;
    if (document == null) {
      state = state.copyWith(availableWidth: width);
      return;
    }
    state = state.copyWith(
      availableWidth: width,
      systems: _engine.layout(document, width),
    );
  }
}
