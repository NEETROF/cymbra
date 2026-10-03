# generalise-lingua-wasm-engine — one engine, a pack per studied language

## Why

The core analyses by language since `generalise-lingua-analysis-by-language` (#624). The WASM
engine the extension drives is still English by construction:
- it holds a single pack;
- 21 bindings hard-code `const EN`, 33 times;
- the sync exports write `"language": "en"` whatever the record, and the applies drop every other
  language.

The programme's architecture (`docs/lingua/spanish-programme.md`) is one engine holding several
packs, with the language on every call. One engine per language would overwrite the other's
backup, and Firefox and Safari serve every tab from one engine. The next change,
`generalise-lingua-extension-port`, binds the TypeScript port to a language and needs an engine
that accepts one. This is change 6, still within R2, a silent English release.

## What Changes

- **A pack per language in one engine.** A host-testable pack set in `lingua-core` holds one pack
  per studied language; the first pack loaded gives the default language. The engine holds such a
  set. `addPack(bytes)` loads another language's pack and returns its tag, and refuses a second
  pack for a language already held.
- **The language on every language-bound binding**, as a trailing optional `language` (ISO
  639-1). This covers:
  - reading: `analyse`, `gloss`, `phraseGloss`, `wordGrammar`;
  - knowledge: calibration, declared level, statuses, exposures and promotion, levels, the
    ladder, the vocabulary estimate;
  - deck capture: `addCard`, `retireCard`, `seedLevel`;
  - attributions: `notice`, `licences`.

  Without a language, a call uses the default language, so every existing call keeps its meaning.
  A language the engine holds no pack for is an error. wasm-bindgen types the parameter as
  optional (`language?: string | null`), so the extension's TypeScript does not change here.
- **Root bindings stay language-less.** Backup, restore, the two resets, the counts, the review
  session, and the sync exports and applies cover every language, as the state already does.
- **Sync records carry their own language.** Each exported status, declared level and card names
  its own language instead of `"en"`. Applies keep the records of languages the engine holds a pack
  for, and skip the others. With only the en-fr pack, that is exactly today's behaviour.
- **Pack-bound work is per language.** The level vocabularies are computed per pack, and a restore
  bounds each held language's exposures against its own pack.
- **English does not move.** The English invariance baseline stays byte-identical: only its
  harness changes, to unwrap the new results, and its golden is untouched. The extension builds,
  type-checks and passes its tests with no change to its source.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *One WASM engine serves every studied language* and *Sync records
  carry their own language*. No existing requirement is rewritten. The open changes touching this
  capability (`add-lingua-phrase-gloss`, `add-lingua-expression-table`) hold other requirements.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core` gains a pack set;
  - `crates/lingua-wasm` (the engine) changes, with its tests.

  The extension's code is unchanged: it keeps loading the en-fr pack and calling without a
  language. The agent plugin has its own engine (`add-lingua-agent-languages`). ID, Music, Live,
  the back office and the site are untouched.
- **Release.** Part of R2, a silent English release. The wasm bundle is rebuilt and behaves
  identically.
- **Compatibility.**
  - The wire is unchanged: records already carried a `language` field, which now names the
    record's own language.
  - The backup format is unchanged. Spanish state can only appear once a reader can load a
    Spanish pack; that later change also brings backup v2.
