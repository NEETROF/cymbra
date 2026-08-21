## ADDED Requirements

### Requirement: Desktop-only update checking

Cymbra Music SHALL check for updates only on the Windows and Linux desktop
builds. On iOS, Android and macOS, where distribution is store-managed, the app
SHALL NOT check, prompt, download or install anything.

#### Scenario: Store-managed platform never checks

- **WHEN** the app starts on iOS, Android or macOS
- **THEN** no update check SHALL be performed and no update user interface
  SHALL be reachable

#### Scenario: Desktop platform checks at launch

- **WHEN** the app starts on Windows or Linux and the last successful check is
  older than 24 hours
- **THEN** the app SHALL fetch the update feed in the background without
  blocking startup

#### Scenario: Check is throttled

- **WHEN** the app starts on Windows or Linux and a check succeeded less than
  24 hours ago
- **THEN** no network request SHALL be made

#### Scenario: Manual check ignores the throttle

- **WHEN** the user triggers a check from the settings entry
- **THEN** the app SHALL check immediately regardless of when the last check ran

#### Scenario: The updater can be disabled remotely

- **WHEN** the runtime feature flag gating the desktop updater is off
- **THEN** the app SHALL neither check nor offer updates

### Requirement: Verify before executing anything

The app SHALL verify a candidate update in a fixed order and SHALL NOT execute
any downloaded file until every step has passed: manifest signature, known
schema, strictly newer version, declared size, then SHA-256 of the downloaded
bytes.

#### Scenario: Signature verification precedes parsing

- **WHEN** an envelope is received
- **THEN** the app SHALL verify the Ed25519 signature over the exact manifest
  bytes against the compiled-in public key selected by the envelope's key id,
  and SHALL parse the manifest only after that verification passes

#### Scenario: Bad signature stops everything

- **WHEN** the signature does not verify, or the key id is not in the app's
  trusted key set
- **THEN** the app SHALL discard the response, download nothing, and record the
  cause in the diagnostic log without surfacing a technical error to the user

#### Scenario: Unknown schema is ignored

- **WHEN** a verified manifest declares a schema version the app does not
  understand
- **THEN** the app SHALL ignore it rather than acting on partially understood
  fields

#### Scenario: No downgrade

- **WHEN** a verified manifest offers a version lower than or equal to the
  running version
- **THEN** the app SHALL take no action, so that replaying an older signed
  manifest cannot move a user backwards

#### Scenario: Checksum mismatch is fatal to the attempt

- **WHEN** the downloaded artifact's SHA-256 does not equal the value in the
  verified manifest, or its length exceeds the declared size
- **THEN** the app SHALL delete the file, SHALL NOT execute it, and SHALL report
  the failure as a retryable update error

#### Scenario: Download is isolated on disk

- **WHEN** an artifact is downloaded
- **THEN** it SHALL be written into a freshly created, uniquely named directory
  inside the application's own temporary storage, not into a shared system
  temporary path

#### Scenario: Only the artifact for the running platform is considered

- **WHEN** a verified manifest carries several targets
- **THEN** the app SHALL select the target matching the running operating system
  and architecture, and SHALL take no action if none matches

### Requirement: Staged rollout is honoured client-side

The app SHALL draw a stable rollout bucket once, persist it locally, and offer a
release only when its rollout percentage covers that bucket.

#### Scenario: Bucket is stable across launches

- **WHEN** the app evaluates a rollout percentage on successive launches
- **THEN** it SHALL use the same locally persisted bucket value each time, so a
  user is not repeatedly moved in and out of a partial rollout

#### Scenario: Outside the rollout

- **WHEN** the release's rollout percentage does not cover the local bucket
- **THEN** the app SHALL behave as if no update were available

#### Scenario: Manual check bypasses the rollout

- **WHEN** the user explicitly checks for updates from settings
- **THEN** the app SHALL offer the verified release regardless of the rollout
  percentage

### Requirement: Silent installation on Windows without elevation

The app SHALL apply an update by running the verified installer silently and
relaunching, without requesting administrator elevation, whenever the running
Windows build was installed by the Cymbra per-user installer.

#### Scenario: Silent update of a per-user install

- **WHEN** the user approves an update on a Windows install carrying the
  installer's marker
- **THEN** the app SHALL start the verified installer detached in silent mode,
  exit, and SHALL be relaunched by the installer once the files are replaced

#### Scenario: No elevation prompt

- **WHEN** an update is applied to a per-user installation
- **THEN** no User Account Control prompt SHALL be shown

#### Scenario: Portable installation cannot self-install

- **WHEN** the running Windows build has no installer marker, as with the
  portable archive
- **THEN** the app SHALL NOT attempt an install and SHALL instead offer to open
  the release page

### Requirement: Self-replacement on Linux AppImage

The app SHALL apply a Linux update by replacing its own file atomically and
relaunching, whenever the running build is an AppImage whose file is writable.

#### Scenario: AppImage replaces itself

- **WHEN** the user approves an update and the running AppImage path is known
  and writable
- **THEN** the app SHALL download the new AppImage into the same directory, make
  it executable, replace the running file with a single atomic rename, relaunch
  it detached, and exit

#### Scenario: Non-AppImage or read-only install

- **WHEN** the running Linux build is not an AppImage, or its file or directory
  is not writable by the current user
- **THEN** the app SHALL NOT attempt an install and SHALL instead offer to open
  the release page

#### Scenario: The updater never escalates privileges

- **WHEN** an install location is not writable
- **THEN** the app SHALL NOT invoke any privilege-escalation mechanism

### Requirement: Update prompts never interrupt playing

The app SHALL NOT present an update prompt while a play or practice session is
in progress, and SHALL NOT install while one is in progress.

#### Scenario: Prompt deferred during a session

- **WHEN** an update becomes available while the user is in a play or practice
  session
- **THEN** the prompt SHALL be withheld until the session ends

#### Scenario: User consent is required to install

- **WHEN** an update is available
- **THEN** the app SHALL NOT download or install it until the user has approved
  it

### Requirement: Dismissed versions are remembered

The app SHALL remember a version the user chose to skip and SHALL NOT offer that
version again, while still offering any later version.

#### Scenario: Skipped version is not re-offered

- **WHEN** the user dismisses an available version and the app checks again
- **THEN** that same version SHALL NOT be presented again

#### Scenario: A newer version is still offered

- **WHEN** a version higher than the skipped one becomes available
- **THEN** the app SHALL present it

### Requirement: Forced update below the minimum supported version

The app SHALL present a blocking, localized screen offering the update when the
running version is lower than the minimum supported version of a verified
manifest, rather than letting the client fail against the backend with errors the
user cannot act on.

#### Scenario: Blocking screen below the floor

- **WHEN** a verified manifest declares a minimum supported version higher than
  the running version
- **THEN** the app SHALL present a blocking localized screen whose only action
  is to update, and SHALL NOT allow it to be dismissed

#### Scenario: The floor is only trusted when signed

- **WHEN** a minimum supported version arrives in a response whose signature did
  not verify
- **THEN** the app SHALL ignore it and SHALL NOT block the user

### Requirement: Update failures never surface raw technical errors

Network, verification and installation failures SHALL be presented, if at all,
as localized messages, with the underlying cause recorded in the diagnostic log.

#### Scenario: Failed background check is silent

- **WHEN** a background update check fails for any reason
- **THEN** the app SHALL continue normally with no user-visible error, and the
  cause SHALL be logged

#### Scenario: Failed manual check is explained

- **WHEN** a user-initiated check or download fails
- **THEN** the app SHALL show a localized message describing the outcome, and
  SHALL NOT display an exception, status code or transport error string
