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
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/account_service.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/oidc_token_source.dart';
import 'package:music/services/token_store.dart';

import 'auth_fakes.dart';

/// Override list for the Cymbra ID seams, so nothing touches a channel or
/// platform plugin. Compose with extra overrides (e.g. `scoreCatalogProvider`).
List<Override> authOverrides({
  FakeTokenStore? store,
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeOidcTokenSource? oidc,
}) => [
  tokenStoreProvider.overrideWithValue(store ?? FakeTokenStore()),
  authServiceProvider.overrideWithValue(auth ?? FakeAuthService()),
  accountServiceProvider.overrideWithValue(account ?? FakeAccountService()),
  oidcTokenSourceProvider.overrideWithValue(oidc ?? FakeOidcTokenSource()),
];

/// A [ProviderContainer] with every Cymbra ID seam overridden by a fake.
ProviderContainer authContainer({
  FakeTokenStore? store,
  FakeAuthService? auth,
  FakeAccountService? account,
  FakeOidcTokenSource? oidc,
}) {
  final container = ProviderContainer(
    overrides: authOverrides(
      store: store,
      auth: auth,
      account: account,
      oidc: oidc,
    ),
  );
  addTearDown(container.dispose);
  return container;
}

/// Helper to read a fresh [Account] with the given handle.
Account fakeAccount({String? handle, int version = 1}) =>
    Account(userId: 'user-1', version: version, handle: handle);
