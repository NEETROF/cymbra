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

import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'pending_social_link.g.dart';

/// The social `id_token` (and its provider) that just created a **brand-new**
/// account which landed on handle onboarding (change: add-account-identity-
/// linking, D7). Kept so the user can choose "Sign in to link" and attach this
/// identity to a pre-existing account instead. Held in memory only — never
/// persisted — and cleared once the orphan is linked, kept, or abandoned.
class PendingSocialLink {
  final String idToken;

  /// `google` or `apple` — the provider whose sheet minted [idToken].
  final String provider;

  const PendingSocialLink({required this.idToken, required this.provider});
}

/// Holds the [PendingSocialLink] for the current sign-in, or null when there is
/// none. Set by the OIDC sign-in path when it provisions a brand-new account;
/// read by the handle-onboarding screen to offer the "Sign in to link" option.
@Riverpod(keepAlive: true)
class PendingSocialLinkController extends _$PendingSocialLinkController {
  @override
  PendingSocialLink? build() => null;

  void set(PendingSocialLink link) => state = link;

  void clear() => state = null;
}
