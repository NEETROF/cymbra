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

/// The client-owned feature-usage action taxonomy (change: add-feature-usage-
/// analytics, task 2.1, design D7).
///
/// The single source of truth for trackable action names — a shared constant, NOT
/// scattered string literals. An action only ever originates from an instrumented
/// call site here, so it can never reach the backend without a client release;
/// governance is this registry + code review. The `action` travels as a
/// shape-validated string (`^[a-z][a-z0-9_]{0,63}$`), so a new action ships with a
/// normal release without a coordinated backend deploy, and the back-office filter
/// list is derived from the actions actually seen in the data.
abstract final class UsageActions {
  // Auth.
  static const authSignIn = 'auth_sign_in';
  static const authSignUp = 'auth_sign_up';

  // Play (carry `subjectId` = the score id).
  static const playStart = 'play_start';
  static const playStop = 'play_stop';

  // Play mode switch (carries `variant` = the render mode).
  static const playModeSwitch = 'play_mode_switch';

  // Settings change (carries `variant` = the setting *category*, never the value).
  static const settingsChange = 'settings_change';

  // Scores (carry `subjectId` = the score id).
  static const scoreUpload = 'score_upload';
  static const scorePropose = 'score_propose';

  // SoundFonts.
  static const soundfontUpload = 'soundfont_upload';
  static const soundfontPropose = 'soundfont_propose';

  // Profiles.
  static const profileView = 'profile_view';

  // Favourites (carry `subjectId` = the score id).
  static const favoriteAdd = 'favorite_add';
  static const favoriteRemove = 'favorite_remove';

  // Catalog daily access (change: add-score-daily-access-rewards; carry
  // `subjectId` = the catalog id): the quota refused an open, a points day-slot
  // was bought, a locked piece's audio teaser was auditioned.
  static const catalogQuotaReached = 'catalog_quota_reached';
  static const catalogDaySlotUnlock = 'catalog_day_slot_unlock';
  static const catalogPreviewAudition = 'catalog_preview_audition';

  // NOTE: `guest_session_start` is deliberately DEFERRED (design D9): it is the
  // only action needing an unauthenticated ingress, so its slice ships later.

  /// The 12 in-scope actions (guest_session_start excluded). Exposed for tests and
  /// documentation; runtime emission never enumerates this — each call site names
  /// its action constant directly.
  static const all = <String>[
    authSignIn,
    authSignUp,
    playStart,
    playStop,
    playModeSwitch,
    settingsChange,
    scoreUpload,
    scorePropose,
    soundfontUpload,
    soundfontPropose,
    profileView,
    favoriteAdd,
    favoriteRemove,
    catalogQuotaReached,
    catalogDaySlotUnlock,
    catalogPreviewAudition,
  ];
}

/// Low-cardinality `variant` values (design D8). `play_mode_switch` records which
/// render mode; `settings_change` records the setting CATEGORY only — never the
/// chosen value — so "which settings people change" is measurable without
/// capturing what they chose.
abstract final class UsageVariants {
  // settings_change categories (the setting that changed, NOT its new value).
  static const pianoType = 'piano_type';
  static const hand = 'hand';
  static const tempo = 'tempo';
  static const metronome = 'metronome';
  static const language = 'language';
  static const readingAid = 'reading_aid';
}
