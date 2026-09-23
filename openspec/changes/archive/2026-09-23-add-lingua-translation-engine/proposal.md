## Why

Readers select whole sentences, and fragments they want understood *inside* that sentence. The
pack cannot answer either: its expression table holds fixed idioms of two to five words, so an
ordinary sentence falls through to the word-by-word list — four labelled rows out of nineteen
words, announced as not being a translation. No dictionary can ever hold the sentence a reader
is looking at, because that sentence is not known in advance.

A neural engine can, offline and on the device. This change brings the engine in and proves it
carries its weight, while deliberately reaching no reader: nothing is downloaded, no setting
appears, and no public promise moves. Those come later, once the engine is known to work here.

## What Changes

- **The engine is built from source in our CI**, from `mozilla/translations` at a pinned commit,
  and published as a build artefact. Measured: 7 min 48 s on `ubuntu-latest`, no patching,
  4.7 MB of wasm (1.03 MiB brotli) plus 92 KB of JS glue.
- **A `TranslatorPort` seam**, in the shape of the existing `LinguaPort`: the reading code asks
  for a translation and never learns where the engine runs.
- **The engine runs in a classic Worker, never on a thread that paints.** On Chromium an
  offscreen document owns that Worker, because an MV3 service worker cannot construct one
  (measured: `typeof Worker` is `undefined` there). On Firefox the event page owns it directly.
  Measured: while the engine translates, the page's worst frame gap is unchanged from idle
  (18 ms, against 202 ms when the page itself blocks) and the service worker answers in 1 ms.
- **The answer is the reader's whole sentence, with their selection marked in its French place.**
  The selection is wrapped in a tag and the sentence translated as HTML; the engine repositions
  the tag onto the corresponding French span. Measured: `<b>gave up</b>` comes back
  `<b>a abandonné</b>` — conjugated and agreed with its subject, which the fragment alone never
  gives.
- The model is **side-loaded by hand into a development build**. It is not downloaded, not
  bundled, and not shipped.

Explicitly **not** in this change, each its own later step: the "Traduction étendue" setting,
the engine and model download, any change to store copy or the privacy promises, Safari, and
Android.

## Capabilities

### New Capabilities
- `lingua-translation`: the neural translation engine — where it comes from, where it is allowed
  to run, what it is asked, and what shape its answer takes. It covers the engine's provenance
  and reproducibility, the rule that it never occupies a thread that paints, the seam the
  reading code uses, and the sentence-with-marked-fragment answer.

### Modified Capabilities

None. The reader-visible behaviour does not move in this change: with no setting and no model,
every surface answers exactly as it does today. In particular:

- `lingua-browser-extension`'s **Word-by-word gloss is a labelled last resort** already says a
  card shows those rows "only when it has no better answer". The engine becomes such an answer
  in the change that puts it in readers' hands, and that change carries the modification.
- `lingua-browser-extension`'s **No network requests** is untouched, because nothing here
  fetches anything. The change that introduces the download is the one that must revisit it.

## Impact

**Products.** Cymbra Lingua only — the browser extension (`apps/lingua-extension`) and CI.
Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and no backend,
`.proto` or database is touched. No cost lands on any other product.

**New in this change.** A CI workflow that builds the engine; a `TranslatorPort` and its
implementations; an offscreen document and its Worker on Chromium, a Worker on Firefox; a
development-only path that side-loads a model.

**Consumed, not rebuilt.** `sentenceForRange` already finds the sentence a selection sits in
*by position*. It computed the selection's offsets on the way and discarded them; this change
keeps them (`sentenceAndSelection`), which is exactly what placing the tag needs. The engine
seam mirrors `LinguaPort` and its messaging port rather than inventing a second RPC shape.

**Carried costs.** The offscreen document is a new surface on Chromium and needs the `offscreen`
permission, which store review will see. The engine is a C++/emscripten build in a repository
whose CI otherwise builds Rust, Dart and TypeScript. Both are accepted here rather than
discovered later.

**Not paid here.** Bundle size and the privacy posture are unchanged: no artefact from this
change ships to a reader.
