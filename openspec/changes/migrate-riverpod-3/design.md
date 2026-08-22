## Context

`apps/music` resolves `flutter_riverpod` / `riverpod` / `riverpod_annotation` 2.6.1 and
`riverpod_generator` / `riverpod_lint` 2.6.5, constrained `^2.6.1` / `^2.6.5` so the 3.x
line is unreachable without an explicit bump. The pin is a recorded decision — *"pinned
to the 2.x line per request"*
([2026-06-23-adopt-riverpod-state/design.md:16](../archive/2026-06-23-adopt-riverpod-state/design.md))
— whose rationale was not written down. Riverpod 3.0.0 has been stable since
2025-09-10.

The surface, measured rather than estimated:

- **136 providers** (68 `@Riverpod(keepAlive: true)`, 68 autoDispose) over 95 files.
- **500 `overrideWithValue`** call sites in tests (478 `test/`, 22 `integration_test/`)
  — the DI mechanism `CLAUDE.md` mandates. Verified present in `flutter_riverpod`
  3.0.3 (its own docs use it for provider overrides), so task 8.1 is a confirmation
  pass, not a gamble.
- **52 `AsyncValue.valueOrNull`** (47 in `lib/`).
- **45** async providers whose errors escape into `AsyncValue`.
- **5** sites type-testing an error cause (`is AuthException`, `is GrpcError`).
- **4** `Stream` providers, one of which is `isOnlineNow`.
- **60** `ref.listen` sites.
- **Zero** `StateProvider`, `StateNotifierProvider`, `ChangeNotifierProvider`,
  `FamilyNotifier`, `ProviderObserver`, or `AsyncValue.copyWithPrevious`.

That last line is the headline. The breaking changes that dominate most Riverpod 3
migration guides — legacy providers moved behind a `legacy` import, `FamilyNotifier`
removed, `ProviderObserver` signatures changed — have **no instances here**. And the
`overrideWithValue` removal that would have been catastrophic at 500 sites belongs to
**2.0.0**, not 3.0.0; it does not apply.

Dart SDK is not a constraint: `riverpod 3.0.3` needs `^3.7.0`, the app is on `^3.10.9`.

## Goals / Non-Goals

**Goals:**

- Land on the 3.x line at **behavioural parity** — the app does what it does today.
- Turn two behaviours that are currently accidental into stated ones: what happens to a
  write after disposal, and what happens when a provider fails.
- Keep the 500 test overrides working untouched.
- Leave the codebase using `ref.mounted` instead of hand-rolled disposal flags.

**Non-Goals:**

- Adopting 3.x **offline persistence**. It overlaps the hand-rolled encrypted score
  cache, which has its own key-derivation and entitlement design; replacing it is a
  change of its own, and mixing it into a version bump would make both unreviewable.
- Adopting **mutations** or the new provider-transformer APIs.
- Restructuring providers, splitting notifiers, or "while we're in here" cleanups. A
  migration diff should be boring enough that a reviewer can check it mechanically.
- Renaming the `state-management` capability to `platform-state-management`. Allowed by
  the convention since this change touches it, but a folder rename referenced by 60+
  archived changes is separate risk with no bearing on the migration.

## Decisions

### D1 — Sequence after `add-client-transport-deadlines`, not before

That change introduces a hand-rolled disposal flag (its task 6.1) and leans on the
current error-classification path (`catch (e)` → `e is AuthException` →
`ScoreLoadFailure`). Both are things 3.x changes. Migrating first would mean designing
the transport work against a framework the repo is not on yet; migrating after means one
clean follow-up that deletes the flag in favour of `ref.mounted` and adjusts five catch
sites. The alternative — doing transport work twice — is strictly worse.

### D2 — `valueOrNull` → `.value` is a review, not a rename

3.x removes `AsyncValue.valueOrNull` and changes `.value` to return `null` **during
errors**. A blind `sed` compiles and silently changes behaviour at every site shaped
`ref.watch(p).valueOrNull ?? fallback`: today an error state that carries a previous
value can yield that value; after the migration it yields the fallback.

52 sites is small enough to read each one and answer "was the error case meant to fall
back here?". `library_screen.dart:265` — `ref.watch(isOnlineNowProvider).valueOrNull ??
true` — is the archetype: defaulting to "online" on error is a deliberate bias that must
survive, and it happens to be one where the new semantics give the same answer. Others
may not be.

### D3 — Retry policy: off by default, opt in where it helps

3.x retries a failed provider automatically and marks it loading while retrying. For 45
async providers here, that turns a settled error into a spinner that resolves later — the
same failure shape `add-client-transport-deadlines` exists to eliminate. Worse, it
interacts badly with classified failures: `ScoreLoadFailure.offlineUnavailable` is a
*terminal* answer for the user, not a transient condition worth retrying behind a
spinner.

Chosen: disable automatic retry at the container, then opt in per provider where a retry
is genuinely right (a pure cache warm, say). Set in one place, per the added
requirement, rather than discovered screen by screen.

Alternative rejected: keep the default and let each screen cope. It would silently
re-introduce indefinite loading states across 45 providers, which is precisely the class
of bug the transport change was written to kill.

### D4 — Unwrap `ProviderException` at the classification sites, not globally

3.x wraps an error rethrown through `ref.watch`/`ref.read`. Five sites type-test the
cause, including `notation_notifier._classify` — the one that decides between the
offline-cache path and a generic error.

Chosen: unwrap at those five sites. A global unwrapping helper applied everywhere would
be tidier-looking but would hide, at every catch in the app, whether the error came
through a provider read or directly — information the wrapper exists to convey.

The failure mode here is silent: a missed unwrap does not crash, it just makes a
classified failure fall through to "generic". So each of the five gets a test that
asserts classification through a provider read, not only a direct throw.

### D5 — Re-verify the four stream providers, starting with `isOnlineNow`

3.x pauses a `StreamSubscription` when its provider is not being listened to. That is a
sensible default and a hazard for exactly one use here: connectivity. The offline
behaviours — the score-open race, the cache fallback, the outbox drains in
`play_sync_notifier` and `usage_tracking_notifier` — depend on transitions arriving.

A paused connectivity stream would not fail loudly; it would just stop aborting loads,
and the symptom would be the original bug reappearing. So this gets a behavioural test
(transition delivered while no widget watches), not a code inspection.

## Risks / Trade-offs

- **A silent `valueOrNull` semantic shift** → 52 sites reviewed individually (D2), not
  swept; the ones with an `?? fallback` get explicit attention for the error case.
- **A missed `ProviderException` unwrap degrades classification silently** → the five
  sites get tests that go through a provider read (D4). This is the highest-risk item
  because nothing breaks visibly.
- **Auto-retry reintroduces indefinite spinners** → retry disabled at the container by
  default (D3), which is also what keeps `add-client-transport-deadlines` meaningful.
- **A paused connectivity stream silently disables the offline paths** → behavioural
  test rather than inspection (D5).
- **`riverpod_lint` 3.x may lag or change rules** → `custom_lint` is a CI gate, so the
  matching major is confirmed on pub **before** starting, not discovered at the end.
- **Coverage swings** → a parity migration should barely move the number. A large move
  in either direction is evidence that behaviour changed; investigate rather than
  re-baseline.

## Migration Plan

One branch, in dependency order: confirm the toolchain majors exist → bump and
regenerate → fix compile errors (`valueOrNull`, any removed `Ref` subclasses) → set the
retry policy → unwrap at the five classification sites → re-verify the stream providers
→ swap the hand-rolled disposal flag for `ref.mounted` → update `CLAUDE.md` and the two
skills.

Rollback is the dependency bump reverted; nothing here touches stored data, the wire, or
the backend. The one-way door is developer-facing only: once `CLAUDE.md` says Riverpod 3,
new code written against it does not trivially revert — so the doc change lands last,
after the gates are green.

## Open Questions

- ~~Does a `riverpod_lint` 3.x exist on pub?~~ **Resolved 2026-08-23**: latest on pub
  is `riverpod_lint` 3.1.8, well past 3.0. What remains of task 1.1 is only confirming
  the repo's `custom_lint` gate passes with it — a run, not an existence question.
- Should any provider opt back **into** retry once the default is off (D3)? Answer per
  provider during the migration, not up front — the honest default is "none until one
  proves it wants it".
- Does 3.x's `==`-based rebuild filtering change any observed rebuild counts? Freezed
  models give value equality, but states carrying a byte buffer or a plain list fall back
  to identity. Expected to be a no-op; worth one pass over the golden/widget tests.
