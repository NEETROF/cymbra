# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The Italian Wiktionary's rules (itwiktionary), for the pairs glossed in Italian
(add-lingua-italian-edition-rules).

en-it, fr-it and es-it read it (changes 36, 47 and 51): their glosses are the Italian Wiktionary's
senses of its English, French and Spanish sections. Its rules come from a census of those sections,
derived by `pack_sources.derive` from kaikki's dump of the whole edition of 2026-10-21 (43,849,715 B):
41,754 senses in the English section, 35,026 in the French, 8,660 in the Spanish.

- The edition writes in lower case (93.5–95.3 % of its senses), and its labels — obsolete, vulgar,
  heraldry — in kaikki's tags, not in the text: a gloss made of translation-table words keeps its
  words' case (`capitalised=False`, D9).
- Pointers (D2). Kaikki tags `form-of` every sense of a « Voce verbale » or « …, forma flessa » entry
  and names in `form_of` the gloss's last link, often an Italian word (« plurale di eye, occhi » →
  `occhi`). Untagged, a sense still only points when it is one of the edition's wordings — an
  inflection (« plurale di », « femminile di », a person, « participio … di », « passato … di »), a
  variant (« variante statunitense di »), « comparativo di », « superlativo di », « forma corta di »,
  the English grammar wordings (« simple past tense e past participle di »), the Spanish section's
  English bot pointers (« The first-person singular of congelar in the future subjunctive. »), or
  « vedi » and a word — followed by « di » or an articled form of it and a target of at most three
  words that ends the sense. With text after the target, it is a meaning (« plurale di that: quelli,
  quelle »). « contrazione di » never points (`isn't` « contrazione di is not »), and « abbreviazione
  di » only when the word it names lends a gloss.
- Notes to its own readers (D6): the placeholder « definizione mancante; se vuoi, aggiungila tu »,
  « vedi italiano » after a meaning, « (inv) », and template residue (« ( approfondimento) », « () »,
  « [[]] », stray brackets).
- A letter's name (D7): « nome della lettera X », « La lettera Zeta ».

`read_as_meanings`, a pre-pass a pair runs after `reduce_common.without_letter_senses` and before
the shared rules read the file, reads the senses as meanings: template residue out before a sense is
tested as a pointer (D6); a pointer lends the word its wording names, not kaikki's last link (D3); a
sense tagged `form-of` that is not worded as an inflection is a meaning when its word has no other
(D4); a gloss nested under a pointer is read by its own text (D5); « vedi italiano » alone, or followed
by a qualifier, is the headword itself (D6); obsolete, archaic and heraldic senses after the others in
their entry, and a headword — a word or an expression — whose every sense with a gloss is heraldic
left out, so that the translation tables gloss it (D8); every gloss in the edition's typography
(`typography`, D9).

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does — tuning it re-pins
only the pairs whose definitions come from the Italian Wiktionary (D1).
"""

import json
import re

import reduce_common as common

# — Pointers (D2) —

# A word of a pointer's target: letters or digits, with inner apostrophes, hyphens and periods
# (« e-mail », « o'clock », « 0-day »).
_WORD = r"[^\W_](?:[\w'’.-]*\w)?"
# The words an inflection's or a variant's wording is made of, after its first: « terza persona
# singolare del presente semplice », « participio passato e passato semplice », « variante non
# corretta ». An articled « di » inside the wording (« del », « dell' ») is one of them.
_WORDING_WORDS = (
    "plurale|singolare|sing\\.?|femminile|maschile|irregolare|regolare|alternativa|corta|ortografica|"
    "statunitense|americana|britannica|grafica|informale|tradizionale|arcaica|non|corretta|comparativo|"
    "superlativo|relativo|assoluto|participio|passato|presente|prossimo|remoto|semplice|gerundio|prima|"
    "seconda|terza|[123](?:ª|ᵃ|a)|pers\\.?|persona|indicativo|congiuntivo|condizionale|imperativo|futuro|"
    "imperfetto|storico|affermativo|negativo|forma|meno|comune|alternativo|obsoleta|obsoleto|antiquata|"
    "eufonica|simple|past|present|tense|participle|e|and|al|del|dello|della"
)
# A parenthesis inside the wording is one of its words: « seconda persona singolare (pronome vos)
# dell'imperativo di abjurar », « variante ortografica (utilizzata soprattutto in Gran Bretagna) di ».
_WORDING_WORD = rf"(?:dell'|\([^()]*\)|(?:{_WORDING_WORDS})(?!\w))"
# A wording's first words: an inflection, a person, a variant, a degree, a short form, English grammar.
_HEAD = (
    r"(?:plurale|singolare|femminile|maschile|forma\s+(?:corta|alternativa|femminile|maschile|plurale)|"
    r"variante|comparativo|superlativo|participio|passato(?!\s+di\s+moda\b)|gerundio|condizionale|"
    r"congiuntivo|indicativo|imperativo|(?:prima|seconda|terza|[123](?:ª|ᵃ|a))[\s-]+pers(?:ona|\.)?|"
    r"simple|past|present)"
    r"(?!\w)"
)
# « di » or one of its articled forms, then the target; a verb's « verbo » or « to » before it.
_OF = r"(?:di|del|dello|della|dei|degli|delle|dell'|d')"
_BEFORE_TARGET = r"\s*(?:(?:verbo|to)\s+)?"
# The target is never a word of the wording (« terza persona singolare del presente di … »).
_NOT_WORDING = rf"(?!{_WORDING_WORD}\s)"
_TARGET = rf"[\"«]?(?P<target>{_WORD}(?:[\s-]+{_WORD}){{0,2}})[\"»]?"
# A label in a parenthesis may open the sense: « (meno comune) participio passato di hang ».
_LABEL = r"(?:\([^()]*\)\s*)?"
_INFLECTION = rf"{_LABEL}{_HEAD}(?:[\s,]*{_WORDING_WORD})*\s*{_OF}{_BEFORE_TARGET}{_NOT_WORDING}"
# The Spanish section's English bot pointers, read whole: « The first-person singular of congelar in
# the future subjunctive. », « third-person singular indicative present of visitar ».
_BOT = rf"(?:the\s+)?(?:first|second|third)[\s-]person\s+(?:singular|plural)\b[^.;:]*?\bof\s+(?P<bot>{_WORD})"
_BOT_REST = r"(?:\s+in\s+the\s+[\w\s]+)?\s*\.?"
# « vedi » and a word, alone: « vedi italiano » is a note (D6), never a pointer.
_SEE = rf"vedi\s+(?!italiano\b)(?P<see>{_WORD}(?:\s+{_WORD}){{0,2}})"
# « abbreviazione di », a pointer only when the word it names lends a gloss.
_ABBREVIATION = re.compile(rf"^{_LABEL}abbreviazione\s+{_OF}{_BEFORE_TARGET}{_TARGET}", re.IGNORECASE)

# An untagged sense that only points: one of the wordings and a target of at most three words that
# ends the sense. Anchored at both ends: with text after its target, a sense is a meaning.
_FORM_OF = re.compile(
    rf"^(?:{_INFLECTION}{_TARGET}|{_BOT}{_BOT_REST}|{_SEE})\s*\.?$",
    re.IGNORECASE,
)
# A sense worded as a pointer, whatever follows its target: what a tagged sense must open on to stay
# one (D4), and a nested gloss's parent (D5).
_WORDED = re.compile(rf"^(?:{_INFLECTION}{_TARGET}|{_BOT})", re.IGNORECASE)

# — Notes (D6) —

# The template residue the dump leaves in a sense: « ( approfondimento) », « () », « [[ » and « ]] »,
# the link « , Approfondimento » after a meaning.
# Case-sensitive: « approfondimento » is also a word (`insight` « introspezione, approfondimento »).
_RESIDUE = re.compile(r"\s*\(\s*[Aa]pprofondimento\s*\)|\s*\(\s*\)|\[\[|\]\]|,\s*Approfondimento\s*$")
_PLACEHOLDER = r"definizione mancante;\s*se vuoi,\s*aggiungila tu\.?"
_SEE_ITALIAN = r"vedi italiano"
# What the edition writes for its own readers, taken out wherever it sits: its placeholder, « vedi
# italiano » after a meaning (a qualifier after it stays), « (inv) », and its template residue.
_NOTES = re.compile(
    rf"\s*{_PLACEHOLDER}"
    rf"|\s*,?\s*\b{_SEE_ITALIAN}\b"
    r"|\s*\(inv\)"
    rf"|(?-i:{_RESIDUE.pattern})",
    re.IGNORECASE,
)
# « vedi italiano » alone, or followed by a qualifier: the Italian word is the headword itself.
_SAME_WORD = re.compile(rf"^{_SEE_ITALIAN}\s*(?P<qualifier>\([^()]*\))?\s*\.?$", re.IGNORECASE)

# A sense naming a letter (D7): « nome della lettera X », « La lettera Zeta ».
_LETTER = re.compile(r"\bnome della lettera\b|^la lettera\b", re.IGNORECASE)

IT = common.Edition(
    code="it",
    form_of=_FORM_OF,
    notes=_NOTES,
    letter=_LETTER,
    capitalised=False,
)


# — Typography (D9) —

# A gender or number mark after a word, before a separator or the end: « merda f », « evento m., ».
_MARK = re.compile(
    r"(?<=\w)(?<!\bdi)(?<!\bof)(?<!\blettera)\s+(?:m|f|n|mf|pl|mpl|fpl|[mf]\s+pl)\.?(?=\s*[,;)]|\s*$)"
)
_SPACE_BEFORE = re.compile(r"\s+(?=[,)])")
# An empty item a space before a comma leaves: « esperto, , provetto ».
_EMPTY_ITEM = re.compile(r",(?:\s*,)+")
_SPACE_AFTER = re.compile(r"(?<=\()\s+")
_ELLIPSIS = re.compile(r"\.{3,}")


def typography(text):
    """`text` in the edition's typography (D9): gender and number marks after a word taken out
    (« merda f » → « merda », « avvenimento m., evento m., fatto m » → « avvenimento, evento,
    fatto »), never after « di », « of » or « lettera », so that the word a pointer names keeps its
    letters; no space before a comma or a closing parenthesis, nor after an opening one, and no empty
    item between two commas; three dots
    or more written « … »; an even number of straight double quotes paired « », the quotes the
    Italian interface writes, an odd number kept. Nothing else of the text changes."""
    text = _MARK.sub("", text)
    text = _EMPTY_ITEM.sub(",", _SPACE_AFTER.sub("", _SPACE_BEFORE.sub("", text)))
    text = _ELLIPSIS.sub("…", text)
    quotes = text.count('"')
    if quotes and quotes % 2 == 0:
        parts = text.split('"')
        text = parts[0] + "".join(f"«{part.strip()}»" if i % 2 == 0 else part for i, part in enumerate(parts[1:]))
    return text


def without_residue(text):
    """`text` without the dump's template residue (D6): « ( approfondimento) », « () », « [[ », « ]] »,
    « , Approfondimento » at its end, and the square brackets of a text whose brackets do not pair
    (`cream` « [crema]], fior fiore » → « crema, fior fiore »)."""
    text = _RESIDUE.sub("", text)
    if text.count("[") != text.count("]"):
        text = text.replace("[", "").replace("]", "")
    return text.strip()


# — Meanings in their order (D3–D8) —

# The labels of a sense the edition marks no longer used (D8): kaikki's tags, and its raw labels.
_NO_LONGER_USED = frozenset({"obsolete", "archaic", "arcaico", "datato"})
_HERALDRY = "heraldry"


def _texts(glosses):
    """Whether a sense's `glosses` is what kaikki writes: a non-empty list of strings."""
    return isinstance(glosses, list) and bool(glosses) and all(isinstance(g, str) for g in glosses)


def _strings(sense, field):
    value = sense.get(field)
    return {x for x in value if isinstance(x, str)} if isinstance(value, list) else set()


def _headword(entry):
    word = entry.get("word")
    return word.strip() if isinstance(word, str) else ""


def _tagged(sense):
    """Whether kaikki marks a sense as a pointer: tagged `form-of`, or naming its word in `form_of`."""
    return not IT.pointer_tags.isdisjoint(_strings(sense, "tags")) or any(sense.get(f) for f in IT.pointer_fields)


def _target(found):
    """The word a pointer's wording names, from its match: « verbo » and « to » already left out."""
    target = next(found.group(name) for name in ("target", "bot", "see") if found.groupdict().get(name))
    return re.sub(r"\s+", " ", target).strip().rstrip(".")


def _as_meaning(sense, glosses):
    """`sense` read as a meaning with `glosses`: its pointer's tag and field dropped, the rest kept."""
    meaning = {field: value for field, value in sense.items() if field not in IT.pointer_fields}
    meaning["glosses"] = glosses
    if isinstance(sense.get("tags"), list):
        meaning["tags"] = [tag for tag in sense["tags"] if tag not in IT.pointer_tags]
    return meaning


def _as_pointer(sense, target):
    """`sense` read as a pointer at `target` (D3): tagged `form-of`, naming `target` in `form_of`."""
    tags = sense.get("tags") if isinstance(sense.get("tags"), list) else []
    pointer = {**sense, "form_of": [{"word": target}]}
    pointer["tags"] = tags if "form-of" in tags else [*tags, "form-of"]
    return pointer


def _read(sense, headword):
    """A sense read by the edition's rules, before what depends on the other senses: `(kind, sense,
    target)`. `kind` is "meaning", "pointer" (it names `target`), "abbreviation" (a pointer at
    `target` when that word lends a gloss) or "tagged" (tagged `form-of` and not worded as a pointer:
    a meaning when its word has no other, D4). Its glosses lose their template residue (D6)."""
    glosses = [without_residue(g) for g in sense["glosses"]]
    sense = {**sense, "glosses": glosses}
    first = glosses[0]
    # D5: a gloss nested under a pointer is read by its own text.
    if len(glosses) > 1 and _WORDED.match(first):
        sense = _as_meaning(sense, [glosses[-1]])
        first = glosses[-1]
    # D6: « vedi italiano » alone, or with a qualifier, is the headword itself.
    same = _SAME_WORD.match(first)
    if same:
        qualifier = same.group("qualifier")
        return "meaning", _as_meaning(sense, [f"{headword} {qualifier}" if qualifier else headword]), None
    abbreviation = _ABBREVIATION.match(first)
    if abbreviation and (_tagged(sense) or re.fullmatch(_ABBREVIATION.pattern + r"\s*\.?$", first, re.I)):
        return "abbreviation", sense, _target(abbreviation)
    if _tagged(sense):
        worded = _WORDED.match(first)
        if worded:
            return "pointer", _as_pointer(sense, _target(worded)), _target(worded)
        return "tagged", sense, None
    pointer = _FORM_OF.match(first)
    if pointer:
        return "pointer", _as_pointer(sense, _target(pointer)), _target(pointer)
    return "meaning", sense, None


def _glossed(sense):
    """Whether a sense says something once the edition's notes are out of it."""
    return bool(common.clean_gloss(sense["glosses"][0], 300, edition=IT))


def _rank(sense):
    """A sense's place in its entry (D8): a current sense, then one no longer used, then a heraldic one."""
    if not isinstance(sense, dict):
        return 0
    if _HERALDRY in _strings(sense, "topics"):
        return 2
    labels = _strings(sense, "tags") | _strings(sense, "raw_tags")
    return 1 if not _NO_LONGER_USED.isdisjoint(labels) else 0


def _entries(path):
    """`(entry, line)` for each line of `path`; `entry` is None for a line that is no JSON object or
    has no list of senses."""
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            ok = isinstance(entry, dict) and isinstance(entry.get("senses"), list)
            yield (entry if ok else None), line


def _readings(entry):
    """Each sense of an entry as `_read` reads it, or `(None, sense, None)` when it is not kaikki's shape."""
    headword = _headword(entry)
    return [
        _read(sense, headword) if isinstance(sense, dict) and _texts(sense.get("glosses")) else (None, sense, None)
        for sense in entry["senses"]
    ]


def read_as_meanings(src, dst):
    """The edition's entries with their senses read as meanings, in their order, written to `dst` —
    a pre-pass a reducer runs after `reduce_common.without_letter_senses` and before the shared rules
    read the file. It reads `src` twice: what the words mean, then the rewrite.

    - D2, D3: an untagged sense worded as a pointer (`_FORM_OF`) is tagged `form-of`, and a sense
      tagged so and worded as one names in `form_of` the word its wording names, not kaikki's last
      link: the shared rules lend that word's gloss. « abbreviazione di » points only when the word
      it names has a meaning in the same part of speech, three letters or more and not the headword;
      otherwise it is a meaning, its tag dropped.
    - D4: a sense tagged `form-of` and not worded as a pointer is a meaning when no other sense of the
      headword's — in any entry — is one; otherwise it stays a pointer.
    - D5: a sense whose first gloss is a pointer's wording and that nests more is its last gloss.
    - D6: template residue out of every gloss before the pointer test; « vedi italiano » alone, or
      followed by a qualifier, read as the headword.
    - D8: senses tagged obsolete or archaic, or raw-tagged « arcaico » or « datato », then heraldic
      ones, after the others of their entry, each group in its order; a headword — a word or an
      expression, in lower case — whose every sense with a gloss is heraldic is left out.
    - D9: every gloss in the edition's typography (`typography`).

    A line this pass cannot read is written as it is, and so is an entry it does not change."""
    meanings, keyed, heraldic, other = set(), set(), set(), set()
    for entry, _ in _entries(src):
        if entry is None:
            continue
        word = _headword(entry).lower()
        for (kind, sense, _), original in zip(_readings(entry), entry["senses"]):
            if kind is None or not _glossed(sense):
                continue
            if kind == "meaning":
                meanings.add(word)
                keyed.add((word, entry.get("pos")))
            (heraldic if _HERALDRY in _strings(original, "topics") else other).add(word)
    heraldry_only = heraldic - other
    meanings -= heraldry_only

    def lends(target, pos, word):
        target = target.lower()
        return (
            len(target) >= common._MIN_BASE
            and target != word
            and target not in heraldry_only
            and (target, pos) in keyed
        )

    with open(dst, "w", encoding="utf-8") as out:
        for entry, line in _entries(src):
            if entry is not None:
                word = _headword(entry).lower()
                if word in heraldry_only:
                    continue
                senses = []
                for kind, sense, target in _readings(entry):
                    if kind == "abbreviation" and lends(target, entry.get("pos"), word):
                        sense = _as_pointer(sense, target)
                    elif kind == "abbreviation" or (kind == "tagged" and word not in meanings):
                        sense = _as_meaning(sense, sense["glosses"])
                    if kind is not None:
                        sense = {**sense, "glosses": [typography(g) for g in sense["glosses"]]}
                    senses.append(sense)
                senses = sorted(senses, key=_rank)
                if senses != entry["senses"]:
                    line = json.dumps({**entry, "senses": senses}, ensure_ascii=False) + "\n"
            out.write(line if line.endswith("\n") else line + "\n")
    return dst
