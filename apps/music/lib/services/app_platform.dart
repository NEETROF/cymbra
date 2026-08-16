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

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'app_platform.g.dart';

/// The platform the build runs on, as the plan system needs it (change:
/// add-premium-subscription): it decides which purchase channel the server may
/// offer — store builds their store, desktop/web the merchant of record.
enum AppPlatform { ios, macos, android, linux, windows, web }

extension AppPlatformX on AppPlatform {
  /// True for the App Store / Play builds — the ones that must never show an
  /// external purchase link nor a code entry (store rules).
  bool get isStoreBuild =>
      this == AppPlatform.ios ||
      this == AppPlatform.macos ||
      this == AppPlatform.android;

  /// True where a purchase goes through the web checkout in the browser.
  bool get usesWebCheckout =>
      this == AppPlatform.linux ||
      this == AppPlatform.windows ||
      this == AppPlatform.web;
}

/// The running platform, behind a provider (an injectable seam) so widget tests
/// drive every platform deterministically. The `kIsWeb` guard keeps `dart:io`
/// off the web path.
@riverpod
AppPlatform appPlatform(Ref ref) {
  if (kIsWeb) return AppPlatform.web;
  if (Platform.isIOS) return AppPlatform.ios;
  if (Platform.isMacOS) return AppPlatform.macos;
  if (Platform.isAndroid) return AppPlatform.android;
  if (Platform.isWindows) return AppPlatform.windows;
  return AppPlatform.linux;
}
