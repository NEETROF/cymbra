# Design

## Context

See proposal.md — Why. What constrains the shape:

- The word card reads a form's readings from the pack's `paradigms.zst` (`Pack::readings`), tags in
  the pack's pool; `word_grammar` answers them; the extension's card names them in the interface
  language (`describeReadings`, `nameReadings`; Italian's names are change 18's). Spanish's
  (`add-lingua-spanish-grammar-tables`, archived) and French's (`add-lingua-french-grammar-tables`,
  matrix change 45, open) readings are written by their reference reducer into
  `tables/<studied>/grammar.tsv` with a person-written pinned pool in `tags.tsv`.
- Change 9 commits `tables/it/forms.tsv`, `freq.tsv` (60,000 lemmas), `studied.json`, an empty
  `lexical.tsv` and `tags.tsv` holding the placeholder `NOUN` — the split (change 3) refuses an input
  with no pool; change 4 builds each pair's base and layer and records both in `pin.json`. Its
  amendment reads a table form with its stress marks dropped (a final mark kept unless the cell's
  entry is bare) and a reflexive row without its pronoun, under the plain verb where the section has
  one. Change 7's cascade and change 11's acronym rows sit on those tables.
- The section, measured (proposal, Why): 13,382 verbs with a table, 441,628 form–lemma pairs over
  every cell in 49 tags; the tags kaikki writes — `indicative`, `subjunctive`, `conditional`,
  `imperative`, `present`, `imperfect`, `historic` + `past`, `future`, persons, numbers,
  `infinitive`, `gerund`, `participle` + `present`/`past` — and the rows that are bookkeeping or
  repeats: `canonical`, `auxiliary` (*avere*/*essere*), `imperative` + `negative` (the second person
  singular is the infinitive, the plurals the affirmative's forms), `formal` + `imperative` +
  `second-person-semantically` (the present subjunctive's third persons, *Lei*/*Loro*) — 87,141
  negative and formal rows. `it-noun` heads: `m` 36,150, `f` 30,150, `mfbysense` 3,162, `m-p` 183,
  `f-p` 138, `mf` 113; a second argument `#` or `-`, no plural: 8,915. Adjectives: feminine plural
  18,132, masculine plural 18,116, feminine 17,616, plural alone 7,648 (two-form adjectives).
- UD Italian (PUD, MarkIT) writes the passato remoto `Tense=Past`, the imperfetto `Tense=Imp`, the
  future `Tense=Fut`, the conditional `Mood=Cnd|Tense=Pres`, the imperative `Mood=Imp`, the gerund
  `VerbForm=Ger`, the participles `VerbForm=Part` with `Tense`.

## Goals / Non-Goals

**Goals:**

- Every form of change 9's tables that the section reads as a verb, noun, adjective, determiner,
  pronoun or numeral says what it is, as UD tags, unmerged.
- Italian's pool pinned, so a native's sense tags never reshuffle it, and one base serves it-fr,
  it-en and it-es.
- Measured on held-out treebanks; nothing else moves.

**Non-Goals:**

- The card's wording of Italian's forms, and merging moods on it (change 18).
- Readings of a form the tables do not hold (a cell no text writes).
- A reflexive verb's own readings apart from its plain verb's (change 9 files them under the plain
  verb; the card says « lavarsi » through the enclitic rule's lemma).
- The polite imperative named on the card (`Lei`): change 18's call; the subjunctive's reading
  carries the form.

## Decisions

### D1 — it-fr's reducer writes Italian's readings

`reduce-it-fr.py` writes `grammar.tsv` in the same pass as the forms, after they are chosen, on
`reduce_common.py` without editing it, as `reduce-fr-en.py` does French's: one line per reading,
`form<TAB>lemma<TAB>tag<TAB>other|-`, sorted. A later pair of Italian reads `tables/it/` as
committed.

### D2 — Which forms get readings

The forms `tables/it/forms.tsv` holds, under the lemmas `tables/it/freq.tsv` ranks — Spanish's and
French's D2: a card opens on a form the analysis resolved through the table. The 441,628 pairs
over every cell are cut to the attested forms of 60,000 lemmas; the figures at the pin (readings,
forms with one, `other` marks, tags) are recorded at task 2.1.

### D3 — Verb forms

| kaikki | UD |
|---|---|
| `indicative` + `present` / `imperfect` / `future` | `Mood=Ind`, `Tense=Pres` / `Imp` / `Fut` |
| `historic` + `past` (with or without `indicative`) | `Mood=Ind`, `Tense=Past` — the passato remoto |
| `subjunctive` + `present` / `imperfect` | `Mood=Sub`, `Tense=Pres` / `Imp` |
| `conditional` | `Mood=Cnd`, no tense (UD Italian writes `Tense=Pres`; D7) |
| `imperative` (not `negative`, not `formal`) | `Mood=Imp`, no tense; second person singular, first and second plural |
| persons and numbers | `Person`, `Number`, with `VerbForm=Fin` |
| `infinitive` | `VerbForm=Inf` |
| `gerund` | `VerbForm=Ger` |
| `participle` + `present` | `VerbForm=Part`, `Tense=Pres` (`parlante`) |
| `participle` + `past`, the table's row | `VerbForm=Part`, `Tense=Past`, `Gender=Masc`, `Number=Sing` |
| a past participle's agreed forms | its gender and number, through the participle's entry (D5) |

Every verb is `VERB`: `AUX` is a role in a sentence, which a card cannot see. A sense of a form's
own entry that merges persons or moods reads as each combination; a table row names one.

**Never a reading**: a negative imperative row — its second person singular is the infinitive
(« non parlare »), read as such already, and its plurals repeat the affirmative's forms —; a formal
imperative row — the present subjunctive's third persons, read as such already —; the `canonical`
and `auxiliary` rows (bookkeeping); a compound tense (not one token); a defective verb's empty cell
written `-` (68 pairs over the section's tables).

**A reflexive verb's rows read without their pronoun**, under the lemma change 9 files the bare
form under: `mi làvo` → `lavo`, *lavare*'s present indicative, first person singular, beside its
own row; `mi pènto` → `pento`, *pentirsi*'s, the section having no *pentire*.

### D4 — Nouns, adjectives, determiners, pronouns, numerals

- **A noun's gender** comes from `it-noun`'s first argument (`m`, `f`; `mfbysense` and `mf` a
  reading per gender; `m-p`, `f-p` plural only), else from its senses' tags. Its own form reads
  `NOUN|Gender=…|Number=Sing` (`Plur` for a plural-only noun), its plural `Number=Plur`.
- **A noun the dictionary gives no plural for** — `it-noun`'s second argument `#` or `-`, or a sense
  tagged `invariable` with no plural listed: `città`, `re`, `crisi`, `film` — reads singular and
  plural, as Spanish's `crisis` and French's `temps`: 8,915 heads over the section.
- **A noun's irregular plural of another gender** (`uovo` → `uova`, feminine; `braccio` → `braccia`)
  reads with the gender the head gives the plural (`NOUN|Gender=Fem|Number=Plur`), the plural's row
  carrying it.
- **A feminine noun's masculine row gives no reading** (`attrice`: masculine `attore`), as
  French's D4.
- **An adjective's own form** reads `ADJ|Gender=Masc|Number=Sing` when it has a feminine of its own
  (`rosso`), `ADJ|Number=Sing` when one form serves both (`grande`); its forms read by their tags
  (`rosse`: `Gender=Fem|Number=Plur`; `grandi`: `Number=Plur`); the apocopated masculines (`bel`,
  `buon`, `gran`, `san`, `quel`) are masculine singulars; a superlative reads `Degree=Sup`.
- **Determiners, the articles among them, pronouns and numerals** read their gender and number:
  `la` of *il*, `gli` of *il*, `una` of *uno*, `questa` of *questo*. An article is a determiner,
  as UD writes it. A pronoun's form names its gender or gives no reading, and a personal pronoun's
  other persons and cases are other words, not inflections (French's D4).

### D5 — Where a reading comes from, and what never gives one

French's D5, for Italian:
- **A lemma's table first**, **a form's own entry** for the pairs no table lists.
- **A form of a form, along one part of speech**, as change 9's D3 maps the form: `creduta`, the
  feminine of the participle `creduto`, reads `VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part`
  of *credere*. Added only where the form has no reading of that lemma in that part of speech and
  verb form already.
- **A reading's part of speech is one its lemma's entries hold**: a participle change 9 files under a
  noun's or an adjective's spelling (`parlata` under the noun *parlata*, when D5 of change 9 keeps the
  noun) gives no verb reading of that word and names its verb (*parlare*).
- **A spelling variant** reads as the word it spells (change 10's rows): `perchè` carries
  *perché*'s readings.
- **Never a reading**: an inflection or a sense the dictionary marks doubtful (the tags change 9
  leaves out of the forms); a form's own sense marked regional or of a register; a capitalised
  headword (`NATO`, `Roma`: change 11's rows read as themselves, with no reading); an entry whose
  every sense is an alternative form of another word (an archaic spelling with a table of its own);
  a link toward the word a reviewed override row of change 9 sets aside.

### D6 — `other` marks, and one form, one lemma (M8)

A reading of another ranked lemma is marked `other`, and the card names that word below, as
French's `fils` names *fil*: `vite` (the vines' plural, *vite*, and *vita*'s plural, whichever
change 9 files it under), `porti` (*portare*'s present and *porto*'s plural), `legge` (*leggere*'s
present and the noun *legge*, when the noun is ranked). Only toward an entry of the dictionary that
is not only regional. The mark changes no count: the form's status, exposures, level and review
belong to its one lemma.

**M8's cost on the card**: a dictionary noun whose own form the forms table reads as another lemma
(`porta` → *portare*, `pesca` → *pescare*) is no lemma of the tables, so no reading of it is kept;
counted among the 1,000, 2,000 and 5,000 commonest lemmas at task 2.1, beside change 9's figures.

### D7 — What the word card needs to merge moods (M21, read for Italian)

The readings are stored unmerged, as French's D7 does for change 51: `parli` carries four readings
— the present indicative of the second person singular and the present subjunctive of the first,
second and third —; `parla` two — the present indicative of the third person singular and the
imperative of the second singular —; `parliamo` three — present indicative, present subjunctive and
imperative of the first person plural. The two presents carry `Tense=Pres`, the imperative and the
conditional no tense, Spanish's and French's encoding, not UD Italian's `Mood=Cnd|Tense=Pres`: a
merge keyed by tense would otherwise fold the conditional into the present. All under the form's
own lemma, none marked `other`. Measured over the section's tables: 20,475 pairs read both the
present indicative and the present subjunctive; whether change 18 merges them is its call.

### D8 — Italian's tag pool, pinned

`tables/it/tags.tsv` holds **the tags Italian's readings carry**, one per line, each once, no
other, in byte order as the first reduction gives them (`cut -f3 grammar.tsv | LC_ALL=C sort -u`),
and a tag a later reduction adds appended after them. It **replaces** change 9's placeholder `NOUN`:
no Italian pack has shipped, so no index of the placeholder is held anywhere; `NOUN` stays a line
only if a reading carries it bare. A person writes it and reviews it in the pull request; no
reducer writes it (`KEPT_INPUTS`). Under the split (change 3), the base's pool is this pin and the
readings' tags outside it; a pair's sense tags live in the layer's `senses.tags`, so it-fr, it-en
and it-es share one base.

### D9 — Measured against held-out treebanks

`scripts/lingua-data/measure/it_readings.py`, run by change 9's `it-ud.sh` after its gates, over
the two files that script fetched — UD Italian-PUD and MarkIT's test section, never read by the
reduction: for each part of speech (finite verbs, participles, infinitives, gerunds, nouns,
adjectives, determiners, pronouns), the words whose form the tables map to the treebank's lemma,
the share that carry a reading, and the share of those whose treebank part of speech (`VERB` for a
verb or an auxiliary) and features are among their readings — the conditional's and the
imperative's tense left aside. It reads the committed tables and reports without deciding the exit
status. Each rule of D3–D6 is switched off alone once and its effect recorded, as French's D10.

### D10 — What moves, and what cannot

- **`it-fr.golden` does not move.** The Italian baseline runs over its fixture until change 16
  switches it to the committed tables (change 9's D9); the fixture keeps no grammar (its one-tag
  `tags.tsv`): a hand-written reading table would test the fixture, not the reduction. The readings
  are pinned instead by `it_fr_grammar.rs` over the committed tables.
- **No other pair moves**: no file of their rule digests changes, no file of `tables/en/`,
  `tables/es/`, `tables/fr/` or their pairs' folders; their packs rebuild to their pins and their
  goldens pass without re-blessing.
- **it-fr moves**: `tables/it/grammar.tsv` is new, `tags.tsv` is the pin, `manifest.json`
  (`pack_version`, the rule digest) and `pin.json` (base and layer) are recorded again;
  `forms.tsv`, `freq.tsv`, `lexical.tsv` and `studied.json` are byte for byte change 9's (and 10's,
  11's rows).
- **Italian's analyser version does not move**: readings are pack data; no token or lemma changes.

## Known data defects

Filled at task 2.1 from the reduction, as French's lists `fatiguée` and `supe`: rows the section
writes wrong, each with where its fix lives. Known before reducing: the 68 defective cells written
`-` (no form, D3).

## Risks / Trade-offs

- [Thin treebanks for agreement] → the figures are reported per part of speech, never gating; the
  rules are switched off one by one to show what each moves (D9).
- [A two-form adjective read without gender] → `ADJ|Number=Plur` for `grandi`, as UD writes it; the
  card names the number alone.
- [The formal imperative unnamed] → the subjunctive's reading carries the form; change 18 may name
  it.
- [M8's cost] → counted (D6); an override row of change 9 brings a noun back.

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr. The tables arrive by the update
workflow's pull request; rollback is a revert.

## Effort

3.5–5 ideal days, against the programme's 3–5: the reducer's readings (verbs, nouns, adjectives,
the closed classes, D5's sources and exclusions, `other`) 2–2.75; its tests 0.5–0.75; the pin, the
reduction and the README 0.25; `it_fr_grammar.rs` and `committed_tables.rs` 0.5; the measurement
and its switches 0.5–0.75.
