# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""it-en's own reading of the English Wiktionary's Italian section (add-lingua-pack-it-en D4,
refine-lingua-it-en-glosses).

A rule module of its own: imported by `reduce-it-en.py` and by its tests, never by another pair. The
English edition's rules (`reduce_edition_en.py`) are in es-en's and fr-en's pins, so what Italian
needs from the English edition goes here, beside them and never in them; fr-en keeps its own rules in
its reducer, which no other module can import. This file is part of it-en's rule digest
(`pack_sources.rule_files`): editing it re-pins it-en and no other pair.

Change 22 (add-lingua-pack-it-en) reads each entry alone: a pointer's carried meaning by a reviewed
list of wordings, acronyms off common words and names off function words, no word glossed by a
letter's name, the page's notes out (D4.1–D4.4). Change 23 (refine-lingua-it-en-glosses) reads each
headword's entries together (`HeadwordPass`), each rule measured alone on the committed tables, so
that a row reads an Italian word's meanings in the order Italian uses them (the census, rows changed
by band of the 1,000 / 5,000 / 20,000 commonest and beyond):

- the row's order (D2): MarkIT's commonest part of speech opens a function word's row, fr-en's rule
  (`reduce_french_treebank.commonest_first`) over MarkIT's counts, as it-fr imports both — 16 rows,
  all of the 1,000; a row opening on a meaning the section marks archaic, obsolete, dated or
  historical opens on its first meaning that is not — 173 rows (9 / 17 / 55 / 92); and a reviewed
  table, `FIRST_RUN`, opens 25 rows on the sense it names — 19 moving alone;
- pointers (D3): the wordings the English edition does not know read as pointers, and a pointer to a
  word of one or two letters lending that word's meanings after the row's own — 26 rows;
- carried meanings (D4): a meaning nested under a degree's wording (31 rows), a degree's (5) and a
  clipping's (7) wherever they stand, a plural's beside another part of speech (2);
- names' notes (D6): a surname's, a given name's and a town's note off a common word, `TOWNS_KEPT`
  keeping Massa — 1,260 rows (81 / 271 / 518 / 390);
- the page's notes and typography (D7): `lo`'s note, a list marker, a quoted opener;
- `CORRECTIONS` (D8, reduce-it-en.py's list): a section's entry corrected, or an entry added, by name.

Together, with reduce-it-en.py's « etc. » and `all'antica`: 1,578 rows (132 / 300 / 601 / 545), 341 in
their first sense, no lemma gained or lost; and D5's four corrections on top (`amica`, `bambina`,
`compagna`, `spagnola`).
"""

import collections
import json
import re
import types
import unicodedata

import reduce_common as common
import reduce_edition_en as english
import reduce_french_treebank as french_treebank

# Italian letters, lowercased, and an Italian word (change 9's `_TOKEN`, it-fr's): letters, words
# joined by hyphens or by an inner apostrophe, and an elided piece's or a truncation's final
# apostrophe (`l'`, `po'`). Repeated here because it-fr's reducer cannot be imported;
# test_reduce_it_en.py holds them to it-fr's.
LETTERS = "a-zàèéìíîòóùúâêôäëïöüçñ"
TOKEN = re.compile(rf"[{LETTERS}]+(?:['-][{LETTERS}]+)*'?")

# The pieces Italian's pre-pass reads before an apostrophe and splits a word at (change 6's
# `ITALIAN_ELISIONS` and `ITALIAN_ELIDED_PREPOSITIONS`, it-fr's `ELISIONS` and `ELIDED_PREPOSITIONS`):
# test_reduce_it_en.py holds them to it-fr's tables.
ELIDED_PIECES = frozenset(
    {
        "alcun'", "anch'", "bell'", "buon'", "c'", "cent'", "ch'", "ciascun'", "cinquant'", "com'", "cos'",
        "d'", "dev'", "diciott'", "diss'", "dov'", "fin'", "gl'", "gliel'", "grand'", "l'", "m'", "mezz'",
        "n'", "neanch'", "nessun'", "nient'", "novant'", "null'", "ogn'", "ottant'", "pover'", "prim'",
        "qual'", "qualcos'", "qualcun'", "quand'", "quant'", "quarant'", "quell'", "quest'", "s'", "sant'",
        "senz'", "sessant'", "settant'", "sott'", "t'", "tant'", "trent'", "tutt'", "ultim'", "un'", "v'",
        "vent'", "ventiquattr'",
        "all'", "coll'", "dall'", "degl'", "dell'", "nell'", "sull'",
    }
)  # fmt: skip
_ELIDED_START = re.compile(
    "(?:" + "|".join(re.escape(piece) for piece in sorted(ELIDED_PIECES, key=lambda p: (-len(p), p))) + rf")[{LETTERS}]"
)


def straight(text):
    """An Italian word as Italian's tables write it: in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip()).replace("’", "'")


def _gloss_text(sense):
    glosses = sense.get("glosses") if isinstance(sense, dict) else None
    return glosses[0].strip() if isinstance(glosses, list) and glosses and isinstance(glosses[0], str) else ""


def _entries(path):
    """`(entry, line)` for each line of `path`; `entry` is None for a line that is no JSON object."""
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            yield (entry if isinstance(entry, dict) else None), line


# — D4.1: a meaning a pointer carries, by a reviewed list of wordings —
#
# Italian's own shapes, whose meaning follows the pointer, read wherever they stand — each is a
# function word's own reading: a form chosen for euphony (« alternative form of a for euphony,
# especially before /a/; to, at, in »: `ad`, `ed`), the article's form (« form of the article i (“the”)
# used before a vowel…; the »: `gli`), each after its last semicolon; and a pronoun's clitic case form
# (« clitic accusative of io. me »: `mi`), after the full stop closing the pointer — the English
# edition's `_CASE_FORM` needs the case word to open the gloss and a colon or a semicolon after it; and
# the `di` family's contraction (« contraction of di + i; of the, from the »: `dei`, whose other entries
# are the plural of *dio* and two alternative forms), read as the other carrying wordings read their
# meaning (`_carried_text`), the `di` family being lemmas of their own by Italian's tokenisation.
_DI_CONTRACTION = re.compile(r"^contraction of di\b", re.IGNORECASE)
_EUPHONY = re.compile(r"\bfor euphony\b.*;\s*([^;]+?)\s*$", re.IGNORECASE | re.DOTALL)
_ARTICLE_FORM = re.compile(r"^form of the article\b.*;\s*([^;]+?)\s*$", re.IGNORECASE | re.DOTALL)
_CASE_FORM = re.compile(
    r"^(?:clitic |tonic |stressed |unstressed )?(?:nominative|accusative|dative|genitive|oblique|disjunctive)\b"
    r"[^.:;]*?\bof\s+[^.:;\s]+[.:;]\s*(.+?)\s*$",
    re.IGNORECASE | re.DOTALL,
)
# fr-en's carrying wordings (refine-lingua-fr-en-glosses D4) without its `synonym`, which would read
# `quanti` (« synonym of chi sg (“who”): those who ») « who » in place of `quanto`'s « how much, how
# many »: a contraction (« contraction of di + il; some, of the, from the »), a plural, a comparative or
# a superlative degree (« superlative degree of allegro (“very cheerful”) »). Never every wording: an
# abbreviation would read `giu`, the web's spelling of `giù`, « June ». Read only for a word whose
# lowercase entries hold nothing but such pointers — no meaning, and no other pointer, which lends its
# target's senses (the implementation's narrowing, measured): a contraction or a plural would otherwise
# open or join a row its own senses gloss — `colla` « with the » before « glue », `cuori` « when talking
# about the organ », `fusa` « spindle » before « purr » — or replace a verb form's lent senses (`cogli`,
# *cogliere*'s, « with the »).
_CARRYING_POINTER = re.compile(r"^(?:comparative degree|superlative degree|plural|contraction) of\b", re.IGNORECASE)
# The meaning such a pointer writes, read from its gloss — never from kaikki's `extra`, which drops the
# quotation marks and keeps a note as readily as a meaning (`magi` « archaic, literary, dialectal »):
# its text past a semicolon or a colon outside quotation marks and parentheses (« contraction of di +
# il; some, of the, from the », « superlative degree of bene: very well »), else its first quoted text
# (« plural of guaio (“trouble”) », « contraction of ve + lo, literally “it to you” »).
_QUOTED_MEANING = re.compile(r"“(.+?)”", re.DOTALL)
_PIECES = re.compile(rf"[{LETTERS}']+", re.IGNORECASE)
# A meaning written only in capitals is an abbreviation's expansion, not a meaning (« NE »).
_CAPITALS = re.compile(r"[A-Z0-9 .,/&-]+")


def _after_separator(gloss):
    """A gloss's text past its first semicolon or colon outside quotation marks and parentheses, or
    None."""
    depth = 0
    for i, ch in enumerate(gloss):
        if ch in "“(":
            depth += 1
        elif ch in "”)":
            depth = max(0, depth - 1)
        elif ch in ";:" and depth == 0:
            return gloss[i + 1 :].strip() or None
    return None


def _carried_text(gloss):
    """The meaning a carrying wording writes in its gloss (`_CARRYING_POINTER`, `_DI_CONTRACTION`): its
    text past a semicolon or a colon outside quotation marks and parentheses, else its first quoted text;
    None when there is none, or when it is made only of the pointer's own words (« contraction of sono +
    mi: mi sono »)."""
    quoted = _QUOTED_MEANING.search(gloss)
    meaning = _after_separator(gloss) or (quoted.group(1) if quoted else None)
    if meaning:
        pointer = gloss[: gloss.find(meaning)] if meaning in gloss else gloss
        words = {w.lower() for w in _PIECES.findall(meaning)}
        if words and words <= {w.lower() for w in _PIECES.findall(pointer)}:
            return None
    return meaning


def carried_meaning(sense, pos, alone=True, edition=english.EN):
    """The meaning a pointer sense carries (D4.1), or None: a pointer (`reduce_common._is_form_of`) worded
    as Italian's four shapes — a form for euphony, the article's form, a pronoun's clitic case form, the
    `di` family's contraction —, or, when the word's lowercase entries hold nothing else (`alone`), as one
    of `_CARRYING_POINTER`'s. A meaning in capitals only is none, and so is one made only of the pointer's
    own words (« contraction of sono + mi: mi sono »)."""
    gloss = _gloss_text(sense)
    if not gloss or not common._is_form_of(sense, gloss, edition=edition):
        return None
    euphony, article = _EUPHONY.search(gloss), _ARTICLE_FORM.match(gloss)
    case = _CASE_FORM.match(gloss) if pos == "pron" else None
    if euphony or article or case:
        meaning = (euphony or article or case).group(1)
    elif _DI_CONTRACTION.match(gloss) or (alone and _CARRYING_POINTER.match(gloss)):
        meaning = _carried_text(gloss)
    else:
        return None
    meaning = (meaning or "").strip().rstrip(".").strip()
    if not meaning or _CAPITALS.fullmatch(meaning):
        return None
    return meaning


# — D4.2: acronyms off common words, names off function words —

# The parts of speech of a function word, under which a name's entry spelled with a capital is left out
# (fr-en's `_FUNCTION_POS`): `Io` « Io » under `io`, `Meno` « Main (German river) » under `meno`.
FUNCTION_POS = frozenset({"prep", "conj", "pron", "det", "article", "particle"})
# A sense saying it is an acronym's, whatever its headword's case (`NdR` « initialism of nota della
# redazione… »), as it-fr's D5 reads « Sigle de ».
_ACRONYM_SENSE = re.compile(r"^(?:initialism|acronym|abbreviation) of\b", re.IGNORECASE)

# — D4.3: no word glossed with a letter's name —

# A sense that only names a Greek letter (« mu (Greek letter) », « koppa (Greek letter) »).
_GREEK_LETTER = re.compile(r"^[\w-]+ \(Greek letter\)\.?$", re.IGNORECASE)

# — D4.4: the page's notes, which are no glosses —

# « (all senses) » after a meaning (« work (all senses) »), and its two other wordings, « (in all
# senses) » and « (all meanings) » (« market (in all senses) », « phenomenon (all meanings) »), never one
# that says more (« (all senses, except for the furniture element) »); and a pointer to the wiki's
# categories (« to have (done something); See Category:Italian transitive verbs and … »), with the
# separator before it. A sense that is only « Same as entry above » (`Mio`) goes whole.
_NOTES = (
    re.compile(r"\s*\((?:in )?all (?:senses|meanings)\)"),
    re.compile(r"\s*[;,]?\s*See Category:.*$", re.DOTALL),
)
_NOTE_ALONE = re.compile(r"^same as (?:the )?entry above\.?$", re.IGNORECASE)

# The rules of the pre-pass, in the order they run: change 22's, reading each entry alone (D4.1–D4.4),
# then change 23's, reading a headword's entries together (`HEADWORD_RULES`); each can be read alone
# (the tests, the designs' measurements).
RULES = (
    "carried meanings",
    "acronyms and names",
    "letters",
    "notes",
    "corrections",
    "typography",
    "nested degrees",
    "pointers",
    "article notes",
    "degrees",
    "clippings",
    "plurals",
    "short pointers",
    "names' notes",
    "treebank order",
    "outdated first",
    "first run",
)
HEADWORD_RULES = RULES[4:]


class SectionReading:
    """What the section says of itself, read in a first pass, for `read_as_italian`: by word as the
    shared rules key it (lowercased), the parts of speech of its lowercase entries that are no proper
    noun's (`lowercase`), the words those entries' senses point at when they hold nothing else
    (`points_only`, None when one holds a meaning), whether one holds a sense other than a carrying
    pointer's — a meaning, or another pointer, which lends its target's senses (`others`) —, and whether
    one holds a meaning that is no letter's name (`beyond_letters`)."""

    def __init__(self, edition=english.EN):
        self.edition = edition
        self.lowercase = collections.defaultdict(set)
        self.points_only = {}
        self.beyond_letters = set()
        self.others = set()

    def read(self, entry):
        headword = straight(entry.get("word") if isinstance(entry.get("word"), str) else "")
        senses = entry.get("senses") if isinstance(entry.get("senses"), list) else []
        if not headword or headword != headword.lower() or entry.get("pos") == "name" or not senses:
            return
        self.lowercase[headword].add(entry.get("pos"))
        targets = self.points_only.get(headword, set())
        for sense in senses:
            text = _gloss_text(sense)
            if not text:
                continue
            if not (common._is_form_of(sense, text, edition=self.edition) and _CARRYING_POINTER.match(text)):
                self.others.add(headword)
            if common._is_form_of(sense, text, edition=self.edition):
                if targets is not None:
                    targets |= {
                        straight(ref.get("word"))
                        for ref in common._pointers(sense, self.edition)
                        if isinstance(ref, dict) and isinstance(ref.get("word"), str)
                    }
                continue
            targets = None
            if not _GREEK_LETTER.match(text):
                self.beyond_letters.add(headword)
        self.points_only[headword] = targets

    def name_under_a_function_word(self, entry, headword):
        """D4.2: a proper noun's entry of a capitalised headword that is no acronym, when the lowercase
        headword has an entry whose part of speech is a function word's (`FUNCTION_POS`)."""
        return (
            entry.get("pos") == "name"
            and headword != headword.lower()
            and not common._acronym(headword)
            and bool(self.lowercase.get(headword.lower(), set()) & FUNCTION_POS)
        )

    def acronym_off(self, headword, sense):
        """D4.2: an acronym's sense — its headword in capitals, or the sense saying it is one — of a
        capitalised headword whose word the section gives a lowercase entry of its own, unless that
        entry only points at the acronym itself (`pil` « alternative form of PIL »)."""
        key = headword.lower()
        if headword == key or key not in self.lowercase:
            return False
        if not (common._acronym(headword) or _ACRONYM_SENSE.match(_gloss_text(sense))):
            return False
        targets = self.points_only.get(key)
        return not (targets and targets <= {headword})

    def drops_letter(self, headword, sense):
        """D4.3: a sense only naming a Greek letter, of a word whose lowercase entry holds another
        meaning: `mi`, `chi`, `coppa`, `ni`; `eta`, `ro`, `psi` and `rho` keep theirs."""
        return bool(_GREEK_LETTER.match(_gloss_text(sense))) and headword.lower() in self.beyond_letters


def without_notes(text):
    """A sense's text without the page's notes (D4.4): None when nothing else is left."""
    if _NOTE_ALONE.match(text.strip()):
        return None
    for note in _NOTES:
        text = note.sub("", text)
    return text.strip() or None


# — Change 23 (refine-lingua-it-en-glosses): the rules reading a headword's entries together —
#
# Each rule below is it-en's own, measured alone against the committed rows (the design's *Measured*,
# criteria written before the figures), a class counted in ranked lemmas by band (1–1,000,
# 1,001–5,000, 5,001–20,000, beyond), and kept only where every row it changes among the 5,000
# commonest reads right or no worse and those of 5,001–20,000 read right in a majority. The census,
# each rule alone: D2a 16 rows (all of the 1,000), D2b 173, D2c 19, D3 26, D4a 31, D4b 5, D4c 7, D4d 2,
# D6 1,260, D7's article note 1, and, with « etc. » and D8's `all'antica` (reduce-it-en.py), 72;
# together 1,578 rows (132 / 300 / 601 / 545), 341 in their first sense, no lemma gained or lost.

# D2a — a function word's row opens on the part of speech UD Italian-MarkIT reads it as: fr-en's rule
# (`reduce_french_treebank.commonest_first`: at least 10 and twice the section's first, a function
# word's part of speech, never a proper noun, a content word never moved ahead of another), over
# MarkIT's counts, which reduce-it-en.py reads from its work folder
# (`reduce_italian_treebank.markit_pos_counts`), as it-fr reads both (change 17's D2). A conjunction
# is told coordinating by the conjunctions MarkIT's training and development sections tag CCONJ at
# least twice (it-fr's coordinators, which test_reduce_it_en.py holds to it-fr's).
COORDINATORS = frozenset({"e", "ed", "o", "ma", "sia", "oppure", "né", "cioè", "ovvero", "ossia", "nonché", "bensì"})
_ITALIAN = types.SimpleNamespace(coordinators=COORDINATORS)

# D2b — the marks of an outdated sense: a row whose first meaning carries one opens on its first
# meaning that carries none. Not « rare », « uncommon », « literary » or « poetic », measured and
# left out: `grande` would open on « really (intensifier) », `testo` on « an earthenware baking tray ».
OUTDATED = frozenset({"archaic", "obsolete", "dated", "historical"})

# D2c — the reviewed table: a row the rules miss or misread, opened on the sense the table names — by
# the headword's case, the entry's part of speech as the section writes it, and the sense's text, a
# sense the section nests under another read by its own text (`parlamento`) —, that sense first in its
# entry and that entry first in the row. A row moves entries and senses, never adds or removes one; it
# applies after D2a and D2b, on the senses D3 lends. Nineteen rows settled by the owner on 2026-10-25
# (task 0.1); the last six added on the review of #1065, where D2b and D3 opened them on a lesser
# sense, approved by the owner on 2026-10-25. The reduction fails on a row whose sense it does not
# find (`unfound_first_runs`). Editing it changes the rules' sha256, as for any rule.
FIRST_RUN = {
    "come": (
        "lowercase", "adv", "how",
        "MarkIT reads come 39 times as a subordinating conjunction and 16 as an adverb, so its rule would open "
        "the row on « as soon as »: held against the treebank, as it-fr's row",
    ),
    "loro": (
        "lowercase", "pron", "they, them",
        "MarkIT's rule opens loro on the determiner « their »; the pronoun first, as it-fr's row and card",
    ),
    "bene": (
        "lowercase", "adv", "well, nicely, OK, right",
        "the adjective « upper-class, posh, high » first; MarkIT reads 12 nouns and 8 adverbs, and its rule moves "
        "no content word",
    ),
    "gente": (
        "lowercase", "noun", "people, guys, folks (body of human beings)",
        "the archaic adjective « noble » first; every noun sense is marked archaic and poetic too, so D2b cannot "
        "choose",
    ),
    "piano": (
        "lowercase", "noun", "floor, storey (of a building)",
        "the adjective's « plane » first, and « plan » lost to the eight-sense bound; back with the noun first",
    ),
    "forte": ("lowercase", "adj", "strong", "the noun « fort, fortress » first"),
    "partito": (
        "lowercase", "noun", "party, political party (political group)",
        "the heraldic adjective « party per pale » first",
    ),
    "squadra": ("lowercase", "noun", "team, squad, group, side", "« set square » first, in the same entry"),
    "presto": ("lowercase", "adv", "soon", "the literary adjective « ready » first"),
    "valore": ("lowercase", "noun", "value", "the adverb « as much as, as many as » first"),
    "testo": ("lowercase", "noun", "text (writing)", "the other noun entry's « an earthenware lid » first"),
    "usato": ("lowercase", "adj", "used, worn", "« usual, customary » first"),
    "morire": ("lowercase", "verb", "to die", "the noun « death » first"),
    "medio": ("lowercase", "adj", "mean, average", "the noun « middle finger » first"),
    "prego": (
        "lowercase", "intj", "you're welcome, don't mention it etc.",
        "the noun « prayer » first",
    ),
    "stazione": (
        "lowercase", "noun", "station (stopping place for ground transportation)",
        "« position, posture » first, every sense marked archaic, so D2b cannot choose",
    ),
    "tratta": (
        "lowercase", "noun", "draft (financial instrument)",
        "D2b would open the row on the regional « seine fishing »",
    ),
    "signore": ("capitalised", "noun", "Mister or Sir", "D2b opens the row on « Lord, God »"),
    "compagno": (
        "lowercase", "noun", "mate, companion, partner, live-in partner, fellow, friend, pal, pardner",
        "the informal adjective « alike, like, similar » first",
    ),
    # Added on the review of #1065, approved by the owner on 2026-10-25 with the gloss diff.
    "dì": ("lowercase", "noun", "day", "D2b opens the row on « daytime », « day » being marked dated and literary"),
    "contrastare": (
        "lowercase", "verb", "to oppose, to impede, to hinder",
        "D2b opens the row on « to quarrel »",
    ),
    "contea": (
        "lowercase", "noun", "county (administrative region)",
        "the historical « county (territory of a count) » first, D2b « earldom »",
    ),
    "pro": (
        "lowercase", "prep", "for, in favor of/in favour of",
        "D2b opens the row on the noun « pro », the preposition being marked dated",
    ),
    "parlamento": (
        "lowercase", "noun", "parliament",
        "« parliament » never shows: the section nests it under « public assembly or meeting »",
    ),
    "ce": (
        "lowercase", "pron", "us",
        "D3 lends ci's pronoun after the row's own « reinforcing particle, “do” »",
    ),
}  # fmt: skip
_FIRST_RUN_CASES = frozenset({"lowercase", "capitalised"})

# D3 — the pointers the English edition does not know, read as pointers to their word: a labelled
# form or spelling (labels, then « form of » or « spelling of », the word last or before a colon:
# « Marche, Milan, Rome, and Venice form of di », « proscribed spelling of ché », « Old Italian form of
# storia »), a clitic compound (kaikki's `compound-of`, or « compound of apri, the second-person
# singular imperative form of aprire, with le »), whose meaning after its closing full stop is read
# when it writes one (`vendesi`), a truncation, and an untagged alternative form no field names
# (« # alternative form of lontra »).
_LABEL = (
    r"(?:archaic|regional|dialectal|obsolete|Old Italian|old|literary|poetic|proscribed|censored|dominant|"
    r"nonstandard|rare|popular|colloquial|informal|Tuscan|[A-Z][a-z]+)"
)
_LABELLED_FORM = re.compile(
    rf"^(?P<labels>{_LABEL}(?:(?:,\s*|,?\s+and\s+|\s+){_LABEL})*)\s+(?:form|spelling) of "
    r"(?P<target>[^\s,;:()“”]+)\s*(?:[:;].*)?\.?$"
)
_COMPOUND = re.compile(r"^compound of .*?\bform of (?P<target>[^\s,;:()]+)")
_COMPOUND_MEANING = re.compile(r"^compound of .*?, with \S+?\.\s+(?P<meaning>.+)$")
_TRUNCATION = re.compile(r"^truncation of (?P<target>[^\s,;:()]+)")
_FORM_OF_WORD = re.compile(r"\bform of (\S+?)[,\s]")
_NAMED_IN_TEXT = re.compile(r"^(?:alternative|obsolete|archaic|dated|rare|nonstandard) (?:form|spelling) of ([^\s,;:()“”]+)\.?$")
# D3 — a pointer to an Italian word of one or two letters, which the shared rules never follow
# (`reduce_common._MIN_BASE`), lends that word's meanings of its part of speech, after the row's own,
# unless the section tags the pointer so (its tags, not its wording's labels: `el`'s « archaic,
# regional, and Rome form of il » lends, `il`'s « Tuscany form of lo », tagged obsolete, does not);
# none of the lent senses so tagged, and none whose words a sense of the row already says.
_NOT_LENDING = frozenset({"archaic", "obsolete", "dated", "misspelling"})
_NOT_LENT = frozenset({"regional", "dialectal", "archaic", "obsolete", "dated"})
_ITEMS = re.compile(r"[,;]\s*")

# D4 — the meanings a pointer carries beyond change 22's wordings: a meaning nested under a degree's
# wording read by its own gloss (`minimo`, `minore`), a degree's carried meaning wherever it stands
# (`più` « more »), a clipping's (« clipping of automobile: car »), and a plural's beside another part
# of speech only (`guai` « woe!; trouble »; `fusa` keeps « purr » alone).
_NESTED_DEGREE = re.compile(r"^(?:comparative|superlative) degree of \S+\.?$", re.IGNORECASE)
_DEGREE = re.compile(r"^(?:comparative|superlative) degree of\b", re.IGNORECASE)
_CLIPPING = re.compile(r"^clipping of [^:;]+[:;]\s*(?P<meaning>.+?)\s*$")
_PLURAL = re.compile(r"^plural of\b", re.IGNORECASE)

# D6 — the names' notes a common word does not need, settled by the owner on 2026-10-25 (task 0.2)
# and narrowed on the review of #1065: a surname's (a proper noun's sense opening on « a » or « an »
# and at most three words before « surname »), a given name's (« a male given name… », « a diminutive
# of the female given name… »), and a place's that is only a town, a village, a hamlet, a frazione, a
# comune or a municipality — not a city or a capital itself (« a city and comune », « the capital
# of »; never « the Metropolitan City of … », which names its province), nor a town and province.
_SURNAME = re.compile(r"^an? (?:[\w-]+ ){0,3}surname\b", re.IGNORECASE)
_GIVEN_NAME = re.compile(
    r"^an? (?:[\w-]+ ){0,3}given name\b|^an? (?:[\w-]+ )?diminutive of the (?:male |female )?given name\b", re.IGNORECASE
)
_PLACE = re.compile(
    r"^(?:[^()]*\(\s*)?(?:an? |the )?(?:[\w-]+ ){0,2}(?:town|village|hamlet|frazione|comune|municipality|commune)\b",
    re.IGNORECASE,
)
_CITY = re.compile(r"\b(?:a|an|the)\s+(?:[a-z-]+\s+){0,2}(?:city|capital)\b|\bcapital of\b")
_AND_PROVINCE = re.compile(r"\band province\b")
# D6 — the towns a reviewed list keeps beside their word: a provincial capital the section calls a town
# alone (settled by the owner on 2026-10-25, task 0.2: Massa and Andria kept, Andria by the rule, since
# its town opens its row). Editing it changes the rules' sha256, as for any rule.
TOWNS_KEPT = {
    "massa": "Massa, the capital of the province of Massa-Carrara, which the section calls « a town in Tuscany »: "
    "without the row, massa reads « mass; crowd » alone",
}

# D7 — the page's notes and typography: an article's sense describing its form read by its last
# clause (`lo` « the form of il that is used before the so-called impure consonants…; the »), a list
# marker opening a sense, and a carried meaning the English edition leaves in its quotation marks
# (« (“sea”) (used in poetry and in names of some seas) », `mar`), read as that meaning.
_ARTICLE_NOTE = re.compile(r"^the form of \S+ (?:that is )?used before\b.*;\s*(?P<meaning>[^;]+?)\s*$", re.DOTALL)
_LIST_MARKER = "# "
_QUOTED_OPENER = re.compile(r"^\(“(?P<meaning>[^”]+)”\)\s*(?P<rest>.*)$")

# D8 — a source's slip corrected, or a meaning it leaves out added, by name (it-fr's `CORRECTIONS`, for
# it-en; reduce-it-en.py holds the list): the source (`section`, the English Wiktionary's Italian
# section; `translations`, the Italian Wiktionary's English translations), the headword as written,
# its part of speech, its texts as the source writes them, the corrected texts or None (the entry
# goes), the reason, and the page to report or suggest it to. A section's correction replaces the
# senses of the entry whose texts are exactly `text`; one that `added` writes an entry of its part of
# speech, read before the section's own entries of the word, when the word's entries of that part of
# speech read `text`, in order.
Correction = collections.namedtuple(
    "Correction", "source headword pos text corrected reason report added", defaults=(False,)
)


def unfound_first_runs(found, table=None):
    """The rows of the reviewed table (`FIRST_RUN`) whose sense the reduction did not find, each as the
    reduction names it: the page was rewritten, and the row is to be read again (D2c)."""
    table = FIRST_RUN if table is None else table
    return [f"{word} ({table[word][0]} {table[word][1]} « {table[word][2]} »)" for word in sorted(set(table) - set(found))]


class Reviewed:
    """What the reviewed lists found, recorded by `read_as_italian`: the `FIRST_RUN` rows whose sense it
    found (`first_runs`), the section's corrections that fired (`fired`), the `TOWNS_KEPT` rows that kept
    a town (`kept`)."""

    def __init__(self):
        self.first_runs = set()
        self.fired = set()
        self.kept = set()


class Meanings:
    """What the section as read says of its lowercase entries' meanings — a sense that is no pointer —,
    read before any rule reading a headword's entries together runs: by word as the shared rules key it,
    the parts of speech of its lowercase entries that hold a meaning (`poses`), and, for a word of one or
    two letters, those meanings, which a pointer to the word may lend (D3, `short`)."""

    def __init__(self, edition=english.EN):
        self.edition = edition
        self.poses = collections.defaultdict(set)
        self.short = collections.defaultdict(list)

    def read(self, entry):
        headword = straight(entry.get("word"))
        senses = entry.get("senses") if isinstance(entry.get("senses"), list) else []
        if headword != headword.lower() or entry.get("pos") == "name" or not senses:
            return
        for sense in senses:
            if _a_meaning(sense, self.edition):
                self.poses[headword].add(entry.get("pos"))
                if len(headword) < common._MIN_BASE:
                    self.short[(headword, entry.get("pos"))].append(sense)


def _a_pointer(sense, edition):
    text = _gloss_text(sense)
    return bool(text) and common._is_form_of(sense, text, edition=edition)


def _a_meaning(sense, edition):
    return isinstance(sense, dict) and bool(_gloss_text(sense)) and not _a_pointer(sense, edition)


def _with_text(sense, text):
    return {**sense, "glosses": [text]}


def _as_pointer(sense, target):
    """A sense read as a pointer to `target` (D3), as the English edition reads an alternative form."""
    tags = [tag for tag in sense.get("tags") or () if tag != "compound-of"] + ["alt-of"]
    return {**sense, "tags": tags, "alt_of": [{"word": target}]}


class _Slot:
    """An entry of a headword's row as the pass reads it: its line in the section (None for an entry a
    correction adds), the entry, whether a rule changed it, and whether a short word lent it senses."""

    __slots__ = ("index", "entry", "changed", "lent")

    def __init__(self, index, entry, changed=False):
        self.index, self.entry, self.changed, self.lent = index, entry, changed, False

    def replace(self, entry):
        self.entry, self.changed = entry, True


class HeadwordPass:
    """The rules reading a headword's entries together (change 23), over the section as change 22's
    rules leave it, given as its lines: every case of a headword is read at once, written where its
    first entry stood when the row's order moves, each entry in its place otherwise — so the section is
    written byte for byte where no rule fires. In the order they run: `CORRECTIONS` (D8), the senses
    read alone (D7's typography, D4a, D3's wordings, D7's article note, D4b–d), a short word's pointer
    lending (D3), the names' notes (D6), then the row's order — MarkIT's (D2a), the marks' (D2b), the
    lent senses last, the reviewed table's (D2c)."""

    def __init__(self, lines, *, rules, edition, counts, first_run, towns_kept, corrections, stats, reviewed):
        self.lines, self.rules, self.edition, self.counts = lines, rules, edition, counts
        self.first_run, self.towns_kept, self.corrections = first_run, towns_kept, corrections
        self.stats, self.reviewed = stats, reviewed
        self.meanings = Meanings(edition)
        self.keys = []
        self.groups = collections.defaultdict(list)
        for index, line in enumerate(lines):
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if not isinstance(entry, dict) or not isinstance(entry.get("word"), str):
                self.keys.append(None)
                continue
            key = straight(entry["word"]).lower()
            self.keys.append(key)
            self.groups[key].append(index)
            self.meanings.read(entry)

    def write(self, dst):
        """The section as these rules read it, into `dst`."""
        later = {}
        with open(dst, "w", encoding="utf-8") as out:
            for index, line in enumerate(self.lines):
                if index in later:
                    text = later.pop(index)
                    if text is not None:
                        out.write(text)
                    continue
                key = self.keys[index]
                if key is None:
                    out.write(line)
                    continue
                slots, moved = self.read(key)
                if moved:
                    out.write("".join(self._line(slot) for slot in slots))
                    later.update((other, None) for other in self.groups[key][1:])
                    continue
                # An entry a rule took out (`by_index` lacks it) is written nowhere.
                by_index = {slot.index: self._line(slot) for slot in slots}
                for other in self.groups[key][1:]:
                    later[other] = by_index.get(other)
                if index in by_index:
                    out.write(by_index[index])
        return dst

    def _line(self, slot):
        if slot.changed:
            return json.dumps(slot.entry, ensure_ascii=False) + "\n"
        return self.lines[slot.index]

    def read(self, key):
        """A headword's row as these rules read it: `(slots, moved)`, `moved` when its entries are no
        longer the section's, in the section's order."""
        slots = self.prepared(key)
        if "names' notes" in self.rules:
            slots = self._names_read(key, slots)
        return self._ordered(key, slots)

    def prepared(self, key):
        """A headword's entries through the rules before the names' notes: the corrections, the senses
        read alone, a short word's lending."""
        slots = [_Slot(index, json.loads(self.lines[index])) for index in self.groups.get(key, ())]
        if "corrections" in self.rules and self.corrections:
            slots = self._corrected(key, slots)
        for slot in slots:
            entry = slot.entry
            senses = entry.get("senses") if isinstance(entry.get("senses"), list) else []
            headword = straight(entry.get("word"))
            read = [self._sense_read(sense, key, headword, entry.get("pos")) for sense in senses]
            if any(new is not old for new, old in zip(read, senses)):
                slot.replace({**entry, "senses": read})
        if "short pointers" in self.rules:
            self._short_lent(key, slots)
        return slots

    # — D8 —

    def _corrected(self, key, slots):
        added = []
        for correction in self.corrections:
            if correction.source != "section" or correction.headword.lower() != key:
                continue
            if correction.added:
                if not any(straight(s.entry.get("word")) == correction.headword for s in slots):
                    continue
                ofpos = [
                    s for s in slots
                    if straight(s.entry.get("word")) == correction.headword and s.entry.get("pos") == correction.pos
                ]  # fmt: skip
                if tuple(_gloss_text(x) for s in ofpos for x in s.entry.get("senses") or ()) != tuple(correction.text):
                    continue
                self.reviewed.fired.add(correction)
                self.stats["corrections"] += 1
                senses = [{"glosses": [text]} for text in correction.corrected or ()]
                if senses:
                    added.append(_Slot(None, {"word": correction.headword, "pos": correction.pos, "senses": senses}, True))
                continue
            for slot in slots:
                entry = slot.entry
                if straight(entry.get("word")) != correction.headword or entry.get("pos") != correction.pos:
                    continue
                if tuple(_gloss_text(x) for x in entry.get("senses") or ()) != tuple(correction.text):
                    continue
                self.reviewed.fired.add(correction)
                self.stats["corrections"] += 1
                if correction.corrected is None:
                    slots = [s for s in slots if s is not slot]
                else:
                    by_text = {_gloss_text(x): x for x in entry.get("senses") or ()}
                    slot.replace({**entry, "senses": [by_text.get(t) or {"glosses": [t]} for t in correction.corrected]})
                break
        return added + slots

    # — D3, D4, D7: a sense read alone —

    def _sense_read(self, sense, key, headword, pos):
        text = _gloss_text(sense)
        if not isinstance(sense, dict) or not text:
            return sense
        rules, edition, stats = self.rules, self.edition, self.stats
        glosses = sense.get("glosses")
        own = headword == key and pos != "name"
        if "typography" in rules and text.startswith(_LIST_MARKER):
            sense = _with_text(sense, text[len(_LIST_MARKER) :].strip())
            text = _gloss_text(sense)
            stats["list markers out"] += 1
        if "typography" in rules:
            quoted = _QUOTED_OPENER.match(text)
            if quoted:
                sense = _with_text(sense, (quoted["meaning"] + (" " + quoted["rest"] if quoted["rest"] else "")).strip())
                text = _gloss_text(sense)
                stats["quoted openers read as their meaning"] += 1
        if (
            "nested degrees" in rules
            and isinstance(glosses, list)
            and len(glosses) >= 2
            and _NESTED_DEGREE.match(glosses[0].strip())
            and not _a_pointer(sense, edition)
        ):
            sense = {**sense, "glosses": [english.english_typography(glosses[-1])]}
            text = _gloss_text(sense)
            stats["meanings nested under a degree read"] += 1
        if "pointers" in rules and own and not _a_pointer(sense, edition):
            found = _LABELLED_FORM.match(text) or _TRUNCATION.match(text) or _COMPOUND.match(text)
            if found or "compound-of" in (sense.get("tags") or ()):
                target = found["target"] if found else None
                if not target:
                    named = _FORM_OF_WORD.search(text)
                    target = named.group(1) if named else key
                carried = _COMPOUND_MEANING.match(text)
                if carried:
                    sense = _with_text(sense, carried["meaning"].strip())
                    stats["compounds read as their meaning"] += 1
                else:
                    sense = _as_pointer(sense, straight(target).lower())
                    stats["senses read as pointers"] += 1
        if (
            "pointers" in rules
            and own
            and _a_pointer(sense, edition)
            and not common._pointers(sense, edition)
            and not (set(sense.get("tags") or ()) & edition.pointer_tags)
        ):
            named = _NAMED_IN_TEXT.match(text)
            if named:
                sense = _as_pointer(sense, straight(named.group(1)).lower())
                stats["senses read as pointers"] += 1
        if "article notes" in rules and pos in ("article", "det") and headword == key:
            note = _ARTICLE_NOTE.match(text)
            if note:
                sense = _with_text(sense, note["meaning"])
                text = _gloss_text(sense)
                stats["article notes read by their last clause"] += 1
        if "degrees" in rules and own and _a_pointer(sense, edition) and _DEGREE.match(text):
            meaning = _carried_text(text)
            if meaning:
                sense = english._as_meaning(sense, english.english_typography(meaning.strip().rstrip(".")))
                stats["degrees read as their meaning"] += 1
        if "clippings" in rules and own and _a_pointer(sense, edition):
            clipped = _CLIPPING.match(text)
            if clipped:
                sense = english._as_meaning(sense, english.english_typography(clipped["meaning"].rstrip(".")))
                stats["clippings read as their meaning"] += 1
        if "plurals" in rules and own and _a_pointer(sense, edition) and _PLURAL.match(text):
            poses = self.meanings.poses.get(key, set())
            if poses - {pos} and pos not in poses:
                meaning = _carried_text(text)
                if meaning:
                    sense = english._as_meaning(sense, english.english_typography(meaning))
                    stats["plurals read as their meaning beside another part of speech"] += 1
        return sense

    # — D3: a short word's pointer lends —

    def _shared_rules_lend(self, slots):
        """Whether the shared rules lend the row a word's senses: it holds no meaning, and a pointer of
        it names a word of three letters or more with a meaning of its part of speech."""
        edition = self.edition
        if any(_a_meaning(x, edition) for s in slots for x in s.entry.get("senses") or ()):
            return False
        for slot in slots:
            pos = slot.entry.get("pos")
            for sense in slot.entry.get("senses") or ():
                if not (isinstance(sense, dict) and _a_pointer(sense, edition)):
                    continue
                for ref in common._pointers(sense, edition):
                    word = ref.get("word") if isinstance(ref, dict) else None
                    if (
                        isinstance(word, str)
                        and len(word.strip()) >= common._MIN_BASE
                        and pos in self.meanings.poses.get(straight(word).lower(), ())
                    ):
                        return True
        return False

    def _short_lent(self, key, slots):
        edition = self.edition
        if self._shared_rules_lend(slots):
            return
        for slot in slots:
            entry = slot.entry
            pos = entry.get("pos")
            if straight(entry.get("word")) != key or pos == "name":
                continue
            senses = entry.get("senses") or []
            if any(_a_meaning(x, edition) for x in senses):
                continue
            read, changed = [], False
            for sense in senses:
                if isinstance(sense, dict) and _a_pointer(sense, edition):
                    refs = [
                        ref.get("word") for ref in common._pointers(sense, edition)
                        if isinstance(ref, dict) and isinstance(ref.get("word"), str)
                    ]  # fmt: skip
                    target = straight(refs[0]).lower() if refs else ""
                    if (
                        target
                        and len(target) < common._MIN_BASE
                        and target != key
                        and self.meanings.short.get((target, pos))
                        and not (set(sense.get("tags") or ()) & _NOT_LENDING)
                    ):
                        lent = self._lent_senses(slot, slots, read, target, pos)
                        if lent:
                            read.extend(english._as_meaning(x, _gloss_text(x)) for x in lent)
                            changed = slot.lent = True
                            self.stats["pointers to a short word lending its meanings"] += 1
                            continue
                read.append(sense)
            if changed:
                slot.replace({**entry, "senses": read})

    def _lent_senses(self, slot, slots, read, target, pos):
        """The meanings `target` lends a pointer of `slot`: none tagged `_NOT_LENT`, none whose words a
        sense of the row says already, at most as many as the eight-sense bound leaves room for after the
        row's own."""
        edition = self.edition
        own = []
        for other in slots:
            if other is slot:
                continue
            for sense in other.entry.get("senses") or ():
                if _a_meaning(sense, edition) and _gloss_text(sense) not in own:
                    own.append(_gloss_text(sense))
        said = {item.strip().lower() for text in own for item in _ITEMS.split(text)}
        room = common.WORD_GLOSS["max_senses"] - len(own) - sum(
            1 for x in read if isinstance(x, dict) and not _a_pointer(x, edition)
        )
        lent = []
        for sense in self.meanings.short[(target, pos)]:
            if set(sense.get("tags") or ()) & _NOT_LENT:
                continue
            items = {item.strip().lower() for item in _ITEMS.split(_gloss_text(sense))}
            if items <= said or _gloss_text(sense) in [_gloss_text(x) for x in lent]:
                continue
            lent.append(sense)
        return lent[: max(0, room)]

    # — D6: names' notes —

    def _names(self, key, slots):
        return [
            k for k, slot in enumerate(slots)
            if slot.entry.get("pos") == "name"
            and straight(slot.entry.get("word")) != key
            and not common._acronym(straight(slot.entry.get("word")))
        ]  # fmt: skip

    def _lends(self, key, slots):
        """Whether a lowercase pointer of the row names a word of three letters or more with a meaning of
        its part of speech, which the shared rules would lend."""
        for slot in slots:
            entry = slot.entry
            if straight(entry.get("word")) != key or entry.get("pos") == "name":
                continue
            for sense in entry.get("senses") or ():
                if not (isinstance(sense, dict) and _a_pointer(sense, self.edition)):
                    continue
                for ref in common._pointers(sense, self.edition):
                    word = ref.get("word") if isinstance(ref, dict) else None
                    if (
                        isinstance(word, str)
                        and len(word.strip()) >= common._MIN_BASE
                        and entry.get("pos") in self.meanings.poses.get(word.strip().lower(), ())
                    ):
                        return True
        return False

    def _name_notes(self, key, slots):
        """The names' notes a row leaves out: `{(slot, sense): kind}`, and the `TOWNS_KEPT` row it kept
        a town by, or None."""
        names = self._names(key, slots)
        if not names:
            return {}, None
        edition = self.edition
        first = next((s for s in slots if any(_a_meaning(x, edition) for x in s.entry.get("senses") or ())), None)
        opens_on_name = first is not None and first.entry.get("pos") == "name"
        own_meaning = key in self.meanings.poses
        kinds, kept = {}, None
        for k in names:
            for n, sense in enumerate(slots[k].entry.get("senses") or ()):
                if not _a_meaning(sense, edition):
                    continue
                text = _gloss_text(sense)
                kind = None
                if _SURNAME.match(text):
                    kind = "surname"
                elif _GIVEN_NAME.match(text) and own_meaning and not opens_on_name:
                    kind = "given name"
                elif _PLACE.match(text) and not _CITY.search(text) and not _AND_PROVINCE.search(text) and not opens_on_name:
                    if key in self.towns_kept:
                        kept = key
                    else:
                        kind = "place"
                kinds[(k, n)] = kind
        # A lowercase pointer is the row's common meaning only when nothing of the names stays, so that
        # it lends (`obbiettivo`): else the row would be left with another name alone (`clinton`).
        common_word = own_meaning or (all(kinds.values()) and self._lends(key, slots))
        if not common_word:
            return {}, None
        return {at: kind for at, kind in kinds.items() if kind}, kept

    def _names_read(self, key, slots):
        """D6 over a row: the notes out, a proper noun's entry left with no sense out with them."""
        notes, kept = self._name_notes(key, slots)
        if key not in self.meanings.poses and self._through_a_name(key, slots):
            notes, kept = self._name_notes(key, slots)
        if kept and kept in self.towns_kept:
            self.reviewed.kept.add(kept)
            self.stats["towns kept by the reviewed list"] += 1
        read = []
        for k, slot in enumerate(slots):
            gone = [n for (at, n) in notes if at == k]
            if gone:
                for n in gone:
                    self.stats[f"{notes[(k, n)]}s' notes out"] += 1
                senses = [x for n, x in enumerate(slot.entry.get("senses") or ()) if n not in gone]
                if not senses:
                    self.stats["names' entries left with no sense"] += 1
                    continue
                slot.replace({**slot.entry, "senses": senses})
            read.append(slot)
        return read

    def _through_a_name(self, key, slots):
        """D6: a name whose senses only point at another name, whose notes the rules leave out, reads that
        name's senses as the section writes them, before the notes go (`pinuccio` keeps « a diminutive
        of the male given name Giuseppe »)."""
        edition = self.edition
        names = self._names(key, slots)
        for k in names:
            if any(_a_meaning(x, edition) for j in names for x in slots[j].entry.get("senses") or ()):
                return False
            targets = [
                straight(ref.get("word")).lower()
                for sense in slots[k].entry.get("senses") or ()
                if isinstance(sense, dict) and _a_pointer(sense, edition)
                for ref in common._pointers(sense, edition)
                if isinstance(ref, dict) and isinstance(ref.get("word"), str)
            ]
            if not targets or targets[0] not in self.groups:
                continue
            # The named word's row as the rules before the notes read it, counted once, when it is read.
            counted, self.stats = self.stats, collections.Counter()
            try:
                target = self.prepared(targets[0])
            finally:
                self.stats = counted
            if not self._name_notes(targets[0], target)[0]:
                continue
            lent = [
                dict(sense) for j in self._names(targets[0], target)
                for sense in target[j].entry.get("senses") or () if _a_meaning(sense, edition)
            ]  # fmt: skip
            if lent:
                slots[k].replace({**slots[k].entry, "senses": lent})
                self.stats["names read through the name they point at"] += 1
                return True
        return False

    # — D2: the row's order —

    def _ordered(self, key, slots):
        edition, stats = self.edition, self.stats
        order = list(range(len(slots)))
        if "treebank order" in self.rules and self.counts is not None:
            upos = [common.kaikki_upos(s.entry.get("pos") or "", key, studied=_ITALIAN) for s in slots]
            if len(set(upos)) >= 2:
                firsts = [u for s, u in zip(slots, upos) if not common._acronym(straight(s.entry.get("word")))]
                first = firsts[0] if firsts else upos[0]
                got = french_treebank.commonest_first(french_treebank.word_key(key), upos, first, self.counts)
                if got is not None:
                    order = got
                    stats["rows opened by MarkIT's commonest part of speech"] += 1
        if "outdated first" in self.rules:
            order = self._outdated_last(key, slots, order)
        lent = [k for k in order if slots[k].lent]
        if lent:
            order = [k for k in order if not slots[k].lent] + lent
        if "first run" in self.rules and key in self.first_run:
            order = self._first_run(key, slots, order)
        moved = order != list(range(len(slots))) or any(slot.index is None for slot in slots)
        return [slots[k] for k in order], moved

    def _outdated_last(self, key, slots, order):
        """D2b: a row opening on a meaning marked outdated opens on its first meaning that is not, that
        meaning first in its entry and that entry first; every other sense in its place."""
        edition = self.edition

        def meanings(slot):
            return [x for x in slot.entry.get("senses") or () if _a_meaning(x, edition)]

        own = [
            k for k in order
            if straight(slots[k].entry.get("word")) == key and slots[k].entry.get("pos") != "name" and meanings(slots[k])
        ]  # fmt: skip
        first = next((k for k in order if meanings(slots[k])), None)
        if not own or first not in own or not set(meanings(slots[first])[0].get("tags") or ()) & OUTDATED:
            return order
        for k in own:
            fresh = [x for x in meanings(slots[k]) if not set(x.get("tags") or ()) & OUTDATED]
            if fresh:
                senses = slots[k].entry.get("senses") or []
                slots[k].replace({**slots[k].entry, "senses": [fresh[0]] + [x for x in senses if x is not fresh[0]]})
                self.stats["rows opened on a meaning not outdated"] += 1
                return [k] + [j for j in order if j != k]
        return order

    def _first_run(self, key, slots, order):
        """D2c: the row opened on the sense the reviewed table names, a nested sense read by its own text."""
        case, pos, text, _ = self.first_run[key]

        def nested(sense):
            glosses = sense.get("glosses") or []
            return len(glosses) > 1 and isinstance(glosses[-1], str) and glosses[-1].strip() == text

        for k in order:
            entry = slots[k].entry
            if (straight(entry.get("word")) == key) != (case == "lowercase") or entry.get("pos") != pos:
                continue
            senses = [x for x in entry.get("senses") or () if isinstance(x, dict)]
            first = next((x for x in senses if _gloss_text(x) == text), None)
            if first is None:
                inner = next((x for x in senses if nested(x)), None)
                if inner is None:
                    continue
                rest = [x for x in entry.get("senses") or () if x is not inner]
                first = {**inner, "glosses": [text]}
            else:
                rest = [x for x in entry.get("senses") or () if x is not first]
            slots[k].replace({**entry, "senses": [first, *rest]})
            self.reviewed.first_runs.add(key)
            self.stats["rows opened by the reviewed table"] += 1
            return [k] + [j for j in order if j != k]
        return order


def _read_alone(src, reading, rules, stats, edition):
    """Change 22's rules (D4.1–D4.4) over `src`, entry by entry: each line of the section as they read
    it, ending in a line break."""
    for entry, line in _entries(src):
        if entry is None or not isinstance(entry.get("senses"), list):
            yield line if line.endswith("\n") else line + "\n"
            continue
        headword = straight(entry.get("word") if isinstance(entry.get("word"), str) else "")
        pos = entry.get("pos")
        if "acronyms and names" in rules and reading.name_under_a_function_word(entry, headword):
            stats["names under a function word"] += 1
            continue
        senses = entry["senses"]
        read = []
        for sense in senses:
            if not isinstance(sense, dict):
                read.append(sense)
                continue
            if "acronyms and names" in rules and reading.acronym_off(headword, sense):
                stats["acronym senses off a common word"] += 1
                continue
            if "letters" in rules and reading.drops_letter(headword, sense):
                stats["Greek letters beside a common word"] += 1
                continue
            if "carried meanings" in rules and pos != "name" and not common._acronym(headword):
                meaning = carried_meaning(sense, pos, headword.lower() not in reading.others, edition)
                if meaning:
                    sense = english._as_meaning(sense, english.english_typography(meaning))
                    stats["pointers read as their meaning"] += 1
            if "notes" in rules:
                glosses = sense.get("glosses")
                if isinstance(glosses, list) and glosses and isinstance(glosses[0], str):
                    text = without_notes(glosses[0])
                    if text is None:
                        stats["senses that are only a note"] += 1
                        continue
                    if text != glosses[0]:
                        stats["notes taken out"] += 1
                        sense = {**sense, "glosses": [text, *glosses[1:]]}
            read.append(sense)
        if senses and not read:
            stats["entries left with no sense"] += 1
            continue
        if read != senses:
            yield json.dumps({**entry, "senses": read}, ensure_ascii=False) + "\n"
        else:
            yield line if line.endswith("\n") else line + "\n"


def read_as_italian(
    src,
    dst,
    *,
    edition=english.EN,
    rules=RULES,
    stats=None,
    counts=None,
    first_run=None,
    towns_kept=None,
    corrections=(),
    reviewed=None,
):
    """The section as it-en reads an Italian word (change 22's D4, change 23), written to `dst`: a
    pre-pass after the English edition's `read_as_meanings` and before its etymology merging, which
    reuses the edition's `_as_meaning` and `english_typography` and edits neither. Change 22's rules
    read each entry alone, in two passes over `src` (`SectionReading`, then the rewrite):

    - **A meaning a pointer carries** (D4.1, `carried_meaning`): read in its place, as the meaning the
      reviewed wordings write after their target — never in a name's or an acronym's entry: `del`
      « some, of the, from the », `dei` « of the, from the », `ed` « and », `ad` « to, at, in », `gli`
      « the », `mi` « me ».
    - **Acronyms off common words, names off function words** (D4.2): an acronym's sense goes from a
      word with a lowercase entry of its own unless that entry only points at the acronym (`ad` loses
      `AD` « CEO », `pil` keeps `PIL`'s); a proper noun's entry goes from a word with a function word's
      entry (`Io` off `io`, `Meno` off `meno`).
    - **No word glossed with a letter's name** (D4.3): a Greek letter's sense goes from a word holding
      a common word's meaning (`mi` « mu (Greek letter) »); a word that is only the letter keeps it.
    - **The page's notes out** (D4.4, `without_notes`): « (all senses) » and its wordings « (in all
      senses) » and « (all meanings) », « See Category:… », « Same as entry above ».

    Change 23's rules then read each headword's entries together, every case of it (`HeadwordPass`),
    measured alone on the committed tables (the census by rule, design *Measured*):

    - **Corrections** (D8): `corrections` (reduce-it-en.py's `CORRECTIONS`, the section's), each
      correcting an entry, or adding one read before the section's own (`amica` « friend »).
    - **The page's typography** (D7): a list marker out, a quoted opener read as its meaning (`mar`
      « sea (used in poetry…) ») — 2 senses.
    - **Nested degrees** (D4a): a meaning nested under a degree's wording read by its own gloss
      (`minimo`, `minore`) — 31 rows.
    - **Pointers** (D3): a labelled form or spelling, a clitic compound (its meaning after its closing
      full stop read where it writes one), a truncation and an untagged alternative form read as
      pointers to their word (`che` without « proscribed spelling of ché », `aprile` « April »).
    - **The article's note** (D7): `lo` « the; him; it… », the note read by its last clause — 1 row.
    - **Degrees, clippings and plurals** (D4b–d): a degree's and a clipping's carried meaning read
      wherever they stand (`più` « more », `auto` « car; bus »), a plural's beside another part of
      speech only (`guai` « woe!; trouble ») — 5, 7 and 2 rows.
    - **Short pointers** (D3): a pointer to a word of one or two letters lends that word's meanings of
      its part of speech, after the row's own (`de` → *di*, `el` « he; the », `ce` → *ci*) — with D3's
      wordings, 26 rows.
    - **Names' notes** (D6): a surname's note out of a row whose word holds a common meaning, a given
      name's out of a row a common word opens, a town's out of a common word unless the row opens on it
      or `towns_kept` keeps it (`massa`); a name pointing at another name reads its senses first; a
      proper noun's entry left with no sense goes — 1,260 rows.
    - **The row's order** (D2): MarkIT's commonest part of speech opens a function word's row
      (`counts`, `reduce_italian_treebank.markit_pos_counts`; none, no move) — 16 rows; a row opening
      on an outdated meaning opens on its first meaning that is not (`polizia` « police; polity ») —
      173 rows; the senses a short word lends come last; and the reviewed table (`first_run`,
      `FIRST_RUN` by default) opens a row on the sense it names — 25 rows, 19 moving alone.

    `rules` names the rules that run (`RULES`, all by default). `towns_kept` defaults to `TOWNS_KEPT`.
    `reviewed` (a `Reviewed`) records the reviewed rows that found what they name. `(dst, stats)`: how
    many senses and entries each rule moved. An entry left with no sense goes; a line this pass cannot
    read is written as it is, and so is an entry it does not change."""
    stats = stats if stats is not None else collections.Counter()
    reading = SectionReading(edition)
    for entry, _ in _entries(src):
        if entry is not None:
            reading.read(entry)
    lines = _read_alone(src, reading, rules, stats, edition)
    if not set(rules) & set(HEADWORD_RULES):
        with open(dst, "w", encoding="utf-8") as out:
            out.writelines(lines)
        return dst, stats
    HeadwordPass(
        list(lines),
        rules=frozenset(rules),
        edition=edition,
        counts=counts,
        first_run=FIRST_RUN if first_run is None else first_run,
        towns_kept=TOWNS_KEPT if towns_kept is None else towns_kept,
        corrections=tuple(corrections),
        stats=stats,
        reviewed=reviewed if reviewed is not None else Reviewed(),
    ).write(dst)
    return dst, stats


# — D4.5: the words without a space that Italian's pre-pass splits —


def split_word(word, forms):
    """Whether `word` is one word Italian's pre-pass splits into two tokens or more (D4.5, it-fr's
    `split_word`): no space, an Italian token beginning with an elided piece (`d'accordo`,
    `all'improvviso`, `c'è`, `senz'altro`), which the pre-pass splits there, and no form of the tables
    — a form is looked up whole. An articulated preposition written whole (`alla`) is none: its pieces
    are a preposition and an article, which a key writes as the page does."""
    return " " not in word and bool(TOKEN.fullmatch(word)) and bool(_ELIDED_START.match(word)) and word not in forms


def section_words(path, forms, wanted, *, edition=english.EN):
    """One pass over the section as read: `(split, entries)` — the words Italian's pre-pass splits
    (`split_word`), glossed as expressions are (`reduce_common.reduce_expressions`' sense rules: a
    proper noun's entry and a pointer left out, each sense whole but past `EXPRESSION_SENSE_LEN`, three
    joined across the word's entries), and the `wanted` headwords (lowercased) glossed the same way
    whatever their part of speech, a proper noun's included (D6: an acronym row's synonyms)."""
    split, entries = {}, {}
    for entry, _ in _entries(path):
        if entry is None:
            continue
        word = re.sub(r"\s+", " ", straight(entry.get("word") if isinstance(entry.get("word"), str) else "").lower())
        named = (entry.get("pos") or "") == "name"
        into = []
        if word in wanted:
            into.append(entries)
        if not named and split_word(word, forms):
            into.append(split)
        if not into:
            continue
        senses = []
        for sense in entry.get("senses") or ():
            text = _gloss_text(sense)
            if not text or common._is_form_of(sense, text, edition=edition):
                continue
            cleaned = common.clean_gloss(text, common.EXPRESSION_SENSE_LEN, edition=edition)
            if cleaned:
                senses.append(cleaned)
        if senses:
            for table in into:
                table.setdefault(word, []).append(senses)
    return tuple({w: common._join_senses(per_entry, 3) for w, per_entry in table.items()} for table in (split, entries))


def table_split_words(sources, forms, taken, *, edition=english.EN):
    """The words Italian's pre-pass splits that a translation table glosses and the section does not
    (`taken`), from each `(source, order)` in turn, as `reduce_common.fallback_expressions` glosses a
    headword with a space — which it skips for having none (D4.5: `all'inizio`, `sott'acqua`)."""
    out = {}
    for source, order in sources:
        for headword, by_pos in source.items():
            if headword in taken or headword in out or not split_word(headword, forms):
                continue
            words = []
            for found in by_pos.values():
                words.extend(w for w in found if w not in words)
            gloss, _ = common.translation_gloss({"X": words}, order, edition=edition)
            if gloss:
                out[headword] = gloss
    return out
