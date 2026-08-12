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

import '../services/push_service.dart';

part 'foreground_notification_state.freezed.dart';

/// State of the in-app foreground-notification surface (change:
/// add-foreground-notifications).
///
/// [banner] is the message currently on screen, or null when nothing is
/// surfaced — the notifier only ever sets it from a message that *carries* the
/// foreground indication. [tappedRoute] is the routing payload of a banner the
/// user just tapped, pending consumption by the listener widget (which
/// navigates, then clears it); it is transient hand-off state, not something a
/// widget renders.
@freezed
sealed class ForegroundNotificationState with _$ForegroundNotificationState {
  const factory ForegroundNotificationState({
    PushForegroundMessage? banner,
    String? tappedRoute,
  }) = _ForegroundNotificationState;
}
