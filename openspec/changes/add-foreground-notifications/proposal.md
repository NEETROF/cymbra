## Why

A notification that arrives while the user is **in the app** currently does one of
three different things depending on the platform: Android drops it into Dart's
`onMessage` where nothing handles it, iOS suppresses it, and macOS shows a system
banner. Nobody chose that — it is three OS defaults leaking through.

Two problems follow. The behaviour is **inconsistent**, so a category that reads
well on macOS is invisible on the other two. And it is **uniform per app** when the
right answer is per *category*: an evening "you're about to lose your streak"
reminder is absurd while the user is practising — they are doing the thing — but
"someone just beat your score" is worth surfacing the moment it happens.

The push platform (`platform-push-notifications`) deliberately left this out: with
no concrete notification type declared, there was nothing to decide. The first
types are now coming, so the decision needs a home.

## What Changes

- **Per-category foreground behaviour** — a category declares whether an arriving
  notification is surfaced while the app is in the foreground. Default: **not**
  surfaced, which is today's behaviour on iOS/Android and the safer choice.
- **In-app presentation, not a system banner** — a foreground notification is shown
  as an **in-app** surface (a dismissible banner inside the app), not as an OS
  notification. The user is already looking at the app; a system banner over the app
  it came from is redundant, and this avoids a native local-notification dependency
  entirely.
- **One consistent baseline across platforms** — macOS is brought in line with iOS
  and Android by suppressing the system banner while the app is in the foreground,
  so all three platforms behave identically and the *category* decides.
- **Tap routing** — an in-app banner honours the same `data` payload the platform
  already transports (e.g. `{"route": "/practice"}`), so tapping it goes where the
  system notification would have.
- **The message is already localized** by the feature that owns the category (the
  platform transports copy, it does not translate), so the banner renders it as-is.

Out of scope: OS-level local notifications (`flutter_local_notifications`) and any
new native dependency; a notification centre or history; grouping, rate-limiting or
coalescing of foreground banners; changing anything about background delivery.

## Capabilities

### New Capabilities
<!-- None. This extends the existing push platform. -->

### Modified Capabilities
- `platform-push-notifications`: gains foreground presentation — a per-category
  declaration, an in-app banner as the presentation surface, a consistent
  cross-platform baseline, and tap routing through the existing `data` payload.

## Impact

- **App** (`apps/music`): `PushCategory` gains a foreground field; a foreground
  message listener (a dedicated listener widget, per the Riverpod rules) that
  reacts to `onMessage`; an in-app banner widget; the Apple foreground presentation
  options set to suppress, so macOS matches iOS.
- **Backend**: none. The wire format, the selection core, the dispatcher and the
  flags are untouched — this is entirely a client presentation concern.
- **Back office**: none.
- **Coverage**: widget/state tests over the faked push seam — banner shown vs
  suppressed by category, tap routing, dismissal, and the default (no declaration ⇒
  nothing shown). No new native code, so nothing lands outside the measured surface.
