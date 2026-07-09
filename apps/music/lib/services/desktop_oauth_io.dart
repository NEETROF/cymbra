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

// Concrete native glue for the desktop loopback OAuth flow: a `dart:io`
// `HttpServer` loopback listener, the `url_launcher` browser launch, and the
// `http` token exchange. This is the un-unit-testable seam boundary (like the
// native SDK adapters) — kept out of the tested orchestration in
// desktop_oidc_token_source.dart and imported only by the production provider.

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:url_launcher/url_launcher.dart';

import 'desktop_oauth_core.dart';
import 'desktop_oidc_token_source.dart';
import 'oidc_config.dart';

/// Minimal HTML shown in the browser tab after the redirect is captured.
const String _kSuccessHtml =
    '<!doctype html><html><head><meta charset="utf-8">'
    '<title>Cymbra</title></head><body style="font-family:sans-serif;'
    'text-align:center;padding-top:3rem">'
    '<h2>You\'re signed in.</h2><p>You can close this tab and return to Cymbra.</p>'
    '</body></html>';

/// `dart:io` loopback listener bound to `127.0.0.1` on an ephemeral port.
class IoLoopbackServer implements LoopbackServer {
  HttpServer? _server;

  @override
  Future<String> bind() async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    _server = server;
    return redirectUri;
  }

  @override
  String get redirectUri {
    final server = _server;
    if (server == null) {
      throw StateError('LoopbackServer.redirectUri read before bind()');
    }
    return 'http://127.0.0.1:${server.port}/';
  }

  @override
  Future<Uri?> waitForRedirect(Duration timeout) async {
    final server = _server;
    if (server == null) return null;
    final captured = Completer<Uri?>();
    // Serve requests until one carries the OAuth `code`/`error` — a browser may
    // hit the loopback with an unrelated request first (favicon, preconnect),
    // which `first` would otherwise mistake for the redirect. Answer those with
    // 404 and keep waiting (still bounded by `timeout`).
    final sub = server.listen((request) async {
      final uri = request.uri; // path + query — enough for parseRedirect
      final q = uri.queryParameters;
      final isRedirect = q.containsKey('code') || q.containsKey('error');
      try {
        request.response
          ..statusCode = isRedirect ? HttpStatus.ok : HttpStatus.notFound
          ..headers.contentType = ContentType.html;
        if (isRedirect) request.response.write(_kSuccessHtml);
        await request.response.close();
      } catch (_) {
        // Client disconnected (e.g. tab closed) — the success page is cosmetic;
        // never let a write failure discard an already-captured auth code.
      }
      if (isRedirect && !captured.isCompleted) captured.complete(uri);
    });
    try {
      return await captured.future.timeout(timeout, onTimeout: () => null);
    } finally {
      await sub.cancel();
      await close();
    }
  }

  @override
  Future<void> close() async {
    final server = _server;
    _server = null;
    await server?.close(force: true);
  }
}

/// `http`-backed token exchange against Google's token endpoint. Sends the PKCE
/// `code_verifier`, and the `client_secret` only when configured (Web client).
class HttpTokenExchanger implements OauthTokenExchanger {
  final String clientId;
  final String clientSecret;

  const HttpTokenExchanger({required this.clientId, this.clientSecret = ''});

  /// Bound on the token exchange so a stalled endpoint can't hang sign-in
  /// forever (the redirect wait has its own timeout; this leg needs one too).
  static const Duration _exchangeTimeout = Duration(seconds: 30);

  @override
  Future<String> exchangeCode({
    required String code,
    required String verifier,
    required String redirectUri,
  }) async {
    final http.Response response;
    try {
      response = await http
          .post(
            Uri.parse(kGoogleTokenEndpoint),
            body: <String, String>{
              'client_id': clientId,
              'code': code,
              'code_verifier': verifier,
              'grant_type': 'authorization_code',
              'redirect_uri': redirectUri,
              if (clientSecret.isNotEmpty) 'client_secret': clientSecret,
            },
          )
          .timeout(_exchangeTimeout);
    } on TimeoutException {
      throw const DesktopOauthException('token_exchange_timeout');
    }
    if (response.statusCode != HttpStatus.ok) {
      // The HTTP status + the `error` code (e.g. invalid_client when a Desktop
      // client's secret is missing) are safe to log — never the body's tokens.
      final error = _errorCode(response.body);
      throw DesktopOauthException(
        'token_exchange_http_${response.statusCode}${error == null ? '' : '_$error'}',
      );
    }
    final Map<String, dynamic> body;
    try {
      final decoded = jsonDecode(response.body);
      if (decoded is! Map<String, dynamic>) {
        throw const DesktopOauthException('token_exchange_bad_body');
      }
      body = decoded;
    } on FormatException {
      throw const DesktopOauthException('token_exchange_bad_body');
    }
    final idToken = body['id_token'] as String?;
    if (idToken == null || idToken.isEmpty) {
      throw const DesktopOauthException('token_exchange_no_id_token');
    }
    return idToken;
  }

  /// Best-effort, non-sensitive OAuth `error` code from an error response body.
  static String? _errorCode(String body) {
    try {
      final json = jsonDecode(body);
      if (json is Map<String, dynamic>) return json['error'] as String?;
    } on FormatException {
      // Non-JSON error body — nothing safe to extract.
    }
    return null;
  }
}

/// Wire a production [DesktopOidcTokenSource] from build-time config.
DesktopOidcTokenSource buildDesktopOidcTokenSource() => DesktopOidcTokenSource(
  clientId: kDesktopGoogleClientId,
  serverFactory: IoLoopbackServer.new,
  launch: (url) => launchUrl(url, mode: LaunchMode.externalApplication),
  exchanger: HttpTokenExchanger(
    clientId: kDesktopGoogleClientId,
    clientSecret: kDesktopGoogleClientSecret,
  ),
);
