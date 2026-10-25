# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the it-es reducer (add-lingua-pack-it-es), its pre-pass (`reduce_italian_es.py`) and the
Spanish table rules copied from fr-es (`reduce_spanish_tables.py`).

The entries are recorded from the files `pack_sources.derive` writes from the dumps served on 2026-10-25
(it-es's local update; design D8): the Spanish Wiktionary's Italian section (`kaikki-es-Italiano.jsonl`,
sha256 e9443572…) and its Spanish entries' Italian translations (`kaikki-es-traductions-it.jsonl`), the
Italian Wiktionary's Spanish translations and Spanish section (`kaikki-it-traductions-es.jsonl`,
`kaikki-it-Spagnolo.jsonl`), Wikidata's lexeme pairs (`wikidata-lexemes-it-es.jsonl`) and the seven
pivot files, cut down to the fields the rules read. A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import pack_sources as ps  # noqa: E402
import reduce_common as common  # noqa: E402
import reduce_edition_es as spanish  # noqa: E402
import reduce_italian_acronyms as acronyms  # noqa: E402
import reduce_italian_es as italian  # noqa: E402
import reduce_spanish_tables as tables  # noqa: E402
import test_reduce_fr_es as fr_es_cases  # noqa: E402 — fr-es's own cases, which the copied rules must hold


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(f"reduce_{pair.replace('-', '_')}_it_es", os.path.join(_HERE, f"reduce-{pair}.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


it_es = _reducer("it-es")
it_fr = _reducer("it-fr")
fr_es = fr_es_cases.fr_es


def sense(gloss, tags=(), form_of=None):
    out = {"glosses": [gloss]}
    if tags:
        out["tags"] = list(tags)
    if form_of is not None:
        out["form_of"] = [{"word": w} for w in form_of]
    return out


def entry(word, pos, *senses, tags=()):
    out = {"word": word, "pos": pos, "senses": list(senses)}
    if tags:
        out["tags"] = list(tags)
    return out


# — The Spanish Wiktionary's Italian section (kaikki-es-Italiano.jsonl), recorded —

CASA = entry("casa", "noun", sense("Casa."), sense("Tienda."), tags=["feminine"])
TU = entry(
    "tu",
    "pron",
    sense("Pronombre personal nominativo de 2 persona del singular, equivale al tú del español.", ["colloquial"]),
    tags=["personal"],
)
VEDERE = entry("vedere", "verb", sense("ver."), tags=["transitive"])
VERO_ADJ = entry("vero", "adj", sense("Vero²₁, verdadero, cierto."), sense("Vero²₂, genuino, auténtico."))
VERO_NOUN = entry("vero", "noun", sense("Verdad."), tags=["masculine"])
QUALE_PREP = entry("quale", "prep", sense("Como_(1, 5)."))
CAZZO_INTJ = entry(
    "cazzo",
    "intj",
    sense('Se usa para mostrar desagrado o enfatizar el aspecto negativo de algo. "Coño"₆, "carajo"₃, "mierda"₄.'),
)
SETTE_CARDINAL = entry("sette", "num", sense("Siete."), tags=["cardinal"])
SETTE_FORM = entry(
    "sette",
    "noun",
    sense('Forma del plural de setta: "sectas".', ["form-of"], form_of=["setta"]),
    tags=["form-of", "substantive"],
)
STA = entry("sta", "adj", sense("Forma del femenino de sto: «esta».", ["form-of"], form_of=["sto"]), tags=["adjectival", "form-of"])
AVERE_AUXILIARY = entry(
    "avere",
    "verb",
    sense("Verbo auxiliar de verbos transitivos en tiempos compuestos."),
    sense("Verbo auxiliar de algunos verbos intransitivos en tiempos compuestos."),
    tags=["auxiliary"],
)
AVERE = entry("avere", "verb", sense("Haber."), sense("Tener."), tags=["transitive"])
AVER = entry("aver", "verb", sense("Variante de avere."), tags=["auxiliary"])
SONO_NOUN = entry("sono", "noun", sense("Variante de suono."), tags=["masculine"])
SONO_VERB = entry(
    "sono",
    "verb",
    sense("Primera persona del singular (io) del presente de indicativo de essere.", ["form-of"], form_of=["essere"]),
    sense("Tercera persona del plural (esse, essi) del presente de indicativo de essere.", ["form-of"], form_of=["essere"]),
    tags=["form-of"],
)
SUONO = entry("suono", "noun", sense("Sonido."), tags=["masculine"])  # made up: the section has no `suono`
ESSERE = entry("essere", "verb", sense("Ser."), sense("Estar (algo o alguien en un determinado estado o condición)."))
SECONDO_NAME = entry("Secondo", "name", sense("Apellido"))
SECONDO_NUM = entry("secondo", "num", sense("Segundo."), tags=["ordinal"])
SECONDO_PREP = entry("secondo", "prep", sense("Según."))
CLOAKED_IT_NAME = entry("Segreto", "name", sense("Apellido."))  # made up: a name's note on a word only a table glosses
STATI_UNITI = entry("Stati Uniti", "name", sense("Estados Unidos."))
AMERICA = entry("America", "name", sense("América."), tags=["feminine"])

SECTION = [
    CASA, TU, VEDERE, VERO_ADJ, VERO_NOUN, QUALE_PREP, CAZZO_INTJ, SETTE_CARDINAL, SETTE_FORM, STA, AVERE_AUXILIARY,
    AVERE, AVER, SONO_NOUN, SONO_VERB, SUONO, ESSERE, SECONDO_NAME, SECONDO_NUM, SECONDO_PREP, CLOAKED_IT_NAME,
    STATI_UNITI, AMERICA,
]  # fmt: skip

# — The Italian Wiktionary's Spanish translations (kaikki-it-traductions-es.jsonl), recorded —

CASA_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"sense": "edificio destinato all'abitazione", "word": "casa"},
        {"sense": "domicilio", "word": "hogar"},
        {"sense": "(astrologia) ognuna delle dodici suddivisioni del cielo", "word": "casa"},
    ],
    "word": "casa",
}
CLOAKED_IT_TRANSLATED = [
    {"pos": "adj", "translations": [{"word": "secreto"}, {"word": "oculto"}], "word": "segreto"},
    {"pos": "noun", "translations": [{"word": "secreto"}, {"word": "oculto"}], "word": "segreto"},
]
ACCADUTO_TRANSLATED = [
    {"pos": pos, "translations": [{"sense": "sostantivo, fatto lieto o cattivo", "word": "hecho"}], "word": "accaduto"}
    for pos in ("adj", "noun", "verb")
]
CORSO_TRANSLATED = [{"pos": pos, "translations": [{"sense": "lingua", "word": "corso"}], "word": "corso"} for pos in ("adj", "noun")]
CREDO_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"sense": "formula liturgica cristiana", "word": "credo"},
        {"sense": "insieme delle credenze, concetti e principi di una religione", "word": "credo"},
        {"sense": "insieme delle credenze, concetti e principi di una religione", "word": "fe"},
    ],
    "word": "credo",
}
ABBONAMENTO_TRANSLATED = {
    "pos": "noun",
    "translations": [{"sense": "pagamento in anticipo e scontato di un servizio periodico", "word": "suscripciòn"}],
    "word": "abbonamento",
}
QUALCUNO_TRANSLATED = {"pos": "pron", "translations": [{"word": "alguien"}, {"word": "°"}], "word": "qualcuno"}
D_ACCORDO_TRANSLATED = {"pos": "adv", "translations": [{"word": "de acuerdo"}], "word": "d’accordo"}  # made up
UN_PO_TRANSLATED = {"pos": "adv", "translations": [{"word": "un poco"}], "word": "un po'"}  # made up

DIRECT = [CASA_TRANSLATED, *CLOAKED_IT_TRANSLATED, *ACCADUTO_TRANSLATED, *CORSO_TRANSLATED, CREDO_TRANSLATED,
          ABBONAMENTO_TRANSLATED, QUALCUNO_TRANSLATED, D_ACCORDO_TRANSLATED, UN_PO_TRANSLATED]  # fmt: skip

# — The Spanish Wiktionary's Spanish entries' Italian translations (kaikki-es-traductions-it.jsonl), recorded —

MANZANO_LISTS = {"pos": "noun", "translations": [{"word": "it"}, {"word": "melo"}], "word": "manzano"}
CLOAKED_ES_LISTS = {"pos": "noun", "translations": [{"word": "segreto"}], "word": "secreto"}
CAMBIO_LISTS = {
    "pos": "noun",
    "translations": [{"word": "cambio"}, {"word": "cambiamento"}, {"word": "scambio"}, {"word": "corso"}],
    "word": "cambio",
}
INVERTED = [MANZANO_LISTS, CLOAKED_ES_LISTS, CAMBIO_LISTS]

# — Wikidata's lexeme pairs (wikidata-lexemes-it-es.jsonl), recorded —

ESSERE_LINK = {
    "by": "translation",
    "es": {"category": "Q24905", "lemma": "ser", "lexeme": "L5140", "sense": "L5140-S2"},
    "it": {"category": "Q24905", "lemma": "essere", "lexeme": "L5139", "sense": "L5139-S4"},
}
APE_GLOSS = {"by": "gloss", "es": {"gloss": "abeja"}, "it": {"category": "Q1084", "lemma": "ape", "lexeme": "L640927", "sense": "L640927-S1"}}
FILA_ITEM = {
    "by": "item",
    "es": {"category": "Q1084", "lemma": "cola", "lexeme": "L306920", "sense": "L306920-S6"},
    "it": {"category": "Q1084", "lemma": "fila", "lexeme": "L1145631", "sense": "L1145631-S1"},
    "item": "Q1783115",
}
ROMA_ITEM = {  # made up: a proper noun's lexemes
    "by": "item",
    "es": {"category": "Q147276", "lemma": "Roma", "lexeme": "L1", "sense": "L1-S1"},
    "it": {"category": "Q147276", "lemma": "Roma", "lexeme": "L2", "sense": "L2-S1"},
    "item": "Q220",
}
WIKIDATA = [ESSERE_LINK, APE_GLOSS, FILA_ITEM, ROMA_ITEM]

# — The Italian Wiktionary's Spanish section (kaikki-it-Spagnolo.jsonl), recorded —

ANALOGIA = {"word": "analogia", "pos": "noun", "senses": [{"glosses": ["analogia, vedi italiano"]}]}
GUERRA = {"word": "guerra", "pos": "noun", "senses": [{"glosses": ["vedi italiano"]}]}
DERIVA = {"word": "deriva", "pos": "noun", "senses": [{"glosses": ["definizione mancante; se vuoi, aggiungila tu"]}]}
SEGURO = {
    "word": "seguro",
    "pos": "adj",
    "senses": [{"glosses": ["sicuro"]}, {"glosses": ["sicuro, certo"]}, {"glosses": ["(di azienda) solido, sicuro"]}],
}
CHICA = {"word": "chica", "pos": "noun", "senses": [{"glosses": ["femminile di chico, ragazza"]}]}
SPANISH_SECTION = [ANALOGIA, GUERRA, DERIVA, SEGURO, CHICA]

# — The pivot (kaikki-<edition>-pivot-it-es.jsonl) —

BUYER = {  # recorded, English
    "by": "sense",
    "es": ["cliente", "clienta", "comprador", "compradora", "mercante"],
    "it": ["cliente", "compratore", "compratrice", "acquirente"],
    "pos": "noun",
    "sense": "person who makes purchases",
    "several_senses": True,
    "word": "buyer",
}
ACHETEUR = {  # made up, French: a second table giving « comprador » for `acquirente`
    "by": "sense",
    "es": ["comprador"],
    "it": ["acquirente"],
    "pos": "noun",
    "sense": "Celui qui achète",
    "several_senses": True,
    "word": "acheteur",
}
EN_TO_SORT = {  # recorded, French: a table left to sort
    "by": "sense",
    "es": ["en", "en cuanto a", "sobre", "dentro de", "a", "hacia"],
    "it": ["a", "dentro"],
    "pos": "prep",
    "sense": "Traductions à trier suivant le sens",
    "several_senses": True,
    "word": "en",
}
ADAPTER = {  # recorded, English
    "by": "sense",
    "es": ["adaptador"],
    "it": ["adattatore"],
    "pos": "noun",
    "sense": "device to allow compatibility (in general)",
    "several_senses": True,
    "word": "adapter",
}
ADAPTER_ADJ = {  # made up: the same Spanish word under another part of speech
    "by": "sense",
    "es": ["adaptador"],
    "it": ["adattatore"],
    "pos": "adj",
    "sense": "that adapts",
    "several_senses": True,
    "word": "adapting",
}
PARIS = {"by": "entry", "es": ["París"], "it": ["Parigi"], "pos": "name", "several_senses": False, "word": "Paris"}  # made up
LONDRA = {"by": "sense", "es": ["Londres"], "it": ["londra"], "pos": "noun", "sense": "a city", "several_senses": True, "word": "london"}  # made up
WAGON = {"by": "entry", "es": ["vagón"], "it": ["vagone"], "pos": "noun", "several_senses": True, "word": "wagon"}  # made up
DWORZEC = {  # recorded, Polish: a translation split into its words
    "by": "sense_index",
    "es": ["estación", "de", "autobuses"],
    "it": ["stazione", "degli", "autobus"],
    "pos": "noun",
    "sense_index": "1.1",
    "several_senses": False,
    "word": "dworzec autobusowy",
}
ZAINO = {"by": "sense", "es": ["mochìla"], "it": ["zaino"], "pos": "noun", "sense": "bag", "several_senses": False, "word": "backpack"}  # made up
PIVOT = {"en": [BUYER, ADAPTER, ADAPTER_ADJ, PARIS, LONDRA, WAGON, ZAINO], "fr": [ACHETEUR, EN_TO_SORT], "pl": [DWORZEC]}

# — UD Italian-MarkIT's training and development sections (it_markit-ud-{train,dev}.conllu), made up:
# ten adverbs of `dentro` against one preposition (change 28's D3) —

MARKIT = (
    "".join(f"{i}\tdentro\tdentro\tADV\t_\t_\t0\troot\t_\t_\n\n" for i in range(1, 11)),
    "1\tdentro\tdentro\tADP\t_\t_\t0\troot\t_\t_\n\n",
)

# The ranked lemmas of the made-up studied tables, commonest first.
RANKED = [
    "di", "casa", "tu", "vedere", "vero", "quale", "cazzo", "sette", "sta", "avere", "aver", "sono", "essere",
    "secondo", "segreto", "accaduto", "corso", "credo", "abbonamento", "qualcuno", "melo", "it", "ape", "fila",
    "roma", "analogia", "guerra", "deriva", "sicuro", "certo", "solido", "ragazza", "acquirente", "compratore",
    "adattatore", "parigi", "londra", "vagone", "stazione", "degli", "autobus", "zaino", "dentro", "suono", "cambio",
    "del", "dello", "della", "dei", "delle",
]  # fmt: skip
# Italian's readings of these words, as tables/it/grammar.tsv commits them (made up).
READINGS = "accaduto\taccaduto\tVERB|VerbForm=Part\t-\nadattatore\tadattatore\tNOUN|Gender=Masc\t-\n"


def zipf(word, language):
    """wordfreq, made up for the cases: Italian and Spanish frequencies."""
    italian_, spanish_ = {
        "corso": (5.4, 3.2),
        "credo": (5.0, 2.9),
        "casa": (6.0, 6.0),
        "comprador": (0.0, 3.5),
        "cliente": (4.0, 4.6),
        "clienta": (0.0, 2.0),
        "compradora": (0.0, 1.5),
        "mercante": (0.0, 2.6),
        "hogar": (0.0, 4.7),
        "ser": (0.0, 6.9),
    }.get(word, (0.0, 0.0))
    return italian_ if language == "it" else spanish_


class Fixture(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def jsonl(self, *entries, name="section.jsonl"):
        path = self.dir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        return str(path)

    def studied(self, ranked=RANKED, extra_forms=()):
        studied = self.dir / "it"
        studied.mkdir(exist_ok=True)
        forms = {w: w for w in ranked} | dict(extra_forms) | acronyms.acronym_forms()
        (studied / "forms.tsv").write_text("".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())), encoding="utf-8")
        (studied / "freq.tsv").write_text("".join(f"{w}\t{i}\n" for i, w in enumerate(ranked, start=1)), encoding="utf-8")
        (studied / "grammar.tsv").write_text(READINGS, encoding="utf-8")
        return studied

    def read(self, *entries, tabled=frozenset(), rules=italian.RULES):
        src = self.jsonl(*entries)
        dst, stats = italian.read_section(src, str(self.dir / "read.jsonl"), tabled=tabled, rules=rules)
        return [json.loads(line) for line in Path(dst).read_text(encoding="utf-8").splitlines()], stats

    def work(self, section=SECTION, direct=DIRECT, inverted=INVERTED, wikidata=WIKIDATA, spanish_section=SPANISH_SECTION, pivot=PIVOT, markit=MARKIT):
        work = self.dir / "work"
        work.mkdir(exist_ok=True)
        self.jsonl(*section, name=f"work/{it_es.SECTION}")
        self.jsonl(*direct, name=f"work/{it_es.DIRECT}")
        self.jsonl(*inverted, name=f"work/{it_es.INVERTED}")
        self.jsonl(*wikidata, name=f"work/{it_es.WIKIDATA}")
        self.jsonl(*spanish_section, name=f"work/{it_es.SPANISH_SECTION}")
        for edition, name in it_es.PIVOT.items():
            self.jsonl(*pivot.get(edition, ()), name=f"work/{name}")
        for name, text in zip(it_es.MARKIT, markit):
            (work / name).write_text(text, encoding="utf-8")
        return work

    def reduce(self, **sources):
        work = self.work(**sources)
        studied = self.studied()
        lines = []
        summary = it_es.reduce(str(work), str(studied), 60000, "test", zipf=zipf, report=lines.append)
        self.report = "\n".join(lines)
        self.summary = summary

        def table(name):
            rows = {}
            for line in (work / name).read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition("\t")
                rows[key] = value
            return rows

        return work, table("gloss.tsv"), table("senses.tsv"), table("mwe.tsv")


class TheSourcesInOrder(Fixture):
    """D2: the definitions, the two Wiktionaries' tables, Wikidata's links and glosses, then its items,
    the Italian Wiktionary's Spanish section read backwards, then the pivot."""

    def test_spec_scenario_a_direct_source_first(self):
        # `casa`: the section's definition first, though the direct table and the pivot also gloss it
        # (made up: a pivot table pairing `casa` with « vivienda »).
        home = {"by": "sense", "es": ["vivienda"], "it": ["casa"], "pos": "noun", "sense": "a home", "several_senses": True, "word": "home"}
        _, glosses, _, _ = self.reduce(pivot={**PIVOT, "en": [*PIVOT["en"], home]})
        self.assertEqual(glosses["casa"], "Casa; Tienda")
        measured = json.loads((self.dir / "work" / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(
            list(measured["by source"]), ["definitions", "direct", "inverted", "wikidata links", "wikidata items", "spanish section", "pivot"]
        )

    def test_each_source_s_place(self):
        _, glosses, senses, _ = self.reduce()
        steps = self.summary["glossed by source"]
        # The direct table, then the inverted one, Wikidata's links and glosses, its items, the Spanish
        # section read backwards, the pivot — each for what the sources before it left out.
        self.assertEqual(glosses["segreto"], "Secreto, oculto")
        self.assertEqual(glosses["melo"], "Manzano")
        self.assertEqual(glosses["ape"], "Abeja")
        self.assertEqual(glosses["fila"], "Cola")
        self.assertEqual(glosses["sicuro"], "Seguro")
        self.assertEqual(glosses["acquirente"], "Comprador, cliente, mercante")
        self.assertEqual(senses["acquirente"], "NOUN:1")
        self.assertEqual(steps["direct"], 4, "segreto, accaduto, credo, qualcuno")
        self.assertEqual(steps["wikidata links"], 1, "ape; essere is glossed by its definitions")
        self.assertEqual(steps["wikidata items"], 1)
        self.assertIn("wikidata items", self.report)
        # A table that lists a word Spanish-ward after an earlier source glossed it adds nothing.
        self.assertEqual(glosses["essere"], "Ser; Estar (algo o alguien en un determinado estado o condición)")

    def test_the_native_side_step_by_step_is_native_tables(self):
        # With the sources in D2's order as one list of fallbacks, the shared native side gives the same
        # tables: the steps only keep each lemma's source apart.
        section = self.jsonl(*SECTION)
        ranks = {w: i for i, w in enumerate(RANKED, start=1)}
        direct = it_es.read_translated(self.jsonl(*DIRECT, name="d.jsonl"), str(self.dir / "d-read.jsonl"), inverted=False, readings={}, zipf=zipf)
        inverted = it_es.read_translated(self.jsonl(*INVERTED, name="i.jsonl"), str(self.dir / "i-read.jsonl"), inverted=True, readings={}, zipf=zipf)
        order = common.by_native_frequency(lambda w: zipf(w, "es"))
        sources = [("direct", direct, list), ("inverted", inverted, order)]
        glosses, runs, expressions, steps = it_es.native_side(section, ranks, sources)
        want = common.native_tables(section, ranks, studied=it_es.IT, edition=spanish.ES, fallbacks=[(direct, list), (inverted, order)])
        self.assertEqual((glosses, runs, expressions), want[:3])
        self.assertEqual(len(steps["definitions"]), want[3])


class ThePivot(Fixture):
    """D3: the pivot's groups, filters and order."""

    def pivot(self, *records, edition="en"):
        pivot = it_es.Pivot()
        pivot.read(self.jsonl(*records, name=f"pivot-{edition}.jsonl"), edition)
        return pivot

    def test_spec_scenario_a_pivot_ordered_by_support(self):
        # `acquirente`: « comprador » two tables give comes before « cliente », which one gives, though
        # « cliente » is commoner in Spanish; then Spanish frequency, then the alphabet.
        pivot = it_es.Pivot()
        pivot.read(self.jsonl(BUYER, name="en.jsonl"), "en")
        pivot.read(self.jsonl(ACHETEUR, name="fr.jsonl"), "fr")
        table = pivot.table(lambda w: zipf(w, "es"))
        self.assertEqual(table["acquirente"]["NOUN"][:3], ["comprador", "cliente", "mercante"])
        self.assertEqual(pivot.provenance("acquirente", "comprador"), "en: buyer (noun), « person who makes purchases »")
        glossed = common.fallback_glosses({"acquirente"}, {}, [(table, list)], edition=spanish.ES)
        self.assertEqual(glossed, {"acquirente": ("Comprador, cliente, mercante", [("NOUN", 1)])})

    def test_spec_scenario_a_name_kept_off(self):
        # A name's entry, a capitalised Italian or Spanish word, a « to sort » table, an entry of several
        # senses read whole: none glosses.
        table = self.pivot(PARIS, LONDRA, WAGON, EN_TO_SORT).table(lambda w: 0.0)
        self.assertEqual(table, {})
        for spelling in ("Traductions à trier suivant le sens", "À trier", "Traductions à trier", "à trier", "A trier", "Sens à trier"):
            self.assertFalse(it_es.pivot_group_kept({**ADAPTER, "sense": spelling}, "fr"), spelling)
        self.assertFalse(it_es.pivot_group_kept({**ADAPTER, "sense": "Vertalingen: na te kijken en onderverdelen per verklaring"}, "nl"))
        self.assertTrue(it_es.pivot_group_kept({**ADAPTER, "sense": "onprettig om naar te kijken, niet mooi"}, "nl"), "a sense, not a note")
        # A one-sense entry read whole glosses; a German noun's capital is its spelling; an acronym of at
        # most six letters stays.
        self.assertTrue(it_es.pivot_group_kept({**WAGON, "several_senses": False}, "en"))
        self.assertTrue(it_es.pivot_group_kept({**ADAPTER, "word": "Adapter"}, "de"))
        self.assertFalse(it_es.pivot_group_kept({**ADAPTER, "word": "Adapter"}, "en"))
        self.assertFalse(it_es.pivot_group_kept({**ADAPTER, "word": "a"}, "en"), "a one-letter headword")
        self.assertTrue(it_es.pivot_word_kept("IVA"))
        self.assertFalse(it_es.pivot_word_kept("UNESCOS"), "an acronym of seven letters")
        self.assertFalse(it_es.spanish_word_kept("x" * 61))

    def test_spec_scenario_a_translation_split_into_its_words(self):
        # The Polish extract lists « estación de autobuses » word by word: the group glosses nothing, and
        # `degli` takes no « De ».
        self.assertTrue(it_es.split_translation(DWORZEC))
        self.assertEqual(self.pivot(DWORZEC, edition="pl").table(lambda w: 0.0), {})
        # A function word's entry lists such words as translations of their own.
        self.assertFalse(it_es.split_translation({**EN_TO_SORT, "sense": "in"}))

    def test_a_word_repeated_across_parts_of_speech_is_listed_once(self):
        table = self.pivot(ADAPTER, ADAPTER_ADJ).table(lambda w: 0.0)
        self.assertEqual(table["adattatore"], {"NOUN": ["adaptador"], "ADJ": ["adaptador"]})
        self.assertEqual(it_es.table_rules(table, {"adattatore": {"NOUN"}}, zipf)["adattatore"], {"NOUN": ["adaptador"]})

    def test_a_spanish_word_as_the_tables_write_it(self):
        # The review's cases, recorded from the pivot's tables: several words in one item, labels, a
        # parenthesis narrowing it, a bracket or a quotation mark left open, a separator at an end.
        cases = {
            "por ahí “": ["por ahí"],
            "automático , ( automáticamente": ["automático", "automáticamente"],
            "inmigrante /": ["inmigrante"],
            "respuesta ) negativa": ["respuesta negativa"],
            "por encima (de)": ["por encima"],
            "duricia (disused)": ["duricia"],
            "Bulle (ugs.)": ["Bulle"],
            "adondequiera (whitherever)": ["adondequiera"],
            "¡ay, Dios mío!": ["¡ay, Dios mío!"],
            "1-metil-2,4,6-trinitrobenceno": ["1-metil-2,4,6-trinitrobenceno"],
            "(": [],
        }
        for item, words in cases.items():
            self.assertEqual(it_es.pivot_spanish_words(item), words, item)
        presso = {**ADAPTER, "it": ["presso"], "es": ["por ahí “", "en casa de"], "pos": "adv", "word": "nearby"}
        self.assertEqual(self.pivot(presso).table(lambda w: 0.0), {"presso": {"ADV": ["en casa de", "por ahí"]}})

    def test_a_grave_accented_word_glosses_nothing(self):
        self.assertFalse(it_es.spanish_word_kept("mochìla"))
        self.assertEqual(self.pivot(ZAINO).table(lambda w: 0.0), {})

    def test_the_pivot_supplies_no_expression(self):
        phrase = {**ADAPTER, "it": ["presa multipla"], "es": ["regleta"]}
        self.assertEqual(self.pivot(phrase).table(lambda w: 0.0), {})
        _, _, _, mwe = self.reduce(pivot={"en": [phrase]})
        self.assertNotIn("presa multipla", mwe)


class TheSectionAsRead(Fixture):
    """D4: it-es's pre-pass over the section."""

    def test_spec_scenario_a_pointer_lends_its_word_s_gloss(self):
        _, glosses, _, _ = self.reduce()
        self.assertEqual(glosses["aver"], "Haber; Tener")
        # `sono`'s verb senses are tagged forms of `essere`: its noun's « Variante de suono » lends nothing.
        self.assertNotIn("Sonido", glosses["sono"])
        read, stats = self.read(AVER, SONO_NOUN, SONO_VERB, SUONO, AVERE)
        self.assertEqual(read[0]["senses"][0]["form_of"], [{"word": "avere"}])
        self.assertNotIn("form_of", read[1]["senses"][0])
        self.assertEqual(stats["worded pointers lending their word"], 1)

    def test_a_pointer_s_target(self):
        self.assertEqual(italian.pointer_target("Variante de avere."), "avere")
        self.assertEqual(italian.pointer_target("Variante obsoleta de ello (antes de consonantes)."), "ello")
        self.assertEqual(italian.pointer_target("Primera persona del presente de indicativo de orare."), "orare")
        self.assertEqual(italian.pointer_target("Variante de santo. Se usa ante nombres propios."), "santo")
        self.assertIsNone(italian.pointer_target("Variante de 3."), "no Italian word")
        self.assertIsNone(italian.pointer_target("Variante."), "no word named")

    def test_spec_scenario_a_meaning_after_a_colon(self):
        # `sta` holds no meaning of its own: its pointer's « esta »; `sette` holds « Siete »: no « Sectas ».
        _, glosses, _, _ = self.reduce()
        self.assertEqual(glosses["sta"], "Esta")
        self.assertEqual(glosses["sette"], "Siete")
        self.assertEqual(italian.carried_meaning('Forma del plural de setta: "sectas".'), "sectas")
        self.assertIsNone(italian.carried_meaning("Variante de avere."))

    def test_spec_scenario_a_function_word_s_description_replaced(self):
        _, glosses, _, _ = self.reduce()
        self.assertEqual(glosses["tu"], "Tú")
        self.assertEqual(glosses["avere"], "Haber; Tener")
        fired = set()
        self.assertEqual(italian.as_function_word(TU, fired)["senses"], [{"glosses": ["Tú."]}])
        self.assertIsNone(italian.as_function_word(AVERE_AUXILIARY, fired))
        # Keyed by the section's text: a corrected page fires nothing, and the summary names the row.
        self.assertEqual(italian.as_function_word({**TU, "senses": [sense("Tú.")]}), {**TU, "senses": [sense("Tú.")]})
        self.assertEqual(
            sorted(italian.unfired(fired)),
            sorted(f"{row.headword} ({row.pos})" for row in italian.FUNCTION_WORDS if row.headword not in ("tu", "avere")),
        )
        # `sono` is a form of `essere` in Italian's tables: no row of its own, it borrows nothing described.
        self.assertEqual(italian.as_function_word(SONO_VERB), SONO_VERB)

    def test_every_function_word_names_its_reason_and_its_page(self):
        self.assertEqual(len(italian.FUNCTION_WORDS), 11)
        self.assertEqual(
            {row.headword.lower() for row in italian.FUNCTION_WORDS}, {"tu", "essere", "avere", "lo", "ne", "lei", "loro"}
        )
        for row in italian.FUNCTION_WORDS:
            self.assertTrue(row.reason and row.report.startswith("https://es.wiktionary.org/wiki/"), row.headword)
        # Every row's word is a lemma of Italian's committed tables: a form's row would reach nothing.
        ranked = it_es.read_studied(os.path.join(_HERE, "tables", "it"), 60000)
        for row in italian.FUNCTION_WORDS:
            self.assertIn(row.headword.lower(), ranked, row.headword)
        # One row is written by a person, and says so: `ne`'s adverb (the owner, 2026-10-25).
        written = [row for row in italian.FUNCTION_WORDS if "a person wrote" in row.reason]
        self.assertEqual([(row.headword, row.pos, row.corrected) for row in written], [("ne", "adv", ("De allí, de ahí.",))])

    def test_spec_scenario_secondo_without_its_surname(self):
        _, glosses, _, _ = self.reduce()
        self.assertEqual(glosses["secondo"], "Segundo; Según")
        # The thin-pair clause: `segreto` has no lowercase entry, and the direct table glosses it.
        self.assertEqual(glosses["segreto"], "Secreto, oculto")
        read, stats = self.read(CLOAKED_IT_NAME, tabled=frozenset())
        self.assertEqual(read, [CLOAKED_IT_NAME], "no table glosses it: the name's note stays")
        read, stats = self.read(CLOAKED_IT_NAME, tabled=frozenset({"segreto"}))
        self.assertEqual((read, stats["names' notes off a word a table glosses"]), ([], 1))

    def test_spec_scenario_sense_links_superscripts_and_subscripts(self):
        _, glosses, _, _ = self.reduce()
        self.assertEqual(glosses["quale"], "Como")
        self.assertEqual(glosses["vero"], "Vero, verdadero, cierto; Vero, genuino, auténtico; Verdad")
        self.assertEqual(glosses["vedere"], "Ver")
        self.assertIn("«Coño», «carajo», «mierda»", glosses["cazzo"])
        self.assertEqual(italian.without_notes("Atrapar_(1, 4), pillar₃, agarrar₃."), "Atrapar, pillar₃, agarrar₃.")
        self.assertEqual(italian.without_notes("Útil¹."), "Útil.")
        self.assertEqual(italian.without_notes("C₄H₁₀"), "C₄H₁₀", "a formula keeps its digits")

    def test_each_rule_can_be_read_alone(self):
        read, _ = self.read(VERO_ADJ, rules=())
        self.assertEqual(read, [VERO_ADJ])
        read, stats = self.read(VERO_ADJ, rules=("notes",))
        self.assertEqual(read[0]["senses"][0]["glosses"], ["Vero, verdadero, cierto."])
        self.assertEqual(stats["notes taken out or openings capitalised"], 2)


class TheTables(Fixture):
    """D4: every table-shaped source read through fr-es's table rules and it-es's."""

    def test_spec_scenario_the_italian_word_given_as_spanish(self):
        # `corso` « Corso » (the Corsican language, whose Spanish is the same word) is refused; `credo`
        # is a Spanish word too (`ITALIAN_ALIKE`).
        direct = it_es.read_translated(self.jsonl(*DIRECT, name="d.jsonl"), str(self.dir / "d-read.jsonl"), inverted=False, readings={}, zipf=zipf)
        self.assertNotIn("corso", direct)
        self.assertEqual(direct["credo"], {"NOUN": ["credo", "fe"]})
        self.assertEqual(direct["casa"], {"NOUN": ["casa", "hogar"]}, "a cognate Spanish writes as often")
        self.assertEqual(italian.ITALIAN_ALIKE, frozenset({"credo", "friulano", "nutrimento", "visivo"}))
        _, glosses, _, _ = self.reduce()
        self.assertEqual(glosses["corso"], "Cambio", "the next source glosses it")

    def test_spec_scenario_a_word_with_a_grave_accent_or_a_degree_sign(self):
        direct = it_es.read_translated(self.jsonl(*DIRECT, name="d.jsonl"), str(self.dir / "d-read.jsonl"), inverted=False, readings={}, zipf=zipf)
        self.assertNotIn("abbonamento", direct)
        self.assertEqual(direct["qualcuno"], {"PRON": ["alguien"]})

    def test_spec_scenario_a_spanish_word_listed_once(self):
        # `accaduto` « Hecho; Hecho; Hecho »: listed once, under the part of speech Italian's readings name.
        studied = self.studied()
        readings = tables.read_readings(str(studied / "grammar.tsv"))
        direct = it_es.read_translated(self.jsonl(*DIRECT, name="d.jsonl"), str(self.dir / "d-read.jsonl"), inverted=False, readings=readings, zipf=zipf)
        self.assertEqual(direct["accaduto"], {"VERB": ["hecho"]})
        direct = it_es.read_translated(self.jsonl(*DIRECT, name="d.jsonl"), str(self.dir / "d-read.jsonl"), inverted=False, readings={}, zipf=zipf)
        self.assertEqual(direct["accaduto"], {"ADJ": ["hecho"]}, "else under the first listed")

    def test_spec_scenario_the_language_code(self):
        # `manzano` lists the template's « it »: read backwards, it glosses no `it`.
        inverted = it_es.read_translated(self.jsonl(*INVERTED, name="i.jsonl"), str(self.dir / "i-read.jsonl"), inverted=True, readings={}, zipf=zipf)
        self.assertNotIn("it", inverted)
        self.assertEqual(inverted["melo"], {"NOUN": ["manzano"]})

    def test_a_letter_glosses_no_word(self):
        letter = {"pos": "character", "translations": [{"word": "b"}], "word": "b"}
        one = {"pos": "noun", "translations": [{"word": "be"}], "word": "b"}
        direct = it_es.read_translated(self.jsonl(letter, one, name="d.jsonl"), str(self.dir / "d-read.jsonl"), inverted=False, readings={}, zipf=zipf)
        self.assertEqual(direct, {})

    def test_spec_scenario_the_spanish_section_s_notes(self):
        # « vedi italiano » and the placeholder take no Italian key; « analogia, vedi italiano » is « analogia ».
        table = it_es.read_spanish_section(self.jsonl(*SPANISH_SECTION, name="s.jsonl"))
        self.assertEqual(table["analogia"], {"NOUN": ["analogia"]})
        for note in ("vedi italiano", "vedi", "italiano", "definizione mancante", "se vuoi", "aggiungila tu"):
            self.assertNotIn(note, table)
        self.assertIsNone(italian.without_spanish_section_notes("vedi italiano"))
        self.assertIsNone(italian.without_spanish_section_notes("definizione mancante; se vuoi, aggiungila tu"))
        self.assertEqual(italian.without_spanish_section_notes("analogia, vedi italiano"), "analogia")
        # Each part of at most two words; a parenthesis is no word; three parts at most.
        self.assertEqual(table["sicuro"], {"ADJ": ["seguro"]})
        self.assertNotIn("solido", table, "a sense opening on a parenthesis narrows it: no word")
        self.assertEqual(table["ragazza"], {"NOUN": ["chica"]})
        self.assertNotIn("femminile di chico", table)
        self.assertEqual(it_es.spanish_section_words("a, b, c, d"), ["a", "b", "c"])
        # A parenthesis narrows the sense (`cruel` « (di dolore) insopportabile »): kept, so the part is no
        # word; a gender mark says nothing of it, and goes.
        self.assertEqual(it_es.spanish_section_words("(di dolore) insopportabile"), [])
        self.assertEqual(it_es.spanish_section_words("antenna (di insetto)"), [])
        self.assertEqual(it_es.spanish_section_words("rana (f), ranocchio (m)"), ["rana", "ranocchio"])
        self.assertEqual(it_es.spanish_section_words("cantante (m,f)"), ["cantante"])

    def test_wikidata_s_two_tables(self):
        links, items = it_es.read_wikidata(self.jsonl(*WIKIDATA, name="w.jsonl"))
        self.assertEqual(links, {"essere": {"VERB": ["ser"]}, "ape": {"NOUN": ["abeja"]}})
        self.assertEqual(items, {"fila": {"NOUN": ["cola"]}}, "a proper noun's lexemes gloss nothing")


class TheReducer(Fixture):
    """reduce-it-es.py over a work folder holding its sources."""

    def test_the_native_side_alone_from_the_committed_studied_tables(self):
        work, glosses, senses, mwe = self.reduce()
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith(("kaikki-", "wikidata-", "it_markit-")))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        self.assertEqual(senses["secondo"], "NUM:1\tADP:1")

    def test_spec_scenario_expressions(self):
        _, _, _, mwe = self.reduce()
        self.assertEqual(mwe["d'accordo"], "De acuerdo", "a word Italian's pre-pass splits, its apostrophe straight")
        self.assertEqual(mwe["un po'"], "Un poco")
        self.assertNotIn("stati uniti", mwe, "a proper noun's entry")

    def test_spec_scenario_usa_s_row(self):
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["USA"], "América; Estados Unidos")
        self.assertEqual(senses["USA"], "PROPN:2")
        self.assertNotIn("NATO", glosses, "a row with no expansion or synonym glossed, none")

    def test_the_sample_marks_its_sources(self):
        self.reduce()
        measured = json.loads((self.dir / "work" / "measures.json").read_text(encoding="utf-8"))
        rows = {lemma: source for lemma, _, source, _ in measured["sample"]}
        self.assertEqual(rows["casa"], "definitions")
        self.assertEqual(rows["acquirente"], "pivot (en: buyer (noun), « person who makes purchases »)")
        self.assertEqual(rows["degli"], "DI_FAMILY", "a row of the `di` family's table no source glosses (change 28's D11)")
        self.assertEqual(measured["pivot by edition"], {"en": 3, "fr": 1})
        self.assertEqual(list(measured["coverage by source"]), list(it_es.SOURCES))

    def test_the_update_report_names_every_source(self):
        # pack_report.py's line over it-es's measures: the share counts every source after the definitions,
        # and the coverage after each source is shown.
        import pack_report

        self.reduce()
        line = pack_report.share_line("it-es", self.dir / "work" / "measures.json")
        self.assertIn("come from a source after the definitions", line)
        self.assertIn("Coverage of the 5,000 / 10,000 / 20,000 commonest after each source: definitions ", line)
        self.assertIn("; pivot ", line)

    def test_spec_scenario_the_credits(self):
        work, *_ = self.reduce()
        notice = (work / "NOTICE").read_text(encoding="utf-8")
        for credit in (
            "Spanish Wiktionary (eswiktionary), Italian section",
            "Italian Wiktionary (itwiktionary)",
            "its Spanish section, read backwards",
            "Wikidata Lexemes",
            "CC0 1.0",
            "English (enwiktionary), French (frwiktionary), German (dewiktionary)",
            "Portuguese (ptwiktionary), Russian (ruwiktionary), Polish (plwiktionary) and Dutch (nlwiktionary)",
            "wordfreq",
            "UD Italian-MarkIT",
            "Morph-it!",
            "French Wiktionary (frwiktionary), Italian section",
            "The levels are estimated",
        ):
            self.assertIn(credit, " ".join(notice.split()), credit)
        self.assertNotIn("PUD", notice)
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("it", "es"))
        self.assertTrue(manifest["meta"]["levels_estimated"])
        self.assertEqual(manifest["meta"]["analyzer_version"], it_fr.analyser_version())
        self.assertIn({"name": "Wikidata", "licence": "Cc0"}, manifest["sources"])
        for source in manifest["sources"]:
            self.assertIn(source["name"], notice, "every source the builder's licence guard reads is named in the notice")

    def test_a_reduction_without_its_sources_stops(self):
        with self.assertRaisesRegex(SystemExit, "wikidata-lexemes-it-es.jsonl"):
            it_es.reduce(str(self.dir), str(self.studied()), 60000, "test", zipf=zipf, report=lambda line: None)

    def test_main_reads_its_arguments(self):
        work = self.work()
        argv = ["reduce-it-es.py", "--work", str(work), "--studied", str(self.studied()), "--built-at", "2026-10-25", "--pack-version", "test"]
        # wordfreq, doubled: the unit tests run without the reducer's dependencies.
        wordfreq = types.ModuleType("wordfreq")
        wordfreq.zipf_frequency = zipf
        with (
            mock.patch.object(sys, "argv", argv),
            mock.patch.dict(sys.modules, {"wordfreq": wordfreq}),
            contextlib.redirect_stderr(io.StringIO()) as err,
        ):
            it_es.main()
        self.assertIn("reduced it-es: lemmas=50", err.getvalue())


class TheStudiedSide(Fixture):
    """D1: Italian's studied tables as committed, it-fr's."""

    def test_the_acronym_rows_are_no_ranked_lemmas(self):
        studied = self.studied(["casa", "nato"], extra_forms=[("case", "casa")])
        self.assertEqual(it_es.read_studied(str(studied), 60000), {"casa": 1, "nato": 2})

    def test_tables_that_disagree_are_not_it_fr_s(self):
        studied = self.dir / "it"
        studied.mkdir()
        (studied / "forms.tsv").write_text("casa\tcasa\n", encoding="utf-8")
        (studied / "freq.tsv").write_text("casa\t1\ncane\t2\n", encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "disagree on the lemmas"):
            it_es.read_studied(str(studied), 60000)
        with self.assertRaisesRegex(SystemExit, "it-es reads Italian's committed tables"):
            it_es.read_studied(str(self.dir / "missing"), 60000)

    def test_it_es_s_copy_of_italian_is_it_fr_s(self):
        self.assertEqual(it_es.IT, it_fr.IT)
        self.assertEqual(italian.TOKEN.pattern, it_fr._TOKEN.pattern)
        self.assertEqual(italian.ELIDED_PIECES, frozenset(it_fr.ELISIONS) | frozenset(it_fr.ELIDED_PREPOSITIONS))
        forms = {"aujourd'hui": "x", "nell'": "x"}
        for word in ("d'accordo", "all'improvviso", "c'è", "senz'altro", "alla", "un po'", "po'", "nell'", "trent'anni"):
            self.assertEqual(italian.split_word(word, forms), it_fr.split_word(word, forms), word)


class SpanishTableRules(Fixture):
    """`reduce_spanish_tables.py`: fr-es's table rules copied, every fr-es global a parameter (D4). Given
    fr-es's globals, the copy gives what fr-es's own functions give on fr-es's own cases."""

    FR_RULES = tables.Rules(
        studied=fr_es.FR,
        language_code=fr_es._LANGUAGE_CODE,
        corrections=fr_es.CORRECTIONS,
        other_senses=fr_es.OTHER_SENSES,
        alike=fr_es.SPANISH_ALIKE,
        gap=fr_es.FRENCH_GAP,
    )
    C = fr_es_cases
    DIRECT = [
        C.MAISON_TRANSLATED, C.INTERET_TRANSLATED, C.ET_TRANSLATED, C.VENIR_TRANSLATED, C.TROLL_TRANSLATED,
        C.ELITE_TRANSLATED, C.PIERRE_TRANSLATED, C.A_TRANSLATED, C.Y_TRANSLATED, C.O_TRANSLATED, C.I_TRANSLATED,
        C.MAIN_D_OEUVRE_TRANSLATED, C.ALLER_DE_L_AVANT_TRANSLATED, C.PARTI_ADJ_TRANSLATED, C.PARTI_NOUN_TRANSLATED,
        C.RUSSE_ADJ_TRANSLATED, C.RUSSE_NOUN_TRANSLATED, C.CLAIR_ADJ_TRANSLATED, C.CLAIR_ADV_TRANSLATED, *C.NET_TRANSLATED,
        C.ARNAQUE_TRANSLATED, C.RETRAITE_TRANSLATED, C.DIAPORAMA_TRANSLATED, C.CLUB_TRANSLATED, C.CONTROLE_CONTINU_TRANSLATED,
        C.DS_TRANSLATED, C.HALL_TRANSLATED, C.EL_TRANSLATED, C.SE_LEVER_TRANSLATED, C.SE_REVEILLER_TRANSLATED,
    ]  # fmt: skip
    INVERTED = [
        C.TRAVES_LISTS, C.ESTE_ADJ_LISTS, C.ESTE_PRON_LISTS, C.H_LISTS, C.X_LISTS, C.I_LISTS, C.CASA_LISTS,
        C.GOLPE_MILITAR_LISTS, C.ADIESTRADOR_ADJ_LISTS, C.ADIESTRADOR_NOUN_LISTS, *C.CONDOM_LISTS, C.EEUU_LISTS, C.ONU_LISTS,
        C.AEC_LISTS, C.CALABAZA_LISTS, C.SAN_LUCAS_LISTS, C.PASCUA_LISTS, C.SANTO_TOME_LISTS, C.ORIENTE_LISTS,
        C.SECUESTRO_LISTS, C.MAR_ROJO_LISTS, C.LAISSEZ_FAIRE_LISTS, C.DESPERTARSE_LISTS,
    ]  # fmt: skip
    READINGS = {"condom": {"NOUN"}, "entraîneur": {"NOUN"}}

    def copy_direct(self, src, fired):
        words = tables.translation_words(src, str(self.dir / "copy-d.jsonl"), inverted=False, rules=self.FR_RULES, fired=fired)
        table = common.read_translations(words, inverted=False, studied=fr_es.FR)
        table = tables.without_studied_words(tables.in_typography(table), fr_es_cases.zipf, self.FR_RULES)
        return fr_es.listed_once_direct(table)

    def copy_inverted(self, src, readings):
        words = tables.translation_words(src, str(self.dir / "copy-i.jsonl"), inverted=True, rules=self.FR_RULES)
        table = common.read_translations(words, inverted=True, studied=fr_es.FR)
        return tables.listed_once({w: by_pos for w, by_pos in table.items() if len(w) != 1}, readings)

    def test_the_direct_table_reads_as_fr_es_reads_it(self):
        src = self.jsonl(*self.DIRECT, name="direct.jsonl")
        fired, copy_fired = set(), set()
        want = fr_es.read_translated(src, str(self.dir / "fr-es-d.jsonl"), inverted=False, readings={}, zipf=fr_es_cases.zipf, fired=fired)
        self.assertEqual(self.copy_direct(src, copy_fired), want)
        self.assertEqual(copy_fired, fired)
        self.assertEqual(tables.unfired(copy_fired, self.FR_RULES), fr_es.unfired(fired))
        self.assertNotIn("arnaque", want, "the case the rule holds is in the comparison")

    def test_the_inverted_table_reads_as_fr_es_reads_it(self):
        src = self.jsonl(*self.INVERTED, name="inverted.jsonl")
        for readings in ({}, self.READINGS):
            want = fr_es.read_translated(src, str(self.dir / "fr-es-i.jsonl"), inverted=True, readings=readings)
            self.assertEqual(self.copy_inverted(src, readings), want)
        self.assertNotIn("us", want, "the case the rule holds is in the comparison")

    def test_each_rule_is_fr_es_s(self):
        for french, native in (("fr", "calabaza"), ("lOrient", "Oriente"), ("Luc", "San Lucas"), ("US", "EEUU"), ("ONU", "ONU"), ("rap", "secuestro"), ("maison", "casa")):
            self.assertEqual(tables.read_backwards(french, native, self.FR_RULES), fr_es.read_backwards(french, native), (french, native))
        for entry_ in (*self.DIRECT, *self.INVERTED):
            self.assertEqual(tables.names_a_letter(entry_), fr_es._names_a_letter(entry_), entry_.get("word"))
        for word, native in (("arnaque", "arnaque"), ("diaporama", "diaporama"), ("club", "club"), ("retraite", "retraite")):
            self.assertEqual(
                tables.is_the_studied_word(word, native, fr_es_cases.zipf, self.FR_RULES), fr_es.is_the_french_word(word, native, fr_es_cases.zipf)
            )
        for case in fr_es_cases.fr_es.CORRECTIONS:
            original = {"word": case.headword, "pos": case.pos}
            original.update(
                {"senses": [{"glosses": [t]} for t in case.text]} if case.source == "section" else {"translations": [{"word": t} for t in case.text]}
            )
            self.assertEqual(tables.corrected(original, case.source, self.FR_RULES), fr_es._corrected(original, case.source), case.headword)
        table = {"cet": {"ADJ": ["este"], "PRON": ["este"]}, "x": {"NOUN": ["a...b", "a"], "ADJ": ["a"]}}
        self.assertEqual(tables.listed_once(table, {"x": {"ADJ"}}), fr_es.listed_once(table, {"x": {"ADJ"}}))
        self.assertEqual(tables.in_typography(table), fr_es.in_typography(table))
        self.assertEqual(tables.read_readings(str(self.dir / "missing.tsv")), {})

    def test_italian_s_parameters(self):
        rules = it_es.TABLE_RULES
        self.assertEqual((rules.studied, rules.language_code, rules.corrections, rules.other_senses), (it_es.IT, "it", (), {}))
        self.assertIs(rules.alike, italian.ITALIAN_ALIKE)
        self.assertFalse(tables.read_backwards("it", "manzano", rules))
        self.assertTrue(tables.read_backwards("fr", "calabaza", rules), "French's code is an Italian pair's word")
        self.assertEqual(rules.straight("d’accordo"), "d'accordo")
        self.assertTrue(tables.is_the_studied_word("corso", "corso", zipf, rules))
        self.assertFalse(tables.is_the_studied_word("credo", "credo", zipf, rules))


# — Change 28 (refine-lingua-it-es-glosses) —
#
# The rows are recorded from it-es's pinned release (lingua-pack-sources-it-es-2026.10.25), as the
# reduction reads each source's gloss of a lemma (`reduce_italian_es.source_glosses`), and the pivot's
# tables as counts; a case marked « made up » is not from the data.


def _row(lemma, sources, pivot=None):
    """`reduce_italian_es.Row` over a lemma's recorded sources (`name → (gloss, [(UPOS, n), …])`) and its
    pivot tables (`UPOS → {Spanish word: tables}`)."""
    tables_ = {lemma: {u: {n: {(u, n, k) for k in range(count)} for n, count in words.items()} for u, words in (pivot or {}).items()}}
    return italian.Row(lemma, {name: (g, [tuple(r) for r in runs]) for name, (g, runs) in sources.items()}, types.SimpleNamespace(tables=tables_))


def _zipf(table):
    """A made-up wordfreq: `{(word, language): Zipf}`, 0 elsewhere."""
    return lambda word, language: table.get((word, language), 0.0)


CASO = {
    "inverted": ("Azar", [("NOUN", 1)]),
    "wikidata links": ("Caso", [("NOUN", 1)]),
    "wikidata items": ("Caso", [("NOUN", 1)]),
    "spanish section": ("Suerte", [("NOUN", 1)]),
    "pivot": ("Caso, casualidad, causa", [("NOUN", 1)]),
}
CASO_PIVOT = {"NOUN": {"caso": 16, "causa": 2, "azar": 2, "suerte": 1, "casualidad": 4}}
ANDARE = {
    "definitions": ("Andar", [("VERB", 1)]),
    "direct": ("Ir, andar", [("VERB", 1)]),
    "inverted": ("Ir, irse", [("VERB", 1)]),
    "spanish section": ("Ir", [("VERB", 1)]),
    "pivot": ("Ir, andar, caminar", [("VERB", 1)]),
}
ANDARE_PIVOT = {"VERB": {"ir": 15, "caminar": 4, "andar": 8}}
INSIEME = {
    "inverted": ("Juntamente", [("ADV", 1)]),
    "spanish section": ("Conjunto", [("NOUN", 1)]),
    "pivot": ("Juntos, junto, a la vez; Conjunto, grupo, familia", [("ADV", 1), ("NOUN", 1)]),
}
INSIEME_PIVOT = {"ADV": {"junto": 3, "juntos": 5, "a la vez": 1, "juntamente": 1}, "NOUN": {"conjunto": 7, "grupo": 2, "familia": 1}}
DON = {"wikidata items": ("D.", [("NOUN", 1)]), "pivot": ("Don, padre, doña", [("NOUN", 1)])}
DON_PIVOT = {"NOUN": {"don": 2, "doña": 1, "padre": 1}, "INTJ": {"don": 1}}
WEB = {"wikidata items": ("Web", [("NOUN", 1)]), "pivot": ("Www", [("NOUN", 1)])}
MALE = {
    "definitions": ("Mal, malamente; Aflicción, dolor, enfermedad; Daño, perjuicio", [("ADV", 1), ("NOUN", 2)]),
    "direct": ("Mal", [("NOUN", 1)]),
    "inverted": ("Malo, daño", [("NOUN", 1)]),
    "spanish section": ("Mal, enfermedad", [("NOUN", 1)]),
    "pivot": ("Mal, dolor, daño; Malamente, pobremente; Malvado", [("NOUN", 1), ("ADV", 1), ("ADJ", 1)]),
}
MALE_PIVOT = {"NOUN": {"mal": 5, "daño": 2, "dolor": 3}, "ADV": {"malamente": 3, "mal": 3, "pobremente": 1}, "ADJ": {"malvado": 1}}
ENTRAMBI = {"inverted": ("Ambos", [("PRON", 1)]), "pivot": ("Los dos, cada, uno y otro; Ambas", [("DET", 1), ("PRON", 1)])}
ENTRAMBI_PIVOT = {"DET": {"ambos": 1, "los dos": 1, "uno y otro": 1, "cada": 1}, "PRON": {"ambos": 1, "ambas": 1}, "NUM": {"ambos": 2, "ambas": 1}}
SPESSO = {
    "definitions": ("Gordo", [("ADJ", 1)]),
    "inverted": ("Grueso", [("ADJ", 1)]),
    "spanish section": ("A menudo", [("X", 1)]),
    "pivot": ("Espeso, denso, gordo; A menudo, frecuentemente, con frecuencia", [("ADJ", 1), ("ADV", 1)]),
}
PIU = {
    "definitions": ("Más", [("ADJ", 1)]),
    "direct": ("Más", [("ADJ", 1)]),
    "inverted": ("Más", [("ADJ", 1)]),
    "spanish section": ("Ya", [("ADV", 1)]),
    "pivot": ("En más, ya…, ya no; Más, positivo, contrachapado", [("ADV", 1), ("ADJ", 1)]),
}
SUO = {
    "inverted": ("Su", [("ADJ", 1)]),
    "spanish section": ("Su", [("ADJ", 1)]),
    "pivot": ("Su, sus, del mismo", [("DET", 1)]),
}
RAPIDAMENTE = {"spanish section": ("Aquejadamente", [("ADV", 1)]), "pivot": ("Rápidamente, deprisa, velozmente", [("ADV", 1)])}
RAGAZZA = {
    "definitions": ("Chico, muchacho, cabro; Novio, pololo", [("NOUN", 2)]),
    "direct": ("Chica, muchacha, niña", [("NOUN", 1)]),
    "inverted": ("Jovencito", [("ADJ", 1)]),
    "spanish section": ("Chica", [("NOUN", 1)]),
    "pivot": ("Chica, muchacha, niña", [("NOUN", 1)]),
}
POSTO = {
    "direct": ("En", [("NOUN", 1)]),
    "inverted": ("Sitio", [("NOUN", 1)]),
    "spanish section": ("En; Punto", [("X", 1), ("NOUN", 1)]),
    "pivot": ("Lugar, sitio, puesto", [("NOUN", 1)]),
}
PORCA = {
    "definitions": ("(Sus scrofa domesticus) cerdo, puerco, chancho", [("NOUN", 1)]),
    "direct": ("Caballón, camellón", [("NOUN", 1)]),
    "inverted": ("Chancha", [("NOUN", 1)]),
    "pivot": ("Caballón, zorra, cerda", [("NOUN", 1)]),
}
PORCA_PIVOT = {"NOUN": {"caballón": 2, "zorra": 1, "cerda": 1, "puerca": 1}}
FACCIA = [  # the Italian Wiktionary's translations, recorded: one entry of two
    {"pos": "noun", "word": "faccia", "translations": [
        {"word": "cara", "sense": "(anatomia) viso"},
        {"word": "haga", "sense": "1ª persona singolare del congiuntivo presente di fare"},
        {"word": "hagas", "sense": "2ª persona singolare del congiuntivo presente di fare"},
        {"word": "haga", "sense": "3ª persona singolare dell'imperativo di fare"},
    ]},
    {"pos": "noun", "word": "studio", "translations": [
        {"word": "Atelier", "sense": "locale per usi professionali"},
        {"word": "estudio", "sense": "prima persona singolare dell'indicativo presente dell'indicativo presente di studiare"},
    ]},
]  # fmt: skip
NUDO = entry(
    "nudo",
    "adj",
    sense("Desnudo."),
    {"glosses": ["Dicho de un mueble heráldico representando una cabalgadura, alegre."], "topics": ["heraldry"]},
)
SENATORE_NAME = entry("Senatore", "name", sense("Apellido"))
ESPOSITO_NAME = entry("Esposito", "name", sense("Apellido italiano."))
PAOLO_NAME = entry("Paolo", "name", sense("Nombre de pila de varón, equivalente del español Pablo."))
GIUSEPPE_NAME = entry("Giuseppe", "name", sense("Nombre de varón, equivalente del español José"))
RAGAZZA_ENTRY = entry("ragazza", "noun", sense("Forma del femenino singular de ragazzo.", ["form-of"], form_of=["ragazzo"]))
INOLTRE = (
    "Además, adicionalmente, allende, a mayor abundamiento, demás, de yapa, en añadidura, encima, es más, más, más "
    "encima, otrosí, para más recacha por añadidura o también"
)


class Change28(Fixture):
    """Helpers for change 28's rules."""

    def refine_section(self, *entries, readings=None, tabled=frozenset(), rules=italian.RULES):
        src = self.jsonl(*entries)
        dst, record = italian.refine_section(src, str(self.dir / "refined.jsonl"), readings=readings or {}, tabled=tabled, rules=rules)
        return [json.loads(line) for line in Path(dst).read_text(encoding="utf-8").splitlines()], record

    def rows(self, by_lemma, *, ranks=None, rules=italian.RULES, record=None, readings=None, zipf=_zipf({}), pivot=None, counts=None):
        """`refine_rows` over recorded rows: `lemma → sources`, each lemma's native row its first source's."""
        glossed_by = {lemma: {name: (g, [tuple(r) for r in runs]) for name, (g, runs) in sources.items()} for lemma, sources in by_lemma.items()}
        glosses, runs = {}, {}
        for lemma, sources in glossed_by.items():
            first = next(name for name in italian.SOURCES if name in sources)
            glosses[lemma], runs[lemma] = sources[first]
        ranks = ranks or {lemma: k for k, lemma in enumerate([*by_lemma, *italian.DI_FAMILY], start=1)}
        unfired = italian.refine_rows(
            glosses, runs, glossed_by, ranks, record=record or italian.SectionRecord(), readings=readings or {}, zipf=zipf,
            pivot=pivot or types.SimpleNamespace(tables={}), counts=counts or {}, rules=rules,
        )  # fmt: skip
        return glosses, runs, unfired


class TheRowsAgree(Change28):
    """Change 28's D2: a row opens on the word its sources agree on."""

    def test_spec_scenario_a_lone_first_word_yields(self):
        # `caso` « Azar », the Spanish tables read backwards alone; Wikidata's links and items and the pivot's
        # sixteen tables give « Caso ».
        row = _row("caso", CASO, CASO_PIVOT)
        self.assertEqual(row.support["caso"], 3 + 15, "each of the pivot's tables a source")
        self.assertEqual(italian.agreement(row, CASO["inverted"]), ("Caso", [("NOUN", 1)]))

    def test_spec_scenario_most_sources_open_on_another_word(self):
        # « Ir » opens four sources, « Andar » the section alone: the direct table's sense opens the row, and
        # the section's « Andar », which it says again, goes with the repeats (D9b).
        row = _row("andare", ANDARE, ANDARE_PIVOT)
        moved = italian.agreement(row, ANDARE["definitions"])
        self.assertEqual(moved, ("Ir, andar; Andar", [("VERB", 2)]))
        self.assertEqual(italian.without_repeats(moved), ("Ir, andar", [("VERB", 1)]))

    def test_spec_scenario_the_winner_keeps_the_row_s_part_of_speech(self):
        # « Conjunto » is given by two sources, but as a noun: the adverb row opens on the adverb two give.
        row = _row("insieme", INSIEME, INSIEME_PIVOT)
        gloss, runs = italian.agreement(row, INSIEME["inverted"])
        self.assertEqual(gloss, "Juntos, junto, a la vez")
        self.assertEqual(runs, [("ADV", 1)])

    def test_spec_scenario_a_reviewed_row_stays(self):
        # `ne`'s adverb « De allí, de ahí », a person's (change 27), which the pivot glosses otherwise.
        ne = {
            "definitions": ("De allí, de ahí; De él, de ella, de ellos, de ellas", [("ADV", 1), ("PRON", 1)]),
            "pivot": ("De esto, del mismo, de ello", [("ADV", 1)]),
        }
        self.assertIn("ne", italian.REVIEWED)
        pivot = types.SimpleNamespace(tables={"ne": {"ADV": {"de esto": {1, 2}, "de ello": {3}, "del mismo": {4}}}})
        for rules in (italian.RULES, tuple(r for r in italian.RULES if r != "reviewed rows")):
            glosses, _, _ = self.rows({"ne": ne}, pivot=pivot, rules=rules)
            self.assertEqual(glosses["ne"], "De allí, de ahí; De él, de ella, de ellos, de ellas")

    def test_spec_scenario_wikidata_s_lone_item(self):
        self.assertEqual(italian.lone_wikidata_items(_row("don", DON, DON_PIVOT)), DON["pivot"])
        # `web` « Web »: the pivot's « Www » is one table's, so the item stays.
        self.assertIsNone(italian.lone_wikidata_items(_row("web", WEB, {"NOUN": {"www": 1}})))

    def test_a_row_its_sources_agree_on_stays(self):
        # `male`: its section's adverb, which the pivot also gives first in that part of speech.
        self.assertIsNone(italian.agreement(_row("male", MALE, MALE_PIVOT), MALE["definitions"]))

    def test_a_word_s_feminine_is_another_word(self):
        # `entrambi`: « ambos » and « ambas » are counted apart, so the row opens on the pivot's « Ambas »
        # (« ambas » in two of its tables), which FIRST_RUN answers with the Spanish tables' « Ambos ».
        row = _row("entrambi", ENTRAMBI, ENTRAMBI_PIVOT)
        self.assertNotEqual(row.support["ambos"], row.support["ambos"] + row.support["ambas"])
        self.assertEqual(italian.FIRST_RUN["entrambi"].text, "Ambos")

    def test_a_short_or_splits_an_item(self):
        self.assertEqual(italian.words_of("Mas o pero, sino"), ["mas", "pero", "sino"])
        self.assertEqual(italian.words_of("¡Gracias!, (eso es) correcto"), ["gracias", "correcto"])
        self.assertEqual(italian.winner_first("Andar, ir", "ir"), "Ir, andar")


class PartsOfSpeech(Change28):
    """Change 28's D3: MarkIT's order, and the missing adverb or preposition."""

    def test_spec_scenario_a_function_word_s_commonest_part_of_speech(self):
        counts = collections.Counter({("ancora", "ADV"): 26})
        self.assertEqual(
            italian.markit_order("ancora", ("Ancla; Todavía", [("NOUN", 1), ("ADV", 1)]), counts), ("Todavía; Ancla", [("ADV", 1), ("NOUN", 1)])
        )

    def test_spec_scenario_a_missing_adverb(self):
        row = _row("spesso", SPESSO)
        got = italian.missing_adverb_or_preposition(row, SPESSO["definitions"], {"spesso": {"ADV": 27, "ADJ": 1}})
        self.assertEqual(got, ("A menudo, frecuentemente, con frecuencia; Gordo", [("ADV", 1), ("ADJ", 1)]))

    def test_a_row_with_a_function_word_or_a_lone_source_gains_nothing(self):
        # `più`: the adverb « Ya » one source gives; `suo`: a determiner beside the Spanish edition's adjective.
        self.assertIsNone(italian.missing_adverb_or_preposition(_row("più", PIU), PIU["definitions"], {"più": {"ADV": 126, "DET": 5}}))
        self.assertIsNone(italian.missing_adverb_or_preposition(_row("suo", SUO), SUO["inverted"], {"suo": {"DET": 46, "PRON": 1}}))
        # `dopo` holds an adverb already: no function word is filled in beside one.
        dopo = {"definitions": ("Después, luego; Tras", [("ADV", 2)])}
        self.assertIsNone(italian.missing_adverb_or_preposition(_row("dopo", dopo), dopo["definitions"], {"dopo": {"ADP": 80, "ADV": 30}}))

    def test_markit_is_read_from_the_work_folder(self):
        self.assertEqual(it_es.MARKIT, ("it_markit-ud-train.conllu", "it_markit-ud-dev.conllu"))
        self.assertEqual([spec["file"] for spec in ps.PINNED["it-es"].values()], list(it_es.MARKIT))
        self.assertEqual(ps.PINNED["it-es"], ps.ITALIAN_MARKIT)
        work = self.work()
        (work / it_es.MARKIT[1]).unlink()
        with self.assertRaisesRegex(SystemExit, "MarkIT"):
            it_es.reduce(str(work), str(self.studied()), 60000, "test", zipf=zipf, report=lambda _: None)


class SpanishWords(Change28):
    """Change 28's D4, D5, D6 and D10: the glosses hold Spanish words, read as their sources mean them."""

    ZIPF = staticmethod(_zipf({("evident", "en"): 4.6, ("friulano", "it"): 3.1, ("haga", "es"): 5.0}))

    def test_spec_scenario_a_word_that_is_not_spanish(self):
        tables_ = {"direct": {"evidente": {"ADJ": ["evident"]}}, "inverted": {"evidente": {"ADJ": ["evidente"]}}}
        got = italian.refine_tables(tables_, readings={}, zipf=self.ZIPF, lemmas=set(), verb_forms=set())
        self.assertEqual(got, {"direct": {}, "inverted": {"evidente": {"ADJ": ["evidente"]}}})

    def test_spec_scenario_a_word_spanish_writes_alike(self):
        self.assertIn("friulano", italian.ITALIAN_ALIKE)
        tables_ = {"direct": {"friulano": {"ADJ": ["friulano"]}}}
        self.assertEqual(italian.refine_tables(tables_, readings={}, zipf=self.ZIPF, lemmas=set(), verb_forms=set()), tables_)

    def test_spec_scenario_a_place_label(self):
        tables_ = {
            "direct": {"ladro": {"ADJ": ["ladrón", "chorizo", "Spagna", "chorro"]}, "gesù": {"INTJ": ["Jesús"]}},
            "pivot": {"ladro": {"NOUN": ["ladrón", "Spagna"]}},
        }
        got = italian.refine_tables(tables_, readings={}, zipf=self.ZIPF, lemmas={"spagna", "jesús"}, verb_forms=set())
        self.assertEqual(got["direct"]["ladro"], {"ADJ": ["ladrón", "chorizo", "chorro"]})
        self.assertEqual(got["direct"]["gesù"], {"INTJ": ["Jesús"]}, "a list's first word is never a label")
        self.assertEqual(got["pivot"], tables_["pivot"], "the two Wiktionaries' tables alone")

    def test_spec_scenario_a_rare_first_word(self):
        zipf_ = _zipf({("rápidamente", "es"): 3.9, ("aquejadamente", "es"): 0.0})
        row = _row("rapidamente", RAPIDAMENTE)
        self.assertEqual(italian.rare_first_word(row, RAPIDAMENTE["spanish section"], zipf_), RAPIDAMENTE["pivot"])
        # A commoner word of the first sense opens it first.
        self.assertEqual(
            italian.rare_first_word(row, ("Movíl, celular", [("NOUN", 1)]), _zipf({("celular", "es"): 4.0})),
            ("Celular, movíl", [("NOUN", 1)]),
        )

    def test_spec_scenario_a_feminine_s_own_word(self):
        readings = {"ragazza": {"NOUN"}, "comunicativa": {"NOUN"}}
        _, record = self.refine_section(RAGAZZA_ENTRY, readings=readings)
        self.assertEqual(record.feminine, {"ragazza"})
        self.assertNotIn("ragazza", record.meaning)
        got = italian.feminine_s_own_word(_row("ragazza", RAGAZZA), RAGAZZA["definitions"], record, readings)
        self.assertEqual(got, RAGAZZA["direct"])

    def test_spec_scenario_a_lent_masculine_no_other_source_glosses(self):
        record = italian.SectionRecord()
        record.feminine.add("comunicativa")
        lent = {"definitions": ("Comunicativo", [("ADJ", 1)])}
        self.assertIsNone(italian.feminine_s_own_word(_row("comunicativa", lent), lent["definitions"], record, {"comunicativa": {"NOUN"}}))

    def test_spec_scenario_a_verb_form_inside_a_noun_s_row(self):
        src = self.jsonl(*FACCIA, name="direct.jsonl")
        dst = italian.without_verb_forms_translations(src, str(self.dir / "forms.jsonl"), {"studio": {"estudio"}})
        kept = {e["word"]: [t["word"] for t in e["translations"]] for e in map(json.loads, Path(dst).read_text(encoding="utf-8").splitlines())}
        self.assertEqual(kept["faccia"], ["cara"])
        self.assertEqual(kept["studio"], ["Atelier", "estudio"], "a word another source gives stays")

    def test_a_verb_s_words_off_another_verb_s_form(self):
        # `dati`, which Italian reads as `dare`'s form and as a noun: the pivot's « Sea » goes; a verb's
        # words that are a row's only part of speech stay.
        tables_ = {"pivot": {"dati": {"NOUN": ["datos", "facto"], "VERB": ["sea"]}, "fatti": {"VERB": ["hechos"]}}}
        got = italian.refine_tables(tables_, readings={"dati": {"NOUN"}}, zipf=self.ZIPF, lemmas=set(), verb_forms={"dati", "fatti"})
        self.assertEqual(got["pivot"], {"dati": {"NOUN": ["datos", "facto"]}, "fatti": {"VERB": ["hechos"]}})

    def test_spec_scenario_a_heraldic_sense(self):
        read, _ = self.refine_section(NUDO)
        self.assertEqual([s["glosses"] for s in read[0]["senses"]], [["Desnudo."]])


class NotesAndRepeats(Change28):
    """Change 28's D7, D8 and D9: no name's note, list or repeat a common word does not need."""

    def test_spec_scenario_a_surname_s_note_off_a_common_word(self):
        readings = {"senatore": {"NOUN"}, "esposito": {"NOUN"}}
        read, _ = self.refine_section(SENATORE_NAME, ESPOSITO_NAME, readings=readings, tabled=frozenset({"senatore"}))
        self.assertEqual([e["word"] for e in read], ["Esposito"], "`senatore`'s only sense goes: Wikidata glosses it")

    def test_spec_scenario_a_surname_alone(self):
        read, _ = self.refine_section(ESPOSITO_NAME, readings={"esposito": {"NOUN"}})
        self.assertEqual(read, [ESPOSITO_NAME])

    def test_spec_scenario_a_given_name(self):
        read, _ = self.refine_section(PAOLO_NAME, GIUSEPPE_NAME)
        self.assertEqual([e["senses"][0]["glosses"] for e in read], [["Pablo (nombre de pila)"], ["José (nombre de pila)"]])

    def test_spec_scenario_a_list_cut(self):
        self.assertEqual(italian.long_lists_cut((INOLTRE, [("SCONJ", 1)])), ("Además, adicionalmente, allende", [("SCONJ", 1)]))
        # `sì`'s six items stay, « sí » among them.
        si = ("Afirmativo, así es, bien, (eso es) correcto, de acuerdo, sí", [("INTJ", 1)])
        self.assertIsNone(italian.long_lists_cut(si))

    def test_spec_scenario_a_taxon_before_the_meaning(self):
        self.assertEqual(italian.without_taxa(("(Equus caballus) Caballo", [("NOUN", 1)])), ("Caballo", [("NOUN", 1)]))

    def test_spec_scenario_a_repeated_sense(self):
        self.assertEqual(
            italian.without_repeats(("Persona (ser humano); Persona", [("NOUN", 2)])), ("Persona (ser humano)", [("NOUN", 1)])
        )
        self.assertIsNone(italian.without_repeats(("Firma (leyenda); Firma (empresa)", [("NOUN", 2)])))
        self.assertEqual(italian.without_repeats(("Islandés, Islandés", [("ADJ", 1)])), ("Islandés", [("ADJ", 1)]))
        # Made up: « Gracias » again under another part of speech, once « ¡ » and « ! » are out.
        self.assertEqual(italian.without_repeats(("¡Gracias!; Gracias", [("INTJ", 1), ("NOUN", 1)])), ("¡Gracias!", [("INTJ", 1)]))

    def test_spec_scenario_an_expression_glossed_twice(self):
        expressions = {
            "d'accordo": "De acuerdo; De acuerdo, vale",
            "vicolo cieco": "Callejón sin salida; En sentido figurado, callejón sin salida, situación sin solución real o aparente",
        }
        italian.without_expression_repeats(expressions)
        self.assertEqual(expressions["d'accordo"], "De acuerdo, vale")
        self.assertTrue(expressions["vicolo cieco"].startswith("Callejón sin salida; "), "a sense contained in another stays")


class TheReviewedTables(Change28):
    """Change 28's D11 and D12: the `di` family, `FIRST_RUN` and `CORRECTIONS`."""

    def test_spec_scenario_an_article_read_through_the_table(self):
        della = {"definitions": ("Contracción de la preposición di y el artículo la", [("X", 1)])}
        glosses, runs, _ = self.rows({"della": della})
        self.assertEqual((glosses["della"], runs["della"]), ("De la", [("ADP", 1)]))
        self.assertIn("written by a person", italian.DI_FAMILY["della"].reason)

    def test_spec_scenario_a_form_no_source_glosses(self):
        glosses, _, _ = self.rows({})
        self.assertEqual({w: glosses[w] for w in italian.DI_FAMILY}, {
            "del": "Del", "dello": "Del", "della": "De la", "dei": "De los", "degli": "De los", "delle": "De las",
        })  # fmt: skip
        for word in ("della", "dei", "degli", "delle"):
            self.assertIn("written by a person", italian.DI_FAMILY[word].reason, word)
        for word in ("del", "dello"):
            self.assertNotIn("written by a person", italian.DI_FAMILY[word].reason, word)

    def test_a_family_s_row_that_is_no_ranked_lemma_fails(self):
        with self.assertRaisesRegex(SystemExit, "DI_FAMILY dei"):
            self.rows({}, ranks={w: 1 for w in italian.DI_FAMILY if w != "dei"})

    def test_spec_scenario_a_row_the_reviewed_table_opens(self):
        glosses, runs, _ = self.rows({"posto": POSTO})
        self.assertEqual((glosses["posto"], runs["posto"]), ("Lugar, sitio, puesto", [("NOUN", 1)]))

    def test_spec_scenario_a_sense_the_table_no_longer_finds(self):
        moved = {**POSTO, "pivot": ("Lugar, sitio", [("NOUN", 1)])}
        with self.assertRaisesRegex(SystemExit, "FIRST_RUN posto"):
            self.rows({"posto": moved})

    def test_spec_scenario_a_lent_sense_a_source_s_word_restores(self):
        pivot = types.SimpleNamespace(tables={"porca": {u: {n: set(range(c)) for n, c in w.items()} for u, w in PORCA_PIVOT.items()}})
        glosses, runs, _ = self.rows({"porca": PORCA}, pivot=pivot, rules=tuple(r for r in italian.RULES if r != "reviewed rows"))
        self.assertEqual(glosses["porca"], "Caballón, camellón", "agreement drops the lent « cerdo »")
        glosses, runs, _ = self.rows({"porca": PORCA}, pivot=pivot)
        self.assertEqual((glosses["porca"], runs["porca"]), ("Cerda; Caballón, camellón", [("NOUN", 2)]))
        for word in ("cerda", "caballón", "camellón"):
            self.assertTrue(any(word in g.lower() for g, _ in PORCA.values()), f"« {word} » is a source's")

    def test_spec_scenario_a_word_taken_for_another_language_s(self):
        glosses, _, _ = self.rows({"the": {"pivot": ("Té", [("NOUN", 1)])}})
        self.assertNotIn("the", glosses)

    def test_spec_scenario_a_correction_that_finds_nothing(self):
        # Made up: the pivot no longer writes `the` « Té »; the summary line names the row.
        glosses, _, unfired = self.rows({"the": {"pivot": ("Té negro", [("NOUN", 1)])}})
        self.assertEqual(glosses["the"], "Té negro")
        self.assertIn("CORRECTIONS the (pivot)", unfired)
        self.assertIn("FIRST_RUN posto", unfired, "a FIRST_RUN row whose lemma is not glossed")
        work, *_ = self.reduce()
        self.assertIn("change 28's reviewed rows: 20 of FIRST_RUN, 22 of CORRECTIONS, 6 of DI_FAMILY; rows that found nothing", self.report)
        self.assertIn("CORRECTIONS the (pivot)", self.report)

    def test_every_reviewed_row_names_its_reason_and_its_page(self):
        self.assertEqual(len(italian.FIRST_RUN), 20)
        self.assertEqual(len(italian.CORRECTIONS), 22)
        for lemma, row in italian.FIRST_RUN.items():
            self.assertIn(row.source, italian.SOURCES, lemma)
            self.assertTrue(row.reason, lemma)
        for row in italian.CORRECTIONS:
            self.assertIn(row.source, italian.SOURCES, row.headword)
            self.assertTrue(row.reason and row.report.startswith("https://"), row.headword)
        self.assertEqual(len({row.headword for row in italian.CORRECTIONS}), len(italian.CORRECTIONS), "one row a word")
        self.assertFalse(set(italian.FIRST_RUN) & {row.headword for row in italian.CORRECTIONS})

    def test_every_gloss_written_by_a_person(self):
        # Tasks 0.1, 0.2 and 0.4 (the owner, 2026-10-25), and change 27's `ne`: each marked in its table.
        self.assertEqual(
            italian.BY_A_PERSON,
            {
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
            },
        )
        corrections = {row.headword: row for row in italian.CORRECTIONS}
        marked = {row.headword for row in italian.CORRECTIONS if "written by a person" in row.reason}
        marked |= {word for word, row in italian.DI_FAMILY.items() if "written by a person" in row.reason}
        marked |= {row.headword for row in italian.FUNCTION_WORDS if "a person wrote" in row.reason}
        self.assertEqual(marked, set(italian.BY_A_PERSON))
        for word, words in italian.BY_A_PERSON.items():
            if word in corrections:
                written = "; ".join(s for _, s in corrections[word].corrected)
                self.assertIn(words.lower(), written.lower(), word)
        self.assertFalse(any("written by a person" in row.reason for row in italian.FIRST_RUN.values()), "FIRST_RUN opens on a source's sense")

    def test_the_owner_s_three_words(self):
        # Task 0.4 (the owner, 2026-10-25): `tessera`, `cancellato`, `ognuno`.
        rows = {
            "tessera": {"pivot": ("Tesela", [("NOUN", 1)])},
            "cancellato": {"direct": ("Celosiado", [("ADJ", 1)]), "spanish section": ("Celosiado", [("ADJ", 1)])},
            "ognuno": {"direct": ("Quienquiera, cualquiera", [("PRON", 1)]), "pivot": ("Sendos; Todo, cada, cada una", [("ADJ", 1), ("PRON", 1)])},
        }
        glosses, _, unfired = self.rows(rows)
        self.assertEqual({w: glosses[w] for w in rows}, {"tessera": "Tarjeta, carné", "cancellato": "Cancelado, borrado", "ognuno": "Cada uno"})
        self.assertFalse([u for u in unfired if u.split(" ")[1] in rows])


class TheRulesAlone(Change28):
    """Change 28's task 1.1: each rule named, and the committed rows written byte for byte when none fires."""

    def test_each_rule_is_named(self):
        self.assertEqual(italian.RULES, italian.PRE_PASS + italian.REFINED)
        self.assertEqual(len(set(italian.REFINED)), 16)
        self.assertEqual(italian.SOURCES, it_es.SOURCES)

    def test_no_rule_firing_writes_the_rows_as_before(self):
        _, before, before_senses, before_mwe = self.reduce(markit=("", ""))
        self.dir = self.dir / "again"
        self.dir.mkdir()
        work = self.work(markit=("", ""))
        it_es.reduce(str(work), str(self.studied()), 60000, "test", zipf=zipf, report=lambda _: None, rules=italian.PRE_PASS)
        unread = {line.partition("\t")[0]: line.partition("\t")[2] for line in (work / "gloss.tsv").read_text(encoding="utf-8").splitlines()}
        # The `di` family's rows alone differ: the fixture's rows give no other rule anything to do.
        self.assertEqual({w: g for w, g in before.items() if w not in italian.DI_FAMILY}, {w: g for w, g in unread.items() if w not in italian.DI_FAMILY})
        self.assertEqual(before_mwe, {line.partition("\t")[0]: line.partition("\t")[2] for line in (work / "mwe.tsv").read_text(encoding="utf-8").splitlines()})


class TheRules(unittest.TestCase):
    """it-es's rule digest (D1, D4; change 28's D1: fr-en's treebank rule and MarkIT's counts)."""

    def test_it_es_s_rule_files(self):
        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-it-es.py")],
            [
                "reduce-it-es.py",
                "reduce_common.py",
                "reduce_edition_es.py",
                "reduce_french_treebank.py",
                "reduce_italian_acronyms.py",
                "reduce_italian_es.py",
                "reduce_italian_treebank.py",
                "reduce_spanish_tables.py",
            ],
        )

    def test_it_es_s_own_modules_are_no_other_pair_s(self):
        for reducer in sorted(Path(_HERE).glob("reduce-*.py")):
            if reducer.name != "reduce-it-es.py":
                loaded = [p.name for p in ps.rule_files(reducer)]
                self.assertNotIn("reduce_italian_es.py", loaded, reducer.name)
                self.assertNotIn("reduce_spanish_tables.py", loaded, reducer.name)

    def test_spec_scenario_nothing_else_moves(self):
        # The shared modules and the Spanish edition's are not edited: every committed pair's pin still
        # records the rules it was reduced by.
        for pin in sorted((Path(_HERE) / "tables").glob("*/pin.json")):
            reducer = Path(_HERE) / f"reduce-{pin.parent.name}.py"
            self.assertEqual(ps.rules_sha256(reducer), ps.get(ps.load(pin), "reducer.sha256"), pin.parent.name)

    def test_it_es_reads_the_files_its_pin_records(self):
        self.assertEqual(
            {name for names in ps.DUMPS["it-es"].values() for name in names},
            {it_es.SECTION, it_es.DIRECT, it_es.INVERTED, it_es.WIKIDATA, it_es.SPANISH_SECTION, *it_es.PIVOT.values()},
        )


if __name__ == "__main__":
    unittest.main()
