## Why

Cymbra's player was laid out for tablets and desktops: the top bar, on-screen
keyboard, and notation area use fixed pixel sizes (e.g. a hard-coded 150 px
keyboard height and 12–18 px chrome padding). In landscape on a smartphone the
usable height is only ~330–420 px, so those fixed chunks crowd out the render
area, controls become cramped, and an 88-key keyboard is barely playable. We want
smartphones to be first-class targets — the app must adapt its layout to the
device's screen size while keeping the existing landscape-only lock.

## What Changes

- Introduce a device-class notion (phone / tablet / desktop) derived from the
  landscape viewport's shortest side, exposed to the UI so widgets can adapt
  without scattering raw `MediaQuery` math.
- Make the player chrome adaptive: the top bar compacts on phones (tighter
  padding, smaller title/subtitle, condensed control cluster) so more vertical
  space goes to the render area.
- Replace the fixed 150 px keyboard height with an adaptive height computed from
  the available viewport height, clamped to legibility bounds, so the keyboard
  stays playable on small phones without dominating tablets/desktops. **BREAKING**
  for the `keyboard-display` spec's fixed-height assumption.
- Ensure interactive targets (buttons, chips, keys) keep a usable minimum size on
  phones and that no player element overflows a small landscape viewport.
- Reaffirm and verify the landscape-only lock on phones (already enforced) as an
  explicit part of smartphone support.

## Capabilities

### New Capabilities
- `responsive-layout`: Device-class breakpoints for the landscape app and the
  adaptive sizing rules for the player chrome (top bar, controls, spacing) so the
  UI fits phone, tablet, and desktop viewports without overflow.

### Modified Capabilities
- `keyboard-display`: The on-screen keyboard height changes from a fixed constant
  to an adaptive, viewport-derived height (clamped for legibility); the
  landscape-lock requirement gains an explicit smartphone scenario.

## Impact

- **Code**: `apps/music/lib/screens/player_screen.dart` (top bar, keyboard height,
  render-area layout), a new responsive helper under
  `apps/music/lib/` (e.g. `layout/` or `theme/`), and the piano keyboard sizing
  in `apps/music/lib/painters/`. Auth screens already constrain width and are
  unaffected.
- **Specs**: new `responsive-layout` spec; delta on `keyboard-display`.
- **Platforms**: primarily mobile (iOS/Android phones); tablet and
  desktop/web behavior must be preserved (no regression).
- **Tests**: widget tests at representative phone/tablet viewport sizes; keep
  Flutter line coverage ≥ 80%. Golden refresh may be needed if chrome dimensions
  change.
