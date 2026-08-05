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

import 'dart:ui' show FlutterView;

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../layout/device_class.dart';

part 'usage_environment.g.dart';

/// The originating `platform` for a usage event — one of the six shipped targets
/// the backend accepts. Pure mapping from [defaultTargetPlatform] / [kIsWeb].
String usagePlatform() {
  if (kIsWeb) return 'web';
  switch (defaultTargetPlatform) {
    case TargetPlatform.iOS:
      return 'ios';
    case TargetPlatform.android:
      return 'android';
    case TargetPlatform.macOS:
      return 'macos';
    case TargetPlatform.windows:
      return 'windows';
    case TargetPlatform.linux:
      return 'linux';
    case TargetPlatform.fuchsia:
      // Not a shipped target; classify as its closest accepted platform.
      return 'android';
  }
}

/// The `device_class` (`phone` / `tablet` / `desktop`) derived on-device without a
/// [BuildContext]: desktop/web is always `desktop`; otherwise the primary view's
/// shortest side decides via the shared [deviceClassForShortestSide] breakpoints.
/// [view] defaults to the implicit view (injectable for tests).
String usageDeviceClass({FlutterView? view}) {
  if (isDesktopOrWebPlatform) return DeviceClass.desktop.name;
  try {
    final v = view ?? WidgetsBinding.instance.platformDispatcher.implicitView;
    if (v == null) return DeviceClass.phone.name; // headless / not yet attached
    final size = v.physicalSize / v.devicePixelRatio;
    return deviceClassForShortestSide(size.shortestSide).name;
  } catch (_) {
    // No initialised binding (e.g. a pure unit test): fall back to phone.
    return DeviceClass.phone.name;
  }
}

/// Seam for the app version string (e.g. `1.17.0`). Behind a provider so state is
/// testable without the native `package_info_plus` plugin.
abstract class AppInfoService {
  Future<String> version();
}

/// Production [AppInfoService] over `package_info_plus`.
class PackageInfoAppInfoService implements AppInfoService {
  const PackageInfoAppInfoService();

  @override
  Future<String> version() async => (await PackageInfo.fromPlatform()).version;
}

@Riverpod(keepAlive: true)
AppInfoService appInfoService(Ref ref) => const PackageInfoAppInfoService();
