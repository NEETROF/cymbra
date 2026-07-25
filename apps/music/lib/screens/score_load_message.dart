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

import '../l10n/gen/app_localizations.dart';
import '../state/notation_data.dart';

/// The localized, user-facing message for a typed [ScoreLoadFailure]. Keeps the
/// specific cause (missing / not-ready / offline) readable without ever exposing
/// the raw exception or gRPC text.
String scoreLoadFailureMessage(
  AppLocalizations l10n,
  ScoreLoadFailure failure,
) => switch (failure) {
  ScoreLoadFailure.notFound => l10n.playerScoreNotFound,
  ScoreLoadFailure.notAvailableYet => l10n.playerScoreNotAvailableYet,
  ScoreLoadFailure.unavailable => l10n.playerScoreUnavailable,
  ScoreLoadFailure.generic => l10n.playerScoreLoadError,
};
