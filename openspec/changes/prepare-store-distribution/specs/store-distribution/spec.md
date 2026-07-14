## ADDED Requirements

### Requirement: Branded app icon on both platforms

The app SHALL present the Cymbra brand icon (not the default Flutter logo) on iOS and Android, generated from a single checked-in source asset so both platforms stay in sync.

#### Scenario: iOS home-screen icon

- **WHEN** the app is installed on an iOS device
- **THEN** the home-screen and App Store icon SHALL be the Cymbra icon at every required size, with no alpha channel on the 1024×1024 marketing icon

#### Scenario: Android launcher icon

- **WHEN** the app is installed on an Android device
- **THEN** the launcher SHALL display the Cymbra icon across all density buckets, and SHALL render a correct **adaptive icon** (separate foreground and background layers) on Android 8.0+ (API 26+)

#### Scenario: Icons regenerate from source

- **WHEN** a developer runs the configured `flutter_launcher_icons` generator against the source asset
- **THEN** all iOS `AppIcon.appiconset` entries and Android mipmap/adaptive resources SHALL be regenerated, and no default Flutter icon SHALL remain in the tree

### Requirement: Android release build has network access

The Android application's main manifest SHALL declare `android.permission.INTERNET` so that release builds can perform network I/O.

#### Scenario: Release build reaches the backend

- **WHEN** a release-signed Android build starts and issues a gRPC call to `api.cymbra.app:443`
- **THEN** the call SHALL be permitted by the OS (the INTERNET permission is present in the merged release manifest), rather than failing with a security exception

### Requirement: Apple privacy manifest

The iOS Runner bundle SHALL include a `PrivacyInfo.xcprivacy` manifest that declares required-reason API usage and collected data types accurately for the app's dependencies.

#### Scenario: App Store accepts the upload

- **WHEN** the IPA is uploaded to App Store Connect
- **THEN** it SHALL NOT be rejected for a missing privacy manifest, and the declared required-reason APIs SHALL cover the usage introduced by `flutter_secure_storage` and `shared_preferences`

#### Scenario: Manifest is bundled

- **WHEN** the release IPA is inspected
- **THEN** `PrivacyInfo.xcprivacy` SHALL be present in the Runner target's bundle resources

### Requirement: Branded splash screen

The app SHALL show a Cymbra-branded launch/splash screen instead of the default blank screen on both platforms.

#### Scenario: Launch shows branding

- **WHEN** the app is cold-started on iOS or Android
- **THEN** the launch screen SHALL display the Cymbra splash (correct in both light and dark mode) rather than a plain white/black screen

### Requirement: Store-listing assets

The repository SHALL contain the marketing image assets required by both stores, meeting each store's size and format rules, sourced from the app's actual landscape UI.

#### Scenario: iOS screenshots

- **WHEN** preparing the App Store listing
- **THEN** landscape screenshots SHALL be available for the required 6.7" iPhone and 12.9" iPad display sizes

#### Scenario: Android screenshots and graphics

- **WHEN** preparing the Google Play listing
- **THEN** at least two phone screenshots, a 512×512 hi-res icon, and a 1024×500 feature graphic SHALL be available in the required formats

### Requirement: Store-listing copy

The repository SHALL contain version-controlled store-listing text so the listing is reproducible and reusable by automation.

#### Scenario: Listing text is present

- **WHEN** submitting either store listing
- **THEN** the checked-in copy SHALL provide an app description, a subtitle/short promo, iOS keywords, and the chosen store category
