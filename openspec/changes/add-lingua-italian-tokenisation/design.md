# Design

## Context

See proposal.md — Why. What constrains the shape:

- The tokeniser (`crates/lingua-core/src/analysis/tokenize.rs`) runs language-neutral rules —
  UAX #29 segmentation, the hyphen run, the digit drop, the apostrophe mapping `’` → `'` in
  `push_word` — and a per-language arm: English splits its contractions (`don't` → `do` + `not`,
  sharing the span), Spanish splits `al` and `del` and normalises to NFC, French (matrix change
  40) has `french_words` (U+202F as a space), `push_french_elisions` (a closed table, a span per
  piece, the read word capitalised on its own first letter), `au`/`aux` split sharing the span,
  `du`/`des` whole, hyphenated inversions. `Token` holds a text and a source span; two tokens may
  share a span (a contraction's halves) or each hold its own (an elision's pieces).
- Change 5 gave Italian the baseline arm: the words seam is UAX #29, `nfc_for` returns the text
  as it came, `split_contraction` answers `None`, no elision; its golden records « l'uomo » and
  « nella » as one token each at `0.1.0`, and « po' », « perche' » and « e' » as `po`, `perche`
  and `e`: the edge-apostrophe trim is a rule of no language (`sailors'` → `sailors`), and UAX #29
  leaves a trailing apostrophe outside the word.
- The owner's I7: the `di` family whole — also the partitive article, as French's `du`/`des` —, the
  others split with their own spans, as `au`/`aux`; elision pieces with their own spans, as
  French's; truncations read as their word.
- The extension already opens, on a click between two pieces, the piece that starts there (change
  40's rule, language-neutral); no Italian pair is listed before change 20.
- A word card asks the grammar of a written word alone (`l’`, `al`): French answers one token for
  an elided piece on its own and the pieces for a contraction.

## Goals / Non-Goals

**Goals:**

- Every Italian article, pronoun and preposition written joined to the next word becomes a word
  of its own for the pack, the counts and the card, with a span the reader can click.
- The `di` family keeps one form, one lemma (M8), as French's `du`/`des`.
- Nothing of English, Spanish or French moves; the six goldens are byte for byte.

**Non-Goals:**

- Enclitic pronouns (`dirmelo`), NFC, the cascade, closed classes, names: change 7.
- The apostrophe that stands for an accent (`perche'`, `piu'`, `E'`): change 10.
- The expression keys that read the pieces (`al di là`, `in fin dei conti`): change 12.
- A gender for `l'` (`lo`/`la`) or for `un'`: the tokeniser reads a written word, it does not
  parse the phrase; the forms table resolves the lemma.

## Decisions

### D1 — What I7 puts here, and what it leaves to changes 7, 10 and 12

Here: elision pieces, the articulated prepositions, truncation, `0.2.0`. Change 7: the enclitic
rule (a verb's clitics are read off its end, as Spanish's are, and that needs the lexicon's
verbs), NFC, the cascade, closed classes, names. Change 10: a trailing apostrophe read as an
accent. Change 12: the keys an expression is reached by once its pieces are words. The order
mirrors French's 40 → 41 → 43d/e → 44.

### D2 — No narrow no-break space; the typographic apostrophe as every language has it

Italian typography puts no space before `?`, `!`, `:` or `;` and sets « » tight, so French's
U+202F rule (change 40's D2) has no Italian counterpart; `push_word` already maps `’` to `'` for
every language, and a piece's span holds whichever apostrophe the page wrote.

### D3 — Elision: a closed list, the word it stands for, a span per piece

**The rule.** In an Italian word, when the part before the first apostrophe — straight `'` or
typographic `’` — is one of the elided forms below, followed by a letter, the part up to and
including the apostrophe is a token of its own, and the rule runs again on what follows:

| Elided | Read as |
|---|---|
| `l'` | `lo` — the form that elides before a vowel; `la` would need the next word's gender, as French's `l'` → `le` |
| `un'` | `una` |
| `d'` | `di` — also `da` in « d'ora in poi »; `di` is the commoner |
| `c'` | `ci` |
| `v'` | `vi` |
| `m'` | `mi` |
| `t'` | `ti` |
| `s'` | `si` — also `se` in literary « s'io fossi »; `si` is the commoner |
| `n'` | `ne` |
| `ch'` | `che` |
| `gl'` | `gli` |
| `dov'` | `dove` |
| `com'` | `come` |
| `cos'` | `cosa` |
| `quand'` | `quando` |
| `dev'` | `deve` |
| `diss'` | `disse` |
| `quest'` | `questo` |
| `quell'` | `quello` |
| `anch'` | `anche` |
| `senz'` | `senza` |
| `tutt'` | `tutto` |
| `bell'` | `bello` |
| `buon'` | `buona` — the masculine elides without an apostrophe (`buon uomo`) |
| `grand'` | `grande` |
| `pover'` | `povero` — also `povera`; the masculine, as `l'` |
| `sant'` | `santo` |
| `mezz'` | `mezzo` — also `mezza` in « mezz'ora », its commonest use; the tables file both under one lemma |
| `nient'` | `niente` |
| `quant'` | `quanto` |
| `cent'` | `cento` |
| `vent'` | `venti` |
| `sott'` | `sotto` |
| `nessun'` | `nessuna` — the masculine elides without an apostrophe |
| `qualcos'` | `qualcosa` |
| `qualcun'` | `qualcuna` |
| `ciascun'` | `ciascuna` |
| `alcun'` | `alcuna` |

Matching is on the lowercased part; the read word takes the written first letter's capital on its
own first letter (`L'` → `Lo`, `Quest’` → `Questo`), by French's casing (the read word's own first
letter, never a byte slice of the written one). After the last elision, what remains goes on
through D4 and `push_word` as any word does: `dall'altro` is `da` + `lo` + `altro`,
`un'altr'ora` is `una` + `altro` + `ora`? No: `altr'` is not on the list, so `altr'ora` stays one
token — the list is closed on purpose, as French's is, and a form outside it is read as written.

**What stays whole.** A word whose part before the apostrophe is not on the list: `po'`, `fa'`
and the truncations (D5), `perche'` and the apostrophe-accent spellings (change 10), and any
lexicalised or rare elision the list leaves out. A plain word on the list is split whatever the
pack lists, as `al`, `del` and `don't` are: `c'è` and `l'ho` are entries of the Wiktionary, and
keeping them whole when the forms tables list them would hide *essere* and *avere*.

**The read word is the token's text**, as French's is: the pack then resolves a dictionary word
(`lo`, `una`, `ci`), change 7's closed classes and change 12's keys read dictionary words, and the
names rule sees a word the pack lists. At `0.1.0`, `L'uomo`, `C'era` and `Quest'anno` are
unlisted tokens (`l'uomo`, `c'era`, `quest'anno`), unglossed; here their pieces are pack words.

*Rejected — reading `l'` as `il`.* `il` never elides; a reader clicking `l'` would be shown a word
the page could not have written there. `lo` is the elided form before a vowel; the forms table
files `lo` and `la` under the article's lemma either way.

### D4 — The `di` family whole, the other articulated prepositions split, sharing the span (I7)

`del`, `dello`, `della`, `dei`, `degli`, `delle` stay whole: each is also the partitive article
(« del pane » — some bread), the tokeniser cannot tell its two readings apart, and « one form, one
lemma » keeps the pack's lemma for each; `dell'` is read as `dello`, its span holding the
apostrophe, by D3's mechanism with `dello` the read word and no second piece. The others are
preposition + article sharing the span, as `al`/`del` are in Spanish and `au`/`aux` in French:

| Written | Read as |
|---|---|
| `al`, `allo`, `alla`, `ai`, `agli`, `alle` | `a` + `il`, `lo`, `la`, `i`, `gli`, `le` |
| `dal`, `dallo`, `dalla`, `dai`, `dagli`, `dalle` | `da` + the same |
| `nel`, `nello`, `nella`, `nei`, `negli`, `nelle` | `in` + the same |
| `sul`, `sullo`, `sulla`, `sui`, `sugli`, `sulle` | `su` + the same |
| `col`, `coi` | `con` + `il`, `i` |
| `all'`, `dall'`, `nell'`, `sull'` | `a`, `da`, `in`, `su` + `lo`, the span holding the apostrophe |

`collo`, `colla`, `cogli`, `colle` (archaic, and nouns: *collo*, *colla*) and `pel`, `pei`
(archaic) stay whole. The capital: `Al` → `A` + `il`, `NEL` → `In` + `il`, by French's casing
rule. Nothing is split inside a hyphenated compound; a remainder after an elision is.

*Rejected — splitting the `di` family too.* « del pane » would be `di` + `il` + `pane`, a
partitive read as a definite article; I7 settled it as M21 did for `du`/`des`.

### D5 — Truncation: a closed list of words ending in an apostrophe

Italian's words seam (D2's counterpart of `french_words`) re-attaches a trailing apostrophe —
straight or typographic — to the word before it only when the lowercased word, apostrophe
included, is on a list: D3's elided forms written alone (a word card hands the engine « l’ »: read
as `lo`, one token spanning the apostrophe, as French reads « l’ » alone through
`apostrophe_ending_a_word`) or the truncations below. Any other trailing apostrophe — the closing
quote of « ‘ciao’ », the apostrophe that stands for an accent in « perche' » — stays outside the
word, as the rule of no language leaves it (`‘ciao’` is `ciao`, `perche'` is `perche`); change 10
extends the list with its reviewed spellings, which is where `perche'` belongs. A listed word
whose last character is an apostrophe is then one token whose span holds the apostrophe. A
truncation's read word is final: `da'` → `dai` is not re-read by D4, while a written `dai` is
`da` + `i` by I7 — the verb « dai! » is the cost, named in Risks. When the lowercased word is on the list, it is read as the word it stands for:

| Written | Read as |
|---|---|
| `po'` | `poco` |
| `mo'` | `modo` |
| `fa'` | `fai` |
| `va'` | `vai` |
| `da'` | `dai` |
| `sta'` | `stai` |
| `di'` | `dici` |
| `be'` | `bene` |

Any other word ending in an apostrophe (`perche'`, `e'`, `piu'`, `citta'`) loses it, as today:
the apostrophe that stands for an accent is change 10's, where a reviewed list maps each to its
accented word — change 5's design placed it here; the programme's row 6 is corrected in this
change. The imperatives are read as their full second-person forms (`fai`, `vai`), which
the forms table files under the verb.

*Alternative.* Reading the imperatives as their dictionary form (`fare`): the tokeniser reads
written words, not lemmas; `fai` is what the page could have written.

### D6 — Spans: the `Token` model does not change

An elided piece and the word after it each hold their own span (D3); a split articulated
preposition's two tokens share theirs (D4); a truncation is one token (D5). This is the shape
change 40 fixed for French, and the extension's hit test (change 40's D7) resolves a click between
two pieces to the piece that starts there. Two Italian tokens hold a span ending on an
apostrophe — an elided form alone and a truncation —, so `is_elided`'s invariant (no other
token's span ends on one) widens to « an elided piece or a truncation », its doc and the page
joins' say so; harmless today (`meets_as_written` runs for French alone) and inherited by change
12. A token's text maps `’` to `'`, as every language's does.

### D7 — `0.2.0`, the fixture, the probes

`ITALIAN_ANALYZER_VERSION = "0.2.0"`, the fixture's manifest with it. The fixture gains the words
the pre-pass writes where it lacks them (change 5's holds the plain articles, pronouns and `a`,
`e`, `è`, `o`) — the articles `il`, `lo`, `la`, `i`, `gli`, `le`, `una`; `di`,
`a`, `da`, `in`, `su`, `con`; the clitics `ci`, `vi`, `mi`, `ti`, `si`, `ne`; `che`, `dove`,
`come`, `questo`, `quello`, `anche`, `senza`, `tutto`, `bello`, `grande`, `santo`, `mezzo`,
`niente`, `quanto`; `dello`, `della`, `dei`, `degli`, `delle`; `poco`, `modo`, `fai`, `vai`,
`dai`, `stai`, `dici`, `bene` — so the golden shows them resolved, not unglossed. The support
module's phrase and grammar probes gain « C'è un po' di pane », « all'alba » and « Nel mezzo del
cammin » (Dante, a line in the public domain), and (`l’`, `lo`), (`al`, `a`).

### D8 — What the golden shows, and what does not move

The re-bless of `it-fr.golden` is the diff of this change: every elision of the `elisioni`,
`manzoni`, `nomi`, `narrativa`, `tecnico` (`l'API`) and `notizie` pages becomes pieces, every
`al`/`nel`/`sul`/`dal`/`col` of the `preposizioni` and `notizie` pages two tokens, the `di` family
unchanged, `po'` read as `poco`. The `informale` page's `perche'`, `piu'`, `citta'`, `e'` and `E'`
do not move: their trailing apostrophe stays outside the word, as at `0.1.0`, until change 10. The six goldens, S0 and every pin are byte for byte; the English, Spanish and French arms
are not touched.

## Risks / Trade-offs

- [`l'` read as `lo` before a feminine noun] → the card and the counts file `lo` and `la` under
  the article's lemma; the written piece and its span are exact; the alternative needs the next
  word's gender, which no tokeniser has.
- [A rare elision outside the list] → read as written, as French leaves `prud'homme`; the list is
  reviewed by the owner (task 4.1) and can grow at a later analyser bump.
- [A truncation that is also a word] → none of the eight is; `da'` and `di'` with the apostrophe
  are distinct from `da` and `di`.
- [« dai! », the verb, read as `da` + `i`] → I7's split; the verb's card is reached through `da'`
  and its other forms; named for the owner (task 4.1).
- [`s'`, `d'`, `mezz'`, `pover'` read as their commoner word] → the piece's text is the word the
  page could have written; the tables file both readings' lemmas where they differ.
- [The corpus's `informale` page] → untouched here; change 10's diff shows it alone.

## Migration Plan

Nothing deploys and nothing is released; no listed pair studies Italian. Rollback is a revert
with the golden's.
