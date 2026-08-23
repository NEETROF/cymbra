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

import 'package:flutter/foundation.dart' show mapEquals;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../painters/smufl.dart';
import '../state/note_label.dart';
import '../state/player_data.dart';
import '../state/player_notifier.dart';
import '../theme/cymbra_theme.dart';

/// What the reading aid currently has to say.
///
/// A value type with structural equality (the names map included) so `select`
/// can compare it: the aid only rebuilds when the *displayed* content changes,
/// not on every playhead frame.
class ReadingAidView {
  /// Whether the aid has anything to show at all.
  final bool show;

  /// Localized name of each awaited note, by MIDI pitch — drawn on that note's
  /// own key, so there is no line to overflow and no need to bound a chord.
  final Map<int, String> names;

  /// The rhythmic figure shared by every awaited note, when they agree on one
  /// and the rhythm level is enabled.
  final FigureToken? figure;

  const ReadingAidView({
    required this.show,
    this.names = const {},
    this.figure,
  });

  static const hidden = ReadingAidView(show: false);

  @override
  bool operator ==(Object other) =>
      other is ReadingAidView &&
      other.show == show &&
      other.figure == figure &&
      mapEquals(other.names, names);

  @override
  int get hashCode => Object.hash(
    show,
    figure,
    Object.hashAllUnordered(
      names.entries.map((e) => Object.hash(e.key, e.value)),
    ),
  );
}

/// Derives what the aid should say from the player state.
///
/// Shows nothing unless Wait Mode is actively holding at an onset: that is the
/// one moment where the playhead is frozen, so nothing the aid draws can compete
/// with a moving score. Pure, so the display rules are host-testable.
ReadingAidView readingAidViewOf(
  PlayerData data, {
  required bool solfege,
  required bool frenchRe,
}) {
  if (data.readingAid == NoteReadingAid.off) return ReadingAidView.hidden;
  if (!data.waitMode || !data.blocked) return ReadingAidView.hidden;

  final notes = data.expectedNotes;
  if (notes.isEmpty) return ReadingAidView.hidden;

  // The key signature is only consulted for notes with no written spelling, and
  // then it is the one in force at that point in the piece.
  final measure = data.measureAt(notes.first.startMs.toDouble());
  final fifths =
      (measure != null && measure.index < data.measureKeyFifths.length)
      ? data.measureKeyFifths[measure.index]
      : data.keyFifths;

  final names = {
    for (final n in notes)
      n.pitch: noteLabel(
        n,
        solfege: solfege,
        frenchRe: frenchRe,
        keyFifths: fifths,
      ),
  };

  FigureToken? figure;
  if (data.readingAid == NoteReadingAid.nameAndRhythm) {
    final beatMs = data.beatDurationMsAt(notes.first.startMs.toDouble());
    final tokens = [
      for (final n in notes)
        // A grace note is an ornament, not a held figure: quoting its written
        // type ("eighth — hold half a beat") would tell the player to hold a
        // note the score says to flick. No figure is better than a wrong one.
        n.isGrace
            ? null
            : figureFor(
                noteType: n.noteType,
                dots: n.dots,
                beatType: data.beatType,
                durationMs: n.durationMs.toDouble(),
                beatMs: beatMs > 0 ? beatMs : null,
              ),
    ];
    // One figure for the whole onset only when every note agrees on it; mixed
    // figures show none rather than implying a wrong one.
    final first = tokens.first;
    if (first != null && tokens.every((t) => t == first)) figure = first;
  }

  return ReadingAidView(show: true, names: names, figure: figure);
}

/// Whether the app's active language names notes in solfège, and whether it
/// writes `Ré` (French) rather than `Re` (Spanish, Italian).
({bool solfege, bool frenchRe}) namingConventionOf(BuildContext context) {
  final lang = Localizations.localeOf(context).languageCode;
  return (solfege: lang != 'en', frenchRe: lang == 'fr');
}

/// The localized name of a rhythmic figure, dots included.
String figureProse(AppLocalizations l10n, FigureToken token) {
  final base = switch (token.figure) {
    NoteFigure.doubleWhole => l10n.figureDoubleWhole,
    NoteFigure.whole => l10n.figureWhole,
    NoteFigure.half => l10n.figureHalf,
    NoteFigure.quarter => l10n.figureQuarter,
    NoteFigure.eighth => l10n.figureEighth,
    NoteFigure.sixteenth => l10n.figureSixteenth,
    NoteFigure.thirtySecond => l10n.figureThirtySecond,
  };
  return switch (token.dots) {
    1 => l10n.figureDotted(base),
    >= 2 => l10n.figureDoubleDotted(base),
    _ => base,
  };
}

/// Names the rhythmic figure of the note Wait Mode is holding for.
///
/// A floating card, not a reserved band: it is only ever on screen while the
/// gate blocks — when the playhead is frozen and there is nothing underneath it
/// to obscure — so it costs the score no layout space at all. The note *names*
/// need no card; they are drawn on the keys themselves.
class ReadingAidOverlay extends ConsumerWidget {
  const ReadingAidOverlay({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final naming = namingConventionOf(context);

    final figure = ref.watch(
      playerProvider.select(
        (d) => readingAidViewOf(
          d,
          solfege: naming.solfege,
          frenchRe: naming.frenchRe,
        ).figure,
      ),
    );
    if (figure == null) return const SizedBox.shrink();

    return IgnorePointer(
      child: Align(
        alignment: Alignment.bottomCenter,
        child: Padding(
          padding: const EdgeInsets.only(bottom: 8),
          child: DecoratedBox(
            key: const Key('reading-aid-figure'),
            decoration: BoxDecoration(
              // Deliberately see-through: the card sits over the score, and the
              // player should keep reading what is underneath it. Kept opaque
              // enough that the text stays legible over a busy passage.
              color: CymbraColors.surfaceContainerHigh.withValues(alpha: 0.7),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    figure.glyphWithDots,
                    style: const TextStyle(
                      fontFamily: Smufl.fontFamily,
                      color: CymbraColors.onSurface,
                      fontSize: 22,
                      height: 1,
                    ),
                  ),
                  const SizedBox(width: 10),
                  Text(
                    figureProse(l10n, figure),
                    style: const TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  if (figure.beats != null) ...[
                    const SizedBox(width: 8),
                    Text(
                      l10n.readingAidHoldBeats(figure.beats!),
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                        fontSize: 13,
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
