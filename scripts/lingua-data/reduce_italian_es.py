# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""it-es's own reading of its sources (add-lingua-pack-it-es D4).

A rule module of its own: imported by `reduce-it-es.py` and by its tests, never by another pair. The
Spanish edition's rules (`reduce_edition_es.py`) are in en-es's and fr-es's pins, so what Italian needs
from the Spanish edition goes here, beside them and never in them; and it-es imports no Italian edition
module (`add-lingua-italian-edition-rules`), whose later edits would re-pin it. This file is part of
it-es's rule digest (`pack_sources.rule_files`): editing it re-pins it-es and no other pair.

- **Names and acronyms** (it-fr's amended D5, and the thin-pair clause): an acronym's sense, and a
  proper noun's sense that only says the word is a surname or a given name, keep off a word the section
  gives a lowercase entry of its own, even one of pointers only; a name's note also keeps off a word
  the Italian or the Spanish Wiktionary's translation table glosses (`secondo` « Segundo; Según », not
  « Apellido »).
- **Pointers**: a pointer the edition words but does not tag (« Variante de avere. ») lends the word it
  names only to a headword with no meaning and no tagged pointer of its own (`aver` from `avere`; never
  `sono`, whose verb senses are tagged forms of `essere`, from `suono`); the meaning a pointer carries
  after a colon (« Forma del femenino de sto: «esta». ») is read only for a headword with no meaning of
  its own (`sta` « Esta »; never `sette` « Sectas »).
- **Notes**: sense links « _(1, 5) », superscripts, and the subscripts the Spanish edition's rule leaves
  after a quotation mark or a superscript, taken out; a gloss opening in lower case opens on a capital.
- **Function words**: the commonest a reviewed list glosses (`FUNCTION_WORDS`), where the section
  describes them (`tu` « Tú », `è` « Es », `li` « Los »).
- **The Italian word given as Spanish**: refused unless `ITALIAN_ALIKE` names it a Spanish word too —
  the rule `reduce_spanish_tables.is_the_studied_word` reads.
- **The Italian Wiktionary's notes** in its Spanish section, read backwards: « vedi italiano » and
  « definizione mancante; se vuoi, aggiungila tu » taken out, a sense left empty dropped.
- **Marked words**: no Spanish word of a table with a grave accent or « ° ».
- **Split words**: the reader of the words without a space that Italian's pre-pass splits — a copy of
  it-en's D4.5 reader (`reduce_italian_en.py`), so that neither pair's digest follows the other's.
"""

import collections
import json
import re
import unicodedata

import reduce_common as common
import reduce_edition_es as spanish
import reduce_spanish_tables as tables

# Italian letters, lowercased, and an Italian word (change 9's `_TOKEN`, it-fr's): letters, words
# joined by hyphens or by an inner apostrophe, and an elided piece's or a truncation's final
# apostrophe (`l'`, `po'`). Repeated here because it-fr's reducer cannot be imported;
# test_reduce_it_es.py holds them to it-fr's.
LETTERS = "a-zàèéìíîòóùúâêôäëïöüçñ"
TOKEN = re.compile(rf"[{LETTERS}]+(?:['-][{LETTERS}]+)*'?")

EDITION = spanish.ES


def straight(text):
    """An Italian word as Italian's tables write it: in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip()).replace("’", "'")


def gloss_text(sense):
    glosses = sense.get("glosses") if isinstance(sense, dict) else None
    return glosses[0].strip() if isinstance(glosses, list) and glosses and isinstance(glosses[0], str) else ""


def _with_text(sense, text):
    return {**sense, "glosses": [text, *sense["glosses"][1:]]}


def entries(path):
    """`(entry, line)` for each line of `path`; `entry` is None for a line that is no JSON object."""
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            yield (entry if isinstance(entry, dict) else None), line


def _tagged(sense):
    """A sense the edition tags as a pointer: `form-of`, or naming its target (`form_of`, `alt_of`)."""
    return isinstance(sense, dict) and (
        "form-of" in (sense.get("tags") or ()) or bool(sense.get("form_of")) or bool(sense.get("alt_of"))
    )


def _is_pointer(sense):
    """A sense that only points at another word: tagged, or worded as the Spanish edition words one."""
    text = gloss_text(sense)
    return bool(text) and common._is_form_of(sense, text, edition=EDITION)


def _holds_a_meaning(sense):
    """A sense that is no pointer and keeps a gloss once the edition's rules clean it."""
    text = gloss_text(sense)
    return (
        bool(text)
        and not _is_pointer(sense)
        and bool(common.clean_gloss(text, common.WORD_GLOSS["per_sense"], edition=EDITION))
    )


# — The function words, from a reviewed list —
#
# The section describes the commonest function words rather than glossing them (`tu` « Pronombre
# personal nominativo de 2 persona del singular… », `essere` « Verbo auxiliar … »), and `è` and `sono`
# borrow `essere`'s description through their pointers. Each row is fr-es's `CORRECTIONS` shape: keyed
# by the section's text as the pinned file writes it — an entry's senses, in their order —, so that it
# fires on nothing once the page is corrected; the update that takes the correction in says so on its
# summary line, and the row is removed. The corrected senses are meanings: they replace the entry's
# senses whole, pointers included. Their words come from the sources where one has them — the Italian
# Wiktionary's Spanish translations (« Tú », « Ella », « Usted », « Soy », « Son »), the Spanish
# Wiktionary's Spanish tables (« El », « Los »), the section's own text (« Las », « De él, de ella, de
# ellos, de ellas »); a person wrote the rest (« Es », « Lo », « De allí, de ahí »). `corrected` None
# leaves out the entry, the word keeping its other entries' senses (`essere`'s and `avere`'s
# auxiliaries). The owner reads each; each is reported to the Spanish Wiktionary.
Correction = tables.Correction
_PAGE = "https://es.wiktionary.org/wiki/{}"
_DESCRIBED = "the section describes the word rather than gloss it"

FUNCTION_WORDS = (
    Correction(
        "section", "tu", "pron",
        ("Pronombre personal nominativo de 2 persona del singular, equivale al tú del español.",),
        ("Tú.",),
        f"{_DESCRIBED}; « tú » is the Italian Wiktionary's own Spanish for « tu »",
        _PAGE.format("tu"),
    ),
    Correction(
        "section", "è", "verb",
        ("Tercera persona del singular (lui, lei) del presente de indicativo de essere.",),
        ("Es.",),
        "a pointer lending essere's whole row, its auxiliaries' descriptions with it; the third person's « es »",
        _PAGE.format("è"),
    ),
    Correction(
        "section", "sono", "verb",
        (
            "Primera persona del singular (io) del presente de indicativo de essere.",
            "Tercera persona del plural (esse, essi) del presente de indicativo de essere.",
        ),
        ("Soy.", "Son."),
        "pointers lending essere's whole row; « soy » and « son » are the Italian Wiktionary's own Spanish for « sono »",
        _PAGE.format("sono"),
    ),
    Correction(
        "section", "essere", "verb",
        (
            "Verbo auxiliar de la mayoría de los verbos intransitivos, para la conjugación de tiempos compuestos.",
            "Verbo auxiliar de verbos transitivos en la conjugación de verbos de la forma pasiva.",
            "Verbo auxiliar en la conjugación de tiempos compuestos de verbos reflexivos.",
        ),
        None,
        f"{_DESCRIBED}: the auxiliary's uses, which its other entries gloss « Ser », « Estar »",
        _PAGE.format("essere"),
    ),
    Correction(
        "section", "avere", "verb",
        (
            "Verbo auxiliar de verbos transitivos en tiempos compuestos.",
            "Verbo auxiliar de algunos verbos intransitivos en tiempos compuestos.",
        ),
        None,
        f"{_DESCRIBED}: the auxiliary's uses, which its other entry glosses « Haber », « Tener »",
        _PAGE.format("avere"),
    ),
    Correction(
        "section", "lo", "article",
        (
            "Artículo definido masculino singular. Se emplea solamente cuando la palabra siguiente comienza con "
            "una i seguida de una vocal, z, s seguida de una consonante, gn, pn, ps, y o x.",
        ),
        ("El.",),
        f"{_DESCRIBED}; the Spanish Wiktionary's article « el » lists « lo »",
        _PAGE.format("lo"),
    ),
    Correction(
        "section", "lo", "pron",
        ("Pronombre personal acusativo de género masculino, tercera persona singular.",),
        ("Lo.",),
        f"{_DESCRIBED}: the accusative's « lo »",
        _PAGE.format("lo"),
    ),
    Correction(
        "section", "le", "article",
        ("Artículo determinado femenino del plural de la, equivale al español: las.",),
        ("Las.",),
        f"{_DESCRIBED}; « las » is the section's own Spanish",
        _PAGE.format("le"),
    ),
    Correction(
        "section", "le", "pron",
        (
            'Pronombre personal acusativo 3ª persona femenino plural, "a esas" (como complemento directo).',
            "Pronombre personal dativo de femenino de la tercera persona del singular: A ella, a esa, (como "
            "complemento indirecto).",
            "Grafía alternativa de Le.",
        ),
        ("Las.", "Le, a ella."),
        f"{_DESCRIBED}: the accusative's « las », the dative's « le, a ella », the section's own words",
        _PAGE.format("le"),
    ),
    Correction(
        "section", "gli", "article",
        ("Artículo definido plural masculino.",),
        ("Los.",),
        f"{_DESCRIBED}; the Spanish Wiktionary's article « los » lists « gli »",
        _PAGE.format("gli"),
    ),
    Correction(
        "section", "gli", "pron",
        (
            "Pronombre personal dativo de la tercera persona masculino del singular.",
            "Pronombre personal dativo de la tercera persona del plural, tanto para masculino como femenino "
            "(plural de le).",
        ),
        ("Le, a él.", "Les."),
        f"{_DESCRIBED}: the dative's « le », « les »",
        _PAGE.format("gli"),
    ),
    Correction(
        "section", "li", "pron",
        ("Pronombre personal acusativo de género masculino, tercera persona. Es plural de lo.",),
        ("Los.",),
        f"{_DESCRIBED}; the Spanish Wiktionary's pronoun « los » lists « li »",
        _PAGE.format("li"),
    ),
    Correction(
        "section", "ne", "adv",
        (
            "Indica proveniencia de un lugar ya nombrado precedentemente. No tiene una traducción al español.",
        ),
        ("De allí, de ahí.",),
        f"{_DESCRIBED}: where one comes from, « de allí »",
        _PAGE.format("ne"),
    ),
    Correction(
        "section", "ne", "pron",
        (
            "Pronombre personal de tercera persona, invariante en género y número. Se refiere a una o más "
            'personas, animales o cosas ya nombradas precedentemente. Significa "de él" "de ella" "de ellos" o '
            '·"de ellas".',
        ),
        ("De él, de ella, de ellos, de ellas.",),
        f"{_DESCRIBED}; its meaning is the section's own words",
        _PAGE.format("ne"),
    ),
    Correction(
        "section", "lei", "pron",
        (
            "Pronombre personal nominativo de la tercera persona femenino del singular, equivale al del español ella.",
            "Pronombre personal tónico de la tercera persona femenino del singular, equivale al del español ella.",
        ),
        ("Ella.",),
        f"{_DESCRIBED}; « ella » is the Italian Wiktionary's own Spanish for « lei »",
        _PAGE.format("lei"),
    ),
    Correction(
        "section", "Lei", "pron",
        (
            "Pronombre personal de la tercera persona del singular, invariante en género. Su plural es Loro. "
            "Equivale al pronombre en español Usted.",
        ),
        ("Usted.",),
        f"{_DESCRIBED}; « usted » is the Italian Wiktionary's own Spanish for « Lei »",
        _PAGE.format("Lei"),
    ),
    Correction(
        "section", "loro", "pron",
        (
            "Pronombre personal nominativo de la tercera persona del plural, invariante en género, equivale al "
            "pronombre en español ustedes, ellos, ellas. Se pueden usar las alternativas essi (en español esos) y "
            "esse (en español esas).",
            "Pronombre personal tónico de la tercera persona del plural, invariante en género. Equivale a los "
            "pronombres en español ustedes, ellos, ellas.",
        ),
        ("Ellos, ellas.",),
        f"{_DESCRIBED}; the Spanish Wiktionary's « ellos » and « ellas » list « loro »",
        _PAGE.format("loro"),
    ),
    Correction(
        "section", "Loro", "pron",
        (
            "Pronombre personal de la tercera persona del plural, invariante en género. Es el plural de Lei. "
            "Equivale al pronombre en español Ustedes.",
        ),
        ("Ustedes.",),
        f"{_DESCRIBED}; « ustedes » is the section's own Spanish",
        _PAGE.format("Loro"),
    ),
)  # fmt: skip


def as_function_word(entry, fired=None):
    """`entry` as `FUNCTION_WORDS` reads it: a row of its headword and part of speech whose texts are
    exactly the entry's senses' replaces them with its meanings — plain senses, no pointer —, or leaves
    the entry out (None) when it writes none. Each row that fires is added to `fired`."""
    headword = entry.get("word").strip() if isinstance(entry.get("word"), str) else None
    senses = entry.get("senses")
    if not isinstance(senses, list) or not all(isinstance(sense, dict) for sense in senses):
        return entry
    texts = tuple(gloss_text(sense) for sense in senses)
    for row in FUNCTION_WORDS:
        if row.headword != headword or row.pos != entry.get("pos") or row.text != texts:
            continue
        if fired is not None:
            fired.add(row)
        if row.corrected is None:
            return None
        return {**entry, "senses": [{"glosses": [text]} for text in row.corrected]}
    return entry


def function_words(src, dst, fired=None):
    """The section's entries as `FUNCTION_WORDS` reads them, written to `dst`."""
    return spanish.rewrite_entries(src, dst, lambda entry: as_function_word(entry, fired))


def unfired(fired):
    """The rows of `FUNCTION_WORDS` that fired on nothing, to be removed."""
    return [f"{row.headword} ({row.pos})" for row in FUNCTION_WORDS if row not in fired]


# — The Italian word given as Spanish —
#
# A table's Spanish word spelled as the Italian word, which wordfreq rates `tables.Rules.gap` Zipf
# points commoner in Italian than in Spanish, is left out (`reduce_spanish_tables.is_the_studied_word`),
# unless this list, reviewed by the owner, names it a Spanish word too — the RAE's dictionary (DLE) the
# reference. A candidate a later update brings is left out until it is listed, and that update's pull
# request names it.
ITALIAN_ALIKE = frozenset(
    {
        # In the DLE: « credo » (the creed, a set of beliefs), « friulano » (of Friuli, its language),
        # « nutrimento » (nourishment), « visivo » (that serves to see).
        "credo",
        "friulano",
        "nutrimento",
        "visivo",
    }
)


# — Names and acronyms (it-fr's amended D5, the thin-pair clause) —

# A proper noun's sense that only says the word is a surname or a given name (fr-es's wording, which
# the Italian section shares: « Apellido », « Apellido italiano. », « Nombre de pila de varón… »,
# « Hipocorístico de… »).
_NAME_NOTE = re.compile(r"^(?:Apellido|Nombre de pila|Nombre personal|Hipocorístico)\b")
# An acronym's sense its headword does not show in capitals alone: « Sigla de… », « Acrónimo de… ».
_ACRONYM_NOTE = re.compile(r"^(?:Sigla|Acrónimo|Abreviatura|Abreviación)\b", re.IGNORECASE)


def name_note(entry, sense):
    """Whether a sense only says its proper noun is a surname or a given name."""
    return entry.get("pos") == "name" and bool(_NAME_NOTE.match(gloss_text(sense)))


def acronym_sense(entry, sense):
    """Whether a sense is an acronym's: its headword in capitals, or the sense saying it is one."""
    return common._acronym(straight(entry.get("word"))) or bool(_ACRONYM_NOTE.match(gloss_text(sense)))


# — Pointers —

# The word a pointer the edition words names: the last « de » or « del » of its first sentence, before
# any colon, without a parenthesis (« Variante obsoleta de ello (antes de consonantes). » → `ello`).
_PARENTHESIS = re.compile(r"\s*\([^()]*\)")
_TARGET = re.compile(r".*\bdel?\s+(.+?)\s*$", re.DOTALL)
# The meaning a pointer carries after a colon outside quotation marks: « …de sto: «esta». » → « esta ».
_QUOTES = "«»\"“”'"


def pointer_target(text):
    """The Italian word a worded pointer names, as the section keys it (lowercased); None when its
    words are no Italian word."""
    first = text.split(":", 1)[0]
    first = _PARENTHESIS.sub("", first).split(". ", 1)[0].strip().rstrip(".").strip()
    found = _TARGET.match(first)
    if not found:
        return None
    target = straight(found.group(1)).lower()
    return target if target and all(TOKEN.fullmatch(part) for part in target.split(" ")) else None


def carried_meaning(text):
    """The meaning a pointer writes after its first colon, without quotation marks or the closing
    period; None when there is none."""
    _, colon, after = text.partition(":")
    if not colon:
        return None
    meaning = after.strip().rstrip(".").strip().strip(_QUOTES).strip().rstrip(".").strip()
    return meaning or None


# — Notes —

# A sense link « _(1, 5) », « _(1,3) »; superscripts; a subscript the Spanish edition's rule leaves —
# after a closing quotation mark, a bracket or a superscript (« «Coño»₆ », « Vero²₁ »).
_SENSE_LINK = re.compile(r"_\(\s*\d+(?:\s*,\s*\d+)*\s*\)")
_SUPERSCRIPT = "¹²³⁰⁴⁵⁶⁷⁸⁹"
_SUPERSCRIPTS = re.compile(rf"[{_SUPERSCRIPT}]+")
_SUBSCRIPT_AFTER_A_MARK = re.compile(rf"(?<=[»\"”’'\)\]{_SUPERSCRIPT}])[₀-₉]+(?:₋[₀-₉]+)?")


def without_notes(text):
    """A sense without its sense links, superscripts and the subscripts after a mark, opening on a
    capital when it opened on a lower-case letter."""
    cut = _SUPERSCRIPTS.sub("", _SUBSCRIPT_AFTER_A_MARK.sub("", _SENSE_LINK.sub("", text)))
    if cut != text:
        text = re.sub(r"\s+([,.;:])", r"\1", cut).strip()
    if text[:1].islower():
        text = text[:1].upper() + text[1:]
    return text


# The rules of the pre-pass (`read_section`), each of which can be turned off alone (the tests, the
# design's measurements).
RULES = ("names and acronyms", "pointers", "carried meanings", "notes")


class SectionReading:
    """What the section says of itself, read in a first pass, for `read_section`: the headwords written
    in lower case with an entry of their own that is no proper noun's (`lowercase`), and, by word as the
    shared rules key it (lowercased), those holding a meaning (`meaning`) and those holding a tagged
    pointer (`tagged`). A sense the names rule sets aside counts for neither."""

    def __init__(self, tabled=frozenset()):
        self.tabled = tabled
        self.lowercase = set()
        self.meaning = set()
        self.tagged = set()
        self._senses = []

    def read(self, entry):
        word = straight(entry.get("word") if isinstance(entry.get("word"), str) else "")
        if not word:
            return
        if word == word.lower() and entry.get("pos") != "name":
            self.lowercase.add(word)
        self._senses.append((entry, word.lower()))

    def settle(self):
        """The second half of the first pass, once every lowercase entry is known."""
        for entry, key in self._senses:
            for sense in entry.get("senses") or ():
                if not isinstance(sense, dict) or self.sets_aside(entry, sense):
                    continue
                if _tagged(sense):
                    self.tagged.add(key)
                elif _holds_a_meaning(sense):
                    self.meaning.add(key)
        self._senses = []

    def sets_aside(self, entry, sense):
        """Whether the names rule keeps a sense off its common word: an acronym's sense or a name's note
        of a capitalised headword whose word has a lowercase entry of its own, or a name's note of a word
        a translation table glosses (`tabled`). None, or what it was."""
        word = straight(entry.get("word") if isinstance(entry.get("word"), str) else "")
        key = word.lower()
        if word != key and key in self.lowercase and (name_note(entry, sense) or acronym_sense(entry, sense)):
            return "names and acronyms off a common word"
        if name_note(entry, sense) and key in self.tabled:
            return "names' notes off a word a table glosses"
        return None


def read_section(src, dst, *, tabled=frozenset(), rules=RULES, stats=None):
    """The Spanish Wiktionary's Italian section as it-es reads it (D4), written to `dst` — a pre-pass after
    the Spanish edition's `read_as_meanings`, in two passes over `src` (`SectionReading`, then the
    rewrite):

    - **Names and acronyms** off a common word, and a name's note off a word a translation table
      glosses (`tabled`: the lemmas the Italian Wiktionary's Spanish translations or the Spanish
      Wiktionary's tables read backwards gloss).
    - **Pointers**: an untagged pointer the edition words lends the word it names (written as a
      `form_of` target, which the shared rules follow) to a headword with no meaning and no tagged
      pointer of its own.
    - **Carried meanings**: the meaning a pointer writes after a colon is read as a meaning, for a
      headword with no meaning of its own.
    - **Notes** out (`without_notes`).

    `rules` names the rules that run (`RULES`, all by default). `(dst, stats)`: how many senses and
    entries each rule moved. An entry left with no sense goes; a line this pass cannot read is written
    as it is, and so is an entry it does not change."""
    stats = stats if stats is not None else collections.Counter()
    reading = SectionReading(tabled if "names and acronyms" in rules else frozenset())
    for entry, _ in entries(src):
        if entry is not None:
            reading.read(entry)
    reading.settle()
    with open(dst, "w", encoding="utf-8") as out:
        for entry, line in entries(src):
            if entry is None or not isinstance(entry.get("senses"), list):
                out.write(line if line.endswith("\n") else line + "\n")
                continue
            key = straight(entry.get("word") if isinstance(entry.get("word"), str) else "").lower()
            senses = entry["senses"]
            read = []
            for sense in senses:
                if not isinstance(sense, dict) or not gloss_text(sense):
                    read.append(sense)
                    continue
                if "names and acronyms" in rules:
                    why = reading.sets_aside(entry, sense)
                    if why:
                        stats[why] += 1
                        continue
                text = gloss_text(sense)
                if "carried meanings" in rules and key not in reading.meaning and _is_pointer(sense):
                    meaning = carried_meaning(text)
                    if meaning:
                        sense = {k: v for k, v in sense.items() if k not in ("form_of", "alt_of")}
                        tags = [tag for tag in sense.get("tags") or () if tag != "form-of"]
                        sense.pop("tags", None)
                        if tags:
                            sense["tags"] = tags
                        sense["glosses"] = [meaning]
                        stats["pointers read for the meaning after their colon"] += 1
                if (
                    "pointers" in rules
                    and key not in reading.meaning
                    and key not in reading.tagged
                    and _is_pointer(sense)
                    and not _tagged(sense)
                ):
                    target = pointer_target(text)
                    if target and target != key:
                        sense = {**sense, "form_of": [{"word": target}]}
                        stats["worded pointers lending their word"] += 1
                if "notes" in rules:
                    cleaned = without_notes(gloss_text(sense))
                    if cleaned != gloss_text(sense):
                        stats["notes taken out or openings capitalised"] += 1
                        sense = _with_text(sense, cleaned) if cleaned else None
                        if sense is None:
                            continue
                read.append(sense)
            if senses and not read:
                stats["entries left with no sense"] += 1
                continue
            if read != senses:
                out.write(json.dumps({**entry, "senses": read}, ensure_ascii=False) + "\n")
            else:
                out.write(line if line.endswith("\n") else line + "\n")
    return dst, stats


# — The Italian Wiktionary's notes, in its Spanish section read backwards —

# Its notes to its own readers, which the reading would otherwise take for Italian words: « vedi
# italiano » (« analogia, vedi italiano » is « analogia »), and the placeholder « definizione mancante;
# se vuoi, aggiungila tu ».
_SPANISH_SECTION_NOTES = re.compile(r"\bvedi italiano\b|\bdefinizione mancante;\s*se vuoi,\s*aggiungila tu\b", re.IGNORECASE)


def without_spanish_section_notes(text):
    """A sense of the Italian Wiktionary's Spanish section without its notes to its readers; None when
    nothing else is left."""
    cut = _SPANISH_SECTION_NOTES.sub("", text)
    cut = re.sub(r"\s*,\s*(?=,|$)", "", cut).strip(" ,;")
    return cut or None


# — Marked words —

# A Spanish word written with a grave accent (« Suscripciòn ») or a degree sign (« ° »), from the
# Italian Wiktionary's editors' keyboards: Spanish writes neither.
_MARKED = re.compile(r"[àèìòùÀÈÌÒÙ°]")


def without_marked_words(table):
    """A table without its Spanish words written with a grave accent or « ° »: the next word comes up,
    a part of speech left with none goes, a word left with none takes no gloss from the table."""
    out = {}
    for word, by_pos in table.items():
        kept = {}
        for upos, natives in by_pos.items():
            words = [native for native in natives if not _MARKED.search(native)]
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


# — The words without a space that Italian's pre-pass splits (a copy of it-en's D4.5 reader) —

# The pieces Italian's pre-pass reads before an apostrophe and splits a word at (change 6's
# `ITALIAN_ELISIONS` and `ITALIAN_ELIDED_PREPOSITIONS`, it-fr's `ELISIONS` and `ELIDED_PREPOSITIONS`):
# test_reduce_it_es.py holds them to it-fr's tables.
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


def split_word(word, forms):
    """Whether `word` is one word Italian's pre-pass splits into two tokens or more (it-fr's
    `split_word`): no space, an Italian token beginning with an elided piece (`d'accordo`,
    `all'improvviso`, `c'è`), which the pre-pass splits there, and no form of the tables — a form is
    looked up whole."""
    return " " not in word and bool(TOKEN.fullmatch(word)) and bool(_ELIDED_START.match(word)) and word not in forms


def section_words(path, forms, wanted, *, edition=EDITION):
    """One pass over the section as read: `(split, entries)` — the words Italian's pre-pass splits
    (`split_word`), glossed as expressions are (`reduce_common.reduce_expressions`' sense rules: a
    proper noun's entry and a pointer left out, each sense whole but past `EXPRESSION_SENSE_LEN`, three
    joined across the word's entries), and the `wanted` headwords (lowercased) glossed the same way
    whatever their part of speech, a proper noun's included (D7: an acronym row's synonyms)."""
    split, found = {}, {}
    for entry, _ in entries(path):
        if entry is None:
            continue
        word = re.sub(r"\s+", " ", straight(entry.get("word") if isinstance(entry.get("word"), str) else "").lower())
        named = (entry.get("pos") or "") == "name"
        into = []
        if word in wanted:
            into.append(found)
        if not named and split_word(word, forms):
            into.append(split)
        if not into:
            continue
        senses = []
        for sense in entry.get("senses") or ():
            text = gloss_text(sense)
            if not text or common._is_form_of(sense, text, edition=edition):
                continue
            cleaned = common.clean_gloss(text, common.EXPRESSION_SENSE_LEN, edition=edition)
            if cleaned:
                senses.append(cleaned)
        if senses:
            for table in into:
                table.setdefault(word, []).append(senses)
    return tuple({w: common._join_senses(per_entry, 3) for w, per_entry in table.items()} for table in (split, found))


def table_split_words(sources, forms, taken, *, edition=EDITION):
    """The words Italian's pre-pass splits that a translation table glosses and the section does not
    (`taken`), from each `(source, order)` in turn, as `reduce_common.fallback_expressions` glosses a
    headword with a space — which it skips for having none."""
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
