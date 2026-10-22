# add-lingua-italian-card-wording — the word card's grammar, said in Italian

## Why

Change 34 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. The word card describes a form once, in tags that name no language, and renders it with
the interface language's words (`generalise-lingua-card-wording`). Change 33 writes the Italian
interface's 23 text modules and leaves the 24th, `grammar.ts`, to this change. Until it exists,
`reading-copy.ts` renders an Italian interface's cards with the English renderer, and the parity test
exempts `grammar` for Italian. This change writes the Italian renderer.

The renderer names the forms an Italian speaker reads with en-it, fr-it and es-it: English's,
French's and Spanish's. Its Italian row is required by the type and reaches no reader today (no pair
studies its own native), but its names are the ones French and Spanish tenses take. I6 settles
Treccani's names for Italian's tenses and moods. English has no Treccani name. Matrix change 24's
precedent for Spanish readers of English answers it: the native edition's form-of wording (the Italian Wiktionary writes
« passato semplice di eat », « participio presente di go ») and « forma in -ing ».

## What Changes

- **`src/i18n/it/grammar.ts`** (design D1–D5): the Italian renderer, typed after the French as the
  English and Spanish ones are.
  - **Italian forms** in Treccani's names, mood before tense (« indicativo imperfetto »,
    « passato remoto », « congiuntivo presente »).
  - **French and Spanish forms** by the Italian name of the same tense (the passé simple and the
    pretérito perfecto simple are « passato remoto »).
  - **English forms** as the Italian Wiktionary names them, and « forma in -ing ».
  - **Persons, genders and numbers** in Italian (« terza persona singolare », « femminile plurale »).
  - **Contractions** with the tense's article (« del », « dell' »), never before a word of the page.
- **The Italian interface reads it** (D6): `reading-copy.ts` maps Italian to the Italian renderer.
  Change 33's exemption of `grammar` ends, and its requirement on the typed catalogue is MODIFIED
  to match.
- **Pinned before any Italian reader sees it** (D7): a snapshot renders, in Italian, the grammar
  lines of every grammar probe of the en-es, es-en, fr-en and it-fr goldens. A form's readings are the
  same whatever the native, so the probes are real forms; sense headings are the native side's and
  are left out. The snapshot is re-blessed where the other card snapshots are. The full card snapshots, with glosses,
  are the en-it, fr-it and es-it packs' (changes 36, 47, 51).
- **Reviewed by the Italian reviewer agent** (D8, I2), not by the owner; change 44 rereads it.
- **Nothing else moves**: the French, English and Spanish renderers and every existing snapshot
  stay byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *An Italian interface names forms in Italian grammar's terms* and
  *The Italian renderer's grammar lines are pinned on the real packs' readings*. MODIFIED *An Italian
  card names its forms in the interface language's grammar* (held by `add-lingua-italian-word-card`):
  Italian joins its list of interface languages, with Treccani's names.
- `lingua-interface-language`: MODIFIED *The interface's copy is kept in one typed catalogue per
  language* (held by `add-lingua-italian-interface`). The Italian modules are typed after the French
  for every surface, `grammar` included. The scenario *The drafts are whole* reads every Italian entry.

## Impact

- **Products.** Cymbra Lingua only. In `apps/lingua-extension`:
  - new: `src/i18n/it/grammar.ts`, `test/word-grammar-it.spec.ts`, `test/word-card-grammar-it.spec.ts`
    and its committed snapshot;
  - changed: `src/reading/reading-copy.ts`, `src/i18n/README.md`, the comments of
    `src/reading/grammar-description.ts` and `src/analyzer/types.ts`, the card harness
    `test/word-card-french.ts` (a grammar-only export), `test/reading-copy.spec.ts`, and the renderer
    loops of `test/grammar-description.spec.ts`, `test/i18n.spec.ts` and `test/wordpopup.spec.ts`;
  - `.github/workflows/lingua-pack-update.yml`: the snapshot is re-blessed with the goldens.

  ID, Music, Live, the back office, the site and every pack are untouched.
- **Nothing is shown**: the Italian interface appears with change 45.
- **Prerequisites**: changes 18 (`StudiedLanguageCode` holds `it`, Italian's readings, the
  parameterised card harness, the it-fr golden's probes), 32 and 33 implemented.
- **Order**: before changes 36 (en-it's card snapshot), 44 and 45.
- **Effort, against 3–5.5 ideal days**: 3.5–4.5 (design, *Effort*).
