# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The Spanish Wiktionary's rules (eswiktionary), for the pairs glossed in Spanish
(generalise-lingua-gloss-reducer).

en-es reads it (add-lingua-pack-en-es): its glosses are the Spanish Wiktionary's senses of its
English entries. Its rules come from a census of the English entries of kaikki's dump of the whole
edition, the 2026-10-02 dump es-fr pins: 22,965 entries, 35,163 senses.

- Pointers. Only 9.8 % of the senses are tagged `form-of`: most « Forma verbal » and « Forma
  sustantiva » entries are not, and say so in their wording — « Pasado simple del verbo (to) read. »,
  « Participio pasado del verbo (to) read. », « Tercera persona del singular … del verbo (to) be. »,
  « Grafía obsoleta de heart. » (543), « Variante de garnet. », « Comparativo de tall ». A wording
  counts only with a « de » or « del » after it, so « Femenino. » stays a meaning, as does « Forma
  coloquial para designar a un amigo ».
- Notes for its own readers: a sense links a word's numbered sense by a subscript (« Permitir,
  dejar₉. », « Propio o relacionado con el capital₆. »), a range of them (« Madrid₁₋₂. ») or two
  (« bottom₉ o ₁₀ »), taken out whole; 229 senses carry one. It goes after a lower-case letter, after
  the period closing the word (« Rey.₃. », the period with it when the sense ends there) and after a
  stray space (« café ₄ »), but never after a capital, a digit or a bracket, so a chemical formula
  keeps its digits (« C₄H₁₀ »). A subscript standing after a preposition is the sentence's own word
  — it names one of the entry's senses, « Cualquier parte similar a ₁ en un animal » (3 senses) —
  and stays, since the sense would not read without it. Also « Véase también … ».
- A letter's name: « Nombre de la letra Q. ».
- No dangling coordinator; 99.0 % of the senses open on a capital, so a gloss made of
  translation-table words does too.

- Notes to its readers, and meanings in their order (refine-lingua-en-es-glosses). The edition also
  writes its maintenance templates into a sense as the dump renders them (« ^([cita requerida]) »,
  10 senses of the section, « ^([definición imprecisa]) », 5), a disambiguation note (« [sentido del
  sustantivo] »), a reference to its numbered senses (« (definiciones [1,2]) »), its expansion
  notice (« Este lema en este idioma es ampliable. … »), and a usage note after the meaning, a
  sentence opening on one of a closed list (« Un, una. A veces se omite en la traducción. »): all
  taken out (D2), while a second sentence that carries the meaning stays (« Traducida como … »).
  `read_as_meanings`, a pre-pass en-es runs before the shared rules read the file, writes the
  senses the edition marks no longer used (`obsolete` 322 senses of the section, `outdated` 130, or
  raw-labelled « Arcaico », « obsoleta »…) after the other senses of their entry (D5), and every
  gloss in one Spanish typography (`typography`, D6): one ellipsis « … », straight double quotes —
  137 senses, beside « » five times and “ ” never — paired « », as the RAE advises. Measured on
  en-es's tables reduced from its 2026-10-08 snapshot (21,965 glossed lemmas): the notes and the
  usage notes change 15 rows (12 of the top 10,000) and 3 expressions, the order 41 (27), the
  first sense of 20 (12), the typography 39 (26) and 19 expressions; no lemma gains or loses a
  gloss through them.

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does — tuning it never
re-pins en-fr or es-fr.
"""

import json
import re

import reduce_common as common

# An untagged sense that only points at another word: a form, a spelling, a tense or a person,
# followed by « de » or « del » (the word it is a form of). Measured on the census: « out » is
# « Pasado de moda. », a meaning, among the 5 senses opening « Pasado de ».
_FORM_OF = re.compile(
    r"^(?:forma (?:del|de la|flexiva|verbal|sustantiva|adjetiva|plural|femenina|masculina|alternativa|"
    r"arcaica|obsoleta|antigua)|grafía|variante|plural|singular|femenino|masculino|participio|gerundio|"
    r"infinitivo|pasado(?! de moda\b)|presente|pretérito|futuro|condicional|imperativo|subjuntivo|"
    r"(?:primera|segunda|tercera) persona|[123]p|comparativo|superlativo|diminutivo|aumentativo)"
    r"\b[^.:;]*?\bdel?\b",
    re.IGNORECASE,
)

# A link to a word's numbered senses: one (« dejar₉ »), a range (« Madrid₁₋₂ ») or two (« bottom₉ o
# ₁₀ »).
_SENSE = r"[₀-₉]+(?:₋[₀-₉]+)?"
_LINK = rf"{_SENSE}(?:\s+[oy]\s+{_SENSE})?"
# Case-sensitive: a subscript after a capital, a digit or a bracket is a formula's (« C₄H₁₀ »).
_LOWER = "a-záéíóúüñ"
# The words a subscript standing on its own follows when it names one of the entry's own senses
# (« similar a ₁ », « como en ₁ »): the sense's own word, kept.
_PREPOSITIONS = ("a", "al", "ante", "como", "con", "contra", "de", "del", "desde", "en", "entre", "hacia")
_PREPOSITIONS += ("hasta", "para", "por", "que", "según", "sin", "sobre", "tras")
_NOT_AFTER_A_PREPOSITION = "".join(rf"(?<!\b{word})" for word in _PREPOSITIONS)

# A usage note's opening words, after the meaning's period (refine-lingua-en-es-glosses D2): a closed
# list measured on the section. « Traducida », « Nosotros », « Persona » open a second sentence that
# carries the meaning, and are not in it.
_USAGE_OPENERS = ("A veces", "Precediendo", "Usado", "Usada", "Usados", "Usadas", "Utilizado", "Utilizada")
_USAGE_OPENERS += ("Utilizados", "Utilizadas", "Empleado", "Empleada", "Se usa", "Se dice", "Se emplea")
_USAGE_OPENERS += ("Se utiliza", "A no confundir", "Compárese")

# The notes the edition writes for its own readers: a sense link and « Véase también … »; and
# (refine-lingua-en-es-glosses D2) its maintenance templates, its disambiguation notes, its
# references to its numbered senses, its expansion notice and a usage note after the meaning.
_NOTES = re.compile(
    # On the word, with the period before it when the link closes the sense (« Rey.₃. »).
    rf"(?<=[{_LOWER}])(?:\.(?={_LINK}(?:[.,;:]|$)))?{_LINK}"
    # After the period closing a word, in a sense that goes on: the period stays.
    rf"|(?<=[{_LOWER}]\.){_LINK}"
    # After a stray space (« café ₄ »), unless a preposition says the link is the sentence's word.
    rf"|(?<=[{_LOWER}]){_NOT_AFTER_A_PREPOSITION}\s+{_LINK}"
    r"|\s*[Vv]éase también\b[^.;]*\.?"
    # A maintenance template as the dump renders it: « ^([cita requerida]) », « ^([definición
    # imprecisa]) ».
    r"|\^\(\[[^\]]*\]\)"
    # A disambiguation note: « Hacer un favor [sentido del sustantivo] para ».
    r"|\s*\[sentido de[^\]]*\]"
    # A reference to numbered senses: « Hardcore (definiciones [1,2]) ».
    r"|\s*\((?:definici[oó]n|definiciones|acepci[oó]n|acepciones) \[[\d,\s–-]+\]\)"
    # The expansion notice, to the sense's end: « Este lema en este idioma es ampliable. Retira … ».
    r"|\s*Este lema en este idioma es ampliable\..*$"
    # A usage note after the meaning, to the sense's end: « Un, una. A veces se omite … ».
    r"|(?<=[^\s.])\.\s+(?:" + "|".join(_USAGE_OPENERS) + r")\b.*$"
)

# A sense naming a letter: « Nombre de la letra u. », « Letra q. ».
_LETTER = re.compile(r"\bnombre de la letra\b|^letra [a-zñ]\.?$", re.IGNORECASE)

ES = common.Edition(
    code="es",
    form_of=_FORM_OF,
    notes=_NOTES,
    letter=_LETTER,
)


# — Meanings in their order, in one typography (refine-lingua-en-es-glosses) —
#
# The edition often opens an entry on its oldest sense (« go »'s « Andar, marchar, caminar », tagged
# obsolete, before « Ir »), and the round-robin takes each entry's first senses; it writes an ellipsis
# as three dots or more, and quotes as straight double quotes. `read_as_meanings` rewrites the senses
# before the shared rules read them; it is the Spanish edition's, so it re-pins the pairs glossed in
# Spanish alone.

# The labels of a sense the edition marks no longer used (D5): kaikki's tags, and the raw labels it
# leaves as written. « raro » is not one: a rare sense keeps its place.
_NO_LONGER_USED = frozenset({"obsolete", "outdated", "Arcaico", "arcaico", "obsoleta", "Obsoleto", "obsoleto"})

# Three dots or more (D6): between two words, spaced on both sides; anywhere else, as written.
_ELLIPSIS_BETWEEN = re.compile(r"(?<=\w)\s*\.{3,}\s*(?=\w)")
_ELLIPSIS = re.compile(r"\.{3,}")


def typography(text):
    """`text` in one Spanish typography (D6): three dots or more written « … », spaced on both sides
    between two words (« (neither … nor) Ni ») and as written elsewhere (« hubo…? »); an even number
    of straight double quotes paired « » in order, with no space inside them (« it («ello») »), as the
    RAE advises; an odd number kept, as are single quotes — they are also the apostrophes of the
    English words a sense names (« Contracción de 'that is' »). Nothing else of the text changes."""
    text = _ELLIPSIS.sub("…", _ELLIPSIS_BETWEEN.sub(" … ", text))
    quotes = text.count('"')
    if quotes and quotes % 2 == 0:
        parts = text.split('"')
        text = parts[0] + "".join(f"«{part.strip()}»" if i % 2 == 0 else part for i, part in enumerate(parts[1:]))
    return text


def _texts(glosses):
    """Whether a sense's `glosses` is what kaikki writes: a non-empty list of strings."""
    return isinstance(glosses, list) and bool(glosses) and all(isinstance(g, str) for g in glosses)


def _labels(sense):
    """A sense's labels: its tags and its raw tags, as strings."""
    return {
        label
        for field in ("tags", "raw_tags")
        if isinstance(sense.get(field), list)
        for label in sense[field]
        if isinstance(label, str)
    }


def _no_longer_used(sense):
    return isinstance(sense, dict) and not _NO_LONGER_USED.isdisjoint(_labels(sense))


def _in_order(senses):
    """An entry's senses, the ones the edition marks no longer used after the others, each group in
    its order (D5) — all of them as written when every sense is so marked — each gloss in the
    edition's typography (D6). A sense that is not kaikki's shape keeps its place among the others."""
    current = [sense for sense in senses if not _no_longer_used(sense)]
    if current and len(current) < len(senses):
        senses = current + [sense for sense in senses if _no_longer_used(sense)]
    return [
        {**sense, "glosses": [typography(g) for g in sense["glosses"]]}
        if isinstance(sense, dict) and _texts(sense.get("glosses"))
        else sense
        for sense in senses
    ]


def rewrite_entries(src, dst, read):
    """`src`'s entries, each as `read(entry)` gives it, written to `dst`: an entry `read` returns
    unchanged is written as its line was, one it returns as None is left out, and a line that is no
    JSON object is written as it is — the shared rules decide. `dst`."""
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if isinstance(entry, dict):
                rewritten = read(entry)
                if rewritten is None:
                    continue
                if rewritten != entry:
                    line = json.dumps(rewritten, ensure_ascii=False) + "\n"
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def read_as_meanings(src, dst):
    """The edition's entries with their senses read as meanings, in their order, written to `dst` —
    a pre-pass a reducer runs before the shared rules read the file (refine-lingua-en-es-glosses
    D1): the senses tagged `obsolete` or `outdated`, or raw-tagged « Arcaico », « arcaico »,
    « obsoleta », « Obsoleto » or « obsoleto », written after the other senses of their entry in
    their order, an entry whose every sense is so marked kept as written (D5); every gloss in the
    edition's typography (`typography`, D6). Nothing is left out. An entry it does not change, and a
    line it cannot read, are written as they are."""

    def read(entry):
        senses = entry.get("senses")
        return {**entry, "senses": _in_order(senses)} if isinstance(senses, list) else entry

    return rewrite_entries(src, dst, read)
