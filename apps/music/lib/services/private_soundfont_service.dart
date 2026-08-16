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

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show immutable;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'grpc_client.dart';
import 'soundfont_source.dart' show soundFontDeliveryOrigin;
import 'token_store.dart';

part 'private_soundfont_service.g.dart';

/// A font in the user's **private** server-side library (change:
/// add-soundfont-moderation), as returned by `GET /me/soundfonts`. The library
/// is per-user, unmoderated, and synced across the user's devices.
@immutable
class RemoteSoundFont {
  const RemoteSoundFont({
    required this.id,
    required this.label,
    required this.sizeBytes,
    this.proposalStatus,
    this.rejectionReason,
  });

  factory RemoteSoundFont.fromJson(Map<String, dynamic> json) =>
      RemoteSoundFont(
        id: json['id'] as String,
        label: (json['label'] as String?) ?? '',
        sizeBytes: (json['sizeBytes'] as num?)?.toInt() ?? 0,
        proposalStatus: json['proposalStatus'] as String?,
        rejectionReason: json['rejectionReason'] as String?,
      );

  /// Server-assigned id (also the delivery id: `GET /me/soundfonts/{id}`).
  final String id;
  final String label;
  final int sizeBytes;

  /// The moderation status of this font's public-catalog proposal, or `null` when
  /// it has not been proposed (change: add-soundfont-moderation). One of
  /// `pending`/`accepted`/`rejected`.
  final String? proposalStatus;

  /// The moderator's motive when [proposalStatus] is `rejected` (change:
  /// add-soundfont-uploader-attribution); `null` otherwise.
  final String? rejectionReason;

  @override
  bool operator ==(Object other) =>
      other is RemoteSoundFont &&
      other.id == id &&
      other.label == label &&
      other.sizeBytes == sizeBytes &&
      other.proposalStatus == proposalStatus &&
      other.rejectionReason == rejectionReason;

  @override
  int get hashCode =>
      Object.hash(id, label, sizeBytes, proposalStatus, rejectionReason);
}

/// Thrown when a private-library request fails (network, auth, quota, or the
/// server rejecting the operation). The caller surfaces a non-fatal message.
/// [statusCode] carries the HTTP status when known (e.g. 409 = already proposed).
class PrivateSoundFontException implements Exception {
  const PrivateSoundFontException(this.message, {this.statusCode});
  final String message;
  final int? statusCode;

  @override
  String toString() => 'PrivateSoundFontException: $message';
}

/// Seam over the private per-user SoundFont library HTTP routes
/// (`/me/soundfonts`), so the registry notifier can be driven by an in-memory
/// fake in tests (no network, no token store). Mirrors [SoundFontSource].
abstract class PrivateSoundFontService {
  /// The caller's private library (owner-scoped, synced across devices).
  Future<List<RemoteSoundFont>> list();

  /// Upload a `.sf2` into the private library. Idempotent server-side by content
  /// digest, so re-uploading identical bytes returns the existing entry. Returns
  /// the created/existing remote font.
  Future<RemoteSoundFont> import(Uint8List bytes, String label);

  /// Download a private font's bytes (owner-only).
  Future<Uint8List> download(String id);

  /// Remove a private font from the library (owner-only), so it stops syncing to
  /// the user's other devices. A missing font is not an error.
  Future<void> delete(String id);

  /// Propose a private font to the public catalog. Requires an explicit licence
  /// declaration and a right-to-distribute [attestation]; the font enters the
  /// catalog as `pending`, awaiting moderator review. When re-proposing a
  /// previously **rejected** font, [resubmissionNote] carries the mandatory
  /// justification shown to the moderator (change:
  /// add-soundfont-uploader-attribution).
  Future<void> propose(
    String id, {
    required String license,
    String attribution = '',
    required bool attestation,
    String? resubmissionNote,
  });
}

/// Production [PrivateSoundFontService] over the backend HTTP routes,
/// authenticated with the app's access token (like the delivery route).
class HttpPrivateSoundFontService implements PrivateSoundFontService {
  HttpPrivateSoundFontService(this._ref, {http.Client? client})
    : _client = client ?? http.Client();

  final Ref _ref;
  final http.Client _client;

  String get _origin =>
      soundFontDeliveryOrigin(_ref.read(cymbraEndpointProvider));

  Future<Map<String, String>> _authHeaders() async {
    final tokens = await _ref.read(tokenStoreProvider).readTokens();
    final token = tokens?.accessToken;
    return {
      if (token != null && token.isNotEmpty) 'authorization': 'Bearer $token',
    };
  }

  @override
  Future<List<RemoteSoundFont>> list() async {
    final uri = Uri.parse('$_origin/me/soundfonts');
    try {
      final resp = await _client.get(uri, headers: await _authHeaders());
      if (resp.statusCode != 200) {
        throw PrivateSoundFontException('list: HTTP ${resp.statusCode}');
      }
      final data = jsonDecode(resp.body) as List<dynamic>;
      return [
        for (final item in data)
          RemoteSoundFont.fromJson(item as Map<String, dynamic>),
      ];
    } on PrivateSoundFontException {
      rethrow;
    } catch (e) {
      throw PrivateSoundFontException('list: $e');
    }
  }

  @override
  Future<RemoteSoundFont> import(Uint8List bytes, String label) async {
    final uri = Uri.parse(
      '$_origin/me/soundfonts?label=${Uri.encodeQueryComponent(label)}',
    );
    try {
      final resp = await _client.post(
        uri,
        headers: await _authHeaders(),
        body: bytes,
      );
      // 201 created, or 200 when identical content already existed (idempotent).
      if (resp.statusCode != 201 && resp.statusCode != 200) {
        throw PrivateSoundFontException(
          'import: HTTP ${resp.statusCode}',
          statusCode: resp.statusCode,
        );
      }
      return RemoteSoundFont.fromJson(
        jsonDecode(resp.body) as Map<String, dynamic>,
      );
    } on PrivateSoundFontException {
      rethrow;
    } catch (e) {
      throw PrivateSoundFontException('import: $e');
    }
  }

  @override
  Future<Uint8List> download(String id) async {
    final uri = Uri.parse('$_origin/me/soundfonts/${Uri.encodeComponent(id)}');
    try {
      final resp = await _client.get(uri, headers: await _authHeaders());
      if (resp.statusCode != 200) {
        throw PrivateSoundFontException('download: HTTP ${resp.statusCode}');
      }
      return resp.bodyBytes;
    } on PrivateSoundFontException {
      rethrow;
    } catch (e) {
      throw PrivateSoundFontException('download: $e');
    }
  }

  @override
  Future<void> delete(String id) async {
    final uri = Uri.parse('$_origin/me/soundfonts/${Uri.encodeComponent(id)}');
    try {
      final resp = await _client.delete(uri, headers: await _authHeaders());
      // 204/200 on success; 404 (already gone) is fine too.
      if (resp.statusCode != 200 &&
          resp.statusCode != 204 &&
          resp.statusCode != 404) {
        throw PrivateSoundFontException('delete: HTTP ${resp.statusCode}');
      }
    } on PrivateSoundFontException {
      rethrow;
    } catch (e) {
      throw PrivateSoundFontException('delete: $e');
    }
  }

  @override
  Future<void> propose(
    String id, {
    required String license,
    String attribution = '',
    required bool attestation,
    String? resubmissionNote,
  }) async {
    final query = <String, String>{
      'license': license,
      'attribution': attribution,
      'attestation': attestation.toString(),
      if (resubmissionNote != null && resubmissionNote.trim().isNotEmpty)
        'resubmission_note': resubmissionNote.trim(),
    };
    final uri = Uri.parse(
      '$_origin/me/soundfonts/${Uri.encodeComponent(id)}/propose',
    ).replace(queryParameters: query);
    try {
      final resp = await _client.post(uri, headers: await _authHeaders());
      if (resp.statusCode != 201) {
        // 409 = the font is already in the catalog (already proposed, or identical
        // content) — surfaced distinctly by the caller.
        throw PrivateSoundFontException(
          'propose: HTTP ${resp.statusCode}',
          statusCode: resp.statusCode,
        );
      }
    } on PrivateSoundFontException {
      rethrow;
    } catch (e) {
      throw PrivateSoundFontException('propose: $e');
    }
  }
}

/// Production provider. Override in tests with an in-memory fake.
@riverpod
PrivateSoundFontService privateSoundFontService(Ref ref) =>
    HttpPrivateSoundFontService(ref);
