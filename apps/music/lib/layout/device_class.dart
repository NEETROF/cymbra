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

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// Landscape viewport size classes.
///
/// The app is landscape-locked (see the keyboard-display spec), so the
/// viewport's *shortest side* is its height — the dimension under pressure on a
/// phone — and classification keys off it. This is the single source of truth
/// for size-adaptive layout; widgets read it via [BuildContext.deviceClass]
/// instead of recomputing breakpoints inline.
enum DeviceClass { phone, tablet, desktop }

/// Shortest side (logical px) at or above which a viewport is no longer a phone.
const double kPhoneMaxShortestSide = 600;

/// Shortest side (logical px) at or above which a viewport is a desktop.
const double kTabletMaxShortestSide = 900;

/// Pure size → class mapping (no platform awareness); the testable core.
DeviceClass deviceClassForShortestSide(double shortestSide) {
  if (shortestSide < kPhoneMaxShortestSide) return DeviceClass.phone;
  if (shortestSide < kTabletMaxShortestSide) return DeviceClass.tablet;
  return DeviceClass.desktop;
}

/// Whether the current platform is a desktop OS or the web, which always
/// classify as [DeviceClass.desktop] regardless of window size.
bool get isDesktopOrWebPlatform {
  if (kIsWeb) return true;
  switch (defaultTargetPlatform) {
    case TargetPlatform.macOS:
    case TargetPlatform.linux:
    case TargetPlatform.windows:
      return true;
    case TargetPlatform.android:
    case TargetPlatform.iOS:
    case TargetPlatform.fuchsia:
      return false;
  }
}

/// Resolves the [DeviceClass] for [context]: desktop/web platforms are always
/// [DeviceClass.desktop]; otherwise the viewport's shortest side decides.
DeviceClass deviceClassOf(BuildContext context) {
  if (isDesktopOrWebPlatform) return DeviceClass.desktop;
  return deviceClassForShortestSide(MediaQuery.sizeOf(context).shortestSide);
}

extension DeviceClassContext on BuildContext {
  /// The resolved device class for this build context. See [deviceClassOf].
  DeviceClass get deviceClass => deviceClassOf(this);

  /// True on the smallest supported form factor (smartphone landscape).
  bool get isPhoneLayout => deviceClass == DeviceClass.phone;
}
