# refine-lingua-spanish-galician-markers — Spanish's *gives* is not Galician

## Why

Change 42c of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside the 57. `add-lingua-spanish-occitan-guard` (change 42b) measured a defect of Spanish's guard
that is older than it, and left it to a change of its own (its open question 4, settled so by the
owner on 2026-10-10). The guard (`add-lingua-spanish-detection-guard`) counts `da` and `das` as
Galician words — *de* + *a*, « of the » — but Spanish writes them too: *da*, « gives », and *das*,
« you give ». A short Spanish line with one of them and none of the guard's Spanish words is refused
as Galician: « ¿Cuánto se da de propina en España? », « No me da tiempo. », « ¿Te das cuenta de la
hora que es? ».

Measured again on `main` (Occitan's comparison included), the guard refuses 852 of the 453,821
Spanish sentences and paragraphs the detector reads as Spanish, and 796 of them for these two words
alone. A reader of Spanish sees such a line left out: not underlined, not counted, its words never
offered for the deck. It is a dialogue line, a caption, a heading — « Da un paso atrás. », « ¿Me da
una factura? » — and in a Spanish e-book the short lines are the dialogue.

## What Changes

- **`da` and `das` count for Galician only beside another Galician word** (D1). A block whichlang
  reads as Spanish keeps its three comparisons; in Galician's, `da` and `das` are added to the
  count only when the block holds another of Galician's 30 function words. Alone, they count for no
  one: « ¿Cuánto se da de propina en España? » has no Galician word left and stays Spanish. Beside
  one, they count as today: « O profesor Smith é recoñecido por ser un dos máis grandes eruditos da
  filoloxía inglesa. » (`máis` and `da` against `por`) stays refused. Both words stay in Galician's
  table; Catalan's and Occitan's comparisons, and the way the guard reads words, do not change. The
  guard only keeps more: every block it keeps today stays kept.
- **Measured** on change 42b's 597,086 blocks (design, *Measurement*):
  - Spanish: 852 → 56 refused (0.19 → 0.012 % of the blocks read as Spanish) — all 796 refused
    for `da` or `das` given back: 788 Tatoeba sentences, one line of UD AnCora (« Da la impresión de
    que desean que ETA actúe". »), six of Spanish Wikipedia (« …que da paso a la democracia en
    España. »), one of Ricardo Palma's *Tradiciones peruanas*; what stays refused is `polo` (« Polo
    Norte »), `do` (« el do de pecho »), « per se », « ON », and Catalan and Occitan quotations;
  - Galician, the price: 21.2 → 22.2 % of the blocks read as Spanish (16.0 → 17.7 % of the text),
    91 more blocks — every one a Galician line whose only Galician words are `da` or `das` (« A
    esperanza é a razón da vida. », « Todos os alumnos da clase están presentes. »);
  - Portuguese 98 → 100 of 11,070 blocks, a Latin and a Romansh line; Catalan, Occitan, Aranese,
    Asturian and Aragonese unchanged, block for block.
- **The other options, measured** (D1): taking `da` out of the table gives back 700 (152 still
  refused) and lets 76 Galician blocks through; taking both out gives back the same 796 but lets
  110 through; dropping them only where the block holds a Spanish word, or weighing them one half,
  gives back 5 — 791 of the 796 hold no Spanish word at all, and half a word still beats none;
  ignoring them right after a clitic (`me`, `te`, `se`, `le`…) gives back 549 at no Galician cost,
  but not « Da un paso atrás. » nor « El padre de Bob da clase… ».
- **What it costs** (D3): nothing measurable — one counter more, the same lookups; the guard's
  27.3–30.6 µs per KB of Spanish text become 27.9–31.0, within the machine's noise.
- **Spanish's analyser version is bumped** (D4): `1.3.0` → `1.4.0`. English and French keep theirs.
- **What moves** (D5), applied to a scratch checkout of `main` (`eddaf712`): the rule alone moves no
  probe of any golden (the Spanish corpus writes neither word); the bump moves the version, and
  nothing else, on 18 lines of `es-fr.golden` and of `es-en.golden` and on the `beside es-en` line
  of `fr-en.golden`; `tables/es-fr/` and `tables/es-en/` are re-reduced, their manifests and pins
  alone moving (sizes unchanged); the Spanish fixtures, their recorded digests, the agent's
  `es-fr.lingua` fixture and `tests/languages.rs`'s literal follow. en-fr and en-es do not move,
  without re-blessing. es-fr's output moving needs the owner's approval (the programme's rule), in
  this change's pull request.
- **One requirement amended** (D7): *Catalan and Galician are not read as Spanish* — as change 42b
  leaves it — says the core refuses a block whose Galician function words outnumber its Spanish
  ones, which « ¿Cuánto se da de propina en España? » would contradict once kept. It gains the rule
  for `da` and `das`, and « adding it, or changing what it counts, SHALL bump Spanish's analyser
  version ».

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: MODIFIED — *Catalan and Galician are not read as Spanish*: `da` and `das`
  count for Galician only beside another Galician function word; « a tie, or a block with no such
  function word counted, SHALL stay Spanish »; « adding it, or changing what it counts, SHALL bump
  Spanish's analyser version and no other »; a Galician line whose only Galician words are `da` and
  `das` is the leak it states. Its five scenarios are kept word for word, by name, and four are
  added. Change 42b, still open, MODIFIES the same requirement: this change is written on top of 42b's
  text and archives after it (`archiveAfter: add-lingua-spanish-occitan-guard`). *Occitan is not
  read as Spanish* is 42b's and is not touched.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core` — *new*: the two words Galician shares with Spanish and the rule in
    `iberian_neighbour` (`analysis/language.rs`), Spanish's analyser version; *consumed*: whichlang,
    change 42's `detect`, unchanged.
  - `crates/lingua-wasm` — the three goldens re-blessed on the version; `tests/languages.rs`'s literal.
  - `scripts/lingua-data/tables/es-fr/`, `tables/es-en/` — re-reduced: `manifest.json` and `pin.json`;
    `testdata/es-fr/`, `testdata/es-en/` — `manifest.json`; `crates/lingua-pack/tests/pipeline_testdata.rs`
    — the two Spanish fixtures' digests.
  - `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` — rebuilt (a test input; the agent is
    otherwise outside the programme, M17).
  - `apps/lingua-extension` — nothing: it reads Spanish's version from the core and builds its es-fr
    pack from the committed tables.

  ID, Music, Live, the back office, the site, the backend and the Apple host app are untouched. No
  table row, wire field, proto or stored format changes.
- **Release.** es-fr ships: the next extension release carries it, and a reader of Spanish stops
  seeing short Spanish lines with *da* or *das* left out. Nothing else a reader sees changes;
  nothing is migrated.
- **Order.** After change 42b (merged, #856; open until its 6.3), whose requirement it modifies.
  Independent in code of every other open change; in files, any change that moves Spanish's version
  or the Spanish pins — the hidden hyphens' (42b's open question 5), should it be proposed, or an
  es-fr update — meets a conflict on the same golden lines and pins, and whichever merges second
  re-reduces and re-blesses once on top of the other.
- **Not here.** The guard's other refusals of Spanish (`polo`, `do`, `non`, « per se », 56 blocks);
  the e-books' hidden hyphens, which split « vida » into `vi` and `da` (42b's open question 5, a
  change of its own) — this rule makes them refuse less, not never; Galician's own words the table
  lacks, such as `é` (« is »), measured and offered (open question 2); Asturian and Aragonese.
- **Effort, against 0.75–1.5 ideal days.** The rule and its doc comments: 0.1–0.25. Unit tests:
  0.25–0.5. The bump, the re-reductions, the fixtures and the goldens: 0.25–0.5. Spec, programme:
  0.1–0.25.
