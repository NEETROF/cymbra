# Design

## Context

See proposal.md — Why. What constrains the shape:

- The catalogue (`model-manifest.json`) lists models by id (`<from>-<to>/base-memory/<version>`),
  each with three files pinned by address, served size, decompressed size and sha256, a source path
  in Mozilla's registry and the gzip's sha256, a mirror release and a licence; and routes keyed by
  pair. The parser refuses a route that does not start from the studied language, chain, or end in
  the native language. `readerPairs` gives a reader the shipped pairs of their native language, so a
  route of an unshipped pair costs nothing.
- Mozilla's Remote Settings (`translations-models`, read on 2026-10-21) hold it-en at 1.0 and 2.0
  and en-it at 1.0, 2.0 and 2.1: `model.iten.intgemm.alphas.bin` (31,561,787 B decompressed),
  `lex.50.50.iten.s2t.bin` (4,713,616 B), `vocab.iten.spm` (812,724 B); `model.enit.intgemm.alphas.bin`
  (31,561,787 B), `lex.50.50.enit.s2t.bin` (4,133,192 B), `vocab.enit.spm` (812,724 B). The two
  vocabularies carry the same hash.
- The engine holds two models at most (matrix change 9) and respawns after a trap; the soak tool
  runs a pair's route over the committed corpus of the pair's *studied* language — 100 selections
  each for English, Spanish and French; none for Italian.
- The release gates (`lingua-apple-release`, `lingua-extension-release`) refuse a package whose
  catalogue names a model the host does not serve; `lingua-model-deploy` is manual, the owner's.
- Matrix changes 25 (en-es 2.1) and 50 (fr-en 2.0) did exactly this for their models; both are open
  and hold the catalogue requirement.

## Goals / Non-Goals

**Goals:**

- Every Italian pair has its route the day its pack ships, measured through the catalogue like the
  others.
- One deploy of the model host for the whole programme.
- The en-it model tried on real inputs before any Italian speaker meets it.

**Non-Goals:**

- Marking a selection in an Italian pair (`MARKED_PAIRS`): rows 17, 22, 27, 36, 47, 51 measure and
  add each pair.
- The Italian corpus of selections (row 17).
- Anything the setting shows an Italian-native reader: `add-lingua-italian-native-language`
  (change 32) and `add-lingua-italian-interface` (33).

## Decisions

### D1 — The entries are read from the files, and checked against Remote Settings

As matrix change 25's D1: the six gzip files are downloaded once from Mozilla's registry
(`sourceBase` + the entry's run path); `source.sha256` is each gzip's sha256 and `size` its length;
`sha256` and `unpacked` are the decompressed bytes'. Each file's decompressed sha256 must equal the
one Remote Settings publish for it-en 2.0 and en-it 2.1 (D1 of change 25 also checked the
registry's `uncompressedHash`; the same check applies). A mismatch stops the change: the registry
would then not be at those versions. Ids `it-en/base-memory/2.0` and `en-it/base-memory/2.1`, paths
`<id>/<decompressed sha256>/{model.bin.gz,lex.bin.gz,vocab.spm.gz}`, mirrors
`lingua-model-it-en-base-memory-2.0` and `lingua-model-en-it-base-memory-2.1`, licence MPL-2.0.
`assemble_model_site.mjs` run locally over the new catalogue proves both entries: it keeps a file
only when every digest holds.

*Alternative.* Pinning it-en 1.0 or en-it 2.0 — no: the sizes are those of the same model family,
Remote Settings serve the latest to Firefox, and a version Mozilla moved on from is one the mirror
alone would keep alive.

### D2 — Six routes, appended, two models at most

`"it-en": [it-en 2.0]`, `"en-it": [en-it 2.1]`, `"it-fr": [it-en 2.0, en-fr 2.0]`,
`"it-es": [it-en 2.0, en-es 2.1]`, `"fr-it": [fr-en 2.0, en-it 2.1]`, `"es-it": [es-en 2.0, en-it 2.1]`,
appended after the matrix's six; the models listed in the order they were pinned (en-fr, es-en,
en-es, fr-en, it-en, en-it). Each route starts from its studied language, chains and ends in its
native language, so the bundled catalogue loads. Nothing a reader of today holds changes:
`readerPairs` names shipped pairs only, and no shipped pair studies Italian or is glossed in it.

Per native language, the union of the routes now needs three models (French: en-fr, es-en, it-en;
English: es-en, fr-en, it-en; Spanish: en-es, fr-en, it-en; Italian: en-it, fr-en, es-en) where
change 50's requirement *French is translated into English directly, and into Spanish through
English* bounds them at two together; that requirement is modified here (three once Italian's
pairs are routed, loaded two at a time), and its test — `test/model-manifest.spec.ts`, *Every
native language's pairs*, which asserts the routes into each native and the download sums —
is rewritten for the four natives. The device keeps what the reader's pairs need; the engine
still holds two at a time — a route's — and evicts the model no current route needs (change 9).
The cost the setting states is the sum over the reader's pairs' models, as the requirement *The
setting's cost comes from the catalogue* says; an Italian-native reader is
`add-lingua-italian-native-language` (32), `add-lingua-italian-interface` (33) and
`enable-lingua-italian-speakers` (45)'s concern.

### D3 — A file the two models share

`vocab.iten.spm` and `vocab.enit.spm` carry one hash in Remote Settings: the device stores files
by decompressed sha256, so a reader who holds one model and needs the other does not download the
vocabulary again, and `prune` keeps it while either model is kept — exactly change 25's D3 for
es-en and en-es, with the same consequence for the cost shown (counted twice, fetched once).

### D4 — Three soaks by hand, the fourth deferred

`node --experimental-strip-types tool/soak_engine.mjs --pair en-it|fr-it|es-it --models <dir>
--isolate` over the English, French and Spanish corpora: the en-it model is the new one on every
route, and each corpus is the route's studied language's. The it-en model has no corpus to run over
until row 19 (`measure-lingua-it-fr-translation-marks`) selects the Italian one (`select_corpus.mjs`
over UD Italian-PUD, the rule every language shares) — run on it-en today, the tool reports no
selection rather than refusing; its soak is row 19's, before it-fr ships (row 20,
`enable-lingua-italian`). Figures and trapping ids go to `TRANSLATION.md`, as en-es's did.

### D5 — Documents and tests

`TRANSLATION.md` (the models, the routes, the shared vocabulary, the soaks, its « four models …
two together » notes), `tool/marks/README.md` (the pairs the harness can route; its *Soaking a
route* bound paragraph; the Italian corpus named as row 19's), `REVIEWERS.md` (« lists four
models » becomes six), `tool/soak_engine.mjs`'s usage (the twelve routes).
`test/model-manifest.spec.ts` holds the catalogue exactly, and a test over today's shipped list
shows that no reader's pairs need an Italian route and no reader fetches an Italian model, in the
shape of *A route of a pair studying French*.

### D6 — The merge and the deploy

Merging the implementation makes every store submission of the extension and every App Store
delivery of the app refuse until the owner dispatches `lingua-model-deploy` (the host must serve
it-en 2.0 and en-it 2.1; the workflow creates the two mirror releases on the way); builds and
pull requests pass. The implementation is merged when the owner can dispatch right after, and the
pull request says so in its body.

## Risks / Trade-offs

- [The en-it model traps the engine on some inputs, as en-es did] → the three soaks before any
  Italian speaker is offered it; a trap costs one respawn (change 9).
- [The it-en model is pinned without a soak] → no Italian pair ships before row 17 soaks it over
  the Italian corpus; the pin alone changes nothing for a reader.
- [Submissions and deliveries blocked between the merge and the deploy] → D6: merged when the
  owner can dispatch.
- [Mozilla withdraws or moves a file] → the mirror releases, filled by the deploy, serve it.
- [Three models per native language instead of two] → stored, not loaded: the engine's bound is
  per route; the setting's cost is the catalogue's sum, as specified.

## Migration Plan

Nothing deploys with the merge; the owner dispatches `lingua-model-deploy` after it, and
`check_model_host` passes from outside before any release. Rollback: revert the catalogue; a device
holding an Italian model keeps it until `prune` finds no route needing it.
