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
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/notation_fakes.dart';

/// Upload seam whose fetch either throws [error], never completes
/// ([hang] = true, completable later via [pending]), or returns [bytes].
class _FakeUpload extends Fake implements ScoreUploadService {
  _FakeUpload(this.bytes, {this.error, this.hang = false});
  final Uint8List bytes;
  final Object? error;
  final bool hang;
  final pending = Completer<ScoreBytesResult>();
  int fetchCalls = 0;

  @override
  Future<ScoreBytesResult> fetchScoreBytes(String id, {String? ifNoneMatch}) {
    fetchCalls++;
    if (hang) return pending.future;
    if (error != null) throw error!;
    return Future.value(ScoreBytesResult(data: bytes, etag: 'e'));
  }
}

/// Strict seam: any network call fails the test (the pre-flight gate must
/// prevent it from ever being reached).
class _NeverCalledUpload extends Fake implements ScoreUploadService {
  @override
  Future<ScoreBytesResult> fetchScoreBytes(String id, {String? ifNoneMatch}) {
    throw StateError('offline pre-flight must prevent this call');
  }
}

/// Controllable connectivity: point-in-time reading + a broadcast transition
/// stream the test drives.
class _Conn extends Fake implements ConnectivityService {
  _Conn(this.online);
  bool online;
  final ctrl = StreamController<bool>.broadcast();

  @override
  Stream<bool> get onlineStatus => ctrl.stream;
  @override
  Future<bool> isOnline() async => online;
  @override
  Future<bool> isDefinitelyOffline() async => !online;
}

Future<void> _flush() async {
  for (var i = 0; i < 6; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

CatalogEntry _upload() => const CatalogEntry(
  id: 'contrib-1',
  title: 'My Upload',
  composer: 'Me',
  level: PracticeLevel.beginner,
  contributedId: '1',
  favorite: true,
);

ProviderContainer _build({
  required ScoreUploadService upload,
  required _Conn conn,
  InMemoryOfflineScoreCache? cache,
}) {
  final c = ProviderContainer(
    overrides: [
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      offlineScoreCacheProvider.overrideWithValue(
        cache ?? InMemoryOfflineScoreCache(),
      ),
      scoreUploadServiceProvider.overrideWithValue(upload),
      connectivityServiceProvider.overrideWithValue(conn),
    ],
  );
  addTearDown(c.dispose);
  c.listen(notationProvider, (_, _) {}, fireImmediately: true);
  return c;
}

void main() {
  Uint8List bytes() => Uint8List.fromList(const [1, 2, 3, 4]);

  group('DEADLINE_EXCEEDED classification (the offline-cache coupling)', () {
    // The regression this change must not introduce: a deadline overrun MUST
    // take the same path as UNAVAILABLE, or adding deadlines converts a slow
    // failure into a fast one that skips the cache fallback.
    Object deadlineExceeded() =>
        authExceptionFromGrpc(GrpcError.deadlineExceeded('deadline'));

    test('offline + uncached → the dedicated offline message', () async {
      // The pre-flight gate short-circuits before the fetch; the outcome is
      // identical to what an in-flight timeout would have produced.
      final c = _build(
        upload: _FakeUpload(bytes(), error: deadlineExceeded()),
        conn: _Conn(false),
      );
      c.read(selectedScoreProvider.notifier).select(_upload());
      await _flush();
      expect(
        c.read(notationProvider).failure,
        ScoreLoadFailure.offlineUnavailable,
      );
    });

    test('online, backend times out → unavailable, not generic', () async {
      final c = _build(
        upload: _FakeUpload(bytes(), error: deadlineExceeded()),
        conn: _Conn(true),
      );
      c.read(selectedScoreProvider.notifier).select(_upload());
      await _flush();
      expect(c.read(notationProvider).failure, ScoreLoadFailure.unavailable);
    });

    test('a timed-out access check plays the cached copy', () async {
      // Cached catalog piece + online: the conditional fetch times out → the
      // offline grace applies, exactly as for UNAVAILABLE.
      final cache = InMemoryOfflineScoreCache();
      await cache.write('contributed:1', bytes(), etag: 'e');
      final c = _build(
        upload: _FakeUpload(bytes(), error: deadlineExceeded()),
        conn: _Conn(true),
        cache: cache,
      );
      c.read(selectedScoreProvider.notifier).select(_upload());
      await _flush();
      expect(c.read(notationProvider).hasDocument, isTrue);
    });
  });

  group('mid-flight offline race', () {
    test('airplane mode resolves a hung load at once', () async {
      // The reported bug, as a unit test: the fetch never completes, and the
      // load must NOT wait for any deadline once the device says offline.
      final upload = _FakeUpload(bytes(), hang: true);
      final conn = _Conn(true);
      final c = _build(upload: upload, conn: conn);

      c.read(selectedScoreProvider.notifier).select(_upload());
      await _flush();
      expect(upload.fetchCalls, 1, reason: 'load is in flight');
      expect(c.read(notationProvider).failure, isNull, reason: 'still loading');

      conn.online = false;
      conn.ctrl.add(false); // the transition event
      await _flush();
      expect(
        c.read(notationProvider).failure,
        ScoreLoadFailure.offlineUnavailable,
        reason: 'resolved by the transition, not by a timeout',
      );
    });

    test('a late result from the abandoned fetch changes nothing', () async {
      final upload = _FakeUpload(bytes(), hang: true);
      final conn = _Conn(true);
      final c = _build(upload: upload, conn: conn);

      c.read(selectedScoreProvider.notifier).select(_upload());
      await _flush();
      conn.ctrl.add(false);
      await _flush();
      expect(
        c.read(notationProvider).failure,
        ScoreLoadFailure.offlineUnavailable,
      );

      // The orphaned fetch finally succeeds — too late; the offline outcome
      // must stand.
      upload.pending.complete(ScoreBytesResult(data: bytes(), etag: 'e'));
      await _flush();
      expect(
        c.read(notationProvider).failure,
        ScoreLoadFailure.offlineUnavailable,
      );
      expect(c.read(notationProvider).hasDocument, isFalse);
    });
  });

  group('pre-flight gate', () {
    test('offline at open issues no network call at all', () async {
      final c = _build(upload: _NeverCalledUpload(), conn: _Conn(false));
      c.read(selectedScoreProvider.notifier).select(_upload());
      await _flush();
      expect(
        c.read(notationProvider).failure,
        ScoreLoadFailure.offlineUnavailable,
      );
    });

    test(
      'a positive reading proves nothing: the call still goes out',
      () async {
        // Captive-portal case: the OS says online, the call proceeds under its
        // deadline (here: succeeds normally).
        final upload = _FakeUpload(bytes());
        final c = _build(upload: upload, conn: _Conn(true));
        c.read(selectedScoreProvider.notifier).select(_upload());
        await _flush();
        expect(upload.fetchCalls, 1);
        expect(c.read(notationProvider).hasDocument, isTrue);
      },
    );
  });
}
