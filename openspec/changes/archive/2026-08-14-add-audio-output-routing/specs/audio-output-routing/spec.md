## ADDED Requirements

### Requirement: Instrument Sounds Itself

The system SHALL offer a setting under which notes played on the **connected MIDI
instrument** are not synthesized by the app, so the instrument's own sound is
heard without the app duplicating it. This setting SHALL apply only to notes
whose source is the MIDI instrument. Notes played on the on-screen keyboard or
the computer-keyboard fallback SHALL continue to be synthesized, and all other
app audio — score playback, metronome clicks, SoundFont preview clips, and any
other sound effect — SHALL be unaffected. Everything other than sounding —
scoring, key feedback, and Wait Mode gating — SHALL behave identically whether
the setting is on or off.

#### Scenario: Instrument note is not duplicated

- **WHEN** the setting is on and a note arrives from the connected MIDI instrument
- **THEN** the app does not synthesize that note

#### Scenario: On-screen keyboard still sounds

- **WHEN** the setting is on and the user presses a key on the on-screen keyboard
- **THEN** the app synthesizes that note as usual

#### Scenario: Computer keyboard still sounds

- **WHEN** the setting is on and a note arrives from the computer-keyboard fallback
- **THEN** the app synthesizes that note as usual

#### Scenario: Other app audio is unaffected

- **WHEN** the setting is on and the score plays back, a metronome click sounds, or
  a SoundFont preview clip is played
- **THEN** each is heard exactly as when the setting is off

#### Scenario: Scoring and feedback are unaffected

- **WHEN** the setting is on and a note arrives from the connected MIDI instrument
- **THEN** it is scored, shown as key feedback, and satisfies Wait Mode exactly as
  when the setting is off

#### Scenario: Setting is unavailable without an instrument

- **WHEN** no MIDI input port is connected
- **THEN** the setting is offered in a disabled state with the reason shown, rather
  than silently having no effect

### Requirement: Desktop Audio Output Selection

On desktop platforms the system SHALL list the host's available audio output
devices, SHALL let the user select one or follow the system default, and SHALL
route **all** app audio through the selected device. Changing the selection SHALL
take effect without restarting the app and SHALL NOT leave any voice sounding
across the change.

#### Scenario: Outputs are listed

- **WHEN** the sound output section is opened on a desktop platform
- **THEN** the available output devices are listed together with an option to
  follow the system default

#### Scenario: Selecting a device reroutes all app audio

- **WHEN** the user selects an output device
- **THEN** synthesized notes, score playback, metronome clicks and preview clips
  are all heard through that device

#### Scenario: Switch leaves no hanging voice

- **WHEN** the output device is changed while notes are sounding
- **THEN** no voice keeps sounding on the previous device

#### Scenario: Following the system default

- **WHEN** the user chooses to follow the system default
- **THEN** the app uses whichever device the operating system designates as
  default

### Requirement: Output Selection Persistence And Fallback

The system SHALL remember the selected output device across restarts and SHALL
apply the remembered selection at startup before the first sound, without
requiring any screen to be opened first. When the remembered device is not
present at startup, or a requested device cannot be opened, the system SHALL
fall back to the system default and SHALL report the device actually in use
rather than the one requested. A failed selection SHALL NOT interrupt audio
that is currently working, and SHALL NOT leave the app silent. The selection
SHALL survive the device's absence: when the selected device disappears
mid-session and later returns, the system SHALL re-adopt it without user
action.

#### Scenario: Selection survives a restart

- **WHEN** an output device is selected and the app is relaunched with that device
  present
- **THEN** the app uses that device again

#### Scenario: Remembered device is absent

- **WHEN** the app starts and the remembered device is not present
- **THEN** the app falls back to the system default, keeps working, and reports
  the device actually in use

#### Scenario: Requested device cannot be opened

- **WHEN** the user selects a device that fails to open
- **THEN** the previously working audio keeps running, and a localized failure
  message is shown — never a raw platform or engine error string

#### Scenario: Returning device is re-adopted

- **WHEN** the selected device went away mid-session (audio fell back to a
  working output) and the device is plugged back in
- **THEN** the app's audio returns to the selected device without user action

### Requirement: Active Route Reporting On iOS

On iOS the system SHALL display the audio route currently in use and SHALL
provide access to the operating system's route picker, rather than presenting a
device list it cannot honor — the operating system owns the route there. The
reported route SHALL carry a kind — built-in, headphones, Bluetooth, USB, or
other — and SHALL refresh after the user changes the route.

#### Scenario: Active route is displayed

- **WHEN** the sound output section is opened on iOS
- **THEN** the currently active audio route is displayed

#### Scenario: System route picker is reachable

- **WHEN** the user activates the route control on iOS
- **THEN** the operating system's route picker is presented

#### Scenario: Route change is reflected

- **WHEN** the user changes the audio route from the system picker
- **THEN** the displayed route updates to the new route

#### Scenario: Unknown route kind degrades safely

- **WHEN** the platform reports a route whose type is not recognized
- **THEN** the route is shown with the "other" kind and the section keeps
  functioning

### Requirement: Android Audio Output Selection

On Android the system SHALL list the platform's audio outputs and SHALL route
the **app's own** audio through the selected one, without moving the
system-wide media route. The list SHALL contain at most one entry per device
name and SHALL NOT offer outputs the app cannot actually play on. USB-audio
outputs SHALL be offered only as an explicitly labelled experimental choice —
the platform's USB-audio path proved broken below the app on tested devices —
and the app SHALL NOT route to a USB-audio output unless the user selects one:
when the operating system would route media to USB by default, the app SHALL
play on the best available non-USB output instead.

#### Scenario: Outputs are listed and selectable

- **WHEN** the sound output section is opened on Android
- **THEN** the platform's outputs are listed, each name at most once, together
  with an option to follow the default, and selecting one moves the app's audio
  to it

#### Scenario: USB output is an informed opt-in

- **WHEN** a USB-audio output is connected on Android
- **THEN** it appears in the list labelled as experimental, and carries the
  app's audio only if the user selects it

#### Scenario: The default never lands on USB

- **WHEN** a USB-audio device is connected on Android and the user has not
  selected an output
- **THEN** the app's audio plays on the best non-USB output, even though the
  operating system routes media to the USB device

### Requirement: Output Route Self-Healing

The system SHALL rebuild the audio output when the route carrying it stops
delivering sound — its device disappears, the platform invalidates the stream,
or the output stops consuming audio at the stream's nominal rate — so that
sound survives. It SHALL fall back to a working output when the same route
keeps failing, and SHALL stop retrying rather than rebuild against a failing
route indefinitely. A failing audio route SHALL NOT take unrelated
functionality — such as the MIDI connection of a composite USB instrument —
down with it through repeated rebuild attempts.

#### Scenario: Sound survives a dead route

- **WHEN** the output the app is playing on stops delivering audio
- **THEN** the app's audio continues on a working output without an app restart

#### Scenario: An off-clock route is rebuilt

- **WHEN** the output consumes audio at a rate that sustainably diverges from
  the stream's sample rate
- **THEN** the output is rebuilt rather than left playing broken audio
  indefinitely

#### Scenario: A persistently failing route is abandoned

- **WHEN** rebuilding on the same route fails repeatedly within a short window
- **THEN** the system falls back to a working output, then stops retrying, and
  the MIDI connection of a composite instrument on that route is not disturbed

### Requirement: Wireless Route Warning

The system SHALL identify when the active audio route is wireless and SHALL warn
that such a route is suitable for listening but not for playing, because the
sound arrives noticeably after the key is pressed. The system SHALL NOT block a
wireless route. The warning SHALL be based on the route's reported kind, not on
matching its name.

#### Scenario: Wireless route is flagged

- **WHEN** the active audio route's kind is Bluetooth
- **THEN** the sound output section warns that this route delays the sound and
  suits listening rather than playing

#### Scenario: Wired route is not flagged

- **WHEN** the active route's kind is built-in, headphones or USB
- **THEN** no wireless warning is shown

#### Scenario: Wireless route stays usable

- **WHEN** a wireless route is active
- **THEN** the app keeps playing through it and does not force a different route

### Requirement: Output Offset Setting

The system SHALL offer an adjustable output offset, in milliseconds, defaulting
to zero, used to compensate a delayed audio route. When a wireless route becomes
active the system SHALL suggest a starting value without applying it on the
user's behalf. The offset SHALL be persisted and SHALL be adjustable back to
zero.

#### Scenario: Offset defaults to zero

- **WHEN** the user has never set an offset
- **THEN** the offset is zero and the app behaves exactly as if the setting did
  not exist

#### Scenario: Suggestion on a wireless route

- **WHEN** a wireless route becomes active and the offset is still zero
- **THEN** a starting value is suggested to the user, and is applied only if the
  user accepts it

#### Scenario: Offset is persisted

- **WHEN** the user sets an offset and relaunches the app
- **THEN** the same offset is still in effect

### Requirement: Injectable Audio Routing Seam

The audio routing integration SHALL be exposed behind an injectable provider with
an abstract interface, so device lists, active routes, route kinds, and selection
failures can be driven deterministically in unit and widget tests without audio
hardware, without changing the host's route, and without the native library.

#### Scenario: Production wiring uses the real routing

- **WHEN** the app runs normally
- **THEN** the provider resolves to the implementation backed by the engine and
  the platform route reporting

#### Scenario: Tests override the seam

- **WHEN** a test needs a deterministic routing outcome, such as a wireless route,
  an absent remembered device, or a device that fails to open
- **THEN** the provider is overridden with a test double and the UI renders that
  outcome without touching audio hardware
