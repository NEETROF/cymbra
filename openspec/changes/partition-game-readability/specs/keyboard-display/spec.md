# keyboard-display delta — partition-game-readability

## MODIFIED Requirements

### Requirement: Hideable Keyboard In Notation Modes

The user SHALL be able to hide the on-screen keyboard while in the scrolling
staff (Portée) render mode, handing the freed height to the score; the setting
SHALL be reachable from the player settings. In Synthesia the keyboard SHALL
always be shown regardless of the setting, because its cascade aligns to the
keys, and the hide option SHALL NOT be offered in Synthesia. In the engraved
Partition mode the keyboard SHALL NOT be shown at all and the hide option SHALL
NOT be offered there: the notation's own expected-note emphasis already tells
the player what to play, and the freed height is what keeps the current and
next staff lines on screen together. The setting SHALL default to visible and
be session-scoped.

#### Scenario: Hide the keyboard in the scrolling staff
- **WHEN** the user turns the keyboard off while in Staff (Portée) mode
- **THEN** the on-screen keyboard is not rendered and the score takes the freed
  height

#### Scenario: Synthesia always keeps the keyboard
- **WHEN** the mode is Synthesia and the keyboard has been set hidden
- **THEN** the keyboard is still shown, and the hide option is absent from the
  settings

#### Scenario: Partition never shows the keyboard
- **WHEN** the mode is the engraved Partition, whatever the keyboard setting
- **THEN** the on-screen keyboard is not rendered, the hide option is absent
  from the settings, and the engraving takes the full height

#### Scenario: Default is visible
- **WHEN** a session starts
- **THEN** the on-screen keyboard is visible (in the modes that show it)
