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
import 'package:http/http.dart' as http;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'grpc_client.dart';
import 'soundfont_source.dart' show soundFontDeliveryOrigin;
import 'token_store.dart';

part 'score_preview_service.g.dart';

/// Thrown when a score teaser can't be fetched for a reason other than "absent" —
/// a network/backend fault. A *missing* teaser (HTTP 404) is NOT an error:
/// [ScorePreviewService.fetchClip] returns `null` so the caller greys the control.
class ScorePreviewException implements Exception {
  const ScorePreviewException(this.message);
  final String message;

  @override
  String toString() => 'ScorePreviewException: $message';
}

/// Fetches a catalog piece's server-rendered audio teaser
/// (`GET /scores/{id}/preview`, change: add-score-daily-access-rewards, design
/// D8) — the twin of the SoundFont preview service: the piece's MusicXML is never
/// delivered to audition it. Playback belongs to `ScorePreviewPlayback` (one clip
/// at a time, played once). Behind a provider so the cards and the unlock sheet
/// are testable with a fake (no network).
abstract class ScorePreviewService {
  /// The WAV bytes of [catalogId]'s teaser, or `null` when none exists yet (404)
  /// — the caller greys the control. Throws [ScorePreviewException] on any other
  /// failure.
  Future<Uint8List?> fetchClip(String catalogId);
}

/// Production [ScorePreviewService]: authenticated fetch of the teaser object.
class ScorePreviewServiceImpl implements ScorePreviewService {
  ScorePreviewServiceImpl(this._ref, {http.Client? client})
    : _client = client ?? http.Client();

  final Ref _ref;
  final http.Client _client;

  @override
  Future<Uint8List?> fetchClip(String catalogId) async {
    final ep = _ref.read(cymbraEndpointProvider);
    // The score-preview route lives on the same HTTP origin as the SoundFont
    // delivery/preview routes.
    final uri = Uri.parse(
      '${soundFontDeliveryOrigin(ep)}/scores/$catalogId/preview',
    );
    final http.Response resp;
    try {
      final tokens = await _ref.read(tokenStoreProvider).readTokens();
      final token = tokens?.accessToken;
      resp = await _client.get(
        uri,
        headers: {
          if (token != null && token.isNotEmpty)
            'authorization': 'Bearer $token',
        },
      );
    } catch (e) {
      throw ScorePreviewException('fetch $catalogId preview: $e');
    }
    if (resp.statusCode == 404) return null;
    if (resp.statusCode != 200) {
      throw ScorePreviewException(
        'preview $catalogId: HTTP ${resp.statusCode}',
      );
    }
    return resp.bodyBytes;
  }
}

/// Production score-preview provider. Override in tests with a fake.
@riverpod
ScorePreviewService scorePreviewService(Ref ref) =>
    ScorePreviewServiceImpl(ref);
