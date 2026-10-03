# Design — add-lingua-spanish-analysis

## Context

See proposal.md (Why). Today Spanish is the baseline (`analysis/*.rs`):
- `tokenize` with no contraction split;
- `lemmatize_baseline` (the pack's forms, else the form);
- no function words;
- `SPANISH_ANALYZER_VERSION = "0.1.0"`.

The English pre-pass, cascade and tables are untouched by anything here. The English invariance
baseline (`crates/lingua-wasm/tests/english_baseline.rs`) is the gate.

The forms tables (`add-lingua-spanish-forms-tables`) come later and are measured on real text. These
rules are written against small test lexicons, which is how the English cascade was written too.

## Goals / Non-Goals

**Goals:**
- Spanish's contractions, enclitics, old spellings and unlisted plurals reach their lemmas.
- Spanish's closed classes are left out of a word-by-word gloss.
- Each rule is deterministic, guarded, and pinned by fixtures; Spanish moves to `1.0.0`.

**Non-Goals:**
- Suffix rules for diminutives, superlatives or `-mente` adverbs: Wiktionary lists them, and the
  forms tables carry them.
- Homographs (`como`, `sobre`, `vino`): the forms tables pick one lemma per form
  (`add-lingua-spanish-forms-tables`).
- Detection of Catalan and Galician (`add-lingua-spanish-detection-guard`).
- Adding accents a text left out (`esta` for `está`): a missing accent can be on any vowel, and
  guessing would merge different words.

## Decisions

### D1 — The pre-pass: NFC, then `al` and `del`

- **NFC.** Spanish tokens are normalised to NFC before anything reads them, so a decomposed `á`
  meets the pack's precomposed one. This is Spanish only: English's output must not move. The byte
  spans still point into the source text.
- **Contractions.** A whole token `al` or `del` (any case) becomes two tokens, `a` + `el` or
  `de` + `el`, sharing the source span. The first keeps the token's leading capital (`Del` → `De`,
  `el`), as the English pre-pass does for `Don't`. Inside a hyphenated compound nothing is split.

### D2 — The cascade's order

For a token lowercased and in NFC:
1. **The pack's forms** (`lemma_of`).
2. **Accent retry.** When the form carries an acute accent (á é í ó ú), the form without its acute
   accents. ü and ñ are kept. This reads the spellings the 2010 rules retired (`fué`, `dió`, `vió`,
   the preposition `á`, `sólo`, `éste`). It only removes accents, never adds them.
3. **Enclitics** (D3).
4. **Plural fallback** (D4), only for a form nothing above resolved.
5. **The form itself.**

The order is the determinism contract. A change to it bumps Spanish's analyser version.

### D3 — Enclitics, a guarded rule

The clitics are `me`, `te`, `se`, `nos`, `os`, `lo`, `la`, `los`, `las`, `le`, `les`. The rule runs
only when steps 1 and 2 found nothing: a listed word is never split. It tries, in a fixed order, one
clitic stripped from the end, then two. At each step longer clitics come first (`nos` before `os`,
`los` before `lo`). The first candidate whose base passes every check wins.

**The written base**, the form minus its clitics, is checked against the lexicon in one of two ways:
- **as written**: `reírse` → `reír`, `hacerlo` → `hacer`;
- **with its one acute accent removed**: `dámelo` → `dá` → `da`, `diciéndole` → `diciendo`. The
  removed accent must sit on the vowel the base itself stresses. That is the stress shift the
  clitics caused, and nothing else.

To find the stress, take the last vowel group if the base ends in a consonant other than `n` or
`s`, else the one before it. Within a group, the stressed vowel is its strong vowel (a, e, o), else
its last vowel.

**Restorations.** After `nos`, a base ending in `mo` is also tried with `s`: `vámonos` → `vamos`,
`sentémonos` → `sentemos`. After `os`, a base ending in a vowel is also tried with `d`: `sentaos` →
`sentad`. `idos` is `id` + `os`, read as written.

**The base's shape** must be one that takes enclitics:
- **an infinitive**: it ends in `ar`, `er`, `ir` or `ír`;
- **a gerund**: it ends in `ndo`;
- **another verb form of two vowel groups or more**: an imperative such as `levanta` or `mira`, or
  a subjunctive such as `diga`;
- **a monosyllable** only from the closed list of affirmative imperatives: `da`, `dad`, `di`, `haz`,
  `id`, `pon`, `sal`, `sé`, `ten`, `ve`, `ved`, `ven`.

The lemma is the base's lemma in the pack.

*Rejected — a table of verb + clitic forms.* Two clitics after any infinitive, gerund or imperative
of any verb is combinatorial, and the forms tables would grow by an order of magnitude for nothing a
rule cannot do.

*Rejected — splitting enclitics into tokens.* `dámelo` is one word on the page and one card, and
the pronouns are closed-class words a learner does not count. The token keeps its surface and takes
the verb's lemma.

### D4 — Plurals outside the lexicon

Only for a form the lexicon does not know at all, so that the singular and plural of an unlisted
word count as one. The rules, in order:
1. **`-ces`** after a vowel → `-z`: `luces` → `luz`, `veces` → `vez`, `actrices` → `actriz`. After
   a consonant, it is the `-s` rule: `dulces` → `dulce`.
2. **`-iones`** → `-ión`: `canciones` → `canción`.
3. **`-es`** after a vowel and one of `l`, `r`, `n`, `d`, `j`, `y` → that consonant: `árboles` →
   `árbol`, `ciudades` → `ciudad`, `reyes` → `rey`, `relojes` → `reloj`. A stem ending in a vowel
   and `n` drops an accent the plural added: `exámenes` → `examen`.
4. **`-s`** after a vowel → stripped: `casas` → `casa`, `madres` → `madre`, `posibles` →
   `posible`.

Forms of four letters or fewer are left alone, and so are those ending in `-is` or `-us` (`crisis`,
`virus`, `análisis`).

### D5 — Closed classes

Six sorted tables, as English has, checked after lemmatisation:
- **determiners**: `el`, `la`, `los`, `las`, `un`, `una`, `unos`, `unas`, possessives,
  demonstratives, quantifiers;
- **pronouns**: personal, clitic, reflexive, relative and interrogative, indefinite;
- **prepositions**: `a`, `ante`, `bajo`, `con`, `contra`, `de`, `desde`, `durante`, `en`, `entre`,
  `hacia`, `hasta`, `mediante`, `para`, `por`, `según`, `sin`, `sobre`, `tras`;
- **conjunctions**: `y`, `e`, `o`, `u`, `ni`, `pero`, `sino`, `aunque`, `porque`, `pues`, `que`,
  `si`, `como`, `cuando`, `mientras`;
- **auxiliaries and modals**: `haber`, `ser`, `estar`, `poder`, `deber`, `soler`;
- **negation**: `no`.

The tables hold the dictionary form and the inflected forms a pack may keep as their own lemmas
(`la`, `las`, `esta`). The homograph policy of the forms tables is not decided yet, and a gloss must
not show a row for `la` either way. `nunca` is left out, like English's `never`: it is an adverb
with content of its own.

### D6 — Spanish `1.0.0`, English unchanged

`SPANISH_ANALYZER_VERSION` becomes `1.0.0`. A Spanish pack built before carries the baseline version
and is refused by its own version check, which is what that check exists for. No Spanish pack has
shipped. English's code paths are not touched, and its baseline is the gate.

## Risks / Trade-offs

- **A false enclitic split on a word the pack does not list.** It needs a base the lexicon lists as
  a verb-shaped form of two syllables or more, and an accent consistent with its stress. The closed
  list keeps monosyllables out. Fixtures pin the guards: listed words, a wrong accent, an
  unlisted monosyllable.
- **The plural fallback on an unlisted singular ending in `-s`.** `-is` and `-us` are excluded.
  Other cases (`lunes`) are in the forms tables. The fallback only ever groups forms the pack does
  not know.
- **Accent retry merges a retired spelling with a word that kept its accent** (`sólo`/`solo` are
  one word since 2010). It only runs when the accented form is not listed.
