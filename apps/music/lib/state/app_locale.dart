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

import 'dart:async';
import 'dart:ui' show Locale, PlatformDispatcher;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/grpc_client.dart';
import '../services/preferences_service.dart';
import 'app_language.dart';
import 'session_notifier.dart';
import 'usage_tracking_notifier.dart';

part 'app_locale.g.dart';

/// The device's current locale, behind a provider so tests can drive the
/// default-language behaviour deterministically (the VM reports the host locale
/// otherwise). Read once at startup by [AppLocale].
@riverpod
Locale deviceLocale(Ref ref) => PlatformDispatcher.instance.locale;

/// The active UI locale that drives `MaterialApp.locale`.
///
/// The state is always a concrete supported `Locale`. It is seeded
/// synchronously from the device locale (so the first frame is already
/// correct-by-default) and then reconciled against the persisted choice, which
/// loads asynchronously. Selecting a language updates the state — rebuilding
/// `MaterialApp` for an immediate, restart-free switch — and persists the code.
///
/// Resolution precedence is specified by [resolveLanguage]: a supported
/// persisted choice wins, else the device language, else English.
@Riverpod(keepAlive: true)
class AppLocale extends _$AppLocale {
  /// Preferences key under which the selected language [AppLanguage.code] lives.
  static const String prefsKey = 'app_language';

  @override
  Locale build() {
    final device = ref.watch(deviceLocaleProvider);
    // Load the persisted override (if any) and reconcile once it arrives; until
    // then, fall back to the device-derived language so nothing blocks startup.
    _restore(device);
    return resolveLanguage(deviceLocale: device).locale;
  }

  Future<void> _restore(Locale device) async {
    final prefs = ref.read(preferencesServiceProvider);
    // Storage is best-effort: if it is unavailable, keep the device-derived
    // default rather than failing (mirrors the app's other stores).
    String? storedCode;
    try {
      storedCode = await prefs.getString(prefsKey);
    } catch (_) {
      return;
    }
    final resolved = resolveLanguage(
      persistedCode: storedCode,
      deviceLocale: device,
    );
    state = resolved.locale;
    // Self-heal: a stored value that is no longer supported resolves to the
    // fallback above; re-persist it so storage matches what is shown.
    if (storedCode != null && AppLanguage.fromCode(storedCode) == null) {
      try {
        await prefs.setString(prefsKey, resolved.code);
      } catch (_) {}
    }
  }

  /// Switches the UI to [language] immediately, persists the choice, and — when a
  /// session is authenticated — records it on the account (change: sync-account-
  /// language-preference). The switch is applied even if persistence or the sync
  /// fails, so the UI never gets stuck.
  Future<void> select(AppLanguage language) async {
    state = language.locale;
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, language.code);
    } catch (_) {}
    // Usage telemetry (change: add-feature-usage-analytics): the language *setting*
    // was changed (category only — the chosen language is never recorded).
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(UsageActions.settingsChange, variant: UsageVariants.language),
    );
    await _pushLocale(language.code);
  }

  /// Applies [language] as the active UI language and persists it locally
  /// **without** pushing it back to the account (change: sync-account-language-
  /// preference). The reconcile path uses this so applying the server's value
  /// never echoes a redundant `SetLocale`.
  Future<void> applyFromAccount(AppLanguage language) async {
    state = language.locale;
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, language.code);
    } catch (_) {}
  }

  /// Reconciles the account's stored language into the UI after sign-in (change:
  /// sync-account-language-preference, design D4):
  /// - a set, displayable server locale wins → apply + persist locally (no echo);
  /// - an unset (`null`/empty) server locale → keep the local choice and push it up;
  /// - a set but undisplayable server locale → leave the UI and the stored value.
  Future<void> syncOnLogin(String? serverLocale) async {
    if (serverLocale == null || serverLocale.isEmpty) {
      await _pushLocale(state.languageCode);
      return;
    }
    final language = AppLanguage.fromCode(serverLocale);
    if (language == null) return; // not displayable here — leave both untouched
    await applyFromAccount(language);
  }

  /// Best-effort push of [code] to the account: a no-op when the session cannot
  /// use online services (guest / signed out / not yet resolved), and swallows
  /// failures so a sync never blocks or breaks the local switch.
  Future<void> _pushLocale(String code) async {
    if (!ref.read(canUseOnlineServicesProvider)) return;
    try {
      await ref.read(accountServiceProvider).setLocale(code);
    } catch (_) {}
  }
}
