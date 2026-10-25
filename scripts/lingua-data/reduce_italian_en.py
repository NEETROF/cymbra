# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""it-en's own reading of the English Wiktionary's Italian section (add-lingua-pack-it-en D4).

A rule module of its own: imported by `reduce-it-en.py` and by its tests, never by another pair. The
English edition's rules (`reduce_edition_en.py`) are in es-en's and fr-en's pins, so what Italian
needs from the English edition goes here, beside them and never in them; fr-en keeps its own rules in
its reducer, which no other module can import. This file is part of it-en's rule digest
(`pack_sources.rule_files`): editing it re-pins it-en and no other pair.
"""

import collections
import json
import re
import unicodedata

import reduce_common as common
import reduce_edition_en as english

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

# The rules of the pre-pass, in the order they run (D4.1–D4.4); each can be read alone (the tests, the
# design's measurements).
RULES = ("carried meanings", "acronyms and names", "letters", "notes")


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


def read_as_italian(src, dst, *, edition=english.EN, rules=RULES, stats=None):
    """The section as it-en reads an Italian word (D4), written to `dst`: a pre-pass after the English
    edition's `read_as_meanings` and before its etymology merging, in two passes over `src`
    (`SectionReading`, then the rewrite), which reuses the edition's `_as_meaning` and
    `english_typography` and edits neither.

    - **A meaning a pointer carries** (D4.1, `carried_meaning`): read in its place, as the meaning the
      reviewed wordings write after their target — never in a name's or an acronym's entry: `del`
      « some, of the, from the », `dei` « of the, from the », `ed` « and », `ad` « to, at, in », `gli`
      « the », `mi` « me ».
    - **Acronyms off common words, names off function words** (D4.2): an acronym's sense goes from a
      word with a lowercase entry of its own unless that entry only points at the acronym (`ad` loses
      `AD` « CEO », `pil` keeps `PIL`'s); a proper noun's entry goes from a word with a function word's
      entry (`Io` off `io`, `Meno` off `meno`). A name beside a common word's senses stays (`Natale`).
    - **No word glossed with a letter's name** (D4.3): a Greek letter's sense goes from a word holding
      a common word's meaning (`mi` « mu (Greek letter) »); a word that is only the letter keeps it.
    - **The page's notes out** (D4.4, `without_notes`): « (all senses) » and its wordings « (in all
      senses) » and « (all meanings) », « See Category:… », « Same as entry above ».

    `rules` names the rules that run (`RULES`, all by default). `(dst, stats)`: how many senses and
    entries each rule moved. An entry left with no sense goes; a line this pass cannot read is written
    as it is, and so is an entry it does not change."""
    stats = stats if stats is not None else collections.Counter()
    reading = SectionReading(edition)
    for entry, _ in _entries(src):
        if entry is not None:
            reading.read(entry)
    with open(dst, "w", encoding="utf-8") as out:
        for entry, line in _entries(src):
            if entry is None or not isinstance(entry.get("senses"), list):
                out.write(line if line.endswith("\n") else line + "\n")
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
                out.write(json.dumps({**entry, "senses": read}, ensure_ascii=False) + "\n")
            else:
                out.write(line if line.endswith("\n") else line + "\n")
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
