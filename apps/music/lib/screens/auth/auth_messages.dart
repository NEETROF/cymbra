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
import '../../state/connected_accounts_state.dart';
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
      // The email-credential copy ("Incorrect email or password.") is NOT the
      // default: only the local sign-in flow passes it as an explicit [fallback].
      // Every other flow (OIDC sign-in, link/unlink) gets a neutral message so a
      // Google/Apple failure never reads as a wrong password (change:
      // add-account-identity-linking, D5).
      return fallback ?? l10n.authErrSignInFailed;
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

/// A message for a link/unlink failure on the Connected accounts screen (change:
/// add-account-identity-linking). The mapping is action-aware because an
/// `ALREADY_EXISTS` means different things: for a social link the identity is
/// bound to another account; for "Set a password" the email is already in use.
/// A refused unlink (`FAILED_PRECONDITION`) is the last-identity guard — never the
/// email-verification copy the generic mapper would return (D5).
String linkErrorMessage(
  AppLocalizations l10n,
  AuthException e,
  ConnectedAccountsAction action,
) {
  switch (e.error) {
    case AuthError.alreadyExists:
      return action == ConnectedAccountsAction.setPassword
          ? l10n.authErrAlreadyExists
          : l10n.authErrAlreadyLinkedElsewhere;
    case AuthError.failedPrecondition:
      return l10n.authErrOnlySignInMethod;
    default:
      return authErrorMessage(l10n, e, fallback: l10n.authErrLinkFailed);
  }
}

/// Show an [AuthException] as a SnackBar using [authErrorMessage].
void showAuthError(BuildContext context, AuthException e, {String? fallback}) {
  showAppSnackBar(
    ScaffoldMessenger.of(context),
    authErrorMessage(AppLocalizations.of(context), e, fallback: fallback),
  );
}
