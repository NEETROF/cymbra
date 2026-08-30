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

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/update/update_manifest.dart';

/// The Dart half of the cross-language golden-fixture test (task 9.2).
///
/// `crates/update-manifest/tests/golden.rs` reads EXACTLY these files and
/// asserts exactly these outcomes. Ed25519 and base64 are re-implemented
/// independently on each side, which is precisely where a drift would hide — so
/// a disagreement fails here rather than on a user's machine.
const _fixtures = '../../crates/update-manifest/fixtures';

String _read(String name) => File('$_fixtures/$name').readAsStringSync();

Map<String, String> _trustedKeys() {
  final raw = jsonDecode(_read('trusted_keys.json')) as Map<String, dynamic>;
  return raw.map((k, v) => MapEntry(k, v as String));
}

Future<UpdateVerifyResult> _verify(String fixture) =>
    verifyUpdateEnvelope(_read(fixture), _trustedKeys());

void main() {
  test('the fixture is present (regenerate with the gen_fixtures example)', () {
    expect(Directory(_fixtures).existsSync(), isTrue, reason: _fixtures);
  });

  test('a valid envelope verifies and carries both targets', () async {
    final result = await _verify('envelope_valid.json');
    expect(result, isA<UpdateVerified>());
    final verified = result as UpdateVerified;
    final manifest = verified.manifest;
    expect(manifest.schema, kUpdateSchemaVersion);
    expect(manifest.product, 'music');
    expect(manifest.channel, 'stable');
    expect(manifest.version.toString(), '1.25.0+34');
    expect(manifest.minSupportedVersion.toString(), '1.20.0+27');
    expect(manifest.notesUrl, isNotNull);
    expect(verified.rolloutPercent, 25);

    final windows = manifest.targetFor('windows-x64')!;
    expect(windows.kind, 'inno-setup');
    expect(windows.size, 48123904);
    expect(windows.sha256.length, 64);
    expect(manifest.targetFor('linux-x64')!.kind, 'appimage');
    // A platform with no artifact is simply absent — not an error.
    expect(manifest.targetFor('macos-arm64'), isNull);
  });

  test('tampered manifest bytes are refused', () async {
    // Valid JSON, a different version, the original signature: exactly the
    // attack the sign-the-exact-bytes design exists to defeat.
    expect(
      await _verify('envelope_tampered_manifest.json'),
      isA<UpdateRefused>().having(
        (r) => r.failure,
        'failure',
        UpdateVerifyFailure.badSignature,
      ),
    );
  });

  test('a tampered signature is refused', () async {
    expect(
      await _verify('envelope_tampered_signature.json'),
      isA<UpdateRefused>().having(
        (r) => r.failure,
        'failure',
        UpdateVerifyFailure.badSignature,
      ),
    );
  });

  test('an unknown key id is refused', () async {
    expect(
      await _verify('envelope_unknown_key_id.json'),
      isA<UpdateRefused>().having(
        (r) => r.failure,
        'failure',
        UpdateVerifyFailure.unknownKeyId,
      ),
    );
  });

  test(
    'a future schema is refused even though the signature is good',
    () async {
      expect(
        await _verify('envelope_unknown_schema.json'),
        isA<UpdateRefused>().having(
          (r) => r.failure,
          'failure',
          UpdateVerifyFailure.unsupportedSchema,
        ),
      );
    },
  );

  test('rollout 0 still verifies — the gate is client-side', () async {
    final result = await _verify('envelope_rollout_zero.json');
    expect(result, isA<UpdateVerified>());
    expect((result as UpdateVerified).rolloutPercent, 0);
  });

  group('malformed input', () {
    Future<UpdateVerifyResult> verify(String body) =>
        verifyUpdateEnvelope(body, _trustedKeys());

    test('a non-JSON body is refused', () async {
      expect(
        await verify('not json at all'),
        isA<UpdateRefused>().having(
          (r) => r.failure,
          'failure',
          UpdateVerifyFailure.malformed,
        ),
      );
    });

    test('an empty body is refused (a 200 with nothing in it)', () async {
      expect(await verify(''), isA<UpdateRefused>());
    });

    test('missing envelope fields are refused', () async {
      expect(
        await verify('{"manifest":"e30=","key_id":"golden-1"}'),
        isA<UpdateRefused>().having(
          (r) => r.failure,
          'failure',
          UpdateVerifyFailure.malformed,
        ),
      );
    });

    test('non-base64 fields are refused', () async {
      final valid = jsonDecode(_read('envelope_valid.json')) as Map;
      final broken = {...valid, 'manifest': '!!! not base64 !!!'};
      expect(
        await verify(jsonEncode(broken)),
        isA<UpdateRefused>().having(
          (r) => r.failure,
          'failure',
          UpdateVerifyFailure.malformed,
        ),
      );
    });

    test('a signature of the wrong length is refused', () async {
      final valid = jsonDecode(_read('envelope_valid.json')) as Map;
      final broken = {...valid, 'signature': base64.encode(List.filled(8, 0))};
      expect(
        await verify(jsonEncode(broken)),
        isA<UpdateRefused>().having(
          (r) => r.failure,
          'failure',
          UpdateVerifyFailure.malformedSignature,
        ),
      );
    });

    test('an empty trusted-key set refuses everything', () async {
      expect(
        await verifyUpdateEnvelope(_read('envelope_valid.json'), const {}),
        isA<UpdateRefused>().having(
          (r) => r.failure,
          'failure',
          UpdateVerifyFailure.unknownKeyId,
        ),
      );
    });

    test('a missing rollout_percent reads as 0, never as everyone', () async {
      final valid =
          jsonDecode(_read('envelope_valid.json')) as Map<String, dynamic>;
      valid.remove('rollout_percent');
      final result = await verify(jsonEncode(valid));
      expect((result as UpdateVerified).rolloutPercent, 0);
    });
  });
}
