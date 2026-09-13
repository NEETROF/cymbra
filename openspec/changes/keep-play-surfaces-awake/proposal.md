## Why

Two of the app's heaviest users report the same thing: **the phone falls asleep
while they are using Cymbra.** It is not a device misconfiguration — the app has
never asked the OS to stay awake. There is no `wakelock_plus` (or equivalent) in
`apps/music/pubspec.yaml`, no `FLAG_KEEP_SCREEN_ON` on the Android window, and no
`isIdleTimerDisabled` on the Apple platforms. The only lifecycle observer
(`main.dart:123`) silences audio and refreshes flags; nothing touches the screen.

The reason it hits *these* users first is structural. Every OS measures idle time
in **touch events**, and the whole point of Cymbra is that the player's hands are
on an instrument, not on the glass. Someone practising a piece on a connected MIDI
keyboard can go a full movement without a single tap. Wait Mode makes it worse by
design: the gate *holds* the playhead at an onset until the right note is played,
so the harder the passage, the longer the screen sits untouched — and the screen
goes dark exactly when the player is struggling, mid-piece, with both hands
committed. Recovering costs a hand, a tap, and the run.

## What Changes

- **The play surfaces keep the screen awake while they are open.** Four screens
  are play surfaces, i.e. screens whose normal use is "look at it while your hands
  are elsewhere": `PlayerScreen`, `LessonPlayerScreen`, `DrumCalibrationScreen`
  and `MidiMonitorScreen`. Everywhere else (library, catalog, hub, profile, shop,
  settings) keeps the device's normal sleep behaviour.
- **The hold spans the whole visit, not just an active run.** It is taken when the
  surface mounts and released when it is popped — deliberately *not* gated on
  `isPlaying`, because reading the score before pressing play, and pausing to work
  out a bar, are the same activity as playing it and produce just as few touches.
- **The hold is released whenever the app is not in the foreground**, and
  re-taken on resume if a play surface is still mounted. This matters beyond
  tidiness: on the desktop platforms a wakelock is an OS-level power assertion
  that outlives the window's focus, so a minimised Cymbra would otherwise keep the
  machine from sleeping.
- **Nested and stacked surfaces are counted, not toggled.** Opening the measure
  selector over the player, or a lesson that is itself a player, must not have the
  inner screen's dismissal drop a hold the outer one still needs.
- **No user-facing setting.** The behaviour is implicit, as in every sheet-music
  and tablature app; the scope is already narrow enough that a toggle would be a
  setting nobody goes looking for.

## Capabilities

### New Capabilities
- `music-screen-wakelock`: which surfaces of the Music app keep the device's
  screen awake, for how long, and how that interacts with backgrounding and with
  several surfaces being open at once.

### Modified Capabilities

None. No existing requirement changes — this adds a behaviour none of the play
surfaces' specs (`wait-mode`, `pre-play-setup`, `playback-progress`,
`midi`) currently say anything about.

## Impact

**Products.** Cymbra **Music** only (`apps/music`). Nothing is consumed from the
backend, from Cymbra ID, or from the `platform-*` socle: no RPC, no flag, no
account state, no persisted preference. Cymbra Live, the back office and the
public site are untouched.

**Code.**
- `apps/music/pubspec.yaml` — new `wakelock_plus` dependency (one plugin covering
  Android, iOS, macOS, Windows and Linux; the app's five targets).
- `apps/music/lib/services/` — a new wakelock seam behind a Riverpod provider, so
  widget tests never load the plugin, per the repo's injectable-seam convention
  (`midi_service.dart` shape).
- `apps/music/lib/widgets/` — a wrapper widget acquiring on mount / releasing on
  dispose. Two of the four screens are `ConsumerWidget`, so a wrapper avoids
  converting them to stateful widgets.
- `apps/music/lib/main.dart` — the existing `_AudioLifecycleObserver` grows the
  foreground/background release-and-retake, or gains a sibling.
- The four play-surface screens, each wrapping its body.

**Platform.** No new Android permission: `FLAG_KEEP_SCREEN_ON` is a window flag,
not a permission, so the manifest is unchanged and the Play listing's permission
set does not move. No iOS/macOS entitlement.

**Not affected.** No change to audio, scoring, the Wait Mode gate, the MIDI
stream, or any persisted state. Battery cost is bounded by the scope: the screen
only stays lit on screens the user is actively reading, and never when Cymbra is
in the background.
