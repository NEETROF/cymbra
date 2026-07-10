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

import 'package:music/services/preferences_service.dart';

/// In-memory [PreferencesService] for tests: no platform channels, deterministic.
///
/// Seed it via the [store] map to model a "previous launch", and inspect that
/// same map to assert what was persisted.
class FakePreferencesService implements PreferencesService {
  FakePreferencesService([Map<String, String>? initial])
    : store = {...?initial};

  /// The backing store — readable/writable by tests.
  final Map<String, String> store;

  @override
  Future<String?> getString(String key) async => store[key];

  @override
  Future<void> setString(String key, String value) async => store[key] = value;

  @override
  Future<void> remove(String key) async => store.remove(key);
}
