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

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show rootBundle;

/// Path of the generated Rust third-party notices asset (change:
/// add-oss-license-attributions), produced by `melos run gen-licenses` /
/// `apps/music/tool/gen_licenses.sh`.
const String rustLicenseNoticesAssetPath =
    'assets/generated/rust_licenses.json';

/// Parses the generated Rust notices JSON into [LicenseEntry] objects for
/// [LicenseRegistry]. Pure data transform — no asset I/O — so it's
/// host-testable without a widget/asset bundle. Malformed input degrades to
/// an empty list rather than throwing, so one bad entry can't blank the
/// whole license page.
List<LicenseEntry> parseRustLicenseNotices(String jsonString) {
  final Object? decoded;
  try {
    decoded = jsonDecode(jsonString);
  } on FormatException {
    return const [];
  }
  if (decoded is! Map<String, dynamic>) return const [];
  final licenses = decoded['licenses'];
  if (licenses is! List) return const [];

  final entries = <LicenseEntry>[];
  for (final raw in licenses) {
    if (raw is! Map<String, dynamic>) continue;
    final text = raw['text'];
    final packages = raw['packages'];
    if (text is! String || packages is! List) continue;
    final packageNames = packages.whereType<String>().toList();
    if (packageNames.isEmpty) continue;
    entries.add(LicenseEntryWithLineBreaks(packageNames, text));
  }
  return entries;
}

/// Registers the Rust third-party notices into Flutter's [LicenseRegistry],
/// alongside the Dart/Flutter pub package licenses it already collects
/// automatically. Call once at startup, before `runApp`.
///
/// Degrades gracefully: a missing or unreadable asset (e.g. a dev build that
/// skipped `melos run gen-licenses`) yields no Rust entries instead of
/// throwing — the Dart/Flutter license list still shows.
void registerRustLicenseNotices() {
  LicenseRegistry.addLicense(_collectRustLicenseNotices);
}

Stream<LicenseEntry> _collectRustLicenseNotices() async* {
  final String jsonString;
  try {
    jsonString = await rootBundle.loadString(rustLicenseNoticesAssetPath);
  } catch (_) {
    return;
  }
  for (final entry in parseRustLicenseNotices(jsonString)) {
    yield entry;
  }
}
