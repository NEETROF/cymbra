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

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/push_service.dart';
import 'foreground_notification_state.dart';

part 'foreground_notification.g.dart';

/// The `data` key carrying the category's foreground decision, stamped by the
/// backend dispatcher (`cymbra-notifications`' `FOREGROUND_DATA_KEY` — keep the
/// two in sync). `"true"` = surface the message in-app; anything else, including
/// absence, = silence.
const String kPushForegroundDataKey = 'foreground';

/// The `data` key naming a tapped notification's destination (design D4) — the
/// same routing payload a background tap uses, so a category describes its
/// destination once.
const String kPushRouteDataKey = 'route';

/// Messages delivered while the app is in the foreground, bridged from the
/// [PushService] seam so widgets and notifiers never touch the SDK. On an
/// unconfigured build the underlying stream is empty — nothing ever fires.
@riverpod
Stream<PushForegroundMessage> foregroundMessages(Ref ref) =>
    ref.watch(pushServiceProvider).foregroundMessages;

/// Destinations a foreground notification's routing payload can point at.
///
/// **Empty by design**, like `pushCategories`: the feature that declares a
/// notification category registers its destination here, keyed by the value its
/// dispatch job puts under [kPushRouteDataKey] (e.g. `/practice`). A tapped
/// banner whose route is absent from this map simply dismisses — an older build
/// receiving a newer category degrades to silence, never to a crash.
@riverpod
Map<String, WidgetBuilder> pushRouteBuilders(Ref ref) =>
    const <String, WidgetBuilder>{};

/// Owns the visible in-app banner (change: add-foreground-notifications).
///
/// The decision to surface is read off the **message itself** — the
/// dispatcher-stamped [kPushForegroundDataKey] — never off a compiled-in
/// category list, which is what keeps the policy hot-reloadable: a category
/// this build has never heard of still behaves exactly as the back office says.
@riverpod
class ForegroundNotification extends _$ForegroundNotification {
  @override
  ForegroundNotificationState build() => const ForegroundNotificationState();

  /// Hand over one delivered foreground message. Surfaced only if it carries
  /// the foreground indication; absent or anything but `"true"` ⇒ silence. A
  /// new banner replaces the current one.
  void show(PushForegroundMessage message) {
    if (message.data[kPushForegroundDataKey] != 'true') return;
    state = state.copyWith(banner: message);
  }

  /// The user dismissed the banner; nothing further surfaces.
  void dismiss() => state = state.copyWith(banner: null);

  /// The user tapped the banner: it dismisses, and its routing payload (if
  /// any) is surfaced for the listener widget to navigate on. A payload-less
  /// message just dismisses (design D4).
  void tap() {
    final route = state.banner?.data[kPushRouteDataKey];
    state = ForegroundNotificationState(
      tappedRoute: (route == null || route.isEmpty) ? null : route,
    );
  }

  /// The listener consumed [ForegroundNotificationState.tappedRoute].
  void routeHandled() => state = state.copyWith(tappedRoute: null);
}
