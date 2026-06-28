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

import 'package:flutter/material.dart';

import '../../services/auth_service.dart';

/// A human, non-enumerating message for an [AuthException]. Pure (host-testable);
/// screens may pass a [fallback] tuned to their context (e.g. sign-in vs reset).
String authErrorMessage(AuthException e, {String? fallback}) {
  switch (e.error) {
    case AuthError.unauthenticated:
      return fallback ?? 'Incorrect email or password.';
    case AuthError.alreadyExists:
      return 'That email is already in use.';
    case AuthError.rateLimited:
      return 'Too many attempts. Please wait and try again.';
    case AuthError.failedPrecondition:
      return 'Please verify your email first.';
    case AuthError.invalidArgument:
      return fallback ?? 'That code is invalid or has expired.';
    case AuthError.conflict:
      return 'That was just taken — please try again.';
    case AuthError.notFound:
      return 'Not found.';
    case AuthError.unavailable:
      return 'Can’t reach Cymbra. Check your connection.';
    case AuthError.unknown:
      return fallback ?? 'Something went wrong. Please try again.';
  }
}

/// Show an [AuthException] as a SnackBar using [authErrorMessage].
void showAuthError(BuildContext context, AuthException e, {String? fallback}) {
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text(authErrorMessage(e, fallback: fallback))),
  );
}
