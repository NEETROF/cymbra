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
import '../state/coaching_notifier.dart';
import 'coach_mark.dart' show CoachAnchor;

/// Localized copy of a coaching hint or guided step.
typedef CoachCopy = ({String title, String body});

/// The copy for a one-time hint. Single source of truth: the hint itself, and
/// the help surface that lets a user re-read it, use the same strings.
CoachCopy coachHintCopy(AppLocalizations l10n, CoachHint hint) =>
    switch (hint) {
      CoachHint.ratingDeck => (
        title: l10n.ratingCoachTitle,
        body: l10n.ratingCoachBody,
      ),
      CoachHint.rewards => (
        title: l10n.coachRewardsTitle,
        body: l10n.coachRewardsBody,
      ),
      CoachHint.goingPublic => (
        title: l10n.coachGoingPublicTitle,
        body: l10n.coachGoingPublicBody,
      ),
      CoachHint.playerTour => (
        title: l10n.coachPlayerTourTitle,
        body: l10n.coachPlayerTourBody,
      ),
    };

/// The copy for one step of the guided player sequence.
CoachCopy playerCoachCopy(AppLocalizations l10n, PlayerCoachStep step) =>
    switch (step) {
      PlayerCoachStep.pianoSound => (
        title: l10n.coachPlayerSoundTitle,
        body: l10n.coachPlayerSoundBody,
      ),
      PlayerCoachStep.midiDevice => (
        title: l10n.coachPlayerMidiTitle,
        body: l10n.coachPlayerMidiBody,
      ),
      PlayerCoachStep.hands => (
        title: l10n.coachPlayerHandsTitle,
        body: l10n.coachPlayerHandsBody,
      ),
    };

/// The control each guided step points at.
extension PlayerCoachStepAnchor on PlayerCoachStep {
  CoachAnchor get anchor => switch (this) {
    PlayerCoachStep.pianoSound => CoachAnchor.pianoSound,
    PlayerCoachStep.midiDevice => CoachAnchor.midiDevice,
    PlayerCoachStep.hands => CoachAnchor.hands,
  };
}
