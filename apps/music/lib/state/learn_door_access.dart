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

import 'package:cymbra_flags/cymbra_flags.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'learn_door_access.g.dart';

/// The server-evaluated **`music.learn_door.enabled`** feature flag key
/// (change: add-music-learn-play-home) — one constant so the provider override
/// in `main.dart` and any consumer name the same string.
const String kLearnDoorEnabledFlag = 'music.learn_door.enabled';

/// Whether the fetched flags show the two-door home (Play / Learn). A
/// kill-switch, not a rollout gate: it defaults **on**, so a snapshot that does
/// not carry the key yet — before the first fetch, or offline with no cache —
/// shows the doors, and only an explicit `false` from the server folds the home
/// back into its single body (courses card above the favorites).
bool learnDoorEnabledIn(FlagSnapshot flags) =>
    flags.getBool(kLearnDoorEnabledFlag, or: true);

/// Whether the two-door home is shown to this caller. Plain `true` by default
/// (the shipped behaviour, and tests never build the flag client); `main.dart`
/// overrides it with [learnDoorEnabledIn] over the remote flags.
@Riverpod(keepAlive: true)
bool learnDoorEnabled(Ref ref) => true;
