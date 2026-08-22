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

import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:music/services/rpc_deadlines.dart';

/// Marker thrown by the capturing invoker so the interceptor call short-circuits
/// without needing a real channel.
class _Probe implements Exception {
  const _Probe();
}

ClientMethod<int, int> _method(String path) =>
    ClientMethod<int, int>(path, (r) => const [], (b) => 0);

/// Runs [RpcDeadlines.interceptUnary] against a capturing invoker and returns
/// the [CallOptions] that would reach the channel.
CallOptions _sentOptions(String path, CallOptions options) {
  CallOptions? seen;
  ResponseFuture<int> invoker(
    ClientMethod<int, int> method,
    int request,
    CallOptions o,
  ) {
    seen = o;
    throw const _Probe();
  }

  expect(
    () =>
        const RpcDeadlines().interceptUnary(_method(path), 1, options, invoker),
    throwsA(isA<_Probe>()),
  );
  return seen!;
}

void main() {
  group('deadlineForMethod', () {
    test('every override path returns its category budget', () {
      for (final entry in kRpcDeadlineOverrides.entries) {
        expect(deadlineForMethod(entry.key), entry.value, reason: entry.key);
      }
      // The table holds the intended categories, not accidents.
      expect(
        deadlineForMethod('/cymbra.music.v1.ScoreService/UploadScore'),
        kLongDeadline,
      );
      expect(
        deadlineForMethod('/cymbra.music.v1.ScoreService/GetCatalogScoreBytes'),
        kTransferDeadline,
      );
    });

    test('an unknown path falls back to the interactive budget', () {
      expect(
        deadlineForMethod('/cymbra.music.v1.ScoreService/SearchCatalog'),
        kInteractiveDeadline,
      );
      expect(deadlineForMethod('/nope.v1.Nope/Nope'), kInteractiveDeadline);
    });
  });

  group('RpcDeadlines interceptor', () {
    test('attaches the category deadline to the outgoing options', () {
      final sent = _sentOptions(
        '/cymbra.music.v1.ScoreService/SearchCatalog',
        CallOptions(),
      );
      expect(sent.timeout, kInteractiveDeadline);
    });

    test('attaches the override deadline for a table path', () {
      final sent = _sentOptions(
        '/cymbra.music.v1.ScoreService/UploadScore',
        CallOptions(),
      );
      expect(sent.timeout, kLongDeadline);
    });

    test(
      'merge direction: an explicit per-call timeout is never overwritten',
      () {
        // The one invertible line (design D3): the policy must be the BASE.
        // `mergedWith` resolves `other.timeout ?? timeout`, so if this assert
        // fails the merge was written the wrong way round.
        const explicit = Duration(seconds: 3);
        final sent = _sentOptions(
          '/cymbra.music.v1.ScoreService/UploadScore',
          CallOptions(timeout: explicit),
        );
        expect(sent.timeout, explicit);
      },
    );

    test('call metadata (the bearer header) survives the merge', () {
      final sent = _sentOptions(
        '/cymbra.music.v1.ScoreService/SearchCatalog',
        CallOptions(metadata: {'authorization': 'Bearer t'}),
      );
      expect(sent.metadata['authorization'], 'Bearer t');
      expect(sent.timeout, kInteractiveDeadline);
    });

    test('interceptStreaming applies the same policy', () {
      CallOptions? seen;
      ResponseStream<int> invoker(
        ClientMethod<int, int> method,
        Stream<int> requests,
        CallOptions o,
      ) {
        seen = o;
        throw const _Probe();
      }

      expect(
        () => const RpcDeadlines().interceptStreaming(
          _method('/cymbra.music.v1.ScoreService/SearchCatalog'),
          const Stream<int>.empty(),
          CallOptions(),
          invoker,
        ),
        throwsA(isA<_Probe>()),
      );
      expect(seen!.timeout, kInteractiveDeadline);
    });
  });
}
