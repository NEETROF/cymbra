## Why

We're building a retention loop (starting with practice streaks) that needs to
reach users **when the app is closed** — an evening "you're about to lose your
streak" reminder is worthless if it only fires while the app is open. There will be
**more** engagement notifications after that (leaderboard beaten, re-engagement,
new content). The app has no notification infrastructure today. Rather than bolt a
one-off reminder onto the streak feature, this change builds a **reusable
server-driven push platform** that later features declare a *notification type*
against.

## What Changes

- **Device-token registry** — a client registers/refreshes its **FCM token** (RPC);
  the server stores it per user + platform and invalidates it on logout / on a
  send-time "unregistered" error. Only the FCM-capable platforms register:
  **iOS, Android, macOS** (macOS is reached via FCM→APNs). Windows/Linux do **not**
  register (see below).
- **Per-user timezone** — stored so scheduled sends fire at the user's **local**
  time (e.g. an 8pm reminder), mirroring the existing locale-persistence pattern.
- **Consent + per-category preferences** — OS notification permission is opt-in;
  the server keeps a per-user, per-**category** preference (e.g. "streak reminders"
  on/off) so a send job never messages a user who opted out. A global kill-switch +
  per-category enable live as feature flags.
- **Send seam (`PushSender`)** — a backend port with an **FCM (HTTP v1)**
  implementation that reaches iOS + Android + macOS through the **single** FCM API
  (Android natively, Apple via the APNs auth key uploaded to FCM). A host-testable
  core decides **who** receives a given send (tokens × consent × flags); the actual
  FCM HTTP call is coverage-excluded glue. `PushSender` is a trait so senders are
  mockable.
- **Scheduled + triggered sends via the worker** — notification types run either on
  a schedule (a `cymbra-worker` job at a flag-configured local hour) or in response
  to an event; this change ships the **platform + one worker entry point**, not the
  types themselves (each feature adds its type).
- **Back office** — a notifications panel: global kill-switch, per-category enable,
  and the schedule-hour flags (hot-reloadable).
- **Desktop degradation** — Windows/Linux have no reliable app-closed push (no FCM;
  WNS/MSIX is out of scope, Linux has no OS push service). They **do not register a
  token** and are never targeted by server sends; the streak etc. stay visible
  in-app, with an optional in-app nudge left to each feature. `flutter_local_notifications`
  (local, all platforms) is **not** part of this change.

Out of scope: Windows (WNS/MSIX) and Linux background push; local/on-device
notifications; email/SMS channels; the concrete notification **types** (streak
reminder etc. ship with their features); rich campaign/AB tooling.

## Capabilities

### New Capabilities
- `platform-push-notifications`: the server-driven push platform — an FCM device-token
  registry (iOS/Android/macOS), per-user timezone, per-category consent +
  flag-gated enablement, a mockable `PushSender` port with an FCM implementation, a
  recipient-selection core, and a worker entry point for scheduled/triggered sends.
  Windows/Linux register no token and are never targeted.

### Modified Capabilities
<!-- None. This is a new platform capability; features layer notification types on top. -->

## Impact

- **Backend**: a token registry (table + RPC register/refresh + invalidate); a
  per-user timezone column; a per-category consent/preference store; the
  `PushSender` trait + FCM HTTP v1 client (new dependency / config: FCM service
  account + APNs auth key in Firebase); a recipient-selection core; a worker job
  entry point; feature flags (kill-switch, per-category, schedule hour).
- **App** (`apps/music`): `firebase_messaging` dependency; request permission
  (opt-in) and register/refresh the FCM token on iOS/Android/macOS via an injectable
  service seam (only notifiers call it); a per-category preference toggle in
  settings; **no** token registration on Windows/Linux.
- **Back office**: a notifications panel (kill-switch, categories, schedule hours).
- **Platform setup**: Firebase project + FCM HTTP v1 credentials; APNs auth key
  uploaded to Firebase (reuses the iOS APNs identity, extended to macOS); macOS push
  entitlements + signing.
- **Coverage**: Rust ≥ 80% for the recipient-selection + consent/flag cores
  (host-testable); the FCM HTTP client + gRPC/route glue coverage-excluded. App ≥ 80%
  via a faked push service seam; the Vue panel under its test setup.
