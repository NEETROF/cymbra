# match-lingua-french-elided-pieces — a French expression's elided pieces meet elided words

## Why

Row 44c of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. Change 44 (`add-lingua-french-expression-keys`, merged) keys a French pack's
expressions as French is read: French's pre-pass reads an elided piece as the word it stands for
(`l'` → `le`, `d'` → `de`, `qu'` → `que`), and the key is made of those words. The key no longer
says how a piece was written, and neither does the match: an elided piece and the same word written
in full meet each other.

The owner's case, on fr-en's committed table: the expression `de l'` (« some; the singular
prevocalic partitive… ») is keyed `de le`, so « Il a décidé **de le** faire », where `le` is a
pronoun, answers `de l'` and the card shows a wrong meaning. The owner decided on 2026-10-10, in a
change of its own: **an expression piece written with an elision (`l'`, `d'`, `qu'`, `j'`…) meets
only an elided word on the page, and a piece written in full only a word written in full**, so
« de l'eau » keeps its meaning and « de le faire » is no longer touched.

Measured over 51,790 French selections — change 39's corpus, UD French-GSD (the files fr-en's pin
records), four Gutenberg novels, French Wikipedia and Tatoeba, 1,085,056 tokens of which 57,643 are
elided pieces —, of fr-en's 62,803 matches today:
- **an elided piece meets a word written in full 492 times**: `de l'` on « de le » 165 times (« offert
  de le payer »), `c'est` on « ce sont », « ce fut », « ce soit », « ce sera » 282 times, and 45
  more, 34 of them across a number the tokeniser drops (« de 1278 entre » answers `d'entre`) and 5
  across an inversion (« Pourrais-je avoir » answers `j'ai`);
- **a piece written in full meets an elided word 4,047 times**: 1,526 inside the expression, of which
  1,513 are `de un` (« first, first up ») on every « d'un » (« ne se souvenait d’un hiver » in change
  39's own corpus); and 2,521 on the expression's last piece, all right: `parce que` on « parce
  qu'il », `pas de` on « pas d'argent », `bien que` on « Bien qu'il pût venir ».

The last figure is why the rule needs one reading the owner's wording does not spell out: French
elides a word before the word that follows it, and after an expression's last piece that word is the
page's, outside the expression. Applied there too, the rule would stop « parce qu'il pleut »
answering `parce que` — 2,521 right matches in fr-en, 1,996 in fr-es, and the « Qu’est-ce qu’il
attend » and « Bien qu'il pût venir » change 44 kept on purpose (its D1).

## What Changes

- **A French expression's pieces meet the page's words as the headword writes them** (design D1): a
  piece the headword writes elided meets only an elided word, a piece it writes in full only a word
  written in full — except its last piece written in full, which meets either, the next word on the
  page deciding (open question 1). Where a run's pieces do not meet, the run does not match at that
  length and the shorter runs are tried as before: « d'un peu plus » answers `un peu` instead of
  `de un`.
- **The page's elision is read from the token's span** (D2): the pre-pass gives an elided piece a
  span holding its apostrophe, and no other token's span ends on one (57,643 of 57,643 on the
  corpus). The phrase gloss computes it beside `shares_span`; no token, field or JSON changes.
- **The headword's elision is read from its name, already in the pack** (D3): every French headword
  holding an elided piece differs from its key, so the pack names it (`expr.names.zst`, change 44):
  fr-en's 1,541 and fr-es's 869, measured. The name is read by French's pre-pass, the function the
  key was built from, only when it holds an apostrophe. No key, section, pack byte or pin moves.
- **The cost, measured and accepted** (D4): `c'est` no longer answers « ce sont », « ce fut »,
  « ce sera » (282 in fr-en; « c'était » still does), nor `ça a été` « ç'a été » (5), and the
  colloquial « d'la », « coup d'pied », « c'qui » lose their expression (open question 2).
- **What moves** (D5): four phrase probes added to the French golden — « de l’eau », « Il a décidé de
  le faire », « parce qu’il pleut », « d’un hiver » —, so `fr-en.golden` and `fr-es.golden` gain 8
  lines each and move none; in fr-en, two of them record no expression where the engine today
  reports `de l'` and `de un`. The two word-card snapshots gain the four cards. French's analyser
  version, `pack_version`, every pack, pin and table, and every other probe do not move.
- **English and Spanish do not move** (D6): the check runs for French alone, and Spanish has no
  elision — 0 elided tokens in 11,768 Spanish selections.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A French expression's pieces meet the words as its headword writes
  them* (elided or in full, the last piece in full meeting either, read from the token's span and
  the expression's name; English and Spanish unmoved).

No requirement is modified. The new one narrows, for French, how *French expressions are found on
French's reading of a selection* (change 44) and *Expression lookup in a phrase gloss*
(`add-lingua-expression-table`) find a match, as change 51's *A French match never ends inside a
written word* did beside them; those changes are open, so it says so rather than MODIFY them. It
reads elided pieces as *French tokenisation pre-pass* (change 40) defines their spans, and the names
change 44's *A French pack keys its expressions as French is read* puts in the pack. The changes
whose requirements it builds on are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: whether a token is an elided piece (`analysis/tokenize.rs`, read
    from its span), the elision check in `engine::match_expressions` for a French run; *consumed*:
    the French pre-pass, `headword_reading`, `Pack::expression_name`, `shares_span`'s place in
    `gloss_phrase`.
  - `crates/lingua-pack` — a test only: every French headword holding an elided piece is named, on
    the committed tables.
  - `crates/lingua-wasm` — four phrase probes in the French scenario; `fr-en.golden` and
    `fr-es.golden` re-blessed by additions.
  - `apps/lingua-extension` — *consumed*, its code unchanged: the card shows what the phrase gloss
    reports. `word-card-fr-en.txt` and `word-card-fr-es.txt` gain the four probes' cards, their
    specs' count 30 → 34.

  ID, Music, Live, the back office, the site, the backend, the Apple host app, the agent plugin
  (which studies no French) and every reducer and table are untouched.
- **Release.** Silent: no listed pair studies French. It lands before change 52 lists fr-en, so no
  reader is shown `de l'` on « de le faire ».
- **Compatibility.** No stored format, wire field, table, pin, pack or version moves. A status or
  card a reader set on an expression is untouched (no reader studies French).
- **Effort, against no programme estimate (outside the 57)**: 0.5–1 ideal day (design, *Effort*).
