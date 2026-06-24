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

import 'package:flutter/services.dart' show rootBundle;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'score_asset_source.g.dart';

/// Production score-asset source provider. Override in tests with a fake that
/// returns in-memory bytes, so notation state/widgets test without the bundle.
@riverpod
ScoreAssetSource scoreAssetSource(Ref ref) =>
    const RootBundleScoreAssetSource();

/// Seam over MusicXML asset loading.
///
/// The notation notifier depends on this interface rather than on `rootBundle`
/// directly, mirroring [MidiService], so it can be driven by an in-memory fake
/// in unit/widget tests.
abstract class ScoreAssetSource {
  /// Reads the asset at [assetPath] and returns its raw bytes.
  Future<Uint8List> load(String assetPath);
}

/// Production [ScoreAssetSource] backed by the Flutter asset bundle.
class RootBundleScoreAssetSource implements ScoreAssetSource {
  const RootBundleScoreAssetSource();

  @override
  Future<Uint8List> load(String assetPath) async {
    final data = await rootBundle.load(assetPath);
    return data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes);
  }
}
