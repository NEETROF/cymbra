## 1. Deadline policy (pure, no channel)

- [ ] 1.1 Add `apps/music/lib/services/rpc_deadlines.dart` with the budget constants
      (`kInteractiveDeadline` 10 s, `kTransferDeadline` 30 s, `kLongDeadline` 120 s)
      and a `deadlineForMethod(String path)` function: an explicit override table
      keyed by generated method path, defaulting to `kInteractiveDeadline`.
- [ ] 1.2 Populate the override table — `transfer`: `GetCatalogScoreBytes`,
      `GetScoreBytes`, `GetRatingPreviewBytes`, `GetCourse` (carries the course
      manifest blob); `long`: `UploadScore`. Take the exact paths from the `_$…`
      `ClientMethod` constants in `lib/src/grpc/*.pbgrpc.dart`, not from guesses.
- [ ] 1.3 Comment the table with the known trap from design D2: an RPC added later
      inherits `interactive`, so a new long-running RPC must be added here or it will
      time out on first use.
- [ ] 1.4 Add `RpcDeadlines implements ClientInterceptor` in the same file,
      overriding `interceptUnary` to call
      `invoker(method, request, CallOptions(timeout: deadlineForMethod(method.path)).mergedWith(options))`
      — policy as **base**, caller's options as override (design D3).
- [ ] 1.5 Override `interceptStreaming` the same way, so the policy holds if a
      streaming RPC is ever added (the app has none today).
- [ ] 1.6 Expose a `keepAlive` `rpcDeadlinesProvider` returning the shared instance
      (it is stateless, so one instance serves every adapter).

## 2. Wire the interceptor and the channel options

- [ ] 2.1 `grpc_client.dart`: add `connectTimeout: kConnectTimeout` (10 s) to
      `cymbraChannel`'s `ChannelOptions`. Leave `connectionTimeout` and `idleTimeout`
      alone — `connectionTimeout` is connection *reuse* (50 min default), not connect.
- [ ] 2.2 Same `ChannelOptions`: add
      `keepAlive: ClientKeepAliveOptions(pingInterval: …, timeout: …, permitWithoutCalls: false)`
      (design D9). Do **not** pick the interval blind — task 9.6 validates it against
      Caddy first; land the wiring with the validated values.
- [ ] 2.3 Give each adapter an injected `RpcDeadlines` and pass it as
      `interceptors: [deadlines]` on its generated client constructor. Adapters:
      `GrpcAuthService`, `GrpcAccountService` (both in `grpc_client.dart`),
      `GrpcCatalogService`, `GrpcScoreUploadService`, `GrpcCourseCatalogService`,
      `GrpcCourseProgressService`, `GrpcRatingService`, `GrpcLeaderboardService`,
      `GrpcGlobalLeaderboardService`, `GrpcPlaySyncService`, `GrpcStreakService`,
      `GrpcProfileService`, `GrpcNotificationRegistryService`,
      `GrpcAchievementsService`, `GrpcCuratorRewardsService`,
      `GrpcSoundFontCatalogService`, `GrpcUsageTrackingService`.
- [ ] 2.4 Wire the interceptor into the `AuthServiceClient` that `tokenRefresher`
      builds directly (`grpc_client.dart:398`) — it bypasses every adapter, and an
      unbounded refresh there means a hung sign-in.
- [ ] 2.5 Update the adapter providers to read `rpcDeadlinesProvider`, and run
      `dart run build_runner build --delete-conflicting-outputs`.
- [ ] 2.6 Grep `lib/` for any remaining generated-client construction not covered by
      2.3/2.4 and wire it too.

## 3. Status-code classification (must land in the same commit as section 2)

- [ ] 3.1 `auth_service.dart`: add `case 4: return AuthError.unavailable;` to
      `authErrorFromCode`, with a comment stating that a deadline overrun is
      deliberately indistinguishable from an unreachable backend (design D4), and
      that `notation_notifier._classifyLoad` gates the offline fallback on it.
- [ ] 3.2 Verify `notation_notifier._classify` / `_classifyLoad` need no edit — a
      timeout now arrives as `AuthError.unavailable` and takes the existing path.
- [ ] 3.3 Check the other `AuthError.unavailable` consumers still read correctly for
      a timeout: `score_upload_notifier.dart:44`, `auth_messages.dart:49`,
      `connected_accounts_notifier.dart:114`.
- [ ] 3.4 Confirm `CoordinatedTokenRefresher` already treats code 4 as transient (it
      documents "unavailable, deadline exceeded, unknown"); fix if it does not.

## 4. Abort on connectivity loss (design D7, D8)

- [ ] 4.1 Add a **shared** race helper in its own file (e.g.
      `apps/music/lib/state/offline_race.dart`) — not private to the notation notifier:
      run an awaited future against the first `false` **event** on
      `connectivityService.onlineStatus`, throwing a sentinel when the transition wins.
      What to do on abort stays at each call site.
- [ ] 4.2 The helper MUST own an explicit `StreamSubscription` and cancel it in a
      `finally` — `onlineStatus` is a broadcast stream, so an unsatisfied listener
      leaks one subscription per score open.
- [ ] 4.3 The helper MUST listen for the transition only, never re-read the current
      value, so it cannot double-fire with the pre-flight check of 4.5.
- [ ] 4.4 Apply the race to the two network waits in `_load`: `_fetchScoreBytes` on
      the cache-miss path (line ~112) and `_decideCachedCatalogOpen` (line ~91).
      Offline-wins resolves to the cached bytes when a copy exists, else
      `ScoreLoadFailure.offlineUnavailable`.
- [ ] 4.5 Add the pre-flight `isOnline()` gate to the cache-**miss** path, mirroring
      the one `_decideCachedCatalogOpen` already has at line 164. Short-circuit on a
      `false` reading only; a `true` reading proves nothing and the call proceeds.
- [ ] 4.6 Do **not** roll the race out to the other screens (`community`, `profile`,
      leaderboards, `score_hub`, rating deck). They have no offline behaviour to fall
      back to — only `notation_notifier` and `library_screen` touch connectivity today
      — so aborting early would just change *when* the same generic error shows. They
      adopt the helper when they gain a real offline outcome (design D7).

## 5. Escapable blocking wait (design D10)

- [ ] 5.1 `open_score.dart`: add a cancel affordance to the progress dialog. Keep
      `barrierDismissible: false` (an accidental barrier tap should not cancel a load)
      — the exit is the explicit control.
- [ ] 5.2 On cancel: clear `selectedScoreProvider` (the existing
      `if (ref.read(selectedScoreProvider) != entry) return` guards then discard the
      late result — no notifier-side "cancelled" flag) and resolve the wait with a
      **cancelled outcome distinct from failure**, e.g. a local tri-state in
      `openScore`. Completing the existing `Completer<bool>` with `false` is wrong:
      the `else` branch reads `notationProvider.failure` and shows a snackbar
      (`open_score.dart:127`), which must not run on cancel.
- [ ] 5.3 The cancel control must **not** pop the dialog itself — the main flow
      already pops once the completer resolves (`open_score.dart:110`), and a second
      pop would dismiss the underlying screen. One pop, owned by the main flow.
- [ ] 5.4 Cancelling shows **no** error banner and returns the user to the previous
      screen — it is a user decision, not a failure.
- [ ] 5.5 Add the localized cancel label to the ARB files and regenerate l10n.
- [ ] 5.6 Make sure `sub.close()` still runs on the cancel path (today it only runs
      via the loaded/failed branches).

## 6. Abandon the upload on leave (design D11)

- [ ] 6.1 `score_upload_notifier.dart`: add a disposal flag set from `ref.onDispose`
      in `build()`, and check it before every post-`await` `state = …` write in
      `submit()`. `Ref.mounted` does NOT exist in riverpod 2.6.1 — do not reach for it.
- [ ] 6.2 Do **not** add a `PopScope` to `ScoreUploadScreen`. Leaving stays free; the
      exit must never wait on a network answer (design D11).
- [ ] 6.3 Show **no** message on leave — not "cancelled", not "failed", not "sent".
      The app cannot know which. `MyUploads` already refreshes on the `result`
      transition and reports the truth.
- [ ] 6.4 Reword the `AlreadyExists` case in `uploadErrorMessage` as a statement of
      fact ("this score is already in your library"), not as an error — a retry after
      an abandoned upload is the expected path, not an exception. Update the ARB
      strings.
- [ ] 6.5 Confirm the dedup claim before relying on it: `(owner_id, sha256)` UNIQUE
      (`0003_user_scores.sql:42`) over the **canonical** decoded MusicXML
      (`module.rs` step 4), so a re-zip of the same piece still collides.

## 7. HTTP seam deadlines

- [ ] 7.1 `private_soundfont_service.dart`: `interactive` timeout on `list`,
      `delete`, `propose`. Leave `import` and `download` **uncapped** with a comment
      explaining why a wall-clock bound on a 400 MiB transfer is a bug (design D5).
- [ ] 7.2 `soundfont_source.dart`: font-bytes download is `bulk` — uncapped, same
      comment.
- [ ] 7.3 `soundfont_preview_service.dart` and `score_preview_service.dart`: bounded
      media fetches — `transfer` timeout.
- [ ] 7.4 Make sure a timeout surfaces as the seam's own exception type
      (`PrivateSoundFontException` etc.), not a raw `TimeoutException`, so no raw
      technical text can reach the UI.
- [ ] 7.5 `imported_soundfonts.dart` `importSoundFont()`: persist the local registry
      entry **before** awaiting the server upload (design D5). The upload is already
      designed non-fatal ("syncs on a later build"), but a hung uncapped upload never
      throws, so today it wedges the import it was not supposed to be able to fail.

## 8. Tests

- [ ] 8.1 `test/services/rpc_deadlines_test.dart` — table test over
      `deadlineForMethod`: every override path returns its category budget, and an
      unknown path returns `interactive`.
- [ ] 8.2 Assert the interceptor actually attaches the deadline: call
      `interceptUnary` with a fake invoker that captures the `CallOptions` it
      receives, and assert `timeout` equals the category budget.
- [ ] 8.3 Assert the merge direction (design D3): an explicit per-call
      `CallOptions(timeout: …)` reaches the invoker unchanged and is **not**
      overwritten by the policy. This test is the guard on the one invertible line.
- [ ] 8.4 Per-adapter wiring test: for each of the 17 clients from 2.3/2.4, assert the
      constructed client carries the interceptor. Drive it through a fake
      `ClientChannel`/`ChannelBase` that records the `CallOptions` reaching
      `createCall`, so the assertion is "the deadline is on the call", not "the
      constructor was passed a list".
- [ ] 8.5 Regression test for the offline fallback: a fetch failing with
      `GrpcError.deadlineExceeded` drives `notation_notifier` to
      `ScoreLoadFailure.offlineUnavailable` when offline with no cached copy, and to
      the cached bytes when a copy exists — the same outcomes as for `UNAVAILABLE`.
- [ ] 8.6 Test that a timed-out refresh is classified transient and leaves the stored
      session intact (extend `test/services/token_refresher_test.dart`).
- [ ] 8.7 Race test: with a never-completing fetch, emit `false` on a controllable
      `onlineStatus` and assert the notifier resolves immediately to the offline
      outcome — this is the airplane-mode repro as a unit test.
- [ ] 8.8 Leak test: on a normal load with no connectivity transition, assert the
      connectivity subscription is cancelled (a fake `ConnectivityService` counting
      listen/cancel).
- [ ] 8.9 Test that a late result from an abandoned load — arriving after cancel or
      after an offline abort — does not overwrite state.
- [ ] 8.10 Pre-flight test: offline at load time produces the offline outcome with the
      catalog service never called (verify with a strict mock).
- [ ] 8.11 Widget test on `open_score`: the progress dialog exposes a cancel control,
      and cancelling pops it, clears the selection and shows no error banner.
- [ ] 8.12 HTTP seam tests: a control call times out and surfaces the seam's own
      exception; a bulk call is issued with no wall-clock timeout.
- [ ] 8.13 Upload-abandon test: dispose the notifier mid-`submit()` and assert the
      late completion writes no state and throws nothing, in both debug and release
      semantics (the flag, not the incidental drop).
- [ ] 8.14 Assert leaving mid-upload surfaces no message at all, and that a subsequent
      `AlreadyExists` is rendered as a fact rather than an error.
- [ ] 8.15 `flutter test --coverage --exclude-tags golden` and confirm the gate stays
      ≥ 80%.

## 9. Verification and gates

- [ ] 9.1 Reproduce the original bug on device with the backend unreachable: tapping a
      catalog score now surfaces the error banner within the interactive budget
      instead of 30+ s. Record the observed delay.
- [ ] 9.2 Reproduce the reported symptom: start a load on an unstable connection, then
      enable airplane mode — the loader must end at once, not after the deadline.
- [ ] 9.3 Verify the cancel control on a genuinely slow load, and confirm the app
      returns to the list with no error banner and no late state change.
- [ ] 9.4 Verify a real score upload still completes (it must not hit the `long`
      budget), and a private SoundFont import of a large file still completes.
- [ ] 9.5 On device: start an upload, leave the screen mid-flight, and confirm no
      message is shown, no duplicate appears, and the contribution shows up in
      `MyUploads` if it landed.
- [ ] 9.6 **Blocking for 2.2**: validate the keepalive ping interval against prod's
      Caddy front end — confirm pings at the chosen rate are answered and do not draw
      a `GOAWAY` (`ENHANCE_YOUR_CALM`). If they do, raise the interval or ship
      keepalive disabled; the other mechanisms stand without it.
- [ ] 9.7 Record in the code comment that keepalive pongs come from **Caddy**, not
      tonic (`reverse_proxy h2c://`, PING is hop-by-hop) — it detects a dead link to
      the edge, never a hung backend.
- [ ] 9.8 Verify the offline path end to end on device: airplane mode, open a cached
      favorite (plays) and an uncached one (dedicated offline message, not the
      generic one).
- [ ] 9.9 Sanity-check the budgets on a throttled connection (design Open Questions)
      and adjust the constants if 10 s proves tight on a real slow link.
- [ ] 9.10 `melos run analyze`, `dart format`, and `dart run custom_lint` clean.
- [ ] 9.11 `openspec validate add-client-transport-deadlines --strict` passes.
