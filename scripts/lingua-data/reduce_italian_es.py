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
  describes them (`tu` « Tú », `lo` « El; Lo », `essere` without « Verbo auxiliar »).
- **The Italian word given as Spanish**: refused unless `ITALIAN_ALIKE` names it a Spanish word too —
  the rule `reduce_spanish_tables.is_the_studied_word` reads.
- **The Italian Wiktionary's notes** in its Spanish section, read backwards: « vedi italiano » and
  « definizione mancante; se vuoi, aggiungila tu » taken out, a sense left empty dropped.
- **Marked words**: no Spanish word of a table with a grave accent or « ° ».
- **Split words**: the reader of the words without a space that Italian's pre-pass splits — a copy of
  it-en's D4.5 reader (`reduce_italian_en.py`), so that neither pair's digest follows the other's.

Change 28 (refine-lingua-it-es-glosses) adds three passes, each rule named in `RULES` (`REFINED`) and
measured alone on the committed tables (its design's *Measured*):

- **The section** (`refine_section`): heraldic senses out (D6b: 8 senses, `nudo` « Desnudo »); a
  surname's note off a word Italian's readings make common and a table glosses (D8: 7 rows, `senatore`
  « Senador »); a given name's note read as its Spanish name (D7b: 127 senses, 104 rows, `paolo`
  « Pablo (nombre de pila) », the short form the owner's choice).
- **The tables** (`refine_tables`, `without_verb_forms_translations`): a verb form's translations no
  other source gives (D10a: 85, `faccia` « Cara »); words that are not Spanish, by wordfreq across
  five languages (D5a: 303 items, 40 rows, `evidente` « Evidente »); places' labels (D5b: 17 items,
  `ladro` without « Spagna »); a verb's words off another verb's form in the pivot (D10b: 53 items,
  `dati` « Datos, facto »).
- **The rows** (`refine_rows`), in this order: a feminine's own word (D4: 5 rows, `ragazza` « Chica,
  muchacha, niña »); Wikidata's lone items (D2f: 99, `don` « Don… »); taxa (D9a: 23, `cavallo`
  « Caballo »); a rare first word (D6a: 837, `rapidamente` « Rápidamente… »); agreement (D2: 1,483,
  `caso` « Caso », `andare` « Ir, andar »), keeping a learned suffix's cognate it displaces (D2g, round
  2: 9 rows, `insufficienza` « …; Insuficiencia »); a foreign item a rule brought in out (D5c, round 2:
  154 rows, `marmo` without « alioj »); the missing adverb or preposition (D3b: 2, `spesso` « A
  menudo…; Gordo ») and MarkIT's order (D3a: 9, `ancora` « Todavía; Ancla »), through fr-en's
  `commonest_first` over MarkIT's counts, imported as written; long lists cut (D7a: 11, `inoltre`), their
  items first ordered by the sources giving them (D7c, round 2: `talvolta` « A veces… »); repeats (D9b:
  42 rows and 1 expression, `persona` « Persona (ser humano) »).
- **The reviewed tables**: `FIRST_RUN` (20 rows), `CORRECTIONS` (44 rows, 22 of round 2) and `DI_FAMILY`
  (6 rows), which the rules never move; every gloss a person wrote is in `BY_A_PERSON` (tasks 0.1, 0.2,
  0.4); round 2's rows write a source's words alone (task 0.5).
"""

import collections
import json
import re
import unicodedata

import reduce_common as common
import reduce_edition_es as spanish
import reduce_french_treebank as french_treebank
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
# personal nominativo de 2 persona del singular… », `essere` « Verbo auxiliar … »). Each row is fr-es's
# `CORRECTIONS` shape: keyed by the section's text as the pinned file writes it — an entry's senses, in
# their order —, so that it fires on nothing once the page is corrected; the update that takes the
# correction in says so on its summary line, and the row is removed. The corrected senses are
# meanings: they replace the entry's senses whole, pointers included. Their words come from the
# sources: the Italian Wiktionary's Spanish translations (« Tú », « Ella », « Usted »), the Spanish
# Wiktionary's Spanish tables (« El », « Ellos », « Ellas »), the pivot (« Lo »: the English *him* and
# *it*, the French *le*, the German *ihn*, the Dutch *het* pair `lo` with « lo »), the section's own text
# (« De él, de ella, de ellos, de ellas », « Ustedes »). One row is written by a person, as it-fr's `dati`
# « Données » is: `ne`'s adverb « De allí, de ahí », which no source glosses (the owner, 2026-10-25).
# `corrected` None leaves out the entry, the word keeping its other entries' senses (`essere`'s and
# `avere`'s auxiliaries). Only rows of ranked lemmas: `è`, `sono`, `le`, `gli` and `li` are forms of
# other lemmas in Italian's tables, which a row of theirs would not reach. The owner reads each; each is
# reported to the Spanish Wiktionary.
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
        f"{_DESCRIBED}: the accusative's « lo », which the pivot pairs with `lo` (English him, it; French le; German ihn; Dutch het)",
        _PAGE.format("lo"),
    ),
    Correction(
        "section", "ne", "adv",
        (
            "Indica proveniencia de un lugar ya nombrado precedentemente. No tiene una traducción al español.",
        ),
        ("De allí, de ahí.",),
        f"{_DESCRIBED}: where one comes from, « de allí » — the one row a person wrote, no source glossing it",
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
# design's measurements); `RULES` adds change 28's (`REFINED`).
PRE_PASS = ("names and acronyms", "pointers", "carried meanings", "notes")
RULES = PRE_PASS


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


# ——— Change 28 (refine-lingua-it-es-glosses) ———
#
# it-es's glosses open on the Spanish word their sources agree on, in the order Italian uses the word,
# in Spanish words read as their sources mean them. Three passes, each rule named in `RULES` so that it
# can be measured alone (the design's *Measured*): the section's (`refine_section`, after
# `read_section`), the tables' (`refine_tables`, on the six table-shaped sources before `native_side`
# reads them) and the rows' (`refine_rows`, after it), in the design's order — D4, D2f, D9a, D6a, D2,
# D3, D7a, D9b, then the reviewed tables (D12, D11). fr-en's treebank rule and MarkIT's counts are
# imported as written, as it-fr and it-en import them (D1): they join it-es's rule digest.

# The rules, by the design's decisions.
REFINED = (
    "heraldic senses",  # D6b
    "surnames off common words",  # D8
    "given names",  # D7b
    "verb forms",  # D10
    "not spanish",  # D5a
    "place labels",  # D5b
    "feminine's own word",  # D4
    "wikidata's lone items",  # D2f
    "taxa",  # D9a
    "rare first words",  # D6a
    "agreement",  # D2
    "cognates kept",  # D2g (round 2)
    "foreign items",  # D5c (round 2)
    "missing adverb or preposition",  # D3b
    "markit order",  # D3a
    "long lists",  # D7a
    "lists by use",  # D7c (round 2)
    "repeats",  # D9b
    "reviewed rows",  # D11, D12
)
RULES = PRE_PASS + REFINED

# D2's sources, in their order: the section's definitions, then each table-shaped source.
SOURCES = ("definitions", "direct", "inverted", "wikidata links", "wikidata items", "spanish section", "pivot")


# — The section: what D4 reads of it, heraldry (D6b), given names (D7b), surnames (D8) —

# A feminine's pointer, which the shared rules follow to the masculine's gloss (D4).
_FEMININE = re.compile(r"^Forma del femenino\b")
# A given name's note and its Spanish name (D7b): « Nombre de pila de varón, equivalente del español
# Pablo », « Nombre de varón, equivalente del español José », « Nombre de pila de varón equivalente a Hugo ».
_GIVEN_NAME = re.compile(r"^(?:Nombre de pila|Nombre personal|Nombre)\b[^,]*?,?\s*equivalente (?:del español|al español|a)\s+(.+?)\.?$")


class SectionRecord:
    """What `refine_section` records of the section for D4: the words a feminine's pointer heads
    (`feminine`) and the words holding a sense of their own, no pointer (`meaning`)."""

    def __init__(self):
        self.feminine = set()
        self.meaning = set()


def refine_section(src, dst, *, readings, tabled, rules=RULES, stats=None):
    """The section after `read_section`, as change 28 reads it, written to `dst`:

    - **D6b**: a sense the section files under heraldry goes (`nudo` « Desnudo », not « Dicho de un
      mueble heráldico… »).
    - **D8**: a proper noun's « Apellido… » keeps off a word Italian's readings give a common reading
      (`readings`), where a table-shaped source glosses the word (`tabled`): `senatore` « Senador ».
    - **D7b**: a given name's note reads as its Spanish name: « Pablo (nombre de pila) ».

    and records, for D4, the words a feminine's pointer heads and the words holding a meaning of their
    own (`SectionRecord`). `(dst, record)`. An entry left with no sense goes; a line this pass cannot
    read, and an entry it does not change, are written as they are."""
    stats = stats if stats is not None else collections.Counter()
    record = SectionRecord()
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                out.write(line)
                continue
            if not isinstance(entry, dict):
                out.write(line)
                continue
            key = straight(entry.get("word") if isinstance(entry.get("word"), str) else "").lower()
            senses = entry.get("senses") or []
            named = entry.get("pos") == "name"
            kept = []
            for sense in senses:
                if not isinstance(sense, dict):
                    kept.append(sense)
                    continue
                text = gloss_text(sense)
                if _FEMININE.match(text):
                    record.feminine.add(key)
                elif text and not _tagged(sense):
                    record.meaning.add(key)
                if "heraldic senses" in rules and "heraldry" in (sense.get("topics") or ()):
                    stats["heraldic senses out (D6b)"] += 1
                    continue
                if "surnames off common words" in rules and named and _NAME_NOTE.match(text) and text.startswith("Apellido"):
                    if readings.get(key, set()) - {"PROPN", "X"} and key in tabled:
                        stats["surnames' notes off a word Italian reads as common (D8)"] += 1
                        continue
                if "given names" in rules and named:
                    found = _GIVEN_NAME.match(text)
                    if found:
                        sense = {**sense, "glosses": [f"{found.group(1).strip()} (nombre de pila)"]}
                        stats["given names read as their Spanish name (D7b)"] += 1
                kept.append(sense)
            if senses and not kept:
                continue
            out.write(json.dumps({**entry, "senses": kept}, ensure_ascii=False) + "\n" if kept != senses else line)
    return dst, record


# — The tables: verb forms (D10), words that are not Spanish (D5a), place labels (D5b) —

# A sense of the Italian Wiktionary's translations that names a verb's form (D10a): « 1ª persona
# singolare del congiuntivo presente di fare », « participio passato di … ».
_INFLECTION = re.compile(
    r"\bpersona (?:singolare|plurale) (?:del|dell'|di)\s*(?:congiuntivo|indicativo|imperativo|condizionale)"
    r"|\b(?:participio (?:passato|presente)|gerundio) di\b",
    re.I,
)
# D5a: a one-word Spanish item rarer than this in Spanish…
NOT_SPANISH_BELOW = 1.0
# …and at least this common in one of these languages is no Spanish word.
NOT_SPANISH_ELSEWHERE = 3.0
NOT_SPANISH_LANGUAGES = ("en", "fr", "pt", "it", "de")


def without_verb_forms_translations(src, dst, others, stats=None):
    """The Italian Wiktionary's translations without those filed under a verb form's sense
    (`_INFLECTION`) that no other source gives the word (`others`: word → its Spanish words, lowercased,
    in every other table-shaped source) — D10a: `faccia` « Cara », not « haga, hagas ». Written to `dst`."""
    stats = stats if stats is not None else collections.Counter()
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                out.write(line)
                continue
            if not isinstance(entry, dict):
                out.write(line)
                continue
            key = straight(entry.get("word") or "").lower()
            translations = entry.get("translations") or ()
            kept = [
                t
                for t in translations
                if not (_INFLECTION.search(t.get("sense") or "") and (t.get("word") or "").lower() not in others.get(key, ()))
            ]
            stats["verb forms' translations out (D10a)"] += len(translations) - len(kept)
            out.write(json.dumps({**entry, "translations": kept}, ensure_ascii=False) + "\n")
    return dst


def _filtered(table, keep):
    """A table (`word → {UPOS: [Spanish word, …]}`) with the words `keep(word, upos, native)` refuses
    left out: a part of speech left with none goes, a word left with none too."""
    out = {}
    for word, by_pos in table.items():
        kept = {}
        for upos, natives in by_pos.items():
            words = [native for native in natives if keep(word, upos, native)]
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


def _count(table):
    return sum(len(natives) for by_pos in table.values() for natives in by_pos.values())


def not_spanish(native, zipf):
    """D5a: a one-word item Spanish rarely writes (`NOT_SPANISH_BELOW`) that another language writes
    often (`NOT_SPANISH_ELSEWHERE`, `NOT_SPANISH_LANGUAGES`) — « Evident », « Obéir » —, unless
    `ITALIAN_ALIKE` names it."""
    word = native.strip()
    if " " in word or not word:
        return False
    lowered = word.lower()
    if zipf(lowered, "es") >= NOT_SPANISH_BELOW:
        return False
    return max(zipf(lowered, language) for language in NOT_SPANISH_LANGUAGES) >= NOT_SPANISH_ELSEWHERE and lowered not in ITALIAN_ALIKE


def refine_tables(sources, *, readings, zipf, lemmas, verb_forms, rules=RULES, stats=None):
    """The six table-shaped sources (`sources`: name → table, in D2's order) as change 28 reads them:

    - **D5a**: no word that is not Spanish (`not_spanish`), in every table.
    - **D5b**: in the two Wiktionaries' tables, a capitalised one-word item that follows another in its
      list, no acronym, which Italian's tables keep as a lemma with no reading (`lemmas`, `readings`),
      is a place's label, no word (`ladro` « Ladrón, chorizo », not « Spagna »).
    - **D10b**: in the pivot, a verb's words off a word Italian reads only as another verb's form
      (`verb_forms`) and as no verb, unless they are its only part of speech (`dati` « Datos, facto »).

    The tables, by name, in the same order."""
    stats = stats if stats is not None else collections.Counter()
    out = {}
    for name, table in sources.items():
        if "not spanish" in rules:
            before = _count(table)
            table = _filtered(table, lambda w, u, n: not not_spanish(n, zipf))
            stats["words that are not Spanish out (D5a)"] += before - _count(table)
        if "place labels" in rules and name in ("direct", "inverted"):
            firsts = {(w, u): natives[0] for w, by_pos in table.items() for u, natives in by_pos.items() if natives}

            def label(w, u, n, firsts=firsts):
                return (
                    n[:1].isupper()
                    and " " not in n
                    and not common._acronym(n)
                    and firsts.get((w, u)) != n
                    and not readings.get(n.lower())
                    and n.lower() in lemmas
                )

            before = _count(table)
            table = _filtered(table, lambda w, u, n: not label(w, u, n))
            stats["place labels out (D5b)"] += before - _count(table)
        if "verb forms" in rules and name == "pivot":

            def verb_of_a_form(w, u, n, table=table):
                return u == "VERB" and "VERB" not in readings.get(w, set()) and w in verb_forms and len(table.get(w, {})) > 1

            before = _count(table)
            table = _filtered(table, lambda w, u, n: not verb_of_a_form(w, u, n))
            stats["verb words off another verb's form (D10b)"] += before - _count(table)
        out[name] = table
    return out


# — The rows (D2, D3, D4, D6a, D7a, D9) —

# A row's senses are `(UPOS, sense)` pairs, read from its gloss and its runs (`senses_of`) and written
# back (`joined`).


def senses_of(gloss, runs):
    """A row's senses, each with its part of speech: `[(UPOS, sense), …]`."""
    parts = gloss.split("; ")
    out, i = [], 0
    for upos, n in runs:
        for _ in range(n):
            if i < len(parts):
                out.append((upos, parts[i]))
                i += 1
    while i < len(parts):
        out.append((runs[-1][0] if runs else "X", parts[i]))
        i += 1
    return out


def joined(senses):
    """A row's `(gloss, runs)` from its senses."""
    runs = []
    for upos, _ in senses:
        if runs and runs[-1][0] == upos:
            runs[-1][1] += 1
        else:
            runs.append([upos, 1])
    return "; ".join(s for _, s in senses), [tuple(r) for r in runs]


def word_of(item):
    """A sense's item as D2 compares it: its parentheses out, « . », « ¡ », « ! », « ¿ », « ? » and spaces
    off its ends, lowercased."""
    return re.sub(r"\s*\([^()]*\)", "", item).strip(" .¡!¿?").lower()


def split_items(sense):
    """A sense's items, split at « , » before a space outside parentheses."""
    out, depth, current, k = [], 0, "", 0
    while k < len(sense):
        c = sense[k]
        if c == "(":
            depth += 1
        elif c == ")":
            depth = max(0, depth - 1)
        if c == "," and depth == 0 and sense[k + 1 : k + 2] == " ":
            out.append(current)
            current = ""
            k += 2
            continue
        current += c
        k += 1
    out.append(current)
    return out


_OR = re.compile(r"\s+[ou]\s+")


def or_split(parts):
    """Items split again at « o »/« u » when short (at most five words): « Mas o pero » is two."""
    out = []
    for part in parts:
        if _OR.search(word_of(part)) and len(word_of(part).split()) <= 5:
            out.extend(p for p in _OR.split(part) if p.strip())
        else:
            out.append(part)
    return out


def words_of(sense):
    """A sense's words as D2 counts them (`word_of` of each item)."""
    return [word_of(x) for x in or_split(split_items(sense)) if word_of(x)]


def _capital(s):
    return s[:1].upper() + s[1:]


def _lower(s):
    return s if (len(s) > 1 and s[0].isupper() and s[1].isupper()) else s[:1].lower() + s[1:]


def winner_first(sense, winner):
    """A sense opening on `winner`, the item that writes it moved first."""
    parts = split_items(sense)
    if winner not in [word_of(x) for x in parts]:
        parts = or_split(parts)
    at = [word_of(x) for x in parts].index(winner)
    if at == 0:
        return sense
    return ", ".join([_capital(parts[at]), _lower(parts[0]), *parts[1:at], *parts[at + 1 :]])


def grouped(senses):
    """The senses of the first sense's part of speech first, each part's in its order."""
    return [x for x in senses if x[0] == senses[0][0]] + [x for x in senses if x[0] != senses[0][0]]


# The most senses a row the rules move keeps (D2d).
MOST_SENSES = 8
# D3b: MarkIT reads the missing part of speech at least so often…
FILL_MIN = 10
# D6a: a first word rarer than this in Spanish (Zipf) yields to a commoner one.
RARE_BELOW = 2.0
# D7a: a sense of at least so many items, three quarters of them of at most four words…
LIST_MIN = 8
# …keeps its first three.
LIST_KEPT = 3
# D2g (round 2): a displaced first word is kept when it is the lemma's cognate (`cognate`) and Spanish
# writes it at least this often (Zipf).
COGNATE_COMMON = 3.0
# …a cognate: a lemma of a learned suffix (`LEARNED_SUFFIXES`), written as Spanish writes that suffix,
# and the word, both without accents, at most this normalised edit distance apart.
COGNATE_DISTANCE = 0.2
# The learned suffixes, Italian's and Spanish's (D2g): words formed with them are rarely false friends,
# unlike verbs and short words (`mirare` « apuntar », not « mirar »; `balla`, `accudire`, `contestare`).
LEARNED_SUFFIXES = (
    ("zione", "ción"),
    ("sione", "sión"),
    ("enza", "encia"),
    ("anza", "ancia"),
    ("ità", "idad"),
    ("mente", "mente"),
    ("abile", "able"),
    ("ibile", "ible"),
)
_DOUBLED = re.compile(r"([b-df-hj-np-tv-z])\1")
# D5c (round 2): an item Spanish does not write (Zipf 0), or writes under this while another language
# writes it at least `NOT_SPANISH_ELSEWHERE`, is no Spanish word.
FOREIGN_BELOW = 1.5
# D9a: a taxon's name opening a sense: « (Equus caballus) Caballo ».
_TAXON = re.compile(r"^\([A-Z][a-z]+(?: [a-z]+)+\)\s+")


def _plain(word):
    return "".join(c for c in unicodedata.normalize("NFD", word) if unicodedata.category(c) != "Mn")


def as_spanish(lemma):
    """An Italian lemma of a learned suffix written as Spanish writes it (D2g): its suffix Spanish's,
    its doubled consonants single (`insufficienza` → `insuficiencia`, `appartenenza` →
    `apartenencia`); None for a lemma of no learned suffix."""
    for italian_, spanish_ in LEARNED_SUFFIXES:
        if lemma.endswith(italian_) and len(lemma) > len(italian_) + 2:
            return _DOUBLED.sub(r"\1", lemma[: -len(italian_)]) + spanish_
    return None


def _distance(a, b):
    """Levenshtein's distance between two words."""
    previous = list(range(len(b) + 1))
    for i, x in enumerate(a, 1):
        current = [i]
        for j, y in enumerate(b, 1):
            current.append(min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + (x != y)))
        previous = current
    return previous[-1]


def cognate(lemma, word):
    """Whether a Spanish word is the cognate of a lemma of a learned suffix (D2g): one word ending as
    Spanish writes the suffix, accent included, at most `COGNATE_DISTANCE` from the lemma written as
    Spanish writes it (`as_spanish`), both without accents, over the longer."""
    expected = as_spanish(lemma)
    word = word.lower()
    if expected is None or " " in word:
        return False
    suffix = next(spanish_ for italian_, spanish_ in LEARNED_SUFFIXES if lemma.endswith(italian_))
    if not word.endswith(suffix):
        return False
    a, b = _plain(expected), _plain(word)
    return _distance(a, b) / max(len(a), len(b)) <= COGNATE_DISTANCE


class Row:
    """One lemma's sources as D2 reads them: each source's gloss (`glossed`: name → `(gloss, runs)`), in
    D2's order, its words (`words`: name → [word, …]) and their support — the sources giving each, each
    of the pivot's tables a source (`support`)."""

    def __init__(self, lemma, glossed, pivot):
        self.lemma = lemma
        self.glossed = glossed
        self.names = [name for name in SOURCES if name in glossed]
        self.words = {name: [w for _, s in senses_of(*glossed[name]) for w in words_of(s)] for name in self.names}
        self.support = collections.Counter(w for name in self.names for w in set(self.words[name]))
        if "pivot" in self.names:
            for w in set(self.words["pivot"]):
                self.support[w] += max(0, pivot_tables(pivot, lemma, w) - 1)

    def place(self, word):
        """Where a word first comes: `(source's place, the word's place in it)`."""
        for k, name in enumerate(self.names):
            if word in self.words[name]:
                return (k, self.words[name].index(word))
        return (99, 99)


def pivot_tables(pivot, lemma, word):
    """How many of the pivot's tables pair `lemma` with `word` (D2a), whatever their part of speech."""
    if pivot is None or lemma not in pivot.tables:
        return 0
    found = set()
    for natives in pivot.tables[lemma].values():
        for native, tables_ in natives.items():
            if word_of(native) == word:
                found |= tables_
    return len(found)


def agreement(row, current, zipf=None, rules=RULES):
    """D2: the row opens on the word its sources agree on — when the word most sources open on is opened
    on by two or more, by more than open on the row's first word, and given by more sources than it; or
    when the row's first word is given by its own source alone and another word of the row's part of
    speech by two or more. The sense holding the winner opens the row, the winner first in it; a winner
    the row lacks brings the earliest source's sense holding it, and the row's first sense goes when none
    of its words another source gives. None when the row stays."""
    senses = senses_of(*current)
    if not senses or not words_of(senses[0][1]):
        return None
    first = words_of(senses[0][1])[0]
    names, words, support = row.names, row.words, row.support
    opening = collections.Counter(words[name][0] for name in names if words[name])
    winner = None
    best = sorted(
        opening.items(),
        key=lambda kv: (-kv[1], min(k for k, name in enumerate(names) if words[name] and words[name][0] == kv[0])),
    )
    if best and best[0][0] != first and best[0][1] >= 2 and best[0][1] > opening.get(first, 0) and support[best[0][0]] > support[first]:
        winner = best[0][0]
    if winner is None and support.get(first, 0) <= 1:
        upos = senses[0][0]
        same = {w for name in names for u, s in senses_of(*row.glossed[name]) if u == upos for w in words_of(s)}
        candidates = [(w, n) for w, n in support.items() if n >= 2 and w != first and w in same]
        if candidates:
            candidates.sort(key=lambda wn: (-wn[1], row.place(wn[0])))
            winner = candidates[0][0]
    if winner is None:
        return None
    for k, (upos, sense) in enumerate(senses):
        if winner in words_of(sense):
            moved = [(upos, winner_first(sense, winner))] + [x for j, x in enumerate(senses) if j != k]
            return joined(grouped(moved)[:MOST_SENSES])
    for name in names:
        if winner in words[name]:
            theirs = senses_of(*row.glossed[name])
            k = next(j for j, x in enumerate(theirs) if winner in words_of(x[1]))
            lead = (theirs[k][0], winner_first(theirs[k][1], winner))
            rest = senses
            if all(support[w] <= 1 for w in words_of(senses[0][1])):
                rest = senses[1:]
                # D2g (round 2): the lemma's common cognate stays, as a sense of its own after the lead.
                if "cognates kept" in rules and zipf is not None and cognate(row.lemma, first) and zipf(first, "es") >= COGNATE_COMMON:
                    item = split_items(senses[0][1])[0].strip()
                    rest = [(senses[0][0], _capital(item))] + rest
            moved = [lead] + [x for x in rest if words_of(x[1]) != words_of(lead[1])]
            return joined(grouped(moved)[:MOST_SENSES])
    return None


def lone_wikidata_items(row):
    """D2f: a row Wikidata's lexemes of one item gloss, whose first word no other source gives, takes the
    gloss of the next source whose first word two sources give (`don` « Don… », not « D. »)."""
    names, words, support = row.names, row.words, row.support
    if not names or names[0] != "wikidata items":
        return None
    if not words["wikidata items"] or support[words["wikidata items"][0]] >= 2:
        return None
    for name in names[1:]:
        if words[name] and support[words[name][0]] >= 2:
            return row.glossed[name]
    return None


def feminine_s_own_word(row, current, record, readings):
    """D4: a noun whose section entries hold no meaning of their own but a feminine's pointer, lent its
    masculine's gloss (`ragazza` « Chico… »), takes the gloss of the next source whose first word two
    sources give, when that word is the lent word's feminine (« chico » → « chica »)."""
    lemma = row.lemma
    if lemma not in record.feminine or lemma in record.meaning or "NOUN" not in readings.get(lemma, set()):
        return None
    if "definitions" not in row.glossed:
        return None
    names = [name for name in row.names if name != "definitions"]
    support = collections.Counter(w for name in names for w in set(row.words[name]))
    lent = words_of(senses_of(*current)[0][1])
    feminines = {_plain(w[:-1] + "a") if w.endswith("o") else _plain(w + "a") for w in lent}
    for name in names:
        words = row.words[name]
        if words and support[words[0]] >= 2 and _plain(words[0]) in feminines:
            return row.glossed[name]
    return None


def without_taxa(current):
    """D9a: a taxon's name opening a sense goes (« (Equus caballus) Caballo » → « Caballo »), each sense
    opening on a capital."""
    senses = senses_of(*current)
    out = [(u, _capital(_TAXON.sub("", s))) for u, s in senses]
    return joined(out) if out != senses else None


def rare_first_word(row, current, zipf):
    """D6a: a row whose first word Spanish writes rarely (`RARE_BELOW`) opens on the first commoner word of
    its first sense, or else on the first sense of its part of speech of the next source whose first word
    is commoner (`rapidamente` « Rápidamente… », not « Aquejadamente »)."""
    senses = senses_of(*current)
    upos, sense = senses[0]
    parts = split_items(sense)
    first = word_of(parts[0])
    if not first or zipf(first, "es") >= RARE_BELOW:
        return None
    for k, part in enumerate(parts[1:], 1):
        if zipf(word_of(part), "es") >= RARE_BELOW:
            parts = [_capital(part)] + [_lower(x) for x in parts[:k]] + parts[k + 1 :]
            return joined([(upos, ", ".join(parts))] + senses[1:])
    shown = row.names[0]
    for name in SOURCES[SOURCES.index(shown) + 1 :]:
        if name not in row.glossed:
            continue
        same = [x for x in senses_of(*row.glossed[name]) if x[0] == upos]
        if same and words_of(same[0][1]) and zipf(words_of(same[0][1])[0], "es") >= RARE_BELOW:
            return joined(same[:1] + senses[1:])
    return None


def missing_adverb_or_preposition(row, current, counts):
    """D3b: a row holding only nouns, adjectives and verbs, for a word MarkIT reads at least `FILL_MIN`
    times and twice as often under a function part of speech the row lacks, gains the first sense of that
    part of speech from the next source whose first word of it two sources give, ahead of the row's
    senses (`spesso` « A menudo…; Gordo »). A determiner or a pronoun is not filled in beside an
    adjective (the Spanish edition writes possessives as adjectives)."""
    senses = senses_of(*current)
    have = {u for u, _ in senses}
    found = counts.get(row.lemma, {})
    candidates = {u: n for u, n in found.items() if u != "PROPN"}
    if not candidates:
        return None
    best = max(candidates, key=lambda u: candidates[u])
    if best in have:
        return None
    n_best, first = candidates[best], senses[0][0]
    if n_best < FILL_MIN or n_best < 2 * found.get(first, 0):
        return None
    if best not in french_treebank.TREEBANK_FIRST or any(u not in ("NOUN", "ADJ", "VERB") for u in have):
        return None
    if best in ("DET", "PRON") and "ADJ" in have:
        return None
    for name in SOURCES:
        if name not in row.glossed:
            continue
        added = [x for x in senses_of(*row.glossed[name]) if x[0] == best]
        if not added:
            continue
        word = words_of(added[0][1])[0] if words_of(added[0][1]) else None
        given = sum(1 for other in SOURCES if other in row.glossed and word in {w for _, s in senses_of(*row.glossed[other]) for w in words_of(s)})
        if given < 2:
            continue
        moved = senses + added[:1]
        return joined([x for x in moved if x[0] == best] + [x for x in moved if x[0] != best])
    return None


def markit_order(lemma, current, counts):
    """D3a: a function word's row opens on MarkIT's commonest part of speech
    (`reduce_french_treebank.commonest_first`, it-fr's and it-en's rule): `ancora` « Todavía; Ancla »."""
    senses = senses_of(*current)
    upos = [u for u, _ in senses]
    order = french_treebank.commonest_first(french_treebank.word_key(lemma), upos, upos[0], counts)
    if order is None:
        return None
    return joined([senses[k] for k in order])


def long_lists_cut(current, row=None, zipf=None):
    """D7a: a sense of at least `LIST_MIN` items, three quarters of them of at most four words and none
    ending a sentence, keeps its first `LIST_KEPT` (`inoltre` « Además, adicionalmente, allende »);
    D7c (round 2), given the row's sources and wordfreq: its items first ordered by the sources giving
    them (`Row.support`), then by Spanish frequency, the list's order breaking ties (`talvolta` « A
    veces, de vez en cuando… », `gratis` without « de bóbilis »)."""
    senses = senses_of(*current)
    out = []
    for upos, sense in senses:
        parts = or_split(split_items(sense))
        if (
            len(parts) >= LIST_MIN
            and sum(len(p.split()) <= 4 for p in parts) >= 0.75 * len(parts)
            and not any(". " in p for p in parts)
        ):
            parts = [p.strip() for p in parts]
            if row is not None and zipf is not None:
                parts = [_lower(parts[0])] + parts[1:]
                parts = sorted(parts, key=lambda p: (-row.support[word_of(p)], -zipf(word_of(p), "es")))
            out.append((upos, ", ".join([_capital(parts[0])] + parts[1:LIST_KEPT])))
        else:
            out.append((upos, sense))
    return joined(out) if out != senses else None


def foreign(item, zipf):
    """D5c (round 2): an item Spanish does not write — wordfreq knows it not at all — or writes rarely
    (`FOREIGN_BELOW`) while another language writes it often (`NOT_SPANISH_ELSEWHERE`), unless
    `ITALIAN_ALIKE` names it: « America ispanica », « afogarse », « alioj », « multinational »."""
    word = word_of(item)
    if not word or word in ITALIAN_ALIKE:
        return False
    spanish_ = zipf(word, "es")
    if spanish_ == 0:
        return True
    return spanish_ < FOREIGN_BELOW and max(zipf(word, language) for language in NOT_SPANISH_LANGUAGES) >= NOT_SPANISH_ELSEWHERE


def without_foreign_items(current, native, zipf):
    """D5c (round 2): after agreement, the items that are no Spanish word (`foreign`) go from a sense the
    row rules brought into the row — a sense not among the row's own as `native_side` wrote it
    (`native`) —, never a sense's last item (`criterio` « Criterio, sabiduría », not « …,
    criteriómetro »; `marmo` without « alioj »)."""
    senses = senses_of(*current)
    own = set(native)
    out = []
    for upos, sense in senses:
        if sense in own:
            out.append((upos, sense))
            continue
        parts = split_items(sense)
        kept = [p for p in parts if not foreign(p, zipf)]
        if kept and kept != parts:
            if kept[0] != parts[0]:
                kept[0] = _capital(kept[0].strip())
            out.append((upos, ", ".join(kept)))
        else:
            out.append((upos, sense))
    return joined(out) if out != senses else None


def without_repeats(current):
    """D9b: a sense with no parenthesis equal to an earlier sense of any part of speech goes (`grazie`
    « ¡Gracias! », not « Gracias » again); within a part of speech, a sense equal to another once their
    parentheses are out goes when it has none, or when the other has none (`persona` « Persona (ser
    humano) »), and so does a sense with no parenthesis whose every word another sense says; an item
    repeated in a sense goes (`islandese` « Islandés »)."""
    senses = senses_of(*current)
    out, seen, every = [], collections.defaultdict(list), []

    def bare(s):
        return "(" not in s

    for upos, sense in senses:
        key = tuple(words_of(sense))
        if key in every and bare(sense):
            continue
        every.append(key)
        if bare(sense) and key and any(set(key) < set(k) for k, _ in seen[upos]):
            continue
        before = [b for k, b in seen[upos] if k == key]
        if before and (any(before) or bare(sense)):
            continue
        seen[upos].append((key, bare(sense)))
        out.append((upos, sense))
    once = []
    for upos, sense in out:
        items, said = [], set()
        for item in split_items(sense):
            if word_of(item) in said:
                continue
            said.add(word_of(item))
            items.append(item)
        once.append((upos, ", ".join(items)))
    return joined(once) if once != senses else None


def without_expression_repeats(expressions, stats=None):
    """D9b for expressions: a sense that only opens another goes (`d'accordo` « De acuerdo, vale », not
    « De acuerdo » too), and a repeated one."""
    stats = stats if stats is not None else collections.Counter()
    for headword, gloss in list(expressions.items()):
        parts = gloss.split("; ")
        kept = []
        for part in parts:
            words = words_of(part)
            if part in kept or any(words_of(q)[: len(words)] == words and len(words_of(q)) > len(words) for q in parts if q != part):
                continue
            kept.append(part)
        if kept != parts:
            expressions[headword] = "; ".join(kept)
            stats["expressions' repeated senses out (D9b)"] += 1


# — The reviewed tables (D11, D12) —
#
# A row no rule reaches, read by a person: each with its reason and its page. A gloss no source gives is
# **written by a person** and marked so (`BY_A_PERSON`), approved by the owner (tasks 0.1, 0.2, 0.4).

# D12's `FIRST_RUN`: the row opens on a sense a source D2 reads gives the word — `text`, of `source` — and
# keeps its other senses (`others`) or not. The reduction fails on a row whose sense the source no longer
# writes.
FirstRun = collections.namedtuple("FirstRun", "source text others reason")
_REVIEWED_AFTER = "added after the review of this proposal, same class as task 0.2"
_OWNER_S = "task 0.2, the owner's"

FIRST_RUN = {
    # The 300 commonest.
    "bene": FirstRun("definitions", "Bien (excelentemente)", True, "the adjective « Aristocrático, noble » first; MarkIT reads too few adverbs (8)"),
    "solo": FirstRun("pivot", "Sólo, solamente, únicamente", True, "the adverb's agreed word is the pivot's alone; « Único » kept"),
    "contro": FirstRun("pivot", "Contra", False, "« Versus, en vez de, en contra; …; Por, contrario, conta »"),
    "posto": FirstRun("pivot", "Lugar, sitio, puesto", False, "« En », a slip two sources share"),
    "nulla": FirstRun("pivot", "Nada, vacío, nihilidad", False, "« Ninguna cosa, ni flores, naranjas », a translation split into its words"),
    "subito": FirstRun("pivot", "Inmediatamente, enseguida, ahora mismo", False, "« Súbito », a false friend"),
    "ci": FirstRun("pivot", "Nos, nosotros, ahí", False, "« …; De esto, aquí, por allí; Ce; No, hay »"),
    "che": FirstRun("definitions", "Que", True, "« Qué » first; the relative and the conjunction are the common uses"),
    "sotto": FirstRun("pivot", "Bajo, debajo de, so", True, "D2 opens it on the adverb « Abajo »; the preposition is MarkIT's (10 against 1)"),
    "ora": FirstRun("inverted", "Ahora", True, "the noun « Hora » alone; MarkIT reads 6 adverbs, too few"),
    "ma": FirstRun("pivot", "Pero, mas, sino", False, "« Mas o pero; Indicador emocional o enfático »"),
    # Added after the review of this proposal, same class as task 0.2: a sense a source gives.
    "entrambi": FirstRun("inverted", "Ambos", False, f"D2 opens it on the pivot's feminine « Ambas » — {_REVIEWED_AFTER}"),
    "bravo": FirstRun("pivot", "Bueno, bravo, hábil", True, f"D2 opens it on the obsolete « Bravo, gallardo… », a false friend — {_REVIEWED_AFTER}"),
    "costume": FirstRun("pivot", "Costumbre, traje, disfraz", False, f"« Costumbre » alone, the review's false friend — {_REVIEWED_AFTER}"),
    "alunna": FirstRun("pivot", "Alumna, alumno, escolar", False, f"the lent masculine « Alumno » (D4 needs two sources) — {_REVIEWED_AFTER}"),
    "voglia": FirstRun("inverted", "Gana", True, f"D2 opens it on « Lunar », the birthmark — {_REVIEWED_AFTER}"),
    # Task 0.2, the owner's.
    "stipendio": FirstRun("direct", "Sueldo", False, f"« Estipendio », a false friend — {_OWNER_S}"),
    "volto": FirstRun("pivot", "Rostro, faz, cara", False, f"« Contornado », a heraldic term two sources give — {_OWNER_S}"),
    "decina": FirstRun("spanish section", "Decena", False, f"« Década » (Wikidata's item and the pivot) — {_OWNER_S}"),
    "aderenza": FirstRun("pivot", "Adhesión, adherencia, viscosidad", False, f"« Tractor », a Wikidata item two sources share — {_OWNER_S}"),
}  # fmt: skip

# D12's `CORRECTIONS` (it-fr's and change 23's shape): a source's slip, keyed by the source, the headword,
# the part of speech and the text of the source's sense it answers (`Correction`: `source`, `headword`,
# `pos`, `text`), and the row as the correction writes it — `corrected`, its `(UPOS, sense)` pairs, or
# None for no gloss —, with its reason and the page reported upstream. A correction whose source no
# longer writes its text fires on nothing, and the update's summary line names it (`refine_rows`). The
# page is the one whose entry or table wrote the source's text: a pivot's word, the other Wiktionary's
# entry whose table lists it.
_ES = "https://es.wiktionary.org/wiki/{}"
_IT = "https://it.wiktionary.org/wiki/{}"
_EN = "https://en.wiktionary.org/wiki/{}"
_WD = "https://www.wikidata.org/wiki/Special:Search?search={}&ns146=1"
_PERSON = "written by a person"
_ROUND_2 = "round 2, the specialist's review, the owner's (2026-10-25)"

CORRECTIONS = (
    # The 300 commonest.
    Correction(
        "definitions", "a", "ADP", "En (cuando se trata de un concepto más que un lugar físico)",
        (("ADP", "A"), ("ADP", "En (cuando se trata de un concepto más que un lugar físico)")),
        f"« A » {_PERSON} (task 0.1): no source glosses a one-letter word; the section's sense kept after it",
        _ES.format("a"),
    ),
    Correction(
        "pivot", "te", "PRON", "La, lo, te", (("PRON", "Ti, te"),),
        f"« ti » {_PERSON} (task 0.1): the stressed pronoun after a preposition (« per te »); « la », « lo », clitic tables' words",
        _EN.format("you"),
    ),
    Correction(
        "pivot", "ti", "PRON", "Te, ti, la", (("PRON", "Te, ti"),),
        "« la », a clitic table's word", _EN.format("you"),
    ),
    Correction(
        "inverted", "cui", "PRON", "Quien", (("PRON", "Que"), ("PRON", "Cuyo")),
        "« Quien » alone; the relative's words, the Spanish section's « Que » and the pivot's « Cuyo »",
        _ES.format("cui"),
    ),
    Correction(
        "definitions", "uno", "DET", "Forma de un antes de «s impura» (la que va seguida de consonante)",
        (("NUM", "Uno"), ("PRON", "Uno (referencia a persona de identidad desconocida, indefinida o irrelevante)")),
        "a note on the form, which D3a would open the row on: out, the row's other senses kept",
        _ES.format("uno"),
    ),
    Correction(
        "pivot", "the", "NOUN", "Té", None,
        "the English article on Italian pages, read as the Russian « чай »; it-en leaves `the` unglossed too (change 23's D9)",
        "https://ru.wiktionary.org/wiki/чай",
    ),
    # Task 0.2, the owner's.
    Correction(
        "direct", "tasso", "NOUN", "Tejón, tejo", (("NOUN", "Tasa"), ("NOUN", "Tejón, tejo")),
        f"the rate first, the pivot's « tasa » — {_OWNER_S}", _IT.format("tasso"),
    ),
    Correction(
        "definitions", "svezia", "PROPN", "Suécia", (("PROPN", "Suecia"),),
        f"« Suecia » {_PERSON}: « Suécia » is the Portuguese spelling — {_OWNER_S}", _ES.format("Svezia"),
    ),
    Correction(
        "pivot", "salsa", "NOUN", "Salsa, greve, gravy", (("NOUN", "Salsa"),),
        f"« greve », « gravy », no Spanish words — {_OWNER_S}", _EN.format("gravy"),
    ),
    Correction(
        "pivot", "efficiente", "VERB", "Efecto, producir", (("ADJ", "Eficiente, eficaz, efectivo"),),
        f"a Polish translation split into its words — {_OWNER_S}", "https://pl.wiktionary.org/wiki/działać",
    ),
    Correction(
        "pivot", "tag", "NOUN", "Chapa", None,
        f"no source gives « etiqueta »: unglossed — {_OWNER_S}", _EN.format("tag"),
    ),
    Correction(
        "inverted", "trad", "NOUN", "Maletero", None,
        f"another word's translation (the car's boot): unglossed — {_OWNER_S}", _ES.format("maletero"),
    ),
    # Added after the review of this proposal, same class as task 0.2: a source's word, or a wrong word out.
    Correction(
        "definitions", "porca", "NOUN", "(Sus scrofa domesticus) cerdo, puerco, chancho",
        (("NOUN", "Cerda"), ("NOUN", "Caballón, camellón")),
        f"D2 drops the lent masculine for « Caballón »; *porca* is mostly the sow, the pivot's « cerda » — {_REVIEWED_AFTER}",
        _ES.format("porca"),
    ),
    Correction(
        "pivot", "qualunque", "DET", "Quequier", (("DET", "Cualquier, cualquiera, cualesquiera"),),
        f"« Quequier », no Spanish word — {_REVIEWED_AFTER}", _EN.format("whatever"),
    ),
    Correction(
        "pivot", "appunto", "INTJ", "Exactamente, exacto, école",
        (("ADV", "Exactamente, exacto"), ("NOUN", "Broma, reto, chiste")),
        f"« école », French; the adverb first (MarkIT: 8 adverbs) — {_REVIEWED_AFTER}", _EN.format("exactly"),
    ),
    Correction(
        "wikidata items", "stop", "NOUN", "Fin", (("INTJ", "Alto, pare"),),
        f"Wikidata's lone item; the pivot's words, its « pelota corta » (the German Stoppball's) out — {_REVIEWED_AFTER}", _WD.format("stop"),
    ),
    Correction(
        "definitions", "fanciulla", "NOUN", "Chico, muchacho, chaval, zagal, jovencito, mozo", (("NOUN", "Moza"),),
        f"the lent masculine; the pivot's feminine « Moza » — {_REVIEWED_AFTER}", _ES.format("fanciulla"),
    ),
    Correction(
        "definitions", "ragazza", "NOUN", "Chico, muchacho, cabro",
        (("NOUN", "Chica, muchacha, niña"), ("NOUN", "Novia")),
        f"the lent masculine; D4's row with the pivot's « novia » (English and German tables) — {_REVIEWED_AFTER}",
        _ES.format("ragazza"),
    ),
    Correction(
        "pivot", "presa", "NOUN", "Enchufe, tomacorriente, toma", (("NOUN", "Toma, captura"),),
        f"D2 opens it on the socket and drops the section's « Captura » — {_REVIEWED_AFTER}", _EN.format("socket"),
    ),
    # Task 0.4, the owner's (2026-10-25): three rows a person's word fixes, no source giving it.
    Correction(
        "pivot", "tessera", "NOUN", "Tesela", (("NOUN", "Tarjeta, carné"),),
        f"« Tarjeta, carné » {_PERSON} (task 0.4): a membership or party card, not the mosaic's tile",
        "https://pl.wiktionary.org/wiki/tessera",
    ),
    Correction(
        "direct", "cancellato", "ADJ", "Celosiado", (("ADJ", "Cancelado, borrado"),),
        f"« Cancelado, borrado » {_PERSON} (task 0.4): *cancellare*'s participle, not « garnished with a lattice »",
        _IT.format("cancellato"),
    ),
    Correction(
        "pivot", "ognuno", "PRON", "Todo, cada, cada una", (("PRON", "Cada uno"),),
        f"« Cada uno » {_PERSON} (task 0.4): the pronoun, « each one »", "https://pl.wiktionary.org/wiki/każdy",
    ),
    # Round 2 (task 0.5, the owner's, 2026-10-25): the specialist's review of the gloss diff, every word a
    # source's. The cognate a rule cannot keep, or should open the row on (D2g).
    Correction(
        "inverted", "appartenenza", "NOUN", "Pertenencia", (("NOUN", "Pertenencia, membresía, afiliación"),),
        f"D2 opens it on the pivot's « Membresía »; the cognate leads — {_ROUND_2}", _ES.format("pertenencia"),
    ),
    Correction(
        "inverted", "cattura", "NOUN", "Captura", (("NOUN", "Captura, detención, arresto"),),
        f"D2 opens it on the pivot's « Detención »; the cognate leads — {_ROUND_2}", _ES.format("captura"),
    ),
    Correction(
        "direct", "evidentemente", "ADV", "Evidentemente",
        (("ADV", "Evidentemente, obviamente"), ("ADV", "Por lo visto, a ojos vistas")),
        f"D2 opens it on the pivot's « A ojos vistas »; the cognate leads — {_ROUND_2}", _EN.format("evidently"),
    ),
    Correction(
        "inverted", "percorso", "NOUN", "Recorrido", (("NOUN", "Ruta, camino, vía"), ("NOUN", "Recorrido")),
        f"D2 drops the Spanish tables' « Recorrido », no learned suffix's cognate (D2g) — {_ROUND_2}", _ES.format("recorrido"),
    ),
    Correction(
        "definitions", "incitare", "VERB", "Incitar", (("VERB", "Excitar, espolear, llevar a"), ("VERB", "Incitar")),
        f"D2 drops the section's « Incitar », a verb's cognate D2g does not read — {_ROUND_2}", _ES.format("incitare"),
    ),
    Correction(
        "direct", "incarico", "NOUN", "Encargo, incumbencia", (("NOUN", "Cargo, puesto"), ("NOUN", "Encargo")),
        f"D2 drops the Italian Wiktionary's « Encargo », no learned suffix's cognate (D2g) — {_ROUND_2}", _IT.format("incarico"),
    ),
    Correction(
        "pivot", "insufficienza", "NOUN", "Falta, carencia, cero",
        (("NOUN", "Falta, carencia"), ("NOUN", "Insuficiencia"), ("NOUN", "Superficialidad")),
        f"« cero », a Polish table's (lufa, the grade); D2g keeps « Insuficiencia » — {_ROUND_2}", "https://pl.wiktionary.org/wiki/lufa",
    ),
    Correction(
        "direct", "inaccessibile", "ADJ", "Inaccesible", (("ADJ", "Inaccesible, inasequible"),),
        f"D2 opens it on the pivot's « Inaccessible, inpenetrable », which D5c takes out; the cognate leads — {_ROUND_2}",
        _IT.format("inaccessibile"),
    ),
    Correction(
        "pivot", "autista", "NOUN", "Conductor, autista, chófer", (("NOUN", "Conductor, chófer"),),
        f"« autista » is Spanish for autistic: the English autist's table — {_ROUND_2}", _EN.format("autist"),
    ),
    # Rows the specialist read wrong or partly right, opened on a source's words.
    Correction(
        "definitions", "fermo", "ADJ", "Quieto, inmóvil", (("ADJ", "Quieto, parado, inmóvil"), ("ADJ", "Firme")),
        f"D2 opens it on « Firme »; still first, the Italian Wiktionary's « parado » — {_ROUND_2}", _ES.format("fermo"),
    ),
    Correction(
        "definitions", "chissà", "ADV", "Quién sabe", (("ADV", "Quién sabe"), ("ADV", "Tal vez")),
        f"D2 opens it on « Tal vez (posiblemente) » and the list — {_ROUND_2}", _ES.format("chissà"),
    ),
    Correction(
        "pivot", "riportare", "VERB", "Ir por, ir a por, conseguir", (("VERB", "Devolver, traer"), ("VERB", "Referir")),
        f"« Ir por… » is the English retrieve's; the section's « Devolver », the pivot's « traer », « referir » — {_ROUND_2}",
        _EN.format("retrieve"),
    ),
    Correction(
        "pivot", "galera", "NOUN", "Galera, cárcel, trullo", (("NOUN", "Cárcel, trullo"), ("NOUN", "Galera")),
        f"the prison first, the ship after — {_ROUND_2}", _EN.format("galley"),
    ),
    Correction(
        "pivot", "alimenti", "NOUN", "Manutención, alimentos, pensión alimenticia",
        (("NOUN", "Alimentos, comida"), ("NOUN", "Pensión alimenticia")),
        f"the English alimony's first; food first, the allowance after — {_ROUND_2}", _EN.format("alimony"),
    ),
    Correction(
        "pivot", "richiamare", "VERB", "Reprochar, reprender, amonestar",
        (("VERB", "Llamar nuevamente"), ("VERB", "Reprender, amonestar")),
        f"the section's « Llamar nuevamente » first, the rebuke after — {_ROUND_2}", _EN.format("rebuke"),
    ),
    Correction(
        "pivot", "sportello", "NOUN", "Puerta, panel, postigo", (("NOUN", "Ventanilla, taquilla"), ("NOUN", "Puerta")),
        f"the Italian Wiktionary's counter first, the door after — {_ROUND_2}", _EN.format("door"),
    ),
    Correction(
        "spanish section", "conto", "NOUN", "Cálculo", (("NOUN", "Cuenta"), ("NOUN", "Cálculo")),
        f"the bill, the account first: the pivot's « Cuenta » — {_ROUND_2}", _IT.format("cálculo"),
    ),
    Correction(
        "definitions", "strada", "NOUN", "Camino", (("NOUN", "Calle, carretera, camino"),),
        f"the street first: the Spanish tables' « calle, carretera » — {_ROUND_2}", _ES.format("strada"),
    ),
    Correction(
        "definitions", "ormai", "ADV", "Todavía, ya",
        (("ADV", "Ya"), ("ADV", "Por ahora (ahora, en el tiempo actual)"), ("ADV", "Casi")),
        f"« todavía », the opposite of *ormai* — {_ROUND_2}", _ES.format("ormai"),
    ),
    Correction(
        "pivot", "ricerca", "NOUN", "Búsqueda, investigación, pesquisa", (("NOUN", "Búsqueda"), ("NOUN", "Investigación")),
        f"the search and the research, two senses — {_ROUND_2}", _EN.format("search"),
    ),
    Correction(
        "pivot", "loro", "DET", "Su, sus, suyo",
        (("PRON", "Ellos, ellas"), ("PRON", "Ustedes"), ("DET", "Su, sus")),
        f"the possessive, MarkIT's commonest reading (34 against 15), after the pronoun — {_ROUND_2}", _EN.format("their"),
    ),
    Correction(
        "inverted", "via", "NOUN", "Camino, calle, cañón", (("NOUN", "Camino, calle"),),
        f"« cañón », another sense's word — {_ROUND_2}", _ES.format("cañón"),
    ),
)  # fmt: skip

# D11's `DI_FAMILY`: the contractions of `di` and the article, read whatever the sources give, each as
# a preposition; the reduction fails on a row whose headword is no ranked lemma. Four are written by a
# person (task 0.1); `del` and `dello` read the pivot's « Del ».
DiRow = collections.namedtuple("DiRow", "gloss reason")
DI_FAMILY = {
    "del": DiRow("Del", "the pivot's (`del`'s third sense): « Algo, un tanto…; Para » were other words' senses"),
    "dello": DiRow("Del", "the pivot's, as committed: kept so the family reads alike"),
    "della": DiRow("De la", f"{_PERSON} (task 0.1): the section describes it (« Contracción de la preposición di y el artículo la »)"),
    "dei": DiRow("De los", f"{_PERSON} (task 0.1): unglossed since change 27's split-translation filter"),
    "degli": DiRow("De los", f"{_PERSON} (task 0.1): unglossed since change 27's split-translation filter"),
    "delle": DiRow("De las", f"{_PERSON} (task 0.1): unglossed since change 27's split-translation filter"),
}

# Every gloss of it-es written by a person, by headword: the words no source gives (tasks 0.1, 0.2, 0.4;
# change 27's `ne`, in `FUNCTION_WORDS`). The pull request lists them; each is reported upstream.
BY_A_PERSON = {
    "ne": "De allí, de ahí",
    "a": "A",
    "te": "ti",
    "svezia": "Suecia",
    "della": "De la",
    "dei": "De los",
    "degli": "De los",
    "delle": "De las",
    "tessera": "Tarjeta, carné",
    "cancellato": "Cancelado, borrado",
    "ognuno": "Cada uno",
}

# The rows the reviewed tables write, which D2, D2f, D3 and D6a never move (D2e).
REVIEWED = frozenset(FIRST_RUN) | {row.headword for row in CORRECTIONS} | {row.headword.lower() for row in FUNCTION_WORDS}


def markit_by_word(counts):
    """MarkIT's counts (`reduce_italian_treebank.markit_pos_counts`) by word: `word → {UPOS: count}`, each
    word's parts of speech in the order the sections first read them."""
    out = {}
    for (word, upos), n in counts.items():
        out.setdefault(word, {})[upos] = n
    return out


def source_glosses(glosses, runs, steps, sources, ranks):
    """Each source's gloss of every ranked lemma, as `native_side` would write it were it the lemma's
    only source: `lemma → {source: (gloss, runs)}`, the section's definitions as `native_side` wrote
    them."""
    out = collections.defaultdict(dict)
    for lemma in steps["definitions"]:
        out[lemma]["definitions"] = (glosses[lemma], runs[lemma])
    for name, source, order in sources:
        for lemma in ranks:
            by_pos = source.get(lemma)
            if by_pos:
                gloss, gloss_runs = common.translation_gloss(by_pos, order, edition=EDITION)
                if gloss:
                    out[lemma][name] = (gloss, gloss_runs)
    return out


def _first_run(lemma, current, glossed):
    row = FIRST_RUN[lemma]
    theirs = senses_of(*glossed[row.source]) if row.source in glossed else []
    hit = [x for x in theirs if x[1] == row.text] or [x for x in senses_of(*current) if x[1] == row.text]
    if not hit:
        raise SystemExit(
            f"it-es: FIRST_RUN {lemma}: the {row.source} no longer writes « {row.text} »: read the row again (refine-lingua-it-es-glosses D12)"
        )
    rest = [x for x in senses_of(*current) if x[1] != row.text] if row.others else []
    return joined(grouped([hit[0]] + rest)[:MOST_SENSES])


def _corrects(row, glossed):
    """Whether a correction's source still writes the sense it answers."""
    if row.source not in glossed:
        return False
    return (row.pos, row.text) in senses_of(*glossed[row.source])


def refine_rows(glosses, runs, glossed_by, ranks, *, record, readings, zipf, pivot, counts, rules=RULES, stats=None):
    """The rows as change 28 reads them, in place (`glosses`, `runs`), in the design's order: for each
    ranked lemma each source glosses (`glossed_by`, `source_glosses`), D4, D2f, D9a, D6a, D2, D3b, D3a,
    D7a, D9b — a reviewed row (`REVIEWED`) never moved by D2f, D6a, D2 or D3 —, then `FIRST_RUN`; then
    `CORRECTIONS`, then `DI_FAMILY`. `record` is `refine_section`'s, `pivot` the pivot as read (its
    tables, for D2's support) and `counts` MarkIT's (`reduce_italian_treebank.markit_pos_counts`). The
    reviewed rows that fired on nothing, by name."""
    stats = stats if stats is not None else collections.Counter()
    by_word = markit_by_word(counts or {})
    reviewed = "reviewed rows" in rules
    held = (REVIEWED if reviewed else frozenset()) | {row.headword.lower() for row in FUNCTION_WORDS}
    first_runs = set()
    for lemma in sorted(glosses):
        if lemma not in ranks or lemma not in glossed_by:
            continue
        current = (glosses[lemma], runs[lemma])
        native = [s for _, s in senses_of(*current)]
        row = Row(lemma, glossed_by[lemma], pivot)
        passes = (
            ("feminine's own word", False, lambda c: feminine_s_own_word(row, c, record, readings)),
            ("wikidata's lone items", True, lambda c: lone_wikidata_items(row)),
            ("taxa", False, without_taxa),
            ("rare first words", True, lambda c: rare_first_word(row, c, zipf)),
            ("agreement", True, lambda c: agreement(row, c, zipf, rules)),
            ("foreign items", False, lambda c: without_foreign_items(c, native, zipf)),
            ("missing adverb or preposition", True, lambda c: missing_adverb_or_preposition(row, c, by_word)),
            ("markit order", True, lambda c: markit_order(lemma, c, counts or {})),
            ("long lists", False, lambda c: long_lists_cut(c, row, zipf) if "lists by use" in rules else long_lists_cut(c)),
            ("repeats", False, without_repeats),
        )
        for name, holds, rule in passes:
            if name not in rules or (holds and lemma in held):
                continue
            moved = rule(current)
            if moved and moved[0] and moved != current:
                stats[f"rows moved: {name}"] += 1
                current = moved
        if reviewed and lemma in FIRST_RUN:
            current = _first_run(lemma, current, glossed_by[lemma])
            first_runs.add(lemma)
        glosses[lemma], runs[lemma] = current
    unfired = []
    if not reviewed:
        return unfired
    unfired.extend(f"FIRST_RUN {lemma}" for lemma in FIRST_RUN if lemma not in first_runs)
    for row in CORRECTIONS:
        if not _corrects(row, glossed_by.get(row.headword, {})):
            unfired.append(f"CORRECTIONS {row.headword} ({row.source})")
            continue
        if row.corrected is None:
            glosses.pop(row.headword, None)
            runs.pop(row.headword, None)
        else:
            glosses[row.headword], runs[row.headword] = joined(list(row.corrected))
        stats["rows CORRECTIONS writes (D12)"] += 1
    for headword, di in DI_FAMILY.items():
        if headword not in ranks:
            raise SystemExit(f"it-es: DI_FAMILY {headword} is no ranked lemma: read the row again (refine-lingua-it-es-glosses D11)")
        glosses[headword], runs[headword] = di.gloss, [("ADP", 1)]
        stats["rows DI_FAMILY writes (D11)"] += 1
    return unfired
