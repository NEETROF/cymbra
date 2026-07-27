---
name: flutter-testing
description: Test-double convention for the Flutter app (apps/music). Use when writing or editing any Flutter test — unit, widget, notifier/provider — or when choosing a test double for a dependency (service, repo, port, client). DEFAULT to mockito with generated mocks (@GenerateNiceMocks + build_runner), injected via Riverpod provider overrides; hand-written fakes are reserved for special cases (documented below).
metadata:
  author: cymbra
  version: "1.0"
---

# Flutter test doubles — mockito by default (Cymbra)

Default to **mockito with generated mocks** for a dependency's test double, **not**
hand-written fakes. Dependencies are already Riverpod providers (see the
`flutter-riverpod-architecture` skill), so a mock is injected the same way a fake
was — only the double changes.

Requires the `mockito` dev-dependency (`build_runner` is already present).

## Generate mocks

Annotate a test (or a shared `test/support/mocks.dart`) and run build_runner:

```dart
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/score_upload_service.dart';

@GenerateNiceMocks([MockSpec<ScoreUploadService>()])
import 'contributed_scores_test.mocks.dart';
```

```bash
cd apps/music && dart run build_runner build --delete-conflicting-outputs
```

Prefer **`@GenerateNiceMocks`** (unstubbed calls return sane defaults) over
`@GenerateMocks` (throws `MissingStubError`) unless a test specifically wants the
strict behaviour. Generated `*.mocks.dart` are codegen — gitignored like
`*.g.dart`; run build_runner before analyze/test.

## Use in a test (injected via a provider override)

```dart
final upload = MockScoreUploadService();
when(upload.listMyScores()).thenAnswer((_) async => [ /* … */ ]);

final c = ProviderContainer(overrides: [
  scoreUploadServiceProvider.overrideWithValue(upload),
  canUseOnlineServicesProvider.overrideWithValue(true),
]);
addTearDown(c.dispose);

await c.read(myUploadsProvider.notifier).delete('a');

verify(upload.deleteScore('a')).called(1);
```

- Stub with `when(...).thenAnswer(...)` / `.thenThrow(...)`; assert interactions
  with `verify(...)` / `verifyNever(...)` / `verifyInOrder([...])`.
- Keep the deps-as-providers + `ProviderContainer`/`ProviderScope` override pattern
  (constructor injection is still banned) — the mock is the override value.

## Special cases — a hand-written fake is allowed when…

Use a fake only when a mock would be worse, and say why in a comment:
- **Behavioural in-memory doubles**: the test needs the double to *behave* across a
  sequence (e.g. an in-memory store that persists, a repo that dedups), so
  state-based assertions are clearer than piles of `when`/`verify`.
- **Trivial value stubs**: a tiny stub is obviously simpler than generating a mock.
- **Fakes the production code ships** (e.g. an offline/no-op implementation reused
  in tests).

Otherwise: mockito.

## Checklist

- [ ] Dependency doubles are mockito mocks (`@GenerateNiceMocks`), not new hand fakes.
- [ ] Mocks injected via Riverpod provider overrides (no constructor injection).
- [ ] `when(...)` for stubs, `verify(...)` for interactions.
- [ ] `build_runner` run so `*.mocks.dart` exist (gitignored).
- [ ] Any hand-written fake carries a one-line justification (a special case above).
