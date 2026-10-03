# Design — generalise-lingua-pack-build

## Context

See proposal.md (Why). In `apps/lingua-extension` today:

| Where | What it assumes |
|---|---|
| `package.json` | `gen:pack` → `tool/gen_pack.sh assets/pack.lingua` (testdata en-fr); `gen:pack:real` → `scripts/lingua-data/build.sh en-fr assets/pack.lingua`; `gen:fixtures` → the committed vitest fixture |
| `build.mjs` | `assertPackMatchesEngine()` reads `assets/pack.lingua`'s `analyzer_version` by scanning the container's ASCII metadata, and compares it with `ANALYZER_VERSION` read from `mod.rs`. The copy list has `["assets/pack.lingua", "assets/pack.lingua"]`. |
| `manifest.json` | `web_accessible_resources` names `assets/pack.lingua` |
| `src/analyzer/engine.ts` | `PACK_PATH = "assets/pack.lingua"` |
| `tool/make_source_archive.sh` | the en-fr pin, and the README line with its sha256 |
| `lingua-extension-check` | the reviewer rebuild runs `yarn gen:pack:real` and compares `assets/pack.lingua` with `$RUNNER_TEMP/pack-en-fr.lingua` |
| `lingua-extension-release`, `lingua-apple-release` / `-build` | call `yarn gen:pack:real` / `yarn gen:pack`, then build |

`scripts/lingua-data/build.sh <pair> <out>` already takes a pair and checks the built pack against
that pair's `pin.json`. `lingua-extension-check` already builds every `tables/*` folder (#619).
The pack's metadata carries `studied` and `analyzer_version` (#624).

## Goals / Non-Goals

**Goals:**
- One list, read by every tool, that says which pairs a package ships.
- Each pair's pack built and checked on its own terms.
- A gate that keeps R2's packages English-only.

**Non-Goals:**
- Loading more than one pack at run time, a pack per language on demand, and the memory policy
  (`package-lingua-packs-per-pair`).
- The reader's choice of language (`add-lingua-studied-language-profile`).
- The agent plugin's pack (`add-lingua-agent-languages`).

## Decisions

### D1 — `packs.json`, read through `tool/packs.mjs`

`apps/lingua-extension/packs.json` holds `{ "pairs": ["en-fr"] }`, and the first pair gives the
default language. `tool/packs.mjs` is the only reader. It exports:
- `shippedPairs()`;
- `packFile(pair)` → `assets/packs/<pair>.lingua`;
- `studiedOf(pair)`;
- `coreAnalyzerVersion(language, modRsSource)`;
- `packMeta(bytes)`, which scans the container's ASCII metadata for `studied` and
  `analyzer_version`.

`build.mjs`, `check_variants.mjs`, and the shell scripts (through `node tool/packs.mjs pairs`) all
use it. A vitest spec covers it.

*Rejected — a field in `package.json`.* release-please rewrites that file. A file of its own reads
as what it is, and its history is the history of what shipped.

*Rejected — derive the list from `scripts/lingua-data/tables/`.* A pair being built is not a pair
being shipped. Spanish tables will exist (G1) long before Spanish ships (R4).

### D2 — A path per pair

Packs live at `assets/packs/<pair>.lingua`, gitignored, in the source tree and the package alike.
`gen:pack` and `gen:pack:real` build every listed pair there:
- `tool/gen_pack.sh [--real] <dir>` loops over the list;
- `--real` calls `build.sh <pair>`, which checks the pin;
- the testdata mode needs `scripts/lingua-data/testdata/<pair>/`.

A pair missing what its mode needs fails with the path to add. `gen:fixtures` keeps writing the
committed `test/fixtures/en-fr.testdata.lingua`.

### D3 — Each pack against its own language

`build.mjs` checks every listed pack before bundling:
- the file exists;
- its metadata's `studied` is the pair's studied side;
- its `analyzer_version` equals lingua-core's version for that language.

The language → constant map (`en` → `ANALYZER_VERSION`, `es` → `SPANISH_ANALYZER_VERSION`) lives
in `tool/packs.mjs`. A spec reads `crates/lingua-core/src/analysis/language.rs` and fails when a
language has no entry, or its entry names another constant, so the map cannot drift from the core.
The message keeps today's shape: the pack, both versions, and the command that rebuilds it.

### D4 — The bundle learns the list

`build.mjs`:
- copies each listed pack;
- writes each one into `web_accessible_resources`, replacing the `assets/pack.lingua` entry;
- defines `__LINGUA_PACKS__` (the list, JSON) for esbuild, beside `__TARGET__`.

`engine.ts` fetches `assets/packs/${__LINGUA_PACKS__[0]}.lingua`, the default pair. It loads one
pack, as today.

### D5 — The list is a gate until Spanish is enabled

`check_variants.mjs` reads the list and requires two things:
- every bundle holds exactly `assets/packs/<pair>.lingua` for each listed pair, and its manifest
  exposes them;
- the list equals `SHIPPED_PAIRS = ["en-fr"]`, a constant in the check whose comment names
  `enable-lingua-spanish` as the change that widens it.

The duplication is the point: widening what ships takes two edits in one pull request, and one of
them is in a file reviewers know as the variant gate.

### D6 — Review and release

- `make_source_archive.sh` copies each listed pair's tables and pin, and writes one README line
  per pack (its path and sha256).
- The check lane's reviewer rebuild runs `yarn gen:pack:real`, then compares each listed pack with
  the one built from the tables (`$RUNNER_TEMP/pack-<pair>.lingua`).
- The release lanes keep their command, whose meaning grows.
- REVIEWERS.md and the README give the per-pair commands and paths.

### D7 — English invariance

The en-fr pack's bytes are unchanged: `build.sh en-fr` still matches `pin.json`. The S0 baseline
builds its pack from the tables, so it does not see the path. Nothing a reader sees changes:
- the extension's tests pass;
- `yarn build && yarn check:variants` passes;
- the engine loads the same bytes from the new path.

## Risks / Trade-offs

- [A developer's checkout still has `assets/pack.lingua`, and the build now fails] → The message
  names the missing `assets/packs/en-fr.lingua` and the command (`yarn gen:pack`). The old file is
  gitignored and harmless.
- [A store package misses its pack] → `check_variants` refuses a bundle without each listed pack,
  in the pull request and in the release.
- [The map between language and constant drifts when a language is added] → The spec against
  `language.rs` fails first.
- [Firefox for Android or Safari mishandles a nested asset path] → Every asset already sits under
  `wasm/` and `assets/`. The dogfood pass of R2 covers a load on each target.

## Migration Plan

Developers run `yarn gen:pack` (or `yarn gen:pack:real`) once. No reader data or wire changes.
Rollback is a revert.

## Open Questions

None.
