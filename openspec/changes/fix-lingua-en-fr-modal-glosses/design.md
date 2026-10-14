# Design — fix-lingua-en-fr-modal-glosses

## Context

See proposal.md (Why). Today, on origin/main:

| Where | What it does |
|---|---|
| `scripts/lingua-data/reduce-en-fr.py` | en-fr's reducer. `main()` calls `common.native_tables(...)`, which returns `glosses` (lemma → gloss) and `runs` (lemma → `[(UPOS, count)]`), then writes `gloss.tsv` and `senses.tsv` from them. It already holds `LOCUTIONS`, expressions « written and reviewed by a person » that win over the source — for `mwe.tsv` only. |
| `reduce_common.py` | `native_tables`, `reduce_gloss`, `_join_senses_by_pos`: picks up to eight senses one per entry in turn, groups them by part of speech in first-appearance order, enforces 300/800 characters. |
| `reduce_edition_fr.py` | The French Wiktionary's rules. `_FORM_OF` drops « Prétérit/Passé/Participe… de X » pointer senses, but not « Subjonctif … », so `would` keeps « Subjonctif passé du verbe auxiliaire will ». |
| `tables/en-fr/` | `gloss.tsv`, `senses.tsv`, `mwe.tsv`, NOTICE, `manifest.json` (`pack_version` = snapshot + rule digest), `pin.json` (rule files `reduce-en-fr.py`, `reduce_common.py`, `reduce_edition_fr.py`, one digest; pack sha256). en-fr is English's reference pair, so its reduction also writes `tables/en/` (forms, freq, grammar, level, `lexical.tsv` = glossed lemmas less those glossed by proper-noun senses alone). |
| `crates/lingua-core/src/engine.rs` `group_senses` | Splits a gloss on the sense separator and lays it out by the runs; a run total that disagrees with the sense count falls back to one ungrouped block. The builder already fails when the two disagree. |
| `crates/lingua-wasm/src/lib.rs` `review_current` | Shows `card.gloss` when the card's gloss language is the engine's native language; only a card glossed in another language goes through `readable_gloss` (the pack's gloss first). |

The gates that bind any fix (lingua-data-packs):
- *The committed tables are what the rules make of the pinned sources* — a table edited by hand
  fails the checks.
- *A gloss is in the reader's language, written by a person* — never generated, never
  machine-translated.
- *A pack's dictionary words do not depend on its glosses* — en-fr's glossed lemmas are English's
  dictionary words; a gloss fix must not add or remove one.
- *A pair's reduction rules include the rules it shares* — touching `reduce_common.py` moves every
  pair's digest.

The earlier `OVERRIDES` mechanism (es-fr, fr-en) chooses a form's lemma: it rewrites forms and the
analysis. A wrong entry there (`vino → venir`) would have erased a noun from the lexicon, which is
why es-fr's is empty. Nothing in this change goes near forms.

Measured on the committed table: about eleven en-fr glosses open on a grammar-term wording
(« Subjonctif… », « Verbe auxiliaire… », « Auxiliaire… », « Utilisé(e)… », « S’utilise… »). Among
them, only the modals are common words whose meaning the source never states; the others
(`ancillary` « Auxiliaire », `subjunctive` « Subjonctif ») are correct.

## Goals / Non-Goals

**Goals:**
- `would`, `could`, `should`, `might`, `must`, `shall`, `shalt`, `can`, `may` and `will` read, on
  the word card, as what they mean in French, the modal sense first.
- A gloss fix that cannot touch a lemma, a form or the lexicon, by construction.
- en-fr moves by exactly twenty table lines; no other pair moves.

**Non-Goals:**
- **Refreshing glosses stored in existing deck cards.** Review shows a native-language card's own
  stored gloss (`review_current`), so a `would` card captured before this change keeps the old
  text. Whether review or a migration should refresh stored glosses from the current pack is a
  `lingua-decks-review` question that concerns every pack update, not this one; it gets its own
  change if wanted. Meanwhile the owner can capture the word again, which replaces the card (and restarts its
  schedule).
- **en-es.** Its `could` carries `can`'s « Enlatar » and its `might` only « Fuerza ». Its glosses
  must be written and reviewed by a person reading Spanish, and its reducer has its own digest:
  a follow-up (`fix-lingua-en-es-modal-glosses`), which would also promote the helper to the shared
  module (D2).
- A general rule for grammar-term senses (« Subjonctif… »): no rule recovers a meaning the source
  does not write (D1).
- Contractions (`'d`, `won't`, `couldn't`) and other auxiliaries whose gloss already states the
  meaning: `ought` (« Devrait, indiquant le devoir… »), `need`, `dare`.
- Any engine, card layout, extension or server change.

## Decisions

### D1 — A curated gloss table, not a cleaning rule

The entries are written by a person and live in the reducer, like `LOCUTIONS`: a rule file, so
its bytes are in the digest and the committed-tables check reproduces the result from the pinned
sources.

*Alternative — extend `_FORM_OF` to « Subjonctif… ».* It would drop `would`'s only meaningful-looking
sense and leave the slang sense alone, worse than today; and it changes `_FORM_OF`, which also
feeds en-fr's forms, frequencies and studied-side signals.

*Alternative — edit `gloss.tsv` by hand.* Forbidden by the committed-tables check, and lost at the
next dictionary update.

*Alternative — wait for the French Wiktionary.* The entries are written for dictionary readers
(usage notes, grammar descriptions); improving them upstream is welcome but would not reach a card
before the next snapshot, and nothing guarantees a card-sized wording.

### D2 — One entry replaces a lemma's gloss and runs together, in en-fr's reducer only

`WORD_GLOSSES` in `reduce-en-fr.py`: lemma → ordered list of `(UPOS, sense)`. A small function
applies it to the `glosses`/`runs` that `native_tables` returns, before anything reads them
(`resolve_forms`, `dictionary_words`, the writes):
- the gloss is the senses joined with the separator, the runs are counted from the entry's parts of
  speech — so `senses.tsv` agrees with `gloss.tsv` by construction;
- an entry whose lemma is not already in `glosses` raises, naming it: the glossed set — and so
  forms, `lexical.tsv` and the dictionary words — cannot change;
- an entry that breaks the gloss limits (more than eight senses, a sense over 300 characters, a
  gloss over 800, a separator inside a sense, a part of speech in two runs, an unknown UPOS, only
  `PROPN` senses) raises, naming it. The `PROPN`-only guard keeps the lemma among English's
  dictionary words.

The application returns new dictionaries and is a pure function, so a unit test can assert that
every key it does not name is equal, object for object, before and after.

*Alternative — replace only the gloss text and keep the source's runs.* The counts would disagree
(`would`'s `VERB:1 INTJ:1` against four verb senses), the build would fail, and the card would fall
back to one ungrouped block.

*Alternative — the helper in `reduce_common.py` with a `word_glosses=` argument to `native_tables`,
mirroring `locutions=`.* It is where it belongs once two pairs use it, but editing the shared module
moves the rule digest of every pair and forces all of them to be reduced again (byte-identical
tables, eight new `pack_version`s) for a fix to one. The en-es follow-up promotes it then.

*Alternative — reuse `OVERRIDES`.* It chooses lemmas, not glosses; see Context.

### D3 — The entries: meaning first, the source's own words when they say it

Writing rules, applied to every entry:
1. The first sense is the French meaning a reader needs on a page — a French verb, or the French
   tense with a short « English = French » example when the meaning *is* a tense.
2. A source sense that already states a meaning is reused word for word, only moved (`can`'s and
   `will`'s verb senses, `may`'s names).
3. A noun or a name the source gives is kept after the modal senses, in the source's wording, unless
   it glosses the word with itself (`must`'s « Must, devoir » becomes « Incontournable, chose à ne
   pas manquer »).
4. Slang and usage notes go (`would`'s « S’utilise pour indiquer que l’on trouve une personne… sexuellement
   attrayant », `shall`'s « Aujourd’hui, on emploie plutôt will… »).
5. Short: every proposed gloss is under 240 characters, every sense under 90.

The proposed table (gloss as it lands in `gloss.tsv`, runs as in `senses.tsv`):

| Lemma | Gloss | Runs |
|---|---|---|
| `would` | « Conditionnel : would come = viendrait; Futur dans le passé : he said he would come = il a dit qu’il viendrait; Habitude passée : he would walk = il marchait, il avait l’habitude de marcher; Demande polie : would you… ? = voudriez-vous… ? » | `VERB:4` |
| `could` | « Pouvoir, au passé : pouvait, a pu (I could swim = je savais nager); Pouvoir, au conditionnel : pourrait (could you… ? = pourriez-vous… ?) » | `VERB:2` |
| `should` | « Devrait, conseil ou obligation : you should go = tu devrais y aller; Devrait, attente : it should work = ça devrait marcher; Si jamais : should you need help = si jamais vous avez besoin d’aide » | `VERB:3` |
| `might` | « Pourrait, il se peut que : it might rain = il se peut qu’il pleuve; Passé de may : pouvait; Force, puissance » | `VERB:2 NOUN:1` |
| `must` | « Devoir, obligation : you must go = tu dois partir; Devoir, déduction : it must be late = il doit être tard; Incontournable, chose à ne pas manquer; Moût; Moisi, moisissure » | `VERB:2 NOUN:3` |
| `shall` | « Futur, surtout à la 1re personne : I shall go = j’irai; Proposition : shall we go? = on y va ?; Obligation, dans un texte de loi ou un règlement : devoir » | `VERB:3` |
| `shalt` | « Forme ancienne de shall avec thou : thou shalt not kill = tu ne tueras point » | `VERB:1` |
| `can` | « Pouvoir, savoir, avoir la capacité de; Pouvoir, avoir le droit de; Pouvoir, avoir la possibilité de; Mettre en boite de conserve; Boîte de conserve; Canette métal, souvent cylindrique; Bidon; Pot de chambre » | `VERB:4 NOUN:4` |
| `may` | « Pouvoir, avoir le droit de; Il se peut que, peut-être : it may rain = il se peut qu’il pleuve; Souhait : may you be happy = puisses-tu être heureux; Mois de mai; Nom de famille anglais; Nom féminin anglais, diminutif de Mary et Margaret » | `VERB:3 PROPN:3` |
| `will` | « Futur : she will come = elle viendra; Vouloir, dans le sens d’un acte délibéré ou d’un souhait intense; Léguer (par testament); Volonté; Testament; Prénom masculin » | `VERB:3 NOUN:2 PROPN:1` |

The modals keep the `VERB` tag the source gives them; tagging them `AUX` is a card-wording question outside this change. The owner reviews and may reword each line in
the pull request; the tests pin whatever is merged.

*Alternative — a single short sense per modal (« Devrait », « Pourrait »).* Card-sized, but loses
`would`'s past-habit and future-in-the-past readings, which the reader meets as often as the
conditional; the card pages its gloss, so four short senses cost nothing.

*Alternative — keep `ought` in the list.* Its gloss already reads « Devrait, indiquant le devoir… »;
the table is for glosses that fail, and every entry is a line to review at each dictionary update.

### D4 — Pinned per entry, and the reduction proves nothing else moved

- `test_reduce_en_fr.py`: one test per entry pins its exact gloss and runs; one test asserts the
  table is closed (exactly the ten lemmas); tests for each refusal (unglossed lemma, 301-character
  sense, split part of speech, separator inside a sense, `PROPN` only); one test that applying the
  table to a fixture's `glosses`/`runs` leaves every other key equal.
- The reduction: `build.sh --reduce en-fr` from the pinned snapshot, then a diff against the
  previous tables shows exactly ten changed lines in each of `gloss.tsv` and `senses.tsv`, no change
  in `mwe.tsv`, NOTICE or any `tables/en/` file, and new `pack_version`/`pin.json`. The CI job that
  re-reduces every pair from its pinned sources checks the same bytes.
- `crates/lingua-pack/tests/committed_tables.rs`, on the pack built from the committed tables:
  `would`'s grouped gloss is one verb group opening on « Conditionnel », `can`'s first group is the
  verb, `must` is verb then noun.

### D5 — Scope: en-fr

See Non-Goals. The table is a pair's rule; en-fr is the pack the owner reads, and the only one whose
modal glosses fail on the meaning.

## Risks / Trade-offs

- [A curated gloss is a person's wording, not the dictionary's] → It is what `LOCUTIONS` already
  does, and what *A gloss is in the reader's language, written by a person* allows: written and
  reviewed by a person, never generated. The pull request lists every line for the owner's review.
- [The examples quote English inside a French gloss] → The meaning is in French; the English
  fragment shows the construction, as the source's own glosses already do (« on emploie plutôt will »).
- [A later snapshot improves the source, and the table hides it] → *A dictionary update is a reviewed
  decision*: the update's diff of the reduced tables, read with the source's gloss for these lemmas, shows
  when it changes under an entry; the entry is dropped when the source says it better. Ten lines are cheap to re-read.
- [The table grows into a back door for any gloss the owner dislikes] → It is closed by a test and
  scoped by the requirement to glosses that name a form, give a fringe sense or hide the meaning;
  adding a lemma is a spec-visible rule change in its own pull request.
- [Existing deck cards keep the old text in review] → Stated, and out of scope (Non-Goals); the word
  card on the page shows the new gloss at once.

## Migration Plan

1. Merge: the en-fr tables, `manifest.json` and `pin.json` are committed reduced again; CI rebuilds
   the pack from them and checks its sha256.
2. Readers get the pack with the next releases that build it — the extension (Chrome, Firefox:
   `yarn gen:pack:real` into `assets/packs`, the reviewer's source archive rebuilds the same bytes),
   the Apple app (it embeds the Safari extension's packs), and the agent plugin when its packs are
   copied again. The pack is not served from the models host, so nothing is deployed server-side.
3. Existing deck cards: no migration. A card keeps the gloss stored at capture, and review shows it
   when it is in the engine's native language; the word card shows the pack's new gloss. A reader
   who wants the new text on an existing card captures the word again, which replaces the card and
   restarts its schedule.
4. Rollback: revert the commit; the previous tables and pin come back with it, and the next release
   ships the previous pack.
