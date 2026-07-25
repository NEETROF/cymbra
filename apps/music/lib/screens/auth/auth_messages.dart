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

import '../../l10n/gen/app_localizations.dart';
import '../../services/auth_service.dart';
import '../../widgets/app_snackbar.dart';

/// A human, non-enumerating message for an [AuthException]. Localized via [l10n];
/// screens may pass a [fallback] tuned to their context (e.g. sign-in vs reset).
String authErrorMessage(
  AppLocalizations l10n,
  AuthException e, {
  String? fallback,
}) {
  switch (e.error) {
    case AuthError.unauthenticated:
      return fallback ?? l10n.authErrUnauthenticated;
    case AuthError.alreadyExists:
      return l10n.authErrAlreadyExists;
    case AuthError.rateLimited:
      return l10n.authErrRateLimited;
    case AuthError.failedPrecondition:
      return l10n.authErrUnverified;
    case AuthError.invalidArgument:
      return fallback ?? l10n.authErrInvalidCode;
    case AuthError.conflict:
      return l10n.authErrConflict;
    case AuthError.notFound:
      return l10n.authErrNotFound;
    case AuthError.unavailable:
      return l10n.authErrUnavailable;
    case AuthError.unknown:
      return fallback ?? l10n.authErrUnknown;
  }
}

/// Show an [AuthException] as a SnackBar using [authErrorMessage].
void showAuthError(BuildContext context, AuthException e, {String? fallback}) {
  showAppSnackBar(
    ScaffoldMessenger.of(context),
    authErrorMessage(AppLocalizations.of(context), e, fallback: fallback),
  );
}
