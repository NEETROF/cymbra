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

/// The caller's account, as returned by Cymbra ID's `UserService`. A null
/// [handle] means the user has not completed handle onboarding yet.
class Account {
  final String userId;
  final String? handle;
  final String? displayName;

  /// Optimistic-concurrency version; echoed back on `UpdateAccount`.
  final int version;

  const Account({
    required this.userId,
    required this.version,
    this.handle,
    this.displayName,
  });

  /// Whether the user still needs to choose a handle (drives onboarding).
  bool get needsHandle => handle == null || handle!.isEmpty;
}

/// A sign-in identity bound to an account (change: add-account-identity-linking),
/// as returned by `UserService.ListIdentities`. [provider] is `local`, `google`,
/// or `apple`; [subject] is the opaque provider key used to unlink — never
/// displayed. [linkedAt] is when the identity was attached.
class LinkedIdentity {
  final String provider;
  final String subject;
  final DateTime linkedAt;

  const LinkedIdentity({
    required this.provider,
    required this.subject,
    required this.linkedAt,
  });

  static const String providerLocal = 'local';
  static const String providerGoogle = 'google';
  static const String providerApple = 'apple';
}

/// Seam over Cymbra ID's `UserService` account surface (task 3.3). Protected by
/// the bearer session; the production implementation refreshes transparently on
/// `UNAUTHENTICATED`. Tests override the provider with an in-memory fake.
/// Failures throw [AuthException] (see auth_service.dart).
abstract class AccountService {
  /// Read the caller's account.
  Future<Account> getAccount();

  /// Set/replace the caller's handle, returning the updated account. A
  /// case-insensitive uniqueness conflict surfaces as `AuthError.alreadyExists`.
  Future<Account> updateHandle({
    required String handle,
    required int expectedVersion,
  });

  /// Advisory availability check for [handle] (the write path is authoritative).
  /// An invalid handle surfaces as `AuthError.invalidArgument`.
  Future<bool> checkHandleAvailability(String handle);

  /// Permanently delete the caller's account.
  Future<void> deleteAccount();

  /// List the sign-in identities currently linked to the caller's account
  /// (change: add-account-identity-linking). Drives the Connected accounts screen.
  Future<List<LinkedIdentity>> listIdentities();
}
