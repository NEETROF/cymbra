## Why

A tester reported that on Android their USB MIDI piano is only detected once the
phone's **USB OTG mode** is enabled. OTG is a system/hardware setting the app
cannot toggle programmatically, so until the USB host stack enumerates the
device, the AMidi backend sees zero ports and the app silently shows nothing.
We cannot force OTG on, but we can make the app trigger enumeration on plug-in
and tell the user what to do when no device is found — turning a silent failure
into an actionable, discoverable flow.

## What Changes

- Declare USB host usage in the Android manifest
  (`<uses-feature android:name="android.hardware.usb.host" android:required="false" />`)
  so the platform knows the app operates in USB host mode.
- Add a `USB_DEVICE_ATTACHED` intent-filter on `MainActivity` plus a
  `res/xml/device_filter.xml` targeting the USB MIDI device class, so Android
  offers to open Cymbra when a MIDI keyboard is plugged in and routes the
  attach event to the running app — which on several OEM devices is what
  actually triggers device enumeration.
- Show an in-app guidance message on Android when no MIDI port is detected,
  inviting the user to enable USB OTG (Settings → search "OTG") and to check
  that the cable/adapter supports data transfer.
- No behavior change on macOS, iOS, Linux, or Windows.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `midi`: extend Android native integration to declare USB host mode and handle
  USB device-attach so plugging a keyboard triggers enumeration; add a
  no-device guidance requirement so the UI surfaces actionable OTG/cable help
  when the port list is empty on Android.

## Impact

- **Android manifest**: `apps/music/android/app/src/main/AndroidManifest.xml`
  (add `uses-feature` + `USB_DEVICE_ATTACHED` intent-filter).
- **New resource**: `apps/music/android/app/src/main/res/xml/device_filter.xml`.
- **Flutter UI/state**: the MIDI device selection surface (empty-state hint),
  gated to Android via a platform check so other platforms are unaffected.
- No changes to the Rust engine's MIDI enumeration logic or public FFI API.
- No new runtime permissions; USB host + MIDI framework need none for
  class-compliant devices.
