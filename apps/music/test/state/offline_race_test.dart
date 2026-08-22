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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/state/offline_race.dart';

/// Controllable connectivity with listen/cancel accounting, so the leak test
/// can assert the race releases its subscription on every path.
class _Conn extends Fake implements ConnectivityService {
  final _ctrl = StreamController<bool>.broadcast();
  int listens = 0;
  int cancels = 0;

  @override
  Stream<bool> get onlineStatus {
    final out = StreamController<bool>(onCancel: () => cancels++);
    listens++;
    out.addStream(_ctrl.stream);
    return out.stream;
  }

  void emit(bool online) => _ctrl.add(online);
}

Future<void> _flush() async {
  for (var i = 0; i < 4; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

void main() {
  test('the offline transition wins over never-completing work', () async {
    final conn = _Conn();
    final work = Completer<int>();
    final raced = raceAgainstOffline(work.future, conn);
    conn.emit(false);
    await expectLater(raced, throwsA(isA<OfflineDuringLoad>()));
  });

  test('completed work wins; a later transition changes nothing', () async {
    final conn = _Conn();
    final result = await raceAgainstOffline(Future.value(42), conn);
    expect(result, 42);
    conn.emit(false); // late transition: nothing to affect
    await _flush();
  });

  test('an "online" event never aborts the work', () async {
    final conn = _Conn();
    final work = Completer<int>();
    final raced = raceAgainstOffline(work.future, conn);
    conn.emit(true);
    await _flush();
    work.complete(7);
    expect(await raced, 7);
  });

  test('a work error propagates as-is', () async {
    final conn = _Conn();
    await expectLater(
      raceAgainstOffline(Future<int>.error(StateError('boom')), conn),
      throwsStateError,
    );
  });

  test('the subscription is released on the normal (work-wins) path', () async {
    // A leak here is invisible until it is a lot of leaks: one subscription
    // per score open, on a broadcast stream nobody else drains.
    final conn = _Conn();
    await raceAgainstOffline(Future.value(1), conn);
    await _flush();
    expect(conn.listens, 1);
    expect(conn.cancels, 1);
  });

  test('the subscription is released when offline wins', () async {
    final conn = _Conn();
    final work = Completer<int>();
    final raced = raceAgainstOffline(work.future, conn);
    conn.emit(false);
    await expectLater(raced, throwsA(isA<OfflineDuringLoad>()));
    await _flush();
    expect(conn.cancels, 1);
  });

  test(
    'a late result from abandoned work is swallowed, not unhandled',
    () async {
      final conn = _Conn();
      final work = Completer<int>();
      final raced = raceAgainstOffline(work.future, conn);
      conn.emit(false);
      await expectLater(raced, throwsA(isA<OfflineDuringLoad>()));
      // The orphaned work failing later must not surface as an unhandled error
      // (flutter_test fails the test if one does).
      work.completeError(StateError('late failure'));
      await _flush();
    },
  );
}
