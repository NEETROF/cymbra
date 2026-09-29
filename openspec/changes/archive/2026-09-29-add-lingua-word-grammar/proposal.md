# add-lingua-word-grammar — the word card says what the form is

## Why

Lingua counts words by their dictionary form, and that is what sets it apart. On the word card,
though, it goes unexplained. Opening `went` shows `go` as the headword and `forme vue : « went »`,
and never says that `went` is the prétérit of `go`. The reader sees the analysis happen but learns
nothing from it. The gloss has the same blind spot. It joins up to three senses taken across a
word's parts of speech, so `can` reads « Boîte de conserve; Pouvoir, savoir, avoir la capacité de;
Mettre en boite de con » with no hint that the first sense is a noun and the others are verbs.
Measured with the current reduction rules on the 2026-09-21 kaikki snapshot, 5 218 of the 24 420
glossed words (21 %) mix senses from several parts of speech.

The sources the pack is built from already say all of this, and the reducer throws it away:

| What the card could say | Where it is today |
|---|---|
| `went` is the prétérit of `go`, `gone` its past participle | ESDB lists a verb's derived forms in fixed slots (`go <v>: went, gone, going, goes`). `parse_esdb_relations` flattens them into form→lemma pairs. |
| `mice` is the plural of `mouse`, `better` a comparative | ESDB's noun and adjective slots, flattened the same way |
| « Boîte de conserve » is a noun sense, « Pouvoir » a verb sense | Every kaikki entry carries its part of speech. `_join_senses` reads the entries in turn and drops the label. |
| `leaves` may also be the plural of `leaf`, not only a form of `leave` | Both relations survive the reducer's filters, but `resolve_forms` keeps one lemma per form, so the other is lost. |

**Why now.** The next studied languages are Romance: fr, then it, es and pt. There, all of the
above goes from a courtesy to a necessity:

- A verb has about fifty forms, not five.
- A noun's gender often differs from its French equivalent (*la leche*, *el mar*).
- Pronouns attach to the verb (*dámelo*, *fazê-lo*).
- One written form is routinely two words (*fui*: *ir* and *ser*; *como*: « comme » and « je mange »).

The pack's first grammatical vocabulary will be read by every later pack, and no pack for
another language is built yet. Designing that vocabulary for Romance languages now costs a few
lines of spec. Designing it for English and widening it later would change the format.

## What Changes

- **The pack gains grammar tables.** They are optional and additive, like the level and
  expression tables. They hold three things:
  - the **readings** of each inflected form: the dictionary form it belongs to, and its part of
    speech and grammatical features;
  - for a written form the analyser reads as one dictionary form, the **other dictionary forms**
    in the pack that it is also a form of;
  - the **part of speech of each gloss's senses**, with the features that belong to the word
    rather than the form, such as a noun's gender.

  **One closed vocabulary** covers all three: the Universal Dependencies part-of-speech tags and
  a named subset of the UD morphological features. The subset is chosen to cover tense, mood,
  person, number, gender, verb form, degree and the pronoun categories that Romance clitics need.
  The pack stores codes, and the interface names them. English fills only the part it can.
- **The reducer keeps what it reads.** It keeps ESDB's slots and kaikki's parts of speech, and
  it writes two new committed tables. A gloss's senses are grouped by part of speech, in the
  order the parts of speech first appear, and a `;` inside a sense becomes a `,`, so senses
  split unambiguously. The diff of `gloss.tsv` shows both edits.
- **The core answers a word's grammar.** A new function takes the word as written and the
  card's dictionary form, and returns:
  - the form's readings of that dictionary form;
  - the other dictionary forms the written form also belongs to;
  - the pieces, when the pre-pass split the written word (today `don't` → `do` + `not`);
  - the gloss's senses grouped by part of speech.

  `lingua-wasm` and the extension's `AnalyzerPort` expose it. The page analysis is untouched and
  `ANALYZER_VERSION` does not move.
- **The word card uses it:**
  - The card says what the form is: « went » : prétérit de *go*.
  - It names the other readings as text: aussi pluriel de *leaf*.
  - It shows the pieces of a contraction: « don't » = do + not.
  - It lays the gloss out under the part of speech of its senses: *nom* — Boîte de conserve,
    then *verbe* — Pouvoir….

  The card still stores the flat gloss, so the card model, the review and the sync do not move.
  The grammar arrives in the same call as a known word's gloss. A card that already holds its
  gloss waits for the grammar only for a short bound, then completes as it does today.
- **A re-reduced dictionary names itself.** Tables reduced again from the same pinned sources
  under new rules get a `pack_version` that names both the snapshot and the rules. Today they
  keep the snapshot's bare version, as the ESDB switch did, which the spec already forbids:
  a changed dictionary must report a different version.

Not in this change:

- choosing between readings in context (a part-of-speech tagger);
- grammar as something to learn and review;
- opening another reading's card from the card;
- grammar on the review card;
- highlighting or counting by reading;
- tense names for any studied language other than English.

The vocabulary carries the Romance categories. The names of parts of speech, gender, number and
person are generic and ship here. A tense's name depends on the language: the same code is the
prétérit in English and the passé simple in Spanish, so those names come with each language's
pack. No manifest, permission, privacy text or store listing changes.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED: the grammar tables (their vocabulary, sources and additive contract) and a gloss's
    senses grouped by part of speech.
  - MODIFIED: the container's contents, the size budget's arbitration order, the update
    report (which now covers the new tables), and the `pack_version` of re-reduced tables.
- `lingua-analysis`:
  - ADDED: a word's grammar, answered from the pack on demand, deterministic and identical
    between native and WASM.
- `lingua-browser-extension`:
  - ADDED: the card names the form it was opened on, the card lays the gloss out by part of
    speech, and a card holding its gloss waits only briefly for its grammar.

**Archive order.** `add-lingua-phrase-gloss`, then `add-lingua-expression-table`, then this
change. This change modifies the container and size-budget requirements in the wording
`add-lingua-expression-table` gives them, and it extends the word card and the engine-waiting
card that `add-lingua-phrase-gloss` introduces.

## Impact

- **Products.** Cymbra Lingua only. It touches the data pipeline (`scripts/lingua-data`), the
  pack builder (`crates/lingua-pack`), the core (`crates/lingua-core`), `crates/lingua-wasm`,
  and the extension (`apps/lingua-extension`, all three variants). The Safari variant ships
  inside `apps/lingua-apple`, whose native code does not change.
  - Nothing is consumed from the platform, and everything here is new to Lingua.
  - Cymbra ID, Music, Live, the back office, the site and the backend are untouched: no
    `.proto`, no migration, no flag.
- **Data.** Two new committed tables in `scripts/lingua-data/tables/en-fr/`. `gloss.tsv` is
  re-reduced, with the order and separator edits above. The pack's sha256 and size in
  `pin.json` change. The sources, licences and NOTICE do not change: the tables derive from
  ESDB and kaikki, which the pack already ships. UD contributes only tag names, not data.
- **Pack.** `pack_version` moves and `analyzer_version` stays `1.1.0`. The new sections are
  estimated at a few hundred KB, against a pack of 1.54 MB and a budget of 5 MiB. The real
  figure is measured at build time.
- **Reader data.** None moves. Statuses, cards, decks and sync keep their keys and shapes.
  New cards store the regrouped gloss, and existing cards keep theirs.
