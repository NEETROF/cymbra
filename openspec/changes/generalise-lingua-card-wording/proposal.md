# generalise-lingua-card-wording — the word card describes a form once, and says it in the reader's language

## Why

Change 18 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release. The programme's architecture for the word card: a
language-neutral description of readings, then one renderer per native language; only tense
names are keyed by pair, and goldens lock the order of tenses.

Today the engine already answers in neutral terms — Universal Dependencies tags, `{"pos":
"VERB", "features": {"Mood": "Sub", "Person": "1", "Tense": "Pres"}}` — and one TypeScript
module, `grammar-labels.ts`, turns them into French prose: « 1re et 3e personnes du singulier
du présent du subjonctif et 3e personne du singulier de l'impératif de hablar ». Its tables are
keyed by studied language, its articles and elisions are French (`le`, `l'`, `d'`), its persons
and numbers are French words, and its tense order is the French school's. An English-native
reader of Spanish would read French.

This change separates what the card knows from how it says it: a description of the readings
that names no language, and a renderer per interface language — French first, byte for byte
the current wording, pinned by the same 108 assertions; then English and Spanish, drafted here
(M9, M10) and shown to no one until the interface speaks their language (change 20 and
stage 2). What depends on the studied language — tense names first: the same `Tense=Past` is « prétérit »
for en-fr and "preterite" for es-en — is keyed by it in each renderer.

The studied words of a line also get their `lang`: the card sets none today, so a screen reader
reads a Spanish word with the French voice (the hosts' `lang` is change 14's).

## What Changes

- **A neutral description.** `src/reading/grammar-description.ts` turns a `WordGrammar` into a
  `FormDescription`: the readings grouped as the card groups them today (persons merged and
  duplicates dropped by tag, the dictionary form left out, the other lemmas, the pieces), with
  no word of any language — parts of speech, genders, numbers, persons, moods and tenses as the
  engine's tags, degrees, and the lemmas.
- **One renderer per interface language**: `src/reading/grammar-labels.ts` stays the French one,
  its exported API and wording unchanged, reading its tables from the catalogue;
  `src/i18n/{en,es}/grammar.ts` are the English and Spanish ones behind the same interface — the
  parts of speech, genders, numbers, persons and degrees in that language, its articles and
  elisions where it has them, its joining (« a, b et c », "a, b and c", « a, b y c »), its sense
  headings (« nom féminin », "feminine noun", « sustantivo femenino »). A renderer names what the
  French one names, no more.
- **What depends on the studied language is keyed by it in each renderer**: tense names (en-fr
  « prétérit », es-fr « passé simple », es-en "preterite", en-es « pasado simple »), the gerund's
  name, whether the infinitive is named, which moods are named, the order of tenses — the French
  values today's.
- **The rule for a Spanish card** — « as French schools do » — becomes « as the interface
  language's grammar does »: French school terms in French, RAE/ASALE terms in Spanish, the
  English Wiktionary's form-of wording in English.
- **The studied-language words say their language**: `lang` on each word of a line and on the
  headword (the hosts' `lang` is change 14's).
- **Goldens per renderer**: the French spec unchanged; an English and a Spanish spec over the
  same inputs, each pinning the order of tenses and the named tags.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The word card describes a form once, and says it in the
  interface language*; MODIFIED *A Spanish card names its forms as French schools do* — a rule
  about French grammar that the umbrella rule does not cover: it now names the interface
  language's grammar, every scenario kept. *The word card says what the form is* (held by no open
  change) stands under the umbrella rule.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension/src/reading/grammar-labels.ts` (becomes
  the description and the French renderer's host), `src/reading/grammar-description.ts`,
  `src/i18n/{fr,en,es}/grammar.ts`, `src/reading/wordpopup.ts` (the `lang` attributes and the
  renderer chosen by the interface language), `test/word-grammar*.spec.ts`. The engine, the packs
  and the goldens of `lingua-wasm` are untouched. ID, Music, Live, the back office and the site
  are untouched.
- **No byte moves.** The 108 French assertions pass unchanged; the WASM goldens pin the engine's
  JSON, which does not change; no reader sees English or Spanish wording before the interface
  speaks it.
- **Order.** After change 13 (the catalogue's shape and the interface-language key) and change 14
  (the session hands the card the interface language); `refine-lingua-review-session` (#696)
  merges before, as the programme says, or this change's files do not overlap it (it touches
  review, Réglages and the popup, not the card).
- **Not here.** The card's other copy — « Mot à mot », « ▶ Mot », the rarity bands, the actions —
  is change 14's; the names of languages are change 19's; the lemma-alternatives question (M8)
  is `add-lingua-lemma-alternatives`, optional.
