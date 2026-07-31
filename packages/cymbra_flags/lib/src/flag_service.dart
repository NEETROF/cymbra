import 'dart:convert';

import 'package:grpc/grpc.dart';

import 'flag_snapshot.dart';
import 'grpc/flags.pbgrpc.dart' as pb;

/// Result of a fetch: either the new snapshot, or `unchanged` when the server
/// matched the sent version (the caller keeps its last-good snapshot).
class FlagFetch {
  const FlagFetch.changed(this.snapshot) : unchanged = false;
  const FlagFetch.unchanged() : snapshot = null, unchanged = true;
  final FlagSnapshot? snapshot;
  final bool unchanged;
}

/// The fetch seam. Production talks gRPC; tests inject a fake.
abstract class FlagService {
  /// Fetch the effective flags for [app]. [knownVersion] enables a cheap
  /// "unchanged" answer; [bearer] attaches the identity (`null` ⇒ anonymous set).
  Future<FlagFetch> fetch({
    required String app,
    required String? identity,
    required String knownVersion,
    required String? bearer,
  });
}

CallOptions _bearer(String? token) => (token == null || token.isEmpty)
    ? CallOptions()
    : CallOptions(metadata: {'authorization': 'Bearer $token'});

/// Production [FlagService] over the generated `FlagServiceClient`. The gRPC I/O
/// sits behind the [FlagService] seam and is coverage-excluded like the other
/// network adapters; its pure translation ([snapshotFromResponse]/[entryFromWire])
/// is unit-tested.
// coverage:ignore-start
class GrpcFlagService implements FlagService {
  GrpcFlagService(ClientChannel channel)
    : _client = pb.FlagServiceClient(channel);

  final pb.FlagServiceClient _client;

  @override
  Future<FlagFetch> fetch({
    required String app,
    required String? identity,
    required String knownVersion,
    required String? bearer,
  }) async {
    final resp = await _client.getEffectiveFlags(
      pb.GetEffectiveFlagsRequest(app: app, knownVersion: knownVersion),
      options: _bearer(bearer),
    );
    if (resp.unchanged) return const FlagFetch.unchanged();
    return FlagFetch.changed(
      snapshotFromResponse(resp, app: app, identity: identity),
    );
  }
}
// coverage:ignore-end

/// Convert a wire response to a [FlagSnapshot] (kept separate so it is unit-
/// testable without a channel).
FlagSnapshot snapshotFromResponse(
  pb.GetEffectiveFlagsResponse resp, {
  required String app,
  required String? identity,
}) {
  final entries = <String, FlagEntry>{};
  for (final f in resp.flags) {
    final entry = entryFromWire(f.value);
    if (entry != null) entries[f.key] = entry;
  }
  return FlagSnapshot(
    app: app,
    identity: identity,
    version: resp.version,
    entries: entries,
  );
}

/// Map a wire [pb.FlagValue] to a typed [FlagEntry] (`null` when unset/bad JSON).
FlagEntry? entryFromWire(pb.FlagValue v) {
  switch (v.whichKind()) {
    case pb.FlagValue_Kind.boolValue:
      return FlagEntry(FlagKind.bool_, v.boolValue);
    case pb.FlagValue_Kind.intValue:
      return FlagEntry(FlagKind.int_, v.intValue.toInt());
    case pb.FlagValue_Kind.numberValue:
      return FlagEntry(FlagKind.number, v.numberValue);
    case pb.FlagValue_Kind.stringValue:
      return FlagEntry(FlagKind.string, v.stringValue);
    case pb.FlagValue_Kind.jsonValue:
      try {
        return FlagEntry(FlagKind.json, jsonDecode(v.jsonValue) as Object);
      } catch (_) {
        return null;
      }
    case pb.FlagValue_Kind.notSet:
      return null;
  }
}
