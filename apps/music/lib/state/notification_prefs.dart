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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/notification_service.dart';
import 'push_categories.dart';

part 'notification_prefs.g.dart';

/// The caller's per-category notification preferences (change:
/// add-push-notifications, task 4.4).
///
/// The server is the source of truth (the same value the send job reads), so the
/// switches reflect what will actually happen — not a local mirror. A load
/// failure resolves to empty settings rather than an error state: the user then
/// sees each category's default and a toggle still works.
@Riverpod(keepAlive: true)
class NotificationPrefs extends _$NotificationPrefs {
  @override
  Future<NotificationSettings> build() async {
    try {
      return await ref.read(notificationRegistryServiceProvider).settings();
    } catch (_) {
      // Offline / signed out: show the defaults instead of an error.
      return NotificationSettings.empty;
    }
  }

  /// Whether [category] is currently on, falling back to its declared default
  /// when the user has never chosen.
  bool isEnabled(PushCategory category) =>
      state.value?.prefs[category.id] ?? category.defaultEnabled;

  /// Set the caller's choice for [category].
  ///
  /// Optimistic: the switch flips immediately and reverts if the server refuses,
  /// so the UI never waits on the network and never shows a raw error.
  Future<void> setEnabled(PushCategory category, bool enabled) async {
    final current = state.value ?? NotificationSettings.empty;
    state = AsyncData(
      NotificationSettings(
        prefs: {...current.prefs, category.id: enabled},
        timezone: current.timezone,
        hasRegisteredDevice: current.hasRegisteredDevice,
      ),
    );
    try {
      await ref
          .read(notificationRegistryServiceProvider)
          .setPref(category: category.id, enabled: enabled);
    } catch (_) {
      state = AsyncData(current);
    }
  }
}
