## Context

The app has never asked the OS to keep the screen on. Adding that is a handful of
lines; deciding *where* the request lives, *how long* it lasts and *who* is allowed
to make it is the part worth writing down, because the naive versions all fail in
ways that only show up on a real device.

Constraints inherited from the repo:

- **Five platforms.** Android, iOS, macOS, Windows, Linux — and they do not behave
  alike. Android's `FLAG_KEEP_SCREEN_ON` is a *window* flag: it applies only while
  the window is in front and evaporates on its own when the app backgrounds. iOS's
  `isIdleTimerDisabled` is app-scoped and likewise inert in the background. The
  three desktop platforms are the opposite: `IOPMAssertionCreateWithName`,
  `SetThreadExecutionState` and the freedesktop screensaver inhibitor are all
  **process-level power assertions that survive losing focus**. A design that
  leaned on the mobile platforms' automatic release would let a minimised Cymbra
  pin a laptop awake.
- **The layering rules** (CLAUDE.md): UI never calls a service; only notifiers do.
  So a screen cannot call the wakelock plugin, and the count cannot live in a
  widget.
- **The seam convention**: native plugins sit behind an injectable Riverpod
  provider so unit and widget tests run on the Dart VM with nothing native loaded
  (`midi_service.dart` is the reference shape).
- **Two of the four play surfaces are `ConsumerWidget`** — `DrumCalibrationScreen`
  and `MidiMonitorScreen` have no `State` to hang an acquire/release off.
- **80% coverage gate**, and the play surfaces are covered by widget tests today.

## Goals / Non-Goals

**Goals:**

- The screen stays lit for the whole visit to a play surface, whether or not
  playback is running.
- The hold is bounded: released on leaving the surface, and released whenever the
  app is not in front — including on the desktop platforms, where nothing releases
  it for us.
- Stacked and nested play surfaces cannot drop each other's hold.
- Every rule above is testable on the Dart VM, with no plugin loaded.
- A platform that cannot honour the request degrades to today's behaviour, silently.

**Non-Goals:**

- **No user setting.** Decided with the user: implicit, like every sheet-music app.
- **No gating on `isPlaying`.** Explicitly rejected — see Decisions.
- **Not background audio.** This is about the screen, not about continuing to play
  while the app is away; the existing `_AudioLifecycleObserver` still silences the
  synth on background and that does not change.
- **No new Android permission and no new Apple entitlement.** If the implementation
  ever needs one, the design is wrong.
- **Not a partial/CPU wakelock.** Only the screen.

## Decisions

### D1 — `wakelock_plus`, not a hand-rolled platform channel

`wakelock_plus` is the maintained successor to the discontinued `wakelock`, is a
federated plugin covering all five of our targets, and picks the right primitive on
each one (window flag on Android, idle timer on iOS, power assertions on desktop).
Writing that by hand means five native implementations and five sets of platform
edge cases, for a feature that is a comfort, not a differentiator.

*Alternative considered:* a MethodChannel in the existing native shells. Rejected —
we would be re-implementing a well-tested plugin, and the repo already carries
native shell patches that are painful to keep (`macos-build-pollutes-native-shells`).

*Alternative considered:* Android-only, since the reports came from phones.
Rejected — the same silence-of-touches applies on iPad and on a laptop propped on a
piano, and the plugin costs nothing extra to wire on the other four.

### D2 — The hold spans the surface's lifetime, not the run

Gating on `isPlaying` is the tempting version and it is wrong for this app. Wait
Mode *stops the playhead* while the player works out an onset; a stopped playhead is
not an idle player, it is a struggling one. Reading a bar before pressing play, and
stopping to re-read after a mistake, are the same. A hold that tracked `isPlaying`
would black out the screen in exactly the moments the player most needs to see it.

The cost is bounded by the scope: four screens, foreground only. A device left on
the player screen and abandoned stays lit, which is the same bargain every score
and tablature reader makes.

### D3 — A counted hold in a notifier, not a boolean on a screen

The state is a **count of mounted play surfaces**, plus whether the app is in front:

```
ScreenWakeState(holders: int, foreground: bool)
  bool get shouldHold => holders > 0 && foreground;
```

A boolean breaks on the stacks we already have — the measure selector opens over the
player, and a lesson can host a player — where the inner screen's `dispose` would
release a hold the outer screen still needs. The notifier owns `acquire()`,
`release()` and `setForeground()`, and calls the service **only when `shouldHold`
flips**, so a stack of five surfaces is one platform call, not five.

`foreground` is part of the state rather than a separate listener because the two
inputs must be resolved together: the answer to "should the screen be held" is a
function of both, and splitting them across two owners is how you get a hold that
is re-taken on resume for a screen that has since been popped.

### D4 — A wrapper widget is what acquires, not the screens themselves

A `KeepScreenAwake({required Widget child})` `ConsumerStatefulWidget` acquires in
`initState` and releases in `dispose`. Each of the four play surfaces wraps its
body in it.

This is the only shape that works uniformly: two of the four screens are
`ConsumerWidget` with no `State`, and converting them to stateful widgets just to
own a lifecycle hook is a worse change than adding one wrapper. It also keeps the
layering rule intact — the widget talks to the *notifier*, never to the service —
and it puts the acquire/release on Flutter's own mount lifecycle, which already
handles every way a screen can go away (pop, replace, tab switch, hot reload).

The notifier reference is **captured in `initState`**, not read in `dispose`:
reading a provider while the element tree is being torn down is how you get a
"tried to read a disposed provider" crash on app exit.

*Alternative considered:* a `NavigatorObserver` matching route names. Rejected —
it turns "is this a play surface" into a string-matching rule maintained far from
the screens, and silently stops working when a route is pushed without a name.

### D5 — Foreground tracking extends the observer that already exists

`_AudioLifecycleObserver` (`main.dart:123`) already computes exactly the split this
needs: `paused`/`hidden`/`detached` are background, `resumed` is foreground, and
`inactive` is deliberately *not* treated as background so a brief focus change does
not chop a sounding note. That last exclusion is right here too — pulling down the
Android notification shade, or a permission dialog appearing mid-practice, is
`inactive`, and dropping the hold there would dim the screen the moment the player
dismisses the dialog.

So this becomes a second concern in the same observer, which is renamed to say what
it now covers. One observer, one place where "is Cymbra in front" is decided.

### D6 — The thin plugin adapter stays thin, and is not excluded from coverage

`WakelockPlusScreenWake` is the only untestable code here — a try/catch around one
plugin call. Following the repo's rule (keep pure logic in host-testable modules), it
stays ~10 lines beside the abstract seam, and every decision worth testing — the
count, the foreground interaction, the flip-detection — lives in the notifier where
widget and unit tests reach it with a mockito mock. At that size it needs no entry in
the coverage-exclusion globs, so `.github/workflows/music-check.yml` is unchanged.

### D7 — Failures are swallowed

Wrapped in try/catch with a `debugPrint`, matching `offline_key_provider.dart`. Two
concrete cases make this load-bearing rather than defensive habit: the Linux
implementation talks to the freedesktop screensaver over D-Bus, and CI runs the
integration suite on **Linux desktop under Xvfb**, where no screensaver service
answers; and a headless or restricted environment can refuse the assertion outright.
Neither is a reason to fail a practice session, and per the spec no error reaches the
player.

## Risks / Trade-offs

- **A device left on the player screen never sleeps** → Accepted, and bounded: four
  screens, foreground only. It is the same bargain as every score reader, and the
  alternative (an inactivity timer of our own) re-introduces the bug for the player
  who is playing rather than tapping.
- **Battery draw on a long practice session** → Accepted and intended; a lit screen
  is the feature. Nothing else is held: no CPU wakelock, no background execution.
- **A leaked hold if `dispose` is skipped** → Bounded twice over: the count is
  re-evaluated on every foreground transition, and the next background release
  clears the platform state regardless of the count.
- **Xvfb / headless CI has no screensaver to inhibit** → D7's try/catch; the
  integration suite must stay green with the plugin failing every call.
- **A new plugin touches five native build graphs** → Verify all five build before
  merge, not only the two that reported the bug. Android and iOS/macOS pods are the
  ones that can surprise; note the repo's `LANG=en_US.UTF-8` requirement for
  `pod install` on this machine, and that `flutter test -d macos` rewrites the
  native shells.
- **macOS App Store sandbox** → `IOPMAssertionCreateWithName` needs no entitlement,
  so the sandboxed archive is unaffected. Worth an explicit check on the TestFlight
  build, given how thoroughly re-signing has bitten this repo before.

## Migration Plan

No data, no server, no persisted state — the change is inert until a play surface
mounts. Rollback is reverting the wrappers; the seam and the plugin can stay behind
without effect.

## Open Questions

- **Does the pre-play setup modal need its own hold?** It is a modal over
  `PlayerScreen`, which is already holding, so no — but if it is ever reachable from
  elsewhere, it becomes a fifth play surface.
- **Score audio preview in the library** — a preview plays with the phone possibly
  untouched, but the user is listening, not reading. Left out; revisit only if it is
  reported.
