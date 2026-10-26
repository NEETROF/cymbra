# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""en-it's own reading of its sources (refine-lingua-en-it-glosses, the Italian programme's change 37).

A rule module of its own — English studied, Italian native, as `reduce_italian_es.py` is it-es's —:
imported by `reduce-en-it.py` and by its tests, never by another pair. `reduce_common.py` and the
Italian edition's rules (`reduce_edition_it.py`) are not edited (D1): they are other pairs' too, or
will be (fr-it, es-it). This file is part of en-it's rule digest (`pack_sources.rule_files`): editing it
re-pins en-it and no other pair. Every rule reads what en-it's pin records — the Italian Wiktionary's
English section, the two translation tables and wordfreq's lists, credited in its notice —, never
another pair's tables.

Three passes, in the design's order (D1), each rule named in `RULES` so that it can be measured alone
(the design's *Measured*: rows changed among the ranks 1–1,000 / 1,001–5,000 / 5,001–20,000 / beyond):

- **The section's entries** (`read_section`, after `italian.read_as_meanings`, before the shared
  rules): a name's note off a word with a common entry, a given name read as its Italian name (D6:
  73 rows, `summer` « estate », `john` « Giovanni (nome) »); heraldry wherever it stands, and the
  tinctures (D8: 60, `side`, `or` « o, ovvero »); a wiki's text (D9: 1, `sandbox`); a state's code
  beside another sense (D13: 3, `ma` « media mobile »); an abbreviation's English expansion (D9: 72,
  `uk` « Regno Unito »); a pointer's wording (D7: 11, `cellphone` « telefono cellulare »); a frame
  before the translation (D5: 59, `my` « il mio, la mia, i miei, le mie »). A sense D7, D9 or D13
  would take that is the word's only one stays.
- **The tables** (`split_forms`, before `native_side`): an item holding « ə » out, a one-letter item
  joined to the word before it (D10b: 1 row, `mx` « signor/a »).
- **The rows** (`refine_rows`, after `native_side`): contractions read as pointers (D11: 55 rows, 54
  lemmas gained, `you're` « contrazione di you are »); the part of speech two signals agree on, a verb
  never yielding to a noun (D2: 199, `make` « fare, creare, fabbricare; … »); the missing main part
  of speech (D3a: 76, `prove`, `rescue` « salvare; … ») and a row of descriptions (D3b: 45, `its`
  « suo; … »), opened on the English Wiktionary's first sense; the reviewed rows, `FIRST_RUN` (D4,
  34 rows) and `CORRECTIONS` (D12, 23 rows; with D4 alone, 55 rows); whole repeated senses and
  repeated items last (D10a: 369, `national` « nazionale »).

Together, over the pinned release `lingua-pack-sources-en-it-2026.10.26`, the design measured 1,035
rows changed (154 / 314 / 402 / 165), 618 in their first sense, 54 lemmas gained and none lost, and
the coverage of the 5,000 / 10,000 / 20,000 commonest at 92.1 / 83.1 / 70.6 %.

Every word a row gains is a source's — the section's, a table's or an expression's —, but for
`BY_A_PERSON` (task 0.2, the owner's: `lot`'s « (a lot) ») and the contractions' pointers, composed by
rule (`GENERATED`, task 0.1, the owner's): the frame « contrazione di » is the section's own wording,
the English full form the rule's, and no Italian word is written.

The misspellings `CORRECTIONS` drops were listed by a census against `tables/it/forms.tsv`, read by
the measurement outside the reducer (`scripts/lingua-data/measure/en_it_misspellings.py`), never by a
rule: its 262 words unknown to Italian's forms are nearly all real Italian, so a rule would take them.
"""

import collections
import json
import re

# The rules, by the design's decisions, in the order they run.
RULES = (
    "names",  # D6
    "heraldry",  # D8
    "wiki text",  # D9
    "state codes",  # D13
    "expansions",  # D9
    "pointers",  # D7
    "frames",  # D5
    "split forms",  # D10b
    "contractions",  # D11
    "part of speech",  # D2
    "missing part of speech",  # D3a
    "described rows",  # D3b
    "reviewed rows",  # D4, D12
    "repeats",  # D10a
)

# A row holds at most this many senses once a rule adds one (the shared rules' cap).
MOST_SENSES = 8
# D2's and D3a's margin: a part of speech's best word rates at least this much above the row's first's
# on wordfreq's Italian Zipf scale.
MARGIN = 0.5
# D3b: the table's best word rates at least this on wordfreq's Italian Zipf scale.
DESCRIBED_ZIPF = 3.0
# D9: a word English writes this much above Italian (wordfreq's Zipf) is English.
ENGLISH_MARGIN = 0.5
# D9: an Italian word rates at least this on wordfreq's Italian Zipf scale.
ITALIAN_ZIPF = 3.0
# D9: a row with this many senses that are expansions alone stays as the source writes it.
EXPANSION_LIST = 4


def gloss_text(sense):
    """A sense's first gloss, or '' when it has none."""
    glosses = sense.get("glosses") if isinstance(sense, dict) else None
    return glosses[0] if isinstance(glosses, list) and glosses and isinstance(glosses[0], str) else ""


def _with_text(sense, text):
    return {**sense, "glosses": [text, *sense["glosses"][1:]]}


# ——— The section's entries (D5–D9, D13) ———

# D6: a proper noun's note, and a given name's note naming its Italian equivalent.
_NAME_NOTE = re.compile(r"^(?:nome proprio|nome di persona|cognome\b|un cognome|il più tipico cognome)", re.IGNORECASE)
_GIVEN_NAME = re.compile(
    r"^nome proprio di persona[^,]*,.*?corrispondente all'(?:italiano|it\.)\s+([A-ZÀ-Ý][\w'’]+(?:\s+e\s+[A-ZÀ-Ý][\w'’]+)?)\s*$"
)
# D8: heraldry's wording, and the tinctures, whose sense the edition tags heraldic.
_HERALDIC = re.compile(
    r"araldic|blason|(?:al|come) cimiero|dello scudo|di uno scudo|su uno scudo|quando riferito a (?:nobili|principi)",
    re.IGNORECASE,
)
TINCTURES = frozenset({"or", "argent", "azure", "gules", "vert", "sable", "purpure", "tenné", "tenne", "sanguine", "murrey", "proper"})
# D7: a pointer's wording, in a parenthesis or an item.
_POINTER_PARENTHESIS = re.compile(r"\s*\((?:[^()]*\b(?:vedi|si veda|cfr\.?|più su)\b[^()]*)\)")
_POINTER = re.compile(r"\b(?:vedi|si veda|cfr\.?|più su)\b", re.IGNORECASE)
_VARIANT_PARENTHESIS = re.compile(r"\s*\(variante di [^()]*\)")
_VARIANT_ITEM = re.compile(r"^(?:\(ortografia classica\) )?variante(?: di .*|, soprattutto.*|)$")
_VARIANT_REGION = re.compile(r"^soprattutto nel Regno Unito$")
# D13: a US state's postal code.
_STATE = re.compile(r"^[Ss]igla (?:postale|per lo [Ss]tato)")
# D9: a wiki's text.
_WIKI = re.compile(r"\bwiki", re.IGNORECASE)
# D5: a frame's opening, and an item opening on a preposition (what a frame keeps then).
FRAME = re.compile(
    r"^(?:forma|usat[oa]|utilizzat[oa]|articolo (?:in)?determinativo|complemento|corrisponde|relativ[oa]|risposta|plurale|"
    r"in funzione|riferito|riguardante|pronome|introduce)\b",
    re.IGNORECASE,
)
_PREPOSITION = re.compile(
    r"^(?:a|al|allo|alla|all'|ai|agli|alle|di|del|dello|della|dell'|dei|degli|delle|da|dal|dalla|in|nel|nella|con|per|su|sul|sulla)\b",
    re.IGNORECASE,
)
# D9: an abbreviation's entry, an expansion before its Italian, « abbreviazione di X (Italian) ».
_ABBREVIATION_POS = frozenset({"abbrev", "X", "symbol"})
_ABBREVIATION_TAGS = frozenset({"abbreviation", "acronym", "initialism"})
_EXPANSION = re.compile(r"^([A-Z][^:]*?)\s*:\s*(.+)$")
_ABBREVIATION_OF = re.compile(r"^(?:abbreviazione|sigla|acronimo) di ([^()]+?)\s*\(([^()]+)\)$")
_LATIN = re.compile(r"[A-Za-z][A-Za-z'’]*")
_ACCENTED = re.compile(r"[À-ÿ]")
_LETTERS = re.compile(r"[^\W\d_][^\W\d_'’]*")


def _short_items(items, most=3):
    """Items of `most` words or fewer, none a frame, none opening on a preposition (D5's two guards)."""
    return bool(items) and all(
        0 < len(item.split()) <= most and not FRAME.match(item.strip()) and not _PREPOSITION.match(item.strip())
        for item in items
    )


def without_frame(text):
    """D5: a sense opening on a frame loses it when what follows, after a colon or the next comma or
    semicolon, is short items (`_short_items`): « forma possessiva di I usata come aggettivo
    possessivo, il mio, la mia » reads « il mio, la mia »; « in funzione di soggetto: egli, lui »
    reads « egli, lui ». Otherwise the sense is unchanged (`audio` « relativo all'ascolto, alla
    riproduzione sonora »)."""
    head, colon, tail = text.partition(": ")
    if colon and FRAME.match(head):
        if _short_items([item.strip() for item in tail.split(",")]):
            return tail.strip()
    items = [item.strip() for item in re.split(r"[;,]", text)]
    if len(items) >= 2 and FRAME.match(items[0]) and len(items[0].split()) >= 2 and _short_items(items[1:]):
        return ", ".join(items[1:])
    return text


def english_text(text, zipf):
    """D9: whether `text` is English — every word of it English writes `ENGLISH_MARGIN` above Italian
    (wordfreq), or, three words or more opening on a capital, two thirds of them —; never a text with
    an accented letter."""
    words = _LATIN.findall(text)
    if not words or _ACCENTED.search(text):
        return False
    english = sum(1 for w in words if zipf(w.lower(), "en") > zipf(w.lower(), "it") + ENGLISH_MARGIN)
    return english == len(words) or (len(words) >= 3 and english >= 0.66 * len(words) and text[:1].isupper())


def italian_sense(text, zipf):
    """D9: whether a sense holds an Italian word: three letters or more, `ITALIAN_ZIPF` or more on
    wordfreq's Italian list and at least its English rating."""
    return any(
        len(w) >= 3 and zipf(w.lower(), "it") >= ITALIAN_ZIPF and zipf(w.lower(), "it") >= zipf(w.lower(), "en")
        for w in _LETTERS.findall(text)
    )


def without_expansion(text, zipf):
    """D9: an abbreviation's sense without its English expansion — « United Kingdom : Regno Unito »
    reads « Regno Unito », « abbreviazione di justice of the peace (giudice di pace) » « giudice di
    pace », English items out —; '' when nothing Italian is left."""
    found = _EXPANSION.match(text)
    if found and english_text(found.group(1), zipf) and not english_text(found.group(2), zipf):
        return found.group(2).strip()
    found = _ABBREVIATION_OF.match(text)
    if found and english_text(found.group(1), zipf) and not english_text(found.group(2), zipf):
        return found.group(2).strip()
    if english_text(text, zipf):
        return ""
    items = [item.strip() for item in text.split(",")]
    kept = [item for item in items if not english_text(item, zipf)]
    return ", ".join(kept) if kept != items else text


def without_pointer(text):
    """D7: a parenthesis or an item holding « vedi », « si veda », « cfr », « più su », a « (variante
    di X) », an item « variante di X » or « variante, … » out; '' when nothing is left."""
    if not (_POINTER.search(text) or "variante di" in text):
        return text
    bare = _VARIANT_PARENTHESIS.sub("", _POINTER_PARENTHESIS.sub("", text).strip())
    items = [item.strip() for item in bare.split(",")]
    kept = [
        item
        for item in items
        if not _POINTER.search(item) and not _VARIANT_ITEM.match(item) and not _VARIANT_REGION.match(item)
    ]
    return ", ".join(kept) if kept != items else bare


def _entries(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            yield entry, line


def _common_words(entries):
    """The lowercase words with an entry that is no name's and holds a sense that is no name's note."""
    words = set()
    for entry, _ in entries:
        if isinstance(entry, dict) and entry.get("pos") != "name":
            word = (entry.get("word") or "").strip()
            if (
                word
                and word == word.lower()
                and any(gloss_text(s) and not _NAME_NOTE.match(gloss_text(s)) for s in entry.get("senses") or [])
            ):
                words.add(word)
    return words


class _Dropped:
    """A sense a rule takes out, kept aside until the guards (D7, D9, D13) have read the word whole."""

    __slots__ = ("rule", "sense")

    def __init__(self, rule, sense):
        self.rule, self.sense = rule, sense


def _read_sense(entry, sense, *, word, common_words, zipf, rules, stats):
    """One sense of the section as D5–D9 and D13 read it: the sense, rewritten or not, or `_Dropped`."""
    text = gloss_text(sense)
    if not text:
        return sense
    dropped = None
    if "names" in rules and entry.get("pos") == "name" and word[:1].isupper() and word.lower() in common_words and _NAME_NOTE.match(text):
        dropped = "names"
    elif "heraldry" in rules and (_HERALDIC.search(text) or ("heraldry" in (sense.get("topics") or []) and word.lower() in TINCTURES)):
        dropped = "heraldry"
    elif "wiki text" in rules and _WIKI.search(text) and word.lower() != "wiki":
        dropped = "wiki text"
    elif "state codes" in rules and _STATE.match(text):
        dropped = "state codes"
    if dropped == "heraldry":
        # A first item of two words or fewer, not heraldic, before any comma or semicolon and with no
        # colon, stays (`savage` « selvatico »).
        head = re.split(r"[;,]", text)[0].strip()
        if ":" not in text and re.search(r"[;,]", text) and len(head.split()) <= 2 and not _HERALDIC.search(head):
            stats["senses cut to their first item (D8)"] += 1
            return _with_text(sense, head)
    if dropped:
        return _Dropped(dropped, sense)
    read = text
    if "names" in rules:
        found = _GIVEN_NAME.match(read)
        if found:
            read = f"{found.group(1)} (nome)"
    if "expansions" in rules and (
        entry.get("pos") in _ABBREVIATION_POS or _ABBREVIATION_TAGS & set(entry.get("tags") or [])
    ):
        read = without_expansion(read, zipf)
        if not read:
            return _Dropped("expansions", sense)
    if "pointers" in rules:
        read = without_pointer(read)
    if "frames" in rules:
        read = without_frame(read)
    if read == text:
        return sense
    stats["senses rewritten (D5, D6, D7, D9)"] += 1
    return _with_text(sense, read) if read else _Dropped("pointers", sense)


def read_section(src, dst, *, zipf, rules=RULES, stats=None):
    """The Italian Wiktionary's English section, after `italian.read_as_meanings`, as change 37 reads
    it, written to `dst` (D1's first pass: D6, D8, D9, D13, D7, D5):

    - **D6**: a proper noun's note (« nome proprio di persona… », « cognome… ») goes from a word with a
      common entry holding a meaning (`summer`); a given name's note naming its Italian equivalent reads
      as that name, short — « Giovanni (nome) »; a name alone keeps its note (`scott`).
    - **D8**: a heraldic sense goes wherever it stands, and so does a tincture's (`or` « oro »); a
      sense whose first item is two words or fewer and not heraldic keeps it (`savage` « selvatico »).
    - **D9**: a sense naming a wiki goes (`sandbox`); in an abbreviation's entry, an English expansion
      goes (`uk` « Regno Unito »), only when the word keeps a sense holding an Italian word
      (`italian_sense`) and has fewer than `EXPANSION_LIST` senses that are expansions alone (`csi`
      stays).
    - **D13**: a state's code goes from a word another sense glosses (`ma` « media mobile »).
    - **D7**: a pointer's wording goes (`without_pointer`).
    - **D5**: a frame before the translation goes (`without_frame`).

    A sense D7, D9 (a wiki's) or D13 takes that is the word's only one stays; an entry left with no
    sense is not written. A line it cannot read is written as it is. `zipf(word, language)` is
    wordfreq's Zipf frequency; `rules` the rules that run (`RULES`); `stats` counts what moved."""
    stats = stats if stats is not None else collections.Counter()
    entries = list(_entries(src))
    common_words = _common_words(entries)
    planned = []
    for entry, line in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("senses"), list):
            planned.append((entry, line, None))
            continue
        word = (entry.get("word") or "").strip()
        read = [
            _read_sense(entry, sense, word=word, common_words=common_words, zipf=zipf, rules=rules, stats=stats)
            for sense in entry["senses"]
        ]
        planned.append((entry, line, read))
    # What each word keeps once every entry is read: its senses left, whether one holds an Italian
    # word, and how many of its senses are expansions alone.
    remaining, italian, expansions = collections.Counter(), set(), collections.Counter()
    for entry, _, read in planned:
        if read is None:
            continue
        word = (entry.get("word") or "").strip().lower()
        expansions[word] += sum(1 for s in read if isinstance(s, _Dropped) and s.rule == "expansions")
        remaining[word] += sum(1 for s in read if not isinstance(s, _Dropped) and gloss_text(s))
        if any(not isinstance(s, _Dropped) and italian_sense(gloss_text(s), zipf) for s in read):
            italian.add(word)
    with open(dst, "w", encoding="utf-8") as out:
        for entry, line, read in planned:
            if read is None:
                out.write(line if line.endswith("\n") else line + "\n")
                continue
            word = (entry.get("word") or "").strip().lower()
            kept = []
            for sense in read:
                if not isinstance(sense, _Dropped):
                    kept.append(sense)
                elif (remaining[word] == 0 and sense.rule in ("wiki text", "state codes", "pointers")) or (
                    sense.rule == "expansions" and (word not in italian or expansions[word] >= EXPANSION_LIST)
                ):
                    kept.append(sense.sense)  # the word's only gloss, or what would replace it is no better
                else:
                    stats[f"senses out: {sense.rule}"] += 1
            if not kept:
                continue
            if kept != entry["senses"]:
                line = json.dumps({**entry, "senses": kept}, ensure_ascii=False) + "\n"
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


# ——— The tables (D10b) ———

# Letters a one-letter item may be on its own: Italian's one-letter words.
_ITALIAN_LETTER_WORDS = ("e", "o", "a", "è", "i")


def split_forms(table, rules=RULES, stats=None):
    """D10b: a translation table (`word → {UPOS: [Italian word, …]}`) with an item holding « ə » out
    and a one-letter item after a word joined to it — `mx` « signor/a », not « signor, a ». A part of
    speech left with no word goes, and a word left with none."""
    if "split forms" not in rules:
        return table
    stats = stats if stats is not None else collections.Counter()
    out = {}
    for word, by_pos in table.items():
        kept = {}
        for upos, natives in by_pos.items():
            words = []
            for native in natives:
                if "ə" in native:
                    stats["table items out (D10b)"] += 1
                    continue
                if (len(native) == 1 and words and native.isalpha() and native not in _ITALIAN_LETTER_WORDS) or (
                    len(native) == 1 and words and native == "a" and words[-1].endswith("r")
                ):
                    stats["table items joined (D10b)"] += 1
                    words[-1] = words[-1] + "/" + native
                    continue
                words.append(native)
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


# ——— The rows (D2, D3, D4, D10a, D11, D12) ———

# Parts of speech compared as classes (D2): a determiner, an adjective, a pronoun and a numeral as one;
# the two conjunctions as one; a verb and an auxiliary as one.
CLASSES = {
    "DET": "D", "ADJ": "D", "PRON": "D", "NUM": "D", "SCONJ": "C", "CCONJ": "C", "VERB": "V", "AUX": "V",
    "NOUN": "N", "PROPN": "N", "ADV": "A", "ADP": "P", "INTJ": "I",
}  # fmt: skip
# The English Wiktionary's parts of speech, as UPOS.
_UPOS = {
    "verb": "VERB", "noun": "NOUN", "adj": "ADJ", "adv": "ADV", "prep": "ADP", "conj": "CCONJ", "pron": "PRON",
    "det": "DET", "intj": "INTJ", "num": "NUM", "article": "DET", "particle": "ADV", "name": "PROPN",
    "prep_phrase": "ADP", "phrase": "X",
}  # fmt: skip
_PARENTHESIS = re.compile(r"\s*\([^()]*\)")


def _class(upos):
    return CLASSES.get(upos, upos)


def first_senses(path, typography):
    """The English Wiktionary's first translated sense of each English word (D2, D3), from the direct
    file as derived: `word → (UPOS, [Italian word, …])` — the page's first entry listing translations,
    its first sense's words, at most three, each without its parentheses, in the Italian edition's
    `typography`, none a suffix, none of one letter, none holding « ə ». A capitalised headword is not
    read."""
    first = {}
    for entry, _ in _entries(path):
        if not isinstance(entry, dict):
            continue
        word = (entry.get("word") or "").strip()
        if not word or word != word.lower() or word in first:
            continue
        listed = [t for t in entry.get("translations") or [] if isinstance(t, dict) and t.get("word")]
        if not listed:
            continue
        sense = listed[0].get("sense")
        words = []
        for translation in listed:
            if translation.get("sense") != sense:
                break
            native = typography(_PARENTHESIS.sub("", translation["word"]).strip())
            if native and native not in words and "ə" not in native and not native.startswith("-") and len(native) > 1:
                words.append(native)
        if words:
            first[word] = (_UPOS.get(entry.get("pos"), "X"), words[:3])
    return first


def split_items(sense):
    """A sense's comma items, outside parentheses and « »."""
    out, depth, current, i = [], 0, "", 0
    while i < len(sense):
        c = sense[i]
        if c in "(«":
            depth += 1
        elif c in ")»":
            depth = max(0, depth - 1)
        if c == "," and depth == 0 and sense[i + 1 : i + 2] == " ":
            out.append(current)
            current, i = "", i + 2
            continue
        current += c
        i += 1
    out.append(current)
    return out


def _holds(sense, words):
    """Whether a sense holds one of `words`: as an item (its parentheses aside), or, a word of one
    token, as a token."""
    items = {_PARENTHESIS.sub("", item).strip().lower() for item in sense.split(",")}
    tokens = set(re.findall(r"[\w'’]+", sense.lower()))
    return any(w.lower() in items or (" " not in w and w.lower() in tokens) for w in words)


_DESCRIBING = re.compile(r"^(?:possessivo|stesso significato|componente|indica|esprime)\b")


def describes(sense):
    """D3b: a sense that describes the word rather than translating it — a frame's opening, or items
    of four words or more alone."""
    if FRAME.match(sense) or _DESCRIBING.match(sense):
        return True
    return all(len(item.split()) >= 4 for item in split_items(sense))


def _expand(runs):
    out = []
    for upos, n in runs:
        out += [upos] * n
    return out


def _compress(parts):
    runs = []
    for upos in parts:
        if runs and runs[-1][0] == upos:
            runs[-1] = (upos, runs[-1][1] + 1)
        else:
            runs.append((upos, 1))
    return runs


def _scores(lemma, tables, zipf):
    """Each class's best Italian word in either table (D2, D3): `class → Zipf`, in the tables' order;
    and every word the tables list for the lemma."""
    scores, words = collections.defaultdict(float), set()
    for table in tables:
        for upos, natives in (table.get(lemma) or {}).items():
            words.update(n.lower() for n in natives)
            c = CLASSES.get(upos)
            if c:
                scores[c] = max([scores[c]] + [zipf(n.lower(), "it") for n in natives])
    return scores, words


# — Contractions (D11, task 0.1, answered (a)) —

_HOSTS = frozenset({"i", "you", "he", "she", "it", "we", "they", "there", "here", "that", "what", "who", "where", "how", "when", "this", "why"})
# `'s` reads « is; has » after these alone: never after I, you, we, they (`i's`, `you's`), nor after an
# indefinite pronoun, whose `'s` is a genitive as often (`someone's`).
_IS_HOSTS = _HOSTS - {"i", "you", "we", "they"}
# `'d` reads « did » first after a wh-word.
_WH = frozenset({"what", "where", "how", "why", "when"})
_TAILS = {"'re": "are", "'m": "am", "'ve": "have", "'ll": "will"}
_IRREGULAR_NOT = {"can't": "cannot", "won't": "will not", "ain't": None, "shan't": "shall not"}
_MODALS = frozenset({"would", "should", "could", "might", "must"})
# The section's frame for a contraction it glosses (`isn't` « contrazione di is not »).
CONTRACTION = "contrazione di "


def _host(word):
    return "I" if word == "i" else word


def full_forms(word):
    """The full forms a contraction stands for, composed by rule (D11), or None: a verb and *not*, its
    host two letters or more (`can't` « cannot », `won't` « will not »); `'re`, `'m`, `'ve`, `'ll`
    after a personal pronoun, *there*, *here*, *this*, *that* or a wh-word; `'s` « is; has » after
    *he*, *she*, *it*, *there*, *here*, *this*, *that* or a wh-word; `'d` « would; had », « did »
    first after a wh-word; a modal and `'ve`; `let's` « let us »; « I » capitalised."""
    if word in _IRREGULAR_NOT:
        return [_IRREGULAR_NOT[word]] if _IRREGULAR_NOT[word] else None
    if word.endswith("n't"):
        host = word[:-3]
        return [host + " not"] if len(host) >= 2 and host.isalpha() else None
    if word == "let's":
        return ["let us"]
    if word.endswith("'ve") and word[:-3] in _MODALS:
        return [word[:-3] + " have"]
    if word.endswith("'s") and word[:-2] in _IS_HOSTS:
        host = _host(word[:-2])
        return [f"{host} is", f"{host} has"]
    if word.endswith("'d") and word[:-2] in _HOSTS:
        host = _host(word[:-2])
        return ([f"{host} did"] if word[:-2] in _WH else []) + [f"{host} would", f"{host} had"]
    for tail, full in _TAILS.items():
        if word.endswith(tail) and word[: -len(tail)] in _HOSTS:
            return [f"{_host(word[: -len(tail)])} {full}"]
    return None


def contraction_gloss(word):
    """D11: the pointer a contraction reads as, « contrazione di you are », or None."""
    full = full_forms(word)
    return "; ".join(CONTRACTION + f for f in full) if full else None


def contractions(glosses, runs, ranks, stats=None):
    """D11: each ranked contraction no source glosses reads as the pointer the section writes for those
    it glosses (`contraction_gloss`), a verb's when it is a verb and *not*; a contraction whose row is
    another contraction's pointer, borrowed through its entry (`doesn't` « contrazione di do not »),
    reads its own full form. The lemmas given a row, sorted: listed as generated (`GENERATED`)."""
    stats = stats if stats is not None else collections.Counter()
    written = []
    for lemma in ranks:
        full = full_forms(lemma)
        if not full:
            continue
        want = "; ".join(CONTRACTION + f for f in full)
        gloss = glosses.get(lemma)
        if gloss is None:
            glosses[lemma] = want
            runs[lemma] = [("VERB" if "not" in full[0] else "X", len(full))]
            stats["contractions glossed (D11)"] += 1
            written.append(lemma)
        elif gloss != want and "; " not in gloss and gloss.startswith(CONTRACTION) and _another_s(gloss[len(CONTRACTION) :], lemma):
            glosses[lemma] = want
            stats["contractions read as their own (D11)"] += 1
            written.append(lemma)
    return sorted(written)


def _another_s(full, lemma):
    """Whether `full` is the full form of a contraction other than `lemma` (`do not`, `don't`'s)."""
    host, _, tail = full.partition(" ")
    candidates = [host + "n't"] if tail == "not" else []
    return any(other != lemma and full in (full_forms(other) or ()) for other in candidates)


# — The reviewed rows (D4, D12) —
#
# A row no rule reaches, read by a person: each with its reason and the page that writes what it
# answers. Each op names its source — the section (`section`, the row as the section's definitions
# wrote it), the English Wiktionary's table (`direct`), the Italian Wiktionary's tables read backwards
# (`inverted`), an expression's row (`expression:<expression>`) — and a reviewed row whose source no
# longer writes what an op names fires on nothing for that op: the reduction's summary line names it
# (`refine_rows`), and the tests over the committed tables fail. A gloss no source gives is written by
# a person, marked so in its reason and listed in `BY_A_PERSON`.

# The row opens on `text` (`upos` its part of speech): a sense of the row moved, or the source's words
# added. `source` names one source for every item, or one per item (`connection`).
Open = collections.namedtuple("Open", "upos text source")
# A sense the source writes, out of the row.
Drop = collections.namedtuple("Drop", "text")
# An item, out of every sense of the row holding it.
DropItem = collections.namedtuple("DropItem", "item")
Reviewed = collections.namedtuple("Reviewed", "ops reason page")

_IT = "https://it.wiktionary.org/wiki/{}"
_EN = "https://en.wiktionary.org/wiki/{}"
_PERSON = "written by a person"
_OWNER_S = "task 0.2, the owner's (2026-10-26)"

# D4's `FIRST_RUN`: the commonest grammatical words and the rows no rule tells apart, each opening on a
# sense a source writes. Every word is a source's; none is written by a person.
FIRST_RUN = {
    "that": Reviewed(
        (Open("PRON", "quello, quella", "section"), Open("PRON", "che, il quale, la quale, i quali, le quali", "section")),
        "« così; introduce una proposizione… » first; D2 reads it worse", _IT.format("that"),
    ),
    "this": Reviewed((Open("ADJ", "questo, questa", "section"),), "the adverb « così » first", _IT.format("this")),
    "an": Reviewed((Open("DET", "un, uno, una, un'", "direct"),), "the section describes the article", _EN.format("an")),
    "me": Reviewed((Open("PRON", "mi, me", "direct"),), "« complemento oggetto di I », a description", _EN.format("me")),
    "can": Reviewed(
        (Open("VERB", "potere", "direct"),),
        "the noun « latta, lattina… » alone; D2 cannot reach the verb (the inverted table files « sapere » as a noun)",
        _EN.format("can"),
    ),
    "will": Reviewed(
        (Open("VERB", "usato per esprimere un'intenzione di fare qualcosa nel futuro, o un'aspettativa di qualche evento nel futuro", "section"),),
        "the auxiliary, the word's commonest use, after « volontà; testamento »", _IT.format("will"),
    ),
    "do": Reviewed((Open("VERB", "fare, compiere", "section"),), "« festa, festicciola; taglio di capelli… » first", _IT.format("do")),
    "very": Reviewed((Open("ADV", "molto, assai, -issimo, -errimo", "section"),), "the adjective « autentico, vero… » first", _IT.format("very")),
    "even": Reviewed(
        (Open("ADV", "perfino", "section"), Open("ADV", "anche", "section")),
        "the adjective « pari; piatto… » first", _IT.format("even"),
    ),
    "still": Reviewed(
        (Open("ADV", "ancora, tuttora", "section"), Open("ADV", "ancora, persino, pure (nei comparativi)", "section")),
        "the adjective « calmo, immobile, fermo » first", _IT.format("still"),
    ),
    "down": Reviewed((Open("ADV", "giù", "direct"),), "« abbasso, a basso » alone", _EN.format("down")),
    "big": Reviewed((Open("ADJ", "grande, grosso", "direct"),), "« popolare; adulto; grasso… » first", _EN.format("big")),
    "course": Reviewed((Open("NOUN", "corso, percorso, portata", "direct"),), "« quotazione, decorso… » alone", _EN.format("course")),
    "take": Reviewed((Open("VERB", "prendere, acchiappare", "section"),), "the noun « presa » first", _IT.format("take")),
    "after": Reviewed((Open("ADV", "dopo, poi, in seguito", "section"),), "the adjective « seguente » first", _IT.format("after")),
    "feel": Reviewed((Open("VERB", "sentire", "direct"),), "« percepire; toccare… » without « sentire »", _EN.format("feel")),
    "money": Reviewed((Open("NOUN", "denaro, soldi, quattrini", "section"),), "« moneta, valuta » first", _IT.format("money")),
    "way": Reviewed((Open("NOUN", "via, modo, maniera", "direct"),), "« cammino; strada; direzione » without « modo »", _EN.format("way")),
    "set": Reviewed((Open("VERB", "mettere, porre, collocare", "direct"),), "the adjective « stabilito… » first, the verb's commonest sense missing", _EN.format("set")),
    "since": Reviewed((Open("SCONJ", "da quando", "direct"),), "« poiché, perché; da » without the time's « da quando »", _EN.format("since")),
    "read": Reviewed((Open("VERB", "leggere", "section"),), "the noun « lettura » first", _IT.format("read")),
    "run": Reviewed((Open("VERB", "correre", "section"),), "« amministrare, condurre » first", _IT.format("run")),
    "about": Reviewed((Open("ADP", "su, riguardo", "direct"),), "« a proposito di » alone", _EN.format("about")),
    "through": Reviewed((Open("ADP", "attraverso, per", "direct"),), "the section describes the complements", _EN.format("through")),
    "cock": Reviewed((Open("NOUN", "gallo", "section"),), "« pollo » first, a wrong word", _IT.format("cock")),
    "cervix": Reviewed((Open("NOUN", "cervice, collo dell'utero", "section"),), "« collottola, nuca » first", _IT.format("cervix")),
    "inquisition": Reviewed((Open("NOUN", "inquisizione", "inverted"),), "« domanda, interrogazione » alone", _IT.format("inquisizione")),
    "connection": Reviewed(
        (Open("NOUN", "connessione, collegamento, coincidenza", ("direct", "inverted", "direct")),),
        "« congiuntura, coincidenza » alone; « collegamento » the inverted table's", _EN.format("connection"),
    ),
    "large": Reviewed((Open("ADJ", "grande, ampio, vasto", "direct"),), "« largo », a false friend, first", _EN.format("large")),
    "preservative": Reviewed((Open("NOUN", "conservante", "section"),), "« preservativo », a false friend, first", _IT.format("preservative")),
    "former": Reviewed(
        (Open("ADJ", "ex (indica ciò che in passato si è stati e ora non si è più)", "section"),),
        "D2 opens it on the noun « creatore »", _IT.format("former"),
    ),
    "none": Reviewed((Open("PRON", "nessuno, nessuna", "section"),), "D2 opens it on the adverb « non »", _IT.format("none")),
    "bye": Reviewed((Open("INTJ", "ciao, addio", "section"),), "a sport's status first; the section's own words after D5 takes « forma corta di goodbye » out", _IT.format("bye")),
    "conductor": Reviewed((Open("NOUN", "direttore d'orchestra", "section"),), "« bigliettaio » first", _IT.format("conductor")),
}  # fmt: skip

# D12's `CORRECTIONS`: the false friends the review names, the pages copied from another word, English
# text where Italian is due, and misspellings. Every word is a source's but `lot`'s « (a lot) ».
CORRECTIONS = {
    "some": Reviewed(
        (Drop("molti, tanti"), Drop("la folla"), Drop("molti, molte"), Open("DET", "qualche, alcuni, alcune", "inverted"), Open("PRON", "del, della", "direct")),
        "the section's senses are *many*'s page, copied", _IT.format("some"),
    ),
    "head": Reviewed((DropItem("capoccia"),), "« capoccia », colloquial, opened the row", _IT.format("head")),
    "he": Reviewed((Drop("egli, lui, esso. Usato solo in referenzia al Dio"),), "« referenzia », a misspelling, in a repeated sense", _IT.format("he")),
    "social": Reviewed((Drop("di grupo"),), "« grupo », a misspelling", _IT.format("social")),
    "top": Reviewed((DropItem("sù"),), "« sù », a misspelling of « su »", _IT.format("top")),
    "scale": Reviewed((DropItem("squaglia"),), "« squaglia », a misspelling", _IT.format("scale")),
    "tory": Reviewed((Drop("Oileán Thoraigh"), Drop("nome proprio di persona")), "the island's Irish name and a name's note", _IT.format("tory")),
    "whiplash": Reviewed(
        (Drop("colpo m di frusta, colpo di frusta"), Open("NOUN", "colpo di frusta", "direct")),
        "« colpo m di frusta », a gender mark inside the words", _IT.format("whiplash"),
    ),
    "media": Reviewed(
        (
            Drop("the middle layer of the wall of a blood vessel or lymph vessel which is composed of connective and muscular tissue. (tunica media, between the tunica interna and tunica externa)"),
            Drop("Formats for presenting information"),
            Drop("the journalists and other professionals who compose the mass communication industry"),
            Open("NOUN", "media, mass media", "direct"),
        ),
        "three English sentences", _IT.format("media"),
    ),
    "station": Reviewed((DropItem("stazine"),), "« stazine », a misspelling", _IT.format("station")),
    "original": Reviewed((DropItem("orundo"),), "« orundo », a misspelling of « oriundo »", _IT.format("original")),
    "sort": Reviewed((DropItem("essere in armoia"),), "« armoia », a misspelling of « armonia »", _IT.format("sort")),
    "trash": Reviewed(
        (
            Drop("festa chiassosa o sontuosa"), Drop("battere, percuotere, picchiare"), Drop("pestare"),
            Drop("frustare, fustigare, sferzare"), Drop("battere, sconfiggere"), Drop("trebbiare, battere"), Drop("agitare"),
            Open("NOUN", "spazzatura, immondizia, rifiuti", "direct"), Open("VERB", "scartare", "direct"),
        ),
        "the section's eight senses are *thrash*'s", _IT.format("trash"),
    ),
    "lot": Reviewed(
        (Open("ADV", "molto, parecchio (a lot)", "expression:a lot"),),
        f"the common sense is the expression *a lot*'s « molto, parecchio »; « (a lot) » {_PERSON} — {_OWNER_S}",
        _IT.format("lot"),
    ),
    "tide": Reviewed((Drop("marea, bassa"), Open("NOUN", "marea", "direct")), "« bassa », the next word of « bassa marea », split off", _IT.format("tide")),
    "prove": Reviewed((Drop("prove"), Open("VERB", "dimostrare, provare", "direct")), "« prove », the English word, a false friend", _IT.format("prove")),
    "disposable": Reviewed((Drop("disponibile"), Open("ADJ", "monouso, usa e getta", "direct")), "« disponibile », a false friend", _IT.format("disposable")),
    "ultimately": Reviewed(
        (Drop("finalmente"), Open("ADV", "in definitiva, alla fine, in fin dei conti", "direct")),
        "« finalmente », a false friend", _IT.format("ultimately"),
    ),
    "editor": Reviewed((Drop("editore/editrice"), Open("NOUN", "redattore, curatore, editore", "direct")), "« editore/editrice » alone, a false friend", _IT.format("editor")),
    "fabric": Reviewed((Drop("ordito"), Open("NOUN", "tessuto, stoffa", "inverted")), "« ordito », the warp", _IT.format("fabric")),
    "firm": Reviewed(
        (Drop("comitiva, ditta"), Open("NOUN", "ditta", "section"), Open("ADJ", "fermo, solido, stabile", "inverted")),
        "« comitiva », a false friend; the adjective missing", _IT.format("firm"),
    ),
    "passionate": Reviewed((Drop("passionale"), Open("ADJ", "appassionato", "direct")), "« passionale », a false friend", _IT.format("passionate")),
    "sacred": Reviewed((Open("ADJ", "sacro", "direct"),), "« santo » alone, the holy rather than the sacred", _IT.format("sacred")),
}  # fmt: skip

# Every gloss of en-it written by a person, by headword: the words no source gives (task 0.2). The pull
# request lists them; each is reported upstream.
BY_A_PERSON = {"lot": "(a lot)"}
# The rows composed by rule (D11, task 0.1): the frame « contrazione di » is the section's, the English
# full form the rule's (`contraction_gloss`). The pull request lists every lemma `contractions` writes.
GENERATED = "contractions"


def _the_source_writes(lemma, op, sources):
    """Whether the source an `Open` names gives every item of its words — a person's words aside."""
    text = op.text.replace(BY_A_PERSON[lemma], "").strip() if lemma in BY_A_PERSON else op.text
    items = [item.strip() for item in split_items(text)]
    names = op.source if isinstance(op.source, tuple) else (op.source,) * len(items)
    if len(names) != len(items):
        return False
    return all(item in sources(name, lemma) for item, name in zip(items, names))


def _review(lemma, row, senses, parts, sources, unfired, stats):
    """A reviewed row's ops over the row (`senses`, `parts` their parts of speech), in order: the
    senses each `Open` names first, in its order, then the rest, at most `MOST_SENSES`."""
    senses, parts = list(senses), list(parts)
    front = []
    for op in row.ops:
        if isinstance(op, Drop):
            if op.text in senses:
                i = senses.index(op.text)
                senses.pop(i)
                parts.pop(i)
            else:
                unfired.append(f"{lemma}: « {op.text} »")
        elif isinstance(op, DropItem):
            hit = False
            for i, sense in enumerate(senses):
                items = sense.split(", ")
                kept = [item for item in items if item != op.item]
                if len(kept) != len(items):
                    hit = True
                    senses[i] = ", ".join(kept)
            if not hit:
                unfired.append(f"{lemma}: « {op.item} »")
        elif op.text in senses:
            i = senses.index(op.text)
            senses.pop(i)
            front.append((op.text, parts.pop(i)))
        elif _the_source_writes(lemma, op, sources):
            front.append((op.text, op.upos))
        else:
            unfired.append(f"{lemma}: « {op.text} » ({op.source if isinstance(op.source, str) else ', '.join(op.source)})")
    kept = [(s, p) for s, p in zip(senses, parts) if s]
    row_senses = [t for t, _ in front] + [s for s, _ in kept]
    row_parts = [p for _, p in front] + [p for _, p in kept]
    stats["rows the reviewed tables write (D4, D12)"] += 1
    return row_senses[:MOST_SENSES], row_parts[:MOST_SENSES]


def _without_repeats(senses, parts):
    """D10a: « X / X » reads X, an item repeated inside a sense is listed once, and a sense every item
    of which an earlier sense holds goes."""
    seen, out_senses, out_parts = set(), [], []
    for sense, upos in zip(senses, parts):
        sense = re.sub(r"^(\S+) / \1$", r"\1", sense)
        items = [item.strip() for item in split_items(sense)]
        once = list(dict.fromkeys(items))
        keep = [] if all(item in seen for item in once) else once
        seen.update(items)
        if keep:
            out_senses.append(", ".join(keep) if keep != items else sense)
            out_parts.append(upos)
    return out_senses, out_parts


def _source_items(section, direct, inverted, expressions):
    """`sources(name, lemma)`: the items a source writes for a lemma — the section's row as
    `native_side` wrote it, a table's words, an expression's row."""
    cache = {}

    def items_of(gloss):
        return {item.strip() for sense in (gloss or "").split("; ") for item in split_items(sense)}

    def sources(name, lemma):
        key = (name, lemma)
        if key not in cache:
            if name == "section":
                cache[key] = items_of(section.get(lemma))
            elif name in ("direct", "inverted"):
                table = direct if name == "direct" else inverted
                cache[key] = {n for natives in (table.get(lemma) or {}).values() for n in natives}
            elif name.startswith("expression:"):
                cache[key] = items_of(expressions.get(name.removeprefix("expression:")))
            else:
                cache[key] = set()
        return cache[key]

    return sources


def refine_rows(glosses, runs, steps, *, ranks, first, direct, inverted, expressions, zipf, rules=RULES, stats=None):
    """The rows as change 37 reads them, in place (`glosses`, `runs`), in the design's order (D1): D11;
    then, for a row the section glossed (`steps["entries"]`), D2, D3a, D3b; then the reviewed rows
    (D4, D12); then D10a, last, for every row.

    - **D2**: a row of two classes or more opens on the class the English Wiktionary's first translated
      sense names (`first`, `first_senses`) when it is also the class whose best word, in either table
      (`direct`, `inverted`), Italian writes most, `MARGIN` above the row's first class's; its senses
      move together, the others following in their order; a verb never yields to a noun.
    - **D3a**: when that class is missing from the row and rates `MARGIN` above its first, the row
      opens on the English Wiktionary's first sense, unless a verb would open an adjective's row.
    - **D3b**: when the row's first sense describes the word (`describes`), no sense shares a word with
      either table and the first sense's best word rates `DESCRIBED_ZIPF` or more, the same.
    - Neither D3 touches a lemma of one or two letters, an interjection's row or sense, a first sense
      holding an English word, or a row already holding one of its words; a row holds at most
      `MOST_SENSES`.

    `zipf(word, language)` is wordfreq's. `(generated, unfired)`: the lemmas D11 wrote, and the reviewed
    rows' ops that found nothing, by name."""
    stats = stats if stats is not None else collections.Counter()
    section = {lemma: glosses[lemma] for lemma in steps["entries"] if lemma in glosses}
    generated = contractions(glosses, runs, ranks, stats) if "contractions" in rules else []
    tables = (direct, inverted)
    sources = _source_items(section, direct, inverted, expressions)
    reviewed = {**FIRST_RUN, **CORRECTIONS} if "reviewed rows" in rules else {}
    unfired = []
    english = {}
    for lemma in list(glosses):
        senses = glosses[lemma].split("; ")
        parts = _expand(runs[lemma])
        before = list(senses)
        if lemma in steps["entries"]:
            upos, words = first.get(lemma, ("X", []))
            agreed = CLASSES.get(upos)
            if "part of speech" in rules and len({_class(p) for p in parts}) > 1:
                scores, _ = _scores(lemma, tables, zipf)
                lead = _class(parts[0])
                best = max(scores, key=scores.get) if scores else None
                if (
                    best
                    and best == agreed
                    and not (lead == "V" and best == "N")
                    and best != lead
                    and scores[best] >= scores.get(lead, 0) + MARGIN
                    and any(_class(p) == best for p in parts)
                ):
                    order = [i for i, p in enumerate(parts) if _class(p) == best]
                    order += [i for i in range(len(senses)) if i not in order]
                    senses, parts = [senses[i] for i in order], [parts[i] for i in order]
                    stats["rows reordered (D2)"] += 1
            if "missing part of speech" in rules or "described rows" in rules:
                scores, listed = _scores(lemma, tables, zipf)
                best = max(scores, key=scores.get) if scores else None
                lead = _class(parts[0])
                fire = None
                if (
                    "missing part of speech" in rules
                    and best
                    and best == agreed
                    and words
                    and not any(_class(p) == best for p in parts)
                    and scores[best] >= scores.get(lead, 0) + MARGIN
                ):
                    fire = "D3a" if not (lead == "D" and best == "V") else "none"
                if (
                    fire is None
                    and "described rows" in rules
                    and words
                    and describes(senses[0])
                    and max(zipf(w.lower(), "it") for w in words) >= DESCRIBED_ZIPF
                    and not any(_holds(s, list(listed)) for s in senses)
                ):
                    fire = "D3b"
                if fire in ("D3a", "D3b") and ("INTJ" in parts or upos == "INTJ"):
                    fire = None
                if fire in ("D3a", "D3b") and len(lemma) < 3:
                    fire = None
                if fire in ("D3a", "D3b"):
                    if lemma not in english:
                        english[lemma] = any(english_text(w, zipf) for w in words)
                    if english[lemma]:
                        fire = None
                if fire in ("D3a", "D3b") and not any(_holds(s, words) for s in senses):
                    senses = [", ".join(words), *senses][:MOST_SENSES]
                    parts = [upos if upos != "X" else parts[0], *parts][:MOST_SENSES]
                    stats[f"rows opened on the table's first sense ({fire})"] += 1
        if lemma in reviewed:
            senses, parts = _review(lemma, reviewed[lemma], senses, parts, sources, unfired, stats)
        if "repeats" in rules:
            senses, parts = _without_repeats(senses, parts)
        if senses != before or parts != _expand(runs[lemma]):
            if senses != before:
                stats["rows changed"] += 1
            glosses[lemma] = "; ".join(senses)
            runs[lemma] = _compress(parts)
    for lemma in reviewed:
        if lemma not in glosses:
            unfired.append(f"{lemma}: no row")
    return generated, unfired
