## ADDED Requirements

### Requirement: Percussion strokes converge on the player's note entry points

The player SHALL route every percussion stroke — an on-screen tap on the
drawn kit, a
kick-pedal tap, and every note event arriving from an external MIDI device —
through the same note-on/note-off entry points that keyboard input already
uses, carrying the stroke's General MIDI percussion number as the pitch.
Below the sounding decision (the instrument-sounds-itself rule of
`audio-output-routing`, which is the one place a stroke's source is
consulted), sources SHALL be indistinguishable.

One convergence point is what lets `add-drum-scoring` later arm the gate and
the scorer against strokes without re-plumbing input, exactly as
`audio-output` already relies on it for keyboard sources. A percussion score
is reachable only by the drum audience of `music-drums-visibility`; this
capability adds no gating of its own.

#### Scenario: A tap on the kit reaches the entry points

- **WHEN** the user taps a piece of the drawn kit
- **THEN** a note-on for that piece's General MIDI number enters the player
  through the same entry point a key press uses

#### Scenario: An e-kit stroke reaches the same entry points

- **WHEN** a note-on arrives from a connected electronic kit while a
  percussion score is loaded
- **THEN** it enters the player through the same entry point, carrying its
  General MIDI number as the pitch

#### Scenario: Sources are identical below the sounding decision

- **WHEN** the same General MIDI number arrives once from a tap and once
  from an external device
- **THEN** held-state bookkeeping and feedback behave identically for both;
  only the sounding decision may differ, per the instrument-sounds-itself rule

### Requirement: External percussion input is accepted from any channel

The player SHALL accept external percussion strokes whatever MIDI channel the
device transmits on, and SHALL NOT introduce any percussion-specific channel
filter: the normalized event stream deliberately carries no channel (see
`midi`), and the loaded score's instrument — not the wire — decides how a note
number is interpreted.

General MIDI reserves channel 10 for percussion and many kits default to it,
but kits are configurable and off-by-one channel confusion (0-based versus
1-based numbering) is endemic in their menus. A channel filter would answer a
misconfigured kit with silence — the worst failure an input path can have —
and buys nothing: the instrument context is score-scoped (piano XOR drums), so
for a percussion score every incoming note number is a stroke.

#### Scenario: A channel-10 stroke plays

- **WHEN** a kit transmits a stroke on channel 10, as most do by default
- **THEN** the stroke reaches the player like any other note event

#### Scenario: A stroke on any other channel plays identically

- **WHEN** a kit is configured to transmit on a channel other than 10
- **THEN** its strokes reach the player identically — no stroke is dropped for
  its channel

### Requirement: A stroke sounds its piece as a one-shot

The player SHALL sound each percussion stroke immediately through the
percussion one-shot ("sound this General MIDI number now" — `drum_on`, with
its paired `drum_off`) that `add-drum-audio-channel` exposes on the
injectable audio seam — never through the pitched piano voice path — both
while playback is running and while it is stopped. The instrument-sounds-itself rule SHALL apply unchanged: a stroke
whose source is the connected MIDI instrument is not synthesized while the
rule is on (drum modules sound themselves), and on-screen strokes always
synthesize.

This change consumes the hook; the synth path itself — the drum channel, the
kit SoundFont, the hook's contract — is `add-drum-audio-channel`'s and is not
restated here.

#### Scenario: A tap on the kit is audible while stopped

- **WHEN** playback is stopped and the user taps a drawn piece
- **THEN** the struck piece sounds immediately as a one-shot

#### Scenario: An e-kit stroke is audible during playback

- **WHEN** a stroke arrives from a connected kit while a percussion score is
  playing and the instrument-sounds-itself rule is off
- **THEN** the struck piece sounds through the one-shot, alongside the score
  audio

#### Scenario: A module that sounds itself is not doubled

- **WHEN** the instrument-sounds-itself rule is on and a stroke arrives from
  the connected kit
- **THEN** the app does not synthesize it, while bookkeeping and feedback
  behave exactly as for a synthesized stroke

#### Scenario: On-screen strokes still sound when the module sounds itself

- **WHEN** the instrument-sounds-itself rule is on and the user taps a piece
- **THEN** the tap synthesizes normally — the rule exempts only the
  instrument's own strokes

#### Scenario: No stroke reaches the piano voice

- **WHEN** any percussion stroke is sounded
- **THEN** it is rendered as a percussion one-shot, never as a pitched piano
  note of the same number

### Requirement: Releases are bookkeeping, never meaning

The player SHALL treat a percussion note-off as held-state hygiene only: it
clears the entry its note-on created and SHALL have no audible effect and no
feedback effect. A stroke's meaning is its attack.

Electronic kits send the note-off (or a velocity-0 note-on) within
milliseconds of the attack, and some hardware is sloppy about sending it at
all; a one-shot has no sustain a release could end. Processing releases for
hygiene keeps the shared entry points symmetric with the keyboard path while
attaching no semantics a sloppy kit could violate.

#### Scenario: An immediate release changes nothing audible or visible

- **WHEN** a stroke's note-off arrives milliseconds after its note-on
- **THEN** the one-shot keeps sounding to its natural end and the struck flash
  runs its full course

#### Scenario: A missing release leaves nothing hanging

- **WHEN** a kit never sends the note-off for a stroke
- **THEN** no voice sustains and no feedback lingers — at most a stale
  held-state entry remains, cleared by the pitch's next attack/release pair

### Requirement: Velocity is carried but not yet consumed

The player SHALL sound every percussion stroke at the synthesizer's uniform
default loudness in this change: the velocity the event stream carries (see
`midi`) is received and not consumed, matching the pitched path, which has
never consumed velocity either. This interim is stated deliberately so the
archived spec never implies dynamics the code does not have; velocity-driven
dynamics is one future decision for both instruments, not a percussion
side-door.

#### Scenario: Soft and hard strokes sound alike

- **WHEN** two strokes of the same piece arrive with very different velocities
- **THEN** both sound at the same loudness

### Requirement: Input is never suppressed — filters act on visibility and judgment only

The player SHALL accept and sound every percussion stroke, and flash the
controller surface it resolves to, regardless of the hand selection or any
visibility filter: during hands-only practice a foot stroke (pedal tap or
e-kit kick) still sounds and still flashes the pedal, and during feet-only
practice a hand stroke still sounds and still flashes its piece. Hand selection filters what is drawn and what will one day be
judged (`hand-selection`, `performance-scoring`) — never what the player may
do.

This is how the keyboard already behaves — keys of the unselected hand still
sound — and it is what keeps feedback honest: an instrument that falls silent
because of a display filter reads as broken, not filtered.

#### Scenario: A foot stroke during hands-only practice

- **WHEN** the hand selection is hands-only and a kick stroke arrives from the
  pedal or an external kit
- **THEN** the stroke sounds and the pedal flashes, even though foot events
  are hidden from the cascade

#### Scenario: A hand stroke during feet-only practice

- **WHEN** the hand selection is feet-only and a snare stroke arrives
- **THEN** the stroke sounds and the drawn snare flashes, even though hand
  events are hidden from the cascade

### Requirement: A stroke outside the score's kit is audible, not an error

The player SHALL sound a stroke whose General MIDI number resolves to no lane
of the loaded score — free play on a piece the score does not use — and SHALL
show no feedback for it and raise no error: the controller presents only the
score's pieces, so there is nothing to flash, exactly as a piano key the piece
never uses sounds without any expected-state involvement.

#### Scenario: A crash over a crash-less groove

- **WHEN** the loaded score uses only hi-hat, snare and kick, and a crash
  stroke arrives from an external kit
- **THEN** the crash sounds, no piece flashes, and no error state is raised

#### Scenario: A kick on a kickless score

- **WHEN** the loaded score has no kick notes (so the strip shows no pedal)
  and a kick stroke arrives from an external kit
- **THEN** the stroke sounds and nothing flashes
