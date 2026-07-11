## Context

The player (`apps/music/lib/screens/player_screen.dart`) was built for tablets and
desktop. It composes a fixed-height top bar (`_TopBar`, `EdgeInsets.symmetric(
horizontal: 16, vertical: 12)`, 18/12 px title/subtitle), an `Expanded` render
area, and an on-screen keyboard pinned to a hard-coded `_keyboardHeight = 150`.
`PianoLayout` already derives key widths from the available width, so horizontal
scaling works; the vertical budget does not.

The app is landscape-locked at three layers — Flutter runtime
(`SystemChrome.setPreferredOrientations` in `main.dart`), native iOS, and native
Android — and that lock is already spec'd under `keyboard-display`. So "force
landscape" is largely done; the gap is that a phone's landscape viewport is short
(~330–420 px tall) and the fixed chrome + 150 px keyboard leave the render area
starved and controls cramped.

Auth screens already cap width at 420 px and are fine. Riverpod 2 + Freezed is
mandatory for app state, but device class is derived from `MediaQuery`, not app
state, so it needs no notifier.

## Goals / Non-Goals

**Goals:**
- Make the player usable on smartphone landscape viewports without overflow.
- Derive keyboard height from the viewport, clamped for legibility.
- Compact the top bar chrome on phones; preserve tablet/desktop appearance.
- Provide one shared device-class helper as the single source of truth.
- Keep the existing landscape lock; add explicit smartphone coverage.

**Non-Goals:**
- Portrait layouts or unlocking rotation.
- Redesigning the notation/Synthesia painters' internal typography (only their
  available height changes).
- Tablet/desktop visual redesign — those must not regress.
- Per-widget theming overhaul; only the player chrome and keyboard height adapt.

## Decisions

### 1. Device class from viewport shortest side, via a small helper (not state)
Add a lightweight helper (e.g. `apps/music/lib/layout/device_class.dart`) exposing
`DeviceClass { phone, tablet, desktop }` and a `deviceClassOf(BuildContext)` /
`context.deviceClass` extension that reads `MediaQuery.sizeOf(context).shortestSide`
and applies breakpoints (phone `< 600`, tablet `< 900`, else desktop; desktop/web
platforms resolve to desktop). Because the app is landscape-locked, shortest side
== height, which is the dimension under pressure.

- *Why not a Riverpod provider?* Screen metrics come from `MediaQuery` and rebuild
  the widget tree already; wrapping them in a provider adds indirection with no
  benefit. Project state rules target *app* state, not layout metrics.
- *Why shortest side, not width?* Width is comfortable in landscape; height is the
  constraint. Shortest side is height here and stays correct if the lock is ever
  relaxed.
- *Alternative considered:* `LayoutBuilder` everywhere — rejected as it scatters
  breakpoint math; the spec requires a single source of truth.

### 2. Adaptive keyboard height from the render `LayoutBuilder` constraints
Replace `_keyboardHeight = 150` with a computed height inside the existing
`LayoutBuilder` (`player_screen.dart:224`) that already has `constraints`. Compute
`height = (constraints.maxHeight * fraction).clamp(minH, maxH)` where the fraction
and clamp bounds keep keys legible (min ~96 px so an 88-key keyboard is still
tappable, max ~180 px so tablets don't over-allocate). The same value flows to the
`SizedBox`, the `CustomPaint` size, and `pitchAt` hit-testing (which already takes
height as a parameter), so hit-testing stays consistent automatically.

- *Why fraction-of-height with clamps* over discrete per-class constants? Smooth
  across the device continuum (foldables, split-screen, resized desktop windows)
  and naturally satisfies "shrinks on phones, clamped on tall viewports."
- *Alternative considered:* a per-`DeviceClass` fixed height table — simpler but
  brittle at the phone/tablet boundary and on resizable windows.

### 3. Compact top bar driven by device class
`_TopBar` reads `context.deviceClass` and picks padding + title/subtitle text
sizes from a small token set (phone vs tablet/desktop). Icon buttons keep the
Material 48 px minimum tap target even when visual padding shrinks. The control
cluster already uses `Expanded` + ellipsis for the title, so horizontal overflow
is handled; verify the trailing chip cluster fits on the narrowest phone.

- *Alternative considered:* a fully separate phone top-bar widget — rejected as
  duplication; a couple of size tokens keep one widget.

### 4. Testing at explicit viewport sizes
Widget tests set `tester.view.physicalSize` / `devicePixelRatio` to a phone
landscape (e.g. 812×375) and a tablet (1024×768) and assert: device class resolves
correctly, keyboard height is smaller on phone and within clamp bounds, render
area height stays > 0, and no overflow (`tester.takeException()` is null). This
keeps the native FFI seam untouched (fakes only), honoring the ≥ 80% Flutter gate.

## Risks / Trade-offs

- **Golden drift** → chrome/keyboard dimensions change, so golden tests may need a
  refresh on the pinned platform (`flutter test --tags golden --update-goldens`);
  goldens are excluded from the cross-platform gate, so CI won't block on them.
- **Keyboard too short to play on the smallest phones** → the clamped minimum
  (~96 px) guarantees a floor; if it still crowds the render area on the tiniest
  devices, prefer keyboard legibility (playing is the core loop) and let the render
  area take the remainder.
- **Breakpoint bikeshedding** → 600/900 shortest-side thresholds follow common
  Material guidance; centralizing them in one helper makes future tuning a
  one-line change.
- **Split-screen / resizable windows** → fraction-of-height + `MediaQuery`-driven
  rebuilds handle live resizes without extra work; verified by resizing in a
  widget test.

## Migration Plan

Pure client UI change, no data or API migration. Ship behind normal review; roll
back by reverting the commit. No feature flag needed — tablet/desktop paths are
covered by the "no regression" scenarios and tests.

## Open Questions

- Final clamp bounds and height fraction — tune against a real phone (e.g. iPhone
  landscape) during implementation; the spec fixes the *behavior*, not the exact
  pixels.
- Whether the settings end-drawer needs its own width cap on phones, or if its
  current sizing already fits — verify during implementation and add a token only
  if it overflows.
