## MODIFIED Requirements

### Requirement: Landscape-Locked Orientation

The app SHALL run in landscape orientation only; portrait SHALL be unavailable on
mobile devices. The lock SHALL be enforced both by the Flutter runtime and by the
native iOS and Android configuration. On platforms without device orientation
(desktop/web) the lock SHALL be a no-op with no regression. The lock SHALL apply
equally to smartphones, which are a supported target.

#### Scenario: Portrait is disabled on mobile
- **WHEN** the app runs on a phone or tablet and the device is rotated to portrait
- **THEN** the UI remains in landscape

#### Scenario: Desktop unaffected
- **WHEN** the app runs on macOS/Linux/Windows/web
- **THEN** orientation locking is a no-op and the app renders normally

#### Scenario: Smartphone stays landscape
- **WHEN** the app runs on a smartphone held in portrait
- **THEN** the UI remains in landscape and the player lays out for the phone's
  landscape viewport

## ADDED Requirements

### Requirement: Adaptive Keyboard Height

The on-screen keyboard height SHALL be derived from the available viewport height
rather than a fixed constant, so the keyboard scales down on small phone
viewports and stays proportionate on tablets/desktops. The computed height SHALL
be clamped between a legible minimum and a maximum so the keys remain playable
without the keyboard dominating the render area, and the render area above the
keyboard SHALL always retain a non-zero, usable height.

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
