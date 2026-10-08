# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The English Wiktionary's rules (enwiktionary), for the pairs glossed in English
(generalise-lingua-gloss-reducer).

es-en reads it (add-lingua-pack-es-en): its glosses are the English Wiktionary's senses of its
Spanish entries. Its rules come from a census of kaikki's extract of the English Wiktionary's
Spanish section, the 2026-09-28 dump es-fr pins: 811,049 entries, 875,591 senses with a gloss.

- Pointers. 82.4 % of the senses are tagged `form-of` ("plural of casa", "inflection of angular:")
  and 0.7 % `alt-of` ("superseded spelling of cuórum", "abbreviation of Puebla",
  "misspelling of cachái"): kaikki names the word an `alt-of` sense points at in `alt_of`, as it
  names a form's lemma in `form_of`. Untagged, a sense still only points at a word when it opens
  « only used in » (1,332: "only used in en pos de"), « synonym of » (1,072: "synonym of pues", a
  word of the studied language, never a gloss), « see » or « used other than figuratively or
  idiomatically: see … » (86), « disused form of » (20), or names an inflection or a variant
  followed by « of » ("diminutive of figura").
- No placeholder: kaikki leaves an undefined sense without a gloss and tags it `no-gloss`; the
  shared rules skip a sense with no gloss, and a word left with no sense has no gloss.
- No dangling coordinator: the 13 senses that open on « or » or « and » are meanings (the heraldic
  « or », "and a half").
- A letter's name: "The name of the Latin script letter D/d.", and a sense that only names a letter:
  "the letter r" (`r`), or a word's place in the Spanish spelling alphabet, "the letter E in the
  Spanish spelling alphabet" (36 of es-en's glossed lemmas ended on one: `españa`, `jueves`).
- A single capital letter is no headword of a word (`without_letter_headwords`): the Spanish
  section writes a chess piece, a compass point or a title under it (`A` « bishop », `C`
  « abbreviation of caballo », `N` « abbreviation of norte »), which would gloss the letter `a`, or
  lend `c` the gloss of `caballo`.
- Casing: 98.3 % of the meaning senses open on a lower-case letter, the edition's convention for a
  foreign word's senses, so a gloss made of translation-table words keeps the case its words have.
- Long parentheses (M20): 5,724 of 147,653 meaning senses hold one of 40 characters or more
  ("a former unit of length equivalent to about 27.9 cm"). `LONG_PARENTHESIS`, below, is the
  bound a parenthesis goes at; 0 keeps them all.
- Etymologies (add-lingua-pack-es-en D5): kaikki writes one entry per etymology, so a word with two
  nouns of different origin has two noun entries, and the round-robin across a headword's entries
  (`reduce_common._join_senses_by_pos`) takes one sense of each in turn — 532 of the top 10,000
  es-en lemmas. `MERGE_SAME_POS_ETYMOLOGIES`, below, merges a word's entries of one part of speech
  before the round-robin, so the first etymology's senses come first; off, they are read as written.

The two settings are the English edition's alone: a pair glossed in French loads this module no
more than it did, so a value chosen here re-pins es-en and no other committed pair (D5; the shared
rules in `reduce_common.py` are not edited for them).

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does — tuning it never
re-pins en-fr or es-fr.
"""

import json
import re

import reduce_common as common

# M20: a parenthesis whose text runs this long or longer goes from a gloss; 0 keeps every one.
# Decided on a sample of the top 10,000 es-en lemmas reduced both ways (add-lingua-pack-es-en D5):
# 0 as committed; the owner's pick re-pins es-en alone.
LONG_PARENTHESIS = 0
# Whether a word's entries of one part of speech — its etymologies — are merged before the
# round-robin (D5, below). Decided on the same sample: off as committed.
MERGE_SAME_POS_ETYMOLOGIES = False

# A shortened or respelled form's wording (refine-lingua-es-en-glosses D3): an apocope, an
# apheresis, a syncope, a prepositional form, a pronunciation or an eye-dialect spelling — in any
# case, tagged as a pointer or not (« apheretic form of estás »).
_SHORTENED = r"(?:(?:apocopic|apheretic|syncopic|prepositional) form|(?:pronunciation|eye dialect) spelling) of\b"

# An untagged sense that only points at another word.
# Measured on the census: « see » points unless it greets (« nos vemos »: "see you later!", « ven
# acá »: "see here; come on"), and an inflection's name points unless a parenthesis defines it
# (« femenino »: "feminine (of or relating to women)", « flexión »: "inflection (a change in the form
# of a word …)"). A shortened form `read_as_meanings` reads no meaning from stays a pointer (D3).
_FORM_OF = re.compile(
    r"^(?:only used in|used other than figuratively|synonym of|see\s(?!you\b|here\b)|"
    r"(?:alternative|obsolete|archaic|dated|disused|rare|nonstandard|superseded) (?:form|spelling) of|"
    + _SHORTENED
    + r"|(?:plural|inflection|feminine|masculine|female equivalent|diminutive|augmentative|gerund|"
    r"(?:past|present) participle|infinitive|(?:first|second|third)-person)\b(?!\s*\()[^.:;]*\bof\b)",
    re.IGNORECASE,
)

# A sense naming a letter: "The name of the Latin script letter D/d.", "the letter r", "the letter E
# in the Spanish spelling alphabet" — one letter, never a word ("the letter of the law").
_LETTER = re.compile(
    r"^(?:(?:the )?name of the (?:[\w-]+ )?(?:script )?(?:letter|digraph)\b|the letter \w\b)", re.IGNORECASE
)

EN = common.Edition(
    code="en",
    form_of=_FORM_OF,
    letter=_LETTER,
    pointer_tags=frozenset({"form-of", "alt-of"}),
    pointer_fields=("form_of", "alt_of"),
    capitalised=False,
    long_parenthesis=LONG_PARENTHESIS,
)


def without_letter_headwords(src, dst):
    """The edition's entries without those whose headword is a single capital letter, written to
    `dst` — a pre-pass a reducer runs before the shared rules read the file, as it runs
    `merge_same_pos_etymologies`.

    The shared rules key a headword in lower case, and spare a word an acronym's entries only when
    the acronym has two capitals or more (`reduce_common._acronym`). The English Wiktionary's
    Spanish section writes chess pieces, compass points and titles under a capital letter — `A`
    « bishop », `C` « abbreviation of caballo », `N` « abbreviation of norte », `I` « abbreviation
    of ilustre » — so `a` opened on « bishop », and `c` borrowed the gloss of `caballo`. A letter's
    own entries (`character`) go with `reduce_common.without_letter_senses`. A line this pass
    cannot read is written as it is: the shared rules decide.
    """
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if isinstance(entry, dict):
                headword = (entry.get("word") or "").strip()
                if len(headword) == 1 and headword.isupper():
                    continue
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def merge_same_pos_etymologies(src, dst, *, merged=None):
    """The edition's entries with a word's entries of one part of speech merged into one, their
    senses in source order — a pre-pass a reducer runs before the shared rules read the file
    (add-lingua-pack-es-en D5). With `merged` false (`MERGE_SAME_POS_ETYMOLOGIES` by default),
    nothing is written and `src` is answered as it is: the setting is off.

    kaikki writes one entry per etymology, each with the page's `word` and its `pos`; a page's
    entries are consecutive in the extract, so a word's group is the run of lines sharing its
    `word`. The first entry of a part of speech keeps its other fields and gains the later ones'
    senses, so the round-robin across a word's entries (`reduce_common._join_senses_by_pos`) takes
    the first etymology's senses before the next one's. An entry with no senses merges into none:
    it keeps its place, as written, and when it is the first of its part of speech the later ones'
    senses merge into it. A line that is no JSON object — undecodable, or another JSON value — is
    left out. The headword's exact spelling is the key: an acronym's entries (« CASA ») never merge
    with the common word's.
    """
    if not (MERGE_SAME_POS_ETYMOLOGIES if merged is None else merged):
        return src
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        group_word, group = None, []  # the current word's entries, by part of speech, in order

        def flush():
            for entry in group:
                out.write(json.dumps(entry, ensure_ascii=False) + "\n")
            group.clear()

        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(entry, dict):
                continue
            if entry.get("word") != group_word:
                flush()
                group_word = entry.get("word")
            senses = entry.get("senses") or []
            same = next((e for e in group if e.get("pos") == entry.get("pos")), None) if senses else None
            if same is None:
                group.append(entry)
            else:
                same["senses"] = [*(same.get("senses") or []), *senses]
        flush()
    return dst


# — Meanings, not the page's layout (refine-lingua-es-en-glosses) —
#
# kaikki writes a nested sense with its parents' glosses first, a shortened form as a pointer that
# often carries the meaning after its target, and the edition's typography as written. The shared
# rules read a sense by its first gloss and skip a pointer, so es-en read the page's layout as
# meanings: « venir » by two sense-group labels, « su » « apocopic form of suyo », « lo » only its
# article, « como » opening on an Italian city. `read_as_meanings` rewrites the senses before the
# shared rules read them; it is the English edition's, so it re-pins es-en and no other pair.

# A parent that labels its nested senses rather than meaning anything (D2): « Figurative senses. »,
# « Senses relating to literal movement. ». A parent ending on a colon labels them too.
_SENSE_GROUP = re.compile(r"\bsenses\b", re.IGNORECASE)
# A shortened or respelled form (D3), keyed on its wording, never on a tag alone.
_SHORTENED_FORM = re.compile(r"^" + _SHORTENED + r"\s*", re.IGNORECASE)
# The target a shortened form names in its gloss, when no pointer field names it: up to a comma, a
# semicolon, a colon, an opening parenthesis or the end.
_TARGET = re.compile(r"[^,;:(]+")
# The meaning written after the target: past a comma, a semicolon or a colon, or within “ ”.
_AFTER_TARGET = re.compile(r"\s*[,;:]\s*(.+)", re.DOTALL)
_QUOTED = re.compile(r"\s*\(?“(.+?)”\)?\s*\.?\s*$", re.DOTALL)
# A pronoun's case form that carries its meaning (D3): « dative of nosotros: to us, for us »,
# « accusative of él and usted (…); him, you (formal), it, that » — the text after the first colon or
# semicolon.
_CASE_FORM = re.compile(
    r"^(?:nominative|accusative|dative|genitive|reflexive|prepositional|disjunctive)\b[^:;]*?\bof\b[^:;]*[:;]\s*(.+)",
    re.IGNORECASE | re.DOTALL,
)
# The parts of speech of a function word, which a place's name does not open (D4).
_FUNCTION_WORDS = frozenset({"prep", "conj", "pron", "det", "article"})

# The edition's typography (D5). A source's numbered sense: a bracket first, then a parenthesis that
# still names one, whole.
_SENSE_BRACKET = re.compile(r"\s*\[sense \d+\]")
_SENSE_PARENTHESIS = re.compile(r"\s*\([^()]*\bsense \d+\b[^()]*\)")
# The edition's descriptions, which open on a capital where 98.3 % of its senses open in lower case:
# a closed list of openers, followed by a space and a letter.
_OPENER = re.compile(
    r"^(A|An|The|Any|One|Some|Certain|Various|Either|Used|Said|Indicates?|Expresses|Denotes|Forms|Replaces|"
    r"Introduces|Refers|Related|Relating|Pertaining|Of|Having|In|To|Someone|Something|Term|Expression|"
    r"Interjection) (?=([^\W\d_]))"
)
# An ellipsis between two words, written `...` or `…`, with any spacing.
_ELLIPSIS_BETWEEN = re.compile(r"(?<=\w)\s*(?:\.\.\.|…)\s*(?=\w)")


def english_typography(gloss):
    """A gloss in one English typography (D5), in this order: cut at its first line break (an
    example sentence follows it); a bracketed « [sense N] » removed, then a parenthesis still naming
    a numbered sense, whole; an opener of the edition's descriptions (`_OPENER`) in lower case —
    never « The » before a capital, a title's, a name's or a species' (« The Nutcracker »), nor a
    word followed by no letter (« A (highest grade in testing) »); `...` written `…`, spaced on both
    sides between two words and as written elsewhere; an even number of straight double quotes
    paired “ ”, an odd one kept. Nothing else of the gloss changes."""
    text = gloss.split("\n", 1)[0]
    text = _SENSE_BRACKET.sub("", text)
    text = _SENSE_PARENTHESIS.sub("", text)
    opener = _OPENER.match(text)
    if opener and not (opener.group(1) == "The" and opener.group(2).isupper()):
        text = opener.group(1).lower() + text[opener.end(1) :]
    text = _ELLIPSIS_BETWEEN.sub(" … ", text).replace("...", "…")
    if text.count('"') % 2 == 0:
        quotes = iter("“”" * (text.count('"') // 2))
        text = "".join(next(quotes) if ch == '"' else ch for ch in text)
    return text


def _texts(glosses):
    """Whether a sense's `glosses` is what kaikki writes: a non-empty list of strings."""
    return isinstance(glosses, list) and bool(glosses) and all(isinstance(g, str) for g in glosses)


def _headword(entry):
    word = entry.get("word")
    return word.strip() if isinstance(word, str) else ""


def _own_glosses(sense, pointers):
    """A nested sense read by its own gloss when its parent is no meaning (D2): a label — it ends on
    a colon or names senses — or a pointer — a pointer sense of the same entry (`pointers`) or the
    edition's pointer wording (`_FORM_OF`: a shortened form, a diminutive, an alternative form). A
    parent that is a meaning (« to make » over « to create ») keeps glossing its nested senses."""
    glosses = sense.get("glosses")
    if not _texts(glosses) or len(glosses) < 2:
        return sense
    parent = glosses[0].strip()
    if parent.endswith(":") or _SENSE_GROUP.search(parent) or parent in pointers or _FORM_OF.match(parent):
        return {**sense, "glosses": [glosses[-1]]}
    return sense


def _as_meaning(sense, gloss):
    """`sense` read as `gloss`: its other fields and tags kept, its pointer's tags and fields dropped."""
    meaning = {"glosses": [gloss]}
    for field, value in sense.items():
        if field == "glosses" or field in EN.pointer_fields:
            continue
        if field == "tags" and isinstance(value, list):
            value = [tag for tag in value if tag not in EN.pointer_tags]
        meaning[field] = value
    return meaning


def _carried(after):
    """The meaning a shortened form writes after its target (« , my », « (“mom”) »), or None."""
    found = _AFTER_TARGET.match(after) or _QUOTED.match(after)
    return found.group(1).strip() or None if found else None


def _shortened(sense, rest):
    """A shortened form's target and the meaning it carries, from the text after its wording
    (`rest`): `(target, meaning)`, either None. The target is the first word a pointer field names,
    whole (« malo bad », « cincuenta y uno »), else the gloss's text up to a comma, a semicolon, a
    colon, an opening parenthesis or the end; the meaning is kaikki's `extra` for that target, else
    the gloss's text after the target (`_carried`)."""
    rest = rest.strip()
    refs = [ref for ref in common._pointers(sense, EN) if isinstance(ref, dict) and isinstance(ref.get("word"), str)]
    refs = [ref for ref in refs if ref["word"].strip()]
    if refs:
        target, extra = refs[0]["word"].strip(), refs[0].get("extra")
        if isinstance(extra, str) and extra.strip():
            quoted = _QUOTED.match(extra)
            return target, (quoted.group(1) if quoted else extra).strip() or None
        return target, _carried(rest[len(target) :]) if rest.startswith(target) else None
    found = _TARGET.match(rest)
    if not found:
        return None, None
    return found.group(0).strip().rstrip(".").strip() or None, _carried(rest[found.end() :])


def _read_senses(entry, lend):
    """The entry's senses as meanings: D2, then D3, then D5 on every gloss. `lend(target, pos,
    word)` answers a target's meaning senses in that part of speech, or None. A sense that is not
    kaikki's shape is kept as it is."""
    senses = entry.get("senses")
    if not isinstance(senses, list):
        return senses
    pos, word = entry.get("pos"), _headword(entry).lower()
    pointers = {
        s["glosses"][0].strip()
        for s in senses
        if isinstance(s, dict) and _texts(s.get("glosses")) and len(s["glosses"]) == 1
        and common._is_form_of(s, s["glosses"][0].strip(), edition=EN)
    }
    read = []
    for sense in senses:
        if not isinstance(sense, dict) or not _texts(sense.get("glosses")):
            read.append(sense)
            continue
        sense = _own_glosses(sense, pointers)
        gloss = sense["glosses"][0].strip()
        shortened = _SHORTENED_FORM.match(gloss)
        if shortened:
            target, meaning = _shortened(sense, gloss[shortened.end() :])
            lent = None if meaning or not target else lend(target, pos, word)
            if meaning:
                sense = _as_meaning(sense, meaning)
            elif lent:
                read.extend(_as_meaning(sense, english_typography(g)) for g in lent)
                continue
        elif pos == "pron" and common._is_form_of(sense, gloss, edition=EN):
            case = _CASE_FORM.match(gloss)
            if case and case.group(1).strip():
                sense = _as_meaning(sense, case.group(1).strip())
        read.append({**sense, "glosses": [english_typography(g) for g in sense["glosses"]]})
    return read


def _entries(path):
    """`(entry, line)` for each line of `path`; `entry` is None for a line that is no JSON object."""
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            yield (entry if isinstance(entry, dict) else None), line


def _meanings_of(path, keys):
    """For each `(word, part of speech)` of `keys`: the meaning senses of that word's entries in that
    part of speech, as the shared rules read them after D2 (its first gloss, a pointer skipped), and
    the words its pointer senses name — `(meanings, pointers)`, in file order. An acronym's entries
    lend nothing (`reduce_common._acronym`)."""
    meanings, pointers = {}, {}
    if not keys:
        return meanings, pointers
    for entry, _ in _entries(path):
        if entry is None:
            continue
        headword = _headword(entry)
        key = (headword.lower(), entry.get("pos"))
        if key not in keys or common._acronym(headword) or not isinstance(entry.get("senses"), list):
            continue
        senses = entry["senses"]
        own = {
            s["glosses"][0].strip()
            for s in senses
            if isinstance(s, dict) and _texts(s.get("glosses")) and len(s["glosses"]) == 1
            and common._is_form_of(s, s["glosses"][0].strip(), edition=EN)
        }
        for sense in senses:
            if not isinstance(sense, dict) or not _texts(sense.get("glosses")):
                continue
            sense = _own_glosses(sense, own)
            gloss = sense["glosses"][0]
            if not gloss.strip():
                continue
            if common._is_form_of(sense, gloss.strip(), edition=EN):
                for ref in common._pointers(sense, EN):
                    base = ref.get("word") if isinstance(ref, dict) else None
                    if isinstance(base, str) and base.strip():
                        pointers.setdefault(key, []).append(base.strip().lower())
            else:
                meanings.setdefault(key, []).append(gloss)
    return meanings, pointers


def read_as_meanings(src, dst):
    """The edition's entries with their senses read as meanings, written to `dst` — a pre-pass a
    reducer runs after `without_letter_headwords` and before `merge_same_pos_etymologies`
    (refine-lingua-es-en-glosses D1). It reads `src` at most four times: the function words and the
    targets to look up, the targets' senses, their pointers' targets' senses, then the rewrite.

    - D2, nested senses: a sense nested under a label or a pointer (`_own_glosses`) is read by its
      own gloss, `glosses[-1]`: « venir »'s senses under « Figurative senses. », « casita »'s
      « small house » under « diminutive of casa », « su »'s under « apocopic form of suyo ».
    - D3, shortened and respelled forms: a sense whose gloss opens on « apocopic form of »,
      « apheretic form of », « syncopic form of », « prepositional form of », « pronunciation spelling
      of » or « eye dialect spelling of » is replaced, in its place, by the meaning it carries
      (`_shortened`: « mi » « my », « muy » « very », « cincuenta y un » « fifty-one »), else by its
      target's meaning senses in the same part of speech, from a target of three letters or more
      (`reduce_common._MIN_BASE`), following the target's own pointer once when the target is only a
      form (« toy » → « estoy » → « estar »), else kept, a pointer (`_FORM_OF`). A pointer sense of a
      pronoun whose gloss names a case of a word and carries its meaning after a colon or a semicolon
      is read as that meaning (« lo » « him, you (formal), it, that »). The new sense keeps the
      original's other tags and loses its pointer's tags and fields.
    - D4, a function word spelled like a place: the `name` entries of a headword with an initial
      capital — not all capitals, an acronym's lines keep their place — are written after every other
      line when the lower-case headword has an entry whose part of speech is a preposition, a
      conjunction, a pronoun, a determiner or an article (`_FUNCTION_WORDS`): « como » opens on « as »,
      not on « Como ». A headword's moved lines stay consecutive and in their order.
    - D5, one English typography (`english_typography`) on every gloss, lent or carried ones too.

    A line this pass cannot read is written as it is, and so is an entry it does not change.
    """
    function_words, wanted = set(), set()

    def wants(target, pos, word):
        target = target.lower()
        if len(target) >= common._MIN_BASE and target != word:
            wanted.add((target, pos))
        return None

    for entry, _ in _entries(src):
        if entry is None:
            continue
        headword = _headword(entry)
        if headword and headword == headword.lower() and entry.get("pos") in _FUNCTION_WORDS:
            function_words.add(headword)
        _read_senses(entry, wants)
    meanings, pointers = _meanings_of(src, wanted)
    bases = {
        (base, pos)
        for target, pos in wanted
        if not meanings.get((target, pos))
        for base in pointers.get((target, pos), ())
        if len(base) >= common._MIN_BASE
    }
    meanings.update(_meanings_of(src, bases - meanings.keys())[0])

    def lend(target, pos, word):
        target = target.lower()
        if len(target) < common._MIN_BASE or target == word:
            return None
        if meanings.get((target, pos)):
            return meanings[(target, pos)]
        for base in pointers.get((target, pos), ()):
            if len(base) >= common._MIN_BASE and base != word and meanings.get((base, pos)):
                return meanings[(base, pos)]
        return None

    moved = {}  # headword → its lines, written after every other line (D4)
    with open(dst, "w", encoding="utf-8") as out:
        for entry, line in _entries(src):
            if entry is not None:
                senses = _read_senses(entry, lend)
                if senses != entry.get("senses"):
                    line = json.dumps({**entry, "senses": senses}, ensure_ascii=False) + "\n"
                headword = _headword(entry)
                if (
                    entry.get("pos") == "name"
                    and headword[:1].isupper()
                    and not common._acronym(headword)
                    and headword.lower() in function_words
                ):
                    moved.setdefault(headword, []).append(line if line.endswith("\n") else line + "\n")
                    continue
            out.write(line if line.endswith("\n") else line + "\n")
        for lines in moved.values():
            out.writelines(lines)
    return dst
