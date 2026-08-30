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

part 'acoustic_input_access.g.dart';

/// The server-evaluated **`acoustic_input.enabled`** feature flag key (change:
/// add-acoustic-piano-input) — one constant so the provider override in
/// `main.dart` and any consumer name the same string.
const String kAcousticInputEnabledFlag = 'acoustic_input.enabled';

/// Whether the microphone input source is visible to this caller
/// (`acoustic_input.enabled`, evaluated server-side — staff + the beta
/// campaign). Plain `false` by default so tests never build the flag client;
/// `main.dart` overrides it with the remote flag. While false, no microphone
/// surface exists anywhere and the microphone permission is never requested.
@Riverpod(keepAlive: true)
bool acousticInputEnabled(Ref ref) => false;
