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

import 'package:freezed_annotation/freezed_annotation.dart';

import '../services/account_service.dart';

part 'session_state.freezed.dart';

/// The resolved account-session state that drives the app's launch routing
/// (design D2). `unknown` is the transient startup state while the session is
/// hydrated from secure storage; the entry screen renders only for
/// `unauthenticated`.
@freezed
sealed class SessionState with _$SessionState {
  /// Startup: the session is still being resolved (show a loader, not entry).
  const factory SessionState.unknown() = SessionUnknown;

  /// Persisted guest choice: fully offline, no Cymbra ID session.
  const factory SessionState.guest() = SessionGuest;

  /// A live Cymbra ID session. [account] is null when it could not be fetched
  /// (e.g. offline at startup); [needsHandle] then defaults to false.
  const factory SessionState.authenticated({Account? account}) =
      SessionAuthenticated;

  /// No session and no guest choice — the entry screen is shown.
  const factory SessionState.unauthenticated() = SessionUnauthenticated;

  const SessionState._();

  /// Whether the user must still pick a handle before reaching the library.
  bool get needsHandle => switch (this) {
    SessionAuthenticated(:final account) => account?.needsHandle ?? false,
    _ => false,
  };
}
