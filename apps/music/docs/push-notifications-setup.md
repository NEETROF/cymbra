# Push notifications — one-time platform setup

The app code for push notifications is complete and inert: with no Firebase
configuration present, `FirebasePushService` reports the platform as unsupported,
the device registers no token, and everything else works normally. This document
is the **manual** setup that turns it on. It needs a Firebase project, an Apple
Developer account and a Google Play/Firebase console — none of it can be scripted
from the repo.

Server side is separate and equally required: see
`backend/notifications/README.md` and `CYMBRA_FCM_SERVICE_ACCOUNT_JSON`.

> **Until step 2 is done, signed iOS and macOS builds fail.** The committed
> `aps-environment` entitlement requires the Push Notifications capability on the
> App ID *and* provisioning profiles regenerated with it — including the profiles
> stored as CI secrets, so the **tagged release workflow breaks** too. If you need
> to ship before setting Firebase up, either enable the capability (step 2.1 + 2.4
> alone are enough — no Firebase project needed) or temporarily remove the
> `aps-environment` key from the entitlements files.

## 1. Firebase project

1. Create (or reuse) a Firebase project for Cymbra.
2. Register **two** apps in it — not three:
   - **Android** — package name `com.cymbra.music` (the Gradle `applicationId`,
     *not* the `org.cymbra.music` namespace).
   - **Apple** — bundle id `com.cymbra.music`. iOS and macOS share this bundle id
     here, and Firebase requires bundle ids to be unique inside a project, so
     there is **one** Apple app covering both. Its single
     `GoogleService-Info.plist` is used by both platforms, and the APNs key
     uploaded against it serves both.
3. Download the config files and drop them in:
   - `android/app/google-services.json`
   - `GoogleService-Info.plist` — the **same file** for iOS and macOS, added to
     each project's Runner **target**, not merely copied into the folder.

   Drag it onto the Runner group in Xcode with "Copy items if needed" + the
   Runner target ticked. Xcode decides where the file physically lands (the
   project root, `ios/`, is as valid as `ios/Runner/`); what matters is that it
   ends up in the target's **Copy Bundle Resources** phase, otherwise it is not
   in the built bundle and `Firebase.initializeApp()` fails — which this app
   degrades to "push unsupported", silently. Verify with:

   ```bash
   grep -c "GoogleService-Info.plist in Resources" ios/Runner.xcodeproj/project.pbxproj
   grep -c "GoogleService-Info.plist in Resources" macos/Runner.xcodeproj/project.pbxproj
   ```

   Each must print `1`. Delete any stray copy Xcode did not reference — a
   duplicate that is never read only invites editing the wrong one later.

   These are **not secrets** (they ship inside the app binary and only identify
   the Firebase project), so committing them is the usual choice — nothing in
   `.gitignore` excludes them today. The *server* credential in step 4 **is** a
   secret and must never be committed.

## 2. Apple (iOS + macOS, both reached via FCM→APNs)

1. In the Apple Developer portal, enable **Push Notifications** on the
   `com.cymbra.music` App ID — for **both** the iOS and macOS App IDs. Regenerate
   the provisioning profiles afterwards (Xcode does this automatically with
   automatic signing).
2. Create an **APNs auth key** (`.p8`, Keys → new key with the Apple Push
   Notifications service enabled). One key serves iOS *and* macOS, and it never
   expires — prefer it to certificates.
3. Upload the key to Firebase: *Project settings → Cloud Messaging → Apple app
   configuration → APNs Authentication Key*. Supply the key file, its Key ID and
   your Team ID. One upload against the single Apple app covers iOS **and**
   macOS.
4. **Regenerate the provisioning profiles** and update the CI secrets. The
   release workflow signs **manually** (`CODE_SIGN_STYLE = Manual`,
   `Apple Distribution`) with the profiles named by
   `IOS_PROVISIONING_PROFILE_NAME` / the macOS equivalent. A profile issued
   before the capability was enabled does not carry `aps-environment`, and the
   tagged release archive fails to sign against it.
5. Entitlements are already wired, and the APNs environment follows the build:
   - `ios/Runner/Runner.entitlements` → `aps-environment = $(APS_ENVIRONMENT)`,
     defaulted to `development` in `ios/Flutter/{Debug,Release}.xcconfig`. The
     tagged release job appends `APS_ENVIRONMENT = production` to
     `Release.xcconfig` next to the manual-signing settings, so only the App
     Store archive claims production APNs (an upload with `development` is
     rejected). Note the Profile configuration also reads `Release.xcconfig`,
     which is why the checked-in default is the development value.
   - `macos/Runner/DebugProfile.entitlements` → `development`
   - `macos/Runner/Release.entitlements` → `production` (macOS already has one
     entitlements file per configuration, so nothing dynamic is needed).

   **The macOS key is `com.apple.developer.aps-environment`, not the bare
   `aps-environment` iOS uses.** Sign macOS with the iOS spelling and `codesign`
   drops the key without a word: the app builds, installs, runs — and never
   receives an APNs token, so `getToken()` fails and no device ever registers.
   Check the signed binary, not the source file:

   ```bash
   codesign -d --entitlements - --xml build/macos/Build/Products/Debug/music.app \
     | plutil -convert xml1 -o - - | grep -A1 aps-environment
   ```
6. macOS only: the app is sandboxed and already has
   `com.apple.security.network.client`, which is what APNs needs. No extra key.

## 3. Android — already wired

`google-services.json` is committed and the Google Services Gradle plugin is
applied (`android/settings.gradle.kts` + `android/app/build.gradle.kts`). The
plugin turns that JSON into the resources `firebase_core` reads at startup; it
**fails the build when the file is missing**, which is why the two land together.

Nothing to do for the Android 13+ runtime `POST_NOTIFICATIONS` permission either:
`firebase_messaging` declares it and `PushService.requestPermission()` triggers
the prompt. Verified on a debug build — the merged manifest carries the
permission, `FirebaseInitProvider` and `FirebaseMessagingService`.

To re-check after changing the Firebase project, build once and read the
generated values:

```bash
grep -E "project_id|gcm_defaultSenderId" \
  apps/music/build/app/generated/res/processDebugGoogleServices/values/values.xml
```

The `gcm_defaultSenderId` must match `GCM_SENDER_ID` in the Apple plist — a
mismatch means the two apps were registered in different Firebase projects.

## 4. Server credential (what actually sends)

*Project settings → Service accounts → Generate new private key* produces a JSON
key file. Its whole contents go into the **worker's** environment as
`CYMBRA_FCM_SERVICE_ACCOUNT_JSON`, on one line:

```bash
CYMBRA_FCM_SERVICE_ACCOUNT_JSON='{"type":"service_account","project_id":"...","client_email":"...","private_key":"-----BEGIN PRIVATE KEY-----\n..."}'
```

**The single quotes are required.** Without them the dotenv parser treats the
JSON's own double quotes as quoting and strips them, so the worker gets
`{type:service_account,...}` and refuses to start:
`invalid FCM service-account JSON: key must be a string at line 1 column 2`.

This one **is a secret** — it can send a notification to any of your users. Keep
it out of git; it belongs with the deployment secrets alongside the DB URLs.
Unset, the `push_dispatch` job logs and completes as a no-op.

## 5. Windows / Linux — deliberately nothing

Neither platform has a reliable app-closed push path (Windows would need WNS +
MSIX and custom native code; Linux has no OS push service). `currentPushPlatform()`
returns `null` there, so those clients never request permission and never register
a token, and the server never targets them. Features keep desktop users informed
in-app instead.

## 6. Verify

With the config in place:

1. Sign in on iOS, Android and macOS. Each should prompt for notification
   permission once and then register — check the server:
   ```sql
   SELECT user_id, platform, last_seen_at FROM user_account.push_tokens;
   ```
2. The device's timezone should land on the account:
   ```sql
   SELECT id, timezone FROM user_account.users WHERE timezone IS NOT NULL;
   ```
3. Turn the `notifications.enabled` flag on in the back office, declare a test
   category, and enqueue a `push_dispatch` job for it (see
   `backend/notifications/README.md`).
4. Sign out — the token row should disappear.
5. Run on Windows or Linux: no permission prompt, and no new `push_tokens` row.
