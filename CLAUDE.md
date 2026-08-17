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

**Un seul root OpenSpec pour tout le monorepo**, découpé par **préfixe de
capability** (`specs/` est plat — un sous-dossier serait lu comme une capability
vide, et le CLI ne remonte pas l'arborescence : il faut lancer `openspec` depuis
la racine du repo) :

| Préfixe | Périmètre |
|---|---|
| `id-` | Cymbra ID — identité, comptes, sessions, profils publics |
| `music-` | Cymbra Music — `apps/music` + catalogue |
| `live-` | Cymbra Live — web + Tauri |
| `platform-` | socle transverse — flags, jobs, observabilité, i18n, DB |
| `admin-` | back-office Vue |
| `site-` | site public `cymbra.app` — `apps/site` (Astro) : vitrine, légal, connexion web, rachat de code, checkout |

Un produit **consomme** le socle, il ne le redéclare pas : avant toute nouvelle
capability, vérifier qu'une `id-*`/`platform-*` (ou son équivalent legacy) ne la
couvre pas déjà. Les capabilities sans préfixe précèdent la convention et ne sont
**pas** renommées en masse — seulement quand un change les touche déjà. Le
mapping legacy → domaine et les règles par artefact sont dans
[openspec/config.yaml](openspec/config.yaml).

## State management — Riverpod 2 + Freezed (codegen)

Mandatory for all app state:
- **Riverpod 2** providers/notifiers via code generation (`@riverpod` +
  `riverpod_generator`). No new `ChangeNotifier`/`setState` for app state.
- **Freezed** immutable models for state (`@freezed`, mutate via `copyWith`).
- **Dependencies are providers** (e.g. `midiServiceProvider`, `scoreSourceProvider`),
  overridden in tests via `ProviderScope`/`ProviderContainer` overrides — not
  constructor injection. The override value is a **mockito-generated mock by default**
  (hand fakes only for special cases — see the `flutter-testing` skill).
- Reference implementation: [player_notifier.dart](apps/music/lib/state/player_notifier.dart),
  [player_data.dart](apps/music/lib/state/player_data.dart),
  [midi_service.dart](apps/music/lib/services/midi_service.dart).
- `riverpod_lint`/`custom_lint` is enforced (`dart run custom_lint`).

**Layering & reactivity rules** (see the `flutter-riverpod-architecture` skill):
- **UI never calls a service directly.** Only notifiers call `services/`/gRPC
  clients; widgets/screens call *notifier* methods, never `ref.read(*ServiceProvider)`
  to invoke a side effect.
- **A provider never imperatively invalidates a *sibling* provider.** The dependent
  provider `ref.listen`s the source and `ref.invalidateSelf()`s (invalidating *itself*
  is fine; poking another is not).
- **Never `await` a notifier action's return in the UI and branch on it.** Fire the
  action; react to the resulting state (`AsyncValue` loading/data/error) via a listener.
- **Isolate `ref.listen` side effects** (navigation, snackbars, dialogs, reacting
  invalidations) in a **dedicated listener widget** near the top of the feature
  subtree — not scattered in build methods.

**Codegen**: generated `*.g.dart`/`*.freezed.dart` are gitignored and produced by
`build_runner` — run it before analyze/test (CI does this automatically):
```bash
cd apps/music && dart run build_runner build --delete-conflicting-outputs
# or: melos run generate
```
Notifier rule: never read or assign `state` inside `build()` before it returns —
compute the initial value and return it.

### Web front-ends (Vue 3 + TS)

For Vue web apps (e.g. `apps/back-office`), follow the **`vue-frontend-architecture`**
skill (`.claude/skills/`). The public site `apps/site` is Astro (static, fr/en); its
interactive islands (sign-in, code redemption, account, checkout — Vue) follow the
same two rules and consume the shared **`packages/web-auth`** package
(`@cymbra/web-auth`, Yarn `portal:` — Google/Apple sign-in composables + the web-auth
client; one implementation for the back office and the site, never a copy). Two hard
rules, applied proactively:
- **A screen/component NEVER calls an API directly** — only Pinia stores/composables
  do, behind an injectable client seam (`lib/api.ts` + `setClientsForTest`).
- **Async state is one `ts-pattern` discriminated union** (`Async<T>`:
  `idle|loading|success|error`), matched with `match(...).exhaustive()` — never
  scattered `loading`/`error`/`data` refs. Errors live in the union, not thrown.

Reference: `apps/back-office/src/lib/async.ts` + `src/stores/catalog.ts`.

## Test coverage — minimum 80%

Every change keeps or raises **line coverage ≥ 80%** for both ecosystems; new
code needs tests. CI fails under 80% and also reports to SonarCloud (decoration).

- **Rust**: `cargo llvm-cov --workspace --fail-under-lines 80` (excludes the
  generated bridge, `lib.rs`, the hardware/thread glue in `api/midi.rs`, the
  thin MusicXML FFI seam in `api/musicxml.rs`, and the cpal/rustysynth audio glue
  in `api/audio.rs`, `api/renderer.rs` and `api/platform_log.rs` (`api/android_output.rs`
  is cfg-gated off the host build already), and the thin Postgres/HTTP adapters of the
  backend crates (`pg*.rs`, `grpc.rs`, provider webhook glue).
  Keep pure, testable logic in host-testable modules like `api/midi_core.rs`,
  `api/musicxml_core.rs` and `api/audio_core.rs`. Trait dependencies are doubled
  with **mockall generated mocks by default** (hand fakes only for special cases —
  see the `rust-testing` skill).
- **Flutter**: `flutter test --coverage` (unit + widget) merged with the
  integration run, gated by `very_good_coverage` (excludes `lib/src/rust/**`,
  `main.dart`, generated `*.g.dart`/`*.freezed.dart`). Keep the native FFI behind
  an injectable seam (see `lib/services/midi_service.dart`) so widgets/state are
  testable without the native library.

Run locally before pushing:
```bash
cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex 'frb_generated|/lib\.rs|/midi\.rs|/musicxml\.rs|/audio\.rs|/renderer\.rs|/platform_log\.rs'
cd apps/music && flutter test --coverage --exclude-tags golden   # then check lcov
```

## Tests: layers

- **Unit/widget** (`apps/music/test/`): fast, mocked deps (mockito by default), no
  native lib. Default gate. See the `flutter-testing` skill for the double convention.
- **Golden** (tagged `golden`): pixel comparisons, platform-sensitive — excluded
  from the cross-platform gate. Refresh on a pinned platform with
  `flutter test --tags golden --update-goldens`.
- **Integration** (`apps/music/integration_test/`): drive the real app + FFI.
  Headless pass/fail: `melos run integration` (`flutter test integration_test -d
  macos`; CI: Linux desktop under Xvfb). To **watch** the app being driven, run
  `melos run integration-watch` — it uses `flutter drive` on a **visible iOS
  simulator** (boot one first, `open -a Simulator`) with ~1.2s pauses between
  steps. macOS *desktop* renders a black window under the integration_test
  binding, so it can't be watched — use a simulator. The pause is the optional
  `--dart-define=WATCH_MS=<ms>` (0 in CI, so the gate stays fast). The test uses
  its own MusicXML fixture (`integration_test/support/fixture_score.dart`), not
  the shipping scores.

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

## Code knowledge graphs (Graphify — opt-in)

Optional local tooling that lets an AI assistant answer *relationship* questions
("what calls X", "blast radius of changing Y", "what are the architectural hubs")
from a knowledge graph instead of blind grepping. Local AST only — no API key, no
tokens, nothing leaves the machine. **Not** in CI, **not** required to build.

Install once per machine, then build the graphs:
```bash
uv tool install graphifyy && graphify install   # once
scripts/graphify.sh                              # build/refresh all three (~5s)
scripts/graphify.sh install-hook                 # optional: background refresh after every commit
```

Three separate per-stack graphs (git-ignored, rebuilt locally):
- `graphify-out/graph.json` — **Rust workspace** (backend + crates + apps/music/rust;
  `frb_generated.rs` excluded). High quality: method-level call graph.
- `apps/music/graphify-out/graph.json` — **Flutter** app. Coarser (file/import-level;
  Dart extraction is weaker — `_` nodes pollute the hubs).
- `apps/back-office/graphify-out/graph.json` — **Vue** back office. Good quality.

Query from the **repo root** (default graph is `./graphify-out/graph.json` = Rust;
add `--graph <path>` to hit an app graph):
```bash
graphify god-nodes --top 10                  # the most-connected symbols = architectural hubs
graphify explain "EnqueueRequest"            # a symbol + everything wired to it (callers/callees), with file:line
graphify affected "EnqueueRequest" --depth 2 # reverse impact: what breaks if you change it (run before a refactor)
graphify god-nodes --graph apps/back-office/graphify-out/graph.json   # query the Vue graph instead
```
Symbol names must be **exact** (weak fuzzy matching). Best for Rust impact/orientation.
It **complements** grep — it points to `file:line` anchors, it doesn't replace reading
them. Cross-stack (Rust↔Dart↔Vue) is deliberately NOT linked: each graph is one
language; cross the boundary via the `.proto` / frb API contract.

**First run / gotchas.** The graphs are **git-ignored — they are never committed**, so a
fresh clone or a new worktree has none: run `scripts/graphify.sh` there once to build them
(each checkout keeps its own under its own `graphify-out/`). After `uv tool install`, open a
**new shell** (or `hash -r`) so `graphify` is found. `command not found` → new shell;
`graph.json` not found → you're not at the repo root, or you haven't built the graphs yet.

### Docs graph (OpenSpec — semantic, opt-in, costs tokens)

Optional 4th graph over the OpenSpec corpus (`openspec/specs/` + in-flight
`openspec/changes/`; `changes/archive/` excluded) for *terminology/concept*
questions — "which specs mention entitlement", "how is moderation wired to
propose/attestation" — with file anchors. Unlike the three AST graphs it is
**LLM-extracted** (concept nodes + relations, EXTRACTED/INFERRED/AMBIGUOUS
provenance), so building it **costs tokens**: it is NOT part of
`scripts/graphify.sh` and must **never** run from the post-commit hook.

- **First build** (from an AI session via the graphify skill): exclude the
  archive first — `printf 'changes/archive/\n' > openspec/.graphifyignore`
  (git-ignored) — then run the skill on `openspec/`. One-shot ~1M tokens for
  ~190 files.
- **Refresh**: rerun with `--update` — the per-file extraction cache
  (`openspec/graphify-out/cache/`, keyed by content hash) makes it pay only for
  new/changed `.md` (~6k tokens/file). Archiving a change re-extracts only the
  touched `specs/` files.
- **Query** (same CLI, from the repo root):
  `graphify query|explain "<Concept>" --graph openspec/graphify-out/graph.json`.
- **One home only**: keep the graph + cache in your main checkout; worktrees
  query it via `--graph <main-checkout>/openspec/graphify-out/graph.json`
  instead of rebuilding (a cache-less rebuild re-pays the whole corpus).

## Before opening a PR

- `melos run analyze` and `dart format` clean
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`
- Tests pass and coverage ≥ 80% (Rust + Flutter)
- Ran `flutter_rust_bridge_codegen generate` if the Rust **public API** changed
- Consider `/code-review` and `/security-review` before requesting review
