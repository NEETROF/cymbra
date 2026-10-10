# match-lingua-french-contracted-pieces — a French expression's « au » meets only « au »

## Why

Row 44d of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. Change 44 (`add-lingua-french-expression-keys`, merged) keys a French pack's
expressions as French is read: French's pre-pass reads « au » as `à` + `le` and « aux » as `à` +
`les` (change 40, M21), and the key is made of those words, so `au fait` (« by the way; informed »)
is keyed `à le faire`. The key no longer says the two words were one, and neither does the match:
« Il est prêt **à le faire** », where `le` is a pronoun, answers `au fait`. Change 44c
(`match-lingua-french-elided-pieces`, merged) closed the same gap for elisions and measured this one
in its D7; the owner decided on 2026-10-10, answering its open question 3, that contractions get the
same rule in a change of their own: **a piece the headword writes as a contraction (« au »,
« aux ») meets only a contraction on the page, and pieces it writes apart (« à le ») only words
written apart.**

Measured over change 44c's 51,790 French selections (change 39's corpus, UD French-GSD, four
Gutenberg novels, French Wikipedia, Tatoeba: 1,085,056 tokens, 8,841 written « au » or « aux »), on
fr-en's 60,369 matches and fr-es's 41,556 today:
- **a contraction meets words written apart 10 times in each pair**, every one `au fait` on « à le
  faire » with `le` a pronoun (« Elle n'arrive pas à le faire », « qui commençait à le faire
  souffrir »). The exposure is wider than the corpus: 24 of fr-en's 448 headwords holding « au » or
  « aux » (7 of fr-es's 157) are keyed `à le`/`à les` before a verb's dictionary form — `au revoir`,
  `au pouvoir`, `au dire de`, `au vu de` (`à le voir de`), `tirer au sort` (`à le sortir`) —, which
  a pronoun before an infinitive answers;
- **no headword writes « à le » or « à les » apart**: the 154 fr-en keys holding `à le` without a
  contraction are all elided (`mal à l'aise`), change 44c's ground;
- **an expression's edge meets half a contraction 509 times in fr-en and 373 in fr-es**, the other
  half being the page's word: its last `à` before the page's article, 505 and 334 (`jusqu'à` on
  « jusqu'au » 180, `grâce à` on « grâce au » 62, `face à` on « face aux » 42; `être à`, 91 and 120,
  as right or wrong on « est au » as on « est à »), and its first article after the page's `à`, 4
  and 39, all but one right (`le même` on « au même titre » 21, `les deux` on « aux deux ministres »
  13, `les miennes` on « aux miennes ») — the matches change 44c's D7 found that change 51's D12
  does not cover.

The last figure is why the rule needs one reading the owner's wording does not spell out: French
contracts `à` with the article after it, and at an expression's edge one of the two words is the
page's. Applied there too, the rule would stop « jusqu'au soir » answering `jusqu'à` — the case the
owner settled for change 51's D12 — and « au même titre » `le même`.

## What Changes

- **A French run's tokens are joined into one written word where its headword's pieces are, and
  only there** (design D1): two pieces the headword writes « au » or « aux » meet only the written
  « au » or « aux », two pieces it writes apart only words written apart. At the run's edges the
  contraction is the page's: its last `à` meets the `à` of « au » (change 51's D12 then covers the
  article), its first `le`/`les` meets the article of « au » (open questions 1 and 2). Where the
  tokens do not meet, the run does not match at that length and the shorter runs are tried as before.
- **The page's contractions are read from `shares_span`** (D2), which `gloss_phrase` computes already
  and hands `match_expressions` (change 44b's D5, change 51's D12): in French only the halves of
  « au » and « aux » share a span. No token, field or JSON changes.
- **The headword's contractions are read from its name, already in the pack** (D3): a headword
  holding « au » or « aux » differs from its key, so the pack names it — fr-en's 448 and fr-es's
  157, measured. Change 44c's `meets_as_written` reads the name through French's pre-pass when it
  holds an apostrophe or the word « au » or « aux », and checks both rules on that one reading. No
  key, section, pack byte or pin moves.
- **The matches starting on the article of « au » or « aux » are kept** (D4): 4 in fr-en and 39 in
  fr-es, all but one right, each a row of the card beside `à` (a function word, which has none),
  never the whole selection's answer; refusing them, or extending them back over the `à`, is open
  question 2.
- **What moves** (D5): four phrase probes added to the French golden — « Il est prêt à le faire »,
  « Au fait, tu viens ? », « grâce au soleil », « aux miennes » —, so `fr-en.golden` and
  `fr-es.golden` gain 8 lines each and move none; in both, « Il est prêt à le faire » records no
  expression where the engine today reports `au fait`. `word-card-fr-en.txt` and
  `word-card-fr-es.txt` gain the four cards (11 lines each), their specs' count 34 → 38. French's
  analyser version, `pack_version`, every pack, pin and table, and every other probe do not move.
- **English and Spanish do not move** (D6): the check runs for French alone. Spanish's « al » and
  « del » have the same gap in principle — measured over 464,574 Spanish selections, 1 of es-fr's
  205,743 matches and 5 of es-en's 236,765, every one before a title (« a El Tiempo ») or across a
  dropped number (« de 1924, el mismo ») — and are left out, es-fr's output not moving (open
  question 3).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A French expression's contractions meet the words as its headword
  writes them* (« au »/« aux » only a written « au »/« aux », pieces written apart only words written
  apart, the run's edges the page's; read from the tokens' spans and the expression's name; English
  and Spanish unmoved).

No requirement is modified. The new one narrows, for French, how *French expressions are found on
French's reading of a selection* (change 44) and *Expression lookup in a phrase gloss*
(`add-lingua-expression-table`) find a match, beside change 44c's *A French expression's pieces meet
the words as its headword writes them* and change 51's *A French match never ends inside a written
word*; those changes are open, so it says so rather than MODIFY them. It reads contractions as
*French tokenisation pre-pass* (change 40) splits them, and the names change 44's *A French pack keys
its expressions as French is read* puts in the pack. The changes whose requirements it builds on are
in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: the contraction check in `engine::meets_as_written`, which
    `match_expressions` hands the run's span joins; *consumed*: `shares_span` in `gloss_phrase`, the
    French pre-pass, `headword_reading`, `Pack::expression_name`, change 44c's elision check.
  - `crates/lingua-pack` — a test only: every French headword holding a contraction holds the word
    « au » or « aux » and is named, on the committed tables.
  - `crates/lingua-wasm` — four phrase probes in the French scenario; `fr-en.golden` and
    `fr-es.golden` re-blessed by additions.
  - `apps/lingua-extension` — *consumed*, its code unchanged: the card shows what the phrase gloss
    reports. `word-card-fr-en.txt` and `word-card-fr-es.txt` gain the four probes' cards, their
    specs' count 34 → 38.

  ID, Music, Live, the back office, the site, the backend, the Apple host app, the agent plugin
  (which studies no French) and every reducer and table are untouched.
- **Release.** Silent: no listed pair studies French. It lands before change 52 lists fr-en, so no
  reader is shown `au fait` on « prêt à le faire ».
- **Compatibility.** No stored format, wire field, table, pin, pack or version moves. A status or
  card a reader set on an expression is untouched (no reader studies French).
- **Effort, against no programme estimate (outside the 57)**: 0.25–0.5 ideal day (design, *Effort*).
