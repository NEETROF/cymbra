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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'grpc_client.dart';
import 'rpc_deadlines.dart';
import 'sound_clip_player.dart';
import 'soundfont_source.dart' show soundFontDeliveryOrigin;
import 'token_store.dart';

part 'soundfont_preview_service.g.dart';

/// Thrown when a preview clip can't be fetched for a reason other than "absent" — a
/// network/backend fault. A *missing* preview (HTTP 404) is NOT an error: [audition]
/// returns `false` so the caller greys the control rather than surfacing a failure.
class SoundFontPreviewException implements Exception {
  const SoundFontPreviewException(this.message);
  final String message;

  @override
  String toString() => 'SoundFontPreviewException: $message';
}

/// Auditions a catalog font by fetching its server-rendered preview clip
/// (`GET /soundfonts/{id}/preview`) and playing it (change:
/// add-soundfont-entitlement-previews). This is how a **locked** reward font stays
/// auditionable **without ever downloading its `.sf2` bytes**. Behind a provider so
/// the catalog screen is testable with a fake (no network, no audio device).
abstract class SoundFontPreviewService {
  /// Fetch + play [fontId]'s preview clip. Returns `true` when a preview exists and
  /// playback started, `false` when none exists yet (404) — the caller greys the play
  /// control. Throws [SoundFontPreviewException] on any other failure.
  Future<bool> audition(String fontId);

  /// Stop any current preview playback.
  Future<void> stop();
}

/// Production [SoundFontPreviewService]: authenticated fetch of the public preview
/// object, played through the injectable [SoundClipPlayer].
class SoundFontPreviewServiceImpl implements SoundFontPreviewService {
  SoundFontPreviewServiceImpl(this._ref, this._player, {http.Client? client})
    : _client = client ?? http.Client();

  final Ref _ref;
  final SoundClipPlayer _player;
  final http.Client _client;

  @override
  Future<bool> audition(String fontId) async {
    final ep = _ref.read(cymbraEndpointProvider);
    final uri = Uri.parse(
      '${soundFontDeliveryOrigin(ep)}/soundfonts/$fontId/preview',
    );
    final http.Response resp;
    try {
      final tokens = await _ref.read(tokenStoreProvider).readTokens();
      final token = tokens?.accessToken;
      // Bounded media fetch (change: add-client-transport-deadlines): a
      // preview clip is small, so the transfer budget applies. A timeout
      // surfaces as the seam's own exception below, never raw in the UI.
      resp = await _client
          .get(
            uri,
            headers: {
              if (token != null && token.isNotEmpty)
                'authorization': 'Bearer $token',
            },
          )
          .timeout(kTransferDeadline);
    } catch (e) {
      throw SoundFontPreviewException('fetch $fontId preview: $e');
    }
    // No preview object yet (e.g. a font seeded before this change) — not an error.
    if (resp.statusCode == 404) return false;
    if (resp.statusCode != 200) {
      throw SoundFontPreviewException(
        'preview $fontId: HTTP ${resp.statusCode}',
      );
    }
    await _player.play(resp.bodyBytes);
    return true;
  }

  @override
  Future<void> stop() => _player.stop();
}

/// Production preview-service provider. Override in tests with a fake.
@riverpod
SoundFontPreviewService soundFontPreviewService(Ref ref) =>
    SoundFontPreviewServiceImpl(ref, ref.read(soundClipPlayerProvider));
