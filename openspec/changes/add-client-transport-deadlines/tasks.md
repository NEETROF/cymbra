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

## 2. Wire the interceptor into every gRPC client

- [ ] 2.1 `grpc_client.dart`: add `connectTimeout: kConnectTimeout` (10 s) to
      `cymbraChannel`'s `ChannelOptions`. Leave `connectionTimeout` and `idleTimeout`
      alone — `connectionTimeout` is connection *reuse* (50 min default), not connect.
- [ ] 2.2 Give each adapter an injected `RpcDeadlines` and pass it as
      `interceptors: [deadlines]` on its generated client constructor. Adapters:
      `GrpcAuthService`, `GrpcAccountService` (both in `grpc_client.dart`),
      `GrpcCatalogService`, `GrpcScoreUploadService`, `GrpcCourseCatalogService`,
      `GrpcCourseProgressService`, `GrpcRatingService`, `GrpcLeaderboardService`,
      `GrpcGlobalLeaderboardService`, `GrpcPlaySyncService`, `GrpcStreakService`,
      `GrpcProfileService`, `GrpcNotificationRegistryService`,
      `GrpcAchievementsService`, `GrpcCuratorRewardsService`,
      `GrpcSoundFontCatalogService`, `GrpcUsageTrackingService`.
- [ ] 2.3 Wire the interceptor into the `AuthServiceClient` that `tokenRefresher`
      builds directly (`grpc_client.dart:398`) — it bypasses every adapter, and an
      unbounded refresh there means a hung sign-in.
- [ ] 2.4 Update the adapter providers to read `rpcDeadlinesProvider`, and run
      `dart run build_runner build --delete-conflicting-outputs`.
- [ ] 2.5 Grep `lib/` for any remaining generated-client construction not covered by
      2.2/2.3 and wire it too.

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

## 4. HTTP seam deadlines

- [ ] 4.1 `private_soundfont_service.dart`: `interactive` timeout on `list`,
      `delete`, `propose`. Leave `import` and `download` **uncapped** with a comment
      explaining why a wall-clock bound on a 400 MiB transfer is a bug (design D5).
- [ ] 4.2 `soundfont_source.dart`: font-bytes download is `bulk` — uncapped, same
      comment.
- [ ] 4.3 `soundfont_preview_service.dart` and `score_preview_service.dart`: bounded
      media fetches — `transfer` timeout.
- [ ] 4.4 Make sure a timeout surfaces as the seam's own exception type
      (`PrivateSoundFontException` etc.), not a raw `TimeoutException`, so no raw
      technical text can reach the UI.

## 5. Tests

- [ ] 5.1 `test/services/rpc_deadlines_test.dart` — table test over
      `deadlineForMethod`: every override path returns its category budget, and an
      unknown path returns `interactive`.
- [ ] 5.2 Assert the interceptor actually attaches the deadline: call
      `interceptUnary` with a fake invoker that captures the `CallOptions` it
      receives, and assert `timeout` equals the category budget.
- [ ] 5.3 Assert the merge direction (design D3): an explicit per-call
      `CallOptions(timeout: …)` reaches the invoker unchanged and is **not**
      overwritten by the policy. This test is the guard on the one invertible line.
- [ ] 5.4 Per-adapter wiring test: for each of the 17 clients from 2.2/2.3, assert the
      constructed client carries the interceptor. Drive it through a fake
      `ClientChannel`/`ChannelBase` that records the `CallOptions` reaching
      `createCall`, so the assertion is "the deadline is on the call", not "the
      constructor was passed a list".
- [ ] 5.5 Regression test for the offline fallback: a fetch failing with
      `GrpcError.deadlineExceeded` drives `notation_notifier` to
      `ScoreLoadFailure.offlineUnavailable` when offline with no cached copy, and to
      the cached bytes when a copy exists — the same outcomes as for `UNAVAILABLE`.
- [ ] 5.6 Test that a timed-out refresh is classified transient and leaves the stored
      session intact (extend `test/services/token_refresher_test.dart`).
- [ ] 5.7 HTTP seam tests: a control call times out and surfaces the seam's own
      exception; a bulk call is issued with no wall-clock timeout.
- [ ] 5.8 `flutter test --coverage --exclude-tags golden` and confirm the gate stays
      ≥ 80%.

## 6. Verification and gates

- [ ] 6.1 Reproduce the original bug on device with the backend unreachable: tapping a
      catalog score now surfaces the error banner within the interactive budget
      instead of 30+ s. Record the observed delay.
- [ ] 6.2 Verify a real score upload still completes (it must not hit the `long`
      budget), and a private SoundFont import of a large file still completes.
- [ ] 6.3 Verify the offline path end to end on device: airplane mode, open a cached
      favorite (plays) and an uncached one (dedicated offline message, not the
      generic one).
- [ ] 6.4 Sanity-check the budgets on a throttled connection (design Open Questions)
      and adjust the constants if 10 s proves tight on a real slow link.
- [ ] 6.5 `melos run analyze`, `dart format`, and `dart run custom_lint` clean.
- [ ] 6.6 `openspec validate add-client-transport-deadlines --strict` passes.
