# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the it-en reducer (add-lingua-pack-it-en) and its pre-pass (`reduce_italian_en.py`).

The senses are recorded from the English Wiktionary's Italian section as `pack_sources.derive` writes it
from the English dump of 2026-10-21 (`kaikki-Italian.jsonl`, sha256 ae7de484…, it-fr's release
`lingua-pack-sources-it-fr-2026.10.23`), and the translations from the Italian Wiktionary's English
translations derived from its dump (`kaikki-it-traductions-en.jsonl`, sha256 e192643a…), cut down to the
fields the rules read. A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import pack_sources as ps  # noqa: E402
import reduce_common as common  # noqa: E402
import reduce_edition_en as english  # noqa: E402
import reduce_italian_acronyms as acronyms  # noqa: E402
import reduce_italian_en as italian  # noqa: E402


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(f"reduce_{pair.replace('-', '_')}_it_en", os.path.join(_HERE, f"reduce-{pair}.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


it_en = _reducer("it-en")
it_fr = _reducer("it-fr")


def sense(gloss, tags=(), form_of=None, alt_of=None):
    out = {"glosses": [gloss]}
    if tags:
        out["tags"] = list(tags)
    if form_of is not None:
        out["form_of"] = [{"word": w, **({"extra": x} if x else {})} for w, x in form_of]
    if alt_of is not None:
        out["alt_of"] = [{"word": w, **({"extra": x} if x else {})} for w, x in alt_of]
    return out


def entry(word, pos, *senses):
    return {"word": word, "pos": pos, "senses": list(senses)}


# — Recorded entries of the section —

DEL = entry(
    "del",
    "contraction",
    sense("contraction of di + il; some, of the, from the", ("abbreviation", "alt-of", "contraction"), alt_of=[("di", "+ il; some, of the, from the")]),
)
DELLA = entry(
    "della",
    "contraction",
    sense("contraction of di la; of the, from the", ("abbreviation", "alt-of", "contraction"), alt_of=[("di la", "of the, from the")]),
)
ED = entry(
    "ed",
    "conj",
    sense(
        "alternative form of e for euphony, especially before /e/ or /ɛ/; and",
        ("alt-of", "alternative", "before-vowel"),
        alt_of=[("e for euphony", "especially before /e/ or /ɛ/; and")],
    ),
)
AD = entry(
    "ad",
    "prep",
    sense(
        "alternative form of a for euphony, especially before /a/; to, at, in",
        ("alt-of", "alternative", "before-vowel"),
        alt_of=[("a for euphony", "especially before /a/; to, at, in")],
    ),
)
AD_ACRONYM = entry(
    "AD",
    "noun",
    sense(
        "initialism of amministratore delegato (“CEO”)",
        ("abbreviation", "alt-of", "initialism"),
        alt_of=[("amministratore delegato", "CEO")],
    ),
)
AMMINISTRATORE_DELEGATO = entry("amministratore delegato", "noun", sense("chief executive officer, CEO", ("masculine",)))
GLI_ARTICLE = entry(
    "gli",
    "article",
    sense(
        "form of the article i (“the”) used before a vowel, impure s, gn, pn, ps, sc, x, y, and z, and also with the plural noun dei (“gods”); the",
        ("form-of", "masculine", "plural"),
        form_of=[("the article i", "(“the”) used before a vowel…")],
    ),
)
GLI_PRONOUN = entry("gli", "pron", sense("him, to him; it; to it", ("dative", "masculine")))
MI_PRONOUN = entry(
    "mi",
    "pron",
    sense("clitic accusative of io. me", ("accusative", "clitic", "first-person", "form-of", "objective"), form_of=[("io. me", None)]),
    sense("clitic dative of io. (to) me", ("clitic", "dative", "first-person", "form-of", "objective"), form_of=[("io", "(to) me")]),
    sense("Used as ethical dative.", ("colloquial", "first-person", "objective")),
)
MI_NOTE = entry("mi", "noun", sense("the third note, mi"), sense("E (musical note or key)"))
MI_LETTER = entry("mi", "noun", sense("mu (Greek letter)", ("feminine", "invariable", "masculine")))
CHI = entry("chi", "pron", sense("who, whom, whoever"))
CHI_LETTER = entry("chi", "noun", sense("chi (Greek letter)", ("feminine", "invariable", "masculine")))
ETA = entry("eta", "noun", sense("eta (Greek letter)", ("feminine", "invariable", "masculine")))
QUANTI_DET = entry("quanti", "det", sense("masculine plural of quanto", ("form-of", "masculine", "plural"), form_of=[("quanto", None)]))
QUANTI_PRON = entry("quanti", "pron", sense("synonym of chi sg (“who”): those who", ("formal", "masculine", "plural", "uncommon")))
QUANTO = entry("quanto", "det", sense("how much, how many, however much, however many"))
PIL = entry("pil", "noun", sense("alternative form of PIL", ("alt-of", "alternative", "masculine"), alt_of=[("PIL", None)]))
PIL_ACRONYM = entry(
    "PIL",
    "noun",
    sense(
        "initialism of prodotto interno lordo (“GDP”)",
        ("abbreviation", "alt-of", "initialism", "masculine"),
        alt_of=[("prodotto interno lordo", "GDP")],
    ),
)
PRODOTTO_INTERNO_LORDO = entry("prodotto interno lordo", "noun", sense("gross domestic product", ("masculine",)))
NATALE_ADJ = entry("natale", "adj", sense("native (of one's birth)", ("by-personal-gender", "feminine", "masculine")))
NATALE_NOUN = entry("natale", "noun", sense("birthday", ("masculine",)), sense("birth", ("masculine",)))
NATALE_NAME = entry("Natale", "name", sense("Christmas", ("masculine",)), sense("a male given name, feminine equivalent Natalia", ("masculine",)))
IO = entry("io", "pron", sense("I (the first-person singular nominative pronoun)", ("first-person", "personal")))
IO_NOUN = entry("io", "noun", sense("ego, self", ("invariable", "masculine")))
IO_MOON = entry("Io", "name", sense("Io", ("Greek", "feminine")), sense("85 Io", ("feminine",)))
MENO = entry("meno", "adv", sense("less"))
MENO_DET = entry("meno", "det", sense("less, fewer"))
MENO_RIVER = entry("Meno", "name", sense("Main (German river)"))
PERFINO = entry("perfino", "adv", sense("synonym of persino"))
AVERE = entry(
    "avere",
    "verb",
    sense("to have", ("transitive",)),
    sense("to have (done something); See Category:Italian transitive verbs and Category:Italian reflexive verbs.", ("auxiliary",)),
)
LAVORO = entry("lavoro", "noun", sense("work (all senses)", ("masculine",)), sense("job, task", ("masculine",)))
MERCATO = entry("mercato", "noun", sense("market (in all senses)", ("masculine",)))
MIO = entry("mio", "det", sense("my; of mine (used attributively)"))
MIO_HONORIFIC = entry("Mio", "pron", sense("Same as entry above.", ("masculine",)))
COLLA_CONTRACTION = entry(
    "colla",
    "contraction",
    sense("contraction of con + la; with the", ("abbreviation", "alt-of", "contraction", "dated"), alt_of=[("con", "+ la; with the")]),
)
COLLA_NOUN = entry("colla", "noun", sense("glue or similar sticky material", ("feminine",)))
COGLI_CONTRACTION = entry(
    "cogli",
    "contraction",
    sense("contraction of con gli; with the", ("abbreviation", "alt-of", "contraction", "dated"), alt_of=[("con gli", "with the")]),
)
COGLI_VERB = entry(
    "cogli",
    "verb",
    sense("second-person singular present indicative", ("form-of", "indicative", "present", "second-person", "singular"), form_of=[("cogliere", None)]),
)
COGLIERE = entry("cogliere", "verb", sense("to pick, to gather"), sense("to grasp, to seize, to take"))
CUORI = entry(
    "cuori",
    "noun",
    sense("plural of cuore (when talking about the organ)", ("form-of", "masculine", "plural"), form_of=[("cuore", "when talking about the organ")]),
    sense("hearts (suit of playing cards)", ("masculine", "plural")),
)
SOMMI = entry(
    "sommi",
    "contraction",
    sense(
        "contraction of sono + mi: mi sono",
        ("Tuscany", "abbreviation", "alt-of", "contraction", "obsolete"),
        alt_of=[("sono", "+ mi: mi sono")],
    ),
)
ANCH_IO = entry(
    "anch'io",
    "contraction",
    sense("contraction of anche + io; me too (subject)", ("abbreviation", "alt-of", "contraction"), alt_of=[("anche", "+ io; me too (subject)")]),
)
UN_PO = entry("un po'", "adv", sense("a bit, a little"))
D_ACCORDO = entry("d'accordo", "adv", sense("agreed, OK, all right"))
C_E = entry("c'è", "verb", sense("third-person singular present indicative of esserci", ("form-of",), form_of=[("esserci", None)]))
CANCRO_NAME = entry("Cancro", "name", sense("Cancer (constellation)"))
CANCRO = entry("cancro", "noun", sense("cancer (disease)"))
USA = {
    "word": "USA",
    "pos": "name",
    "synonyms": [{"word": "America"}, {"word": "Stati Uniti"}],
    "senses": [sense("the United States of America", ("plural",))],
}
AMERICA = entry("America", "name", sense("America, the Americas"))
STATI_UNITI = entry("Stati Uniti", "name", sense("United States (a country in North America)"))
A_LETTER = entry("a", "character", sense("The first letter of the Italian alphabet."))
SECTION = [
    DEL, DELLA, ED, AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, GLI_ARTICLE, GLI_PRONOUN, MI_PRONOUN, MI_NOTE, MI_LETTER,
    CHI, CHI_LETTER, ETA, QUANTI_DET, QUANTI_PRON, QUANTO, PIL, PIL_ACRONYM, PRODOTTO_INTERNO_LORDO, NATALE_ADJ,
    NATALE_NOUN, NATALE_NAME, IO, IO_NOUN, IO_MOON, MENO, MENO_DET, MENO_RIVER, PERFINO, AVERE, LAVORO, MERCATO, MIO,
    MIO_HONORIFIC, COLLA_CONTRACTION, COLLA_NOUN, COGLI_CONTRACTION, COGLI_VERB, COGLIERE, CUORI, SOMMI, ANCH_IO,
    UN_PO, D_ACCORDO, C_E, CANCRO_NAME, CANCRO, USA, AMERICA, STATI_UNITI, A_LETTER,
]  # fmt: skip

# — Recorded entries of the Italian Wiktionary's English translations —

TRANSLATED = [
    {"word": "perfino", "pos": "adv", "translations": [{"word": "even"}]},
    {"word": "ex", "pos": "adj", "translations": [{"word": "ex"}, {"word": "former"}]},
    {"word": "ex", "pos": "noun", "translations": [{"word": "ex"}, {"word": "former"}]},
    {"word": "meccanica quantica", "pos": "noun", "translations": [{"word": "quantum mechanics"}]},
    {"word": "all’inizio", "pos": "adv", "translations": [{"word": "at the beginning"}]},
    {"word": "b", "pos": "noun", "translations": [{"word": "b"}]},
    {"word": "Roma", "pos": "name", "translations": [{"word": "Rome"}]},
]
# The English Wiktionary's English entries' Italian translations, which it-en never reads backwards (D2):
# made up, an English entry « etc. » listing `etc`.
INVERTED = [{"word": "etc.", "pos": "adv", "translations": [{"word": "etc"}]}]

# The lemmas the fixture ranks: the forms that are their own lemma, `gli` among them (made up: Italian's
# tables read it as a form of `il`), and the acronym rows, which are their own forms with no rank.
RANKED = [
    "del", "della", "ed", "ad", "gli", "mi", "chi", "eta", "quanti", "quanto", "pil", "natale", "io", "meno", "perfino",
    "avere", "lavoro", "mercato", "mio", "colla", "cogli", "cogliere", "cuori", "sommi", "cancro", "ex", "etc",
]  # fmt: skip


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
        return studied

    def read(self, *entries, rules=italian.RULES):
        src = self.jsonl(*entries)
        dst, stats = italian.read_as_italian(src, str(self.dir / "read.jsonl"), rules=rules)
        return [json.loads(line) for line in Path(dst).read_text(encoding="utf-8").splitlines()], stats

    def glosses(self, *entries, lemmas, rules=italian.RULES):
        src = self.jsonl(*entries)
        dst, _ = italian.read_as_italian(src, str(self.dir / "read.jsonl"), rules=rules)
        return common.reduce_gloss(dst, set(lemmas), **common.WORD_GLOSS, studied=it_en.IT, edition=english.EN)

    def reduce(self, section=SECTION, translated=TRANSLATED, ranked=RANKED):
        work = self.dir / "work"
        work.mkdir(exist_ok=True)
        studied = self.studied(ranked)
        self.jsonl(*section, name="work/kaikki-Italian.jsonl")
        self.jsonl(*translated, name="work/kaikki-it-traductions-en.jsonl")
        self.jsonl(*INVERTED, name="work/kaikki-en-traductions-it.jsonl")
        argv = [
            "reduce-it-en.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-25", "--pack-version", "test"),
        ]
        with mock.patch.object(sys, "argv", argv), contextlib.redirect_stderr(io.StringIO()) as err:
            it_en.main()
        self.report = err.getvalue()

        def table(name):
            rows = {}
            for line in (work / name).read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition("\t")
                rows[key] = value
            return rows

        return work, table("gloss.tsv"), table("senses.tsv"), table("mwe.tsv")


class CarriedMeanings(Fixture):
    """D4.1: the meaning a pointer carries, by a reviewed list of wordings."""

    def test_the_di_family_reads_its_contraction(self):
        self.assertEqual(italian.carried_meaning(DEL["senses"][0], "contraction"), "some, of the, from the")
        self.assertEqual(italian.carried_meaning(DELLA["senses"][0], "contraction"), "of the, from the")

    def test_italian_s_three_shapes(self):
        self.assertEqual(italian.carried_meaning(ED["senses"][0], "conj"), "and")
        self.assertEqual(italian.carried_meaning(AD["senses"][0], "prep"), "to, at, in")
        self.assertEqual(italian.carried_meaning(GLI_ARTICLE["senses"][0], "article"), "the")
        accusative, dative, ethical = MI_PRONOUN["senses"]
        self.assertEqual(italian.carried_meaning(accusative, "pron"), "me")
        self.assertEqual(italian.carried_meaning(dative, "pron"), "(to) me")
        self.assertIsNone(italian.carried_meaning(ethical, "pron"), "no pointer")
        # A case form is a pronoun's alone.
        self.assertIsNone(italian.carried_meaning(accusative, "noun"))
        # The shapes are read beside the word's other senses: they are its function word's own reading.
        self.assertEqual(italian.carried_meaning(ED["senses"][0], "conj", alone=False), "and")

    def test_never_synonym_nor_every_wording(self):
        # `quanti`'s synonym would read « who »; an abbreviation would read `giu` « June ».
        self.assertIsNone(italian.carried_meaning(QUANTI_PRON["senses"][0], "pron"), "untagged and unlisted")
        giu = sense("abbreviation of giugno (“June”)", ("abbreviation", "alt-of"), alt_of=[("giugno", "June")])
        self.assertIsNone(italian.carried_meaning(giu, "noun"))
        self.assertIsNone(italian.carried_meaning(QUANTI_DET["senses"][0], "det"), "« masculine plural of » carries nothing")

    def test_a_carrying_wording_only_where_the_word_holds_nothing_else(self):
        # The implementation's narrowing: a contraction or a plural never opens or joins a row its own
        # senses gloss, nor replaces a verb form's lent senses.
        self.assertEqual(italian.carried_meaning(COLLA_CONTRACTION["senses"][0], "contraction"), "with the")
        self.assertIsNone(italian.carried_meaning(COLLA_CONTRACTION["senses"][0], "contraction", alone=False))
        glosses = self.glosses(COLLA_CONTRACTION, COLLA_NOUN, COGLI_CONTRACTION, COGLI_VERB, COGLIERE, CUORI, lemmas={"colla", "cogli", "cuori"})
        self.assertEqual(glosses["colla"], "glue or similar sticky material")
        self.assertEqual(glosses["cogli"], "to pick, to gather; to grasp, to seize, to take")
        self.assertEqual(glosses["cuori"], "hearts (suit of playing cards)")

    def test_a_meaning_from_the_gloss_never_from_extra_s_notes(self):
        # Recorded: kaikki's `extra` drops the quotation marks and keeps a note as readily as a meaning.
        cuori = CUORI["senses"][0]
        self.assertIsNone(italian.carried_meaning(cuori, "noun"))
        guai = sense("plural of guaio (“trouble”)", ("form-of", "masculine", "plural"), form_of=[("guaio", "trouble")])
        self.assertEqual(italian.carried_meaning(guai, "noun"), "trouble")
        domi = sense("plural of domo (“dome, vault; sky”)", ("form-of", "literary", "plural"), form_of=[("domo", "dome, vault; sky")])
        self.assertEqual(italian.carried_meaning(domi, "noun"), "dome, vault; sky", "a semicolon inside the quotation is no separator")
        stabile = sense("superlative degree of stabile (“stable”): most stable", ("form-of", "superlative"), form_of=[("stabile", "(“stable”): most stable")])
        self.assertEqual(italian.carried_meaning(stabile, "adj"), "most stable")

    def test_a_meaning_made_of_the_pointer_s_own_words_is_none(self):
        self.assertIsNone(italian.carried_meaning(SOMMI["senses"][0], "contraction"), "« mi sono » is Italian")

    def test_spec_scenario_a_meaning_a_pointer_carries(self):
        glosses = self.glosses(DEL, DELLA, ED, AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"del", "della", "ed", "ad"})
        self.assertEqual(glosses["della"], "of the, from the")
        self.assertEqual(glosses["del"], "some, of the, from the")
        self.assertEqual(glosses["ed"], "and")
        self.assertEqual(glosses["ad"], "to, at, in")


class AcronymsAndNames(Fixture):
    """D4.2: acronyms off common words, names off function words."""

    def test_spec_scenario_an_acronym_off_a_common_word(self):
        # Without the rule, `ad` borrows « chief executive officer, CEO » through `AD`'s pointer.
        without = self.glosses(AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"ad"}, rules=())
        self.assertEqual(without["ad"], "chief executive officer, CEO")
        alone = self.glosses(AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"ad"}, rules=("acronyms and names",))
        self.assertNotIn("ad", alone)
        glosses = self.glosses(AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"ad"})
        self.assertEqual(glosses["ad"], "to, at, in")
        self.assertNotIn("CEO", glosses["ad"])

    def test_an_acronym_its_lowercase_entry_points_at_is_kept(self):
        glosses = self.glosses(PIL, PIL_ACRONYM, PRODOTTO_INTERNO_LORDO, lemmas={"pil"})
        self.assertEqual(glosses["pil"], "gross domestic product")

    def test_spec_scenario_a_name_beside_a_common_word_s_senses(self):
        glosses = self.glosses(NATALE_ADJ, NATALE_NOUN, NATALE_NAME, lemmas={"natale"})
        self.assertIn("Christmas", glosses["natale"].split("; "))

    def test_spec_scenario_a_name_off_a_function_word(self):
        glosses = self.glosses(IO, IO_NOUN, IO_MOON, MENO, MENO_DET, MENO_RIVER, lemmas={"io", "meno"})
        self.assertEqual(glosses["io"], "I (the first-person singular nominative pronoun); ego, self")
        self.assertEqual(glosses["meno"], "less; less, fewer")
        read, stats = self.read(IO, IO_NOUN, IO_MOON, MENO, MENO_DET, MENO_RIVER)
        self.assertEqual([e["word"] for e in read], ["io", "io", "meno", "meno"])
        self.assertEqual(stats["names under a function word"], 2)
        # An adverb's word keeps its name (made up): the rule reads function words alone.
        read, _ = self.read(MENO, MENO_RIVER)
        self.assertEqual([e["word"] for e in read], ["meno", "Meno"])


class LettersAndNotes(Fixture):
    """D4.3 and D4.4: no word glossed with a letter's name, and the page's notes out."""

    def test_spec_scenario_a_greek_letter(self):
        glosses = self.glosses(MI_PRONOUN, MI_NOTE, MI_LETTER, CHI, CHI_LETTER, ETA, lemmas={"mi", "chi", "eta"})
        self.assertEqual(glosses["mi"].split("; ")[0], "me")
        self.assertNotIn("Greek letter", glosses["mi"])
        self.assertNotIn("mu", glosses["mi"].split("; "))
        self.assertEqual(glosses["chi"], "who, whom, whoever")
        self.assertEqual(glosses["eta"], "eta (Greek letter)", "a word that is only the letter keeps it")

    def test_a_latin_letter_goes_with_the_english_edition_s_pre_passes(self):
        cut = it_en.without_letters(self.jsonl(A_LETTER, CHI), str(self.dir / "words.jsonl"))
        self.assertEqual([json.loads(line)["word"] for line in Path(cut).read_text().splitlines()], ["chi"])

    def test_the_page_s_notes_are_no_glosses(self):
        glosses = self.glosses(AVERE, LAVORO, MERCATO, MIO, MIO_HONORIFIC, lemmas={"avere", "lavoro", "mercato", "mio"})
        self.assertEqual(glosses["avere"], "to have; to have (done something)")
        self.assertEqual(glosses["lavoro"], "work; job, task")
        self.assertEqual(glosses["mercato"], "market")
        self.assertEqual(glosses["mio"], "my, of mine (used attributively)")
        self.assertIsNone(italian.without_notes("Same as entry above."))
        kept = "table (all senses, except for the furniture element)"
        self.assertEqual(italian.without_notes(kept), kept, "a note that says more stays")

    def test_each_rule_counts_what_it_moved(self):
        _, stats = self.read(*SECTION)
        self.assertEqual(stats["names under a function word"], 2)
        self.assertEqual(stats["acronym senses off a common word"], 1, "AD's; PIL's kept")
        self.assertEqual(stats["Greek letters beside a common word"], 2)
        self.assertEqual(stats["senses that are only a note"], 1)
        self.assertEqual(stats["notes taken out"], 3)
        # del, della, ed, ad, gli, mi twice, anch'io.
        self.assertEqual(stats["pointers read as their meaning"], 8)
        self.assertEqual(sum(stats.values()), 2 + 1 + 2 + 1 + 3 + 8 + stats["entries left with no sense"])

    def test_a_line_the_pre_pass_cannot_read_is_written_as_it_is(self):
        src = self.dir / "section.jsonl"
        src.write_text(json.dumps(DEL) + "\nnot json\n[1]\n", encoding="utf-8")
        dst, _ = italian.read_as_italian(str(src), str(self.dir / "read.jsonl"))
        self.assertEqual(Path(dst).read_text(encoding="utf-8").splitlines()[1:], ["not json", "[1]"])


class TheReducer(Fixture):
    """reduce-it-en.py over a work folder holding the two sources."""

    def test_the_native_side_alone_from_the_committed_studied_tables(self):
        work, glosses, senses, mwe = self.reduce()
        self.assertEqual(glosses["della"], "of the, from the")
        self.assertEqual(glosses["del"], "some, of the, from the")
        self.assertEqual(glosses["ed"], "and")
        self.assertEqual(glosses["ad"], "to, at, in")
        self.assertEqual(glosses["gli"], "the; him, to him, it, to it")
        self.assertEqual(senses["gli"], "DET:1\tPRON:1")
        self.assertEqual(glosses["mi"], "me; (to) me; used as ethical dative; the third note, mi; E (musical note or key)")
        self.assertEqual(glosses["quanti"], "how much, how many, however much, however many")
        self.assertEqual(glosses["pil"], "gross domestic product")
        self.assertEqual(glosses["eta"], "eta (Greek letter)")
        self.assertIn("Christmas", glosses["natale"])
        self.assertNotIn("Io", glosses["io"])
        self.assertEqual(glosses["cancro"], "Cancer (constellation); cancer (disease)", "case keeps a name's sense apart")
        # Nothing of the studied side is computed (D1); measures.json stays in the work folder.
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith("kaikki-"))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        self.assertNotIn("NATO", glosses, "a row with no expansion or synonym glossed, none")

    def test_spec_scenario_a_gloss_from_the_italian_wiktionary(self):
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["perfino"], "even")
        self.assertEqual(senses["perfino"], "ADV:1")
        measured = json.loads((self.dir / "work" / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(measured["by source"], {"definitions": 23, "direct": 2})
        self.assertEqual(measured["direct"], ["ex", "perfino"])

    def test_a_sense_repeated_word_for_word_is_shown_once(self):
        # Recorded: the Italian Wiktionary lists « ex, former » under the adjective and the noun.
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["ex"], "ex, former")
        self.assertEqual(senses["ex"], "ADJ:1")
        self.assertIn("1 glosses repeating a sense", self.report)

    def test_spec_scenario_no_table_read_backwards(self):
        # `etc` is ranked, the section and the Italian Wiktionary say nothing of it, and the work folder
        # holds the English entries' Italian translations: it-en reads none of them.
        _, glosses, _, _ = self.reduce()
        self.assertNotIn("etc", glosses)
        self.assertEqual(it_en.SECTION, "kaikki-Italian.jsonl")
        self.assertEqual(it_en.DIRECT, "kaikki-it-traductions-en.jsonl")
        self.assertEqual(ps.DUMPS["it-en"], {"en": ("kaikki-Italian.jsonl",), "it": ("kaikki-it-traductions-en.jsonl",)})

    def test_spec_scenario_expressions_from_every_source(self):
        _, _, _, mwe = self.reduce()
        self.assertEqual(mwe["un po'"], "a bit, a little")
        self.assertEqual(mwe["d'accordo"], "agreed, OK, all right")
        self.assertEqual(mwe["meccanica quantica"], "quantum mechanics")
        self.assertEqual(mwe["anch'io"], "me too (subject)", "a split word whose pointer carries its meaning")
        self.assertEqual(mwe["all'inizio"], "at the beginning", "the Italian Wiktionary's split word, its apostrophe straight")
        self.assertEqual(mwe["amministratore delegato"], "chief executive officer, CEO")
        self.assertNotIn("c'è", mwe, "a pointer only: its pieces gloss it")
        self.assertNotIn("stati uniti", mwe, "a proper noun's entry")

    def test_the_acronym_rows_glossed_through_their_synonyms(self):
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["USA"], "America, the Americas; United States (a country in North America)")
        self.assertEqual(senses["USA"], "PROPN:2")

    def test_the_credits_name_both_sides(self):
        work, *_ = self.reduce()
        notice = (work / "NOTICE").read_text(encoding="utf-8")
        for source in (
            "English Wiktionary (enwiktionary), Italian section",
            "Italian Wiktionary (itwiktionary)",
            "wordfreq",
            "UD Italian-MarkIT",
            "Morph-it!",
            "French Wiktionary (frwiktionary), Italian section",
            "The levels are estimated",
        ):
            self.assertIn(source, notice)
        self.assertNotIn("PUD", notice, "a gate only, never shipped")
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("it", "en"))
        self.assertTrue(manifest["meta"]["levels_estimated"])
        self.assertEqual(manifest["meta"]["pack_version"], "test")
        self.assertEqual(manifest["meta"]["analyzer_version"], it_fr.analyser_version())
        self.assertEqual(manifest["sources"], it_fr.manifest("x")["sources"])

    def test_a_reduction_without_its_sources_stops(self):
        with self.assertRaisesRegex(SystemExit, "kaikki-it-traductions-en.jsonl missing"):
            it_en.reduce(str(self.dir), str(self.studied()), 60000, "test", report=lambda line: None)

    def test_the_native_side_step_by_step_is_native_tables(self):
        section = self.jsonl(*SECTION)
        direct = it_en.read_translated(self.jsonl(*TRANSLATED, name="direct.jsonl"), str(self.dir / "direct-read.jsonl"))
        ranks = {w: i for i, w in enumerate(RANKED, start=1)}
        glosses, runs, expressions, steps = it_en.native_side(section, ranks, direct)
        want = common.native_tables(section, ranks, studied=it_en.IT, edition=english.EN, fallbacks=[(direct, list)])
        self.assertEqual((glosses, runs, expressions), want[:3])
        self.assertEqual(len(steps["definitions"]), want[3])
        self.assertEqual(steps["direct"], frozenset({"perfino", "ex"}))


class TheStudiedSide(Fixture):
    """D1: Italian's studied tables as committed, it-fr's."""

    def test_the_acronym_rows_are_no_ranked_lemmas(self):
        studied = self.studied(["casa", "nato"], extra_forms=[("case", "casa")])
        self.assertEqual(it_en.read_studied(str(studied), 60000), {"casa": 1, "nato": 2})
        self.assertEqual(it_en.read_studied(str(studied), 1), {"casa": 1}, "capped by rank")

    def test_tables_that_disagree_are_not_it_fr_s(self):
        studied = self.dir / "it"
        studied.mkdir()
        (studied / "forms.tsv").write_text("casa\tcasa\nUSA\tUSA\n", encoding="utf-8")
        (studied / "freq.tsv").write_text("casa\t1\ncane\t2\n", encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "disagree on the lemmas"):
            it_en.read_studied(str(studied), 60000)
        with self.assertRaisesRegex(SystemExit, "it-en reads Italian's committed tables"):
            it_en.read_studied(str(self.dir / "missing"), 60000)

    def test_the_committed_studied_tables(self):
        committed = it_en.read_studied(os.path.join(_HERE, "tables", "it"), 60000)
        self.assertEqual(len(committed), 60000)
        self.assertNotIn("USA", committed)

    def test_it_en_s_copy_of_italian_is_it_fr_s(self):
        # reduce-it-en.py and reduce_italian_en.py repeat it-fr's description of Italian because a reducer
        # loads no other pair's (D1): the two must not drift.
        self.assertEqual(it_en.IT, it_fr.IT)
        self.assertEqual(italian.TOKEN.pattern, it_fr._TOKEN.pattern)
        self.assertEqual(italian.ELIDED_PIECES, frozenset(it_fr.ELISIONS) | frozenset(it_fr.ELIDED_PREPOSITIONS))
        forms = {"aujourd'hui": "x", "nell'": "x"}
        for word in ("d'accordo", "all'improvviso", "c'è", "senz'altro", "alla", "un po'", "po'", "nell'", "trent'anni"):
            self.assertEqual(italian.split_word(word, forms), it_fr.split_word(word, forms), word)

    def test_a_translation_s_italian_word_is_read_as_italian_is_written(self):
        src = self.jsonl(*TRANSLATED, name="direct.jsonl")
        direct = it_en.read_translated(src, str(self.dir / "direct-read.jsonl"))
        self.assertIn("all'inizio", direct)
        self.assertNotIn("all’inizio", direct)
        self.assertNotIn("b", direct, "a letter's own translation says nothing")
        self.assertNotIn("roma", direct, "a proper noun's entry glosses nothing")

    def test_native_fields_cut_changes_no_table(self):
        full = [dict(e, forms=[{"form": "x", "tags": ["plural"]}], head_templates=[{"name": "it-noun"}]) for e in SECTION]
        cut, dropped = it_en.native_fields(self.jsonl(*full, name="full.jsonl"), str(self.dir / "cut.jsonl"))
        self.assertEqual(dropped, 0)
        ranks = {w: i for i, w in enumerate(RANKED, start=1)}

        def tables(path):
            return common.native_tables(path, ranks, studied=it_en.IT, edition=english.EN, fallbacks=[({}, list)])

        self.assertEqual(tables(cut), tables(self.jsonl(*SECTION)))


class TheRules(unittest.TestCase):
    """it-en's rule digest (D1, D4, D11)."""

    def test_it_en_s_rule_files(self):
        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-it-en.py")],
            ["reduce-it-en.py", "reduce_common.py", "reduce_edition_en.py", "reduce_italian_acronyms.py", "reduce_italian_en.py"],
        )

    def test_it_en_s_own_module_is_no_other_pair_s(self):
        for reducer in sorted(Path(_HERE).glob("reduce-*.py")):
            if reducer.name != "reduce-it-en.py":
                self.assertNotIn("reduce_italian_en.py", [p.name for p in ps.rule_files(reducer)], reducer.name)

    def test_spec_scenario_nothing_else_moves(self):
        # The English edition's shared rules and the shared module are not edited: es-en's and fr-en's
        # pins still record the rules they were reduced by, and so does every committed pair.
        for pin in sorted((Path(_HERE) / "tables").glob("*/pin.json")):
            reducer = Path(_HERE) / f"reduce-{pin.parent.name}.py"
            self.assertEqual(ps.rules_sha256(reducer), ps.get(ps.load(pin), "reducer.sha256"), pin.parent.name)


if __name__ == "__main__":
    unittest.main()
