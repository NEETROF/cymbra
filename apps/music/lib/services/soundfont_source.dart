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

import 'dart:io' show File;

import 'package:flutter/services.dart' show rootBundle;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../state/piano_catalog.dart';
import 'grpc_client.dart';
import 'soundfont_importer.dart' show isValidSoundFont;
import 'soundfont_storage.dart';
import 'token_store.dart';

part 'soundfont_source.g.dart';

/// Thrown when a piano's SoundFont bytes cannot be obtained (a download failed
/// or is offline, or an imported file is missing). The selection notifier
/// catches it and falls back to the bundled default.
class SoundFontUnavailableException implements Exception {
  const SoundFontUnavailableException(this.message);
  final String message;

  @override
  String toString() => 'SoundFontUnavailableException: $message';
}

/// Seam that resolves a [PianoEntry] to a local `.sf2` file path the engine can
/// load. Bundled pianos are extracted from the asset bundle; download pianos are
/// fetched once and cached; user pianos load from their copied file. Behind a
/// provider so tests inject a fake returning a fixed path without touching
/// assets, network, or the filesystem.
abstract class SoundFontSource {
  /// Resolves [entry] to a readable local `.sf2` path. Throws
  /// [SoundFontUnavailableException] when the bytes cannot be obtained.
  Future<String> resolve(PianoEntry entry);
}

/// Production [SoundFontSource]: asset extraction, authenticated download with
/// on-disk caching, and imported-file lookup — all into the durable
/// [soundFontStorageDirProvider].
class SoundFontSourceImpl implements SoundFontSource {
  SoundFontSourceImpl(this._ref, {http.Client? client})
    : _client = client ?? http.Client();

  final Ref _ref;
  final http.Client _client;

  @override
  Future<String> resolve(PianoEntry entry) async {
    switch (entry.kind) {
      case PianoKind.bundled:
        return _extractAsset(entry.source);
      case PianoKind.user:
        return _resolveUserFile(entry.source);
      case PianoKind.download:
        return _fetchAndCache(entry.source);
    }
  }

  /// Extracts a bundled asset to the storage dir on first use (the asset
  /// filename is the cache key) and returns the cached file path.
  Future<String> _extractAsset(String assetPath) async {
    final dir = await _ref.read(soundFontStorageDirProvider.future);
    final file = File('${dir.path}/${assetPath.split('/').last}');
    if (!await file.exists()) {
      try {
        final data = await rootBundle.load(assetPath);
        await file.writeAsBytes(
          data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes),
          flush: true,
        );
      } catch (e) {
        throw SoundFontUnavailableException('extract $assetPath: $e');
      }
    }
    return file.path;
  }

  /// Returns the imported file path if it still exists, else fails so the caller
  /// falls back to the default (the file may have been deleted out-of-band).
  Future<String> _resolveUserFile(String path) async {
    if (await File(path).exists()) return path;
    throw SoundFontUnavailableException('imported file missing: $path');
  }

  /// Fetches a download piano from the backend delivery route
  /// `GET /soundfonts/{id}` (with the app's access token) on first use, caches
  /// it, and returns the cached path. Any failure throws so the caller falls
  /// back to the bundled default — a download is never on the critical path.
  Future<String> _fetchAndCache(String fontId) async {
    final dir = await _ref.read(soundFontStorageDirProvider.future);
    final file = File('${dir.path}/$fontId.sf2');
    if (await file.exists()) return file.path;

    final ep = _ref.read(cymbraEndpointProvider);
    final uri = Uri.parse('${soundFontDeliveryOrigin(ep)}/soundfonts/$fontId');
    try {
      final tokens = await _ref.read(tokenStoreProvider).readTokens();
      final token = tokens?.accessToken;
      // Deliberately NO wall-clock timeout (change:
      // add-client-transport-deadlines, design D5): a full SoundFont is a
      // multi-hundred-MiB transfer whose legitimate duration depends on the
      // user's bandwidth — any cap safe on a slow link is useless, any useful
      // cap truncates real downloads. Bounded at connection establishment.
      final resp = await _client.get(
        uri,
        headers: {
          if (token != null && token.isNotEmpty)
            'authorization': 'Bearer $token',
        },
      );
      if (resp.statusCode != 200 || !isValidSoundFont(resp.bodyBytes)) {
        throw SoundFontUnavailableException(
          'download $fontId: HTTP ${resp.statusCode}',
        );
      }
      await file.writeAsBytes(resp.bodyBytes, flush: true);
      return file.path;
    } on SoundFontUnavailableException {
      rethrow;
    } catch (e) {
      throw SoundFontUnavailableException('download $fontId: $e');
    }
  }
}

/// Base origin of the SoundFont delivery route (`GET /soundfonts/{id}`).
///
/// The delivery route is served by the backend's HTTP server, which — unlike a
/// deployed setup — does **not** share a port/origin with gRPC in local dev:
///  - **prod**: Caddy fronts everything on the standard TLS port, so the delivery
///    route lives at `https://<gRPC host>` (443, no explicit port).
///  - **dev**: gRPC is on `:50051` but the HTTP server is on `:8081`, a different
///    port — so deriving the origin from the gRPC endpoint alone (which drops the
///    port) would hit `http://<host>` on port 80 and fail, silently reverting the
///    piano selection to the bundled default.
///
/// Resolution: an explicit `--dart-define=CYMBRA_SOUNDFONT_ORIGIN=<origin>` wins;
/// otherwise a secure endpoint derives `https://<host>` (prod/Caddy) and a
/// plaintext one derives `http://<host>:8081` (the dev HTTP port).
String soundFontDeliveryOrigin(CymbraEndpoint ep) {
  const override = String.fromEnvironment('CYMBRA_SOUNDFONT_ORIGIN');
  if (override.isNotEmpty) return override;
  if (ep.secure) return 'https://${ep.host}';
  return 'http://${ep.host}:8081';
}

/// Production source provider. Override in tests with a fake returning a fixed
/// path (or throwing [SoundFontUnavailableException]).
@riverpod
SoundFontSource soundFontSource(Ref ref) => SoundFontSourceImpl(ref);
