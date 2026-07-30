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
import 'package:freezed_annotation/freezed_annotation.dart';

import '../services/account_service.dart';
import '../services/auth_service.dart';

part 'connected_accounts_state.freezed.dart';

/// The mutating action a [ConnectedAccountsState.actionError] came from, so the
/// listener can pick the right message (an `ALREADY_EXISTS` means "already linked
/// elsewhere" for a social link but "email already in use" for a password).
enum ConnectedAccountsAction { linkGoogle, linkApple, setPassword, unlink }

/// State of the Connected accounts screen (change: add-account-identity-linking).
/// [identities] is the loaded list (loading/data/error drives the list body);
/// [busy] guards the screen while a link/unlink action runs. An action's outcome
/// is carried by [actionSeq] + [actionError]: [actionSeq] increments once per
/// completed action so a dedicated listener fires even when two consecutive
/// actions fail with the *same* error; [actionError] is that error, or null on
/// success. The raw error is mapped to a friendly message by the listener — never
/// shown directly (memory: no-raw-technical-errors-in-ui).
@freezed
sealed class ConnectedAccountsState with _$ConnectedAccountsState {
  const ConnectedAccountsState._();

  const factory ConnectedAccountsState({
    required AsyncValue<List<LinkedIdentity>> identities,
    @Default(false) bool busy,
    @Default(0) int actionSeq,
    AuthException? actionError,
    ConnectedAccountsAction? lastAction,
  }) = _ConnectedAccountsState;

  /// The loaded identities, or empty while loading / on error.
  List<LinkedIdentity> get items => identities.valueOrNull ?? const [];

  /// Only one identity remains — its unlink is blocked (anti-lockout, D4).
  bool get isLastIdentity => items.length <= 1;

  bool _has(String provider) => items.any((i) => i.provider == provider);

  /// Whether the account already has an email/password credential.
  bool get hasLocal => _has(LinkedIdentity.providerLocal);

  /// Whether a Google identity is already linked.
  bool get hasGoogle => _has(LinkedIdentity.providerGoogle);

  /// Whether an Apple identity is already linked.
  bool get hasApple => _has(LinkedIdentity.providerApple);
}
