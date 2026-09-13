## ADDED Requirements

### Requirement: Play Surfaces Keep The Screen Awake

The Music app SHALL prevent the device from dimming or locking its screen for as
long as a **play surface** is on screen, and SHALL let the device sleep normally
everywhere else.

A play surface is a screen whose normal use is reading it while the player's
hands are on an instrument, so it produces no touch events for long stretches.
Exactly four qualify: the player, the lesson player, the drum-input calibration,
and the MIDI monitor. Every other screen — library, catalog, score hub, profile,
plans, sound fonts, settings — SHALL leave the device's sleep behaviour untouched,
because on those the user is either touching the screen or has walked away.

The hold SHALL NOT be conditioned on playback running. Reading the score before
pressing play, and stopping to work out a bar, are the same activity as playing
it and generate the same absence of touches; a hold that tracked `isPlaying` would
black out the screen in precisely the moments the player is stuck.

#### Scenario: Playing a piece without touching the screen
- **WHEN** the player screen is open and the player performs on a connected MIDI
  instrument for longer than the device's screen timeout, touching nothing
- **THEN** the screen stays lit for the whole performance

#### Scenario: Wait Mode holding at a hard onset
- **WHEN** the Wait Mode gate holds the playhead at an onset for longer than the
  device's screen timeout while the player works out the notes
- **THEN** the screen stays lit

#### Scenario: Reading before starting
- **WHEN** the player screen is open with a score loaded and playback stopped, and
  the player reads it for longer than the device's screen timeout
- **THEN** the screen stays lit

#### Scenario: Calibrating a drum kit
- **WHEN** the drum-input calibration screen is open and the player strikes pads
  on their e-kit without touching the device
- **THEN** the screen stays lit

#### Scenario: Watching the MIDI monitor
- **WHEN** the MIDI monitor is open and the player exercises their instrument to
  diagnose it
- **THEN** the screen stays lit

#### Scenario: Ordinary screens sleep normally
- **WHEN** the library, catalog, score hub, profile or any other non-play screen
  is open and untouched past the device's screen timeout
- **THEN** the device dims and locks as it normally would

### Requirement: The Hold Is Released On Leaving A Play Surface

The app SHALL release its hold on the screen as soon as no play surface remains on
screen, so that a device left on a non-play screen sleeps on its own schedule.

#### Scenario: Leaving the player
- **WHEN** the player navigates back from the player screen to the library
- **THEN** the app stops holding the screen awake and the device resumes its
  normal sleep behaviour

#### Scenario: Finishing a lesson
- **WHEN** the lesson player is closed
- **THEN** the app stops holding the screen awake

### Requirement: Overlapping Holds Are Counted

The app SHALL treat concurrent requests to keep the screen awake as a count, not
as a switch: the screen SHALL stay held while at least one play surface is
mounted, and SHALL be released only when the last of them goes away.

Play surfaces stack — the measure selector opens over the player, and a lesson can
put a player inside itself — so a single boolean would let an inner screen's
dismissal drop a hold the screen still underneath it depends on.

#### Scenario: A surface opened over another one closes
- **WHEN** a second play surface is opened on top of a first one and is then
  closed, leaving the first still on screen
- **THEN** the screen stays held

#### Scenario: The last surface closes
- **WHEN** the last remaining play surface is closed
- **THEN** the hold is released

### Requirement: The Hold Never Outlives The Foreground

The app SHALL release its hold whenever it stops being the foreground app, and
SHALL re-take it on returning to the foreground if a play surface is still
mounted.

This is not only tidiness. On the desktop platforms the hold is an OS-level power
assertion that survives losing window focus, so a minimised Cymbra would otherwise
keep the machine awake indefinitely.

#### Scenario: Backgrounding with the player open
- **WHEN** the player screen is open and the app is backgrounded, hidden, or
  detached
- **THEN** the app releases its hold and the device follows its normal sleep
  behaviour

#### Scenario: Returning to a still-open player
- **WHEN** the app returns to the foreground and a play surface is still mounted
- **THEN** the app re-takes the hold and the screen stays lit again

#### Scenario: Returning to a non-play screen
- **WHEN** the app returns to the foreground with no play surface mounted
- **THEN** the app takes no hold

### Requirement: The Wakelock Is An Injectable Seam

The platform wakelock SHALL be reached only through a provider-injected seam, so
that state and widget tests exercise every rule above on the Dart VM without
loading the native plugin, and so no screen or widget calls the plugin directly.

The app SHALL also tolerate a platform that cannot honour the request: a failure to
take or release the hold SHALL be logged and swallowed, never surfaced to the
player and never allowed to break the screen it was taken for. Keeping the screen
lit is a comfort, not a precondition for practising.

#### Scenario: Tests drive the seam
- **WHEN** a widget test mounts a play surface with the wakelock seam overridden
- **THEN** the test observes the acquire and release without any native plugin
  being loaded

#### Scenario: The platform refuses the request
- **WHEN** the platform call to hold or release the screen fails
- **THEN** the screen it was requested for continues to work normally and no error
  is shown to the player
