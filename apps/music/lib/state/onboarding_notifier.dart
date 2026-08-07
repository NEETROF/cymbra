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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'app_language.dart';
import 'app_locale.dart';

part 'onboarding_notifier.freezed.dart';
part 'onboarding_notifier.g.dart';

/// First-run flow state (change: add-welcome-onboarding, D1/D7).
///
/// [loaded] is false until the persisted flags have been read, so the gate shows
/// a neutral splash instead of flashing the language step at a returning user.
/// The two steps are independent one-time flags: [languageChosen] (the language
/// step, shown first) and [welcomeSeen] (the welcome, shown next).
///
/// [tryRunFinished] records that the no-account try produced an end-of-session
/// summary, so the welcome offers sign-in only after a *real* run. It is
/// session-scoped — deliberately not persisted.
@freezed
sealed class OnboardingState with _$OnboardingState {
  const OnboardingState._();

  const factory OnboardingState({
    @Default(false) bool loaded,
    @Default(false) bool languageChosen,
    @Default(false) bool welcomeSeen,
    @Default(false) bool tryRunFinished,
  }) = _OnboardingState;

  /// Whether the language step is the surface to show now.
  bool get needsLanguage => loaded && !languageChosen;

  /// Whether the welcome is the surface to show now (the language step first).
  bool get needsWelcome => loaded && languageChosen && !welcomeSeen;
}

/// Owns the first-run sequence: **language → welcome → (optional) sign-in →
/// handle gate → app**. Neither step requires an account, and the welcome is
/// always skippable — [completeWelcome] is called by *both* the skip and the
/// continue paths.
///
/// Both flags are persisted through the injectable [preferencesServiceProvider]
/// seam (a fake in tests, `shared_preferences` in production), so the first-run
/// surfaces appear exactly once per device.
@Riverpod(keepAlive: true)
class Onboarding extends _$Onboarding {
  /// Preferences key for "the language step has been completed".
  static const String languagePrefsKey = 'onboarding_language_chosen';

  /// Preferences key for "the welcome has been seen (or skipped)".
  static const String welcomePrefsKey = 'onboarding_welcome_seen';

  @override
  OnboardingState build() {
    _restore();
    return const OnboardingState();
  }

  Future<void> _restore() async {
    final prefs = ref.read(preferencesServiceProvider);
    String? language;
    String? welcome;
    String? existingLocale;
    try {
      language = await prefs.getString(languagePrefsKey);
      welcome = await prefs.getString(welcomePrefsKey);
      // A device that already carries a chosen UI language went through the
      // picker before this change existed — don't ask again on upgrade.
      existingLocale = await prefs.getString(AppLocale.prefsKey);
    } catch (_) {
      // Storage unavailable → treat both steps as done rather than trapping the
      // user in a first-run flow that can never be recorded as finished.
      state = const OnboardingState(
        loaded: true,
        languageChosen: true,
        welcomeSeen: true,
      );
      return;
    }
    state = OnboardingState(
      loaded: true,
      languageChosen: language == 'true' || existingLocale != null,
      welcomeSeen: welcome == 'true',
    );
  }

  /// Applies [language] immediately (so the welcome that follows is already in
  /// the user's language) and records the language step as done. Delegates the
  /// switch to [AppLocale] rather than duplicating persistence/sync here.
  Future<void> chooseLanguage(AppLanguage language) async {
    await ref.read(appLocaleProvider.notifier).select(language);
    if (state.languageChosen) return;
    state = state.copyWith(languageChosen: true);
    await _persist(languagePrefsKey);
  }

  /// Records the welcome as seen — from the skip action *and* from finishing it,
  /// so it never reappears either way.
  Future<void> completeWelcome() async {
    if (state.welcomeSeen) return;
    state = state.copyWith(welcomeSeen: true);
    await _persist(welcomePrefsKey);
  }

  /// Records that the no-account try finished a scored run (its summary was
  /// produced), so the welcome can offer sign-in when the player is left.
  void markTryRunFinished() => state = state.copyWith(tryRunFinished: true);

  /// Clears the try marker once the welcome has acted on it.
  void clearTryRun() => state = state.copyWith(tryRunFinished: false);

  /// Best-effort write: the in-memory flag already advanced the flow, so a
  /// storage failure costs at most one extra showing on the next launch.
  Future<void> _persist(String key) async {
    try {
      await ref.read(preferencesServiceProvider).setString(key, 'true');
    } catch (_) {}
  }
}
