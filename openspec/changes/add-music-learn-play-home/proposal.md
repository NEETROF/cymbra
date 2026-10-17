# Proposal

## Why

Cymbra Music already has two ways to spend time well — playing any score freely, and a
42-lesson solfège curriculum — but the app never asks the player which one they came for.
"Learn" is a single continue card sitting above the favorites, so the learning path reads
as an accessory to the library, and nothing in the home communicates a journey. A player
who wants to *get better at the piano* has no door to walk through; one who just wants to
play has a card in the way. This change gives the home two explicit doors, **Play** and
**Learn**, and makes the switch between them feel like one app rather than two piles.

## What Changes

- **A two-door home.** The home header gains a **Play / Learn** switcher, modelled on the
  existing keyboard/drums instrument switcher: sticky, always in view, one tap to change.
  *Play* is today's home (favorites by level, hub, rating deck, community). *Learn* is the
  learning path front and centre: the continue card, the path with its units and nodes,
  and — once `add-music-daily-session` lands — today's session.
- **The Courses card leaves the Play door.** Under *Play* the favorites start at the top
  again; the courses section is the *Learn* door's body, not a card above someone else's
  content. The `notation-courses` requirement that pins courses "above the favorites" is
  rewritten accordingly.
- **Drums keep today's behaviour.** The *Learn* door is keyboard-only for now: under the
  drums instrument context the switcher shows *Play* only, exactly as the courses card
  already steps aside there.
- **Animated transitions between the doors and into their screens.** Switching doors
  cross-fades and slides the body; opening the path, a lesson or a piece from either door
  uses a shared-element transition from the card tapped (the lightweight motion this change
  owns; the broader motion system is `add-music-motion-and-celebrations`).
- **A runtime kill-switch.** The two-door home ships behind the `music.learn_door.enabled`
  flag, default **on** once released, so the server can revert to today's single home
  without an app release — the same pattern as `drums.enabled`.
- **Guests see both doors.** The *Learn* door works signed out exactly as courses do today
  (completion local until sign-in); nothing new is gated on an account.

## Capabilities

### New Capabilities
- `music-learn-play-home`: the two-door home (Play / Learn), its persistence, its
  instrument-context rule, its transitions and its kill-switch.

### Modified Capabilities
- `notation-courses`: the "Courses are on the home screen, above the favorites" requirement
  becomes "Courses are the body of the Learn door" — the continue card and the path are
  reached through the Learn door, no longer pinned above the favorites.

## Impact

Products: **Music only**. Consumed, not new: the runtime feature-flag client
(`platform` flags), the instrument context, the existing courses section, learning path
and lesson player, the session/guest gating. New: the door switcher and its persisted
choice, the Learn door body, the transitions, one feature flag.

- `apps/music/lib/screens/library_screen.dart` (header + two bodies),
  `widgets/courses_section.dart` (moves under Learn), `screens/learning_path_screen.dart`
  (becomes a body, not only a pushed route), a new `state/home_door.dart` notifier
  (Riverpod + Freezed, persisted through the preferences seam), one flag key beside
  `drums_access.dart`.
- Backend: none. Flag declared in the flags back office; no proto change.
- Tests: widget tests for the switcher, persistence, drums rule and flag-off fallback;
  golden refresh for the home on a pinned platform.
