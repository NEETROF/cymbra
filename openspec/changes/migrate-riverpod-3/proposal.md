## Why

`apps/music` is pinned to the Riverpod 2.x line — a deliberate call recorded at
adoption: *"Not migrating to Riverpod 3 (pinned to the 2.x line per request)"*
([2026-06-23-adopt-riverpod-state/design.md:16](../archive/2026-06-23-adopt-riverpod-state/design.md)),
since codified in `CLAUDE.md` as the house stack. The instruction is recorded; the
reasoning behind it is not. Riverpod 3.0.0 went stable on 2025-09-10, so this was never
a "not released yet" decision — it is simply due for a fresh look.

Two concrete pulls, both surfaced while designing `add-client-transport-deadlines`:

- **`Ref.mounted` does not exist in 2.6.1.** Guarding a post-`await` state write means
  a hand-rolled flag set from `ref.onDispose`. 3.x provides `mounted` as the one method
  still legal after disposal.
- **Post-dispose writes are silently dropped in 2.6.1, and throw in 3.x.** The current
  behaviour is an implementation detail, not a contract — code that leans on it changes
  meaning the day the package is upgraded. Making that explicit is worth doing on
  purpose rather than discovering it during an upgrade.

Riverpod 3 also ships native offline persistence, which overlaps the hand-rolled
encrypted score cache. That overlap is worth *knowing about*; it is not adopted here.

## What Changes

- **Bump to `flutter_riverpod` / `riverpod_annotation` 3.x** and
  `riverpod_generator` / `riverpod_lint` to their matching majors, then regenerate.
- **Replace `AsyncValue.valueOrNull`** (removed in 3.x) with `.value` — 52 sites, 47 of
  them in `lib/`. Not a blind rename: `.value` now returns `null` **during errors**, so
  each `valueOrNull ?? fallback` has to be read for whether the error case was meant to
  fall back or not.
- **Decide the retry policy explicitly.** 3.x retries a failed provider automatically
  and flags it as loading while retrying. 45 async providers currently let their errors
  escape into `AsyncValue`, so this changes what the UI shows on failure. The policy is
  set once, on the container, rather than discovered per screen.
- **Handle `ProviderException` wrapping**: in 3.x an error rethrown through
  `ref.watch`/`ref.read` is wrapped. Five sites type-test the cause
  (`is AuthException` / `is GrpcError`) and must unwrap — small, but they include the
  offline-cache classification, so getting it wrong is silent.
- **Re-check the four `Stream` providers**: 3.x pauses a `StreamSubscription` when the
  provider is not listened to. `isOnlineNow` is one of them, and connectivity events are
  load-bearing for the offline paths.
- **Replace the hand-rolled disposal flag with `ref.mounted`** wherever
  `add-client-transport-deadlines` introduced one.
- **Update `CLAUDE.md` and the `flutter-riverpod-architecture` / `flutter-testing`
  skills** so the mandated stack reads Riverpod 3.

Explicitly **not** in scope: adopting 3.x offline persistence, mutations, or the new
transformer APIs. This change is a version migration at behavioural parity, not a
rewrite that happens to bump a version.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `state-management`: the requirement text names **Riverpod 2**; it becomes Riverpod 3.
  A new requirement makes the **failure policy explicit** — with automatic retry now a
  framework default, "what a provider does when it fails" stops being implicit and has
  to be stated. (This capability is legacy-named; per `openspec/config.yaml` it could be
  renamed `platform-state-management` since this change touches it. Deliberately left
  alone: a folder rename that 60+ archived changes reference is its own risk, and
  bundling it here would obscure the migration diff.)

## Impact

**Products**: Cymbra Music (`apps/music`) only. Cymbra ID, Live and the back-office are
untouched — the back office is Vue/Pinia and shares nothing here.

**Measured surface** (this repo, today):

| | count | affected by 3.x? |
|---|---|---|
| `@riverpod` / `@Riverpod` providers | 136 (68 keepAlive, 68 autoDispose) across 95 files | regenerate only |
| `overrideWithValue` in tests | 500 (478 `test/`, 22 `integration_test/`) | **no** — that removal was 2.0.0, not 3.0 |
| `AsyncValue.valueOrNull` | 52 (47 in `lib/`) | **yes** — removed, semantics shift |
| async providers whose errors escape | 45 | **yes** — auto-retry |
| `is AuthException` / `is GrpcError` catch sites | 5 | **yes** — `ProviderException` wrapping |
| `Stream` providers | 4 | **yes** — pause-when-unlistened |
| `ref.listen` sites | 60 | review |
| `StateProvider` / `StateNotifierProvider` / `ChangeNotifierProvider` | **0** | n/a |
| `FamilyNotifier` | **0** | n/a |
| `ProviderObserver` | **0** | n/a |
| `AsyncValue.copyWithPrevious` | **0** | n/a |

The last four rows are the point: the categories that usually make a Riverpod 3
migration painful do not exist in this codebase, and the 500 test overrides — the part
that looked most alarming — are not touched at all.

**Dependencies**: `riverpod 3.0.3` requires Dart `^3.7.0`; the app is on `^3.10.9`, so
the SDK is not a constraint. `flutter_riverpod` and `riverpod_generator` 3.0.3 both
exist; the matching `riverpod_lint` major must be confirmed on pub before starting, as
`custom_lint` is a CI gate.

**Sequencing**: this change lands **after** `add-client-transport-deadlines`. That change
adds a hand-rolled disposal guard and depends on the current error-classification
behaviour; migrating underneath it would move the ground it stands on. Ordering the
other way means doing the transport work twice.

**Coverage**: Flutter gate is 80%. A parity migration should move coverage very little;
a large swing in either direction is a signal that behaviour changed, not that tests did.
