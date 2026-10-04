# Design — fix-lingua-spanish-card-noise

## Context

See proposal.md (Why). In `reduce-es-fr.py`:

| Where | What it does |
|---|---|
| `read_entry` | Every kaikki entry with a sense that is no form-of makes its word a lemma and its own candidate, a proper name (`pos: name`) included: `Miró` makes `miró` a candidate of itself. |
| `choose_lemma` | Among a form's candidates: an override, GSD's counts, the form's own entry, then the lemma's frequency. |
| `common.reduce_gloss` | Shared with en-fr. Reads the French Wiktionary's Spanish entries, up to eight senses grouped by part of speech, in the source's order. |

GSD's counts are worth keeping for names. It meets `Argentina` 92 times as the country and 17 as the
adjective, so they rightly keep the country. They are wrong where GSD lacks the word's genre: news and
encyclopedia text holds no narrative preterite like `miró`.

## Goals / Non-Goals

**Goals:**
- A form that is mostly a common word reads as that word, whatever GSD's genre.
- A country, a city or a famous name keeps the form when it is the commoner reading.
- No letter's sense opens a card.

**Non-Goals:**
- Case. The lexicon is lowercase, so `Irán` the country and `irán` (*ir*'s future) share one reading.
  The commoner wins, the verb here.
- The document's names (`add-lingua-spanish-names`), which set a novel's characters aside at reading
  time.
- en-fr, whose shared gloss reduction does not change.

## Decisions

### D1 — Between a proper name and a word, the commoner reading wins

`read_entry` records, for each word, whether its lemma entries are proper names. A word all of whose
lemma entries are names is a name. When a form's candidates include the form itself as a name and other
words, the form's wordfreq frequency is weighed against the highest of the words' lemmas:
- the name wins when the form is commoner: `argentina` 5.38 against *argentino* 4.83;
- the words win otherwise: `miró` 3.98 against *mirar* 4.74.

The form's frequency counts it met as the name and as the word alike, so it is the name's best case.
The losing side leaves the candidates, and `choose_lemma` then decides as before among the winners.
GSD's counts, which come first there, would otherwise give the name every form it meets.

Only the form's own name competes. A name's table also lists words of their own (`boliviano` under
Bolivia), which are no reading of the form as the name.

*Rejected — leaving GSD's proper nouns out of its counts.* The countries lose to their adjectives,
since GSD meets `argentina` as an adjective too. One mislabelled token then decides alone: GSD tags
`pera` once as a noun of *perar*, and the pear left the pack.

*Rejected — a name never takes a form another word has.* `bosnia`, `china` and `argentina` would read
as their adjectives.

### D2 — A letter is no French gloss

Before the shared gloss reduction reads the French Wiktionary's Spanish entries, the es-fr reducer
writes them without their letters:
- an entry of the `character` part of speech, the letter itself;
- a sense whose gloss names a letter: « Nom de la lettre d. », « Bé, nom de la lettre b. », « Lettre
  s. » alone, « … lettre de l'alphabet … ».

An entry left with no sense goes. `reduce_common.py` does not change, so en-fr's rules and pin do not
move. « Lettre d'amour » (`carta de amor`) is no letter's name and stays.

### D3 — Reduced again, and measured

The tables are reduced again from the pinned snapshot. The PUD gates do not move. The README and
`SOURCES.md` give the new figures.

## Risks / Trade-offs

- **A famous name commoner than its word keeps the form**: `salta`, the province, over *saltar*;
  `ramos`, the surname, over *ramo*. Those were the name before too.
- **A rare given name may go to a word near it**: `conchita` to *concha*, `carmela` to *carmelo*.
  They are rare. A character's card is set aside by `add-lingua-spanish-names` when the word has no
  gloss.
