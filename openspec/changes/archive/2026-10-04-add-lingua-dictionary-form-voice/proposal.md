# add-lingua-dictionary-form-voice — the card reads the dictionary form too

## Why

Dogfooding the Spanish reading (2026-10-04), the owner opened the card of `Es`. The card is headed
with the dictionary form, *ser*, and shows « forme vue : « Es » ». Its « ▶ Mot » button read `es`.

That is the rule `add-lingua-read-aloud` set: the word button reads the selection as it appears on
the page, not its dictionary form (`ran`, not `run`). But the button names neither, and the card is
headed with the other one. A reader who looks at *ser* and hears « es » thinks the voice is wrong.
They cannot hear the dictionary form at all.

Spanish makes it common: `es`, `fue` and `soy` are *ser*'s, `voy` is *ir*'s. The owner chose to
offer both, each button saying what it reads.

## What Changes

- **Two word buttons when the form seen differs.** When the card shows a form seen that differs from
  its dictionary form (the case it already uses for « forme vue »), the listen row offers:
  - « ▶ Es », which reads the form as it appears on the page;
  - « ▶ ser », which reads the dictionary form;
  - then « ▶ Phrase », as before.
- **Otherwise, as before.** A word that is its own dictionary form (`casa`, or `Casa` at the start of
  a sentence) keeps one « ▶ Mot » button. A selection of several words keeps « ▶ Sélection ».
- **Every language.** The rule is the card's: `ran` offers « ▶ ran » and « ▶ run ».

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *The card reads a word's dictionary form when it differs from
  the form seen*. *The card reads the selection and its sentence aloud* is held by
  `add-lingua-read-aloud`, still open, and is not modified: the button that reads the selection as
  it appears on the page stays.

## Impact

- **Products.** Cymbra Lingua's browser extension only: the word card's listen row
  (`src/reading/wordpopup.ts`), which the selection card shares. No engine, pack, server or proto
  change. ID, Music, Live, the back office and the site are not affected.
- **Release.** With the next release of the extension. A reader of English sees two buttons on the
  card of an inflected word (`ran`), one on the card of a word that is its own dictionary form.
