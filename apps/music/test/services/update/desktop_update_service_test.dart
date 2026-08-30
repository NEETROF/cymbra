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
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/update/app_version.dart';
import 'package:music/services/update/desktop_update_service.dart';
import 'package:music/services/update/update_manifest.dart';

const _fixtures = '../../crates/update-manifest/fixtures';

String _fixture(String name) => File('$_fixtures/$name').readAsStringSync();

Map<String, String> _trustedKeys() =>
    (jsonDecode(_fixture('trusted_keys.json')) as Map<String, dynamic>).map(
      (k, v) => MapEntry(k, v as String),
    );

AppVersion v(String raw) => AppVersion.tryParse(raw)!;

void main() {
  late Directory tempRoot;

  setUp(() {
    tempRoot = Directory.systemTemp.createTempSync('cymbra-update-test');
  });
  tearDown(() {
    if (tempRoot.existsSync()) tempRoot.deleteSync(recursive: true);
  });

  ProviderContainer container({AppPlatform platform = AppPlatform.windows}) =>
      ProviderContainer(
        overrides: [
          appPlatformProvider.overrideWithValue(platform),
          cymbraEndpointProvider.overrideWithValue(
            const CymbraEndpoint(host: 'api.test', port: 443, secure: true),
          ),
        ],
      );

  HttpDesktopUpdateService service(
    ProviderContainer c,
    http.Client client, {
    Map<String, String>? keys,
  }) => HttpDesktopUpdateService(
    // `Ref` is what the service reads; the container's own is the honest one.
    c.read(_refProbeProvider),
    client: client,
    trustedKeys: keys ?? _trustedKeys(),
    downloadRoot: () async => tempRoot,
  );

  group('check', () {
    test('offers a strictly newer verified release', () async {
      final c = container();
      addTearDown(c.dispose);
      final s = service(
        c,
        MockClient(
          (_) async => http.Response(_fixture('envelope_valid.json'), 200),
        ),
      );
      final outcome = await s.check(v('1.24.0+32'));
      expect(outcome, isA<UpdateAvailable>());
      final available = outcome as UpdateAvailable;
      expect(available.manifest.version.toString(), '1.25.0+34');
      expect(available.target.kind, 'inno-setup');
      expect(available.rolloutPercent, 25);
    });

    test('never downgrades: an equal or older version is up-to-date', () async {
      final c = container();
      addTearDown(c.dispose);
      final s = service(
        c,
        MockClient(
          (_) async => http.Response(_fixture('envelope_valid.json'), 200),
        ),
      );
      // A replayed old manifest must not walk a user back onto a version with a
      // known hole — which is also why rollback is "pause and ship higher".
      expect(await s.check(v('1.25.0+34')), isA<UpdateUpToDate>());
      expect(await s.check(v('1.26.0+40')), isA<UpdateUpToDate>());
    });

    test(
      '204 (nothing servable, or an unconfigured feed) is up-to-date',
      () async {
        final c = container();
        addTearDown(c.dispose);
        final s = service(c, MockClient((_) async => http.Response('', 204)));
        expect(await s.check(v('1.0.0+1')), isA<UpdateUpToDate>());
      },
    );

    test(
      'a release with no artifact for this platform offers nothing',
      () async {
        // macOS is store-managed: the feed is never consulted at all.
        final c = container(platform: AppPlatform.macos);
        addTearDown(c.dispose);
        var called = false;
        final s = service(
          c,
          MockClient((_) async {
            called = true;
            return http.Response(_fixture('envelope_valid.json'), 200);
          }),
        );
        expect(await s.check(v('1.0.0+1')), isA<UpdateUpToDate>());
        expect(called, isFalse);
      },
    );

    test('a target key the manifest does not carry offers nothing', () async {
      final c = container(platform: AppPlatform.linux);
      addTearDown(c.dispose);
      final envelope =
          jsonDecode(_fixture('envelope_valid.json')) as Map<String, dynamic>;
      final manifest =
          jsonDecode(utf8.decode(base64.decode(envelope['manifest'] as String)))
              as Map<String, dynamic>;
      (manifest['targets'] as Map).remove('linux-x64');
      // Re-encoding invalidates the signature, so verification fails first —
      // which is the correct outcome for a manifest nobody signed.
      envelope['manifest'] = base64.encode(utf8.encode(jsonEncode(manifest)));
      final s = service(
        c,
        MockClient((_) async => http.Response(jsonEncode(envelope), 200)),
      );
      final outcome = await s.check(v('1.0.0+1'));
      expect(outcome, isA<UpdateCheckFailed>());
      expect(
        (outcome as UpdateCheckFailed).cause,
        UpdateFailureCause.verification,
      );
    });

    test('a tampered manifest is refused, and nothing is offered', () async {
      final c = container();
      addTearDown(c.dispose);
      final s = service(
        c,
        MockClient(
          (_) async =>
              http.Response(_fixture('envelope_tampered_manifest.json'), 200),
        ),
      );
      final outcome = await s.check(v('1.0.0+1'));
      expect(
        (outcome as UpdateCheckFailed).cause,
        UpdateFailureCause.verification,
      );
    });

    test('a foreign key is refused', () async {
      final c = container();
      addTearDown(c.dispose);
      final s = service(
        c,
        MockClient(
          (_) async => http.Response(_fixture('envelope_valid.json'), 200),
        ),
        keys: const {
          'golden-1': 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',
        },
      );
      expect(await s.check(v('1.0.0+1')), isA<UpdateCheckFailed>());
    });

    test(
      'a 200 carrying a grpc-status is a network failure, not a manifest',
      () async {
        // The exact shape of a path missing from the edge allow-list: it must not
        // be mistaken for "nothing to offer", or the feature fails invisibly.
        final c = container();
        addTearDown(c.dispose);
        final s = service(
          c,
          MockClient(
            (_) async => http.Response('', 200, headers: {'grpc-status': '12'}),
          ),
        );
        final outcome = await s.check(v('1.0.0+1'));
        expect(
          (outcome as UpdateCheckFailed).cause,
          UpdateFailureCause.network,
        );
      },
    );

    test(
      'an unexpected status and a transport error are network failures',
      () async {
        final c = container();
        addTearDown(c.dispose);
        expect(
          ((await service(
                    c,
                    MockClient((_) async => http.Response('', 500)),
                  ).check(v('1.0.0+1')))
                  as UpdateCheckFailed)
              .cause,
          UpdateFailureCause.network,
        );
        expect(
          ((await service(
                    c,
                    MockClient(
                      (_) async => throw const SocketException('down'),
                    ),
                  ).check(v('1.0.0+1')))
                  as UpdateCheckFailed)
              .cause,
          UpdateFailureCause.network,
        );
      },
    );
  });

  group('download', () {
    const bytes = 'the artifact';
    final payload = Uint8List.fromList(utf8.encode(bytes));
    // sha256 of "the artifact"
    const goodSha =
        '03473f89a784c14707308b73a324b887c10627dc400d8f62fd37a74d66b12438';

    UpdateTarget target({int? size, String? sha}) => UpdateTarget(
      kind: 'inno-setup',
      url: 'https://example.invalid/cymbra-setup.exe',
      size: size ?? payload.length,
      sha256: sha ?? goodSha,
    );

    MockClient streaming(List<int> body, {int status = 200}) =>
        MockClient.streaming(
          (_, _) async => http.StreamedResponse(
            Stream.value(body),
            status,
            contentLength: body.length,
          ),
        );

    test('writes the artifact and reports progress', () async {
      final c = container();
      addTearDown(c.dispose);
      final progress = <int>[];
      final outcome = await service(
        c,
        streaming(payload),
      ).download(target(), onProgress: (received, _) => progress.add(received));
      expect(outcome, isA<UpdateDownloaded>());
      final file = (outcome as UpdateDownloaded).file;
      expect(file.readAsStringSync(), bytes);
      // The URL's own file name is kept: Windows needs a .exe to spawn.
      expect(file.path, endsWith('cymbra-setup.exe'));
      expect(progress.last, payload.length);
    });

    test('a checksum mismatch deletes the file and refuses it', () async {
      final c = container();
      addTearDown(c.dispose);
      final outcome = await service(
        c,
        streaming(payload),
      ).download(target(sha: 'f' * 64));
      expect(
        (outcome as UpdateDownloadFailed).cause,
        UpdateFailureCause.integrity,
      );
      // Nothing may survive that failed its hash — a later run must not mistake
      // it for a good download.
      expect(tempRoot.listSync(), isEmpty);
    });

    test(
      'a body larger than the declared size is cut off and refused',
      () async {
        final c = container();
        addTearDown(c.dispose);
        final outcome = await service(
          c,
          streaming(payload),
        ).download(target(size: 4));
        expect(
          (outcome as UpdateDownloadFailed).cause,
          UpdateFailureCause.integrity,
        );
        expect(tempRoot.listSync(), isEmpty);
      },
    );

    test('a body shorter than the declared size is refused', () async {
      final c = container();
      addTearDown(c.dispose);
      final outcome = await service(
        c,
        streaming(payload),
      ).download(target(size: payload.length + 10));
      expect(
        (outcome as UpdateDownloadFailed).cause,
        UpdateFailureCause.integrity,
      );
      expect(tempRoot.listSync(), isEmpty);
    });

    test('a non-200 artifact response is a network failure', () async {
      final c = container();
      addTearDown(c.dispose);
      final outcome = await service(
        c,
        streaming(payload, status: 404),
      ).download(target());
      expect(
        (outcome as UpdateDownloadFailed).cause,
        UpdateFailureCause.network,
      );
      expect(tempRoot.listSync(), isEmpty);
    });

    test(
      'each attempt gets its own directory (no resume from a partial)',
      () async {
        final c = container();
        addTearDown(c.dispose);
        final first = await service(c, streaming(payload)).download(target());
        final second = await service(c, streaming(payload)).download(target());
        expect(
          (first as UpdateDownloaded).file.parent.path,
          isNot((second as UpdateDownloaded).file.parent.path),
        );
      },
    );
  });

  group('updateTargetKey', () {
    test('only the store-less desktop platforms consult the feed', () {
      expect(updateTargetKey(AppPlatform.windows), 'windows-x64');
      expect(updateTargetKey(AppPlatform.linux), 'linux-x64');
      for (final p in [
        AppPlatform.ios,
        AppPlatform.android,
        AppPlatform.macos,
        AppPlatform.web,
      ]) {
        expect(updateTargetKey(p), isNull);
      }
    });
  });
}

/// Hands a test the container's own `Ref`, so the service under test reads the
/// same overrides the container was built with.
final _refProbeProvider = Provider<Ref>((ref) => ref);
