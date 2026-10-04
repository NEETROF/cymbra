# Design — add-lingua-dictionary-form-voice

## Context

See proposal.md (Why). Today, in `src/reading/wordpopup.ts`:

| Where | What it does |
|---|---|
| `show()` | Heads the card with `headword`, the dictionary form. When `surface` differs from it, case aside, it adds « forme vue : « … » ». |
| `listensFor(content)` | One button for the selection: « ▶ Mot », or « ▶ Sélection » for an expression. It reads `surface`. Then « ▶ Phrase », unless the selection is the whole sentence. |
| `renderListen()` | Rebuilds the row from the speaker's state. The button being read turns into « ■ Arrêter », and pressing another button switches to it. |

The selection card renders through the same card view, so it gets the same row.

## Goals / Non-Goals

**Goals:**
- A reader can hear the form they met and its dictionary form, and knows which button reads which.
- A word that is its own dictionary form keeps the row it has today.

**Non-Goals:**
- An expression's canonical form (`tener en cuenta` for a selected `teniendo en cuenta`). The
  selection card's expressions keep « ▶ Sélection ».
- Reading a contraction as written (`don't` for its `do`). The button reads `surface`, as before.
- Choosing another voice for the dictionary form: both read in the card's language, with the
  reader's voice.

## Decisions

### D1 — When the card offers two word buttons

Exactly when it shows « forme vue »: a word card, not an expression, whose `surface` differs from
its `headword`, case aside. The same test, written once, decides both, so a card never shows a form
seen without offering to hear both forms, nor the reverse. `Casa` at the start of a sentence is
`casa` and keeps one button. Accents count: `esta` and `está` are two words.

### D2 — Each button says what it reads

The two buttons are labelled with their texts: « ▶ Es » then « ▶ ser ». The form seen comes first,
as the reader clicked it, and as it is written on the page. Their accessible names say which is
which: « Écouter la forme vue « Es » », « Écouter la forme du dictionnaire « ser » ».

*Rejected — « ▶ Mot » and « ▶ Dictionnaire ».* Two generic labels for two words make the reader
guess, and the card's heading is already one of them.

*Rejected — one button reading the dictionary form.* It changes what `add-lingua-read-aloud`
specified for every card, and the reader would no longer hear the form they met, except inside the
sentence.

### D3 — The dictionary form's button behaves like the others

It speaks under a key of its own (`headword`). While it reads, it turns into « ■ Arrêter ».
Pressing another button switches, and closing the card or opening another word silences it.
`renderListen()` and the speaker do not change. A card completing with its answer keeps reading the
same text: a pending card that becomes a two-button card does not stop a reading of the form seen.

## Risks / Trade-offs

- **A long dictionary form** → it is one word, and the row wraps like the card's other rows.
- **A form seen that only differs by a diacritic the voice ignores** (`solo`, `sólo`) → two buttons
  that sound alike. They are two spellings, and the card already shows the form seen.
