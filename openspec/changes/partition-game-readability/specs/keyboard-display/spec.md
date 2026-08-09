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

### Requirement: Adaptive Keyboard Height

The on-screen keyboard height SHALL be derived from the available viewport height
rather than a fixed constant, so the keyboard scales down on small phone
viewports and stays proportionate on tablets/desktops. The computed height SHALL
be clamped between a legible minimum and a maximum so the keys remain playable
without the keyboard dominating the render area, and the render area above the
keyboard SHALL always retain a non-zero, usable height. On a viewport too short
for the fixed minimum (e.g. a shrunken desktop window), the minimum SHALL yield
proportionally so the notation above keeps the large majority of the height —
the keyboard never dominates a crushed render area.

#### Scenario: Keyboard shrinks on small phone viewports
- **WHEN** the player renders on a phone-class landscape viewport shorter than a
  tablet
- **THEN** the keyboard height is smaller than on a tablet/desktop viewport while
  remaining at or above the legible minimum

#### Scenario: Keyboard clamped on tall viewports
- **WHEN** the player renders on a very tall viewport
- **THEN** the keyboard height is clamped to its maximum so it does not dominate
  the render area

#### Scenario: Render area retains usable height
- **WHEN** the keyboard height is computed for any supported viewport
- **THEN** the render area above the keyboard keeps a non-zero, usable height

#### Scenario: The floor yields on very short viewports
- **WHEN** the viewport is too short for the fixed keyboard minimum
- **THEN** the keyboard shrinks below it proportionally and the notation keeps
  the majority of the height
