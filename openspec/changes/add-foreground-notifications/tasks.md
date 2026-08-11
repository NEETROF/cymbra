## 1. Category declaration

- [ ] 1.1 Add a foreground field to `PushCategory` (`lib/state/push_categories.dart`), defaulting to "not surfaced", documented as the per-category decision the platform defers.
- [ ] 1.2 Unit-test the default: a category declaring nothing does not surface.

## 2. Foreground message seam

- [ ] 2.1 Expose the foreground message stream on `PushService` (title, body, `data`), so it is mockable like `tokenRefreshes` — no widget or notifier touches `FirebaseMessaging` directly.
- [ ] 2.2 Implement it in `FirebasePushService` over `FirebaseMessaging.onMessage`, guarded like the rest (no Firebase config ⇒ an empty stream, never a throw).
- [ ] 2.3 Suppress the OS foreground banner on Apple platforms via `setForegroundNotificationPresentationOptions(alert: false, badge: false, sound: false)`, so macOS matches iOS and Android (design D3).

## 3. In-app presentation

- [ ] 3.1 A notifier owning the visible banner state: a message arrives → it is shown only if its category declares foreground presentation; dismiss clears it.
- [ ] 3.2 A dedicated listener widget near the top of the app subtree (the `PushRegistrationListener` shape) subscribing to the stream and handing messages to the notifier — the UI never calls the service.
- [ ] 3.3 The banner widget: title, body, dismissible, tappable. Rendered in the app's own idiom, not as an OS alert.
- [ ] 3.4 Tap routing from the message's `data` payload; a payload-less message dismisses without navigating (design D4).

## 4. Tests

- [ ] 4.1 Notifier: surfaced vs suppressed by category declaration; dismissal clears; an unknown category never surfaces.
- [ ] 4.2 Listener widget over a mocked `PushService` stream: a message for a declaring category reaches the notifier, one for a non-declaring category does not.
- [ ] 4.3 Widget: the banner renders title/body, dismiss removes it, tap routes on a payload and is inert without one.
- [ ] 4.4 Regression: with no category declared (the platform's shipped state) nothing renders and nothing subscribes to a real SDK.

## 5. Gates & verification

- [ ] 5.1 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [ ] 5.2 Manual: with a temporary category declaring foreground presentation, a dispatch while the app is open shows the in-app banner on iOS, Android and macOS — and shows **no** OS banner on macOS; the same dispatch with the app backgrounded still shows the OS notification on all three.
- [ ] 5.3 `openspec validate add-foreground-notifications --strict` passes.
