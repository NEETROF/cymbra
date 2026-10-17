# Spec Delta

## Purpose

Music has one motion system: shared timings, a reduced-motion rule, animated celebrations
on the moments that reward a learner, counters that count and a path that is drawn, all
within the learning-safe constraints of the play surface.

## ADDED Requirements

### Requirement: Motion follows shared timings and the reduced-motion setting
Transitions and celebrations SHALL use a shared set of durations and curves. When the
platform requests reduced motion, every such animation SHALL complete immediately, showing
its final state with no animated frames. No animation SHALL require an additional runtime
dependency or bundled animation asset.

#### Scenario: Reduced motion shows final states
- **WHEN** the platform reports reduced motion and a celebration is raised
- **THEN** the celebration appears in its final state with no animated frames

#### Scenario: A door switch uses the standard timing
- **WHEN** the player switches home doors with reduced motion off
- **THEN** the transition completes within the shared standard duration

### Requirement: Reward moments share one animated celebration
Lesson completed, piece passed, badge earned, level up, daily session complete and a
streak milestone SHALL be presented through one celebration surface in which the icon
scales in, a burst of drawn confetti fires once, the title settles and, when points are
involved, the amount counts up. Actions SHALL become interactive only once the sequence is
complete. Each moment's existing content and actions SHALL be unchanged. The reward accent
SHALL be used; success green SHALL NOT be used for a reward.

#### Scenario: Passing a piece celebrates
- **WHEN** a piece step is passed
- **THEN** the celebration surface plays with the pass title and the step's result, and its actions become tappable at the end of the sequence

#### Scenario: A badge celebration keeps its actions
- **WHEN** a badge is earned
- **THEN** the celebration shows the badge's title and message with the same close action as before, animated

#### Scenario: No success green on a reward
- **WHEN** any celebration is shown
- **THEN** its accent is the reward accent and not the success colour used for a correct note

### Requirement: Counters animate between values
The standing pill, the end-of-run points cue and the daily session ring SHALL animate from
their previous value to the new one when the value increases. A decrease SHALL be shown
immediately without animation.

#### Scenario: Points count up after a run
- **WHEN** a run earns twelve points and the pill showed 240
- **THEN** the pill counts from 240 to 252

#### Scenario: A debit shows at once
- **WHEN** points are spent on a streak buy-back
- **THEN** the pill shows the new lower value immediately

### Requirement: The learning path is drawn and animates completion
The learning path SHALL draw a continuous curve through its lesson nodes, filled up to the
last completed node and dashed beyond. Completing a lesson SHALL animate the node's check
then the fill of the segment to the next node. A unit whose lessons are all completed SHALL
light its header once per app session. The next lesson's node SHALL keep its pulse.

#### Scenario: Completing a lesson fills the path
- **WHEN** the learner completes the third lesson of a unit and returns to the path
- **THEN** that node's check animates and the curve fills to the fourth node

#### Scenario: A finished unit lights once
- **WHEN** the last lesson of a unit is completed and the path is shown twice in the same session
- **THEN** the unit header lights on the first showing and not on the second

### Requirement: Haptics follow the moment and the system setting
A light haptic SHALL accompany a pass or a completion celebration on platforms that
support it and when the system haptics setting allows. No haptic SHALL be fired on a miss,
a wrong note or an error.

#### Scenario: A pass vibrates lightly
- **WHEN** a piece step is passed on a device with haptics enabled
- **THEN** one light haptic is fired with the celebration

#### Scenario: A miss stays silent
- **WHEN** a run ends below the threshold
- **THEN** no haptic is fired

### Requirement: Motion never crosses into the play surface's rules
Celebrations and confetti SHALL be drawn over dialogs and the home only, never over a play
surface, and SHALL NOT alter the gamified feedback of a run. The confetti SHALL use a fixed
number of particles with no per-frame allocation.

#### Scenario: No confetti over the notes
- **WHEN** a run is in progress
- **THEN** no celebration surface or confetti is painted over the play surface

#### Scenario: Particle budget is fixed
- **WHEN** the confetti burst plays
- **THEN** the number of particles is the configured constant and does not grow with the sequence
