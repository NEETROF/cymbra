# Proposal

## Why

The app rewards showing up (a daily practice award, a streak with a buy-back, an evening
reminder) but never says what to do when you show up. The Learn door lists forty-two
lessons; a learner with ten minutes has to pick. A short, named **today's session** turns
the open path into a daily habit: one lesson, one piece, one review, a ring that fills, and
a streak that means something because the day had a shape.

## What Changes

- **Today's session on the Learn door.** A block at the top of the Learn door (the slot
  reserved by `add-music-learn-play-home`) listing up to three items built from existing
  data: the **next lesson** in path order, the **next piece step** (or a passed piece to
  replay when the next lesson has none), and a **review** (a short drill from a completed
  unit, or a replay of a passed piece). Each item opens exactly what it names.
- **A daily goal the learner sizes.** Light, normal or full — one, two or three items a
  day. Stored on the device; the ring fills as items are done today.
- **Done is detected, not declared.** A lesson completed today, a piece step run today, a
  review run today mark their item done. The day boundary is the player's local day, the
  same one the streak and the practice award use.
- **Finishing the session is a moment.** When the last item of the goal is done, the ring
  closes and the session is marked complete for the day; the visuals of that moment belong
  to `add-music-motion-and-celebrations`, the state and the trigger live here.
- **The streak is shown where the session is.** The block carries the current streak and
  whether today is covered, so the learner sees why the session matters without opening the
  standing chip.
- **No new server state.** The session is derived on the device from course completion,
  passed steps, today's play sessions and the streak read; nothing is recorded that is not
  already recorded. Cross-device "done today" follows those existing records.

## Capabilities

### New Capabilities
- `music-daily-session`: the composition of today's session, the goal size, done
  detection on the local day, the completion moment and the streak display within it.

### Modified Capabilities
- None. The practice streak and the showing-up award are consumed, not changed.

## Impact

Products: **Music only**. Consumed, not new: course completion and passed state, the path
order, play sessions and the play outbox, the streak read (`GetStreak`), the practice
award, local preferences. New: a daily-session notifier, a local "done today" ledger, the
block widget, a goal setting.

- Flutter: `state/daily_session_notifier.dart` (Riverpod + Freezed), a `state/today_ledger
  .dart` persisted through the preferences seam, `widgets/daily_session_block.dart` mounted
  in the Learn door's slot, a goal-size row in the player settings or the Learn header.
- Backend: none.
- Tests: notifier composition for every path position (nothing done, all lessons done, no
  piece steps yet, offline), done detection across a local-day rollover, goal sizes, widget
  states; golden for the block.
