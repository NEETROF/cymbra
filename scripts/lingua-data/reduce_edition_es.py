# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The Spanish Wiktionary's rules (eswiktionary), for the pairs glossed in Spanish
(generalise-lingua-gloss-reducer).

No committed pair reads it yet: en-es will (the language matrix's change 22). Its rules come from a
census of the English entries of kaikki's dump of the whole edition, the 2026-10-02 dump es-fr pins:
22,965 entries, 35,163 senses.

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

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does — tuning it never
re-pins en-fr or es-fr.
"""

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

# The notes the edition writes for its own readers: a sense link and « Véase también … ».
_NOTES = re.compile(
    # On the word, with the period before it when the link closes the sense (« Rey.₃. »).
    rf"(?<=[{_LOWER}])(?:\.(?={_LINK}(?:[.,;:]|$)))?{_LINK}"
    # After the period closing a word, in a sense that goes on: the period stays.
    rf"|(?<=[{_LOWER}]\.){_LINK}"
    # After a stray space (« café ₄ »), unless a preposition says the link is the sentence's word.
    rf"|(?<=[{_LOWER}]){_NOT_AFTER_A_PREPOSITION}\s+{_LINK}"
    r"|\s*[Vv]éase también\b[^.;]*\.?"
)

# A sense naming a letter: « Nombre de la letra u. », « Letra q. ».
_LETTER = re.compile(r"\bnombre de la letra\b|^letra [a-zñ]\.?$", re.IGNORECASE)

ES = common.Edition(
    code="es",
    form_of=_FORM_OF,
    notes=_NOTES,
    letter=_LETTER,
)
