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

part 'drums_access.g.dart';

/// The server-evaluated **`drums.enabled`** feature flag key (change:
/// add-drums-access) — one constant so the provider override in `main.dart`
/// and any consumer name the same string.
const String kDrumsEnabledFlag = 'drums.enabled';

/// The typed drum-gate refusal code (change: add-drums-access): the local
/// upload refusal's reject code, and the prefix of the backend's
/// `PERMISSION_DENIED` message for the same refusal — both localize to the
/// same message in the UI, never a raw technical string.
const String kDrumsNotAvailableCode = 'drums_not_available';

/// Whether the drum feature is visible to this caller (`drums.enabled`,
/// evaluated server-side — staff + the `beta:midi-drums` campaign). Plain
/// `false` by default so tests never build the flag client; `main.dart`
/// overrides it with the remote flag. Hiding here is defence in depth only —
/// the backend independently enforces the drum audience on every path.
@Riverpod(keepAlive: true)
bool drumsEnabled(Ref ref) => false;
