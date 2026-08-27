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

/// The MIDI input monitor's model (change: add-drum-input-calibration): what
/// arrived, and what the app made of it.
///
/// The app plays back exactly the General MIDI number an instrument sends and
/// resolves a stroke to a kit piece by that same number. That is right for a
/// module on the standard map and silently wrong for anything else — and an
/// e-kit lets its owner reassign every pad, while head/rim/bell zones have no
/// standard numbers to be assigned to. A stroke on a number the app does not
/// recognise is not merely mis-sounded: it resolves to no piece, so there is no
/// pad flash, no Wait-Mode gate release and no scoring credit either, and
/// nothing on screen has ever shown what was actually received.
///
/// Pure and host-testable. The notifier owns the clock and the subscription;
/// everything here is a function of one event plus the loaded score's kit.
library;

import 'drum_kit.dart';

/// How the app resolved an incoming number against the loaded score's kit.
enum MidiResolution {
  /// It denotes a piece the loaded score's kit contains — the case where a
  /// stroke sounds, flashes, gates and scores.
  matchedPiece,

  /// A General MIDI percussion number, but not one this score's kit uses. It
  /// still sounds (the font covers the map); it lights nothing and satisfies
  /// nothing.
  outsideThisKit,

  /// Outside the General MIDI percussion map altogether. The app has no name
  /// for it and the kit font is unlikely to have a sample — this is the shape
  /// of "I hit the pad and nothing happened".
  outsideTheMap,

  /// A keyboard score: numbers are pitches, and the kit vocabulary does not
  /// apply.
  pitched,
}

/// One observed MIDI event, with the app's own reading of it.
class MidiMonitorEntry {
  const MidiMonitorEntry({
    required this.seq,
    required this.pitch,
    required this.velocity,
    required this.channel,
    required this.isNoteOn,
    required this.resolution,
    this.gmName,
    this.pieceLabelKey,
    this.pieceGmName,
  });

  /// Monotonic arrival order. Two strokes can share a millisecond; nothing
  /// else here identifies an entry.
  final int seq;

  /// The number as received, before anything interprets it.
  final int pitch;
  final int velocity;

  /// Transmitting channel, 0-based (channel 10 is `9`). Reported only — the
  /// app's interpretation is channel-agnostic and stays that way.
  final int channel;
  final bool isNoteOn;

  final MidiResolution resolution;

  /// The General MIDI percussion name of [pitch], or null outside the map.
  final String? gmName;

  /// The localisation key of the kit piece it resolved to, when it matched a
  /// *named* piece of the loaded kit.
  final String? pieceLabelKey;

  /// The General MIDI name of the matched piece when it is a generic one (the
  /// terminal bucket carries no localised label).
  final String? pieceGmName;

  /// Whether this stroke did nothing: it lit no pad, released no gate and
  /// earned no credit. The single question the monitor exists to answer.
  bool get isInert =>
      resolution == MidiResolution.outsideThisKit ||
      resolution == MidiResolution.outsideTheMap;
}

/// Read one event against the loaded score's kit.
///
/// [lanes] is the score-derived layout and [hasKick] whether the score writes a
/// kick — together they are "the pieces this score asks for". A percussion
/// score with an empty layout and no kick (nothing loaded yet) still resolves
/// against the General MIDI map, so the monitor is useful before a score is
/// open, which is exactly when a player goes looking for it.
MidiMonitorEntry readMidiEvent({
  required int seq,
  required int pitch,
  required int velocity,
  required int channel,
  required bool isNoteOn,
  required bool percussion,
  List<DrumLane> lanes = const [],
  bool hasKick = false,
}) {
  if (!percussion) {
    return MidiMonitorEntry(
      seq: seq,
      pitch: pitch,
      velocity: velocity,
      channel: channel,
      isNoteOn: isNoteOn,
      resolution: MidiResolution.pitched,
    );
  }

  final gmName = gmPercussionName(pitch);
  // The lane lookup is piece-grained, not number-grained: a score written
  // entirely in 38s is still answered by a stroke on 40, because they are one
  // snare. Resolving on the raw number would report a false miss for exactly
  // the players this surface is for.
  final lane = pieceLaneIndexOf(lanes, pitch);
  final matchesKick = hasKick && kKickGmNumbers.contains(pitch);

  if (lane != null || matchesKick) {
    final matched = lane != null ? lanes[lane] : null;
    return MidiMonitorEntry(
      seq: seq,
      pitch: pitch,
      velocity: velocity,
      channel: channel,
      isNoteOn: isNoteOn,
      resolution: MidiResolution.matchedPiece,
      gmName: gmName,
      pieceLabelKey: matched?.labelKey,
      pieceGmName: matched?.gmName,
    );
  }

  return MidiMonitorEntry(
    seq: seq,
    pitch: pitch,
    velocity: velocity,
    channel: channel,
    isNoteOn: isNoteOn,
    resolution: gmName == null
        ? MidiResolution.outsideTheMap
        : MidiResolution.outsideThisKit,
    gmName: gmName,
  );
}

/// How many entries the monitor keeps. Small on purpose: it is a live read-out,
/// not a log, and a session left open must not grow without bound.
const int kMidiMonitorCapacity = 60;
