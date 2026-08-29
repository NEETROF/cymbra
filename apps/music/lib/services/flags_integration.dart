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

import 'package:cymbra_flags/cymbra_flags.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../state/plan_notifier.dart';
import '../state/session_notifier.dart';
import 'grpc_client.dart';
import 'token_refresher.dart';
import 'token_store.dart';

/// Bearer that reads the app's stored access token (anonymous when signed out),
/// and renews it through the app's coordinated refresher when the server refuses
/// it (change: add-drum-input-mapping — beta fix).
///
/// The flag read is the one authenticated call that does NOT go through
/// [AuthedRunner], because it is optional-auth: there is a legitimate anonymous
/// caller, so the RPC cannot simply demand a token. That used to mean an expired
/// access token was answered as *anonymous* — no error, no cue to refresh, and a
/// downgraded set the client cached. A beta member lost the drums home at
/// random, for as long as it took some other RPC to refresh the token and the
/// next poll to run. The server now refuses a stale bearer, and this renews it
/// through the same single-flight refresher every other adapter shares, so a
/// refused flag read repairs itself in the same round trip.
class AppFlagBearer implements FlagBearer {
  AppFlagBearer(this._store, this._refresher);
  final TokenStore _store;
  final TokenRefresher _refresher;

  @override
  Future<String?> token() async => (await _store.readTokens())?.accessToken;

  @override
  Future<String?> renewed() async => switch (await _refresher.refresh()) {
    RefreshRefreshed(:final accessToken) => accessToken,
    // Rejected: the session is gone and has been cleared — the identity is
    // about to change, which resets the snapshot on its own.
    // Transient: offline or a flaky hop; the last-good snapshot stands and the
    // next poll retries.
    RefreshRejected() || RefreshTransient() => null,
  };
}

/// Wire the shared `cymbra_flags` client onto the app's gRPC channel, session
/// identity, and token store. Applied at the root `ProviderContainer` so the
/// snapshot is identity-scoped (resets on sign-out / user switch) and fetched
/// over the same channel as every other RPC.
List<Override> cymbraFlagOverrides() => [
  flagChannelProvider.overrideWith((ref) => ref.watch(cymbraChannelProvider)),
  // The account, and only the account: this is the key the snapshot is scoped
  // and cached under, and changing it blanks every flag until the network
  // answers. Sign-out and a user switch deserve that; nothing else does.
  flagIdentityProvider.overrideWith((ref) => ref.watch(currentUserIdProvider)),
  // Plan + betas (change: add-premium-subscription): a purchase, an enrolment,
  // a lapse or a beta closing changes what the server evaluates for the SAME
  // person, so it refetches — without the empty window that folding it into
  // the identity used to open, where a beta-gated entry point (the drums home)
  // disappeared for the length of a round trip. `planProvider` resolving at
  // launch is the same transition, on every cold start.
  flagAudienceProvider.overrideWith((ref) {
    final plan = ref.watch(planProvider).valueOrNull;
    if (plan == null) return '';
    final betas = plan.betas.map((b) => b.campaignKey).toList()..sort();
    return '${plan.plan}|${betas.join(',')}';
  }),
  flagBearerProvider.overrideWith(
    (ref) => AppFlagBearer(
      ref.watch(tokenStoreProvider),
      ref.watch(tokenRefresherProvider),
    ),
  ),
];
