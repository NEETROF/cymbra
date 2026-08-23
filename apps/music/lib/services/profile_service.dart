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

import '../src/grpc/user.pbgrpc.dart' as user;
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'profile_service.g.dart';

/// A player's public profile as returned by the server (change: add-play-
/// activity-profile) — the allow-listed fields only. `visibility` is one of
/// `private` | `limited` | `public`.
class PlayerProfile {
  const PlayerProfile({
    required this.userId,
    required this.handle,
    required this.displayName,
    required this.visibility,
  });

  final String userId;
  final String? handle;
  final String? displayName;
  final String visibility;
}

/// Seam for the public-profile read + the visibility control (change: add-play-
/// activity-profile). Behind a provider so profile screens are testable without
/// the backend.
abstract class ProfileService {
  /// Read [userId]'s profile. The server honors visibility + eligibility
  /// fail-closed (a non-public target is refused there), so a `notFound` failure
  /// means "not viewable".
  Future<PlayerProfile> getPlayerProfile(String userId);

  /// Set the caller's own visibility. Going `public` needs [dateOfBirth] the
  /// first time (the neutral age gate); it is sent once and discarded server-side
  /// (only the derived eligibility date is kept). Returns the visibility now in
  /// effect; a `failedPrecondition` failure means the user is not old enough.
  Future<String> setProfileVisibility(
    String visibility, {
    DateTime? dateOfBirth,
  });
}

/// Production [ProfileService] over the generated `UserServiceClient` (the
/// profile RPCs live on the user service). Protected calls run through
/// [authedCall] (refresh-once-and-retry).
class GrpcProfileService implements ProfileService {
  GrpcProfileService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = user.UserServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final user.UserServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<PlayerProfile> getPlayerProfile(String userId) =>
      _authed((bearer) async {
        final p = await _client.getPlayerProfile(
          user.GetPlayerProfileRequest(userId: userId),
          options: bearerOptions(bearer),
        );
        return PlayerProfile(
          userId: p.userId,
          handle: p.hasHandle() ? p.handle : null,
          displayName: p.hasDisplayName() ? p.displayName : null,
          visibility: p.visibility,
        );
      });

  @override
  Future<String> setProfileVisibility(
    String visibility, {
    DateTime? dateOfBirth,
  }) => _authed((bearer) async {
    final req = user.SetProfileVisibilityRequest(visibility: visibility);
    if (dateOfBirth != null) {
      // Neutral age gate: send the DOB once as ISO yyyy-mm-dd; the server derives
      // + stores only the eligibility date and discards this.
      final d = dateOfBirth;
      req.dateOfBirth =
          '${d.year.toString().padLeft(4, '0')}-'
          '${d.month.toString().padLeft(2, '0')}-'
          '${d.day.toString().padLeft(2, '0')}';
    }
    final resp = await _client.setProfileVisibility(
      req,
      options: bearerOptions(bearer),
    );
    return resp.visibility;
  });
}

/// Production profile-service provider. Override in tests with a fake/mock.
@Riverpod(keepAlive: true)
ProfileService profileService(Ref ref) => GrpcProfileService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
  deadlines: ref.watch(rpcDeadlinesProvider),
);
