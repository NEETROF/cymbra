## Context

The app targets iOS, Android, macOS, Windows and Linux, but has **no** notification
infrastructure. We need to reach users when the app is **closed** (an evening streak
reminder), and more engagement notifications will follow — so this is a reusable
platform, not a one-off.

Platform reality (verified): `firebase_messaging` supports **Android, iOS, macOS,
Web — not Windows/Linux**. FCM (HTTP v1) delivers to Android natively and to Apple
platforms (iOS + macOS) by bridging to APNs (an APNs auth key uploaded to Firebase).
So **one** provider (FCM) reaches iOS + Android + macOS with one token type and one
send API. Windows background push needs WNS + MSIX + custom native code (no viable
Flutter plugin); Linux has no OS push service for a closed app. `flutter_local_notifications`
can *display* locally on all desktops, but that only helps while the app runs — it
does not solve the closed-app reminder. Hence: **server push = iOS/Android/macOS via
FCM**; Windows/Linux degrade to in-app only.

The backend already has a **worker** (`cymbra-worker` + `jobs`) and a job registry —
the natural home for scheduled/triggered sends. Per-user locale is already persisted
(a pattern to mirror for timezone).

## Goals / Non-Goals

**Goals:**
- A reusable server-driven push platform: register device tokens, know each user's
  local timezone, respect per-category consent, and send via a mockable seam.
- One provider (FCM HTTP v1) covering iOS + Android + macOS.
- Recipient selection (tokens × consent × flags) as a host-testable core; sends
  driven by the worker (scheduled at a flag hour, or event-triggered).
- Clean seams so a feature adds a **notification type** without touching the platform.

**Non-Goals:**
- Windows (WNS/MSIX) and Linux background push; local/on-device notifications.
- The concrete notification **types** (streak reminder etc. ship with their features).
- Email/SMS; campaign/AB tooling; read receipts/analytics beyond send outcome.

## Decisions

### 1. One provider: FCM HTTP v1 for iOS + Android + macOS
The backend integrates a single API (FCM HTTP v1, OAuth via a Firebase service
account). Apple delivery goes FCM→APNs using the APNs **auth key** uploaded to
Firebase — reusing the iOS APNs identity, extended to macOS (macOS push entitlement
+ signing). The client uses `firebase_messaging` on all three platforms and reports
**FCM tokens** (not raw APNs tokens), so the server stores one token shape.

### 2. Token registry
Table `push_tokens(user_id, token, platform, created_at, last_seen_at)` (token
unique; one user may have several devices). RPCs:
- `RegisterPushToken(token, platform)` — upsert for the caller; refresh `last_seen_at`.
- `UnregisterPushToken(token)` — on logout.
The send path **invalidates** a token when FCM returns `UNREGISTERED`/`INVALID`
(prune dead tokens). Windows/Linux clients never call register.

### 3. Per-user timezone
A `timezone` (IANA name) or UTC offset on the user, set by the client and refreshed
on launch, so a "20:00" schedule fires at the user's local evening. Same persistence
pattern as locale. Recipient selection groups users by local send-hour.

### 4. Consent + preferences + flags
- OS permission is opt-in (client asks; if denied, the user is simply never a
  recipient).
- Server keeps `notification_prefs(user_id, category, enabled)`; a send for a
  category skips users with `enabled = false` (default per category is a product
  choice, e.g. streak reminders on).
- Feature flags (`cymbra-feature-flags`): a **global kill-switch**, a **per-category
  enable**, and a **per-category schedule hour** — all hot-reloadable from the BO.

### 5. `PushSender` seam + recipient-selection core
```
trait PushSender { async fn send(&self, token: &str, msg: &PushMessage) -> SendOutcome; }
// FCM impl = excluded glue; a MockPushSender drives tests.

// Host-testable core: given candidate (user, token, tz) rows + category prefs +
// flags + the current hour, return the tokens to actually send to.
fn select_recipients(candidates, prefs, flags, category, local_hour) -> Vec<Token>
```
The core encodes *all* the "who" logic (kill-switch off? category disabled? user
opted out? right local hour? token present?); the FCM HTTP call and DB reads are
excluded glue. `SendOutcome` distinguishes delivered / retryable / invalid-token so
the caller prunes invalid tokens.

### 6. Worker entry point (types come later)
A generic job (e.g. `push_dispatch`) parameterised by category runs on the worker.
This change ships the **dispatch mechanism + selection** and leaves the *type* (its
schedule cron, its message content, its candidate query) to each feature — e.g.
`add-practice-streak` registers the streak-reminder type: a daily job at the
flag-hour whose candidate query is `streak>0 AND last_played_date < today`. Ordered
vs parallel channel + retry policy follow the existing `jobs` registry conventions.

### 7. Desktop degradation
Windows/Linux clients detect they are not FCM-capable and **skip** token
registration; they are therefore never selected as recipients. The platform exposes
no local-notification path (out of scope). Features keep desktop users informed
**in-app** (e.g. the streak chip; an optional launch-time nudge) — a per-feature
concern, not the platform's.

## Risks / Trade-offs

- **Desktop reminder gap.** Win/Linux users get no app-closed reminder. Accepted:
  the engaged-retention audience is mobile + macOS; building WNS/MSIX now is a
  disproportionate project. Degrade to in-app; revisit if desktop demand appears.
- **FCM/Firebase dependency.** A third-party for delivery + a service account and
  APNs key to manage. Mitigation: `PushSender` is a trait — a second provider (or
  direct APNs) can be added without touching selection/consent logic.
- **Timezone spoofing / drift.** A client-set timezone could misfire a send.
  Mitigation: it only affects *timing* of an opt-in reminder (not security); refresh
  on launch; a user can disable the category.
- **Token lifecycle.** Stale tokens waste sends / leak device presence. Mitigation:
  invalidate on `UNREGISTERED`, unregister on logout, `last_seen_at` for pruning.
- **Consent correctness.** Sending to an opted-out user is a trust breach.
  Mitigation: consent is checked in the covered selection core (unit-tested), not in
  glue; kill-switch is a hard global gate.
