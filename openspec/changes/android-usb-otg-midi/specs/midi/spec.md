## MODIFIED Requirements

### Requirement: Cross-platform Native Integration

The system SHALL operate across macOS, iOS, Android, Linux and Windows via a
single MIDI backend, with the platform-specific wiring required for hot-plug and
device access on each platform. On Android the system SHALL declare USB host
usage and register for USB device-attach events so that plugging in a
class-compliant USB MIDI device routes the attach event to the app and prompts
enumeration.

#### Scenario: macOS/iOS hot-plug delivery
- **WHEN** the app runs on macOS or iOS
- **THEN** a CoreMIDI client is created on the main run loop so configuration-change (hot-plug) notifications keep the process's MIDI view current

#### Scenario: Android native context
- **WHEN** the app runs on Android
- **THEN** the native library is loaded via `System.loadLibrary` so `JNI_OnLoad` initializes the Android context required by the AMidi backend

#### Scenario: Android USB host declared
- **WHEN** the Android app is built
- **THEN** the manifest declares `android.hardware.usb.host` as a non-required feature so the platform recognizes the app operates in USB host mode

#### Scenario: Android USB MIDI device attach
- **WHEN** a class-compliant USB MIDI device is plugged into an Android device with USB host active
- **THEN** the manifest's `USB_DEVICE_ATTACHED` intent-filter, matched by a MIDI-class `device_filter`, offers to open the app and delivers the attach intent to the running activity

#### Scenario: Desktop backends
- **WHEN** the app runs on Linux or Windows
- **THEN** MIDI ports are enumerated and opened via the platform's native backend (ALSA / WinMM) without additional wiring

## ADDED Requirements

### Requirement: No-device Guidance on Android

When running on Android and no MIDI input port is detected, the system SHALL
surface guidance in the UI advising the user to enable USB OTG in the phone
settings and to verify that the cable/adapter supports data transfer. This
guidance SHALL NOT appear on other platforms, and SHALL disappear once a port is
detected.

#### Scenario: Empty port list on Android
- **WHEN** the app runs on Android and the MIDI port list is empty
- **THEN** the UI shows guidance to enable USB OTG (Settings → "OTG") and check the data cable

#### Scenario: Device detected clears guidance
- **WHEN** at least one MIDI port becomes available on Android
- **THEN** the OTG guidance is no longer shown

#### Scenario: Non-Android platforms unaffected
- **WHEN** the app runs on macOS, iOS, Linux, or Windows with no port
- **THEN** the Android-specific OTG guidance is not shown
