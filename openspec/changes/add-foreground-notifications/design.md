## Context

`add-push-notifications` shipped the platform: FCM tokens, per-category consent,
recipient selection, worker dispatch. It deliberately shipped **no** presentation
policy for a notification arriving while the app is open, because no concrete
category existed to have an opinion.

The result, verified on real devices during that change's manual pass, is three
different behaviours:

| Platform | App in foreground | App backgrounded/killed |
| --- | --- | --- |
| Android | nothing — FCM routes the message to Dart's `onMessage`, unhandled | system displays |
| iOS | nothing — suppressed unless the app opts in | system displays |
| macOS | system banner shown | system displays |

Background delivery is correct and consistent; only the foreground is a mess. This
change makes the foreground a decision instead of an accident.

## Goals / Non-Goals

**Goals:**
- One consistent foreground behaviour across iOS, Android and macOS.
- The *category* decides whether it surfaces in the foreground, not the app.
- No new native dependency.
- Testable in widget tests, like the rest of the push client.

**Non-Goals:**
- OS-level local notifications (`flutter_local_notifications`) — see D2.
- A notification centre, history, grouping, or rate-limiting.
- Any backend change: the transport, selection and flags stay as they are.
- Changing background delivery in any way.

## Decisions

### 1. The category decides, and the default is silence

`PushCategory` gains a foreground declaration, defaulting to "do not surface".

The motivating case settles it: the practice-streak reminder fires in the evening
to pull a lapsing user back. If the app is in the foreground, that user is *already
practising* — the notification has achieved its purpose and interrupting them with
it is worse than useless. Whereas "your score was just beaten" is only interesting
while it is fresh.

A global switch cannot express that, and defaulting to "show" would make every
future category interrupt by accident. Silence is the safe direction, consistent
with every other gate in this platform.

### 2. In-app banner, not an OS notification

A foreground notification is rendered **inside the app**, not handed back to the OS.

The alternative — display an OS local notification from `onMessage` — is what most
Flutter apps do, and it is worse here on three counts. It needs
`flutter_local_notifications`: a native dependency, an Android channel, an icon,
and its own permission surface on top of the one we already ask for. It produces a
system banner **over the app the notification came from**, which is redundant
chrome. And it is untestable in widget tests, pushing the behaviour into the
platform-channel layer we deliberately keep behind a seam.

An in-app banner is pure Dart: a listener widget, a widget, and a Riverpod
notifier. It costs no dependency, it is covered by the normal widget-test gate, and
it is the better interaction — the app can present the message in its own idiom
rather than as an OS alert.

The trade-off is real and accepted: an in-app banner makes no sound, adds nothing
to the notification centre, and vanishes with the app. For a message that only
matters while the user is looking at the app, that is the point.

### 3. Make the platforms agree by suppressing, not by adding

macOS currently shows a system banner in the foreground; iOS and Android do not.
Rather than teach the other two to show one, macOS is brought down to the common
baseline via `setForegroundNotificationPresentationOptions(alert: false, …)`.

That call is **app-global**, not per-notification — which is exactly why it cannot
be the mechanism for a per-category decision (D1). Used as a baseline it is the
right tool: it makes the OS uniformly silent in the foreground, after which the
in-app layer applies the category's choice. One rule, three platforms.

### 4. Reuse the existing `data` payload for routing

The dispatch payload already carries an opaque `data` map, transported end to end
(`PushMessage::with_data`, the FCM `data` block). A tapped in-app banner reads the
same keys the background tap path will, so a category describes its destination
once.

No new field, no second convention.

### 5. Where it lives

A dedicated listener widget near the top of the app subtree — the same shape as
`PushRegistrationListener` and `LanguageSyncListener` — subscribes to the foreground
message stream and hands it to a notifier that owns the visible banner state. The
UI never touches the push service; the notifier does. This is the repo's
`flutter-riverpod-architecture` rule, not a preference.

The foreground message stream is exposed through the existing `PushService` seam, so
tests drive it with a mock, exactly like token registration.

## Risks / Trade-offs

- **No sound, no notification centre.** An in-app banner is ephemeral. Accepted: a
  foreground message is by definition being delivered to someone who is looking.
  A category that genuinely needs a persistent, sounding alert while the app is
  open is a signal it should be a background notification instead.
- **Banner storms.** Nothing coalesces or rate-limits banners; two categories firing
  together stack or replace. Deliberately out of scope — with no concrete category
  shipped yet, any policy would be invented against imaginary traffic. Revisit when
  a second category exists.
- **macOS behaviour changes.** Users of the current build see a system banner in the
  foreground and will stop seeing one. Since no category exists, nothing is being
  sent, so no real user is affected — this only lands ahead of the first type.
- **Divergence between foreground and background copy.** The same `title`/`body` is
  rendered by two very different surfaces. Accepted: one message, one meaning; a
  category needing different phrasing per surface is over-designing.
