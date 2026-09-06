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
import '../state/drum_kit.dart';

/// Localised label of a kit piece (change: add-drum-kit-view), used by the
/// pad strip and the lane labels. A generic (terminal-bucket) piece carries
/// its General MIDI instrument name — a notation-level vocabulary that is not
/// translated.
String kitPieceLabel(AppLocalizations l10n, DrumLane lane) =>
    switch (lane.labelKey) {
      'kitPieceHiHat' => l10n.kitPieceHiHat,
      'kitPieceSnare' => l10n.kitPieceSnare,
      'kitPieceTomHigh' => l10n.kitPieceTomHigh,
      'kitPieceTomHiMid' => l10n.kitPieceTomHiMid,
      'kitPieceTomLowMid' => l10n.kitPieceTomLowMid,
      'kitPieceTomLow' => l10n.kitPieceTomLow,
      'kitPieceTomFloorHigh' => l10n.kitPieceTomFloorHigh,
      'kitPieceTomFloorLow' => l10n.kitPieceTomFloorLow,
      'kitPieceRide' => l10n.kitPieceRide,
      'kitPieceCrash' => l10n.kitPieceCrash,
      'kitPieceCrash2' => l10n.kitPieceCrash2,
      'kitPieceSplash' => l10n.kitPieceSplash,
      'kitPieceChina' => l10n.kitPieceChina,
      _ =>
        (lane.gmNumbers.length == 1
                ? gmPieceLabel(l10n, lane.gmNumbers.first)
                : null) ??
            lane.gmName ??
            '',
    };

/// Localised name for a General MIDI percussion number the app names in its own
/// voice (change: add-drum-input-calibration): the **zones** of a kit piece a
/// module triggers separately — the cross-stick, the open hi-hat, the pedal
/// "chick", the ride bell — and the auxiliary pads the calibration pass offers.
/// Null for every other number, which keeps its General MIDI name: those are a
/// notation-level vocabulary and are not translated.
///
/// One table for both surfaces that name a piece to the player — the pad strip
/// and the calibration pass — so the prompt a drummer is asked ("hit your
/// hi-hat pedal") and the pad that lights when they do cannot read differently.
String? gmPieceLabel(AppLocalizations l10n, int gm) => switch (gm) {
  37 => l10n.kitZoneCrossStick,
  39 => l10n.kitPieceHandClap,
  44 => l10n.kitZoneHiHatPedal,
  46 => l10n.kitZoneHiHatOpen,
  53 => l10n.kitZoneRideBell,
  54 => l10n.kitPieceTambourine,
  56 => l10n.kitPieceCowbell,
  75 => l10n.kitPieceClaves,
  76 => l10n.kitPieceWoodBlock,
  _ => null,
};

/// Localised label for a kit piece **identity** (change:
/// add-drum-input-calibration) — the calibration pass and the mapping table
/// name pieces without a score-derived [DrumLane] to read them from.
///
/// The kick is included here and not in [kitPieceLabel] because it has no lane
/// (it is the full-width bar); a generic piece falls back to its General MIDI
/// name, and an identity this build does not know to the raw id, so a table
/// written by a later version is still readable rather than blank.
String kitPieceLabelOf(AppLocalizations l10n, String pieceId) {
  if (pieceId == kKickPieceId) return l10n.kitPieceKick;
  final label = kitPieceLabel(
    l10n,
    DrumLane(role: KitPieceRole.other, gmNumbers: const {}, labelKey: pieceId),
  );
  if (label.isNotEmpty) return label;
  final gm = canonicalGmOfPiece(pieceId);
  if (gm == null) return pieceId;
  return gmPieceLabel(l10n, gm) ?? gmPercussionName(gm) ?? pieceId;
}
