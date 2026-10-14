# fix-lingua-en-fr-modal-glosses — an English modal's card says what it means

## Why

The owner's dogfood of the Lingua review (2026-10-14) met `would`, from « knowing it would not
come ». Its card read:

> « Subjonctif passé du verbe auxiliaire will; S’utilise pour indiquer que l’on trouve une personne
> ou un personnage sexuellement attrayant »

The first sense names a form, the second is slang. Nothing on the card says what the reader needs:
the French conditional, *viendrait*. `would` is the 55th commonest English word and an A1 word.

It is not alone. In the committed en-fr `gloss.tsv`, the English modal auxiliaries are glossed by
their grammar, by a fringe sense, or behind a noun:

| Lemma | en-fr gloss today | What is wrong |
|---|---|---|
| `would` | « Subjonctif passé du verbe auxiliaire will; S’utilise pour indiquer que l’on trouve une personne… » | A form's name, then slang (`senses.tsv`: `VERB:1 INTJ:1`) |
| `could` | « Utilisée pour poser des questions, avec de la politesse » | No « pouvoir » at all |
| `should` | « Verbe auxiliaire exprimant le conditionnel; Devrait, faire mieux de. Verbe auxiliaire exprimant l’obligation… » | Descriptive, long; « devrait » buried |
| `shall`, `shalt` | « Auxiliaire pour formuler le futur … Aujourd’hui, on emploie plutôt will dans ce sens; Devoir » | A usage note, not a translation |
| `might` | « Force, puissance; Peut-être. Il se peut que » | The noun first, though the modal is what is read |
| `must` | « Devoir; Must, devoir; Moût; Moisi, moisissure » | « Must, devoir » glosses the noun with the word itself |
| `can` | « Boîte de conserve; Canette…; Bidon; Pot de chambre; Pouvoir, savoir… » | Four nouns before « pouvoir » |
| `may` | « Mois de mai; Nom de famille anglais; Nom féminin anglais…; Pouvoir… » | Three names before the verb |
| `will` | « Volonté; Testament; Pour indiquer une intention… dans le futur; … » | The future third, and descriptive |

Every one of them is an A1/A2 word the reader meets on every page. The French Wiktionary writes
these entries for readers of a dictionary, not of a card, and no reduction rule can recover a
meaning the source does not state: dropping the « Subjonctif passé » sense would leave `would` with
the slang sense alone.

The en-es pack, glossed from translation tables, reads well for `would` and `should` (« Empleado
para describir una acción … condicionado … », « Habría, debería »), but has two gaps of its own:
`could` carries `can`'s « Enlatar », and `might` has only the noun « Fuerza ». They are left to their
own change (design, Non-Goals).

## What Changes

- **A short, closed table of word glosses written by a person** for en-fr, in the en-fr reducer,
  beside the `LOCUTIONS` table it already holds for expressions. Each entry gives a lemma's senses,
  each with its part of speech, and replaces that lemma's gloss and its part-of-speech runs as one
  unit. It touches no lemma, form, frequency, level, reading or expression: it only rewrites a gloss
  the source already gave.
- **Ten entries**: `would`, `could`, `should`, `might`, `must`, `shall`, `shalt`, `can`, `may`,
  `will`. The modal meaning comes first; a noun or name the source gives is kept after it
  (`must` keeps « Moût », `can` keeps « Boîte de conserve »); `would`'s slang sense goes. Where a
  source sense already says the meaning, it is reused word for word and only reordered (`can`,
  `will`'s « Vouloir… »). The proposed glosses are listed in design D3; each is reviewed by the
  owner before merge.
- **Guard rails**: an entry for a lemma the source does not gloss fails the reduction, naming it;
  an entry must obey the gloss limits every gloss obeys (eight senses, 300 characters a sense, 800 a
  gloss, no separator inside a sense, one run per part of speech); one test pins each entry.
- **The en-fr tables are reduced again** from the pinned snapshot. Exactly ten lines of
  `gloss.tsv` and ten of `senses.tsv` change; every other line, every other en-fr table and every
  `tables/en/` file stays byte for byte; `manifest.json`'s `pack_version` and `pin.json` record the
  new rule digest.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A word gloss written by a person replaces the source's*.

## Impact

- **Products.** Cymbra Lingua only (en-fr pack). ID, Music, Live, back office and site: nothing
  consumed, nothing new. No engine, extension code, server or `.proto` change.
- **Data.** `scripts/lingua-data/reduce-en-fr.py` (the table and its application) and
  `test_reduce_en_fr.py`; `scripts/lingua-data/tables/en-fr/{gloss.tsv,senses.tsv,manifest.json,pin.json}`.
  `reduce_common.py` and the edition modules do not change, so no other pair moves.
- **Reaching readers.** The pack ships inside the extension (`apps/lingua-extension/assets/packs`,
  built from the committed tables by `yarn gen:pack:real` and checked against `pin.json`), the Safari
  build embedded in the Apple app, and the agent plugin's copy. The word card shows the new gloss
  with the next extension and Apple releases. A card already in a deck keeps the gloss stored when it
  was captured: review shows a card's own gloss when it is in the engine's native language (design,
  Migration Plan).
