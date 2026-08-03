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

import '../services/preferences_service.dart';

part 'usage_consent.g.dart';

/// The user's feature-usage-collection consent (change: add-feature-usage-
/// analytics, tasks 6.4 + 8.1).
///
/// Collection defaults to **opt-out** (enabled) under a first-party audience-
/// measurement posture: a user who has never touched the setting is opted in. The
/// choice persists via the injectable [PreferencesService] seam. This per-user
/// consent is distinct from the global `analytics.collection.enabled` kill-switch —
/// either one being off suppresses emission.
@Riverpod(keepAlive: true)
class UsageConsent extends _$UsageConsent {
  /// Preferences key under which the consent boolean lives.
  static const String prefsKey = 'usageConsent';

  @override
  bool build() {
    // Seed to the opt-out default; reconcile the persisted choice once it loads
    // (never block startup, mirror the app's other stores).
    _restore();
    return true;
  }

  Future<void> _restore() async {
    try {
      final stored = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      if (stored == 'true' || stored == 'false') {
        state = stored == 'true';
      }
    } catch (_) {
      // Storage unavailable: keep the default-on posture.
    }
  }

  /// Enable or disable collection for this user, persisting the choice. The switch
  /// applies even if persistence fails, so the UI never gets stuck.
  Future<void> set(bool enabled) async {
    state = enabled;
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, enabled ? 'true' : 'false');
    } catch (_) {}
  }
}
