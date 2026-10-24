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
import 'package:music/services/update/update_signing_keys.dart';

void main() {
  // The Linux privileged helper compiles in `release-keys.json`; the app
  // compiles in `kUpdateTrustedKeys`. If they drift, the app accepts a release
  // that the helper then refuses to install (or the reverse) — fail here, not
  // on a user's machine (change: add-desktop-auto-update, task 7.12).
  test('the app trusts exactly the keys the Linux update helper trusts', () {
    final helperKeys =
        (jsonDecode(
                  File(
                    '../../crates/update-manifest/release-keys.json',
                  ).readAsStringSync(),
                )
                as Map<String, dynamic>)
            .cast<String, String>();
    expect(kUpdateTrustedKeys, equals(helperKeys));
  });
}
