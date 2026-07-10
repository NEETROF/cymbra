## 1. Android manifest & USB device filter

- [x] 1.1 Add `<uses-feature android:name="android.hardware.usb.host" android:required="false" />` to `apps/music/android/app/src/main/AndroidManifest.xml`
- [x] 1.2 Create `apps/music/android/app/src/main/res/xml/device_filter.xml` matching the USB MIDIStreaming class (class 1, subclass 3)
- [x] 1.3 Add a `USB_DEVICE_ATTACHED` intent-filter + `meta-data` referencing `@xml/device_filter` on `MainActivity`
- [x] 1.4 Verify a plugged device while the app is foregrounded reaches `onNewIntent` without spawning a new task (relies on existing `singleTop` + `taskAffinity=""`)

## 2. In-app OTG guidance (Flutter)

- [x] 2.1 Add an Android platform check behind an injectable seam (provider), so widget tests can drive Android/non-Android states deterministically
- [x] 2.2 In the MIDI device selection surface, render the OTG/data-cable guidance when the port list is empty and the platform is Android
- [x] 2.3 Ensure the guidance clears once at least one port is detected and never shows on macOS/iOS/Linux/Windows
- [x] 2.4 Add the guidance string(s) to the localization resources used by the app

## 3. Tests

- [x] 3.1 Widget test: empty port list on Android shows the OTG guidance
- [x] 3.2 Widget test: guidance hidden when a port is present, and hidden on non-Android platforms
- [x] 3.3 Keep Flutter line coverage ≥ 80% for the touched code

## 4. Verification & docs

- [x] 4.1 Manual check on a physical Android device: plug a class-compliant USB MIDI keyboard, confirm the attach prompt appears and the device is detected _(verified on-device: attach prompt + in-app OTG guidance both shown; prompt-while-foregrounded is accepted OEM behavior — see design Risks)_
- [x] 4.2 `melos run analyze` + `dart format` clean; `flutter test` passes
- [x] 4.3 `openspec validate android-usb-otg-midi --strict` passes
