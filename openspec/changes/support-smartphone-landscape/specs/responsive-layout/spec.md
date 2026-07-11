## ADDED Requirements

### Requirement: Device-Class Breakpoints

The app SHALL classify the current landscape viewport into a device class —
**phone**, **tablet**, or **desktop** — derived from the viewport's shortest side
(height in landscape). The classification SHALL be exposed to the UI through a
single shared helper so widgets adapt from one source of truth rather than
duplicating raw `MediaQuery` math. The thresholds SHALL treat a shortest side
below the tablet breakpoint as a phone, and a viewport at or above the desktop
breakpoint (or a desktop/web platform) as desktop.

#### Scenario: Phone-sized landscape viewport
- **WHEN** the app runs with a landscape viewport whose shortest side is below the
  tablet breakpoint (e.g. a 812×375 phone)
- **THEN** the device class resolves to **phone**

#### Scenario: Tablet-sized landscape viewport
- **WHEN** the app runs with a landscape viewport whose shortest side is at or
  above the tablet breakpoint but below the desktop breakpoint (e.g. a 1024×768
  tablet)
- **THEN** the device class resolves to **tablet**

#### Scenario: Desktop viewport
- **WHEN** the app runs on macOS/Linux/Windows/web or a viewport at or above the
  desktop breakpoint
- **THEN** the device class resolves to **desktop**

#### Scenario: Single source of truth
- **WHEN** a widget needs to adapt to device size
- **THEN** it reads the shared device-class helper rather than recomputing
  breakpoints inline

### Requirement: Adaptive Player Chrome

The player's non-render chrome — the top bar and its control cluster — SHALL adapt
its dimensions to the device class so that on phones it consumes less vertical and
horizontal space, leaving more room for the render area, while preserving the
tablet/desktop appearance. On phones the top bar SHALL use reduced padding and
reduced title/subtitle type sizes, and the control cluster SHALL remain reachable
without overflow.

#### Scenario: Compact top bar on phones
- **WHEN** the player renders on a phone-class viewport
- **THEN** the top bar uses reduced padding and smaller title/subtitle text than
  on tablet/desktop, freeing vertical space for the render area

#### Scenario: Tablet/desktop chrome preserved
- **WHEN** the player renders on a tablet- or desktop-class viewport
- **THEN** the top bar keeps its existing padding and type sizes with no
  regression

#### Scenario: Controls never overflow
- **WHEN** the player renders on the smallest supported phone landscape viewport
- **THEN** all top-bar controls (back, MIDI status, tempo chip, settings) remain
  visible within the bar without horizontal overflow

### Requirement: Usable Touch Targets On Phones

Interactive controls in the player SHALL retain a usable minimum touch-target
size on phone-class viewports, even when chrome is compacted, so controls stay
operable on small screens.

#### Scenario: Minimum control size maintained
- **WHEN** the player compacts its chrome on a phone-class viewport
- **THEN** each interactive top-bar control keeps a touch target of at least the
  platform-minimum tappable size

### Requirement: No Overflow On Small Landscape Viewports

The player screen SHALL lay out without vertical or horizontal overflow on the
smallest supported phone landscape viewport, with the render area, keyboard, and
chrome all fitting within the available height.

#### Scenario: Fits the smallest supported phone
- **WHEN** the player renders on the smallest supported phone landscape viewport
- **THEN** no layout overflow occurs and the render area retains a non-zero,
  usable height above the keyboard
