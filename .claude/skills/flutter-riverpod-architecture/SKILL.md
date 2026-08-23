---
name: flutter-riverpod-architecture
description: Architecture rules for the Flutter app (apps/music) using Riverpod 2 + Freezed. Use when creating or editing any widget/screen, Riverpod provider/notifier, or service, or when wiring side effects (navigation, snackbars), cross-provider reactions, or awaiting actions — and whenever adding or editing user-facing strings or ARB translation files (every locale must stay aligned with the template). Enforces four rules — UI never calls services directly (only notifiers do); a provider never imperatively invalidates another (react via listeners); never await a notifier action's return in the UI (react to state via listeners); isolate listeners in a dedicated listener widget — plus the no-translation-drift rule.
metadata:
  author: cymbra
  version: "1.0"
---

# Flutter + Riverpod architecture (Cymbra)

Rules for `apps/music` (Riverpod 2 + Freezed, codegen). They mirror the Vue rules
(see `vue-frontend-architecture`) and extend them with Riverpod-specific reactivity.
Apply proactively — the maintainer challenges architecture in review. Base
conventions (codegen providers, Freezed state, deps-as-providers) are in `CLAUDE.md`.

## Rule 1 — a widget/screen NEVER calls a service directly

Only **notifiers** (`state/`) call services (`services/`) or gRPC clients. Widgets
read notifiers and call *their* methods; they never `ref.read(...ServiceProvider)`
to invoke a side-effectful method.

```dart
// ❌ widget orchestrating a service + a manual refresh
await ref.read(scoreUploadServiceProvider).setFavorite(id, !fav);
await ref.read(catalogSearchProvider.notifier).refresh();

// ✅ the mutation lives in the notifier; the widget just asks
ref.read(contributedScoresProvider.notifier).toggleFavorite(id);
```

Check: `grep -rn "ref\.\(read\|watch\)(.*ServiceProvider)" lib/screens lib/widgets`
should be empty (pure value reads like a clock are the only tolerated exception).

## Rule 2 — a provider NEVER imperatively invalidates another provider

Do not call `ref.invalidate(otherProvider)` / `ref.refresh(...)` from inside a
provider/notifier to poke a *sibling*. That couples them imperatively. Instead the
**dependent** provider **listens** to the source and reacts:

```dart
// ❌ upload notifier reaching across to invalidate the list
ref.invalidate(catalogSearchProvider);

// ✅ the list listens to what it depends on and refreshes itself
@riverpod
class SavedCatalogScores extends _$SavedCatalogScores {
  @override
  Future<List<CatalogHit>> build() {
    ref.listen(libraryRevisionProvider, (_, __) => ref.invalidateSelf());
    return _load();
  }
}
```

A provider may invalidate **itself** (`ref.invalidateSelf`) in reaction to a
listened change — it just must not invalidate *others*.

## Rule 3 — never `await` a notifier action's return in the UI

Actions mutate state; the UI reacts to that state, it does not await the call and
branch on the result. Fire the action; observe the resulting `AsyncValue`
(loading → data/error) via a listener.

```dart
// ❌ imperative: await + branch + drive navigation from the result
final ok = await ref.read(authFlowProvider.notifier).signIn(email, pw);
if (ok) context.go('/home');

// ✅ fire-and-forget; a listener reacts to the state transition
ref.read(authFlowProvider.notifier).signIn(email, pw); // returns void/Future ignored
// …and in the listener widget (Rule 4):
ref.listen(authFlowProvider, (prev, next) {
  next.whenOrNull(
    data: (_) => context.go('/home'),
    error: (e, _) => showErrorSnack(context, e),
  );
});
```

Model action state as `AsyncValue<T>` (Freezed union under the hood) so the four
states are explicit — same spirit as the Vue `Async<T>` union.

## Rule 4 — isolate listeners in a dedicated listener widget

Side effects triggered by state changes (navigation, snackbars, dialogs, reacting
invalidations) belong in **one dedicated listener widget** near the top of the
feature subtree, not scattered through build methods. It renders `child` and only
wires `ref.listen`:

```dart
class PlayerListeners extends ConsumerWidget {
  const PlayerListeners({required this.child, super.key});
  final Widget child;
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(playerProvider.select((s) => s.status), (prev, next) {
      if (next == PlayStatus.finished) _showSummary(context, ref);
    });
    ref.listen(scoreUploadNotifierProvider, (prev, next) {
      next.whenOrNull(error: (e, _) => showErrorSnack(context, e));
    });
    return child;
  }
}
```

Benefits: side effects are discoverable in one place, build methods stay pure, and
the same listener isn't duplicated across rebuilds.

## Testing

Notifiers are the unit under test, with fakes injected via `ProviderScope`/
`ProviderContainer` overrides (never constructor injection). Assert on the resulting
state; a failed action is an `AsyncError` in the state, not a thrown exception the
widget caught. Widget tests can pump the listener widget to assert side effects.
Keep the ≥ 80% coverage gate green.

## Localization: locales never drift

Every user-facing string goes through `AppLocalizations` (ARB files in
`apps/music/lib/l10n/`; `app_en.arb` is the single template — see `l10n.yaml`).

**Hard rule — no translation drift.** A key added, renamed or removed in
`app_en.arb` is mirrored **in the same change** in every other locale
(`app_fr`, `app_es`, `app_it`, and any locale added later): real translations
(informal register, terminology consistent with neighbouring strings), the same
`{placeholders}`, inserted at the same relative position as in the template.
`flutter gen-l10n` does **not** fail on a missing key — it silently falls back
to English — so drift ships unnoticed (es/it shipped 16 English audio-settings
strings before this rule was written down).

Verify before pushing (from `apps/music`):

```bash
python3 -c "
import json,glob
t={k for k in json.load(open('lib/l10n/app_en.arb')) if not k.startswith('@')}
for f in sorted(glob.glob('lib/l10n/app_*.arb')):
    k={x for x in json.load(open(f)) if not x.startswith('@')}
    print(f, 'missing:', sorted(t-k) or 'OK', 'extra:', sorted(k-t) or 'OK')
"
```

Compare **key sets and placeholders only** — ICU `select`/`plural` option texts
legitimately differ per locale and look like mismatches to naive regex checks.

## Checklist

- [ ] No `ref.read/watch(*ServiceProvider)` (side-effectful) in `lib/screens` or `lib/widgets`.
- [ ] Mutations live in notifiers; widgets call notifier methods only.
- [ ] No provider invalidates a *sibling* provider — dependents `ref.listen` + `invalidateSelf`.
- [ ] No `await notifier.action()` + branch in the UI — react via `ref.listen` on the state.
- [ ] `ref.listen` side effects (nav/snackbar/dialog) live in a dedicated listener widget.
- [ ] New/renamed l10n keys mirrored in **all** `app_*.arb` locales (same placeholders) — no locale silently falling back to English.
