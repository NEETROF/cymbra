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

Outputs, in `--work`: Italian's `forms.tsv` and `freq.tsv`, its dictionary words `lexical.tsv` (none
yet: change 11's acronym rows and change 16's glosses write them), it-fr's `gloss.tsv` (empty until
change 16), `NOTICE` and `manifest.json`.

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

One lemma per form (D5, M8): a form that is a name and another word's keeps the commoner reading
(`bianca` → *bianco*; `roma` stays the city); then the override list (`OVERRIDES`, empty), MarkIT's
counts, the form's own entry, the lemma's wordfreq frequency, alphabetical order. The ranks (D6) are
wordfreq `it`'s order over its words that are not only inflected forms, dense, the 60,000 first kept,
a word whose own form reads as another giving its rank to the next; wordfreq's stems of the elided
pieces and truncations and its tokens beginning with a piece are no words (`un` and the apocopes
written without an apostrophe are), a hyphenated word is ranked on MarkIT's evidence, and no verb form
carrying clitics is ranked.
"""

import argparse
import collections
import json
import os
import re
import sys
import tarfile
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
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
# The pieces that are also apocopes written without an apostrophe (`un uomo`, `buon giorno`,
# `nessun dubbio`): words, ranked or read as forms of *uno*, *buono*, … by the dictionary's links (D6).
APOCOPES = frozenset({"un", "buon", "nessun", "alcun", "ciascun", "qualcun"})
# wordfreq's tokeniser splits a short piece off before a vowel (`l'uomo` → `l` + `uomo`) and strips a
# token's edge apostrophes (`po'` → `po`): its list counts a piece's stem at the piece's frequency.
# The stems are no words (D6) — every piece's, the elided prepositions' too, but the apocopes'
# (`APOCOPES`) — and nor are the truncations' stems that are no word of their own (`po`, `mo`, `be`;
# `da`, `di`, `fa`, `va` and `sta` are words).
WORDFREQ_STEMS = frozenset(
    {piece[:-1] for piece in ELISIONS} | {piece[:-1] for piece in ELIDED_PREPOSITIONS} | {"po", "mo", "be"}
) - APOCOPES
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
# The stress marks the section's tables write (D3), each with its bare vowel; a loan's marks (ü, ê,
# ç, ñ) are spelling, never stress, and stay.
_STRESS = {"à": "a", "è": "e", "é": "e", "ì": "i", "í": "i", "ò": "o", "ó": "o", "ù": "u", "ú": "u"}
# How far a form of a form is followed (D3): `creduta` → `creduto` → *credere* is one step.
_FOLLOW_STEPS = 3
# wordfreq's list is read this many times the lemmas kept: enough past the cut for its words of the
# cut's frequency, and for the inflected forms it skips.
_WORDFREQ_DEPTH = 6

# Homographs a person decided (D5), checked before any count: form → (lemma, reason). Empty: no
# homograph has a row, as in Spanish's and French's; M8's cost is listed in tables/it-fr/README.md.
# Editing this table changes the rules' sha256, so the tables must be reduced again, as for any rule.
OVERRIDES = {}

# The rules that decide a form's one lemma (D5), as the reduction reports them.
DECIDED_BY = ("override", "MarkIT's counts", "its own entry", "frequency", "alphabetical order")


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
    """A clitic compound's sense (« compound of the infinitive dire with me and lo »)."""
    return "compound-of" in (sense.get("tags") or ())


class Survey:
    """What the reading of the forms needs to know of the whole section first (a first pass):

    - `verbs`: the words with a verb entry holding a meaning (neither a form-of nor a compound's);
    - `form_links`: word → the words its own entries' form-of senses name (`ha` → {avere}), for the
      tables' stress marks (D3);
    - `compounds`: the words an entry gives as a clitic compound (`dirmelo`, `vattene`, `fatti`);
    - `standing`: the words with an entry holding a meaning of another part of speech than a verb,
      an adverb or an interjection (`fallo`, the foul), which stands beside a compound's.
    """

    def __init__(self):
        self.verbs = set()
        self.form_links = collections.defaultdict(set)
        self.compounds = set()
        self.standing = set()

    def read(self, entry):
        word = nfc_lower(entry.get("word"))
        pos = entry.get("pos")
        senses = entry.get("senses") or []
        meaning = any(not _is_form_of(s) and not _is_compound(s) for s in senses)
        if any(_is_compound(s) for s in senses):
            self.compounds.add(word)
        elif meaning and pos == "verb":
            self.verbs.add(word)
        elif meaning and pos not in _CLITIC_POS:
            self.standing.add(word)
        for sense in senses:
            if not _is_compound(sense):
                self.form_links[word].update(nfc_lower(t) for t in _form_targets(sense))

    def clitic_infinitive(self, word):
        """`(clitic, plain verb or None)` when `word` is an infinitive carrying clitics — a verb of the
        section (a meaning of its own: `pentirsi`, `andarsene`) or one whose plain verb the section has
        (`lavarsi` → `lavare`, `porsi` → `porre`) —, else None: `persi`, `corsi` and `morsi`, forms
        of verbs ending as a reflexive does, are none."""
        m = _CLITIC_INFINITIVE.search(word)
        if not m:
            return None
        stem = word[: m.start(1)]
        plain = next((v for v in (stem + "e", stem + "re") if v in self.verbs), None)
        if plain is None and word not in self.verbs:
            return None
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
    """

    def __init__(self, survey):
        self.survey = survey
        self.candidates = collections.defaultdict(set)
        self.links = collections.defaultdict(lambda: collections.defaultdict(set))
        self.lemmas = set()
        self.poses = collections.defaultdict(set)
        self.unranked = set()
        self.stats = collections.Counter()
        # (form, word): a clitic-bearing row of a reflexive or pronominal verb, whose link to the verb
        # is dropped once the section is read (`drop_clitic_rows`).
        self._clitic_rows = set()

    def link(self, form, word, pos):
        self.candidates[form].add(word)
        self.links[form][word].add(pos)

    def read(self, entry):
        """One kaikki entry (D3, D4)."""
        word = nfc_lower(entry.get("word"))
        pos = entry.get("pos")
        if word in PIECES or (word in CONTRACTIONS and word != DAI) or not is_form(word):
            # A piece is read by its table alone (`seed_pieces`); a split contraction is no form.
            return
        senses = entry.get("senses") or []
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
        for form in self._rows(entry, word, (word,)):
            self.link(form, word, pos)
        for sense in senses:
            for target in _form_targets(sense):
                target = self.survey.plain_of(nfc_lower(target))
                if is_form(target) and target not in PIECES:
                    self.link(word, target, pos)

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

    def _written(self, text, owners):
        """A row as Italian writes it (D3): its stress marks dropped but the last letter's
        (`without_stress`), and that one too when the entry carrying the cell's form-of link to one of
        `owners` (the lemma it is filed under, and the verb whose table it is) is the bare spelling —
        `hà` → `ha`, `dò` → `do` —, not when the marked spelling's own entry carries it (`dà`)."""
        form = without_stress(text)
        plain = bare(form)
        if plain is None:
            return form
        links = self.survey.form_links
        if any(o in links.get(form, ()) for o in owners):
            return form
        if any(o in links.get(plain, ()) for o in owners):
            self.stats["final marks read bare"] += 1
            return plain
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


def read_kaikki(path, survey=None):
    """The Italian section, read entry by entry into a `Lexicon`, after its first pass (`Survey`)."""
    survey = survey or survey_kaikki(path)
    lexicon = Lexicon(survey)
    for entry in _entries(path):
        lexicon.read(entry)
    lexicon.drop_clitic_rows()
    return lexicon


def name_or_word(candidates, poses, frequency):
    """The candidates of a form that is a proper name and another word's (D5). A form whose own lemma
    entries are names alone (`bambini`, a surname; `firmato`, a given name; `argentina`) reads as the
    other word: wordfreq counts the string, which is that word's inflection too, and cannot rate the
    name apart, and a page writes a name with its capital, which the analysis sets aside as a name
    (add-lingua-italian-analysis D4). A form with a common reading of its own beside the name keeps the
    commoner reading by wordfreq, the form's against the other words' lemmas: `bianca` (4.78) and
    `chiara` (4.74) read as *bianco* (4.99) and *chiaro* (5.09), the cue ball and the egg white
    leaving with the names. `roma`, a name and a rice — one lemma string, no other word — stays the
    city. The other words are those that are no name alone. Answers how many forms each branch moved."""
    moved = collections.Counter()
    for form, opts in candidates.items():
        if "name" not in poses.get(form, ()) or form not in opts:
            continue
        words = {w for w in opts - {form} if poses.get(w) != {"name"}}
        if not words:
            continue
        if poses[form] == {"name"}:
            opts.discard(form)
            moved["names read as another word"] += 1
        elif frequency(form) > max(frequency(w) for w in words):
            if opts != {form}:
                opts.intersection_update({form})
                moved["kept the commoner reading, its own"] += 1
        else:
            opts.intersection_update(words)
            moved["kept the commoner reading, another word's"] += 1
    return moved


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


def unwritten_words(forms):
    """The words the pre-pass writes that the forms table does not list: the reduction refuses them
    (D4) — every one must be a form."""
    return [word for word in prepass_words() if word not in forms]


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
    CC BY-SA 2.0 option, while it is pinned. UD Italian-PUD is measured against, never shipped."""
    text = """Cymbra Lingua data pack — IT->FR attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), Italian section: CC BY-SA 4.0 + GFDL —
the forms and their lemmas.

wordfreq (Italian frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 — the commonest lemmas and which forms are attested.

UD Italian-MarkIT, the Universal Dependencies Italian-MarkIT treebank by Teresa Paccosi, Alessio
Palmero Aprosio and Sara Tonelli (https://github.com/UniversalDependencies/UD_Italian-MarkIT):
CC BY 4.0 — how often a form stands for each of its lemmas, to choose one, and how often a
hyphenated word occurs, to rank it; its training and development sections alone.
"""
    if morph_it:
        text += """
Morph-it!, a free morphological lexicon for the Italian language, by Marco Baroni and Eros Zanchetta
(https://docs.sslmit.unibo.it/doku.php?id=resources:morph-it), version 0.4.8: CC BY-SA 2.0 (one
of its two licences, the other being the LGPL) — the forms it lists that the Wiktionary names
nowhere, under the lemmas the tables rank.
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


def reduce(work, max_lemmas, pack_version, frequency, top_n, morph_it=MORPH_IT, report=print):
    """The reduction over the raw sources in `work`, its tables written there. `morph_it` is the
    archive's file name in `work`, or None to read the Wiktionary alone. Answers what the report
    prints, as a dict, for the measurement."""
    kaikki = os.path.join(work, KAIKKI)
    survey = survey_kaikki(kaikki)
    lexicon = read_kaikki(kaikki, survey)
    counts = treebank.markit_counts([os.path.join(work, name) for name in MARKIT])
    named = name_or_word(lexicon.candidates, lexicon.poses, frequency)
    own_words(lexicon)
    forms, ranks, step = reduce_forms(lexicon, counts, treebank.markit_zipf(counts), frequency, top_n, max_lemmas)
    decided = decisions(forms, lexicon, step, counts, frequency)
    added = {}
    if morph_it is not None:
        path = os.path.join(work, morph_it)
        if not os.path.isfile(path):
            raise SystemExit(f"it-fr: {path} is missing: Morph-it! is among it-fr's pinned sources (pack_sources.py PINNED['it-fr'])")
        added = morph_it_additions(read_morph_it(path), forms, ranks, lexicon, frequency)
        forms.update(added)
    missing = seed_pieces(forms)
    unwritten = unwritten_words(forms)
    if missing or unwritten:
        raise SystemExit(
            f"it-fr: {len(missing) + len(unwritten)} word(s) Italian's pre-pass writes are no form: "
            + ", ".join(missing + unwritten)
        )

    common.write(work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())))
    common.write(work, "freq.tsv", "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: (kv[1], kv[0]))))
    # Italian's dictionary words (lexical.tsv) and it-fr's glosses: none yet — change 11's acronym
    # rows and change 16's glosses write them (D9).
    common.write(work, "lexical.tsv", "")
    common.write(work, "gloss.tsv", "")
    common.write(work, "NOTICE", notice(morph_it is not None))
    common.write(
        work, "manifest.json", json.dumps(manifest(pack_version, morph_it is not None), indent=2, ensure_ascii=False) + "\n"
    )

    last = max(ranks, key=ranks.get)
    by_lemma = collections.Counter(added.values())
    summary = {
        "forms": len(forms),
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
    }
    report(f"reduced it-fr: forms={summary['forms']} lemmas={summary['lemmas']} (the last, {last}, at Zipf {summary['last zipf']})")
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

    reduce(
        a.work,
        a.max_lemmas,
        a.pack_version,
        frequency,
        lambda n: top_n_list("it", n),
        report=lambda line: print(line, file=sys.stderr),
    )


if __name__ == "__main__":
    main()
