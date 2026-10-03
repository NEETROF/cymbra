# package-lingua-packs-per-pair — load a pair's pack the first time it is needed

## Why

Since #630 a package carries the pack of every pair `packs.json` lists, but the engine loads only
the default pair's. A view in another listed language would reach the engine and be refused: no
pack for that language. Loading every listed pack at start would answer it, at a price. The
study measured a loaded pack at about 16 MiB of WASM memory whatever its delivery, and memory is
what gets an extension stopped by the OS on a phone.

So the engine should hold the packs the reader actually uses, loaded the first time each is
needed. This is change 9 of the Spanish programme (`docs/lingua/spanish-programme.md`), still
within R2, a silent English release. The list is still en-fr, so nothing new loads in this
release. The path is ready, and tested, for the day Spanish is listed.

## What Changes

- **A listed pair's pack loads on first use.** The first call of a view in a language fetches
  that pair's `assets/packs/<pair>.lingua` and hands it to the engine (`addPack`) before the
  call. It happens once per engine: concurrent first calls share one load, and a failed load is
  retried by the next call rather than remembered.
- **Synced records bring their packs.** A whole-reader apply (status, card or declared-level
  changes) first loads the packs of the listed languages its records name. A record of a language
  the reader studies is never skipped for want of a loaded pack. Records of languages no listed
  pair studies are skipped by the engine, as before.
- **An unlisted language is refused before the engine**, with an error naming the language and
  the shipped pairs.
- **Nothing else loads.**
  - The default pack still loads with the engine.
  - Whole-reader calls (backup, counts, review, exports) never load a pack.
  - A restore keeps the state of languages without a loaded pack untouched (#626).
- **One mapping from language to pair** (`src/analyzer/pairs.ts`): the first listed pair that
  studies the language. The port takes the list as a parameter that defaults to the bundle's, so
  specs exercise two pairs.
- **Every variant gets it.** The background host on Firefox and Safari, and on Chromium pages
  whose policy blocks the in-page engine, uses the same port class.
- **English does not move.** With the list holding en-fr, the engine loads the same single pack
  at the same moment as before, and fetches nothing else.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *A language's pack is loaded the first time it is
  needed*. The open changes on this capability hold requirements under other names, and nothing
  they hold is rewritten.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension/src/analyzer` (the in-page port and a
  small module mapping languages to pairs) and its tests. The engine, the packs, the build, the
  manifest and the gate are unchanged (#630 already packages every listed pack). ID, Music, Live,
  the back office and the site are untouched.
- **Release.** Part of R2, a silent English release. No reader-visible change.
- **Memory.** An engine holds one pack per language used, and no more. The other levers the study
  measured (lazy pack decoding, a sync engine without a pack, the translation engine's mobile
  policy) are not in this change.
