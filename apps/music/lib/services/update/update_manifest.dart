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

/// The client half of the signed update-manifest contract (change:
/// add-desktop-auto-update, design D2/D7). Pure: no I/O, no platform channels.
///
/// This is a **second, independent implementation** of what
/// `crates/update-manifest` does in Rust — which is exactly where cross-language
/// drift hides. Both sides run against the same checked-in golden fixture
/// (`crates/update-manifest/fixtures/`), so an Ed25519, base64 or schema
/// disagreement fails a unit test instead of an install.
library;

import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

import 'app_version.dart';

/// The only manifest schema this build understands. A higher value is refused
/// rather than guessed at.
const int kUpdateSchemaVersion = 1;

/// One downloadable artifact, keyed by `<os>-<arch>`.
class UpdateTarget {
  const UpdateTarget({
    required this.kind,
    required this.url,
    required this.size,
    required this.sha256,
  });

  /// The install method implied: `inno-setup`, `appimage`.
  final String kind;
  final String url;

  /// Exact byte length. Enforced as a hard cap **while streaming**, before the
  /// hash can be computed — a hash check alone would still let an attacker who
  /// controls the host fill the disk.
  final int size;

  /// Lowercase hex SHA-256 of the artifact bytes.
  final String sha256;

  static UpdateTarget? _tryFrom(Object? raw) {
    if (raw is! Map) return null;
    final kind = raw['kind'];
    final url = raw['url'];
    final size = raw['size'];
    final sha256 = raw['sha256'];
    if (kind is! String ||
        url is! String ||
        size is! int ||
        sha256 is! String) {
      return null;
    }
    if (size <= 0) return null;
    return UpdateTarget(kind: kind, url: url, size: size, sha256: sha256);
  }
}

/// The release description that CI signed.
class UpdateManifest {
  const UpdateManifest({
    required this.schema,
    required this.product,
    required this.channel,
    required this.version,
    required this.releasedAt,
    required this.minSupportedVersion,
    required this.notesUrl,
    required this.targets,
  });

  final int schema;
  final String product;
  final String channel;
  final AppVersion version;
  final String releasedAt;

  /// Below this, the client cannot talk to the backend and must be forced to
  /// update. `null` means no floor.
  final AppVersion? minSupportedVersion;
  final String? notesUrl;

  /// Artifacts by `<os>-<arch>` (`windows-x64`, `linux-x64`).
  final Map<String, UpdateTarget> targets;

  UpdateTarget? targetFor(String osArch) => targets[osArch];
}

/// Why an envelope was refused. Every value means **nothing was downloaded**.
enum UpdateVerifyFailure {
  /// The response was not an envelope, or its base64 does not decode.
  malformed,

  /// The signature field is not a 64-byte Ed25519 signature.
  malformedSignature,

  /// No compiled-in key carries this `key_id`.
  unknownKeyId,

  /// The signature does not cover these bytes with that key.
  badSignature,

  /// The verified bytes are not a manifest.
  malformedManifest,

  /// A schema this build does not implement.
  unsupportedSchema,
}

/// Verifying an envelope either yields a manifest or a reason it was refused.
sealed class UpdateVerifyResult {
  const UpdateVerifyResult();
}

class UpdateVerified extends UpdateVerifyResult {
  const UpdateVerified(this.manifest, this.rolloutPercent);
  final UpdateManifest manifest;

  /// Staged rollout, evaluated locally. Outside the signature by design: it is
  /// backend policy, not release identity, and tampering with it can only offer
  /// a legitimately signed update sooner.
  final int rolloutPercent;
}

class UpdateRefused extends UpdateVerifyResult {
  const UpdateRefused(this.failure);
  final UpdateVerifyFailure failure;
}

/// Verifies a feed response body against [trustedKeys] (`key_id` → base64
/// public key).
///
/// The order is the security property, and it is fixed: decode → resolve the key
/// → verify the signature over the decoded bytes → **only then** parse → guard
/// the schema. Nothing downstream ever sees bytes that failed a step, and the
/// bytes that were verified are by construction the bytes that get parsed —
/// there is no re-serialization in between, so no JSON canonicalization rule for
/// Rust and Dart to disagree about.
Future<UpdateVerifyResult> verifyUpdateEnvelope(
  String body,
  Map<String, String> trustedKeys,
) async {
  final Object? decoded;
  try {
    decoded = jsonDecode(body);
  } on FormatException {
    return const UpdateRefused(UpdateVerifyFailure.malformed);
  }
  if (decoded is! Map) {
    return const UpdateRefused(UpdateVerifyFailure.malformed);
  }

  final manifestB64 = decoded['manifest'];
  final signatureB64 = decoded['signature'];
  final keyId = decoded['key_id'];
  final rollout = decoded['rollout_percent'];
  if (manifestB64 is! String || signatureB64 is! String || keyId is! String) {
    return const UpdateRefused(UpdateVerifyFailure.malformed);
  }

  final Uint8List manifestBytes;
  final Uint8List signatureBytes;
  try {
    manifestBytes = base64.decode(manifestB64);
    signatureBytes = base64.decode(signatureB64);
  } on FormatException {
    return const UpdateRefused(UpdateVerifyFailure.malformed);
  }
  if (signatureBytes.length != 64) {
    return const UpdateRefused(UpdateVerifyFailure.malformedSignature);
  }

  final publicKeyB64 = trustedKeys[keyId];
  if (publicKeyB64 == null) {
    return const UpdateRefused(UpdateVerifyFailure.unknownKeyId);
  }
  final Uint8List publicKeyBytes;
  try {
    publicKeyBytes = base64.decode(publicKeyB64);
  } on FormatException {
    return const UpdateRefused(UpdateVerifyFailure.malformed);
  }
  if (publicKeyBytes.length != 32) {
    return const UpdateRefused(UpdateVerifyFailure.malformed);
  }

  final ok = await Ed25519().verify(
    manifestBytes,
    signature: Signature(
      signatureBytes,
      publicKey: SimplePublicKey(publicKeyBytes, type: KeyPairType.ed25519),
    ),
  );
  if (!ok) return const UpdateRefused(UpdateVerifyFailure.badSignature);

  final manifest = _parseManifest(manifestBytes);
  if (manifest == null) {
    return const UpdateRefused(UpdateVerifyFailure.malformedManifest);
  }
  if (manifest.schema != kUpdateSchemaVersion) {
    return const UpdateRefused(UpdateVerifyFailure.unsupportedSchema);
  }
  // A missing or out-of-range rollout reads as 0: unknown policy offers nothing,
  // rather than defaulting to "everyone".
  final percent = rollout is int ? rollout.clamp(0, 100) : 0;
  return UpdateVerified(manifest, percent);
}

UpdateManifest? _parseManifest(Uint8List bytes) {
  final Object? raw;
  try {
    raw = jsonDecode(utf8.decode(bytes));
  } on FormatException {
    return null;
  }
  if (raw is! Map) return null;

  final schema = raw['schema'];
  final product = raw['product'];
  final channel = raw['channel'];
  final version = raw['version'];
  final releasedAt = raw['released_at'];
  final targetsRaw = raw['targets'];
  if (schema is! int ||
      product is! String ||
      channel is! String ||
      version is! String ||
      releasedAt is! String ||
      targetsRaw is! Map) {
    return null;
  }
  final parsedVersion = AppVersion.tryParse(version);
  if (parsedVersion == null) return null;

  final targets = <String, UpdateTarget>{};
  for (final entry in targetsRaw.entries) {
    final key = entry.key;
    final target = UpdateTarget._tryFrom(entry.value);
    // One unreadable target must not discard the rest: a future `macos-arm64`
    // entry this build does not understand should not stop a Windows client
    // from updating.
    if (key is String && target != null) targets[key] = target;
  }
  if (targets.isEmpty) return null;

  final minRaw = raw['min_supported_version'];
  final notesRaw = raw['notes_url'];
  return UpdateManifest(
    schema: schema,
    product: product,
    channel: channel,
    version: parsedVersion,
    releasedAt: releasedAt,
    minSupportedVersion: minRaw is String ? AppVersion.tryParse(minRaw) : null,
    notesUrl: notesRaw is String ? notesRaw : null,
    targets: targets,
  );
}
