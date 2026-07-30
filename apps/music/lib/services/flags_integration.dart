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

import '../state/session_notifier.dart';
import 'grpc_client.dart';
import 'token_store.dart';

/// Bearer that reads the app's stored access token (anonymous when signed out).
/// An expired token is harmless: the backend's optional-auth read returns the
/// anonymous set rather than erroring.
class AppFlagBearer implements FlagBearer {
  AppFlagBearer(this._store);
  final TokenStore _store;

  @override
  Future<String?> token() async => (await _store.readTokens())?.accessToken;
}

/// Wire the shared `cymbra_flags` client onto the app's gRPC channel, session
/// identity, and token store. Applied at the root `ProviderContainer` so the
/// snapshot is identity-scoped (resets on sign-out / user switch) and fetched
/// over the same channel as every other RPC.
List<Override> cymbraFlagOverrides() => [
  flagChannelProvider.overrideWith((ref) => ref.watch(cymbraChannelProvider)),
  flagIdentityProvider.overrideWith((ref) => ref.watch(currentUserIdProvider)),
  flagBearerProvider.overrideWith(
    (ref) => AppFlagBearer(ref.watch(tokenStoreProvider)),
  ),
];
