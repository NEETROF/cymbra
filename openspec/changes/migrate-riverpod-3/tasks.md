## 1. Confirm the toolchain before touching anything

- [ ] 1.1 `riverpod_lint` 3.x existence: **confirmed 2026-08-23** (3.1.8 on pub).
      Remaining: pin the major matching `riverpod_generator` 3.x and confirm the
      repo's `custom_lint` gate still passes with it — it is a CI gate.
- [ ] 1.2 Verify `riverpod_generator` 3.x still pulls a Freezed major compatible with
      the repo's Freezed 3 syntax (`abstract class … with _$…`).
- [ ] 1.3 Confirm this change is starting **after** `add-client-transport-deadlines` has
      landed (design D1). If it has not, stop — the ordering is the decision.

## 2. Bump and regenerate

- [ ] 2.1 Raise `flutter_riverpod` and `riverpod_annotation` to `^3`, and
      `riverpod_generator` / `riverpod_lint` to their matching majors, in
      `apps/music/pubspec.yaml`.
- [ ] 2.2 `dart run build_runner build --delete-conflicting-outputs`, then record the
      full compile-error list before fixing anything — it is the real scope of the
      migration, and it is cheaper to read once than to discover incrementally.
- [ ] 2.3 Fix mechanical breakage: removed `Ref` subclasses (`FutureProviderRef` and
      friends) become plain `Ref`.

## 3. `valueOrNull` → `.value`, one site at a time (design D2)

- [ ] 3.1 Replace all 52 `AsyncValue.valueOrNull` uses with `.value`. Do **not** sweep
      with `sed`: `.value` now returns `null` **during errors**, so every
      `valueOrNull ?? fallback` needs its error case read deliberately.
- [ ] 3.2 For each `?? fallback` site, record in the diff whether falling back on error
      is the intended behaviour. `library_screen.dart:265`
      (`isOnlineNowProvider.valueOrNull ?? true`) is the archetype: defaulting to
      "online" is a deliberate bias that must survive.
- [ ] 3.3 Flag any site where the error case previously yielded a stale-but-useful value
      and now yields the fallback; decide per site rather than in bulk.

## 4. Failure policy (design D3)

- [ ] 4.1 Disable automatic provider retry at the `ProviderContainer` /
      `ProviderScope`, in **one** place.
- [ ] 4.2 Confirm the 45 async providers that let errors escape now settle into an error
      state rather than looping through loading — especially those whose failure is a
      classified, terminal answer (`ScoreLoadFailure.offlineUnavailable`,
      `unavailable`, `rateLimited`).
- [ ] 4.3 Opt individual providers back into retry only where a retry is genuinely
      right, and say why at the call site. Default is none.

## 5. Error wrapping at the classification sites (design D4)

- [ ] 5.1 Unwrap `ProviderException` at the five sites that type-test a cause
      (`is AuthException`, `is GrpcError`, `is PrivateSoundFontException`).
- [ ] 5.2 Do **not** add a global unwrapping helper — it would erase, at every catch,
      whether the error arrived through a provider read.
- [ ] 5.3 `notation_notifier._classify` / `_classifyLoad` are the load-bearing pair: a
      missed unwrap silently downgrades a classified failure to "generic" and disables
      the offline-cache path.

## 6. Stream providers (design D5)

- [ ] 6.1 Re-verify the four `Stream` providers under 3.x's pause-when-unlistened
      behaviour, starting with `isOnlineNow`.
- [ ] 6.2 Confirm the connectivity-dependent behaviours still receive transitions: the
      score-open offline race, `_classifyLoad`, and the outbox drains in
      `play_sync_notifier` and `usage_tracking_notifier`.
- [ ] 6.3 If a stream needs to stay hot, make that explicit at the provider rather than
      relying on something happening to watch it.

## 7. Adopt what 3.x gives back

- [ ] 7.1 Replace the hand-rolled disposal flag introduced by
      `add-client-transport-deadlines` (its task 6.1) with `ref.mounted`.
- [ ] 7.2 Grep for any other post-`await` `state =` write with no mounted guard — under
      3.x these throw instead of being silently dropped.
- [ ] 7.3 Do **not** adopt offline persistence, mutations, or the transformer APIs
      (design Non-Goals).

## 8. Tests

- [ ] 8.1 Confirm the 500 `overrideWithValue` sites still compile and pass untouched —
      if this needs edits, the migration assumption was wrong; stop and re-scope.
- [ ] 8.2 For each of the five classification sites, add a test that drives the error
      **through a provider read**, not only a direct throw — that is the path that
      wraps.
- [ ] 8.3 Test that an offline transition is delivered while no widget watches the
      connectivity provider.
- [ ] 8.4 Test that a failed provider settles into an error state and does not loop
      through loading.
- [ ] 8.5 Test that a late completion after disposal writes no state and throws nothing.
- [ ] 8.6 Run the golden and widget suites and check for rebuild-count changes from
      3.x's `==`-based filtering (design Open Questions).
- [ ] 8.7 `flutter test --coverage --exclude-tags golden`; coverage should barely move.
      A large swing either way means behaviour changed — investigate, do not re-baseline.

## 9. Documentation and gates

- [ ] 9.1 Update `CLAUDE.md`: the mandated stack becomes Riverpod 3 + Freezed.
- [ ] 9.2 Update the `flutter-riverpod-architecture` and `flutter-testing` skills.
- [ ] 9.3 Land the doc changes **last**, after the gates are green — they are the
      one-way door (design Migration Plan).
- [ ] 9.4 `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [ ] 9.5 Full integration run (`melos run integration`) — provider lifetimes are
      exactly what a real app run exercises and unit tests do not.
- [ ] 9.6 `openspec validate migrate-riverpod-3 --strict` passes.
