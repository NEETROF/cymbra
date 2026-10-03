# Design — generalise-lingua-extension-port

## Context

See proposal.md (Why). In `apps/lingua-extension/src/analyzer/`:

| File | Role today |
|---|---|
| `port.ts` | `AnalyzerPort` (reading: `analyse`, `setCalibration`, `setStatus`, `gloss`, `phraseGloss`, `wordGrammar`) and `LinguaPort extends AnalyzerPort` (34 more). No method takes a language. |
| `engine.ts` | `WasmAnalyzerPort`: builds the engine lazily (one pack, `assets/pack.lingua`) and calls it through a hand-written `WasmEngine` interface. |
| `messaging-port.ts` | `MessagingLinguaPort`: `rpc(method, args)` for every method. |
| `rpc.ts`, `rpc-host.ts` | `RpcRequest {type, method, args}`; the host calls `port[method](...args)` on the hosted engine. |
| `create-port.ts` | Chooses the messaging port on Firefox and Safari, and on Chromium pages whose policy blocks the in-page engine (`resolveContentPort`). |

About 40 call sites in ten surfaces use language-bound methods:
- `reading/session.ts` (10), `reading/selection-card.ts` (through its `ports` bundle) and `reading/scan.ts`;
- `reading/settings-view.ts` (`mountSettings`, shared by every Réglages host);
- `onboarding/onboarding.ts`, `stats/view.ts`, `review/review-page.ts` (attributions);
- `state/storage.ts` (the v1 migration), `state/level-choice.ts`;
- `sync/sync.ts` (one calibration after sign-in).

The tests fake the port through one helper, `makeFakePort` (`test/helpers.ts`), plus a few local
fakes.

Since #626, the engine takes `language?: string | null` last on its 21 language-bound bindings,
and adds `addPack` and `languages`. Naming a language the engine holds no pack for throws.

## Goals / Non-Goals

**Goals:**
- No language-bound call compiles without a language.
- The language reaches the engine on every variant and transport.
- Nothing a reader sees changes, and the tests show it.

**Non-Goals:**
- Choosing the language, the profile that stores it, or a second pack in the bundle
  (`add-lingua-studied-language-profile`, `package-lingua-packs-per-pair`).
- Labelling synced cards by language (`add-lingua-language-sync-client`), detection between
  languages (`add-lingua-language-routing`), and settings per language
  (`add-lingua-language-choice`).
- A language on the review card (`add-lingua-language-stats-review`).

## Decisions

### D1 — Split by type: a root port and a language view

```ts
export type StudiedLanguage = "en" | "es";

export interface AnalyzerPort {           // reading, unchanged
  analyse(blocks: string[]): Promise<PageAnalysis>; /* … gloss, phraseGloss, … */
}
export interface LanguagePort extends AnalyzerPort {
  readonly language: StudiedLanguage;
  calibration(): Promise<number>; /* … the 21 language-bound calls … */
}
export interface LinguaPort {             // the whole reader
  for(language: StudiedLanguage): LanguagePort;
  languages(): Promise<StudiedLanguage[]>;
  backup(): Promise<string>; /* … counts, review, sync … */
}
```

The language-bound calls are those design D2 of `generalise-lingua-wasm-engine` lists. They
leave `LinguaPort`, so a forgotten call site is a type error, not a silent default.

*Rejected — keep a flat port with an optional trailing language.* That is the silent default this
change exists to remove.

*Rejected — make the language the first parameter of every method.* It churns every signature
and every fake, and reads worse than `port.for(language).analyse(blocks)`.

*Rejected — a "current language" on the port.* Tabs share the background engine on Firefox and
Safari, and their requests interleave.

### D2 — Views are stateless wrappers

`for(language)` returns a small object holding what its parent holds plus the language:
- the in-page view holds the engine promise;
- the messaging view holds the `send` function.

Views are not cached. They are cheaper than the call they make, and holding nothing means
nothing to lose when Safari suspends the background.

### D3 — The RPC carries the language

`RpcRequest` gains an optional `language`. `MessagingLanguagePort` sets it, and the root
`MessagingLinguaPort` leaves it out. The host answers on `port.for(request.language)` when it is
set, and on `port` otherwise. An unknown method on either target is the error it is today.

The content script, extension pages and background ship in one build, so neither side meets the
other's older shape.

### D4 — One constant, one seam

Every surface asks `port.for(STUDIED_LANGUAGE)`, and so does a surface that keeps a port as a
field (the reading session). `STUDIED_LANGUAGE` stays `"en"` with type `StudiedLanguage`.
`add-lingua-studied-language-profile` replaces it with the reader's languages; a grep of
`STUDIED_LANGUAGE` lists every place it has to look. `scan.ts` and the selection card take a view
(`AnalyzerPort`), so they never see the root.

### D5 — The engine mirror follows the engine

`WasmEngine` in `engine.ts` is the extension's own description of the wasm object (the glue is
dynamically imported, so the generated types do not reach it). It gains `language?: string | null`
on the bound methods, plus `addPack` and `languages`. The in-page view passes its language
explicitly.

### D6 — Tests keep their assertions

`makeFakePort` gains `for(language)`: it returns the fake itself, extended with the requested
`language`, and records each language asked in `calls.languages`. Specs that drive surfaces keep
their expectations, and new assertions check that each surface asks in `"en"`. Local fakes get the
same `for`.

New specs:
- the in-page view hands the engine its language (fake wasm);
- the messaging view sends it;
- the host dispatches to the view, and reports an unknown method.

### D7 — How English is shown not to move

- The S0 harness (`crates/lingua-wasm/tests/english_baseline.rs`) renders its probes twice, with
  no language and with `"en"`. Both must equal the golden, which stays untouched.
- `yarn typecheck`, `yarn lint`, `yarn test` (with the coverage gate), `yarn format:check` and
  `yarn build` must pass for the three variants.
- The built bundles are grepped: the RPC `language` field appears where the messaging port is
  compiled in.

### D8 — OpenSpec

One ADDED requirement in `lingua-browser-extension`, under a name no open change holds. Nothing
held is rewritten.

## Risks / Trade-offs

- [A surface asks in the wrong place, e.g. the popup's own port versus the content script's] →
  Each host already builds its own port (`createLinguaPort` / `resolveContentPort`), and the
  change adds `for(...)` at the call, not a new port.
- [A test fake returns a view that drops calls] → The fake's view is the fake itself, so call
  records stay in one place.
- [The RPC field is forgotten for one method] → The messaging view sends every call through one
  `rpc(method, args)` that always sets `language`. The root never does.
- [Dogfooding finds a regression the tests missed] → The proposal makes this build the R2 dogfood
  build. The release waits for a pass on each target (the programme's R2 release, owned by the
  product owner).

## Migration Plan

No stored data, wire field to the server, or pack changes. The RPC is internal to one build.
Rollback is a revert.

## Open Questions

None.
