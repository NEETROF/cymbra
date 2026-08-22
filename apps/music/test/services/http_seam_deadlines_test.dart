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
import 'dart:typed_data';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/rpc_deadlines.dart';
import 'package:music/services/token_store.dart';

import '../support/auth_fakes.dart';

Provider<PrivateSoundFontService> _svc(http.Client client) =>
    Provider((ref) => HttpPrivateSoundFontService(ref, client: client));

ProviderContainer _container() {
  final c = ProviderContainer(
    overrides: [
      cymbraEndpointProvider.overrideWithValue(
        const CymbraEndpoint(host: 'localhost', port: 50051, secure: false),
      ),
      tokenStoreProvider.overrideWithValue(
        FakeTokenStore()
          ..tokens = const StoredTokens(accessToken: 'tok', refreshToken: 'r'),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('a hung control call fails on the interactive budget as the seam '
      'exception', () {
    fakeAsync((fa) {
      final never = Completer<http.Response>();
      final client = MockClient((_) => never.future);
      final svc = _container().read(_svc(client));

      Object? error;
      svc.list().then<void>((_) {}).catchError((Object e) => error = e);

      fa.elapse(kInteractiveDeadline - const Duration(seconds: 1));
      expect(error, isNull, reason: 'not before the budget');

      fa.elapse(const Duration(seconds: 2));
      fa.flushMicrotasks();
      expect(
        error,
        isA<PrivateSoundFontException>(),
        reason: 'a timeout surfaces as the seam type, never raw',
      );
    });
  });

  test('the bulk import carries NO wall-clock timeout', () {
    // Design D5: at 400 MiB, any cap safe on a slow link is useless and any
    // useful cap truncates real uploads. A transfer still progressing after
    // half an hour must complete, not be aborted.
    fakeAsync((fa) {
      final gate = Completer<http.Response>();
      final client = MockClient((_) => gate.future);
      final svc = _container().read(_svc(client));

      RemoteSoundFont? result;
      Object? error;
      svc
          .import(Uint8List.fromList(const [1, 2, 3]), 'Grand Piano')
          .then<void>((r) => result = r)
          .catchError((Object e) => error = e);

      fa.elapse(const Duration(minutes: 30));
      expect(error, isNull, reason: 'no deadline may fire on a bulk transfer');

      gate.complete(
        http.Response(jsonEncode({'id': 'sf1', 'label': 'Grand Piano'}), 201),
      );
      fa.flushMicrotasks();
      expect(error, isNull);
      expect(result?.id, 'sf1', reason: 'the slow upload completed normally');
    });
  });

  test('the bulk download carries NO wall-clock timeout either', () {
    fakeAsync((fa) {
      final gate = Completer<http.Response>();
      final client = MockClient((_) => gate.future);
      final svc = _container().read(_svc(client));

      Uint8List? result;
      Object? error;
      svc
          .download('sf1')
          .then<void>((r) => result = r)
          .catchError((Object e) => error = e);

      fa.elapse(const Duration(minutes: 30));
      expect(error, isNull);

      gate.complete(http.Response.bytes(const [9, 9], 200));
      fa.flushMicrotasks();
      expect(result, isNotNull);
    });
  });
}
