# seed-lingua-decks-with-glossed-lemmas — a card seeded from a level always carries its gloss

## Why

« Renforcer un niveau », the control of the stats panel that feeds the review deck from a level
(preselected to the reader's declared level; a count up to 50; commonest or rarest first), takes the
level's lemmas as the pack lists them, glossed or not: `Pack::lemmas_at_level` returns every levelled
lemma with its gloss *when the pack carries one*, and `Deck::seed_lemmas` makes a card of it either
way. A card with no gloss has nothing behind « Afficher la réponse »: review reveals an empty answer.

The owner decided on 2026-10-10 (in session, the open question 3 of change 49b,
`refine-lingua-fr-es-glosses`): **seeding a review deck by level skips a lemma the reader's pack does
not gloss**, so that no card arrives without a translation — a core change of its own, for every
pair, required before change 52 (`enable-lingua-french`) ships French. Only the seeding can tell: a
French level is French's, shared by fr-en and fr-es (M24: the studied side is kept once). Change 48b
(`refine-lingua-fr-en-glosses`) takes the level away from a lemma fr-en does not gloss, which keeps
French's levels to French's dictionary words; fr-es cannot do the same without making French's
levels depend on a pair that is not French's reference.

It is not French's alone. Measured on `main` (design *Measured*), of the 8,302 levelled lemmas of
each studied language:

| Pair | Levelled lemmas the pack does not gloss | at A1 | In a seeding of 20, commonest first / rarest first, cards without a gloss today |
|---|---|---|---|
| en-fr (shipped) | 573 (6.9 %): `an`, `mr`, `take-off`, `organisation`, `unscathing` | 12 | up to 4 / up to **20** (C1) |
| en-es | 538 (6.5 %) | 6 | up to 3 / up to 18 |
| es-fr (shipped) | 0 | 0 | 0 / 0 |
| es-en | 51 (0.6 %): letters and names, `d`, `psoe`, `lincoln` | 8 | 0 / 0 |
| fr-en | 40 (0.5 %): `parce`, `quant`; **0** once 48b lands | 5 | 0 / 0 |
| fr-es | **1,188** (14.3 %): `parce`, `part`, `lors`, `afin`, `taux`; 1,152 once 48b lands | 22 | up to 5 / up to 10 |

A French reader of English who asks today for 20 words of C1, rarest first, gets 20 cards with no
answer (`unscathing`, `unmusically`, `unenviably`, …): `en-fr.golden` pins three of them. The figure
the owner was given for fr-es, 1,198, is change 49b's after its own rules; on `main` it is 1,188 (22
at A1), after 48b 1,152 (18 at A1, `parce` no longer levelled).

## What Changes

- **Seeding skips a lemma the pack does not gloss** (design D1): `Deck::seed_lemmas` passes over a
  candidate without a gloss as it passes over one already carded or holding a status, and writes
  nothing for it; `Card::seeded` takes the gloss itself rather than an option, so a seeded card
  without one cannot be built. The engine's `seedLevel` keeps its shape; the extension does not
  change.
- **A skipped lemma's place is taken by the next one** (D2): a skipped lemma takes no place under the
  cap — the core already does so for a tracked lemma, which the spec's scenario said otherwise —,
  so a seeding asked for N adds N while the level holds N glossed untracked lemmas (every level of
  every pair holds at least 649, the cap is 50). A shorter deck was measured and rejected: the
  skipped lemmas get no card, stay first in the order, and the level stops giving cards — en-fr's C1
  rarest first gives 0 from the first seeding on, fr-es's B1 commonest first stops after 150 of its
  1,774 glossed lemmas.
- **What the reader knows does not move** (D3): a skipped lemma keeps its level, its place in the
  ladder and in the vocabulary estimate, its presumption below a declared level, and gets no status.
  The estimate's rule, the ladder, the declared level and its sync are untouched.
- **The seeded deck becomes the native side** (D4): the invariance tests compared every probe of a
  language's baseline through two packs of that language; the cards a level seeds now follow each
  pack's glosses — en-fr and en-es leave out 537 levelled lemmas the other seeds, fr-en and fr-es
  1,158. `cross_native.rs` answers the baselines without the reader's level seeding and checks the
  seeding apart: through each pack, every level, both orders, the cards are that level's lemmas the
  pack glosses, in that order. The golden-to-golden comparisons of changes 23 and 24 (and 51's when
  it lands) stop at the seeding.
- **What moves** (D5): `en-fr.golden` and `en-es.golden`, 9 probes each — the reader's C1 rarest-first
  seeding draws glossed lemmas (`digitalize`, `exhilarate`, `impishly`; `eclectically`, `irately`,
  `tastebud`) and the review that follows marks another card known. `es-fr.golden`, `es-en.golden`
  and `fr-en.golden` do not move. No pack byte, pin, analyser version or `pack_version`; no
  snapshot; no stored format (D6).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-decks-review`: MODIFIED *Level-targeted deck seeding* (no open change holds it): a seeded
  card carries the gloss of the pack of its studied language; a lemma that pack does not gloss is
  skipped and the next lemma takes its place; a skipped lemma keeps its level, ladder place and
  estimate and gets no status. Its scenario *Seed a level into the deck* said « at most 15 new
  cards » for 20 asked with 5 tracked, while the core has always taken the next untracked lemma
  (`seed_lemmas_caps_and_skips_tracked_lemmas`); it now says what the core does, since D2 rests on it.
- `lingua-analysis`: MODIFIED *An analysis does not depend on the native language*, as change 49
  (`add-lingua-pack-fr-es`, merged, not archived) leaves it — hence `archiveAfter`: the cards a
  level seeds are the one part of a reader's history that follows the native language; the test
  compares the baselines without the reader's level seeding and checks the seeding apart.
- `lingua-browser-extension`: *Level-targeted deck feeding control* holds as written (« up to 20 B2
  words are added to the deck and the control reports the number added »).

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-core` (`decks/review.rs`, `decks/card.rs`),
  `crates/lingua-wasm` (`seedLevel`'s comment; `tests/support/mod.rs`, `cross_native.rs`,
  `en_es_baseline.rs`, `es_en_baseline.rs`, `baseline/en-fr.golden`, `baseline/en-es.golden`). The
  extension (Chromium, Firefox, Safari's host app) consumes the rule through its WASM build, with no
  source change; the agent plugin seeds no level (`add_words` adds the words it is given: outside
  this change, M17). ID, Music, Live, the back office and the site are untouched; no server, sync or
  backup format moves.
- **What does not move**: the six committed packs (sha256 = pin), every analyser version and
  `pack_version`, `es-fr.golden`, `es-en.golden`, `fr-en.golden` (on `main` and on 48b's tables),
  `word-card-en-es.txt` and `word-card-es-en.txt` (they read the `word-grammar` probes only), every
  ladder, fresh reader's estimate, `declared-level` and `export-declared-levels` probe.
- **Order.** Before change 52 lists fr-en or fr-es; it joins 52's `archiveAfter`, as 48b and 49b do
  (52's design, *At or above*), which 52 adds when it next moves. Independent of 48b and 49b: no
  table moves here, and their tables move none of this change's probes (fr-en.golden draws only
  glossed lemmas on both). With change 51 (`add-lingua-french-word-card`) in either order: whichever
  lands second has `fr_es_baseline`'s comparison stop at the seeding, and if 51 lands first its
  `fr-es.golden` moves on 6 probes (C1's `cheminement`, which fr-es does not gloss, gives way to
  `carreau` « Baldosa »). Row 49c, outside the 57, like rows 41b and 48b.
- **Owner.** The re-bless of `en-fr.golden` (the programme's rule « en-fr and es-fr output does not
  move ») and of `en-es.golden`, 9 probes each, approved before the implementation merges; two open
  questions (design *Open questions*): the message when only words without a translation are left,
  and the cards without a gloss already in readers' decks. Then the extension's release, before or
  with French.
