## 1. The flag (backend)

- [ ] 1.1 Add `category_foreground_key(category)` beside `category_enabled_key` / `category_hour_key` in `cymbra-feature-flags`'s registry, documented as the third per-category key. Default `false` at every call site.
- [ ] 1.2 Unit-test the key shape and that it sits under the `notifications.` prefix, so the back-office panel picks it up with no change.

## 2. Carrying the decision (backend)

- [ ] 2.1 Resolve the flag in `resolve_flags` (or its sibling), keeping it out of `select_recipients` — it decides *presentation*, never *who receives*.
- [ ] 2.2 Have `Dispatcher` attach the resolved value to every message it sends, so a feature cannot forget and scheduled/event-triggered sends behave alike.
- [ ] 2.3 Unit-test with the mocked sender: the attribute is present and correct for on and off, and selection is unaffected either way.

## 3. Foreground message seam (app)

- [ ] 3.1 Expose the foreground message stream on `PushService` (title, body, `data`), mockable like `tokenRefreshes` — no widget or notifier touches `FirebaseMessaging` directly.
- [ ] 3.2 Implement it in `FirebasePushService` over `FirebaseMessaging.onMessage`, guarded like the rest (no Firebase config ⇒ an empty stream, never a throw).
- [ ] 3.3 Suppress the OS foreground banner on Apple platforms via `setForegroundNotificationPresentationOptions(alert: false, badge: false, sound: false)`, so macOS matches iOS and Android (design D3).

## 4. In-app presentation (app)

- [ ] 4.1 A notifier owning the visible banner state: a message is shown only if it *carries* the foreground indication; absent ⇒ silent. It never consults the category list — that is what keeps the policy hot-reloadable.
- [ ] 4.2 A dedicated listener widget near the top of the app subtree (the `PushRegistrationListener` shape) subscribing to the stream and handing messages to the notifier — the UI never calls the service.
- [ ] 4.3 The banner widget: title, body, dismissible, tappable. Rendered in the app's own idiom, not as an OS alert.
- [ ] 4.4 Tap routing from the message's `data` payload; a payload-less message dismisses without navigating (design D4).
- [ ] 4.5 Confirm `PushCategory` needs no new field — presentation is not a client constant.

## 5. Tests

- [ ] 5.1 Notifier: surfaced when the message says so, silent when it does not and when the indication is absent; dismissal clears.
- [ ] 5.2 Listener widget over a mocked `PushService` stream: a message reaches the notifier; a category the app has never declared still surfaces when the message says to.
- [ ] 5.3 Widget: the banner renders title/body, dismiss removes it, tap routes on a payload and is inert without one.
- [ ] 5.4 Regression: with nothing configured (the platform's shipped state) nothing renders and nothing subscribes to a real SDK.

## 6. Gates & verification

- [ ] 6.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80`.
- [ ] 6.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [ ] 6.3 Manual: with a temporary category, flip its foreground flag **in the back office** and confirm already-installed apps change behaviour with no rebuild — banner on iOS, Android and macOS when on, nothing when off, and **no** OS banner on macOS either way. Backgrounded, the OS notification still shows on all three.
- [ ] 6.4 `openspec validate add-foreground-notifications --strict` passes.
