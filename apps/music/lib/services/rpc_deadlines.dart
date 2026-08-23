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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'rpc_deadlines.g.dart';

/// Per-category gRPC deadlines (change: add-client-transport-deadlines).
///
/// Every RPC carries an app-level deadline so an unreachable backend fails in
/// seconds instead of the OS TCP timeout (~75s on Darwin). One budget per
/// *category*, never one global cap: interactive reads are short, byte
/// transfers are generous, and the user-initiated score upload is longer still.

/// Small request/response calls a user is actively waiting on (search,
/// listings, account reads/writes, ratings, telemetry…). The default.
const Duration kInteractiveDeadline = Duration(seconds: 10);

/// Unary calls carrying a document or media payload (score bytes, preview
/// bytes, course manifests).
const Duration kTransferDeadline = Duration(seconds: 30);

/// User-initiated submissions expected to take a while (score upload).
const Duration kLongDeadline = Duration(seconds: 120);

/// Bound on establishing a connection to the backend (`ChannelOptions.
/// connectTimeout`). Defence in depth: the per-call deadline is armed at call
/// creation and already covers connect time; this bounds any future path that
/// does not go through a deadline-carrying call.
const Duration kConnectTimeout = Duration(seconds: 10);

/// Explicit per-method overrides, keyed by the generated `ClientMethod.path`.
///
/// ⚠️ An RPC absent from this table inherits [kInteractiveDeadline] (10s). A
/// *new* long-running RPC MUST be added here or it will time out on first use.
/// That failure mode is deliberate: loud and immediate beats a new interactive
/// RPC silently inheriting a 120s budget.
const Map<String, Duration> kRpcDeadlineOverrides = {
  // transfer — byte payloads.
  '/cymbra.music.v1.ScoreService/GetCatalogScoreBytes': kTransferDeadline,
  '/cymbra.music.v1.ScoreService/GetScoreBytes': kTransferDeadline,
  '/cymbra.music.v1.ScoreService/GetRatingPreviewBytes': kTransferDeadline,
  // GetCourse returns the whole course manifest blob.
  '/cymbra.music.v1.ScoreService/GetCourse': kTransferDeadline,
  // long — user-initiated submissions. (The 400MiB SoundFont import is HTTP,
  // not gRPC — see private_soundfont_service.dart.)
  '/cymbra.music.v1.ScoreService/UploadScore': kLongDeadline,
};

/// The deadline for a generated method path: its override, else interactive.
Duration deadlineForMethod(String path) =>
    kRpcDeadlineOverrides[path] ?? kInteractiveDeadline;

/// [ClientInterceptor] that attaches the category deadline to every call.
///
/// Installed once per generated client (`interceptors: [deadlines]`), so no
/// call site can opt out by omission — an RPC added tomorrow is bounded whether
/// or not its author thought about it. The policy is merged as the **base**
/// (`CallOptions(timeout: …).mergedWith(options)`): `mergedWith` resolves
/// `other.timeout ?? timeout`, so an explicit per-call timeout always wins and
/// is never silently overwritten.
class RpcDeadlines implements ClientInterceptor {
  const RpcDeadlines();

  CallOptions _withDeadline(String path, CallOptions options) =>
      CallOptions(timeout: deadlineForMethod(path)).mergedWith(options);

  @override
  ResponseFuture<R> interceptUnary<Q, R>(
    ClientMethod<Q, R> method,
    Q request,
    CallOptions options,
    ClientUnaryInvoker<Q, R> invoker,
  ) => invoker(method, request, _withDeadline(method.path, options));

  @override
  ResponseStream<R> interceptStreaming<Q, R>(
    ClientMethod<Q, R> method,
    Stream<Q> requests,
    CallOptions options,
    ClientStreamingInvoker<Q, R> invoker,
  ) => invoker(method, requests, _withDeadline(method.path, options));
}

/// Shared stateless instance wired into every gRPC adapter.
@Riverpod(keepAlive: true)
RpcDeadlines rpcDeadlines(Ref ref) => const RpcDeadlines();
