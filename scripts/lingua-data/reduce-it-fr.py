#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the it-fr pack into its tables (add-lingua-italian-forms-tables).

it-fr is Italian's reference pair (the programme's I5, design D1): its reduction writes Italian's
studied side, which `pack_sources.py split` files into tables/it/, and every other pair studying
Italian (it-en, it-es) reads it as committed. Italian's studied rules live here, as French's live in
reduce-fr-en.py: a reducer named with a hyphen cannot be imported, so no other pair loads them, and
no shared module is edited.

Inputs, in `--work`:
- `kaikki-Italian.jsonl`: the English Wiktionary's Italian section, derived from the English
  edition's dump (pack_sources.py EDITIONS). A lemma's entry lists its inflections, each with tags; a
  form's own entry points at what it is a form of (`form_of`).
- `it_markit-ud-train.conllu`, `it_markit-ud-dev.conllu`: UD Italian-MarkIT's training and
  development sections, read for how often each form stands for each lemma (D5) and how often each
  hyphenated lemma occurs (D6) — `reduce_italian_treebank.py`. Its test section and UD Italian-PUD
  are never read: the measurement holds them out (D2, D8).
- `morph-it-0.4.8.tgz` (`MORPH_IT`), Morph-it!'s archive as its site serves it: a form it lists under
  a lemma the tables rank, which the Wiktionary names nowhere, joins the table (D3). Read while the
  owner keeps it among the pinned sources (I11); `MORPH_IT = None` leaves it out.
- wordfreq `it` (the installed, pinned package): the 60,000 commonest lemmas and which forms are
  attested at all (D6).

Outputs, in `--work`: Italian's `forms.tsv` and `freq.tsv`, its readings `grammar.tsv`
(add-lingua-italian-grammar-tables, the section's `Italian's word grammar`), its dictionary words
`lexical.tsv` (the acronym rows alone until change 16's glosses write the rest), it-fr's `gloss.tsv`
(empty until change 16), `NOTICE` and `manifest.json`. Italian's pinned tag pool, `tables/it/tags.tsv`,
is a person's: no reduction writes it.

Italian's acronyms (keep-lingua-italian-acronyms-and-names D2, D6): Italian's analysis reads a word
standing alone in capitals as written, and the reviewed table `ACRONYMS` of
`reduce_italian_acronyms.py`, a rule module every pair glossing Italian imports, is what it finds —
`NATO` and `USA`. The section's entries headed exactly as a row are read for the table (`check_rows`),
each row's lower case must be a form of the tables read as another word (`check_lowercase`), and each
row is written into `forms.tsv` as its own lemma, capitals kept, and into `lexical.tsv`, with no rank;
`nato` and `usa` keep their readings. The rows' gloss is change 16's (`acronym_gloss`).

What a form is (D3). An Italian word, lowercased and in NFC, the typographic apostrophe read as `'`,
in the letters of the section's Italian entries (`_TOKEN`). A form's candidate lemmas come from the
inflections a lemma's entry lists — never its bookkeeping, a doubtful row, a row naming a derived word
(a diminutive, an augmentative, a pejorative, a derogatory or an endearing form, an adjective's
adverb, a noun's relational adjective, a name's demonym), nor a gender or number marker — and from
the form-of links of the form's own entry, an apocope's own `alt_of` among them (`un` → *uno*, `buon`
→ *buono*); a form of a form reaches the word it is a form of along one part of speech (`creduta` →
`creduto` → *credere*). The section's tables write every verb form with its stress marked (`dàre`,
`darèbbe`): every mark but on the last letter is dropped, and the last letter's is kept — Italian's
written accent — unless the entry carrying the cell's form-of link is the bare spelling (`hà` →
`ha`, `dò` → `do`, while `dà` keeps its accent: `da` is the preposition). A reflexive verb's row
written with its pronoun (`mi làvo`) gives its bare form, filed under the plain verb where the section
has one (`lavo` → *lavare*; `lavarsi` is no lemma) and under the reflexive lemma otherwise (`pento`
→ *pentirsi*), whose rows carrying the pronoun give their bare base (`pentirsi` → `pentire`,
`pentendosi` → `pentendo`) so that change 7's enclitic rule has its bases; a row carrying any other
clitic, a multi-word lemma's cell and a row listing alternatives give no form.

The tables serve Italian's tokenisation as change 6 writes it (D4): every word its pre-pass writes is
a form — the elided pieces' words, the split articulated prepositions' pieces, the truncations'
words —, and so is each elided piece and truncation, as the word it stands for (`l'` → *lo*, `un'`
→ *uno*, `po'` → *poco*), for the treebanks that write them; the `di` family (`del`, `dello`,
`della`, `dei`, `degli`, `delle`) are lemmas of their own, `dell'` a form of `dello`; the other
articulated prepositions are neither forms nor ranks, `dai` excepted — a form of *dare* and no rank,
the truncation `da'`'s word. No plain word beginning with an elided piece is a form (the pre-pass
splits it), nor any verb form carrying clitics (`dirmelo`, `vattene`: the enclitic rule reads them).

One lemma per form (D5, M8): a form whose own lemma entries are names alone reads as the other word
(`bambini`, a surname, → *bambino*; `roma`, which no other word reads, stays the city); then the
override list (`OVERRIDES`: the owner's `stato` and `stati` → *essere*, and `bianca` → *bianco*),
MarkIT's counts, the form's own entry, the lemma's wordfreq frequency, alphabetical order. The ranks
(D6) are wordfreq `it`'s order over its words that are not only inflected forms, dense, the 60,000
first kept, a word whose own form reads as another giving its rank to the next; wordfreq's stems of
the elided pieces and truncations and its tokens beginning with a piece are no words (`cent` is, and
the apocopes written without an apostrophe are forms of their words by a reviewed table, `APOCOPES`:
`bel` → *bello*, `san` → *santo*), a hyphenated word is ranked on MarkIT's evidence, and no verb form
carrying clitics is ranked.

Italian written without its marks, or with others (read-lingua-italian-unmarked-spellings). A word the
section gives only as a spelling of one word — a misspelling, an alternative, obsolete, archaic or
nonstandard spelling or form — is a form of that word when wordfreq rates the word the commoner
(`perchè` → *perché*, `fà` → *fare*; D2, `dictionary_spellings`), its paradigm following it where no
other word claims a form (`Lexicon.drop_spelling_rows`), a form only it reads keeping no reading when
another language writes it more or a source names it (`foreign_readings`); the report names each
spelling read and each kept apart. The web's spellings
without their accent read as their word by a reviewed list and the owner's exceptions (`piu` → *più*,
`perche` → *perché*; D3, `WEB_SPELLINGS`, `OWNER_EXCEPTIONS`), the reduction stopping for a person when
the section names a row (`Lexicon.web_boundary`) or a row's word is no ranked lemma.
"""

import argparse
import collections
import itertools
import json
import os
import re
import sys
import tarfile
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_italian_acronyms as acronyms  # noqa: E402 — Italian's acronym table (keep-lingua-italian-acronyms-and-names)
import reduce_italian_treebank as treebank  # noqa: E402 — UD Italian-MarkIT's counts (D2, D5, D6)

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")

# The raw sources, in `--work` (pack_sources.py DUMPS["it-fr"], PINNED["it-fr"]).
KAIKKI = "kaikki-Italian.jsonl"
MARKIT = ("it_markit-ud-train.conllu", "it_markit-ud-dev.conllu")
# Morph-it! (D3, I11): the archive its site serves, read for its lexicon alone — the member named
# below, in Latin-1 — and never extracted. Its site writes 0.4.7's figures (504,906 entries, 34,968
# lemmas); the archive it serves holds 0.4.8 (2009), which is what is pinned. None when the owner
# leaves it out of the pinned sources (task 5.1): the reduction then reads the Wiktionary alone.
MORPH_IT = "morph-it-0.4.8.tgz"
MORPH_IT_MEMBER = "current_version/morph-it_048.txt"
MORPH_IT_ENCODING = "latin-1"

# Italian letters, lowercased: the section's Italian entries', the loans' included (würstel, crêpe).
_LETTERS = "a-zàèéìíîòóùúâêôäëïöüçñ"
# An Italian form (D3): letters, words joined by hyphens or by an inner apostrophe, and the elided
# pieces' and truncations' final apostrophe (`l'`, `po'`).
_TOKEN = re.compile(rf"[{_LETTERS}]+(?:['-][{_LETTERS}]+)*'?")

# The bookkeeping of kaikki's inflection tables, never forms: the class, the template, the table's
# own tags, the headword repeated — and a verb's auxiliary (`avére`, `èssere`), which its table lists
# among its rows.
_BOOKKEEPING = frozenset({"table-tags", "inflection-template", "class", "romanization", "canonical", "auxiliary"})
# An inflection a source marks doubtful is no inflection (D3), as French's: an alternative or a
# misspelling of the form, an obsolete, archaic, rare, dated or uncommon one, an abbreviation or a
# clipping, a pronunciation spelling, what kaikki could not tag, a multi-word construction.
_DOUBTFUL = frozenset(
    {
        "alternative",
        "obsolete",
        "archaic",
        "rare",
        "dated",
        "uncommon",
        "misspelling",
        "nonstandard",
        "proscribed",
        "misconstruction",
        "abbreviation",
        "clipping",
        "pronunciation-spelling",
        "error-unknown-tag",
        "multiword-construction",
    }
)
# A row naming another word (D3): a noun's or an adjective's diminutive, augmentative, pejorative,
# derogatory and endearing forms (`casa` → `casétta`), an adjective's adverb (`vero` → `veramente`),
# a noun's relational adjective (`mare` → `marittimo`, tagged `relational`) and a name's demonym
# (`Bergamo` → `bergamasco`), and the singular a plural-only noun names (`plural-of`). Each is read
# from its own entry where the section has one.
_DERIVED = frozenset(
    {"diminutive", "augmentative", "pejorative", "derogatory", "endearing", "adverb", "relational", "demonym", "plural-of"}
)
# The gender and number markers a head template writes, which the dump leaves among a few entries'
# forms: a marker is no form of another word (French's `_MARKERS`).
_MARKERS = frozenset({"m", "f", "n", "c", "p", "s", "mf", "pl", "sg", "mpl", "fpl", "inv"})
# The entries of a clitic compound (`dirmelo`, `vattene`, `eccolo`) that read the compound itself:
# the verb's, the adverb's (`ecco`'s) and the interjection's (« vattene! », go away!). A noun or an
# adjective spelled like one (`fallo`, the foul) stands.
_CLITIC_POS = frozenset({"verb", "adv", "intj"})

# Change 6's closed tables (add-lingua-italian-tokenisation D3–D5; lingua-core's
# `ITALIAN_ELISIONS`, `ITALIAN_ELIDED_PREPOSITIONS`, `ITALIAN_CONTRACTIONS`, `ITALIAN_TRUNCATIONS`),
# kept here as reviewed tables: test_reduce_it_fr.py holds them to tokenize.rs. Each elided piece is a
# form of the word the pre-pass reads it as (D4) — the pre-pass never looks a piece up, but a treebank
# writes it (UD: `l'`) and a card can be opened on one alone.
ELISIONS = {
    "alcun'": "alcuna",
    "anch'": "anche",
    "bell'": "bello",
    "buon'": "buona",
    "c'": "ci",
    "cent'": "cento",
    "ch'": "che",
    "ciascun'": "ciascuna",
    "com'": "come",
    "cos'": "cosa",
    "d'": "di",
    "dev'": "deve",
    "diss'": "disse",
    "dov'": "dove",
    "gl'": "gli",
    "grand'": "grande",
    "l'": "lo",
    "m'": "mi",
    "mezz'": "mezzo",
    "n'": "ne",
    "nessun'": "nessuna",
    "nient'": "niente",
    "pover'": "povero",
    "qualcos'": "qualcosa",
    "qualcun'": "qualcuna",
    "quand'": "quando",
    "quant'": "quanto",
    "quell'": "quello",
    "quest'": "questo",
    "s'": "si",
    "sant'": "santo",
    "senz'": "senza",
    "sott'": "sotto",
    "t'": "ti",
    "tutt'": "tutto",
    "un'": "una",
    "v'": "vi",
    "vent'": "venti",
}
# `diss'` is read `dissi` before `io` (change 6's one reading in context): a word the pre-pass writes.
CONTEXT_WORDS = ("dissi",)
# The elided articulated prepositions (D4): `dell'` reads whole as `dello`, a form of it; the others
# are split into the preposition and `lo`, neither of them a form with its apostrophe.
ELIDED_PREPOSITIONS = {"all'": ("a", "lo"), "dall'": ("da", "lo"), "dell'": ("dello",), "nell'": ("in", "lo"), "sull'": ("su", "lo")}
# The articulated prepositions the pre-pass splits into the preposition and the article (I7): never
# handed to the lookup, so neither forms nor ranks — `dai` excepted (`DAI`).
CONTRACTIONS = {
    "agli": ("a", "gli"),
    "ai": ("a", "i"),
    "al": ("a", "il"),
    "alla": ("a", "la"),
    "alle": ("a", "le"),
    "allo": ("a", "lo"),
    "coi": ("con", "i"),
    "col": ("con", "il"),
    "dagli": ("da", "gli"),
    "dai": ("da", "i"),
    "dal": ("da", "il"),
    "dalla": ("da", "la"),
    "dalle": ("da", "le"),
    "dallo": ("da", "lo"),
    "negli": ("in", "gli"),
    "nei": ("in", "i"),
    "nel": ("in", "il"),
    "nella": ("in", "la"),
    "nelle": ("in", "le"),
    "nello": ("in", "lo"),
    "sugli": ("su", "gli"),
    "sui": ("su", "i"),
    "sul": ("su", "il"),
    "sulla": ("su", "la"),
    "sulle": ("su", "le"),
    "sullo": ("su", "lo"),
}
# The truncation `da'` reads as `dai` (change 6 D5), handed to the lookup whole; a written `dai` is
# split by the pre-pass and never reaches it, so listing it hides nothing (I7's cost): a form of
# *dare*, and no rank.
DAI = "dai"
# The truncations (D5 of change 6): each a form of the word it stands for, read as that word's lemma.
TRUNCATIONS = {
    "be'": "bene",
    "da'": "dai",
    "di'": "dici",
    "fa'": "fai",
    "mo'": "modo",
    "po'": "poco",
    "sta'": "stai",
    "va'": "vai",
}
# M21's partitives, kept whole by the pre-pass (I7): each a lemma of its own whatever its entries
# point at, as French's `du` and `des`.
DI_FAMILY = ("del", "dello", "della", "dei", "degli", "delle")
# Every piece the pre-pass reads before an apostrophe, with it.
PIECES = frozenset(ELISIONS) | frozenset(ELIDED_PREPOSITIONS) | frozenset(TRUNCATIONS)
# The apocopes written without an apostrophe (D6), a reviewed table: form → (lemma, reason). Each is
# a form of its word alone, whatever the section's entries make of it — `bel`'s entry is a meaning
# (« preconsonantal masculine singular form of bello »), so it would be a lemma of its own, and
# `san`'s own entry is the Greek letter, which the own-entry rule (D5) would keep. Read as French's
# `UNMARKED_SPELLINGS` are: the row's lemma is the form's one candidate, and the form is no lemma.
# Checked and left out: `bei`, bello's plural before a consonant, no apocope, which the dictionary
# already reads as *bello*. Editing this table changes the rules' sha256, as for any rule.
APOCOPES = {
    "alcun": ("alcuno", "alcuno before a noun (« alcun dubbio »); also the elided piece alcun'"),
    "bel": ("bello", "bello before a consonant (« un bel giorno »); its entry is a meaning, no link"),
    "buon": ("buono", "buono before a noun (« buon giorno »); also the elided piece buon'"),
    "ciascun": ("ciascuno", "ciascuno before a noun; also the elided piece ciascun'"),
    "gran": ("grande", "grande before a consonant (« un gran numero »)"),
    "nessun": ("nessuno", "nessuno before a noun (« nessun dubbio »); also the elided piece nessun'"),
    "qual": ("quale", "quale before a vowel or a consonant (« qual è »)"),
    "qualcun": ("qualcuno", "qualcuno before altro (« qualcun altro »); also the elided piece qualcun'"),
    "quel": ("quello", "quello before a consonant (« quel giorno »)"),
    "san": ("santo", "santo before a consonant (« San Marco »); its own entry, the Greek letter, would win"),
    "tal": ("tale", "tale before a noun (« tal modo »)"),
    "un": ("uno", "the article before a vowel or a consonant (« un uomo »); also the elided piece un'"),
}
# wordfreq's tokeniser splits a short piece off before a vowel (`l'uomo` → `l` + `uomo`) and strips a
# token's edge apostrophes (`po'` → `po`): its list counts a piece's stem at the piece's frequency.
# The stems are no words (D6) — every piece's, the elided prepositions' too, but the apocopes'
# (`APOCOPES`) and `cent`, the euro cent, a word of its own — and nor are the truncations' stems that
# are no word of their own (`po`, `mo`, `be`; `da`, `di`, `fa`, `va` and `sta` are words).
STEM_WORDS = frozenset(APOCOPES) | {"cent"}
WORDFREQ_STEMS = frozenset(
    {piece[:-1] for piece in ELISIONS} | {piece[:-1] for piece in ELIDED_PREPOSITIONS} | {"po", "mo", "be"}
) - STEM_WORDS
# A plain word beginning with a piece (`c'è`, `l'ho`, `dov'è`, `dell'acqua`): the pre-pass splits it
# there — a hyphenated run too (`l'anglo-americano`) —, so the whole string is never looked up.
_ELIDED_START = re.compile(
    "(?:"
    + "|".join(re.escape(piece) for piece in sorted(set(ELISIONS) | set(ELIDED_PREPOSITIONS), key=len, reverse=True))
    + rf")[{_LETTERS}]"
)
# The pronoun a reflexive verb's row is written with (`mi làvo`, `ci laviàmo`).
REFLEXIVE_PRONOUNS = frozenset({"mi", "ti", "si", "ci", "vi"})
# An infinitive carrying clitics: the `r` of a truncated infinitive and one clitic, or a combined one
# (`lavarsi`, `esserci`, `andarsene`, `farcela`).
_CLITIC_INFINITIVE = re.compile(r"r((?:glie|me|te|se|ce|ve)(?:lo|la|li|le|ne)|mi|ti|si|ci|vi|lo|la|li|le|gli|ne)$")
# A single clitic at the end of a reflexive verb's form (`pentèndosi`, `pentìtevi`).
_REFLEXIVE_TAIL = re.compile(r"(mi|ti|si|ci|vi)$")
# Any clitic, single or combined, ending a verb form (`pentirmi`, `andarmene`, `dirglielo`).
_CLITIC_TAIL = re.compile(r"(?:(?:glie|me|te|se|ce|ve)(?:lo|la|li|le|ne)|mi|ti|si|ci|vi|lo|la|li|le|gli|ne)$")
# The stress marks the section's tables write (D3), each with its bare vowel; a loan's marks (ü, ê,
# ç, ñ) are spelling, never stress, and stay.
_STRESS = {"à": "a", "è": "e", "é": "e", "ì": "i", "í": "i", "ò": "o", "ó": "o", "ù": "u", "ú": "u"}
# How far a form of a form is followed (D3): `creduta` → `creduto` → *credere* is one step.
_FOLLOW_STEPS = 3
# wordfreq's list is read this many times the lemmas kept: enough past the cut for its words of the
# cut's frequency, and for the inflected forms it skips.
_WORDFREQ_DEPTH = 6

# Homographs a person decided (D5), checked before any count: form → (lemma, reason). The reason is
# part of the rule: a row without one is not a decision. A row costs the other lemma its place in the
# pack — a lemma's own form always reads as itself —; M8's cost is listed in tables/it-fr/README.md.
# Editing this table changes the rules' sha256, so the tables must be reduced again, as for any rule.
OVERRIDES = {
    "stato": (
        "essere",
        "the past participle of essere (and stare), Italian's commonest auxiliary participle (« è stato »); "
        "MarkIT's counts (29 nouns, 9 essere, 7 stare) and the own entry read it as the noun, which was 39 of "
        "UD Italian-PUD's 40 missed auxiliaries; the owner's decision of 2026-10-23, « lo Stato » reading as "
        "essere its accepted cost",
    ),
    "stati": (
        "essere",
        "the plural of stato, the past participle of essere and stare (« sono stati »); the owner's decision "
        "of 2026-10-23, with stato's",
    ),
    "bianca": (
        "bianco",
        "the feminine of bianco (« la casa bianca »); the section's noun entry (the cue ball) and MarkIT's one "
        "count, a proper noun's, would read it as the noun, and the names rule leaves a form with a common "
        "reading of its own to D5",
    ),
}

# The rules that decide a form's one lemma (D5), as the reduction reports them.
DECIDED_BY = ("override", "MarkIT's counts", "its own entry", "frequency", "alphabetical order")

# — Italian written without its marks, or with others (read-lingua-italian-unmarked-spellings) —

# The pointers of a sense's `alt_of` that are no spelling of another word (D2): an apocope, which D6
# reads as a form (`_form_targets`), an abbreviation, a clipping, a contraction, an initialism or an
# acronym, and a pronunciation spelling (`'n`).
_NO_SPELLING = frozenset(
    {"abbreviation", "apocopic", "clipping", "contraction", "initialism", "acronym", "pronunciation-spelling"}
)

# Why `dictionary_spellings` keeps a spelling apart (D2), as the reduction reports it.
KEPT_COMMONER = "the spelling as common as its word or commoner"
KEPT_FORM = "a form of the spelling as common as its word or commoner"
KEPT_TWO_WORDS = "its senses spell more than one word"
KEPT_NO_FORM = "no word form"
KEPT_TABLE = "a word of a reviewed table"
KEPT_OWNER = "kept apart by the owner"

# The dictionary's spellings the owner keeps apart (D2, task 4.1, 2026-10-23): spelling → reason. Each
# reads as it read before the rule, a word of its own; editing it changes the rules' sha256, as for any
# rule.
SPELLINGS_KEPT_APART = {
    "casinò": "the gambling house, which the section gives as « alternative form of casino »; il casino is a mess",
    "cè": "a mistyped c'è far more often than the section's « nonstandard spelling of cioè »",
    "omg": "the web's « oh my god », not the section's « alternative form of ogm »",
}

# The web's Italian written without its accents (D3): spelling → word, each row a decision by name —
# the owner's review of the measured list (tasks 2.2 and 4.1) —, never a rule on marks: a rule would
# read `e`, `da`, `si`, `se`, `ne`, `la` and `li` as *è*, *dà*, *sì*, *sé*, *né*, *là* and *lì*.
# A row is a spelling wordfreq attests at Zipf 2.5 or more that the Italian section names nowhere —
# neither a word, nor a form of a word, nor an inflection a word's entry lists, nor its accented
# word's spelling (D2's) —, whose accented word's lemma has a sense of its own in the section that is
# no name's (until Italian's dictionary words are written, change 16, which re-checks every row), at
# least twice as common as the spelling (a Zipf gap of 0.30 or more), and that neither English,
# French nor Spanish writes more often than Italian does. Its word is the lemma the tables give the
# accented form (`puo` → *potere*, `sara` → *essere*): a row follows the tables' choices and never
# makes one. The reduction fails, naming it, when the section names a row so (`Lexicon.web_boundary`).
# `e`, `da`, `si`, `se`, `ne`, `la`, `li`, `papa`, `eta`, `pero` and `cafe` are never rows.
# Merged into one table with the owner's exceptions below; editing it changes the rules' sha256, as for
# any rule.
WEB_SPELLINGS = {
    "affinche": "affinché", "ahime": "ahimè", "altresi": "altresì", "andra": "andare", "andro": "andare",
    "attivita": "attività", "attualita": "attualità", "autorita": "autorità", "avra": "avere", "avro": "avere",
    "bensi": "bensì", "bonta": "bontà", "celebrita": "celebrità", "chissa": "chissà", "cio": "ciò",
    "cioe": "cioè", "civilta": "civiltà", "comunita": "comunità", "contabilita": "contabilità",
    "criminalita": "criminalità", "curiosita": "curiosità", "daro": "dare", "difficolta": "difficoltà",
    "dignita": "dignità", "disponibilita": "disponibilità", "dovra": "dovere", "dovro": "dovere",
    "entita": "entità", "facolta": "facoltà", "fara": "fare", "funzionalita": "funzionalità", "gesu": "gesù",
    "giovedi": "giovedì", "giu": "giù", "granche": "granché", "identita": "identità", "indennita": "indennità",
    "legalita": "legalità", "localita": "località", "lunedi": "lunedì", "martedi": "martedì",
    "maturita": "maturità", "mentalita": "mentalità", "mercoledi": "mercoledì", "modalita": "modalità",
    "nonche": "nonché", "novita": "novità", "nullita": "nullità", "opportunita": "opportunità",
    "parita": "parità", "percio": "perciò", "personalita": "personalità", "piu": "più", "poiche": "poiché",
    "popolarita": "popolarità", "possibilita": "possibilità", "potra": "potere", "poverta": "povertà",
    "priorita": "priorità", "probabilita": "probabilità", "professionalita": "professionalità",
    "proprieta": "proprietà", "provera": "provare", "pubblicita": "pubblicità", "puo": "potere",
    "purche": "purché", "qualita": "qualità", "quantita": "quantità", "realta": "realtà",
    "responsabilita": "responsabilità", "rimarra": "rimanere", "riusci": "riuscire", "sanita": "sanità",
    "sara": "essere", "saro": "essere", "schiavitu": "schiavitù", "sensibilita": "sensibilità",
    "serenita": "serenità", "serieta": "serietà", "societa": "società", "solidarieta": "solidarietà",
    "sovranita": "sovranità", "specialita": "specialità", "succedera": "succedere", "tabu": "tabù",
    "tranquillita": "tranquillità", "trinita": "trinità", "umanita": "umanità", "universita": "università",
    "vabbe": "vabbè", "validita": "validità", "varieta": "varietà", "vedra": "vedere", "vedro": "vedere",
    "velocita": "velocità", "venerdi": "venerdì", "verita": "verità", "virtu": "virtù",
    "visibilita": "visibilità", "volonta": "volontà",
}
WEB_REASON = "written on the web without its accent; the owner's reviewed list"
# The owner's exception rows (D3; task 0.1, settled by the owner on 2026-10-23), outside the method and
# the boundary: spellings the section gives a sense of their own — a rare homograph — that the web
# writes for a commoner word, each with its reason — and `pò`, po' with a wrong accent, added at review on
# the owner's behalf. `finche`, `felicita`, `umidita` and the method's other homographs are not rows: they
# read as the tables read them.
OWNER_EXCEPTIONS = {
    "perche": (
        "perché",
        "perché written without its accent: the section gives perche only as perca's plural (the perch, a fish), "
        "which no page of the corpora writes; wordfreq 4.95 against perché's 6.18, the web's commonest "
        "spelling of it",
    ),
    "cosi": (
        "così",
        "così written without its accent: the section gives cosi as coso's plural (thingy) and cosare's present; "
        "wordfreq 5.12 against così's 6.14; the corpora's 3 lowercase « cosi » are così",
    ),
    "citta": (
        "città",
        "città written without its accent: the section's citta is citto's feminine (a Tuscan, archaic girl); "
        "wordfreq 4.16 against città's 5.77; Tatoeba's 2 « citta » are città",
    ),
    "caffe": (
        "caffè",
        "caffè written without its accent: the section gives caffe as caffo's feminine plural and caffa's plural "
        "(odd, of a number); wordfreq 3.75 against caffè's 4.83",
    ),
    "gia": (
        "già",
        "già written without its accent (the owner, 2026-10-23): the section lists gìa, the archaic imperfect of "
        "gire, so the tables read gia as ire; wordfreq 4.59 against già's 5.95",
    ),
    "liberta": (
        "libertà",
        "libertà written without its accent (the owner, 2026-10-23): the section gives liberta as liberto's "
        "feminine (a freedwoman), which the tables read it as; wordfreq 3.73 against libertà's 5.16",
    ),
    "pò": (
        "poco",
        "po' written with a grave accent (added at review, 2026-10-23): « un pò », wordfreq 4.47, the web's "
        "commonest misspelling of po', which the tables read as poco; the section lists pò in potere's table, an "
        "archaic and southern third person (« nun pò »), which the tables read it as before",
    ),
}


def nfc_lower(text):
    """A word lowercased and in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip().lower()).replace("’", "'")


def is_form(word):
    """Whether a word may be a form of the tables (D3, D4): an Italian token, and no word beginning
    with an elided piece — the pre-pass splits it, a hyphenated run included."""
    return bool(_TOKEN.fullmatch(word)) and not _ELIDED_START.match(word)


def last_letter(word):
    """The index of a word's last letter: a trailing apostrophe (`dà'`) is no letter."""
    return len(word) - 2 if word.endswith("'") and len(word) > 1 else len(word) - 1


def without_stress(word):
    """A table form with every stress mark but the last letter's dropped (D3): `dàre` → `dare`,
    `darèbbe` → `darebbe`, `darà` and `dà'` as they are."""
    end = last_letter(word)
    return "".join(_STRESS.get(c, c) if i != end else c for i, c in enumerate(word))


def bare(word):
    """A word with its last letter's stress mark dropped too (`hà` → `ha`), or None when it has none."""
    end = last_letter(word)
    if end < 0 or word[end] not in _STRESS:
        return None
    return word[:end] + _STRESS[word[end]] + word[end + 1 :]


def written_form(text, owners, form_links):
    """A row as Italian writes it (D3): its stress marks dropped but the last letter's
    (`without_stress`), and that one too when the entry carrying the cell's form-of link to one of
    `owners` (the lemma it is filed under, and the verb whose table it is) is the bare spelling —
    `hà` → `ha`, `dò` → `do` —, not when the marked spelling's own entry carries it (`dà`).
    `form_links` is `Survey.form_links`."""
    form = without_stress(text)
    plain = bare(form)
    if plain is None:
        return form
    if any(o in form_links.get(form, ()) for o in owners):
        return form
    if any(o in form_links.get(plain, ()) for o in owners):
        return plain
    return form


# A sense naming its entry a derived word of another (« diminutive of casa », « augmentative of
# zampa »): a meaning of its own, no form link — the word the row of D3 leaves out, read from its own
# entry (`casetta`, `minestrone`).
_DERIVED_SENSES = frozenset({"diminutive", "augmentative", "pejorative", "derogatory", "endearing"})


def _derived(sense):
    return bool(set(sense.get("tags") or ()) & _DERIVED_SENSES)


def _form_targets(sense):
    """The words a sense makes its entry a form of: each `form_of` target's first word (kaikki writes
    `ha`'s as « avere and », extra « (obsolete) havere »), and an apocope's `alt_of` target (`un`,
    « apocopic form of uno »; `buon`, `star`), which D6 reads as the dictionary's link — never a
    derived word's (`_DERIVED_SENSES`)."""
    if _derived(sense):
        return []
    refs = list(sense.get("form_of") or ())
    if "apocopic" in (sense.get("tags") or ()):
        refs += list(sense.get("alt_of") or ())
    out = []
    for ref in refs:
        words = (ref.get("word") if isinstance(ref, dict) else "") or ""
        if words.split():
            out.append(words.split()[0])
    return out


def _is_form_of(sense):
    return not _derived(sense) and ("form-of" in (sense.get("tags") or ()) or bool(_form_targets(sense)))


def _is_compound(sense):
    """A clitic compound's sense (« compound of the infinitive dire with me and lo »), the tag kaikki
    gives it or the gloss of the two it does not tag (`stammi`, `statemi`: « combined form of sta' …
    and mi »)."""
    if "compound-of" in (sense.get("tags") or ()):
        return True
    gloss = ((sense.get("glosses") or [""])[0] or "").lower()
    return gloss.startswith("combined form of")


class Survey:
    """What the reading of the forms needs to know of the whole section first (a first pass):

    - `verbs`: the words with a verb entry holding a meaning (neither a form-of nor a compound's);
    - `form_links`: word → the words its own entries' form-of senses name (`ha` → {avere}), for the
      tables' stress marks (D3);
    - `compounds`: the words an entry gives as a clitic compound (`dirmelo`, `vattene`, `fatti`);
    - `standing`: the words with an entry holding a meaning of another part of speech than a verb,
      an adverb, an interjection or a name (`fallo`, the foul), which stands beside a compound's — a
      name does not (`Donati`, beside the compound `donati`);
    - `clitic_lemmas`: the words with a verb entry holding a meaning, or a sense tagged reflexive or
      pronominal (`farsi`, « reflexive of fare »): the only words that can be an infinitive carrying
      clitics.
    """

    def __init__(self):
        self.verbs = set()
        self.form_links = collections.defaultdict(set)
        self.compounds = set()
        self.standing = set()
        self.clitic_lemmas = set()

    def read(self, entry):
        word = nfc_lower(entry.get("word"))
        pos = entry.get("pos")
        senses = entry.get("senses") or []
        meaning = any(not _is_form_of(s) and not _is_compound(s) for s in senses)
        if any(_is_compound(s) for s in senses):
            self.compounds.add(word)
        elif meaning and pos == "verb":
            self.verbs.add(word)
        elif meaning and pos not in _CLITIC_POS and pos != "name":
            self.standing.add(word)
        if pos == "verb" and not any(_is_compound(s) for s in senses):
            if meaning or any(set(s.get("tags") or ()) & {"reflexive", "pronominal"} for s in senses):
                self.clitic_lemmas.add(word)
        for sense in senses:
            if not _is_compound(sense):
                self.form_links[word].update(nfc_lower(t) for t in _form_targets(sense))

    def clitic_infinitive(self, word):
        """`(clitic, plain verb or None)` when `word` is an infinitive carrying clitics — a verb entry
        of the section holding a meaning or tagged reflexive or pronominal (`pentirsi`, `andarsene`,
        `lavarsi`), the plain verb being the one the section has (`lavarsi` → `lavare`, `porsi` →
        `porre`) —, else None: a finite form ending as one does (`persi`, `corsi`, `importi`, `firmi`,
        `torci`, `supporti`) is none."""
        m = _CLITIC_INFINITIVE.search(word)
        if not m or word not in self.clitic_lemmas:
            return None
        stem = word[: m.start(1)]
        plain = next((v for v in (stem + "e", stem + "re") if v in self.verbs), None)
        return m.group(1), plain

    def plain_of(self, word):
        """The plain verb a reflexive one is read as when the section has it (`lavarsi` → `lavare`),
        else the word: a reflexive verb whose plain verb the section has is no lemma (D3)."""
        found = self.clitic_infinitive(word)
        return found[1] if found and found[0] == "si" and found[1] else word


def survey_kaikki(path):
    """The section's first pass (`Survey`)."""
    survey = Survey()
    for entry in _entries(path):
        survey.read(entry)
    return survey


def _entries(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(entry, dict):
                yield entry


class Lexicon:
    """What the Italian section says of its words.

    - `candidates`: form → the words it may be a form of (its own word when it is a lemma);
    - `links`: form → candidate → the parts of speech of the entries linking them, which a form of a
      form follows (D3);
    - `lemmas`: the words with an entry that is not only a form of another;
    - `poses`: lemma → the parts of speech of its lemma entries;
    - `unranked`: the words no rank may go to: the verb forms carrying clitics (D4) and the
      infinitives carrying them (`lavarsi`, `andarsene`), whatever wordfreq says of them;
    - `stats`: what the reading counted, for the reduction's report.

    - `acronyms`: what the entries headed exactly as a row of Italian's acronym table give
      (keep-lingua-italian-acronyms-and-names D2), read for the table alone; each such entry is
      lowercased and read as any other.

    `spellings` is `dictionary_spellings`' table: a spelling it names is a form of its word, its
    inflections forms of it, nothing of its own (read-lingua-italian-unmarked-spellings D2). For the
    boundary of `WEB_SPELLINGS` (D3) it records, for every headword it reads, whether the section gives
    it a meaning of its own, and, for every row of the list, its entries' senses and the entries listing
    it among their inflections (`web_boundary`).
    """

    def __init__(self, survey, spellings=None):
        self.survey = survey
        self.spellings = dict(spellings or {})
        self.candidates = collections.defaultdict(set)
        self.links = collections.defaultdict(lambda: collections.defaultdict(set))
        self.lemmas = set()
        self.poses = collections.defaultdict(set)
        self.unranked = set()
        self.stats = collections.Counter()
        self.acronyms = acronyms.Pointers()
        # (form, word): a clitic-bearing row of a reflexive or pronominal verb, whose link to the verb
        # is dropped once the section is read (`drop_clitic_rows`).
        self._clitic_rows = set()
        self._meaning = set()
        self._web_entries = collections.defaultdict(list)
        self._web_listed = collections.defaultdict(list)

    def link(self, form, word, pos):
        self.candidates[form].add(word)
        self.links[form][word].add(pos)

    def read(self, entry):
        """One kaikki entry (D3, D4; read-lingua-italian-unmarked-spellings D2, D3)."""
        word = nfc_lower(entry.get("word"))
        pos = entry.get("pos")
        self.acronyms.read(entry)
        self._record_boundary(entry, word)
        if word in PIECES or (word in CONTRACTIONS and word != DAI) or not is_form(word):
            # A piece is read by its table alone (`seed_pieces`); a split contraction is no form.
            return
        senses = entry.get("senses") or []
        if word in self.spellings:
            # A spelling the dictionary gives only as its word's (D2): a form of the word, its own
            # inflections forms of it, nothing of its own.
            self.link(word, self.spellings[word], pos)
            for form in self._rows(entry, word, (word,)):
                self.link(form, word, pos)
            return
        if any(_is_compound(s) for s in senses) or (word in self.survey.compounds and pos in _CLITIC_POS):
            # A clitic compound's entries read the compound: the enclitic rule reads it (D4).
            if word not in self.survey.standing:
                self.unranked.add(word)
            return
        clitic = self.survey.clitic_infinitive(word) if pos == "verb" else None
        if clitic is not None:
            self._read_clitic_verb(entry, word, *clitic)
            return
        if not (senses and all(_is_form_of(sense) for sense in senses)):
            self.lemmas.add(word)
            self.link(word, word, pos)
            self.poses[word].add(pos)
        # A name's rows are no forms of a word (D3): its demonyms, its other spellings.
        if pos != "name":
            for form in self._rows(entry, word, (word,)):
                self.link(form, word, pos)
        for sense in senses:
            for target in _form_targets(sense):
                target = nfc_lower(target)
                if self._carries_clitics(word, target, sense):
                    # No form and no rank, as a compound's (D4), but where a meaning stands beside.
                    self.stats["forms carrying clitics their own entry links"] += 1
                    if word not in self.survey.standing:
                        self.unranked.add(word)
                    continue
                target = self.survey.plain_of(target)
                if target in WEB_SPELLINGS or target in OWNER_EXCEPTIONS:
                    # An entry pointing at a row of the web's list follows the row
                    # (read-lingua-italian-unmarked-spellings D3).
                    self.link(word, web_spelling(target), pos)
                elif is_form(target) and target not in PIECES:
                    self.link(word, target, pos)

    def _record_boundary(self, entry, word):
        """What `web_boundary` needs of one entry: each sense's kind (`_sense_kind`), whether the entry
        gives its word a meaning, and, for a row of the web's list, its senses and the entries listing
        it among their inflections — a row a source marks doubtful is no inflection (D3). A name's entry
        says nothing of the word (`Gia`, `Sara`): a page writes a name with its capital, which the
        analysis sets aside, as change 43e kept its rows that are also names (`noel`)."""
        name = entry.get("pos") == "name"
        kinds = [] if name else [_sense_kind(sense) for sense in entry.get("senses") or ()]
        meaning = any(kind == "meaning" for kind, _ in kinds)
        if meaning:
            self._meaning.add(word)
        if word in WEB_SPELLINGS:
            self._web_entries[word].append(kinds)
        targets = tuple(target for kind, target in kinds if kind == "form" and target)
        for row in entry.get("forms") or ():
            if set(row.get("tags") or ()) & (_BOOKKEEPING | _DOUBTFUL):
                continue
            form = without_stress(nfc_lower(row.get("form")))
            if form != word and form in WEB_SPELLINGS:
                self._web_listed[form].append((meaning, targets))

    def web_boundary(self):
        """The rows of `WEB_SPELLINGS` the section names (read-lingua-italian-unmarked-spellings D3),
        sorted: a headword with a sense that is a meaning, a form of a word with a meaning, or a
        spelling of a word written with the same letters (`_unmarked`: D2's kind, `perche` « misspelling
        of perché »), or one D2 reads as a word (`spellings`); or an inflection an entry lists whose
        word has a meaning or is a form of one. Read after the section: the reduction fails on any, for
        a person to settle it. A spelling named only as a form of a word without a meaning of its own,
        or as a spelling of another word D2 does not read (`giu`, « alternative form of giu. », the
        abbreviation of *giugno*), passes, as change 43e's do, and so does one only a name's entry heads
        (`gia`, `sara`: *Gia*, *Sara*); the owner's exceptions are not checked."""
        crossed = []
        for spelling in sorted(set(self._web_entries) | set(self._web_listed)):
            headword = spelling in self.spellings or any(
                kind == "meaning"
                or (kind == "form" and target in self._meaning)
                or (kind == "alt" and _unmarked(target) == _unmarked(spelling))
                for kinds in self._web_entries.get(spelling, [])
                for kind, target in kinds
            )
            listed = any(
                meaning or any(target in self._meaning for target in targets)
                for meaning, targets in self._web_listed.get(spelling, [])
            )
            if headword or listed:
                crossed.append(spelling)
        return crossed

    def _carries_clitics(self, word, target, sense):
        """Whether a form-of sense names a verb form carrying clitics (D4): its target an infinitive
        carrying them (`pentirsi`, `recarsi`, `andarsene`), the sense one of the four moods a clitic is
        joined to — an infinitive, a gerund, an imperative, a participle —, the form ending in a clitic
        (`pentirmi`, `avvalendomi`, `recatevi`, `rechiamoci`, `andarmene`). The cascade's enclitic rule
        reads it; a bare form (`ostinato`, the participle of *ostinarsi*) is no such form, and the bare
        forms of the verb's rows come in through them (`_read_clitic_verb`)."""
        if self.survey.clitic_infinitive(target) is None:
            return False
        if not set(sense.get("tags") or ()) & {"infinitive", "gerund", "imperative", "participle"}:
            return False
        return bool(_CLITIC_TAIL.search(word))

    def _read_clitic_verb(self, entry, word, clitic, plain):
        """An infinitive carrying clitics (D3, D4). A reflexive (`si`) is filed under its plain verb
        where the section has one (`lavarsi` → *lavare*: no lemma of its own) and is its own lemma
        otherwise (`pentirsi`); its rows written with their pronoun give their bare form (`mi làvo`
        → `lavo`), and its rows carrying the pronoun (`lavàndosi`, `pentìtevi`) give their bare base
        under a reflexive-only lemma (`pentendo`, `pentite`; the infinitive's `pentire`) and nothing
        under a plain verb. Any other clitic (`andarsene`, `farcela`, `esserci`) is no lemma and its
        rows give no form: a page writes the cluster's words apart."""
        self.unranked.add(word)
        self.stats["infinitives carrying clitics"] += 1
        reflexive = clitic == "si"
        lemma = (plain or word) if reflexive else None
        if reflexive and plain is None:
            self.lemmas.add(word)
            self.link(word, word, "verb")
            self.poses[word].add("verb")
            self.unranked.discard(word)
            self.stats["reflexive-only lemmas"] += 1
        elif reflexive:
            self.stats["reflexives filed under their plain verb"] += 1
        for row in entry.get("forms") or ():
            tags = set(row.get("tags") or ())
            text = nfc_lower(row.get("form"))
            if tags & _BOOKKEEPING or tags & _DOUBTFUL or tags & _DERIVED or not text:
                continue
            words = text.split()
            if len(words) == 2 and words[0] in REFLEXIVE_PRONOUNS and lemma is not None:
                form = self._written(words[1], (lemma, word))
                if is_form(form) and form not in PIECES:
                    self.link(form, lemma, "verb")
                    self.stats["reflexive rows read bare"] += 1
                continue
            if len(words) != 1:
                continue
            form = self._written(text, (lemma or word, word))
            if not is_form(form):
                continue
            self._clitic_rows.add((form, plain or word))
            self._clitic_rows.add((form, word))
            tail = _REFLEXIVE_TAIL.search(form)
            if not reflexive or plain is not None or not tail:
                continue
            base = form[: tail.start()]
            if base.endswith("r"):
                base += "e"
            if len(base) > 2 and is_form(base):
                self.link(base, word, "verb")
                self.stats["bases of a reflexive-only verb's rows"] += 1

    def drop_clitic_rows(self):
        """A reflexive or pronominal verb's rows carrying a clitic are no form of it, nor of its plain
        verb (D4): `lavatosi`, `pentendosi`, `pentiti` (« pentiti! ») lose the link, a reading of
        their own standing (`pentiti`, the plural of `pentito`). The reflexive-only lemma's own
        infinitive stays its form."""
        for form, word in self._clitic_rows:
            if form != word and word in self.candidates.get(form, ()):
                self.candidates[form].discard(word)
                self.links[form].pop(word, None)
                self.stats["clitic rows unlinked"] += 1
        for form in [f for f, opts in self.candidates.items() if not opts]:
            del self.candidates[form]

    def spelling_paradigms(self):
        """The dictionary spellings' paradigms (read-lingua-italian-unmarked-spellings D2, as
        implemented): the spellings, and every form all whose candidates lie inside them, again until
        nothing joins — `accidere` (« alternative form of uccidere »), its participle `acciso`, whose
        one word is `accidere`, and so on. A form with a word of its own beside is none (`accise`, also
        *accisa*'s plural)."""
        paradigm = set(self.spellings)
        grown = True
        while grown:
            grown = False
            for form, opts in self.candidates.items():
                if form not in paradigm and opts and opts <= paradigm:
                    paradigm.add(form)
                    grown = True
        return paradigm

    def drop_spelling_rows(self):
        """A form of a dictionary spelling's paradigm (`spelling_paradigms`) follows the spelling to its
        word only where no other word claims it (read-lingua-italian-unmarked-spellings D2, as
        implemented, transitive at review): `sguardi`, a row of the archaic `sguardare` (« alternative
        form of guardare »), stays *sguardo*'s plural, `annate` *annata*'s, and `accise` *accisa*'s, a
        row of the participle `acciso` of the archaic `accidere` — the choice among a form's words (D5)
        would give each to the commoner verb by its frequency. A form no other word claims follows
        (`alcoolici` → *alcolico*); another spelling's paradigm is no other word."""
        paradigm = self.spelling_paradigms()
        for form, opts in self.candidates.items():
            inside = opts & paradigm
            if form in paradigm or not inside or not opts - paradigm:
                continue
            for member in inside:
                opts.discard(member)
                self.links[form].pop(member, None)
            self.stats["forms of a dictionary spelling another word claims"] += 1

    def _written(self, text, owners):
        """A row as Italian writes it (D3, `written_form`), counted when its last letter's mark is
        read bare."""
        form = written_form(text, owners, self.survey.form_links)
        if form != without_stress(text):
            self.stats["final marks read bare"] += 1
        return form

    def _rows(self, entry, word, owners):
        """The inflections an entry lists that are forms of its word (D3): never the bookkeeping, a
        doubtful row, a derived word's row, a word that is no form, a piece or a split contraction, nor
        a gender or number marker but the word's own; each written as Italian writes it."""
        for row in entry.get("forms") or ():
            tags = set(row.get("tags") or ())
            if tags & _BOOKKEEPING or tags & _DOUBTFUL or tags & _DERIVED:
                continue
            text = nfc_lower(row.get("form"))
            if not text or " " in text:
                continue
            form = self._written(text, owners)
            if not is_form(form) or form in PIECES or (form in CONTRACTIONS and form != DAI):
                continue
            if form in _MARKERS and form != word:
                continue
            yield form


def read_kaikki(path, survey=None, spellings=None, readings=None):
    """The Italian section, read entry by entry into a `Lexicon`, after its first pass (`Survey`), the
    dictionary's spellings `spellings` names read as their word (`dictionary_spellings`) — and, given
    `readings` (`Readings`), each entry's grammar in the same pass (`read_readings`,
    add-lingua-italian-grammar-tables D1)."""
    survey = survey or survey_kaikki(path)
    lexicon = Lexicon(survey, spellings)
    for entry in _entries(path):
        lexicon.read(entry)
        if readings is not None:
            read_readings(entry, readings, survey, spellings)
    lexicon.drop_clitic_rows()
    lexicon.drop_spelling_rows()
    return lexicon


def _alt_target(sense):
    """What a sense's `alt_of` names first, lowercased and in NFC — a word, or several (`vabbè`,
    « alternative form of va bene ») —, or an empty string."""
    alt = sense.get("alt_of") or ()
    return nfc_lower((alt[0].get("word") if alt and isinstance(alt[0], dict) else "") or "")


def spelling_target(sense):
    """The word a sense gives its entry only as a spelling of
    (read-lingua-italian-unmarked-spellings D2) — an `alt_of` pointer at one word, whatever it says of
    the spelling (a misspelling, an alternative, obsolete, archaic or nonstandard spelling or form) but
    an apocope, an abbreviation, a clipping, a contraction, an initialism or a pronunciation spelling
    (`_NO_SPELLING`) —, or None. A pointer at several words (`vabbè`, `menomale`, `daccapo`) spells
    no word: the sense is a meaning of the entry's own."""
    target = _alt_target(sense)
    if set(sense.get("tags") or ()) & _NO_SPELLING or " " in target:
        return None
    return target or None


def _reviewed(word):
    """Whether a word is a row of a reviewed table, which that table alone reads: a piece, a split
    contraction, an apocope, the `di` family, an override, a row of the web's list or an exception."""
    return (
        word in PIECES
        or word in CONTRACTIONS
        or word in APOCOPES
        or word in DI_FAMILY
        or word in OVERRIDES
        or word in WEB_SPELLINGS
        or word in OWNER_EXCEPTIONS
    )


def _row_forms(entry):
    """The inflections an entry lists, as the forms guard reads them (D2): one word each, never the
    bookkeeping, a doubtful or a derived word's row, its stress marks read as `Lexicon._written` reads
    them before the bare spelling (`without_stress`)."""
    for row in entry.get("forms") or ():
        tags = set(row.get("tags") or ())
        text = nfc_lower(row.get("form"))
        if text and " " not in text and not tags & (_BOOKKEEPING | _DOUBTFUL | _DERIVED):
            yield without_stress(text)


def dictionary_spellings(path, frequency, kept=None):
    """`spelling → word` (read-lingua-italian-unmarked-spellings D2): each word every sense of every
    entry of which the section gives as a spelling of one word (`spelling_target`) — or as a form of
    that word —, which wordfreq rates strictly commoner than the spelling and than each of the
    spelling's forms. Change 43d's rule for French, without its condition that the spelling be the word
    with its marks dropped, on purpose: Italian's dictionary spellings are mostly another accent
    (`perchè` → *perché*, `caffé` → *caffè*, `fà` → *fa*) or a letter (`sopratutto` → *soprattutto*).

    Read per word, in a pass of its own over the section before the forms, each word lowercased and in
    NFC: a word with a sense of its own beside the spelling (`e'`, also the apocope of *ei*), or a form
    of another word, is no spelling. A pointer at one of change 6's pieces is read by the piece's table
    alone and is no word of the spelling's (`fà`, « misspelling of fa » and « misspelling of fa' »,
    spells `fa`); a spelling whose senses spell two words is kept apart (`nè`: *né*, and *neh*). The
    word must be strictly commoner than the spelling and than each of the spelling's forms — the
    inflections its entries list (`_row_forms`) and the words a form-of sense of which points at it. A
    row of a reviewed table is that table's (`_reviewed`), a word the owner keeps apart by name
    (`SPELLINGS_KEPT_APART`: `casinò`, `omg`, `cè`) reads as before, and a word that is no form has no
    form to read. With `kept` (a dict), each spelling kept apart is recorded there with its reason
    (`KEPT_*`)."""
    spells = collections.defaultdict(set)
    formed = collections.defaultdict(set)
    own = set()
    forms = collections.defaultdict(set)
    for entry in _entries(path):
        word = nfc_lower(entry.get("word"))
        senses = entry.get("senses") or []
        if not senses:
            own.add(word)
        for sense in senses:
            target = spelling_target(sense)
            targets = [nfc_lower(t) for t in _form_targets(sense)] if _is_form_of(sense) else []
            if target:
                spells[word].add(target)
            elif targets:
                formed[word].update(targets)
            else:
                own.add(word)
            for t in targets:
                forms[t].add(word)
        forms[word].update(_row_forms(entry))
    table = {}
    for word in sorted(spells):
        if word in own or formed[word] - spells[word]:
            continue
        targets = {t for t in spells[word] if t != word and t not in PIECES}
        if _reviewed(word):
            why = KEPT_TABLE
        elif word in SPELLINGS_KEPT_APART:
            why = KEPT_OWNER
        elif len(targets) > 1:
            why = KEPT_TWO_WORDS
        elif not targets or not is_form(word) or not is_form(next(iter(targets))):
            why = KEPT_NO_FORM
        else:
            (target,) = targets
            if not frequency(target) > frequency(word):
                why = KEPT_COMMONER
            elif any(frequency(form) >= frequency(target) for form in forms[word] - {word, target}):
                why = KEPT_FORM
            else:
                table[word] = target
                continue
        if kept is not None:
            kept[word] = why
    return table


def spellings_report(spellings, kept):
    """The reduction's lines on the dictionary's spellings (D2): each one read, with its word, and each
    one kept apart, by the test that kept it."""
    lines = [
        f"dictionary spellings read as their word={len(spellings)}: "
        + ", ".join(f"{s} → {w}" for s, w in sorted(spellings.items()))
    ]
    for why in (KEPT_COMMONER, KEPT_FORM, KEPT_TWO_WORDS, KEPT_NO_FORM, KEPT_TABLE, KEPT_OWNER):
        words = sorted(w for w, reason in kept.items() if reason == why)
        lines.append(f"dictionary spellings kept apart, {why}={len(words)}: " + ", ".join(words))
    return lines


def name_or_word(candidates, poses, frequency):
    """The candidates of a form whose own lemma entries are names alone and which is another word's
    too (D5), as French's and Spanish's `names` are read: it reads as the other word, the name left
    out (`bambini`, a surname, → *bambino*; `firmato`, a given name, → *firmare*). wordfreq counts the
    string, which is that word's inflection too, and cannot rate the name apart; and a page writes a
    name with its capital, which the analysis sets aside as a name (add-lingua-italian-analysis D4).
    Every other form is D5's — one with a common reading of its own beside a name (`sole`, `santo`,
    `ti`) included —, and a name no other word reads stays (`roma`). The other words are those that are
    no name alone. Answers how many forms it moved. `frequency` is unused, kept for the call's shape."""
    moved = collections.Counter()
    for form, opts in candidates.items():
        if poses.get(form) != {"name"} or form not in opts:
            continue
        if any(poses.get(w) != {"name"} for w in opts - {form}):
            opts.discard(form)
            moved["names read as another word"] += 1
    return moved


def _sense_kind(sense):
    """A sense's kind for `Lexicon.web_boundary`: (`alt`, the word it spells, `spelling_target`),
    (`form`, the first word it is a form of — empty when only its tag says so —) or (`meaning`,
    None)."""
    target = spelling_target(sense)
    if target:
        return "alt", target
    if _is_form_of(sense):
        targets = _form_targets(sense)
        return "form", nfc_lower(targets[0]) if targets else ""
    return "meaning", None


def _unmarked(word):
    """A word with its combining marks dropped — Italian's accents, and a loan's marks —, in NFC:
    `perché` → `perche`, `perchè` → `perche`."""
    bare = "".join(c for c in unicodedata.normalize("NFD", word) if unicodedata.category(c) != "Mn")
    return unicodedata.normalize("NFC", bare)


def web_spelling(spelling):
    """The word a row of the web's list or an owner's exception reads as (D3), or None."""
    if spelling in WEB_SPELLINGS:
        return WEB_SPELLINGS[spelling]
    if spelling in OWNER_EXCEPTIONS:
        return OWNER_EXCEPTIONS[spelling][0]
    return None


def read_web_spellings(lexicon):
    """Each row of the web's list and each owner's exception a form of its word alone (D3), whether or
    not an entry names it, as `read_apocopes` reads its table: the word its one candidate, the spelling
    no lemma — the exceptions' own entries (`perche`, *perca*'s plural) leave the tables."""
    for spelling in (*WEB_SPELLINGS, *OWNER_EXCEPTIONS):
        lexicon.candidates[spelling] = {web_spelling(spelling)}
        lexicon.links.pop(spelling, None)
        lexicon.lemmas.discard(spelling)


def read_apocopes(lexicon):
    """Each apocope of the reviewed table a form of its word alone (D6), whatever the section's entries
    make of it (`bel` → *bello*, `san` → *santo*): the word its one candidate, the form no lemma."""
    for form, (lemma, _) in APOCOPES.items():
        lexicon.candidates[form] = {lemma}
        lexicon.links.pop(form, None)
        lexicon.lemmas.discard(form)


def own_words(lexicon):
    """The `di` family, each a lemma of its own whatever its entries point at (I7, D4)."""
    for word in DI_FAMILY:
        lexicon.candidates[word] = {word}
        lexicon.lemmas.add(word)


def choose_lemma(form, options, counts, lemmas, frequency, overrides=OVERRIDES):
    """The one lemma a form maps to (D5, M8), among `options`: an override first, then MarkIT's
    counts, then the form's own entry, then the lemma's frequency, then alphabetical order — so the
    result never depends on the source's order."""
    if not options:
        return None
    override = overrides.get(form)
    if override and override[0] in options:
        return override[0]
    return min(options, key=lambda lemma: _order(form, lemma, counts, lemmas, frequency))


def _order(form, lemma, counts, lemmas, frequency):
    return (-counts.get((form, lemma), 0), not (lemma == form and form in lemmas), -frequency(lemma), lemma)


def decided_by(form, options, counts, lemmas, frequency, overrides=OVERRIDES):
    """Which rule of D5 chose a form's lemma among several (`DECIDED_BY`), for the report."""
    override = overrides.get(form)
    if override and override[0] in options:
        return DECIDED_BY[0]
    ranked = sorted(_order(form, lemma, counts, lemmas, frequency) for lemma in options)
    first, second = ranked[0], ranked[1]
    for rule, (a, b) in zip(DECIDED_BY[1:], zip(first, second)):
        if a != b:
            return rule
    return DECIDED_BY[-1]


def follow(word, pos, links, poses, choose, overrides=OVERRIDES):
    """What `word` reads as along one part of speech (D3), as French's `follow`: an override's lemma
    when a person decided the word; otherwise the first choice (D5) among itself, when one of its lemma
    entries is of that part of speech, and the words its entries of that part of speech link it to —
    but a word whose entries are all of other parts of speech. `word` itself when nothing is left."""
    if word in overrides:
        return overrides[word][0]
    along = {
        w
        for w, of in links.get(word, {}).items()
        if pos in of and w != word and (not links.get(w) or any(pos in p for p in links[w].values()))
    }
    if pos in poses.get(word, ()):
        along.add(word)
    return choose(word, along) if along else word


def reach(form, options, links, step):
    """A form's lemmas (D3): each candidate followed along the part of speech of the entry linking the
    form to it; `creduta`, the feminine of the participle `creduto` (a verb entry), itself the past
    participle of *credere*, reaches *credere*. The form's own entry, and a candidate no entry links
    it to (`del`), stand as they are. `step(word, pos)` is `follow` over the lexicon."""
    reached = set()
    for candidate in options:
        linked = links.get(form, {}).get(candidate, set())
        if candidate == form or not linked:
            reached.add(candidate)
            continue
        for pos in linked:
            word = candidate
            for _ in range(_FOLLOW_STEPS):
                nxt = step(word, pos)
                if nxt == word:
                    break
                word = nxt
            reached.add(word)
    return reached


def ranks_for(inflected, want, lemmas, compounds_zipf, frequency, top_n, unranked=frozenset()):
    """`lemma → rank`, dense, the `want` first kept (D6).

    wordfreq's order over its words that are Italian forms, neither its stems of the pieces
    (`WORDFREQ_STEMS`) nor an articulated preposition the pre-pass splits (`dai` among them: no rank),
    nor a word `inflected` reads as another's, nor one `unranked` names (a verb form or infinitive
    carrying clitics). A hyphenated lemma wordfreq cannot rank is ranked when MarkIT attests it
    (`compounds_zipf`), at the lower of wordfreq's estimate and MarkIT's own frequency, after
    wordfreq's words of the same frequency.
    """
    order, seen = [], set()
    for word in top_n(want * _WORDFREQ_DEPTH):
        word = nfc_lower(word)
        if word in WORDFREQ_STEMS or word in CONTRACTIONS or word in seen or word in unranked:
            continue
        if not is_form(word) or word in inflected:
            continue
        seen.add(word)
        order.append((frequency(word), word))
    floor = order[min(len(order), want) - 1][0] if order else 0
    for lemma in sorted(lemmas):
        if "-" not in lemma or lemma in seen or lemma in inflected or lemma in unranked:
            continue
        if lemma in compounds_zipf:
            estimate = min(frequency(lemma), compounds_zipf[lemma])
            if estimate >= floor:
                order.append((estimate, lemma))
    # A stable sort: wordfreq's own order among its words, the compounds after them at a tie.
    order.sort(key=lambda kept: -kept[0])
    return {word: rank for rank, (_, word) in enumerate(order[:want], 1)}


def forms_for(lexicon, step, ranks, counts, frequency, overrides=OVERRIDES):
    """`form → lemma` over the ranked lemmas (D3–D6): each form's choice among the ranked lemmas it
    reaches; a split contraction left out; a form kept when it is its lemma's own or attested by
    wordfreq; and each ranked lemma's own form. A clitic compound has no candidate of its own (its
    entries read the compound), so it is a form only where another word's row lists it (`fatti`,
    the plural of `fatto`)."""
    forms = {}
    for form, options in lexicon.candidates.items():
        if form in CONTRACTIONS and form != DAI:
            continue
        options = reach(form, options, lexicon.links, step)
        lemma = choose_lemma(form, {o for o in options if o in ranks}, counts, lexicon.lemmas, frequency, overrides)
        if lemma is None:
            continue
        if lemma == form or frequency(form) > 0:
            forms[form] = lemma
    for lemma in ranks:
        forms.setdefault(lemma, lemma)
    return forms


def reduce_forms(lexicon, counts, compounds_zipf, frequency, top_n, max_lemmas, overrides=OVERRIDES):
    """The forms table and the ranks: `form → lemma` over the kept lemmas, and `lemma → rank`.

    A first choice among every candidate says which forms are only inflected: they are never ranked.
    The ranks follow; a ranked word whose own form reads as another word once the forms of forms are
    followed gives its rank to the next, until every ranked lemma's own form reads as itself (D6): a
    pack finds a lemma by its own form. Answers `(forms, ranks, step)`, `step` being the chain the
    choice followed, for the report.
    """
    first = {
        form: choose_lemma(form, opts, counts, lexicon.lemmas, frequency, overrides)
        for form, opts in lexicon.candidates.items()
    }
    inflected = {form for form, lemma in first.items() if lemma != form}
    steps = {}

    def choose(word, options):
        return choose_lemma(word, options, counts, lexicon.lemmas, frequency, overrides)

    def step(word, pos):
        if (word, pos) not in steps:
            steps[(word, pos)] = follow(word, pos, lexicon.links, lexicon.poses, choose, overrides)
        return steps[(word, pos)]

    unreached = set()
    while True:
        ranks = ranks_for(
            inflected | unreached, max_lemmas, lexicon.lemmas, compounds_zipf, frequency, top_n, lexicon.unranked
        )
        forms = forms_for(lexicon, step, ranks, counts, frequency, overrides)
        lost = {lemma for lemma in ranks if forms.get(lemma) != lemma}
        if not lost:
            return forms, ranks, step
        unreached |= lost


def seed_pieces(forms):
    """Each elided piece and truncation a form of the word it stands for, read as that word's lemma
    (D4): `l'` → *lo*, `un'` → *uno* (`una`'s), `dell'` → *dello*, `po'` → *poco*, `va'` → *andare*
    (`vai`'s). Answers the pieces whose word is no form, which the reduction refuses."""
    missing = []
    stands_for = {**ELISIONS, "dell'": "dello", **TRUNCATIONS}
    for piece, word in sorted(stands_for.items()):
        if word in forms:
            forms[piece] = forms[word]
        else:
            missing.append(piece)
    return missing


def prepass_words():
    """Every word Italian's pre-pass writes (D4): the elided pieces' words, `dissi`, the split
    articulated prepositions' and elided prepositions' pieces, the `di` family and the truncations'
    words."""
    words = set(ELISIONS.values()) | set(CONTEXT_WORDS) | set(TRUNCATIONS.values()) | set(DI_FAMILY)
    for pieces in (*CONTRACTIONS.values(), *ELIDED_PREPOSITIONS.values()):
        words.update(pieces)
    return sorted(words)


def readings_without_spellings(kaikki, survey, counts, frequency, top_n, max_lemmas):
    """The forms the tables read without the dictionary's spellings (D2) — every other rule as the
    reduction applies it —: what a form read before D2, for `foreign_readings`."""
    lexicon = read_kaikki(kaikki, survey)
    name_or_word(lexicon.candidates, lexicon.poses, frequency)
    own_words(lexicon)
    read_apocopes(lexicon)
    read_web_spellings(lexicon)
    forms, _, _ = reduce_forms(lexicon, counts, treebank.markit_zipf(counts), frequency, top_n, max_lemmas)
    return set(forms)


def foreign_readings(forms, paradigm, before, frequency, foreign, names):
    """The forms only a dictionary spelling reads (read-lingua-italian-unmarked-spellings D2, at
    review): a form of a spelling's paradigm the tables did not read before D2 (`before`) is another
    language's word when English, French or Spanish writes it more often than Italian does
    (`foreign`, D3's test: `intro`, `secure`, `provide`, `spare`, `surge`, `antique`, `ata`), or a
    name when a source says so (`names`: the section's name entries, MarkIT's proper nouns,
    Morph-it!'s `NPR` — `abbado`, `ati`, `bucci`): it keeps no reading, as before. Sorted."""
    return sorted(
        form
        for form in forms
        if form in paradigm and form not in before and (foreign(form) > frequency(form) or form in names)
    )


def unread_web_spellings(forms):
    """The rows of the web's list and the owner's exceptions the forms table does not read as their
    word (D3) — their word no ranked lemma: the reduction refuses them, a row following the tables'
    choices and never making one."""
    return sorted(s for s in (*WEB_SPELLINGS, *OWNER_EXCEPTIONS) if forms.get(s) != web_spelling(s))


def unwritten_words(forms):
    """The words the pre-pass writes that the forms table does not list: the reduction refuses them
    (D4) — every one must be a form."""
    return [word for word in prepass_words() if word not in forms]


# — Italian's word grammar (add-lingua-italian-grammar-tables, the Italian programme's change 13) —
#
# Every form kaikki lists carries its grammar as tags (`parlavo`: first-person, imperfect,
# indicative, singular). They become Universal Dependencies tags, the vocabulary the pack's grammar
# sections read (add-lingua-word-grammar D1): `VERB|Mood=Ind|Number=Sing|Person=1|Tense=Imp|VerbForm=Fin`.
# The rules read the same entries as the forms, in the same pass (`read_kaikki`), each table form
# written as the forms read it (`written_form`), and the forms and ranks once chosen (design D1, D2):
# they choose neither. French's rules (reduce-fr-en.py, add-lingua-french-grammar-tables), for Italian.

# kaikki's parts of speech whose forms carry readings, as UPOS: an article is a determiner, as UD
# writes it (D4); every verb is `VERB`, `AUX` being a role in a sentence a card cannot see (D3).
_UPOS = {"verb": "VERB", "noun": "NOUN", "adj": "ADJ", "det": "DET", "article": "DET", "pron": "PRON", "num": "NUM"}
# kaikki's moods and tenses, in the order they are read; the passato remoto is `historic` (D3).
_MOODS = (("conditional", "Cnd"), ("imperative", "Imp"), ("subjunctive", "Sub"))
_TENSES = (("historic", "Past"), ("imperfect", "Imp"), ("future", "Fut"), ("present", "Pres"))
_PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
_GENDER = {"masculine": "Masc", "feminine": "Fem"}
# The imperative's cells (D3): the second person singular, the first and second plural — the third
# persons are the formal imperative's, the present subjunctive's.
_IMPERATIVE_CELLS = frozenset({("2", "Sing"), ("1", "Plur"), ("2", "Plur")})
# `it-noun`'s gender argument, each piece of it (`m`, `f,m`, `m-p`): the genders a noun takes.
_NOUN_GENDER = {"m": ("Masc",), "f": ("Fem",), "mf": ("Masc", "Fem"), "mfbysense": ("Masc", "Fem"), "mfequiv": ("Masc", "Fem")}
# A determiner's, pronoun's or numeral's row reads only when it says nothing but gender, number and
# degree: a personal pronoun's head lists its other persons and cases (`io`: possessive `mio`),
# which are other words, not inflections (D4).
_NOMINAL_ONLY = frozenset(
    {"masculine", "feminine", "singular", "plural", "superlative", "comparative", "form-of", "invariable"}
)
# A row or sense that gives no reading (D5): the tables' bookkeeping, the tags change 9 leaves out of
# the forms (a form a source marks doubtful is no inflection, so no reading either), and what kaikki
# could not parse (`essere`'s `són`, `siàm`).
_NOT_A_READING = _BOOKKEEPING | _DOUBTFUL | {"error-unrecognized-form"}
# The rows that repeat another cell (D3): the negative imperative (its second person singular is the
# infinitive, « non parlare », its plurals the affirmative's) and the formal imperative (`Lei`,
# `Loro`: the present subjunctive's third persons).
_REPEATED_CELLS = frozenset({"negative", "formal", "second-person-semantically"})
# The regions a sense or a row may be marked with: a word whose every sense is one region's is named
# on no standard card (D6).
_REGIONS = frozenset(
    {
        "regional", "dialectal", "Tuscany", "Italy", "Northern", "Southern", "Central", "Eastern", "South",
        "Switzerland", "Roman", "Rome", "Naples", "Lombardy", "Milan", "Bologna", "Genoa", "Sicily", "Venice",
        "Sardinia", "Australia", "US", "UK", "New-Zealand",
    }
)
# A form's own sense, or a table's row, marked as a region's or a register's reads nothing (D3, D5):
# the dialectal and hypercorrect rows (`fò`, `peròro`), a regional sense.
_REGISTERS = _REGIONS | {"slang", "colloquial", "informal", "Internet", "vulgar", "hypercorrect"}
# A row or a form's own sense of the language's poetry and letters (D3, D6): read as its cell, it reads
# under the lemma the forms file the form under, and never names another word — `sono`, a poetic
# Latinism of *suonare*'s present, names no *suonare*, `possente` (*potere*'s literary participle) no
# *potere*, while `veggo` and `vegga` stay *vedere*'s.
_LITERARY = frozenset({"poetic", "Latinism", "literary"})
# A lemma entry whose every meaning is marked archaic or of poetry and letters holds its word as no
# part of speech a card names (D6): `diro` (« dire », literary) is named on no card of `dire`,
# `ideo` on none of `idea`, *cano* on none of `cane`. A rare or dated meaning still names its word
# (`arreso`, *arrendere*'s participle).
_MARKED_MEANINGS = _LITERARY | {"archaic"}
# A determiner's or pronoun's head naming a plural headword (`gli`, `le`), whose table lists the
# singular (D4).
_PLURAL_HEADS = frozenset({"p", "m-p", "f-p", "mf-p", "mfbysense-p"})
# The rows of an entry's table: what follows its first bookkeeping row; the rows before it are the
# head's (its principal parts, or the forms an article's head names).
_TABLE_START = frozenset({"table-tags", "inflection-template"})
# A present indicative's first person plural, the cell that tells a dialect's table from Italian's
# (`_dialect_rows`).
_FIRST_PLURAL_PRESENT = frozenset({"first-person", "indicative", "plural", "present"})


def ud_tag(upos, features):
    """A Universal Dependencies tag: the part of speech, then the features sorted by name."""
    return "|".join([upos, *(f"{name}={value}" for name, value in sorted(features.items()))])


def parse_tag(tag):
    """`ud_tag`'s inverse: the part of speech and the features."""
    upos, *features = tag.split("|")
    return upos, dict(feature.split("=", 1) for feature in features)


def _agreement(tags):
    """Gender and number, as a participle's or an adjective's tags state them."""
    out = {}
    genders = [_GENDER[tag] for tag in tags if tag in _GENDER]
    if len(genders) == 1:
        out["Gender"] = genders[0]
    if "plural" in tags:
        out["Number"] = "Plur"
    elif "singular" in tags:
        out["Number"] = "Sing"
    return out


def _past_participle(tags):
    """A past participle's features: the table's bare row is the masculine singular (D3)."""
    agreement = _agreement(tags)
    return {
        "VerbForm": "Part",
        "Tense": "Past",
        "Gender": agreement.get("Gender", "Masc"),
        "Number": agreement.get("Number", "Sing"),
    }


def _present_participle(tags):
    """A present participle's features (D3): its number, the table's row the singular (`parlante`),
    its own entry's plural row the plural (`parlanti`), as UD Italian writes it; no gender, one form
    serving both."""
    return {"VerbForm": "Part", "Tense": "Pres", "Number": "Plur" if "plural" in tags else "Sing"}


def verb_features(tags):
    """Every reading a verb's row or sense names (D3), as feature sets, none when it names none.

    - The passato remoto (`historic past`) is the indicative past, the imperfetto the imperfect; the
      conditional and the imperative take no tense, as Spanish's and French's tables write them (D7).
    - The gerund is `VerbForm=Ger`; a present participle `VerbForm=Part|Tense=Pres` with its number; a
      past participle reads with its agreement, the bare row and a sense that names an agreement
      without `past` (`creduta`: « feminine singular of creduto ») as the past one.
    - A sense merging persons, numbers or moods (`parli`: « first/second/third-person singular present
      subjunctive ») reads as each of them; a table's row names one; a row with a tense and no mood
      (the head's `èro`) is the indicative.
    - The imperative has its second person singular and its first and second plural alone (the third
      persons are the formal imperative's); the subjunctive no future or passato remoto.
    """
    if "participle" in tags:
        return [_present_participle(tags) if "present" in tags else _past_participle(tags)]
    if "gerund" in tags:
        return [{"VerbForm": "Ger"}]
    if "infinitive" in tags:
        return [{"VerbForm": "Inf"}]
    moods = [mood for kaikki, mood in _MOODS if kaikki in tags]
    tense = next((ud for kaikki, ud in _TENSES if kaikki in tags), None)
    if "indicative" in tags or (not moods and tense):
        moods.append("Ind")
    persons = [_PERSON[tag] for tag in tags if tag in _PERSON]
    numbers = [number for kaikki, number in (("singular", "Sing"), ("plural", "Plur")) if kaikki in tags]
    out = []
    for mood, person, number in itertools.product(moods, persons, numbers):
        features = {"VerbForm": "Fin", "Mood": mood, "Person": person, "Number": number}
        if mood in ("Ind", "Sub"):
            if tense is None or (mood == "Sub" and tense not in ("Pres", "Imp")):
                continue
            features["Tense"] = tense
        elif mood == "Imp" and (person, number) not in _IMPERATIVE_CELLS:
            continue
        out.append(features)
    return out


def nominal_features(tags, genders=()):
    """The features of a noun's, adjective's, determiner's, pronoun's or numeral's form, one set per
    gender: its number (singular unless kaikki says plural), its degree, and its gender — the form's
    own, none when it names both, or each of the lemma's for a form kaikki gives none (a noun's
    plural)."""
    base = {"Number": "Plur" if "plural" in tags else "Sing"}
    if "superlative" in tags:
        base["Degree"] = "Sup"
    elif "comparative" in tags:
        base["Degree"] = "Cmp"
    own = [_GENDER[tag] for tag in tags if tag in _GENDER]
    if len(own) == 1:
        return [{**base, "Gender": own[0]}]
    if own or not genders:
        return [base]
    return [{**base, "Gender": gender} for gender in genders]


def reading_tags(upos, tags, genders=(), participle=None):
    """The UD tags kaikki's tags name for a form of `upos`; none when they name no reading.

    A participle's own entry (`participle`: `"past"` or `"present"`) lists its agreed forms
    (`creduta`: feminine; `parlanti`: plural), which read as that participle with that agreement. A
    determiner's, pronoun's or numeral's row reads only when it names agreement alone, and a
    pronoun's only with a gender (D4)."""
    if upos == "VERB":
        if participle and not ({"participle", "infinitive", "gerund"} & tags) and not (set(_PERSON) & tags):
            return [ud_tag(upos, features) for features in verb_features(tags | {"participle", participle})]
        return [ud_tag(upos, features) for features in verb_features(tags)]
    if upos in ("PRON", "DET", "NUM") and not tags <= _NOMINAL_ONLY:
        return []
    if upos == "PRON" and not tags & set(_GENDER):
        return []
    return [ud_tag(upos, f) for f in nominal_features(tags, genders) if "Degree" not in f or upos == "ADJ"]


def _without_qualifiers(arg):
    """A head template's argument without its inline qualifiers (`f,m<l:colloquial>` → `f,m`),
    nested ones included."""
    text = str(arg or "")
    while True:
        stripped = re.sub(r"<[^<>]*>", "", text)
        if stripped == text:
            return text
        text = stripped


def _noun_head(entry):
    """`it-noun`'s arguments, or an empty dict."""
    for head in entry.get("head_templates") or ():
        if head.get("name") == "it-noun":
            return head.get("args") or {}
    return {}


def noun_genders(entry):
    """A noun's genders, and whether its headword is a plural: `it-noun`'s first argument — `m`, `f`,
    `mfbysense`, `mf`, `m,f`, `f,m` and the qualified ones (`eco`: `f,m<l:colloquial>`), `m-p` and
    `f-p` plural only —, else the genders its senses are tagged with (D4)."""
    pieces = [p.strip() for p in _without_qualifiers(_noun_head(entry).get("1")).split(",") if p.strip()]
    genders, plural = [], []
    for piece in pieces:
        known = _NOUN_GENDER.get(piece.removesuffix("-p"))
        if known is None:
            genders = []
            break
        genders += [g for g in known if g not in genders]
        plural.append(piece.endswith("-p"))
    if genders:
        return tuple(genders), all(plural)
    tags = set()
    for sense in entry.get("senses") or ():
        tags.update(sense.get("tags") or ())
    return tuple(_GENDER[tag] for tag in ("masculine", "feminine") if tag in tags), False


def invariable_noun(entry):
    """A noun the dictionary gives one form for (D4): `it-noun`'s plural `#`, alone or among several
    (`città`, `re`, `crisi`), or a sense tagged `invariable` and no plural listed."""
    plurals = [p.strip() for p in _without_qualifiers(_noun_head(entry).get("2")).split(",")]
    if "#" in plurals:
        return True
    senses = entry.get("senses") or []
    return any("invariable" in (sense.get("tags") or ()) for sense in senses) and not any(
        "plural" in (row.get("tags") or ()) for row in entry.get("forms") or ()
    )


def adjective_agrees(entry):
    """Whether an adjective has a feminine of its own (`rosso`: `rossa`), unlike `grande`."""
    for row in entry.get("forms") or ():
        tags = set(row.get("tags") or ())
        if "feminine" in tags and "masculine" not in tags and not tags & (_NOT_A_READING | _DERIVED):
            return True
    return False


def _alternative(sense):
    """A sense that is only an alternative form of another word (D5): an archaic spelling with a table
    of its own, an apocope (`bel`, `buon`, `quel`)."""
    return "alt-of" in (sense.get("tags") or ()) or bool(sense.get("alt_of"))


def _participle_sense(tags, target):
    """A participle sense's tags, `present` added when it names neither tense and its target is a
    present participle — Italian's end in `-nte` (`dormienti`: « plural of dormiente ») —, so that a
    present participle's plural is not read as a past one (D3)."""
    if "participle" in tags and not tags & {"past", "present"} and target.endswith("nte"):
        return tags | {"present"}
    return tags


def _participle_entry(senses):
    """`"present"` or `"past"` when an entry is a participle's own — every sense a form of a verb,
    one a participle (`parlante`, `creduto`) — whose rows are its agreed forms (D3); else None."""
    marked = [set(sense.get("tags") or ()) for sense in senses]
    if not marked or not all(_is_form_of(sense) for sense in senses):
        return None
    participles = [tags for tags in marked if "participle" in tags]
    if not participles:
        return None
    return "present" if any("present" in tags for tags in participles) else "past"


class Readings:
    """The readings kaikki states, by (form, lemma), and what the rules need of its entries.

    - `table`: a lemma's table — its own form, its inflections —, and a participle's own entry's
      agreed forms; `senses`: a form's own entry's senses, for what no table lists (D5);
    - `spelt`: the table of a spelling the dictionary gives only as one word's (change 10's D2:
      `perchè` → *perché*), read as that word's and kept only where the forms table files the form
      under it (D5);
    - `shared`: the rows of the article table kaikki writes under each of *il*, *lo*, *la*, *gli*,
      *le* that an article's own head does not name, kept only under the article the forms table
      files the form under (D4);
    - `literary_table`, `literary_senses`: the rows and senses tagged poetic, Latinism or literary
      (`_LITERARY`), read as the others are, but never toward another word than the form's own (D6);
    - `misspelt`: form → `(word, part of speech)`, the words an entry of the form gives it as a
      doubtful spelling of (`anno`, a verb entry, « misspelling of hanno »), whose readings it names
      no other word with (D6, `misspelt_readings`);
    - `poses`: lemma → the parts of speech of its lemma entries, among those read;
    - `named_poses`: the same from the entries holding a meaning unmarked (`_MARKED_MEANINGS`), the
      parts of speech another form may name the lemma in (D6);
    - `standard`: the words with a lemma entry that is not only regional (D6).
    """

    def __init__(self):
        self.table = collections.defaultdict(set)
        self.senses = collections.defaultdict(set)
        self.spelt = collections.defaultdict(set)
        self.shared = collections.defaultdict(set)
        self.literary_table = collections.defaultdict(set)
        self.literary_senses = collections.defaultdict(set)
        self.misspelt = collections.defaultdict(set)
        self.poses = collections.defaultdict(set)
        self.named_poses = collections.defaultdict(set)
        self.standard = set()

    def misspelt_readings(self):
        """`(form, lemma) → tags` the form names no other word with (D6): where an entry of the form
        is a doubtful spelling of a cell of the lemma's table, in that entry's part of speech, the
        cell's own readings — `anno` (the year), whose verb entry is « misspelling of hanno », names no
        *avere* as its third person plural, which *avere*'s table lists as `hanno` (and as the disused
        `ànno`, its qualifier dropped by kaikki). Another cell's reading stands (`puzza`, a noun entry
        spelling *puzzo*, still names *puzzare*'s present), and so does every reading of the form's own
        lemma (`diè`, *dare*'s poetic passato remoto)."""
        cells = collections.defaultdict(set)
        for (form, lemma), tags in (*self.table.items(), *self.literary_table.items()):
            if form != lemma:
                cells[form].update((lemma, tag) for tag in tags)
        out = collections.defaultdict(set)
        for form, spelt in self.misspelt.items():
            for word, upos in spelt:
                for lemma, tag in cells.get(word, ()):
                    if parse_tag(tag)[0] == upos:
                        out[(form, lemma)].add(tag)
        return out

    def pairs(self, forms, literary=True):
        """`(form, lemma) → tags`: a table's reading of a pair wins over a form entry's senses of the
        same part of speech — `grandi`, which *grande*'s noun table lists, keeps its adjective entry's
        reading (D5); a spelling's table and the shared article table only where `forms` files the
        form under that lemma, a spelling's only in a part of speech the word's own table does not give
        that pair; the poetic, Latin and literary rows and senses unless `literary` is false."""
        tables = collections.defaultdict(set)
        for pair, tags in self.table.items():
            tables[pair] |= tags
        if literary:
            for pair, tags in self.literary_table.items():
                tables[pair] |= tags
        for pair, tags in self.shared.items():
            if forms.get(pair[0]) == pair[1]:
                tables[pair] |= tags
        for pair, tags in self.spelt.items():
            if forms.get(pair[0]) == pair[1]:
                # The word's own table first: a spelling's inflection reads only in a part of speech
                # the word's table gives nothing of (`moglie`, which *moglia*'s table lists as its
                # plural, stays *moglie*'s singular).
                given = {parse_tag(tag)[0] for tag in self.table.get(pair, ())}
                tables[pair] |= {tag for tag in tags if parse_tag(tag)[0] not in given}
        out = {pair: set(tags) for pair, tags in self.senses.items()}
        for pair, tags in self.literary_senses.items() if literary else ():
            out.setdefault(pair, set()).update(tags)
        for pair, tags in tables.items():
            given = {parse_tag(tag)[0] for tag in tags}
            out[pair] = {tag for tag in out.get(pair, ()) if parse_tag(tag)[0] not in given} | tags
        return out


def _dialect_rows(rows):
    """The rows of a verb's tables that are a dialect's, not Italian's (D3, as implemented): kaikki
    merges into an entry every conjugation table of its page, the Classical Roman Dialect's among
    them, with no tag to say so (*avere*'s third table: `àmo`, `émo`, `ébbe` a first person, `avéssi`
    a passato remoto). When an entry has several tables, one listing a present indicative's first
    person plural none of which ends in Italian's `-iamo` (`avémo`, `dàmo`, `credémo`), while another
    table of the entry lists one that does (`abbiàmo`), is the dialect's, and its rows give no reading;
    a table listing none (*essere*'s usage notes, untagged) goes with it. Answers the rows' indices."""
    starts = [i for i, row in enumerate(rows) if "table-tags" in (row.get("tags") or ())]
    if len(starts) < 2:
        return set()
    tables = [range(start, end) for start, end in zip(starts, [*starts[1:], len(rows)])]

    def italian(table):
        for i in table:
            tags = set(rows[i].get("tags") or ())
            words = nfc_lower(rows[i].get("form")).split()
            if _FIRST_PLURAL_PRESENT <= tags and not tags & _REPEATED_CELLS and words:
                if without_stress(words[-1]).endswith("iamo"):
                    return True
        return False

    kept = [italian(table) for table in tables]
    if not any(kept):
        return set()
    return {i for table, ok in zip(tables, kept) if not ok for i in table}


def _masculine_superlative(tags, form):
    """An adjective's superlative named with no agreement (D4) is its masculine singular, the form a
    head names, in `-o`: *bello*'s head names `bellissimo`, *abile*'s `abilissimo`, and so does the
    superlative's own entry (« superlative of bello »), whose feminine and plurals it lists
    (`bellissima`)."""
    if "superlative" in tags and not tags & {"masculine", "feminine", "plural"} and form.endswith("o"):
        return tags | {"masculine"}
    return tags


def _unmarked_meaning(senses):
    """Whether a lemma entry holds a meaning no archaic, poetic, Latin or literary tag marks
    (`_MARKED_MEANINGS`, D6) — an entry with no sense too."""
    meanings = [sense for sense in senses if not _is_form_of(sense)]
    return not meanings or any(not set(sense.get("tags") or ()) & _MARKED_MEANINGS for sense in meanings)


def _row_skipped(tags):
    """A row that gives no reading of its own (D3–D5): bookkeeping, doubtful, a repeated cell, a
    region's or a register's, a derived word's."""
    return bool(tags & (_NOT_A_READING | _REPEATED_CELLS | _REGISTERS | _DERIVED))


def read_readings(entry, readings, survey, spellings=None):
    """One entry's readings into `readings`, word and form lowercased and in NFC (D3–D5).

    Never from a capitalised headword (`NATO`, `Roma`), a clitic compound's entry, nor from an entry
    whose every sense is an alternative form of another word (an archaic spelling with a table of its
    own, an apocope: `bel`). A spelling `spellings` names (change 10's D2: `perchè`) reads as the word
    it spells. A form's own entry gives each sense toward its target — the plain verb where change 9
    files a reflexive's forms —, but a doubtful, regional or register-marked one; a participle's own
    entry gives its agreed forms too. A lemma's entry gives a noun's or an adjective's own form, and its
    table's rows, each written as change 9 writes it (`written_form`): never a row that repeats a cell
    (the negative and formal imperatives, a head row its table lists), a doubtful, dialectal or
    hypercorrect row, a defective verb's `-`, a clitic cluster, a derived word's row, a feminine noun's
    masculine (`attrice`: `attore`), nor a plural-headed determiner's or pronoun's table (`gli`, `le`).
    A reflexive verb's rows read as change 9 reads them (`_read_reflexive`)."""
    upos = _UPOS.get(entry.get("pos"))
    word = nfc_lower(entry.get("word"))
    if upos is None or not _TOKEN.fullmatch(word):
        return
    raw = unicodedata.normalize("NFC", (entry.get("word") or "").strip())
    if raw != raw.lower():
        return
    senses = entry.get("senses") or []
    if any(_is_compound(sense) for sense in senses):
        return
    for sense in senses:
        # A doubtful spelling of another word (D6): `anno`, « misspelling of hanno ».
        if _alternative(sense) and set(sense.get("tags") or ()) & _DOUBTFUL:
            readings.misspelt[word].update(
                (nfc_lower(ref.get("word")), upos)
                for ref in sense.get("alt_of") or ()
                if isinstance(ref, dict) and ref.get("word")
            )
    spelt = (spellings or {}).get(word)
    if spelt is None and senses and all(_alternative(sense) for sense in senses):
        return
    if upos == "VERB":
        clitic = survey.clitic_infinitive(word)
        if clitic is not None:
            _read_reflexive(entry, word, *clitic, readings, survey)
            return
    home = spelt or word
    table = readings.spelt if spelt else readings.table
    meaning = not senses or not all(_is_form_of(sense) for sense in senses)
    participle = _participle_entry(senses) if upos == "VERB" else None
    for sense in senses:
        tags = set(sense.get("tags") or ())
        if upos == "ADJ":
            tags = _masculine_superlative(tags, word)
        if not _is_form_of(sense) or tags & _NOT_A_READING or tags & _REGISTERS:
            continue
        for target in _form_targets(sense):
            target = survey.plain_of(nfc_lower(target))
            if _TOKEN.fullmatch(target):
                into = readings.literary_senses if tags & _LITERARY else readings.senses
                into[(word, target)].update(reading_tags(upos, _participle_sense(tags, target)))
    if meaning and spelt is None:
        readings.poses[word].add(upos)
        if _unmarked_meaning(senses):
            readings.named_poses[word].add(upos)
        if not all(set(sense.get("tags") or ()) & _REGIONS for sense in senses):
            readings.standard.add(word)
    if not meaning and participle is None and spelt is None:
        return
    heads = entry.get("head_templates") or ()
    genders, plural = noun_genders(entry) if upos == "NOUN" else ((), False)
    if upos == "NOUN" and meaning and not spelt:
        own = {"Number": "Plur" if plural else "Sing"}
        for features in [{**own, "Gender": gender} for gender in genders] or [own]:
            table[(word, home)].add(ud_tag(upos, features))
        if not plural and invariable_noun(entry):
            for features in [{"Number": "Plur", "Gender": gender} for gender in genders] or [{"Number": "Plur"}]:
                table[(word, home)].add(ud_tag(upos, features))
    elif upos == "ADJ" and meaning and not spelt:
        plural_only = all({"plural", "plural-only"} & set(sense.get("tags") or ()) for sense in senses)
        own = {"Number": "Plur" if plural_only else "Sing", **({"Gender": "Masc"} if adjective_agrees(entry) else {})}
        table[(word, home)].add(ud_tag(upos, own))
    if upos in ("DET", "PRON") and any((head.get("args") or {}).get("g") in _PLURAL_HEADS for head in heads):
        return
    rows = entry.get("forms") or ()
    start = next((i for i, row in enumerate(rows) if set(row.get("tags") or ()) & _TABLE_START), len(rows))
    dialect = _dialect_rows(rows) if upos == "VERB" else set()

    def written(row):
        text = nfc_lower(row.get("form"))
        return written_form(text, (word,), survey.form_links) if text and " " not in text else None

    cells = {
        written(row)
        for i, row in enumerate(rows[start:], start)
        if i not in dialect and not set(row.get("tags") or ()) & _BOOKKEEPING
    }
    named = {written(row) for row in rows[:start]}
    article = entry.get("pos") == "article" and start < len(rows)
    for i, row in enumerate(rows):
        tags = set(row.get("tags") or ())
        form = written(row)
        if i in dialect or _row_skipped(tags) or form is None or not _TOKEN.fullmatch(form):
            continue
        if i < start and form in cells:
            continue
        if upos == "NOUN" and "masculine" in tags and ("plural" not in tags or (genders and "Masc" not in genders)):
            # A feminine noun's masculine, singular or plural (`signorina`: `signorino`, `signorini`).
            continue
        if upos == "ADJ":
            tags = _masculine_superlative(tags, form)
        into = readings.shared if article and i >= start and form not in named else table
        if into is readings.table and tags & _LITERARY:
            into = readings.literary_table
        into[(form, home)].update(reading_tags(upos, tags, genders, participle))


def _read_reflexive(entry, word, clitic, plain, readings, survey):
    """A verb whose infinitive carries a clitic (D3), its rows read as change 9 reads its forms
    (`Lexicon._read_clitic_verb`): a reflexive (`si`) is filed under its plain verb where the section
    has one (`lavarsi` → *lavare*) and is its own lemma otherwise (`pentirsi`); a row written with its
    pronoun reads without it (`mi làvo` → `lavo`); a row carrying the pronoun reads as its bare base
    under a reflexive-only lemma (`pentìrsi` → `pentire`, `pèntiti` → `penti`) and gives nothing under
    a plain verb, never as its clitic-bearing string (`làvati`, which would read as *lavare*'s
    participle `lavati`). Any other clitic (`andarsene`, `farcela`) gives no reading: a page writes
    the cluster's words apart."""
    if clitic != "si":
        return
    lemma = plain or word
    senses = entry.get("senses") or []
    if _unmarked_meaning(senses):
        # The reflexive's meaning names its lemma, the plain verb's too where it is filed under it
        # (`arreso`, of *arrendersi*, names *arrendere*, an archaic verb of its own).
        readings.named_poses[lemma].add("VERB")
    if plain is None:
        readings.poses[word].add("VERB")
        if not all(set(sense.get("tags") or ()) & _REGIONS for sense in senses):
            readings.standard.add(word)
    rows = entry.get("forms") or ()
    dialect = _dialect_rows(rows)
    for i, row in enumerate(rows):
        tags = set(row.get("tags") or ())
        text = nfc_lower(row.get("form"))
        if i in dialect or _row_skipped(tags) or not text:
            continue
        into = readings.literary_table if tags & _LITERARY else readings.table
        words = text.split()
        if len(words) == 2 and words[0] in REFLEXIVE_PRONOUNS:
            form = written_form(words[1], (lemma, word), survey.form_links)
            if _TOKEN.fullmatch(form):
                into[(form, lemma)].update(reading_tags("VERB", tags))
            continue
        if len(words) != 1 or plain is not None:
            continue
        form = written_form(text, (lemma, word), survey.form_links)
        tail = _REFLEXIVE_TAIL.search(form)
        if not tail:
            continue
        base = form[: tail.start()]
        if base.endswith("r"):
            base += "e"
        if len(base) > 2 and _TOKEN.fullmatch(base):
            into[(base, word)].update(reading_tags("VERB", tags))


def _kind(tag):
    """A reading's part of speech and verb form: what a form of a form is not added over (D5)."""
    upos, features = parse_tag(tag)
    return upos, features.get("VerbForm")


def compose(outer, inner):
    """A form of a form (D5): the outer reading — of the word the form is a form of — with the inner
    one's agreement, along one part of speech, a verb's only through a participle, a nominal's never
    through a plural; None otherwise. A
    present participle takes the inner number alone (`parlanti` → `parlante` → *parlare*): it has no
    gender, whatever the participle sense of its plural's own entry names (« plural of dormiente »,
    tagged as a participle with neither `present` nor `past`)."""
    upos, features = parse_tag(outer)
    inner_upos, inner_features = parse_tag(inner)
    if upos != inner_upos:
        return None
    if upos != "VERB" and features.get("Number") == "Plur":
        # A plural is no base another form inflects from: `colli`, the plural of `colle` (« hill »),
        # is no form of *colla*, whose plural `colle` also is.
        return None
    if upos == "VERB" and (features.get("VerbForm") != "Part" or inner_features.get("VerbForm") not in (None, "Part")):
        return None
    agreement = ("Number",) if upos == "VERB" and features.get("Tense") == "Pres" else ("Gender", "Number")
    for name in agreement:
        if name in inner_features:
            features[name] = inner_features[name]
    return ud_tag(upos, features)


def set_aside(readings, overrides=OVERRIDES):
    """The links a reviewed override row of change 9 sets aside (D5): an overridden form's own entry
    linking it to another word than the row's, which no lemma's table lists."""
    return {
        (form, target)
        for (form, target) in readings.senses
        if form in overrides and target != overrides[form][0] and (form, target) not in readings.table
    }


def grammar_rows(readings, forms, ranks, overrides=OVERRIDES):
    """`grammar.tsv`'s rows, sorted: the readings of the forms `forms` holds, under the lemmas
    `ranks` keeps (D2), a form's of its own lemma marked `-`, another's `other` (D6).

    - A form of a form along one part of speech reads as the word it is a form of reads, with its
      own agreement (`creduta` → `creduto` → *credere*), through the form's own lemma too — unless
      the form already reads as that word in that part of speech and verb form (D5).
    - A reading's part of speech is one the dictionary holds its lemma as, when it holds the lemma
      as one of those read: a participle filed under a noun's spelling names its verb instead
      (`parlata`, the noun, names *parlare*) (D5).
    - `other` only toward an entry of the dictionary that is not only regional, in a part of speech
      an entry holds the word as with a meaning not marked archaic, poetic or literary (D6:
      `dire` names no *diro*, « dire », literary), and never from
      a poetic, Latin or literary row or sense alone, directly or through a form of a form (D6:
      `sono` names no *suonare*; `asperrima`, the literary superlative's feminine, stays *aspro*'s).
    - `other` never with the reading of a cell an entry of the form gives it as a doubtful spelling
      of (`misspelt_readings`: `anno` names no *avere*).
    - Never through a link an override row sets aside (`set_aside`)."""
    excluded = set_aside(readings, overrides)
    misspelt = readings.misspelt_readings()
    plain = readings.pairs(forms, literary=False)
    by_form = collections.defaultdict(dict)
    literary = {}
    for (form, lemma), tags in readings.pairs(forms).items():
        if tags and (form, lemma) not in excluded:
            by_form[form][lemma] = tags
            literary[(form, lemma)] = tags - plain.get((form, lemma), set())
    rows = set()
    for form, own in forms.items():
        direct = by_form.get(form, {})
        options = {lemma: set(tags) for lemma, tags in direct.items()}
        # The readings some row or sense not of poetry or letters gives (D6): those another word may be
        # named with.
        named = {lemma: tags - literary[(form, lemma)] for lemma, tags in direct.items()}
        for inner_word, inner_tags in direct.items():
            if inner_word == form:
                continue
            for lemma, outer_tags in by_form.get(inner_word, {}).items():
                if lemma in (inner_word, form):
                    continue
                kinds = {_kind(tag) for tag in direct.get(lemma, ())}
                for outer in outer_tags:
                    for inner in inner_tags:
                        tag = compose(outer, inner)
                        if tag is None or _kind(tag) in kinds:
                            continue
                        options.setdefault(lemma, set()).add(tag)
                        if outer not in literary[(inner_word, lemma)] and inner not in literary[(form, inner_word)]:
                            named.setdefault(lemma, set()).add(tag)
        for lemma, tags in options.items():
            if lemma not in ranks:
                continue
            mark = "-" if lemma == own else "other"
            held = readings.poses.get(lemma)
            if mark == "other":
                held = readings.named_poses.get(lemma)
                if not held or lemma not in readings.standard:
                    continue
                tags = named.get(lemma, set()) - misspelt.get((form, lemma), set())
            if held:
                tags = {tag for tag in tags if parse_tag(tag)[0] in held}
            rows.update((form, lemma, tag, mark) for tag in tags)
    return sorted(f"{form}\t{lemma}\t{tag}\t{mark}\n" for form, lemma, tag, mark in rows)


def _carried(rows, sources):
    """`rows` with each spelling of `sources` (spelling → the forms whose readings it carries) carrying
    those forms' rows, re-keyed, and none of its own."""
    by_form = collections.defaultdict(list)
    for row in rows:
        by_form[row.split("\t", 1)[0]].append(row)
    out = {row for row in rows if row.split("\t", 1)[0] not in sources}
    for spelling, carried in sources.items():
        for form in carried:
            out.update(spelling + row[len(form) :] for row in by_form.get(form, ()))
    return out


def spelling_rows(rows, forms, spellings=None):
    """`grammar.tsv`'s rows with each spelling change 10 reads as its word carrying that word's
    readings, as a form of it, and none of its own (read-lingua-italian-unmarked-spellings D4; D5):

    - a dictionary spelling `spellings` names (`dictionary_spellings`, its D2) those of the form it
      spells, where the forms table files both under one lemma and nothing gives the spelling a
      reading of its own: `perchè` *perché*'s, `caffé` *caffè*'s, `fà` those of `fa` (*fare*'s) —
      its own entry's head, a misspelling's, names no gender —, while `dà`, which the section also
      gives as « alternative form of dare », keeps its own, the present of *dare*; its inflections are
      read under the word from its own table (`read_readings`);
    - each row of the web's list and each owner's exception those of every form of its word it writes
      with the same letters, marks dropped (`_unmarked`): `puo` `può`'s, `citta` `città`'s, `perche`,
      whose own entry is *perca*'s plural, `perché`'s (the exceptions' own entries leave the tables).

    Sorted, each row once."""
    read = {row.split("\t", 1)[0] for row in rows if row.endswith("\t-\n")}
    sources = {
        spelling: [word]
        for spelling, word in (spellings or {}).items()
        if spelling in forms and forms.get(word) == forms[spelling] and spelling not in read
    }
    rows = _carried(rows, sources)
    of_letters = collections.defaultdict(list)
    for form, lemma in forms.items():
        of_letters[(_unmarked(form), lemma)].append(form)
    web = {}
    for spelling in (*WEB_SPELLINGS, *OWNER_EXCEPTIONS):
        lemma = forms.get(spelling)
        if lemma is not None:
            web[spelling] = [form for form in of_letters[(_unmarked(spelling), lemma)] if form != spelling]
    return sorted(_carried(rows, web))


def readings_figures(rows, forms):
    """What the readings hold, for the report and the pull request (D2, D6, D7): the readings, the
    forms carrying one of their own, the `other` marks and the forms carrying one, the tags, the
    readings by part of speech, the forms by how many readings of their own they carry, and the forms
    reading the present indicative and the present subjunctive of one person and number."""
    own = collections.defaultdict(set)
    others = set()
    tags = set()
    by_pos = collections.Counter()
    other_marks = 0
    for row in rows:
        form, _, tag, mark = row.rstrip("\n").split("\t")
        tags.add(tag)
        by_pos[tag.split("|", 1)[0]] += 1
        if mark == "-":
            own[form].add(tag)
        else:
            other_marks += 1
            others.add(form)
    both = 0
    for form, held in own.items():
        presents = {
            (f.get("Person"), f.get("Number"), f.get("Mood"))
            for f in (parse_tag(tag)[1] for tag in held)
            if f.get("Tense") == "Pres" and f.get("VerbForm") == "Fin"
        }
        if any((p, n, "Ind") in presents and (p, n, "Sub") in presents for p, n, _ in presents):
            both += 1
    return {
        "readings": len(rows),
        "forms with a reading of their own": len(own),
        "forms without": len(forms) - len(own),
        "other marks": other_marks,
        "forms with an other mark": len(others),
        "tags": len(tags),
        "by part of speech": dict(sorted(by_pos.items())),
        "own readings per form": dict(sorted(collections.Counter(len(held) for held in own.values()).items())),
        "forms reading the present indicative and subjunctive alike": both,
    }



# — Morph-it! (D3) —

# Morph-it!'s verbal categories (a lexical verb, the auxiliaries, the modals, the causative and the
# aspectual), whose inflectional features alone make a form: any other feature names a clitic the form
# carries (`VER:inf+pres+lo`), and no form carrying clitics joins the tables (D4).
_MORPH_IT_VERBS = frozenset({"VER", "AUX", "MOD", "CAU", "ASP"})
_MORPH_IT_INFLECTION = frozenset(
    {"inf", "part", "ger", "ind", "sub", "cond", "impr", "pres", "past", "impf", "fut", "1", "2", "3", "s", "p", "m", "f"}
)


def read_morph_it(path, member=MORPH_IT_MEMBER, encoding=MORPH_IT_ENCODING):
    """Morph-it!'s `(form, lemma)` pairs, read from its archive's lexicon without extracting it — each
    line `form<TAB>lemma<TAB>features`, lowercased and in NFC —, but a form carrying clitics, by its
    features, and a line that is not three fields."""
    pairs = set()
    with tarfile.open(path, "r:gz") as archive:
        data = archive.extractfile(member).read().decode(encoding)
    for line in data.split("\n"):
        fields = line.rstrip("\r").split("\t")
        if len(fields) != 3:
            continue
        form, lemma, features = fields
        category, _, inflection = features.partition(":")
        if category in _MORPH_IT_VERBS and not set(inflection.split("+")) <= _MORPH_IT_INFLECTION:
            continue
        pairs.add((nfc_lower(form), nfc_lower(lemma)))
    return pairs


def read_morph_it_names(path, member=MORPH_IT_MEMBER, encoding=MORPH_IT_ENCODING):
    """The words Morph-it! lists as proper nouns (its `NPR` category), lowercased and in NFC: the name
    check of a form only a dictionary spelling reads (`foreign_readings`)."""
    names = set()
    with tarfile.open(path, "r:gz") as archive:
        data = archive.extractfile(member).read().decode(encoding)
    for line in data.split("\n"):
        fields = line.rstrip("\r").split("\t")
        if len(fields) == 3 and fields[2].startswith("NPR"):
            names.add(nfc_lower(fields[0]))
    return names


def morph_it_additions(pairs, forms, ranks, lexicon, frequency):
    """Morph-it!'s forms that join the table (D3): a form it lists under a lemma the tables rank,
    which the Wiktionary names nowhere (no candidate of the section's), a form of the tables, attested
    by wordfreq, and no piece, split contraction or word no rank may go to. A form it lists under two
    ranked lemmas goes to the commoner. A form the Wiktionary reads is never moved, and a ranked
    lemma's own form is never one. Answers `form → lemma`."""
    added = {}
    by_form = collections.defaultdict(set)
    for form, lemma in pairs:
        if lemma in ranks and form != lemma:
            by_form[form].add(lemma)
    for form, lemmas in sorted(by_form.items()):
        if form in forms or form in lexicon.candidates or form in lexicon.unranked:
            continue
        if not is_form(form) or form in PIECES or form in CONTRACTIONS or frequency(form) <= 0:
            continue
        added[form] = min(lemmas, key=lambda lemma: (-frequency(lemma), lemma))
    return added


# — what one lemma per form costs, and the report —


def m8_cost(forms, ranks, lexicon, top_n, tops=(1000, 2000, 5000)):
    """M8's cost (D5), counted as French's README counts it: among wordfreq's `tops` commonest words,
    the dictionary nouns (a noun lemma entry) whose own form reads as a verb (`porta` → *portare*) and
    which are no ranked lemma. Answers `{top: [(rank, noun, verb), …]}`."""
    words = [nfc_lower(w) for w in top_n(max(tops))]
    out = {}
    for top in tops:
        out[top] = [
            (rank, word, forms[word])
            for rank, word in enumerate(words[:top], 1)
            if "noun" in lexicon.poses.get(word, ())
            and word not in ranks
            and forms.get(word, word) != word
            and "verb" in lexicon.poses.get(forms[word], ())
        ]
    return out


def decisions(forms, lexicon, step, counts, frequency, overrides=OVERRIDES):
    """How many forms of the tables each rule of D5 decided, among the lemmas each reaches when it
    reaches several (`porta`: *porta* and *portare*, MarkIT's counts deciding)."""
    decided = collections.Counter()
    for form in forms:
        options = reach(form, lexicon.candidates.get(form, set()), lexicon.links, step)
        if len(options) > 1:
            decided[decided_by(form, options, counts, lexicon.lemmas, frequency, overrides)] += 1
    return decided


def analyser_version(path=_ANALYSIS_RS):
    """Italian's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const ITALIAN_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no ITALIAN_ANALYZER_VERSION in {path}")
    return m.group(1)


def notice(morph_it=True):
    """it-fr's attribution stack (D9): the section, wordfreq, MarkIT — and Morph-it!, under its
    CC BY-SA 2.0 option, while it is pinned —, each with its licence's address. UD Italian-PUD is
    measured against, never shipped."""
    text = """Cymbra Lingua data pack — IT->FR attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), Italian section: CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/) + GFDL (https://www.gnu.org/licenses/fdl-1.3.html)
— the forms and their lemmas.

wordfreq (Italian frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/) — the commonest lemmas and which forms
are attested.

UD Italian-MarkIT, the Universal Dependencies Italian-MarkIT treebank by Teresa Paccosi, Alessio
Palmero Aprosio and Sara Tonelli (https://github.com/UniversalDependencies/UD_Italian-MarkIT):
CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/) — how often a form stands for each of its
lemmas, to choose one, and how often a hyphenated word occurs, to rank it; its training and
development sections alone.
"""
    if morph_it:
        text += """
Morph-it!, a free morphological lexicon for the Italian language, by Marco Baroni and Eros Zanchetta
(https://docs.sslmit.unibo.it/doku.php?id=resources:morph-it), version 0.4.8: CC BY-SA 2.0
(https://creativecommons.org/licenses/by-sa/2.0/), one of its two licences, the other being the
LGPL — the forms it lists that the Wiktionary names nowhere, under the lemmas the tables rank.
"""
    return text


def manifest(pack_version, morph_it=True):
    """it-fr's manifest (D9): Italian glossed in French, Italian's analyser version; no levels yet
    (change 14), so `levels_estimated` is absent."""
    licences = ["kaikki / enwiktionary (CC BY-SA 4.0 + GFDL)", "wordfreq (CC BY-SA 4.0)", "UD Italian-MarkIT (CC BY 4.0)"]
    sources = [
        {"name": "kaikki", "licence": "CcBySa"},
        {"name": "wordfreq", "licence": "CcBySa"},
        {"name": "UD Italian-MarkIT", "licence": "CcBy"},
    ]
    if morph_it:
        licences.append("Morph-it! (CC BY-SA 2.0)")
        sources.append({"name": "Morph-it!", "licence": "CcBySa"})
    return {
        "meta": {
            "studied": "it",
            "native": "fr",
            "pack_version": pack_version,
            "analyzer_version": analyser_version(),
            "licences": licences,
        },
        "sources": sources,
    }


def reduce(work, max_lemmas, pack_version, frequency, top_n, morph_it=MORPH_IT, report=print, foreign=None):
    """The reduction over the raw sources in `work`, its tables written there. `morph_it` is the
    archive's file name in `work`, or None to read the Wiktionary alone. `foreign(word)` is the highest
    Zipf frequency English, French and Spanish give a word (D3's test, read for D2's forms too), None
    to skip that test. Answers what the report prints, as a dict, for the measurement."""
    kaikki = os.path.join(work, KAIKKI)
    survey = survey_kaikki(kaikki)
    kept = {}
    spellings = dictionary_spellings(kaikki, frequency, kept)
    readings = Readings()
    lexicon = read_kaikki(kaikki, survey, spellings, readings)
    # Italian's acronym rows, each backed by the section's own entry (keep-lingua-italian-acronyms-and-names D2).
    acronyms.check_rows(lexicon.acronyms)
    paradigm = lexicon.spelling_paradigms()
    # The web's list stops for a person when the section names a row (read-lingua-italian-unmarked-
    # spellings D3), as on a word the pre-pass writes and no form.
    crossed = lexicon.web_boundary()
    if crossed:
        raise SystemExit(
            f"it-fr: {len(crossed)} spelling(s) of WEB_SPELLINGS the section gives a meaning of its own, "
            f"or as a spelling of a word: {', '.join(crossed)}"
        )
    counts = treebank.markit_counts([os.path.join(work, name) for name in MARKIT])
    named = name_or_word(lexicon.candidates, lexicon.poses, frequency)
    own_words(lexicon)
    read_apocopes(lexicon)
    read_web_spellings(lexicon)
    forms, ranks, step = reduce_forms(lexicon, counts, treebank.markit_zipf(counts), frequency, top_n, max_lemmas)
    unread = unread_web_spellings(forms)
    if unread:
        raise SystemExit(
            f"it-fr: {len(unread)} row(s) of the web's list read as no form of their word, which the tables "
            f"no longer rank: {', '.join(unread)}"
        )
    decided = decisions(forms, lexicon, step, counts, frequency)
    added = {}
    if morph_it is not None:
        path = os.path.join(work, morph_it)
        if not os.path.isfile(path):
            raise SystemExit(f"it-fr: {path} is missing: Morph-it! is among it-fr's pinned sources (pack_sources.py PINNED['it-fr'])")
        added = morph_it_additions(read_morph_it(path), forms, ranks, lexicon, frequency)
        forms.update(added)
    alien = []
    if foreign is not None and spellings:
        before = readings_without_spellings(kaikki, survey, counts, frequency, top_n, max_lemmas)
        names = {word for word, poses in lexicon.poses.items() if "name" in poses}
        markit = treebank.markit_pos_counts([os.path.join(work, name) for name in MARKIT])
        names |= {word for (word, upos) in markit if upos == "PROPN"}
        if morph_it is not None:
            names |= read_morph_it_names(os.path.join(work, morph_it))
        alien = foreign_readings(forms, paradigm, before, frequency, foreign, names)
        for form in alien:
            del forms[form]
    missing = seed_pieces(forms)
    unwritten = unwritten_words(forms)
    if missing or unwritten:
        raise SystemExit(
            f"it-fr: {len(missing) + len(unwritten)} word(s) Italian's pre-pass writes are no form: "
            + ", ".join(missing + unwritten)
        )

    # Each acronym row's lower case is another word's form in the tables, the case a row repairs; the
    # rows then join the forms, each its own lemma, capitals kept, read by no other rule
    # (keep-lingua-italian-acronyms-and-names D2).
    acronyms.check_lowercase(forms)
    written = {**forms, **acronyms.acronym_forms()}
    common.write(work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(written.items())))
    # The readings of the forms and ranks just chosen (add-lingua-italian-grammar-tables D1): the
    # acronym rows, capitalised headwords, carry none (D5).
    grammar = spelling_rows(grammar_rows(readings, forms, ranks), forms, spellings)
    common.write(work, "grammar.tsv", "".join(grammar))
    common.write(work, "freq.tsv", "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: (kv[1], kv[0]))))
    # Italian's dictionary words (lexical.tsv): the acronym rows alone, glossed or not, until change 16's
    # glosses write the rest — its dictionary words less its names, plus the rows (D5, D6). it-fr's
    # glosses: none yet, change 16's (D9).
    common.write(work, "lexical.tsv", "".join(f"{word}\n" for word in sorted(acronyms.ACRONYMS)))
    common.write(work, "gloss.tsv", "")
    common.write(work, "NOTICE", notice(morph_it is not None))
    common.write(
        work, "manifest.json", json.dumps(manifest(pack_version, morph_it is not None), indent=2, ensure_ascii=False) + "\n"
    )

    last = max(ranks, key=ranks.get)
    by_lemma = collections.Counter(added.values())
    summary = {
        "forms": len(forms),
        "acronyms": sorted(acronyms.ACRONYMS),
        "lemmas": len(ranks),
        "last lemma": last,
        "last zipf": round(frequency(last), 2),
        "decided": dict(decided),
        "names rule": dict(named),
        "morph-it added": len(added),
        "morph-it lemmas": len(by_lemma),
        "morph-it top lemmas": by_lemma.most_common(10),
        "m8": {top: len(nouns) for top, nouns in m8_cost(forms, ranks, lexicon, top_n).items()},
        "read": dict(lexicon.stats),
        "dictionary spellings": dict(spellings),
        "dictionary spellings kept apart": dict(kept),
        "foreign readings": alien,
        "web spellings": len(WEB_SPELLINGS),
        "owner's exceptions": len(OWNER_EXCEPTIONS),
        "grammar": readings_figures(grammar, forms),
    }
    report(f"reduced it-fr: forms={summary['forms']} lemmas={summary['lemmas']} (the last, {last}, at Zipf {summary['last zipf']})")
    report(
        f"reduced it-fr: acronyms={len(acronyms.ACRONYMS)} ({', '.join(summary['acronyms'])}), each a form in capitals "
        "that is its own lemma and a dictionary word, with no rank"
    )
    report(
        "reduced it-fr: one lemma per form, the forms reaching several: "
        + ", ".join(f"{decided.get(rule, 0)} by {rule}" for rule in DECIDED_BY)
        + "; the names rule: "
        + ", ".join(f"{n} {what}" for what, n in sorted(named.items()))
    )
    report(
        f"reduced it-fr: M8's cost, dictionary nouns read as a verb among wordfreq's 1,000 / 2,000 / 5,000: "
        f"{summary['m8'][1000]} / {summary['m8'][2000]} / {summary['m8'][5000]}"
    )
    if morph_it is not None:
        report(f"reduced it-fr: Morph-it! added {len(added)} form(s) under {len(by_lemma)} lemma(s)")
    report("reduced it-fr: read " + ", ".join(f"{n} {what}" for what, n in sorted(lexicon.stats.items())))
    for line in spellings_report(spellings, kept):
        report(f"reduced it-fr: {line}")
    report(
        f"reduced it-fr: forms only a dictionary spelling reads, left unread as another language's word or a "
        f"name={len(alien)}: " + ", ".join(alien)
    )
    report(
        f"reduced it-fr: the web's spellings without their accent, {len(WEB_SPELLINGS)} row(s) of the reviewed "
        f"list and {len(OWNER_EXCEPTIONS)} owner's exception(s), each read as its word"
    )
    figures = summary["grammar"]
    report(
        f"reduced it-fr: readings={figures['readings']} in {figures['tags']} tags; {figures['forms with a reading of their own']} "
        f"forms read as their own lemma, {figures['forms without']} not; {figures['other marks']} other marks on "
        f"{figures['forms with an other mark']} forms"
    )
    report(
        "reduced it-fr: readings by part of speech "
        + ", ".join(f"{pos} {n}" for pos, n in figures["by part of speech"].items())
        + "; forms by their own readings "
        + ", ".join(f"{k}: {n}" for k, n in figures["own readings per form"].items())
        + f"; {figures['forms reading the present indicative and subjunctive alike']} forms read the present "
        "indicative and subjunctive of one person and number"
    )
    return summary


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--max-lemmas", type=int, default=60000)
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    from wordfreq import top_n_list, zipf_frequency

    zipf = {}

    def frequency(word):
        if word not in zipf:
            zipf[word] = zipf_frequency(word, "it")
        return zipf[word]

    def foreign(word):
        """The highest Zipf frequency English, French and Spanish give `word` (D3's test)."""
        return max(zipf_frequency(word, lang) for lang in ("en", "fr", "es"))

    reduce(
        a.work,
        a.max_lemmas,
        a.pack_version,
        frequency,
        lambda n: top_n_list("it", n),
        report=lambda line: print(line, file=sys.stderr),
        foreign=foreign,
    )


if __name__ == "__main__":
    main()
