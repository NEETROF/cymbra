# feature-discovery Specification

## Purpose
TBD - created by archiving change add-welcome-onboarding. Update Purpose after archive.
## Requirements
### Requirement: Progressive first-use coaching

The app SHALL introduce each feature with a **one-time**, **dismissible** coaching hint shown at
the **first relevant use** of that feature, through a single shared mechanism, rather than a
large upfront tour. A hint's "seen" state SHALL be persisted so it is not shown again, and the
hint MUST NOT block the underlying action. The first-run hints for the rating deck, reward
feedback, and the public-sharing age gate SHALL be delivered through this shared mechanism.

#### Scenario: A feature's hint appears at first use

- **WHEN** the user reaches a feature for the first time
- **THEN** a dismissible coaching hint introduces it, without blocking the action

#### Scenario: A hint is shown only once

- **WHEN** the user has already seen and dismissed a feature's hint
- **THEN** it is not shown again on later uses

#### Scenario: Hints do not block

- **WHEN** a coaching hint is visible
- **THEN** the user can still perform the underlying action and dismiss the hint

### Requirement: Guided in-context coaching for the player's controls

The app SHALL provide a **guided, in-context** coaching sequence in the player that points the
user, **one control at a time** (a directed "here is where to tap" style), to the key play
controls: **selecting the instrument's sound**, **viewing the connected MIDI instrument and
selecting one manually**, **choosing which limbs to play**, and **rewinding by measure
and opening the measure-selection mode** (the transport measure-rewind control — one step
covering both the tap and the long-press gesture). It SHALL run the first time the user reaches
the player, SHALL be **skippable** and MUST NOT permanently block playing, and SHALL be
**replayable** later from the help/tips surface. Each highlighted control SHALL point at the
real control in place (for example within the player settings drawer); when a control is not
currently visible, the step SHALL still deliver its explanation (untargeted fallback).

On a **percussion** score the tour points at the same four controls, but three of them mean
something else, and the copy SHALL say what the control under it actually does: the sound is a
**drum kit**, the connected device is an **electronic kit** rather than a MIDI keyboard, and the
selector splits **hands from feet**, not right hand from left. Teaching the keyboard's
vocabulary there is not a rough edge but a contradiction — the words say "right hand, left hand"
while the control under the spotlight reads "Feet / Hands / Both", and the tour is precisely
the moment a first-time player takes the app at its word. The measure-rewind step is
instrument-neutral and SHALL be shared unchanged.

#### Scenario: First player visit walks the key controls

- **WHEN** the user opens the player for the first time
- **THEN** a guided sequence highlights, one at a time, the sound selection, the
  connected-MIDI/device selection, the limb selection, and the measure-rewind control

#### Scenario: The drum tour names the kit, the e-kit and the limbs

- **WHEN** the first player visit happens on a percussion score
- **THEN** the sound step names the drum kit, the device step names an electronic kit, and the
  selector step offers hands / feet / both — none of them naming a piano, a MIDI keyboard or a
  right and left hand

#### Scenario: The rewind step teaches both gestures

- **WHEN** the guided sequence reaches the measure-rewind control
- **THEN** one step explains both the tap (rewind one measure) and the long-press (open the
  measure-selection mode)

#### Scenario: The guided sequence is skippable

- **WHEN** the user chooses to skip the guided coaching
- **THEN** it stops and the user can play immediately, without it blocking them

#### Scenario: Replayable from help

- **WHEN** the user wants to see the guided coaching again later
- **THEN** they can replay it from the help/tips surface

#### Scenario: Highlights point at the real controls

- **WHEN** a step highlights a control (e.g. selecting a piano or a MIDI device)
- **THEN** it points at that actual control in place, so the user learns where it is

### Requirement: Discovery is re-findable via help/tips

The app SHALL provide a help/tips surface, reachable from a stable entry point, where a user can
re-read how the app's systems work (the core loop, ratings, points, shop and badges, the
profile, leaderboards, and going public). This lets users recover explanations that were shown
once as first-use hints.

#### Scenario: User re-reads how a system works

- **WHEN** a user opens the help/tips surface
- **THEN** they can read explanations of the app's systems, including ones previously shown as one-time hints

#### Scenario: Help is reachable from a stable place

- **WHEN** a user looks for help after dismissing the first-use hints
- **THEN** the help/tips surface is reachable from a stable entry point

### Requirement: Onboarding copy is localized and accessible

All welcome, coaching, and help copy SHALL be localized through the app's localization system and
SHALL be accessible (dismissible without relying on a single gesture, adequate contrast, screen-
reader friendly), consistent with the app's responsive/landscape layout.

#### Scenario: Copy follows the app language

- **WHEN** the app is used in a supported language
- **THEN** the welcome, coaching, and help copy appear in that language

#### Scenario: Hints are dismissible accessibly

- **WHEN** a user relies on assistive input
- **THEN** they can dismiss a coaching hint without depending on a single specific gesture

