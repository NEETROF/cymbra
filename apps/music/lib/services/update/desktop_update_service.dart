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

import 'dart:async';
import 'dart:convert';
import 'dart:io' show Directory, File, IOSink;
import 'dart:math';

import 'package:crypto/crypto.dart' as crypto;
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:package_info_plus/package_info_plus.dart';
import 'package:path_provider/path_provider.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../app_platform.dart';
import '../grpc_client.dart';
import '../soundfont_source.dart' show soundFontDeliveryOrigin;
import 'app_version.dart';
import 'update_manifest.dart';
import 'update_signing_keys.dart';

part 'desktop_update_service.g.dart';

/// Why an update check or download did not produce something installable.
///
/// Never rendered raw: the UI maps these to localized strings, and the cause is
/// logged. A failed check is otherwise a silent no-op — an update the user never
/// asked about must not interrupt them with an error.
enum UpdateFailureCause {
  /// The feed could not be reached, or answered with an unexpected status.
  network,

  /// The response was not a verifiable envelope. Deliberately one cause: telling
  /// a user *which* verification step failed helps an attacker, not them.
  verification,

  /// The download did not match the manifest — wrong size or wrong hash.
  integrity,

  /// Writing to disk failed (no space, no permission).
  storage,
}

/// The result of one update check.
sealed class UpdateCheckOutcome {
  const UpdateCheckOutcome();
}

/// Nothing to install: the feed offers nothing, offers this version or older, or
/// offers nothing for this OS/arch.
class UpdateUpToDate extends UpdateCheckOutcome {
  const UpdateUpToDate();
}

/// A strictly newer, verified release with an artifact for this platform.
class UpdateAvailable extends UpdateCheckOutcome {
  const UpdateAvailable({
    required this.manifest,
    required this.target,
    required this.rolloutPercent,
  });

  final UpdateManifest manifest;
  final UpdateTarget target;
  final int rolloutPercent;

  /// Whether the running build is below the manifest's floor and must be forced
  /// to update.
  bool forcesUpdate(AppVersion current) {
    final min = manifest.minSupportedVersion;
    return min != null && current < min;
  }
}

class UpdateCheckFailed extends UpdateCheckOutcome {
  const UpdateCheckFailed(this.cause);
  final UpdateFailureCause cause;
}

/// The result of one download.
sealed class UpdateDownloadOutcome {
  const UpdateDownloadOutcome();
}

class UpdateDownloaded extends UpdateDownloadOutcome {
  const UpdateDownloaded(this.file);
  final File file;
}

class UpdateDownloadFailed extends UpdateDownloadOutcome {
  const UpdateDownloadFailed(this.cause);
  final UpdateFailureCause cause;
}

/// Fetch, verify and download desktop updates (change: add-desktop-auto-update,
/// design D6). An abstract seam so state and widgets are testable without a
/// network or a filesystem.
abstract class DesktopUpdateService {
  /// Checks the feed and decides whether [current] has something newer to
  /// install on this platform. Verification happens **before** anything is
  /// written to disk.
  Future<UpdateCheckOutcome> check(AppVersion current);

  /// Downloads [target] into a fresh private directory, enforcing the declared
  /// size as a hard cap while streaming and the declared SHA-256 at the end.
  /// A mismatch deletes the file and refuses — nothing is ever handed to the
  /// installer that did not match the signed manifest.
  Future<UpdateDownloadOutcome> download(
    UpdateTarget target, {
    void Function(int received, int total)? onProgress,
  });
}

/// The `<os>-<arch>` key used in the manifest's `targets`. x64 only today; the
/// manifest is keyed so adding arm64 is additive.
String? updateTargetKey(AppPlatform platform) => switch (platform) {
  AppPlatform.windows => 'windows-x64',
  AppPlatform.linux => 'linux-x64',
  // Store-managed platforms never consult the feed.
  AppPlatform.ios ||
  AppPlatform.android ||
  AppPlatform.macos ||
  AppPlatform.web => null,
};

/// Production [DesktopUpdateService] over the backend's public update feed.
class HttpDesktopUpdateService implements DesktopUpdateService {
  HttpDesktopUpdateService(
    this._ref, {
    http.Client? client,
    Map<String, String>? trustedKeys,
    Future<Directory> Function()? downloadRoot,
    Random? random,
  }) : _client = client ?? http.Client(),
       _trustedKeys = trustedKeys ?? kUpdateTrustedKeys,
       _downloadRoot = downloadRoot ?? _defaultDownloadRoot,
       _random = random ?? Random.secure();

  final Ref _ref;
  final http.Client _client;
  final Map<String, String> _trustedKeys;
  final Future<Directory> Function() _downloadRoot;
  final Random _random;

  /// A generous ceiling on the feed document itself, so a hostile or broken
  /// origin cannot stream an unbounded body into memory before verification.
  static const int _maxEnvelopeBytes = 256 * 1024;

  /// The check is short: it must never hold up a launch.
  static const Duration _checkTimeout = Duration(seconds: 10);

  static Future<Directory> _defaultDownloadRoot() async {
    // The app's PRIVATE support directory, not a shared `/tmp` path: a
    // world-writable staging area invites a swap between the hash check and the
    // execute.
    final base = await getApplicationSupportDirectory();
    return Directory('${base.path}/updates');
  }

  @override
  Future<UpdateCheckOutcome> check(AppVersion current) async {
    final key = updateTargetKey(_ref.read(appPlatformProvider));
    if (key == null) return const UpdateUpToDate();

    final origin = soundFontDeliveryOrigin(_ref.read(cymbraEndpointProvider));
    final uri = Uri.parse(
      '$origin/updates/desktop?product=music&channel=stable',
    );
    final http.Response resp;
    try {
      resp = await _client.get(uri).timeout(_checkTimeout);
    } on Object catch (e) {
      debugPrint('desktop update: feed unreachable ($e)');
      return const UpdateCheckFailed(UpdateFailureCause.network);
    }
    // 204 is the normal "nothing to offer" answer, and also what an
    // unconfigured deployment returns.
    if (resp.statusCode == 204) return const UpdateUpToDate();
    if (resp.statusCode != 200) {
      debugPrint('desktop update: feed answered HTTP ${resp.statusCode}');
      return const UpdateCheckFailed(UpdateFailureCause.network);
    }
    if (resp.bodyBytes.length > _maxEnvelopeBytes) {
      debugPrint('desktop update: feed body is implausibly large');
      return const UpdateCheckFailed(UpdateFailureCause.verification);
    }
    // A route that fell through to tonic answers 200 with an empty body and a
    // `grpc-status` header — the exact shape a missing Caddy allow-list entry
    // produces. Name it in the log, or the feature fails invisibly in prod.
    if (resp.headers.containsKey('grpc-status')) {
      debugPrint(
        'desktop update: the feed answered a gRPC status, not a manifest — '
        '/updates/* is probably missing from the edge path allow-list',
      );
      return const UpdateCheckFailed(UpdateFailureCause.network);
    }

    final verified = await verifyUpdateEnvelope(
      utf8.decode(resp.bodyBytes, allowMalformed: true),
      _trustedKeys,
    );
    switch (verified) {
      case UpdateRefused(:final failure):
        debugPrint('desktop update: manifest refused ($failure)');
        return const UpdateCheckFailed(UpdateFailureCause.verification);
      case UpdateVerified(:final manifest, :final rolloutPercent):
        // Strictly newer, always. Without this a replayed old manifest could
        // walk a user back onto a version with a known hole — and because there
        // is no downgrade path, rollback is "pause and ship higher", by design.
        if (manifest.version <= current) return const UpdateUpToDate();
        final target = manifest.targetFor(key);
        if (target == null) return const UpdateUpToDate();
        return UpdateAvailable(
          manifest: manifest,
          target: target,
          rolloutPercent: rolloutPercent,
        );
    }
  }

  @override
  Future<UpdateDownloadOutcome> download(
    UpdateTarget target, {
    void Function(int received, int total)? onProgress,
  }) async {
    Directory? dir;
    File? file;
    IOSink? sink;
    try {
      final root = await _downloadRoot();
      // A fresh random-named directory per attempt: nothing is ever appended to
      // or resumed from a previous, possibly tampered, partial download.
      dir = Directory('${root.path}/${_randomName()}');
      await dir.create(recursive: true);
      file = File('${dir.path}/${_fileNameFor(target)}');
      sink = file.openWrite();

      final request = http.Request('GET', Uri.parse(target.url));
      final response = await _client.send(request);
      if (response.statusCode != 200) {
        debugPrint('desktop update: artifact HTTP ${response.statusCode}');
        await _cleanUp(sink, dir);
        return const UpdateDownloadFailed(UpdateFailureCause.network);
      }

      var received = 0;
      // Hash incrementally so a multi-hundred-MB artifact is never held in
      // memory, and cap on the declared size WHILE streaming — a hash check
      // alone still lets a hostile host fill the disk first.
      final accumulator = _DigestAccumulator();
      final sha = crypto.sha256.startChunkedConversion(accumulator);

      await for (final chunk in response.stream) {
        received += chunk.length;
        if (received > target.size) {
          debugPrint('desktop update: artifact exceeds its declared size');
          sha.close();
          await _cleanUp(sink, dir);
          return const UpdateDownloadFailed(UpdateFailureCause.integrity);
        }
        sha.add(chunk);
        sink.add(chunk);
        onProgress?.call(received, target.size);
      }
      sha.close();
      await sink.flush();
      await sink.close();
      sink = null;

      if (received != target.size) {
        debugPrint('desktop update: artifact is short of its declared size');
        await _cleanUp(null, dir);
        return const UpdateDownloadFailed(UpdateFailureCause.integrity);
      }
      final actual = accumulator.value?.toString();
      if (actual == null ||
          actual.toLowerCase() != target.sha256.toLowerCase()) {
        // Delete before returning: a file that failed its hash must not be left
        // where a later run could mistake it for a good download.
        debugPrint('desktop update: artifact checksum mismatch');
        await _cleanUp(null, dir);
        return const UpdateDownloadFailed(UpdateFailureCause.integrity);
      }
      return UpdateDownloaded(file);
    } on Object catch (e) {
      debugPrint('desktop update: download failed ($e)');
      await _cleanUp(sink, dir);
      return const UpdateDownloadFailed(UpdateFailureCause.storage);
    }
  }

  String _randomName() {
    const alphabet = 'abcdefghijklmnopqrstuvwxyz0123456789';
    return List.generate(
      16,
      (_) => alphabet[_random.nextInt(alphabet.length)],
    ).join();
  }

  /// The artifact keeps a name the platform installer understands: Windows needs
  /// a `.exe` to spawn, and an AppImage must stay executable-shaped.
  static String _fileNameFor(UpdateTarget target) {
    final last = Uri.parse(target.url).pathSegments.lastOrNull ?? '';
    if (last.isNotEmpty && !last.contains('/') && !last.contains('\\')) {
      return last;
    }
    return switch (target.kind) {
      'inno-setup' => 'cymbra-setup.exe',
      'appimage' => 'Cymbra.AppImage',
      _ => 'cymbra-update.bin',
    };
  }

  Future<void> _cleanUp(IOSink? sink, Directory? dir) async {
    try {
      await sink?.close();
    } on Object {
      // Already closed or already failed; the directory delete is what matters.
    }
    try {
      if (dir != null && await dir.exists()) {
        await dir.delete(recursive: true);
      }
    } on Object catch (e) {
      debugPrint('desktop update: could not clean up $dir ($e)');
    }
  }
}

/// Captures the final digest of a chunked SHA-256 conversion.
class _DigestAccumulator implements Sink<crypto.Digest> {
  crypto.Digest? value;

  @override
  void add(crypto.Digest data) => value = data;

  @override
  void close() {}
}

/// The running build's version, as `major.minor.patch+build`.
///
/// Read from `package_info_plus` (the platform's own metadata, stamped from
/// `pubspec.yaml` at build time) rather than from a constant, so a build can
/// never disagree with itself about what it is. `null` when the string is not in
/// the app's version format — the updater then offers nothing rather than
/// guessing. Behind a provider so tests set a version without the plugin.
@Riverpod(keepAlive: true)
Future<AppVersion?> currentAppVersion(Ref ref) async {
  final info = await PackageInfo.fromPlatform();
  final parsed = AppVersion.tryParse('${info.version}+${info.buildNumber}');
  if (parsed == null) {
    debugPrint(
      'desktop update: unrecognised app version '
      '"${info.version}+${info.buildNumber}" — updates disabled',
    );
  }
  return parsed;
}

/// Production service provider. Override in tests with a generated mock.
@Riverpod(keepAlive: true)
DesktopUpdateService desktopUpdateService(Ref ref) =>
    HttpDesktopUpdateService(ref);
