## Context

Cymbra reads MIDI input through `midir`, which on Android uses the NDK **AMidi**
backend (`api/midi.rs`). Enumeration is `MidiInput::ports()`; a watcher thread
auto-connects to the first real device and handles hot-plug. AMidi only surfaces
devices that Android's USB host stack has already enumerated.

A tester found their USB piano is invisible until **USB OTG** is enabled on the
phone. USB OTG is a system/hardware setting: on many OEMs (notably Samsung) it
ships behind a toggle and can auto-disable after inactivity. An app cannot enable
it programmatically. When OTG is off, `ports()` returns empty and the current UI
gives no explanation — a silent dead end.

This change does not try to defeat that constraint. It (a) makes Android route
the plug-in event to the app to encourage enumeration, and (b) explains the
situation to the user when no port is found.

## Goals / Non-Goals

**Goals:**
- Declare USB host mode so Android treats Cymbra as a legitimate USB host app.
- Have Android offer to open / notify the app when a USB MIDI device is attached.
- Give the user actionable, Android-only guidance when no MIDI port is detected.
- Zero change to the Rust engine and to non-Android platforms.

**Non-Goals:**
- Forcing or toggling USB OTG from the app (not possible via public APIs).
- Direct raw USB access or a custom USB-MIDI parser — AMidi/`midir` stays the
  single backend.
- Requesting new runtime permissions (class-compliant MIDI needs none).

## Decisions

### Manifest: `uses-feature usb.host` (required=false) + `USB_DEVICE_ATTACHED`

Add `<uses-feature android:name="android.hardware.usb.host" android:required="false" />`
so the app isn't filtered off non-host devices while still declaring host intent,
and add a `USB_DEVICE_ATTACHED` intent-filter on `MainActivity` referencing a
`res/xml/device_filter.xml`. On plug-in Android matches the filter and prompts
"Open Cymbra with this USB device?"; because `MainActivity` is `singleTop`, an
already-running app receives the attach via `onNewIntent`. On several OEM devices
this attach handshake is what actually powers/enumerates the device so AMidi can
see it.

- **Alternative — raw `UsbManager` + custom MIDI parsing:** rejected; duplicates
  what AMidi already does and adds a second, platform-specific code path.
- **Alternative — do nothing in the manifest, only show a hint:** weaker; the
  attach intent measurably improves detection on some phones, so both are kept.

### `device_filter.xml` targets the USB MIDI class, not vendor IDs

Match on the USB Audio class with the MIDIStreaming subclass (class `1`,
subclass `3`) rather than enumerating vendor/product IDs, so any class-compliant
keyboard matches. This keeps the filter maintenance-free.

- **Alternative — per-vendor `usb-device` entries:** rejected; unbounded list,
  breaks for unlisted keyboards.

### Empty-state guidance lives in the Flutter MIDI selection UI, gated to Android

The hint is a UI concern, not engine behavior: when the port list is empty and
`Platform.isAndroid` (via an injectable seam so tests/other platforms stay
deterministic), render a short message pointing to Settings → "OTG" and the data
cable. It clears as soon as a port appears. No Rust/FFI change.

- **Alternative — surface the hint text from Rust:** rejected; platform-string
  localization and layout belong in Flutter, and it would pollute the FFI.

## Risks / Trade-offs

- **[Attach intent doesn't help on every device]** → OTG remains the user's
  responsibility on phones that gate it; the in-app guidance is the fallback so
  the outcome is still actionable rather than silent.
- **[Re-launch/onNewIntent handling]** → `singleTop` + `taskAffinity=""` already
  set; verify plugging a device while the app is foregrounded doesn't spawn a new
  task or reset navigation. Covered in tasks.
- **[Attach prompt shows even when app is foregrounded]** → On stock Android a
  `singleTop` foreground activity receives the attach silently via `onNewIntent`,
  but several OEMs (e.g. Samsung) re-show the system "Open Cymbra?" dialog on
  every plug-in regardless. Confirmed on a physical device. This cannot be
  suppressed from code while the manifest `USB_DEVICE_ATTACHED` filter is present,
  because the launch-when-closed behavior and the while-open prompt are the same
  mechanism. **Decision (accepted):** keep the filter — the auto-open-when-closed
  affordance is worth the redundant prompt while open. Hot-plug detection while
  the app runs is handled by midir's watcher thread regardless, so the prompt is
  purely a launch affordance, not required for detection.
- **[Platform check hurts testability]** → put the Android check behind the
  existing injectable pattern (like `midiServiceProvider`) so widget tests can
  drive both empty-Android and empty-other states.
- **[device_filter class/subclass mismatch]** → validate the numeric values
  against the USB MIDI class spec; a wrong subclass silently never matches.

## Migration Plan

Additive and reversible. Manifest additions and the new `device_filter.xml` have
no effect on platforms other than Android and require no data migration.
Rollback = revert the manifest/resource/UI edits. No release coordination needed
beyond a normal app build.

## Open Questions

- Should the guidance also appear briefly on first launch on Android (education)
  rather than only on empty-state? Default: empty-state only, to avoid nagging.
- Exact placement of the hint (device dropdown empty row vs. a banner in the
  player settings drawer) — decide during implementation against the existing
  settings drawer UX.
