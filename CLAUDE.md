# Cymbra — working agreements

Conventions for working in this monorepo (Rust engine + Flutter app under
`apps/music`, managed by Cargo + Melos). Canonical source of truth for AI-dev
practices; CI enforces the parts that can be automated.

## Spec-driven development (OpenSpec)

Non-trivial changes go through OpenSpec before coding:

1. `/opsx:propose "<idea>"` — create the change + artifacts (proposal, design, specs, tasks).
2. Implement against the tasks; `/opsx:apply` to track progress.
3. `openspec validate <change> --strict` must pass.
4. After review/merge, `/opsx:archive <change>` folds the spec delta into `openspec/specs/`.

Specs live in `openspec/specs/`; in-flight changes in `openspec/changes/`. The
first capability is `midi` (see `openspec/changes/ratify-midi-poc/`).

## State management — Riverpod 2 + Freezed (codegen)

Mandatory for all app state:
- **Riverpod 2** providers/notifiers via code generation (`@riverpod` +
  `riverpod_generator`). No new `ChangeNotifier`/`setState` for app state.
- **Freezed** immutable models for state (`@freezed`, mutate via `copyWith`).
- **Dependencies are providers** (e.g. `midiServiceProvider`, `scoreSourceProvider`),
  overridden in tests with fakes via `ProviderScope`/`ProviderContainer` overrides —
  not constructor injection.
- Reference implementation: [player_notifier.dart](apps/music/lib/state/player_notifier.dart),
  [player_data.dart](apps/music/lib/state/player_data.dart),
  [midi_service.dart](apps/music/lib/services/midi_service.dart).
- `riverpod_lint`/`custom_lint` is enforced (`dart run custom_lint`).

**Codegen**: generated `*.g.dart`/`*.freezed.dart` are gitignored and produced by
`build_runner` — run it before analyze/test (CI does this automatically):
```bash
cd apps/music && dart run build_runner build --delete-conflicting-outputs
# or: melos run generate
```
Notifier rule: never read or assign `state` inside `build()` before it returns —
compute the initial value and return it.

## Test coverage — minimum 80%

Every change keeps or raises **line coverage ≥ 80%** for both ecosystems; new
code needs tests. CI fails under 80% and also reports to SonarCloud (decoration).

- **Rust**: `cargo llvm-cov --workspace --fail-under-lines 80` (excludes the
  generated bridge, `lib.rs`, the hardware/thread glue in `api/midi.rs`, the
  thin MusicXML FFI seam in `api/musicxml.rs`, and the cpal/rustysynth audio glue
  in `api/audio.rs`).
  Keep pure, testable logic in host-testable modules like `api/midi_core.rs`,
  `api/musicxml_core.rs` and `api/audio_core.rs`.
- **Flutter**: `flutter test --coverage` (unit + widget) merged with the
  integration run, gated by `very_good_coverage` (excludes `lib/src/rust/**`,
  `main.dart`, generated `*.g.dart`/`*.freezed.dart`). Keep the native FFI behind
  an injectable seam (see `lib/services/midi_service.dart`) so widgets/state are
  testable without the native library.

Run locally before pushing:
```bash
cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex 'frb_generated|/lib\.rs|/midi\.rs|/musicxml\.rs|/audio\.rs'
cd apps/music && flutter test --coverage --exclude-tags golden   # then check lcov
```

## Tests: layers

- **Unit/widget** (`apps/music/test/`): fast, fakes only, no native lib. Default gate.
- **Golden** (tagged `golden`): pixel comparisons, platform-sensitive — excluded
  from the cross-platform gate. Refresh on a pinned platform with
  `flutter test --tags golden --update-goldens`.
- **Integration** (`apps/music/integration_test/`): drive the real app + FFI.
  Local: `flutter test integration_test -d macos`. CI: Linux desktop under Xvfb.

VSCode: use the `music (debug)` and `music: integration test` launch configs
(`.vscode/launch.json`).

## Commits

Conventional Commits (enforced by `commitlint.yml`). `/caveman-commit` produces
a compliant short message.

## Token discipline (skills)

- **rtk** (Rust Token Killer): installed globally and applied automatically via a
  hook — shell commands are proxied transparently to cut tokens.
- **caveman**: always-on output compression for Claude Code sessions. Install once
  per machine (see `README`/SUPPORT); auto-activates each session. `/caveman lite`
  or uninstall to disable.

## Before opening a PR

- `melos run analyze` and `dart format` clean
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`
- Tests pass and coverage ≥ 80% (Rust + Flutter)
- Ran `flutter_rust_bridge_codegen generate` if the Rust **public API** changed
- Consider `/code-review` and `/security-review` before requesting review
