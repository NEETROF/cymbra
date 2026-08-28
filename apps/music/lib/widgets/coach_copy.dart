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
      CoachHint.notationHelp => (
        title: l10n.notationHelpHintTitle,
        body: l10n.notationHelpHintBody,
      ),
    };

/// The copy for one step of the guided player sequence.
///
/// The tour points at the SAME controls on a percussion score, but three of
/// them mean something else there: the sound is a kit, the connected device is
/// an e-kit rather than a MIDI keyboard, and the selector splits hands from
/// FEET, not right hand from left. A first-time drummer was being taught the
/// piano's vocabulary while looking at "Pieds / Mains / Les deux" — the tour
/// contradicted the control it was pointing at. The rewind step is
/// instrument-neutral and is shared.
CoachCopy playerCoachCopy(
  AppLocalizations l10n,
  PlayerCoachStep step, {
  bool percussion = false,
}) => switch (step) {
  PlayerCoachStep.pianoSound => (
    title: percussion
        ? l10n.coachPlayerSoundDrumsTitle
        : l10n.coachPlayerSoundTitle,
    body: percussion
        ? l10n.coachPlayerSoundDrumsBody
        : l10n.coachPlayerSoundBody,
  ),
  PlayerCoachStep.midiDevice => (
    title: l10n.coachPlayerMidiTitle,
    body: percussion ? l10n.coachPlayerMidiDrumsBody : l10n.coachPlayerMidiBody,
  ),
  // Step 3 points at whichever control isolates part of the piece: the hand
  // selector on a keyboard, the per-piece focus list on a kit (change:
  // add-practice-focus-controls). Same position, same anchor, same purpose —
  // only the grain differs, which is exactly what the copy has to say.
  PlayerCoachStep.hands => (
    title: percussion ? l10n.coachPlayerFocusTitle : l10n.coachPlayerHandsTitle,
    body: percussion ? l10n.coachPlayerFocusBody : l10n.coachPlayerHandsBody,
  ),
  PlayerCoachStep.measureRewind => (
    title: l10n.coachPlayerRewindTitle,
    body: l10n.coachPlayerRewindBody,
  ),
};

/// The control each guided step points at.
extension PlayerCoachStepAnchor on PlayerCoachStep {
  CoachAnchor get anchor => switch (this) {
    PlayerCoachStep.pianoSound => CoachAnchor.pianoSound,
    PlayerCoachStep.midiDevice => CoachAnchor.midiDevice,
    PlayerCoachStep.hands => CoachAnchor.hands,
    PlayerCoachStep.measureRewind => CoachAnchor.measureRewind,
  };
}
