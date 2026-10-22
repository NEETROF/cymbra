# add-lingua-italian-word-card — an Italian word's readings, named in French, English and Spanish

## Why

Change 18 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. The word
card names a form's readings through a language-neutral description and one renderer per interface
language (`generalise-lingua-card-wording`), with tense names keyed by the studied language
(`TENSES`, `GERUNDS` in `src/i18n/{fr,en,es}/grammar.ts`); French's card (matrix change 51) added
French's row to each renderer and merged the moods (M21). Change 13 gives Italian's forms their
readings — the passato remoto as `Tense=Past`, the imperfetto as `Tense=Imp`, the conditional and the
imperative without a tense, the gerund `VerbForm=Ger`, the participles —, stored unmerged. The card
cannot say any of it: `StudiedLanguageCode` is `en | es | fr`, and no renderer has an Italian row.

Italian is read in French first (I5: it-fr), then in English and Spanish (it-en, it-es, stage 2): unlike
French studied, its card needs the **French** renderer too — the one renderer no studied language
has used for a card yet (French's own row serves only the override M2 reserves).

Measured on change 13's prototype readings over the Italian section's tables (441,628 form–lemma
pairs, design *How it was measured*):
- **the indicative and the subjunctive of one tense, person and number** — the first person plural
  of the present (`diamo`, `parliamo`: 10,580 pairs, the imperative's too), the second person
  singular (`parli`: 8,449), the second person plural (`date`: 1,550) — which M21's rule says once;
- **the gerund** (`parlando`, 13,093 pairs): named by no renderer for a Romance language but
  Spanish (`gerundio`);
- **the present participle** (`parlante`): French's change 51 named it.

And two hand-overs:
- **An expression named without a space** (change 12's D8 covered a name with a space): `d'accordo`
  is an expression keyed `di accordo`; review reads a lemma without a space as an expression only for
  French (`pack_gloss`, `crates/lingua-wasm`), so an Italian `d'accordo` card shows its own text.
- **The Italian card's golden**: change 16 puts the Italian baseline on the committed tables; the
  card's lines over its probes are this change's, as French's `word-card-fr-en.txt` was change 51's.

What needs nothing: a selection inside a word written as pieces (`l'uomo`, a drag over « uomo »)
opens the piece selected by change 51's language-neutral rule (`selectWithinWord`); a plural spelled
like its dictionary form stays unnamed by `refine-lingua-card-invariable-plurals`' rule, the same for
every studied language (`città`, `re`); the Spanish card names two genders of one number once (51's
D6), whatever the studied language.

## What Changes

- **Italian is a studied language the card names** (design D1): `StudiedLanguageCode` gains `it`;
  the extension's `StudiedLanguage` stays as it is until change 20, so no Italian reaches a card
  before then — the renderers are driven by tests and the it-fr snapshot.
- **Italian's forms, named by each renderer** (D2):
  - in **French**, French school terms for Italian grammar — « présent de l’indicatif », « imparfait
    de l’indicatif », « passé simple », « futur », « conditionnel », « présent du subjonctif »,
    « imparfait du subjonctif », « impératif », « gérondif », « participe présent », « participe
    passé »;
  - in **English**, the English Wiktionary's Italian form-of wording — "present indicative",
    "imperfect indicative", "past historic (passato remoto)" (M10's form, as French's "past historic
    (passé simple)"), "future", "conditional", "present subjunctive", "imperfect subjunctive",
    "imperative", "gerund", "present participle", "past participle";
  - in **Spanish**, the RAE's terms (M10) — « presente de indicativo », « pretérito imperfecto de
    indicativo », « pretérito perfecto simple de indicativo », « futuro simple de indicativo »,
    « condicional simple », « presente de subjuntivo », « pretérito imperfecto de subjuntivo »,
    « imperativo », « gerundio », « participio presente », « participio pasado ».
  Tenses come in the grammars' order — indicative, conditional, subjunctive, imperative — locked by
  the snapshot.
- **The indicative and the subjunctive said once (M21, for Italian)** (D3): « 1re personne du pluriel
  du présent de l’indicatif ou du subjonctif et de l’impératif de parlare », « 2e personne du
  singulier du présent de l’indicatif ou du subjonctif, 1re et 3e personnes du singulier du présent
  du subjonctif de parlare » — French's `Ind|Sub` keys, which Italian reaches in the present alone.
- **The gerund and the present participle named** (D4).
- **An expression named without a space** (D5): review reads an Italian card whose lemma holds no
  space but reads as two tokens or more (`d'accordo` → `di accordo`) at that key, as French's does;
  `perché` and every one-token lemma read as words.
- **The Italian card pinned on the real packs** (D6): the Italian baseline gains the card's probes —
  one form per name the card says, lemmas with two sense runs or more — and
  `test/word-card-it-fr.spec.ts` renders every grammar and phrase probe of `it-fr.golden` into
  `test/baseline/word-card-it-fr.txt`, run and re-blessed where the others are; it-en's and it-es's
  snapshots are stage 2's, their renderers held by unit tests meanwhile.
- **Nothing else moves** (D7): every form of the six other packs renders byte for byte; the other
  card snapshots pass as committed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *An Italian card names its forms in the interface language's
  grammar* and *The Italian card is pinned on the real packs*.
- `lingua-decks-review`: ADDED — *Review finds an Italian expression card named without a space*.
  For an Italian card, it takes the place of the sentence of *Review finds a French expression card
  named without a space* that reads every other card as before (51, open), as change 12's review
  requirement did; this change archives after 51, after the card wording change, and after the
  Italian changes 6, 12, 13 and 16 whose pieces, keys, readings and golden it reads.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `apps/lingua-extension/src/i18n/index.ts` (`StudiedLanguageCode`), `src/i18n/{fr,en,es}/grammar.ts`
  (`TENSES.it`, `GERUNDS.it`, the participles), `src/reading/grammar-description.ts` where a key is
  per language; `test/word-card-it-fr.spec.ts`, `test/baseline/word-card-it-fr.txt`, the renderers'
  unit tests.
- `crates/lingua-wasm/src/lib.rs` (`pack_gloss`'s Italian no-space branch), its test;
  `crates/lingua-wasm/tests/support/italian.rs` (the card's probes) and `baseline/it-fr.golden`.
- Untouched: lingua-core, the packs, the six other goldens and snapshots, the server, the site.
- **Effort, against 3.5–6 ideal days**: 3–4.5 (design, *Effort*).
