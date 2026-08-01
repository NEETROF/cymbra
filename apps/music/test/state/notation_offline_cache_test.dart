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
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/notation_fakes.dart';

/// Minimal [ScoreUploadService] — only `fetchBytes` matters here.
class _FakeUpload extends Fake implements ScoreUploadService {
  _FakeUpload(this.bytes, {this.error});
  final Uint8List bytes;
  final Object? error;
  int fetchCalls = 0;

  @override
  Future<Uint8List> fetchBytes(String id) async {
    fetchCalls++;
    if (error != null) throw error!;
    return bytes;
  }
}

class _FakeConnectivity extends Fake implements ConnectivityService {
  _FakeConnectivity(this.online);
  final bool online;
  @override
  Stream<void> get onOnline => const Stream.empty();
  @override
  Future<bool> isOnline() async => online;
}

Future<void> _flush() async {
  for (var i = 0; i < 6; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

CatalogEntry _upload({bool favorite = true}) => CatalogEntry(
  id: 'contrib-1',
  title: 'My Upload',
  composer: 'Me',
  level: PracticeLevel.beginner,
  contributedId: '1',
  favorite: favorite,
);

void main() {
  Uint8List bytes() => Uint8List.fromList(const [1, 2, 3, 4]);

  ProviderContainer build({
    required InMemoryOfflineScoreCache cache,
    required _FakeUpload upload,
    bool online = true,
  }) {
    final c = ProviderContainer(
      overrides: [
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
        offlineScoreCacheProvider.overrideWithValue(cache),
        scoreUploadServiceProvider.overrideWithValue(upload),
        connectivityServiceProvider.overrideWithValue(
          _FakeConnectivity(online),
        ),
      ],
    );
    addTearDown(c.dispose);
    c.listen(notationProvider, (_, _) {}, fireImmediately: true);
    return c;
  }

  test('a cache hit plays offline without any network fetch', () async {
    final cache = InMemoryOfflineScoreCache();
    await cache.write('contributed:1', bytes(), etag: 'e');
    final upload = _FakeUpload(bytes());
    final c = build(cache: cache, upload: upload);

    c.read(selectedScoreProvider.notifier).select(_upload());
    await _flush();

    expect(upload.fetchCalls, 0, reason: 'served from the encrypted cache');
    expect(c.read(notationProvider).hasDocument, isTrue);
  });

  test('a cache miss fetches and writes the encrypted copy (favorite)', () async {
    final cache = InMemoryOfflineScoreCache();
    final upload = _FakeUpload(bytes());
    final c = build(cache: cache, upload: upload);

    c.read(selectedScoreProvider.notifier).select(_upload());
    await _flush();

    expect(upload.fetchCalls, 1);
    expect(await cache.has('contributed:1'), isTrue, reason: 'cached on open');
    expect(c.read(notationProvider).hasDocument, isTrue);
  });

  test('a non-favorite upload is fetched but never cached', () async {
    final cache = InMemoryOfflineScoreCache();
    final upload = _FakeUpload(bytes());
    final c = build(cache: cache, upload: upload);

    c.read(selectedScoreProvider.notifier).select(_upload(favorite: false));
    await _flush();

    expect(upload.fetchCalls, 1);
    expect(await cache.has('contributed:1'), isFalse);
  });

  test('offline + uncached → dedicated offlineUnavailable failure', () async {
    final cache = InMemoryOfflineScoreCache();
    final upload = _FakeUpload(
      bytes(),
      error: AuthException(AuthError.unavailable),
    );
    final c = build(cache: cache, upload: upload, online: false);

    c.read(selectedScoreProvider.notifier).select(_upload());
    await _flush();

    expect(c.read(notationProvider).failure, ScoreLoadFailure.offlineUnavailable);
  });

  test('online-but-unavailable backend → generic unavailable failure', () async {
    final cache = InMemoryOfflineScoreCache();
    final upload = _FakeUpload(
      bytes(),
      error: AuthException(AuthError.unavailable),
    );
    final c = build(cache: cache, upload: upload, online: true);

    c.read(selectedScoreProvider.notifier).select(_upload());
    await _flush();

    expect(c.read(notationProvider).failure, ScoreLoadFailure.unavailable);
  });
}
