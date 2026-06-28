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

import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:sign_in_with_apple/sign_in_with_apple.dart';

import 'oidc_config.dart';

part 'oidc_token_source.g.dart';

/// Seam over the native Google/Apple sign-in SDKs (design D1). Each method drives
/// the platform consent sheet and returns the provider `id_token` to post to
/// Cymbra ID's `SignInOidc`, or **null** when the user cancels. The app never
/// performs the OAuth exchange itself. Kept abstract so the OIDC sign-in flow is
/// tested with a fake token source (task 6.5) — no native channel.
///
/// The `*Available` flags gate the entry buttons: when a provider is not
/// configured (no client ID / capability) its button is hidden, so the native
/// SDK is never invoked unconfigured — that would throw an uncatchable native
/// exception and crash the app.
abstract class OidcTokenSource {
  /// Obtain a Google `id_token`, or null if the user dismissed the sheet.
  Future<String?> googleIdToken();

  /// Obtain an Apple `id_token`, or null if the user cancelled.
  Future<String?> appleIdToken();

  /// Whether Google sign-in is configured (a client ID is present).
  bool get googleAvailable;

  /// Whether Sign in with Apple is offered (enabled + on an Apple platform).
  bool get appleAvailable;
}

/// Production [OidcTokenSource] backed by `google_sign_in` and
/// `sign_in_with_apple`. A user cancellation is normalized to null (no error).
class NativeOidcTokenSource implements OidcTokenSource {
  const NativeOidcTokenSource();

  @override
  bool get googleAvailable => kGoogleClientId.isNotEmpty;

  @override
  bool get appleAvailable =>
      kAppleSignInEnabled && !kIsWeb && (Platform.isIOS || Platform.isMacOS);

  @override
  Future<String?> googleIdToken() async {
    if (!googleAvailable) return null; // not configured — caller guards too
    // Pass the client ID in Dart so no GIDClientID Info.plist entry is required;
    // the reversed-client-id URL scheme is still needed for the callback.
    final account = await GoogleSignIn(clientId: kGoogleClientId).signIn();
    if (account == null) return null; // user dismissed the sheet
    final auth = await account.authentication;
    return auth.idToken;
  }

  @override
  Future<String?> appleIdToken() async {
    if (!appleAvailable) return null;
    try {
      final cred = await SignInWithApple.getAppleIDCredential(
        scopes: const [AppleIDAuthorizationScopes.email],
      );
      return cred.identityToken;
    } on SignInWithAppleAuthorizationException catch (e) {
      if (e.code == AuthorizationErrorCode.canceled) return null;
      rethrow;
    }
  }
}

/// Production OIDC-token-source provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
OidcTokenSource oidcTokenSource(Ref ref) => const NativeOidcTokenSource();
