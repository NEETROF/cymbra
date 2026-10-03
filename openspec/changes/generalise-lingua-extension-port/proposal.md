# generalise-lingua-extension-port — the extension asks the engine in a language

## Why

Since #626 the engine serves every studied language, and each language-bound binding takes the
language as its last parameter. The extension never passes one. Its `LinguaPort` has 40 methods
and none carries a language. Ten surfaces consume it through three implementations:
- the engine in the page;
- the messaging port, which forwards to the event page on Firefox and Safari;
- the host that answers it.

As long as a call can leave the language out, the day a second pack is loaded a Spanish page
would be answered in English, silently. This change makes the language part of every
language-bound call, enforced by the type checker, while every surface still passes English.
It is change 7 of the Spanish programme (`docs/lingua/spanish-programme.md`), the last port
change of R2, and the first build of that silent English release worth dogfooding.

## What Changes

- **The port takes two shapes.**
  - `LinguaPort` keeps what concerns the whole reader: backup, restore, resets, counts, the review
    session, sync exports and applies. It gains `for(language)` and `languages()`.
  - `LanguagePort`, bound to one studied language, holds the 21 language-bound calls: reading,
    calibration, statuses, declared level, levels, ladder, estimate, exposures, deck capture and
    attributions. It extends the reading `AnalyzerPort`, which keeps its methods.

  A surface cannot reach a language-bound call without naming a language: the compiler refuses
  it.
- **`StudiedLanguage` (`"en" | "es"`)** sits beside `STUDIED_LANGUAGE`, which stays `"en"`. That
  is the language every surface passes until `add-lingua-studied-language-profile`, through one
  constant that change replaces.
- **The three implementations follow.**
  - The in-page port passes its bound language to each engine call, through the engine's
    `language` parameter, and reads `languages()`.
  - The messaging port sends the language with each language-bound request (a `language` field on
    the RPC request), and the event-page host answers it through `port.for(language)`.
  - The hand-written `WasmEngine` interface mirrors the engine's bindings (`language?`, `addPack`,
    `languages`).
- **Every surface names its language.** The reading session, selection card and scan, Réglages
  (every host), onboarding, stats, the review page's attributions, the storage migration, the
  level-choice prompt and the sync's post-sign-in calibration all go through
  `port.for(STUDIED_LANGUAGE)`.
- **Tests.** The shared fake port implements `for`, returning a view that records the language it
  was asked in, so the existing specs keep their assertions and new ones check the language. The
  RPC round trip carries the language.
- **English does not move.** Where the engine received nothing it now receives `"en"`, which
  resolves to the same pack and language. The English invariance baseline also runs every probe
  with `"en"` named and expects the same golden. Nothing a reader sees changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *Every language-bound call names its studied language*.
  The extension never asks the engine a language-bound question without a language, on every
  variant and through every transport, and asks in English in this release. The open changes
  touching this capability hold requirements under other names. Nothing they hold is rewritten.

## Impact

- **Products.** Cymbra Lingua only. In `apps/lingua-extension` this touches `src/analyzer`, the
  ten surfaces named above, and their tests. The Safari app ships the same extension code, so it
  follows. The engine is unchanged, except that its baseline test also names `"en"`. The agent
  plugin is untouched. ID, Music, Live, the back office and the site are untouched.
- **Release.** Part of R2, a silent English release. This is the first build to dogfood the
  generalised path on Chrome, Firefox desktop and Android, and Safari before that release.
- **Risk.** High in the study, because it touches every surface. It is mitigated by:
  - the type-level split, so a forgotten call site does not compile;
  - the 1,566 existing tests;
  - the RPC round-trip tests;
  - the doubled invariance baseline.
