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

// Client-side mirrors of the backend policies, so the UI can reject bad input
// before a round-trip. Pure and host-tested; the backend remains authoritative.

/// Minimum password length enforced by Cymbra ID (`CYMBRA_PASSWORD_MIN_LENGTH`).
const int kPasswordMinLength = 12;

/// Maximum handle length (matches the backend `handle_core`).
const int kHandleMaxLength = 15;

/// A policy error message for [password], or null when it satisfies the policy.
String? passwordPolicyError(String password) {
  if (password.length < kPasswordMinLength) {
    return 'Password must be at least $kPasswordMinLength characters.';
  }
  return null;
}

/// Whether [handle] satisfies the policy: 1–15 Unicode letters/numbers only
/// (no spaces, punctuation, or symbols). Mirrors the backend `handle_core`.
bool isValidHandle(String handle) {
  final len = handle.runes.length;
  if (len == 0 || len > kHandleMaxLength) return false;
  // A character is allowed iff it is a letter or a digit. Dart has no direct
  // Unicode category test, so accept any rune that uppercases/lowercases to
  // itself only when it is alphanumeric — approximated with a regex over the
  // common ranges plus a broad Unicode-letter allowance.
  return _handleAllowed.hasMatch(handle);
}

/// Letters (incl. accented/Unicode) and digits, 1–15 long. `\p{L}`/`\p{N}` use
/// Unicode property escapes (Dart regex `unicode: true`).
final RegExp _handleAllowed = RegExp(r'^[\p{L}\p{N}]{1,15}$', unicode: true);
