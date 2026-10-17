# Proposal

## Why

Everyone starts the learning path at "The piano and middle C". A player who already reads
treble clef scrolls past twelve lessons they do not need, or gives up on the path as a
beginners' thing. A two-minute **placement** at the first visit of the Learn door puts the
continue card on the right unit, without pretending to certify anything: the earlier units
stay open, they are just not where the path starts.

## What Changes

- **A short placement at the first Learn visit.** Offered, never forced: a card on the
  Learn door the first time it opens ("Already play a little? Find your starting point"),
  also reachable later from the path header. Five to seven quick exercises spanning the
  units (name a note, place a note, tap a rhythm, hear an interval) and one short piece
  step, each tagged with the unit it probes.
- **A starting unit, not a diploma.** The result is the first unit whose exercise the
  learner missed. The continue card and today's session start there; units before it are
  shown as **skipped by placement**, distinct from completed, awarding no badge and
  remaining open to take at any time.
- **Authored as a course.** The placement is a course manifest in the existing format with
  a `placement` track, so its content is server-side, localized inline and editable without
  an app release; blocks carry a `placementUnit` tag the app reads. Older apps simply do not
  list the `placement` track.
- **Retakable and reversible.** Taking it again replaces the starting unit; "Start from the
  beginning" clears it.
- **Stored on the device.** The starting unit is a pointer, cheap to re-establish; a device
  without it starts at unit 1 and offers the placement again.

## Capabilities

### New Capabilities
- `music-placement-test`: the offer and its timing, the placement course and its unit tags,
  the resulting starting unit and the skipped-by-placement state, retake and reset.

### Modified Capabilities
- None. The course format's forward compatibility already lets a block carry a tag older
  apps ignore; the `placement` track is a new value of an existing field.

## Impact

Products: **Music only**. Consumed, not new: the course format, the lesson player and its
exercise blocks, the `piece` block from `add-music-piece-lessons`, the course listing,
local preferences. New: the placement offer card, the result computation, the starting-unit
preference, the skipped state on the path.

- Flutter: `state/placement_notifier.dart` (Riverpod + Freezed, persisted), a result
  computation in a pure module, `widgets/placement_offer_card.dart`, path and continue-card
  changes to honour the starting unit, the `placementUnit` tag in `course_manifest.dart`.
- Backend: none beyond content — one placement course JSON under
  `backend/content/courses/` and the regenerated seed script; `ListCourses` already returns
  any track.
- Tests: result computation for every miss pattern, notifier persistence and reset, path
  rendering of skipped units, offer timing, the corpus test accepting the `placement` track.
