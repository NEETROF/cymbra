# choose-lingua-overlapping-expressions — an expression built on a verb gives its last word to a longer one

## Why

Row 44f of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. The phrase gloss finds a selection's expressions from left to right: from
each token, the longest run that is a key wins, and the next run starts past it
(`add-lingua-expression-table`'s D3, change 44). When two expressions overlap, the one that starts
first always wins, however short. Change 44d (`match-lingua-french-contracted-pieces`, merged)
measured the case the owner then made a change of its own on 2026-10-10 (its open question 4): in
« Elles sont **au** nombre de sept », `être à` (« to pertain to; to belong to ») takes the `à` of
« au », and the card shows it and `nombre de` (« quite a few ») where `au nombre de` (« numbering;
among ») fits.

The first proposal measured nine rules (design D5) and found that what separates the right reading
from the wrong one is what the first expression is: an expression built on a verb (`être à`, `tenir
à`, `j'ai`) ends on a word the words after it often make part of a longer expression, while a
compound preposition (`jusqu'à`, `grâce à`) owns its last word — « jusqu'au début » means *until the
beginning*. The owner settled its questions on 2026-10-10: **the rule for French, Spanish and English
in this change**, es-fr's and en-fr's output moving approved; **widened to any last word now** (`j'ai`
gives « ai » to `avoir besoin de`), with **a guard holding a French headword's hyphen to the page
where its expression would take a word**, so that « c'est ce que » never answers `est-ce que`;
compound prepositions keep their « à »; es-fr's and en-fr's output moving confirmed at the final
figures. The same
mistake is there in the other two languages: « Lo hizo de todos modos » shows `hacer de` (« to pretend
to ») where `de todos modos` (« anyway ») fits, « went over the top » `go over` (« to examine ») where
`over the top` fits.

## What Changes

- **An expression built on a verb gives its last word to the expression that starts on it, when that
  one is longer, or (French, Spanish) when the word is the first half of a written « au », « aux »,
  « al » or « del »** (design D1), in the three languages: « Elles sont au nombre de sept » answers `au
  nombre de`, « J'ai besoin de toi » `avoir besoin de`, « Lo hizo de todos modos » `de todos modos`,
  « went over the top » `over the top`. The giving expression is tried shorter, as a run that does not
  meet its headword is (changes 44c, 44d); « jusqu'au début » keeps `jusqu'à`, « Il venait de
  l'étranger » `venir de`.
- **Built on a verb** is read from the pack's grammar on the studied side: a token, the last included,
  that it reads as a verb or an auxiliary, with no preposition before it (D2). **A preposition** is one
  of the studied language's closed classes, exposed as `is_preposition`.
- **Three guards keep today's reading** (D3): the expression that would take the word must meet its
  French headword's hyphens (`est-ce que` takes no « est » from « c'est ce que », 23 selections, all
  better; the hyphen is read nowhere else, so nothing else moves); a preposition the page follows
  with « que » stays (« Pese a que no tengo dinero » keeps `pese a`, not `a que no`, 8 selections, all
  better); and an expression that would cut the next one does not take the word (« iba al día
  siguiente » keeps `ir a` and `día siguiente`, 10 selections, 7 better, 2 worse). Five more were
  measured and left out — among them the hyphen held for every French match (53 selections more,
  38 better, 15 worse) — and no English guard was found.
- **Measured** (D1): fr-en 275 selections move, fr-es 86, es-fr 228, es-en 389, en-fr 71, en-es 124
  (of 51,790 French, 464,574 Spanish and 68,484 English selections). Judged: French in full — fr-en 251
  better, 7 worse, 17 neither; fr-es 85, 0, 1 —; sampled 30 per language — French 30, 0, 0; Spanish
  22, 2, 6; English 22, 2, 6 ([review.md](review.md)). Gloss time +0.9 % to +4.3 % (en-fr within the noise).
- **What moves** (D6): probes added, none moved — `fr-en.golden` and `fr-es.golden` 14 lines each
  (seven French probes), `es-fr.golden` and `es-en.golden` 10 each (five Spanish), `en-fr.golden` and
  `en-es.golden` 6 each (three English); `word-card-fr-en.txt` and `word-card-fr-es.txt` 22 and 21 lines,
  their specs' count 42 → 49; the Spanish and English card snapshots do not move. es-fr's and en-fr's
  shipped output moves on the selections above, approved and confirmed by the owner (D7).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *An expression built on a verb gives its last word to a longer
  expression* (which of two overlapping matches the phrase gloss keeps, in the three languages, with
  its three guards).

No requirement is modified. The new one narrows the left-to-right choice that *Expression lookup in a
phrase gloss* (`add-lingua-expression-table`), *French expressions are found on French's reading of a
selection* (change 44) and *Spanish expressions are found on Spanish's reading of a selection*
(change 44b) make, beside *A French match never ends inside a written word* (change 51) and changes
44c's and 44d's requirements on how a French piece is written; those changes are open, so it says
so rather than MODIFY them. It reads verbs as *French's word grammar* (change 45) and *A word's
grammar, from the pack* store them, a lemma as *A dictionary form is read as itself* (change 41b)
finds it, prepositions as the closed classes list them (*French closed classes*, change 41), and a
hyphen as *A soft hyphen is not part of a word* (change 42d) reads it. The changes whose requirements
it builds on are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: the choice in `engine::match_expressions` and its guards, the
    hyphen check G1 asks for, `gloss_phrase`'s hyphen joins, `is_preposition`
    (`analysis/function_words.rs`) and a public hyphen-join test (`analysis/tokenize.rs`); *consumed*:
    `Pack::readings`, the closed classes, changes 44c's and 44d's checks, change 51's cover of the
    article.
  - `crates/lingua-wasm` — probes in the French, Spanish and English scenarios; the six goldens
    re-blessed by additions.
  - `apps/lingua-extension` — *consumed*, its code unchanged: the card shows what the phrase gloss
    reports. `word-card-fr-en.txt` and `word-card-fr-es.txt` gain the French probes' cards.
  - `apps/lingua-agent` and `apps/lingua-apple` — *consumed*: they run the same core; where they
    gloss a selection, its expressions are chosen the same way.

  ID, Music, Live, the back office, the site and the backend, every reducer, table and pack are
  untouched.
- **Release.** French is silent (no listed pair studies it); es-fr, es-en, en-fr and en-es readers see
  the new choice with the release that carries this core (D7).
- **Compatibility.** No stored format, wire field, table, pin, pack or version moves. A status or card
  a reader set on an expression is kept; where another expression now shows, its own status shows.
- **Effort, against no programme estimate (outside the 57)**: 1–2 ideal days (design, *Effort*).
