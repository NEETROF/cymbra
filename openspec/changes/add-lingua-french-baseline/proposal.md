# add-lingua-french-baseline — French as a studied language: the variant, its golden, backup v3

## Why

Change 39 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the second of stage 3 (French studied: fr-en, fr-es). The programme's architecture says what
French as a studied language gets: « its own analyser version, elision as pieces with their own
spans, and backup schema v3, written only when French is studied, so released builds refuse such a
backup as unsupported rather than malformed ». This change lays the groundwork those words name,
and none of the behaviour that follows: change 40 (`add-lingua-french-tokenisation`, M21), 41
(`add-lingua-french-analysis`), 42 (the Catalan and Occitan guard), 43 (the forms tables), 45
(grammar), 46 (levels), 48 (the fr-en pack) each add behaviour behind what this change puts in
place.

Today the core knows two studied languages. French is a native language only:
`NativeLanguage::French.studied()` answers `None`, « until French is studied ». A `fr` tag is
refused wherever a studied language is read; no pack studying French can load; a backup that named
French would be malformed to every build. Three things have to exist before the first French rule
is written:

- the variant, with an analyser version of its own, so that a French rule never moves English or
  Spanish, and the seams that must answer for a third language are answered once, verbatim for the
  two others;
- a golden that freezes what the engine makes of raw French text today, so that each later change
  of the stage shows its effect as the diff of a re-bless — PUD does not exercise tokenisation (UD
  splits elision already), while elided words are 6.5 % of literary French, so the French analyser
  needs raw-text fixtures from its first day;
- the backup version that keeps released builds honest when French data exists.

The decisions M21 (French tokenisation), M7 (levels), M8 (lemma alternatives) and M6 (fr-es
coverage) are open before stage 3. This change depends on none of them: the French arm is the
baseline analysis the specs already define for a language whose rules are not written — `au`,
`aux`, `du`, `des` and every elision stay whole, nothing is a function word, there is no level and
no second lemma. The golden shows the questions M21 settles; it answers none of them.

## What Changes

- **`StudiedLanguage::French`**, appended after Spanish, tagged `fr`, detected through whichlang's
  French class, listed in `ALL`, with `FRENCH_ANALYZER_VERSION = "0.1.0"`: a `0.x` version, as the
  spec requires of a language served by the baseline.
- **French's arm at every seam is the baseline analysis**: the tokeniser's language-neutral rules
  and no pre-pass of its own (no contraction or elision split, text read as it came), the pack's
  form→lemma lookup else the lowercased form (`lemmatize_baseline`, retired when Spanish got its
  cascade, returns), no function-word table, no names rule. A rule written for English or Spanish
  never runs on French.
- **A pack may study French.** It loads against French's analyser version, as the general rule
  says; `NativeLanguage::French.studied()` is now French, so a French-native reader cannot study
  French and a pack glossed in French cannot study it.
- **A French invariance baseline**, `crates/lingua-wasm/tests/french_baseline.rs`, beside the
  English (S0) and Spanish ones, on the harness they share: a 13-page raw-text corpus
  (`baseline/pages-fr.txt`: authored pages per phenomenon plus two public-domain excerpts, named
  in the design), its probes, and `baseline/fr-en.golden`. It runs over a hand-written fixture
  pack (`scripts/lingua-data/testdata/fr-en/`) until the French tables are committed, with the
  engine started on es-en, as an English-native reader's is. It runs wherever the other two run,
  and `lingua-pack-update` re-blesses the three.
- **Backup schema version 3**, written only when the state names French as a studied language —
  in its profile or in any per-language record — and read first, like 1 and 2. Every released
  build since the studied-language profile reads the version before the rest of the file, so it
  refuses a version 3 backup as « unsupported version 3 », by name, never as malformed. A reader
  of English or Spanish keeps the version they have, byte for byte.
- **Two compile-forced lines outside the core.** `apps/lingua-extension/tool/packs.mjs` maps `fr`
  to its constant (`test/packs.spec.ts` holds that map to `language.rs`, so a variant added there
  fails the extension's tests first); the agent plugin's `language_name` names French. Neither
  changes anything a reader sees: no pair studying French is listed, the extension's `StudiedLanguage`
  type and its labels stay `en | es`, and no French pack exists to install.
- **English and Spanish do not move.** The en-fr and es-fr goldens are unchanged byte for byte,
  without re-blessing; every pin is unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *French is a studied language served by the baseline analysis* (the
  variant, what the baseline does to French text, its version, English and Spanish unmoved) and
  *A French invariance baseline runs beside the English and Spanish ones* (the corpus, the fixture
  pack, where it runs, what a re-bless must say). No existing requirement is rewritten; the open
  changes on this capability hold other requirements.
- `lingua-data-packs`: MODIFIED — *A pack names the language it studies*: the text and its four
  scenarios are kept; three scenarios are added for a French pack and for the shipped packs'
  bytes. No open change holds it.
- `lingua-decks-review`: MODIFIED — *The backup records the reader's language profile*: written on
  top of `add-lingua-native-language-sync-client`'s wording (this change archives after it,
  `archiveAfter`), the version sentence gains schema version 3, every other sentence and every
  scenario kept, four scenarios added. MODIFIED — *A restore reads the backup's schema version
  first*: versions 1 to 3 are read, the scenario names kept (« Both known versions » now reads
  three, « A later version » is 4), one scenario added for a build released before version 3. No
  other open change holds either.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: the variant, its version, the baseline arm, backup schema 3;
    *consumed*: the dispatch seams of `generalise-lingua-analysis-by-language`, the header-first
    restore of `add-lingua-studied-language-profile`, the profile's native-studied check of
    `generalise-lingua-native-language`.
  - `crates/lingua-wasm` — *new*: the French baseline (scenario, corpus, golden) and the harness's
    choice of a pack source; *consumed*: the engine, unchanged (no binding is added).
  - `crates/lingua-pack` — *consumed*: the builder; one test table gains French.
  - `scripts/lingua-data/testdata/fr-en/` — *new*: the hand-written fixture pack's tables.
  - `apps/lingua-extension` — *consumed*: one entry in the analyser-constant map; no surface, type,
    label or pack list changes.
  - `apps/lingua-agent` — one label; no French pack can be installed.
  - CI — `lingua-extension-check` runs the third baseline, `lingua-pack-update` re-blesses it with
    the two others, `sonar-project.properties` excludes the French scenario from copy-paste
    detection as the English and Spanish ones are.

  ID, Music, Live, the back office, the site, the backend and the Apple host app are untouched.
- **Release.** Silent. Nothing a reader sees changes: no listed pair studies French, Réglages offers
  the listed pairs' languages only, and a `fr` reaches the engine from tests alone.
- **Compatibility.** Builds from this change on read backups of schema versions 1, 2 and 3 and
  write 3 only for a state that names French. Nothing writes such a state outside tests before
  `enable-lingua-french` (change 52). A released build that met one would refuse it as unsupported
  version 3 and leave it in place.
- **Not here.** Elision, `au`/`aux` and the hyphenated inversion (40, M21); the cascade, the
  function words, the names rule and NFC (41); the Catalan and Occitan guard (42); the tables,
  grammar and levels (43, 45, 46, M7); the fr-en and fr-es packs and the cross-native test for
  French (48, 49); the word card, the extension's type and labels, `packs.json` (51, 52); the
  back office's name for `fr` (53).
- **Effort, against 3–5 ideal days.** The variant, its seams, versions and tests: 0.5–1. Backup 3
  and its tests: 0.5–1. The corpus, the fixture pack, the scenario, the harness's pack source, the
  first bless and the CI lines: 1.5–2.5. Spec and programme: 0.5. The corpus is the uncertain
  part — thirteen pages written by hand around the phenomena the later changes must show — and the
  owner's reading of it (task 5.1) can move a page before the golden is blessed.
