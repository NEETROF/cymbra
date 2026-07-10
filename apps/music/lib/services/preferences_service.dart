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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

part 'preferences_service.g.dart';

/// Production preferences store provider. Override in tests with an in-memory
/// [FakePreferencesService] so state/widgets never touch native storage.
@Riverpod(keepAlive: true)
PreferencesService preferencesService(Ref ref) => SharedPreferencesService();

/// Seam over on-device key-value storage for small, non-secret user
/// preferences (starting with the selected UI language).
///
/// Deliberately separate from `flutter_secure_storage`, which is reserved for
/// auth tokens: preferences are plain, low-stakes values that do not warrant the
/// keychain. State depends on this interface rather than `SharedPreferences`
/// directly, so tests inject a fake without initializing platform channels.
abstract class PreferencesService {
  /// Returns the value stored under [key], or `null` if it was never written.
  Future<String?> getString(String key);

  /// Persists [value] under [key], replacing any previous value.
  Future<void> setString(String key, String value);

  /// Removes any value stored under [key] (a no-op if absent).
  Future<void> remove(String key);
}

/// Production [PreferencesService] backed by `shared_preferences`.
///
/// The `SharedPreferences` instance is fetched lazily and memoized, so the first
/// access pays the one-time platform-channel round-trip and later calls reuse it.
class SharedPreferencesService implements PreferencesService {
  Future<SharedPreferences>? _prefs;

  Future<SharedPreferences> get _instance =>
      _prefs ??= SharedPreferences.getInstance();

  @override
  Future<String?> getString(String key) async =>
      (await _instance).getString(key);

  @override
  Future<void> setString(String key, String value) async =>
      (await _instance).setString(key, value);

  @override
  Future<void> remove(String key) async => (await _instance).remove(key);
}
