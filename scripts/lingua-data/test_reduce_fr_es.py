# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for fr-es's native side (add-lingua-pack-fr-es): French glossed in Spanish.

The entries are recorded from the three files the prototype read — the Spanish Wiktionary's French
section (`kaikki-es-Frances.jsonl`) and the French translations its Spanish entries list
(`kaikki-es-traductions.jsonl`), derived from its dump regenerated 2026-10-02 12:12, and the Spanish
translations the French Wiktionary's French entries list (`kaikki-fr-traductions.jsonl`), from its
dump regenerated 2026-10-02 00:10 — shaped as the dumps write them, cut down to the fields the rules
read. À_SECTION keeps its entry's first senses only. A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
import contextlib
import importlib.util
import inspect
import io
import json
import os
import re
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import reduce_common as common  # noqa: E402
import reduce_edition_es as spanish  # noqa: E402
from reduce_edition_es import ES  # noqa: E402


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(
        f"reduce_{pair.replace('-', '_')}_fr_es_tests", os.path.join(_HERE, f"reduce-{pair}.py")
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


fr_es = _reducer("fr-es")
fr_en = _reducer("fr-en")

# — The Spanish Wiktionary's French section (kaikki-es-Frances.jsonl) —

MAISON = {"word": "maison", "pos": "noun", "tags": ["feminine"], "senses": [{"glosses": ["Casa."]}]}
ET = {"word": "et", "pos": "conj", "senses": [{"glosses": ["Et."]}]}
VENIR = {"word": "venir", "pos": "verb", "tags": ["intransitive"], "senses": [{"glosses": ["Venir."]}]}
TROLL = {"word": "troll", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Troll."]}]}
ELITE = {"word": "élite", "pos": "noun", "tags": ["feminine"], "senses": [{"glosses": ["Élite."]}]}
PIERRE = {"word": "pierre", "pos": "noun", "tags": ["feminine"], "senses": [{"glosses": ["Piedra."]}]}
PIERRE_VERB = {
    "word": "pierre",
    "pos": "verb",
    "tags": ["form-of"],
    "senses": [
        {
            "glosses": ["Primera persona del singular (je) del presente de indicativo de pierrer."],
            "tags": ["form-of"],
            "form_of": [{"word": "pierrer"}],
        },
        {
            "glosses": ["Tercera persona del singular (elle, on, il) del presente de indicativo de pierrer."],
            "tags": ["form-of"],
            "form_of": [{"word": "pierrer"}],
        },
    ],
}
PIERRE_NAME = {
    "word": "Pierre",
    "pos": "name",
    "senses": [{"glosses": ["Nombre de pila de varón, equivalente del español Pedro."]}],
}
JEAN_ADJ = {"word": "jean", "pos": "adj", "senses": [{"glosses": ["Color de mezclilla."]}]}
JEAN_PHRASE = {
    "word": "jean",
    "pos": "phrase",
    "tags": ["substantive"],
    "senses": [{"glosses": ["Mezclilla."]}, {"glosses": ["Tejanos."], "raw_tags": ["Vestimenta"]}],
}
JEAN_NAME = {"word": "Jean", "pos": "name", "senses": [{"glosses": ["Nombre de pila de varón, equivalente del español Juan"]}]}
FRANCOIS_NAME = {
    "word": "François",
    "pos": "name",
    "senses": [{"glosses": ["Nombre de pila de varón, equivalente del español Francisco"]}],
}
MON = {"word": "mon", "pos": "adj", "tags": ["irregular", "possessive"], "senses": [{"glosses": ["Mi."]}]}
MES = {
    "word": "mes",
    "pos": "adj",
    "tags": ["adjectival", "form-of"],
    "senses": [
        {"glosses": ["Forma del masculino plural de mon."], "tags": ["form-of"], "form_of": [{"word": "mon"}]},
        {"glosses": ["Forma del femenino plural de mon."], "tags": ["form-of"], "form_of": [{"word": "mon"}]},
    ],
}
MA = {
    "word": "ma",
    "pos": "adj",
    "tags": ["adjectival", "form-of"],
    "senses": [{"glosses": ["Forma del femenino singular de mon; mi."], "tags": ["form-of"], "form_of": [{"word": "mon"}]}],
}
CE_ADJ = {
    "word": "ce",
    "pos": "adj",
    "tags": ["demonstrative", "irregular"],
    "senses": [
        {
            "glosses": ["Este."],
            "raw_tags": ["toma la forma cet delante de un sustantivo masculino que empieza en una vocal o una h muda"],
        }
    ],
}
CE_PRON = {
    "word": "ce",
    "pos": "pron",
    "tags": ["demonstrative", "neuter"],
    "senses": [
        {
            "glosses": ["Esto, eso, aquello."],
            "raw_tags": ["se usa solamente como sujeto del verbo être o como antecedente de un pronombre relativo"],
        }
    ],
}
SON_PRON = {"word": "son", "pos": "pron", "tags": ["possessive"], "senses": [{"glosses": ["Su."]}]}
SON_NOUN = {"word": "son", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Sonido."]}]}
CHAPELET = {
    "word": "chapelet",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [{"glosses": ["Guirnalda."], "raw_tags": ["obsoleta"]}, {"glosses": ["Rosario."]}],
}
H_SECTION = {
    "word": "h",
    "pos": "character",
    "senses": [
        {
            "glosses": ["Octava letra y sexta consonante del alfabeto francés. Se llama ache."],
            "tags": ["lower case"],
            "raw_tags": ["puede ser muda, como en español, o sonar como [h], como en inglés"],
        }
    ],
}
X_SECTION = {
    "word": "x",
    "pos": "character",
    "senses": [{"glosses": ["Vigesimocuarta letra y decimonovena consonante del alfabeto francés."], "tags": ["lower case"]}],
}
A_SECTION = {
    "word": "à",
    "pos": "prep",
    "senses": [{"glosses": ["A (un lugar)."]}, {"glosses": ["A, hasta (un tiempo)."]}, {"glosses": ["A (una dirección)."]}],
}
Y_CHARACTER = {
    "word": "y",
    "pos": "character",
    "senses": [{"glosses": ["Vigesimoquinta letra y vigésima consonante del alfabeto francés."], "tags": ["lower case"]}],
}
Y_ADV = {"word": "y", "pos": "adv", "tags": ["place"], "senses": [{"glosses": ["Allí, ahí."]}]}
# Made up: a headword with a typographic apostrophe, as French text and the French Wiktionary write
# it — the section writes 6 (« prud’homme »).
PRUDHOMME = {"word": "prud’homme", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Juez laboral."]}]}

# — The French Wiktionary's Spanish translations (kaikki-fr-traductions.jsonl), the direct table —

MAISON_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"sense": "Bâtiment servant de logis, d’habitation, de demeure", "word": "casa"},
        {"sense": "Foyer", "word": "casa"},
        {"sense": "Lignée, famille", "word": "linaje"},
        {"sense": "Entreprise, commerce", "word": "casa"},
    ],
    "word": "maison",
}
INTERET_TRANSLATED = {"pos": "noun", "translations": [{"sense": "Ce qui importe", "word": "interés"}], "word": "intérêt"}
ET_TRANSLATED = {
    "pos": "conj",
    "translations": [
        {"sense": "Conjonction", "word": "y"},
        {"sense": "Conjonction", "word": "e"},
        {"sense": "Emphatique en début de phrase", "word": "y"},
    ],
    "word": "et",
}
VENIR_TRANSLATED = {"pos": "verb", "translations": [{"sense": "Survenir, arriver", "word": "venir"}], "word": "venir"}
TROLL_TRANSLATED = {"pos": "noun", "translations": [{"word": "trol"}], "word": "troll"}
ELITE_TRANSLATED = {"pos": "noun", "translations": [{"word": "elite"}], "word": "élite"}
PIERRE_TRANSLATED = {"pos": "noun", "translations": [{"sense": "roche", "word": "piedra"}], "word": "pierre"}
A_TRANSLATED = {
    "pos": "prep",
    "translations": [
        {"sense": "Complément circonstanciel de lieu avec déplacement", "word": "a"},
        {"sense": "Complément circonstanciel de lieu avec déplacement", "word": "en"},
        {"sense": "Complément circonstanciel de lieu sans déplacement", "word": "en"},
        {"sense": "Complément circonstanciel de provenance", "word": "de"},
        {"word": "sobre"},
    ],
    "word": "à",
}
Y_TRANSLATED = {
    "pos": "adv",
    "translations": [{"sense": "À cet endroit", "word": "ahí"}, {"sense": "À cet endroit", "word": "allí"}],
    "word": "y",
}
O_TRANSLATED = {"pos": "particle", "translations": [{"word": "oh"}], "word": "ô"}
I_TRANSLATED = {
    "pos": "character",
    "translations": [{"sense": "Nom de la lettre I, i", "word": "i latina"}, {"sense": "Nom de la lettre I, i", "word": "i"}],
    "word": "i",
}
MAIN_D_OEUVRE_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"sense": "Travail de l’ouvrier considéré surtout par rapport au prix.", "word": "mano de obra"},
        {"sense": "L’ensemble des ouvriers de tel ou tel métier.", "word": "mano de obra"},
    ],
    "word": "main-d’œuvre",
}
ALLER_DE_L_AVANT_TRANSLATED = {"pos": "verb", "translations": [{"word": "echar para adelante"}], "word": "aller de l’avant"}
PARTI_ADJ_TRANSLATED = {"pos": "adj", "translations": [{"sense": "Héraldique", "word": "partido"}], "word": "parti"}
PARTI_NOUN_TRANSLATED = {"pos": "noun", "translations": [{"sense": "parti politique", "word": "partido"}], "word": "parti"}

# — The Spanish Wiktionary's French translations (kaikki-es-traductions.jsonl), read backwards —

TRAVES_LISTS = {"pos": "noun", "translations": [{"word": "travers"}], "word": "través"}
ESTE_ADJ_LISTS = {
    "pos": "adj",
    "translations": [{"word": "ce"}, {"word": "cet"}, {"word": "cette"}, {"word": "celui-ci"}, {"word": "celle-ci"}],
    "word": "este",
}
ESTE_PRON_LISTS = {
    "pos": "pron",
    "translations": [{"word": "ce"}, {"word": "cet"}, {"word": "cette"}, {"word": "celui-ci"}, {"word": "celle-ci"}],
    "word": "este",
}
H_LISTS = {"pos": "character", "translations": [{"word": "h"}], "word": "h"}
X_LISTS = {"pos": "character", "translations": [{"word": "x"}], "word": "x"}
I_LISTS = {"pos": "noun", "translations": [{"sense": "letra", "word": "i"}], "word": "i"}
Y_LISTS_ET = {"pos": "conj", "translations": [{"word": "et"}], "word": "y"}
CASA_LISTS = {"pos": "noun", "translations": [{"word": "maison"}], "word": "casa"}
GOLPE_MILITAR_LISTS = {"pos": "phrase", "translations": [{"word": "coup d’État"}], "word": "golpe militar"}
ADIESTRADOR_ADJ_LISTS = {"pos": "adj", "translations": [{"word": "entraîneur"}, {"word": "dresseur"}], "word": "adiestrador"}
ADIESTRADOR_NOUN_LISTS = {"pos": "noun", "translations": [{"word": "entraîneur"}, {"word": "dresseur"}], "word": "adiestrador"}
CONDOM_LISTS = [
    {"pos": "adj", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "preservativo"},
    {"pos": "noun", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "preservativo"},
    {"pos": "noun", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "condón"},
    {"pos": "adj", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "profiláctico"},
    {"pos": "noun", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "profiláctico"},
]
# French's readings of those words, as tables/fr/grammar.tsv commits them.
CONDOM_ENTRAINEUR_GRAMMAR = (
    "condom\tcondom\tNOUN|Gender=Masc|Number=Sing\t-\n"
    "condoms\tcondom\tNOUN|Gender=Masc|Number=Plur\t-\n"
    "entraîneur\tentraîneur\tNOUN|Gender=Masc|Number=Sing\t-\n"
)

# wordfreq's Spanish Zipf frequency, as the reducer reads it (recorded for the words read here).
SPANISH_ZIPF = {"preservativo": 3.3, "profiláctico": 1.8, "condón": 3.2, "adiestrador": 1.9, "través": 4.9}


def spanish_zipf(word):
    return SPANISH_ZIPF.get(word, 0.0)

# — refine-lingua-fr-es-glosses: the entries its rules read, recorded from the same three files —

_FIGURATIVE_VULGAR = {
    "categories": ["FR:Términos en sentido figurado", "FR:Términos malsonantes"],
    "tags": ["figurative", "vulgar"],
}
BAISER_TRANSITIVE = {
    "word": "baiser",
    "pos": "verb",
    "tags": ["transitive"],
    "senses": [
        {
            "glosses": ["Besar."],
            "categories": ["FR:América", "FR:Bélgica", "FR:Canadá", "FR:Términos anticuados"],
            "tags": ["Canada", "outdated"],
        },
        {"glosses": ["Coger (sexualmente)."], **_FIGURATIVE_VULGAR},
        {"glosses": ["Dominar o joder."], **_FIGURATIVE_VULGAR},
        {"glosses": ["Quebrar o romper."], **_FIGURATIVE_VULGAR},
        {
            "glosses": ["Grapar."],
            "categories": ["FR:Términos en sentido figurado", "FR:Términos jergales"],
            "tags": ["figurative", "slang"],
        },
    ],
}
BAISER_INTRANSITIVE = {
    "word": "baiser",
    "pos": "verb",
    "tags": ["intransitive"],
    "senses": [{"glosses": ["Culear, follar, fornicar, joder o realizar el coito."], **_FIGURATIVE_VULGAR}],
}
BAISER_NOUN = {"word": "baiser", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Beso, besuqueo u ósculo."]}]}
OUI_INTJ = {
    "word": "oui",
    "pos": "intj",
    "senses": [
        {"glosses": ["Afirmativo, así es, bien, ciertamente, correcto."]},
        {
            "glosses": ["Ciertamente o en verdad (contra preguntas negativas)."],
            "categories": ["FR:América", "FR:Quebec"],
            "tags": ["Quebec"],
        },
        {"glosses": ["¿Sí?"]},
    ],
}
AVEC_ADV = {
    "word": "avec",
    "pos": "adv",
    "senses": [
        {
            "glosses": ["También."],
            "categories": ["FR:América", "FR:Bélgica", "FR:Canadá", "FR:Quebec"],
            "tags": ["Canada", "Quebec"],
            "raw_tags": ["conjuntivo"],
        },
        {"glosses": ["Con este, consigo, con ella etc."]},
    ],
}
_MALSONANTE = {"categories": ["FR:Anatomía", "FR:Términos malsonantes"], "tags": ["vulgar"]}
CUL_NOUN = {
    "word": "cul",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [
        {"glosses": ["Culo."], **_MALSONANTE},
        {"glosses": ["Ano, ojete, recto (culo)."], **_MALSONANTE},
        {"glosses": ["Coito, sexo."], **_FIGURATIVE_VULGAR},
        {
            "glosses": ["Pornografía."],
            "categories": ["FR:Términos jergales", "FR:Términos malsonantes"],
            "tags": ["slang", "vulgar"],
        },
        {"glosses": ["Fundo (de un objeto)."], "categories": ["FR:Términos en sentido figurado"], "tags": ["figurative"]},
        {
            "glosses": ["Suerte propicia (ojete)."],
            "categories": ["FR:Europa", "FR:Francia", "FR:Términos jergales", "FR:Términos malsonantes"],
            "tags": ["France", "slang", "vulgar"],
        },
    ],
}
CUL_ADJ = {"word": "cul", "pos": "adj", "senses": [{"glosses": ["Estúpido."], "categories": ["FR:Términos jergales"], "tags": ["slang"]}]}
# An editor's template that named the wrong language: « ES:Términos anticuados ».
MAITRESSE = {
    "word": "maîtresse",
    "pos": "noun",
    "tags": ["feminine"],
    "senses": [{"glosses": ["Amante (femenina)."], "categories": ["ES:Términos anticuados"], "tags": ["outdated"]}],
}
RIEN_PRON = {
    "word": "rien",
    "pos": "pron",
    "tags": ["indefinite", "neuter"],
    "senses": [
        {"glosses": ["Nada."], "raw_tags": ["en construcciones negativas"]},
        {"glosses": ["Algo."], "categories": ["FR:Términos obsoletos"], "tags": ["obsolete"]},
        {"glosses": ["Poca cosa."]},
    ],
}
RIEN_NOUN = {"word": "rien", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Pequeño cantidad de algo."], "sense_index": "1"}]}
RIEN_ADV = {
    "word": "rien",
    "pos": "adv",
    "tags": ["quantitative"],
    "senses": [
        {
            "glosses": ["Muy."],
            "categories": ["FR:Términos coloquiales", "FR:Términos irónicos"],
            "tags": ["colloquial", "ironic"],
            "raw_tags": ["antífrasis"],
        },
        {"glosses": ["Mucha, muchas, mucho o muchos."]},
    ],
}
MAL_AUX_CHEVEUX = {
    "word": "mal aux cheveux",
    "pos": "phrase",
    "tags": ["substantive"],
    "senses": [
        {
            "glosses": ["Resaca, caña, chaqui, chuchaqui, cruda, goma, guayabo, hachazo, hangover, perseguidora, ratón."],
            "categories": ["FR:Términos anticuados"],
            "tags": ["outdated"],
        }
    ],
}
_VULGAR = {"categories": ["FR:Términos vulgares"], "tags": ["vulgar"]}
_DESPECTIVO = {
    "categories": ["FR:Términos despectivos", "FR:Términos en sentido figurado", "FR:Términos malsonantes"],
    "tags": ["derogatory", "figurative", "vulgar"],
}
FILS_DE_PUTE = {
    "word": "fils de pute",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [
        {"glosses": ["Hijo de puta, hijoputa o máncer."], **_VULGAR},
        {
            "glosses": [
                "Bastardo, hijo de la chingada, hijo de la Malinche, hijo de la tiznada, hijo de perra, hijo de puta, "
                "hijoputa o hijueputa."
            ],
            **_DESPECTIVO,
        },
        {"glosses": ["Objeto bajo, desgraciado o despreciable o situación incómoda."], **_DESPECTIVO},
        {"glosses": ["Chingado, desgraciado, maldito, pinche o puto."], **_VULGAR},
    ],
}
ETRE_NOUN = {
    "word": "être",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [{"glosses": ["Ser."], "raw_tags": ["Hace referencia a cualquier ser vivo."]}],
}
ETRE_VERB = {"word": "être", "pos": "verb", "tags": ["intransitive"], "senses": [{"glosses": ["Ser."]}, {"glosses": ["Estar."]}]}
ETRE_AUXILIARY = {
    "word": "être",
    "pos": "verb",
    "tags": ["auxiliary"],
    "senses": [{"glosses": ["(être + participio) Haber."]}, {"glosses": ["(être + participio) ser."]}],
}
DEVOIR_NOUN = {"word": "devoir", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Deber."]}]}
DEVOIR_VERB = {"word": "devoir", "pos": "verb", "tags": ["transitive"], "senses": [{"glosses": ["Deber."]}]}
JAUNE_ADJ = {"word": "jaune", "pos": "adj", "senses": [{"glosses": ["Amarillo."], "categories": ["FR:Colores"]}]}
JAUNE_NOUN = {
    "word": "jaune",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [{"glosses": ["Amarillo."], "categories": ["FR:Colores"]}, {"glosses": ["Amarillo (persona asiática)."]}],
}
DES_ARTICLE = {"word": "des", "pos": "article", "tags": ["indeterminate"], "senses": [{"glosses": ["Algunos, algunas, unos o unas."]}]}
DES_CONTRACTION = {
    "word": "des",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [
        {
            "glosses": ["Contracción de la preposición de y el artículo les; de las o de los."],
            "categories": ["FR:Contracciones", "FR:Contracciones de preposiciones"],
        }
    ],
}
DU_CONTRACTION = {
    "word": "du",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ["Contracción de la preposición de y el articulo le; del."]}],
}
DUQUEL_CONTRACTION = {
    "word": "duquel",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ["Contracción de la preposición de y el pronombre lequel; de quién; de cuál."]}],
}
C_EST = {
    "word": "c'est",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ["Contracción de el pronombre ce y el verbo est."]}],
}
QUI_INTERROGATIVE = {"word": "qui", "pos": "pron", "tags": ["interrogative"], "senses": [{"glosses": ["Quién. (Pronombre nominativo.)"]}]}
QUI_RELATIVE = {"word": "qui", "pos": "pron", "tags": ["relative"], "senses": [{"glosses": ["Que. (Pronombre nominativo.)"]}]}
QUOI_INTERROGATIVE = {
    "word": "quoi",
    "pos": "pron",
    "tags": ["interrogative", "neuter"],
    "senses": [{"glosses": ["Qué. (Pronombre oblicuo.)"]}],
}
NOUS_AUTRES = {
    "word": "nous autres",
    "pos": "pron",
    "tags": ["personal"],
    "senses": [{"glosses": ["Nosotros [y no tú o vosotros]. (Plural exclusivo.)"]}],
}
IL_NEUTER = {
    "word": "il",
    "pos": "pron",
    "tags": ["neuter", "personal"],
    "senses": [{"glosses": ["Pronombre sujeto expletivo impersonal. (No tiene traducción al español. No existe en español.)"]}],
}
COCHON = {
    "word": "cochon",
    "pos": "noun",
    "tags": ["feminine", "masculine"],
    "senses": [{"glosses": ["Cerdo, marrano, guarro, cochino, etc."], "categories": ["FR:Ganadería", "FR:Mamíferos"]}],
}
PAS_NOUN = {"word": "pas", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Paso."]}]}
PAS_ADV = {"word": "pas", "pos": "adv", "tags": ["negative"], "senses": [{"glosses": ["No."]}]}
PENDANT_ADJ = {"word": "pendant", "pos": "adj", "senses": [{"glosses": ["Pendiente."]}]}
PENDANT_NOUN = {
    "word": "pendant",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [{"glosses": ["Pendiente."]}, {"glosses": ["Juego (cosas relacionadas entre si)."]}],
}
PENDANT_PREP = {"word": "pendant", "pos": "prep", "senses": [{"glosses": ["Durante."]}]}
IL_Y_A = {"word": "il y a", "pos": "phrase", "tags": ["prepositional"], "senses": [{"glosses": ["Hace."], "sense_index": "1"}]}
AMIE_ADJ = {
    "word": "amie",
    "pos": "adj",
    "tags": ["adjectival", "form-of"],
    "senses": [
        {
            "glosses": ["Forma del femenino singular de ami."],
            "categories": ["FR:Formas adjetivas en femenino"],
            "tags": ["form-of"],
            "form_of": [{"word": "ami"}],
        }
    ],
}
AMIE_NOUN = {
    "word": "amie",
    "pos": "noun",
    "tags": ["feminine"],
    "senses": [{"glosses": ["Amia o lamia."], "categories": ["FR:Peces"], "topics": ["fish"]}],
}
EL_PRON = {"word": "el", "pos": "pron", "tags": ["personal"], "senses": [{"glosses": ["Ella, ello o él."]}]}

RUSSE_ADJ_TRANSLATED = {"pos": "adj", "translations": [{"word": "ruso"}], "word": "russe"}
RUSSE_NOUN_TRANSLATED = {"pos": "noun", "translations": [{"word": "ruso"}, {"word": "rusa"}], "word": "russe"}
CLAIR_ADJ_TRANSLATED = {
    "pos": "adj",
    "translations": [
        {"word": w}
        for w in ("claro", "luminoso", "claro", "luminoso", "claro", "límpido", "transparente", "claro", "brillante")
    ],
    "word": "clair",
}
CLAIR_ADV_TRANSLATED = {"pos": "adv", "translations": [{"word": "claro"}, {"word": "claramente"}], "word": "clair"}
NET_TRANSLATED = [
    {
        "pos": "adj",
        "translations": [{"word": w} for w in ("puro", "limpio", "puro", "neto", "puro", "claro", "puro", "nítido", "claro")],
        "word": "net",
    },
    {"pos": "adv", "translations": [{"word": "claro"}], "word": "net"},
    {"pos": "noun", "translations": [{"word": "red"}], "word": "net"},
    {"pos": "adj", "translations": [{"word": "red"}], "word": "net"},
]
ARNAQUE_TRANSLATED = {"pos": "noun", "translations": [{"word": "arnaque"}], "word": "arnaque"}
RETRAITE_TRANSLATED = {
    "pos": "noun",
    "translations": [{"word": w} for w in ("retraite", "jubilación", "retiro", "jubilación", "pensión", "retiro")],
    "word": "retraite",
}
DIAPORAMA_TRANSLATED = {"pos": "noun", "translations": [{"word": "diaporama"}], "word": "diaporama"}
CLUB_TRANSLATED = {"pos": "noun", "translations": [{"word": "club"}], "word": "club"}
CONTROLE_CONTINU_TRANSLATED = {"pos": "noun", "translations": [{"word": "contrôle continu"}], "word": "contrôle continu"}
DS_TRANSLATED = {"pos": "noun", "translations": [{"word": "tiburón"}], "word": "DS"}
HALL_TRANSLATED = {"pos": "noun", "translations": [{"word": "explanada"}], "word": "hall"}
EL_TRANSLATED = {"pos": "pron", "translations": [{"sense": "Pronom neutre", "word": "elle"}], "word": "el"}

EEUU_LISTS = {
    "pos": "abbrev",
    "translations": [{"word": "USA"}, {"word": "US"}, {"word": "É.-U."}, {"word": "ÉU"}],
    "word": "EEUU",
}
ONU_LISTS = {"pos": "abbrev", "translations": [{"word": "ONU"}], "word": "ONU"}
AEC_LISTS = {"pos": "abbrev", "translations": [{"word": "AEC"}, {"word": "av. è. c."}], "word": "a. e. c."}
CALABAZA_LISTS = {
    "pos": "noun",
    "translations": [{"word": w} for w in ("fr", "citrouille", "fr", "potiron", "fr", "courge")],
    "word": "calabaza",
}
SAN_LUCAS_LISTS = {"pos": "phrase", "translations": [{"word": "Luc"}, {"word": "Lucas"}], "word": "San Lucas"}
PASCUA_LISTS = {"pos": "noun", "translations": [{"word": "Pâques"}], "word": "Pascua"}
SANTO_TOME_LISTS = {"pos": "phrase", "translations": [{"word": "Sao Tomé-et-Principe"}], "word": "Santo Tomé y Príncipe"}
ORIENTE_LISTS = {"pos": "noun", "translations": [{"word": "lOrient"}], "word": "Oriente"}
SECUESTRO_LISTS = {
    "pos": "noun",
    "translations": [{"word": w} for w in ("enlèvement", "kidnapping", "rap", "Détournement illégal de véhicule", "séquestration")],
    "word": "secuestro",
}
MAR_ROJO_LISTS = {"pos": "phrase", "translations": [{"word": "mer Rouge"}], "word": "mar Rojo"}

# wordfreq's Zipf frequency of the words the French-word rule reads, French then Spanish.
ZIPF = {
    "arnaque": (3.93, 0.0),
    "retraite": (4.8, 1.23),
    "diaporama": (3.1, 0.0),
    "club": (5.17, 4.98),
    "contrôle continu": (4.08, 0.0),
}


def zipf(word, language):
    french, spanish_ = ZIPF.get(word, (0.0, SPANISH_ZIPF.get(word, 0.0)))
    return french if language == "fr" else spanish_


# UD French-GSD's counts of these words by part of speech (`treebank.gsd_pos_counts` over the two
# sections fr-en's pin records).
TREEBANK = collections.Counter(
    {
        ("pas", "ADV"): 981,
        ("pas", "NOUN"): 8,
        ("pas", "ADP"): 1,
        ("pendant", "ADP"): 196,
        ("pendant", "NOUN"): 3,
        ("pendant", "ADJ"): 1,
        ("jaune", "ADJ"): 16,
        ("jaune", "NOUN"): 4,
        ("jaune", "PROPN"): 1,
        ("devoir", "NOUN"): 9,
        ("devoir", "VERB"): 351,
        ("des", "DET"): 1730,
        ("des", "ADP"): 3,
    }
)
# French's readings of these words, as tables/fr/grammar.tsv commits them.
READINGS = {"être": {"NOUN", "VERB"}, "devoir": {"NOUN", "VERB"}, "jaune": {"ADJ", "NOUN"}}


class Tables(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def jsonl(self, *entries, name="entries.jsonl"):
        path = self.dir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        return str(path)

    def section(self, *entries, readings=None, counts=None, fired=None):
        """The section through fr-es's passes, in `main`'s order: the file the shared rules read.
        `readings` French's parts of speech by lemma, `counts` UD French-GSD's (none by default),
        `fired` the corrections that fire."""
        src = self.jsonl(*entries, name="section.jsonl")
        src = fr_es.straight_apostrophes(src, str(self.dir / "section-apostrophes.jsonl"))
        src = fr_es.corrected_section(src, str(self.dir / "section-corrected.jsonl"), fired)
        src = common.without_letter_senses(src, str(self.dir / "section-words.jsonl"), edition=ES)
        src = spanish.read_as_meanings(src, str(self.dir / "section-meanings.jsonl"))
        src = fr_es.french_entries(src, str(self.dir / "section-glossing.jsonl"), readings or {})
        src = fr_es.treebank_order(src, str(self.dir / "section-treebank.jsonl"), counts or collections.Counter())
        return fr_es.with_labels(src, str(self.dir / "section-labels.jsonl"))

    def direct(self, *entries, fired=None):
        src = self.jsonl(*entries, name="direct.jsonl")
        return fr_es.read_translated(
            src, str(self.dir / "direct-words.jsonl"), inverted=False, readings={}, zipf=zipf, fired=fired
        )

    def inverted(self, *entries, readings=None):
        src = self.jsonl(*entries, name="inverted.jsonl")
        return fr_es.read_translated(src, str(self.dir / "inverted-words.jsonl"), inverted=True, readings=readings or {})

    def tables(self, lemmas, *section, direct=(), inverted=(), readings=None, counts=None, fired=None):
        """fr-es's native side over `lemmas`, ranked in their order: `(glosses, runs, expressions,
        steps)`."""
        ranks = {lemma: rank for rank, lemma in enumerate(lemmas, 1)}
        sources = [
            (self.direct(*direct, fired=fired), list),
            (self.inverted(*inverted, readings=readings), fr_es.by_spanish_frequency(spanish_zipf)),
        ]
        return fr_es.native_side(self.section(*section, readings=readings, counts=counts, fired=fired), ranks, sources)


class FrenchGlossedInSpanish(Tables):
    """*French is glossed in Spanish from the Spanish Wiktionary's French section and the French
    Wiktionary's translation tables*, scenario by scenario."""

    def test_spec_scenario_a_definition_first(self):
        # `maison` has a definition: it wins over the direct table's « casa, linaje » and the inverted
        # table's `casa`.
        glosses, runs, _, steps = self.tables(
            ["maison"], MAISON, direct=[MAISON_TRANSLATED], inverted=[CASA_LISTS]
        )
        self.assertEqual((glosses["maison"], runs["maison"]), ("Casa", [("NOUN", 1)]))
        self.assertEqual(steps["entries"], {"maison"})

    def test_spec_scenario_a_direct_table_gloss(self):
        # No definition of `intérêt`: the French Wiktionary's Spanish word, opening on a capital as
        # the edition's senses do. At most three Spanish words per part of speech, in the table's
        # order, each once: `à` read with no definition (made up — the section defines it).
        glosses, runs, _, steps = self.tables(["intérêt", "à"], direct=[INTERET_TRANSLATED, A_TRANSLATED])
        self.assertEqual((glosses["intérêt"], runs["intérêt"]), ("Interés", [("NOUN", 1)]))
        self.assertEqual((glosses["à"], runs["à"]), ("A, en, de", [("ADP", 1)]))
        self.assertEqual(steps["direct"], {"intérêt", "à"})

    def test_spec_scenario_an_inverted_table_gloss(self):
        glosses, runs, _, steps = self.tables(["travers"], inverted=[TRAVES_LISTS])
        self.assertEqual((glosses["travers"], runs["travers"]), ("Través", [("NOUN", 1)]))
        self.assertEqual(steps["inverted"], {"travers"})

    def test_spec_scenario_the_studied_word_is_no_gloss(self):
        # The section defines the conjunction `et` « Et. »: the French word itself. The French
        # Wiktionary's Spanish words gloss it, and the lemma is the direct table's.
        glosses, runs, _, steps = self.tables(["et"], ET, direct=[ET_TRANSLATED], inverted=[Y_LISTS_ET])
        self.assertEqual((glosses["et"], runs["et"]), ("Y, e", [("CCONJ", 1)]))
        self.assertEqual((steps["entries"], steps["direct"], steps["yielded"]), (set(), {"et"}, {"et"}))
        # Up to case: « Élite. » for `élite` and « Troll. » for `troll`, the table's words being other.
        glosses, _, _, steps = self.tables(
            ["élite", "troll"], ELITE, TROLL, direct=[ELITE_TRANSLATED, TROLL_TRANSLATED]
        )
        self.assertEqual(glosses, {"élite": "Elite", "troll": "Trol"})
        self.assertEqual(steps["yielded"], {"élite", "troll"})

    def test_spec_scenario_a_cognate_keeps_its_definition(self):
        # « Venir. » repeats `venir`, and the table lists « venir » too: a Spanish word spelled alike.
        glosses, _, _, steps = self.tables(["venir"], VENIR, direct=[VENIR_TRANSLATED])
        self.assertEqual(glosses, {"venir": "Venir"})
        self.assertEqual((steps["entries"], steps["yielded"]), ({"venir"}, set()))

    def test_the_studied_word_keeps_its_definition_when_no_table_lists_it(self):
        # Made up: nothing else glosses `et`, or `et` defined beside a sense of its own: kept.
        glosses, _, _, steps = self.tables(["et"], ET)
        self.assertEqual((glosses, steps["yielded"]), ({"et": "Et"}, set()))
        two = {**ET, "senses": [{"glosses": ["Et."]}, {"glosses": ["Y."]}]}
        glosses, _, _, steps = self.tables(["et"], two, direct=[ET_TRANSLATED])
        self.assertEqual((glosses, steps["yielded"]), ({"et": "Et; Y"}, set()))

    def test_no_self_definition_reads_a_gloss_as_written_up_to_case(self):
        glosses = {"et": "Et", "élite": "ÉLITE.", "venir": "Venir", "maison": "Casa", "slip": "Slip; Slip"}
        runs = {lemma: [("X", 1)] for lemma in glosses}
        direct = {"et": {"CCONJ": ["y"]}, "élite": {"NOUN": ["elite"]}, "venir": {"VERB": ["Venir"]}, "slip": {"NOUN": ["bragas"]}}
        self.assertEqual(fr_es.no_self_definition(glosses, runs, direct), ["et", "slip", "élite"])
        self.assertEqual(glosses, {"venir": "Venir", "maison": "Casa"})
        self.assertEqual(set(runs), {"venir", "maison"})

    def test_spec_scenario_a_name_on_a_common_word_s_card(self):
        # `Pierre`'s note is left out of `pierre`'s card, and the entry, left with no sense, goes;
        # `Jean`'s out of `jean`'s.
        glosses, runs, _, _ = self.tables(
            ["pierre", "jean"], PIERRE, PIERRE_VERB, PIERRE_NAME, JEAN_ADJ, JEAN_PHRASE, JEAN_NAME
        )
        self.assertEqual((glosses["pierre"], runs["pierre"]), ("Piedra", [("NOUN", 1)]))
        self.assertEqual(glosses["jean"], "Color de mezclilla; Mezclilla; Tejanos")
        written = [json.loads(line)["word"] for line in Path(self.dir / "section-glossing.jsonl").read_text().splitlines()]
        self.assertEqual(written, ["pierre", "pierre", "jean", "jean"])

    def test_spec_scenario_a_name_s_own_row(self):
        # No `françois` in lower case: the note says what the capitalised token is.
        glosses, runs, _, _ = self.tables(["françois"], FRANCOIS_NAME)
        self.assertEqual(
            (glosses["françois"], runs["françois"]),
            ("Nombre de pila de varón, equivalente del español Francisco", [("PROPN", 1)]),
        )
        # Made up: an entry in lower case that only points at another word holds no meaning.
        pointer = {"word": "françois", "pos": "adj", "senses": [{"glosses": ["Forma del plural de français."]}]}
        glosses, _, _, _ = self.tables(["françois"], FRANCOIS_NAME, pointer)
        self.assertEqual(glosses["françois"], "Nombre de pila de varón, equivalente del español Francisco")

    def test_spec_scenario_possessives_and_their_forms(self):
        # `mon` tagged possessive, `ce` demonstrative: determiners; `mes` and `ma`, every sense a form
        # of `mon`, too — so they borrow `mon`'s « Mi » in its part of speech. Only the runs move.
        glosses, runs, _, _ = self.tables(["mon", "mes", "ma", "ce"], MON, MES, MA, CE_ADJ, CE_PRON)
        self.assertEqual(glosses, {"mon": "Mi", "mes": "Mi", "ma": "Mi", "ce": "Este; Esto, eso, aquello"})
        self.assertEqual(
            runs,
            {"mon": [("DET", 1)], "mes": [("DET", 1)], "ma": [("DET", 1)], "ce": [("DET", 1), ("PRON", 1)]},
        )
        # Without the second clause, `mes` lost its gloss: a form lends in its own part of speech.
        with mock.patch.object(fr_es, "_forms_of", lambda entry, words: False):
            glosses, _, _, _ = self.tables(["mon", "mes"], MON, MES)
        self.assertEqual(glosses, {"mon": "Mi"})
        # The edition heads `son` « pronombre posesivo »: kept as written.
        _, runs, _, _ = self.tables(["son"], SON_PRON, SON_NOUN)
        self.assertEqual(runs, {"son": [("PRON", 1), ("NOUN", 1)]})

    def test_a_form_of_a_word_that_is_no_determiner_stays_an_adjective(self):
        # Made up: an adjective whose senses point at an adjective that is no determiner, or at a
        # determiner and another word.
        grand = {"word": "grande", "pos": "adj", "senses": [{"glosses": ["Forma del femenino de grand."], "form_of": [{"word": "grand"}]}]}
        mixed = {
            "word": "mas",
            "pos": "adj",
            "senses": [
                {"glosses": ["Forma de mon."], "form_of": [{"word": "mon"}]},
                {"glosses": ["Forma de grand."], "form_of": [{"word": "grand"}]},
            ],
        }
        none = {"word": "vide", "pos": "adj", "senses": []}
        written = Path(self.section(MON, grand, mixed, none)).read_text(encoding="utf-8").splitlines()
        self.assertEqual([json.loads(line)["pos"] for line in written], ["det", "adj", "adj", "adj"])

    def test_spec_scenario_a_letter_glosses_no_word(self):
        # `h` and `x`: the section's letters are left out, and the only word a table gives each is
        # the letter itself — the Spanish Wiktionary's `h` and `x` list theirs, its noun `i` the
        # letter, the French Wiktionary's `i` « i latina, i ». `à` and `y`, words of one letter, keep
        # the section's definitions; `ô` the direct table's « Oh ».
        glosses, _, _, _ = self.tables(
            ["h", "x", "i", "à", "y", "ô"],
            H_SECTION,
            X_SECTION,
            A_SECTION,
            Y_CHARACTER,
            Y_ADV,
            direct=[I_TRANSLATED, A_TRANSLATED, Y_TRANSLATED, O_TRANSLATED],
            inverted=[H_LISTS, X_LISTS, I_LISTS, Y_LISTS_ET],
        )
        self.assertEqual(
            glosses,
            {"à": "A (un lugar); A, hasta (un tiempo); A (una dirección)", "y": "Allí, ahí", "ô": "Oh"},
        )
        # Read backwards, a one-letter French word is dropped whatever lists it (made up: a Spanish
        # word, no letter, listing `h`).
        self.assertEqual(self.inverted({"pos": "intj", "translations": [{"word": "h"}], "word": "hache"}), {})

    def test_spec_scenario_a_spanish_word_listed_once(self):
        # `este`, adjective and pronoun, both list `cet`: listed once, under the first listed — no
        # reading of `cet` names either.
        inverted = self.inverted(ESTE_ADJ_LISTS, ESTE_PRON_LISTS)
        self.assertEqual(inverted["cet"], {"ADJ": ["este"]})
        glossed = common.fallback_glosses({"cet"}, {}, [(inverted, list)], edition=ES)
        self.assertEqual(glossed, {"cet": ("Este", [("ADJ", 1)])})
        # French's readings say where a word goes: `entraîneur` and `condom` are nouns.
        studied = self.dir / "fr"
        studied.mkdir()
        (studied / "grammar.tsv").write_text(CONDOM_ENTRAINEUR_GRAMMAR, encoding="utf-8")
        readings = fr_es.read_readings(str(studied))
        self.assertEqual(readings, {"condom": {"NOUN"}, "entraîneur": {"NOUN"}})
        order = fr_es.by_spanish_frequency(spanish_zipf)
        lists = (ADIESTRADOR_ADJ_LISTS, ADIESTRADOR_NOUN_LISTS, *CONDOM_LISTS)
        read = common.fallback_glosses({"condom", "entraîneur"}, {}, [(self.inverted(*lists, readings=readings), order)], edition=ES)
        self.assertEqual(
            read,
            {"condom": ("Preservativo, condón, profiláctico", [("NOUN", 1)]), "entraîneur": ("Adiestrador", [("NOUN", 1)])},
        )
        # Without them, the first listed: an adjective.
        read = common.fallback_glosses({"condom", "entraîneur"}, {}, [(self.inverted(*lists), order)], edition=ES)
        self.assertEqual(
            read,
            {
                "condom": ("Preservativo, profiláctico; Condón", [("ADJ", 1), ("NOUN", 1)]),
                "entraîneur": ("Adiestrador", [("ADJ", 1)]),
            },
        )
        (studied / "grammar.tsv").unlink()
        self.assertEqual(fr_es.read_readings(str(studied)), {}, "no readings: an empty mapping")

    def test_the_direct_table_keeps_the_french_word_s_parts_of_speech(self):
        # `parti`, translated as an adjective and as a noun, read « Partido; Partido » (Open Question
        # 3): `in_typography` keeps both; `listed_once_direct` lists the word once
        # (refine-lingua-fr-es-glosses D3, `TranslationTables`).
        with mock.patch.object(fr_es, "listed_once_direct", lambda table: table):
            glossed = common.fallback_glosses(
                {"parti"}, {}, [(self.direct(PARTI_ADJ_TRANSLATED, PARTI_NOUN_TRANSLATED), list)], edition=ES
            )
        self.assertEqual(glossed, {"parti": ("Partido; Partido", [("ADJ", 1), ("NOUN", 1)])})

    def test_spec_scenario_a_typographic_apostrophe(self):
        # The French Wiktionary's `main-d’œuvre` is the committed lemma `main-d'œuvre`; its
        # expression « aller de l’avant », and the Spanish Wiktionary's « coup d’État » read
        # backwards, are keyed as French's tokens read them; a section headword likewise (made up).
        glosses, _, expressions, _ = self.tables(
            ["main-d'œuvre", "prud'homme"],
            PRUDHOMME,
            direct=[MAIN_D_OEUVRE_TRANSLATED, ALLER_DE_L_AVANT_TRANSLATED],
            inverted=[GOLPE_MILITAR_LISTS],
        )
        self.assertEqual(glosses, {"main-d'œuvre": "Mano de obra", "prud'homme": "Juez laboral"})
        self.assertEqual(expressions, {"aller de l'avant": "Echar para adelante", "coup d'état": "Golpe militar"})
        # Without it, neither table glosses the word: `’` is no letter of a French token.
        self.assertEqual(common.read_translations(self.jsonl(MAIN_D_OEUVRE_TRANSLATED), inverted=False, studied=fr_es.FR), {})

    def test_the_outdated_senses_come_after_the_others(self):
        # The Spanish edition's pre-pass, as en-es reads it: `chapelet`'s « Guirnalda », labelled
        # obsolete, after « Rosario ».
        glosses, _, _, _ = self.tables(["chapelet"], CHAPELET)
        self.assertEqual(glosses, {"chapelet": "Rosario; Guirnalda"})

    def test_the_passes_write_what_they_cannot_read_as_they_are(self):
        # Made up: lines that are no JSON object, an entry they do not change, senses and words of no
        # kaikki shape. Each is written as it is, byte for byte, by each pass.
        text = (
            "not json\n[1, 2]\n"
            '{"word":"maison","pos":"noun","senses":[{"glosses":["Casa."]}]}\n'
            '{"word": "raro", "pos": "adj", "tags": "possessive", "senses": ["odd", {"glosses": [1]}, {"form_of": "mon"}]}\n'
            '{"word": 3, "senses": 4, "translations": 5}\n'
            '{"word": "Raro", "pos": "name", "senses": "Apellido."}\n'
            '{"pos": "noun", "translations": [1, {"word": 2}]}\n'
        )
        src = self.dir / "entries.jsonl"
        src.write_text(text, encoding="utf-8")
        for name, run in (
            ("apostrophes", lambda dst: fr_es.straight_apostrophes(str(src), dst)),
            ("corrected", lambda dst: fr_es.corrected_section(str(src), dst)),
            ("glossing", lambda dst: fr_es.french_entries(str(src), dst)),
            ("treebank", lambda dst: fr_es.treebank_order(str(src), dst, collections.Counter())),
            ("labels", lambda dst: fr_es.with_labels(str(src), dst)),
            ("direct", lambda dst: fr_es.translation_words(str(src), dst, inverted=False)),
            ("inverted", lambda dst: fr_es.translation_words(str(src), dst, inverted=True)),
        ):
            out = run(str(self.dir / f"{name}.jsonl"))
            self.assertEqual(Path(out).read_text(encoding="utf-8"), text, name)


class TheReducer(Tables):
    """fr-es reduced as its build runs it (`main()`), from a studied folder as fr-en writes it (D1),
    and what it measures of its tables (D9)."""

    def studied(self, forms, freq, grammar=None):
        folder = self.dir / "fr"
        folder.mkdir(exist_ok=True)
        (folder / "forms.tsv").write_text(forms, encoding="utf-8")
        (folder / "freq.tsv").write_text(freq, encoding="utf-8")
        if grammar is not None:
            (folder / "grammar.tsv").write_text(grammar, encoding="utf-8")
        return folder

    def gsd(self, work, adverbs=0):
        """UD French-GSD's two sections in the work folder (made up): `pas` read `adverbs` times as an
        adverb in the training section, once as a noun in the development one."""
        work.mkdir(parents=True, exist_ok=True)
        token = "1\tpas\tpas\t{}\t_\t_\t0\troot\t_\t_\n\n"
        (work / "fr_gsd-ud-train.conllu").write_text("# made up\n" + token.format("ADV") * adverbs, encoding="utf-8")
        (work / "fr_gsd-ud-dev.conllu").write_text(token.format("NOUN"), encoding="utf-8")

    def reduce(self, work, studied):
        wordfreq = types.ModuleType("wordfreq")
        wordfreq.zipf_frequency = lambda word, lang: spanish_zipf(word) if lang == "es" else 0.0
        argv = [
            "reduce-fr-es.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-10", "--pack-version", "test"),
        ]
        with (
            mock.patch.dict(sys.modules, {"wordfreq": wordfreq}),
            mock.patch.object(sys, "argv", argv),
            contextlib.redirect_stderr(io.StringIO()) as err,
        ):
            fr_es.main()
        return err.getvalue()

    def test_spec_scenarios_through_main(self):
        work = self.dir / "work"
        studied = self.studied(
            "condom\tcondom\ncondoms\tcondom\net\tet\nh\th\nintérêt\tintérêt\nmain-d'œuvre\tmain-d'œuvre\n"
            "maison\tmaison\nmaisons\tmaison\nmes\tmes\nmon\tmon\npierre\tpierre\ntravers\ttravers\n",
            "et\t1\nmon\t2\nmes\t3\nmaison\t4\nh\t5\npierre\t6\nintérêt\t7\ntravers\t8\nmain-d'œuvre\t9\ncondom\t10\n",
            CONDOM_ENTRAINEUR_GRAMMAR,
        )
        self.jsonl(MAISON, ET, MON, MES, PIERRE, PIERRE_NAME, H_SECTION, name="work/kaikki-es-Frances.jsonl")
        self.jsonl(
            ET_TRANSLATED, INTERET_TRANSLATED, MAIN_D_OEUVRE_TRANSLATED, ALLER_DE_L_AVANT_TRANSLATED, I_TRANSLATED,
            name="work/kaikki-fr-traductions.jsonl",
        )
        self.jsonl(TRAVES_LISTS, H_LISTS, *CONDOM_LISTS, name="work/kaikki-es-traductions.jsonl")
        self.gsd(work)
        err = self.reduce(work, studied)
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "condom\tPreservativo, condón, profiláctico\n"
            "et\tY, e\n"
            "intérêt\tInterés\n"
            "main-d'œuvre\tMano de obra\n"
            "maison\tCasa\n"
            "mes\tMi\n"
            "mon\tMi\n"
            "pierre\tPiedra\n"
            "travers\tTravés\n",
        )
        self.assertEqual(
            (work / "senses.tsv").read_text(encoding="utf-8"),
            "condom\tNOUN:1\net\tCCONJ:1\nintérêt\tNOUN:1\nmain-d'œuvre\tNOUN:1\nmaison\tNOUN:1\n"
            "mes\tDET:1\nmon\tDET:1\npierre\tNOUN:1\ntravers\tNOUN:1\n",
        )
        self.assertEqual((work / "mwe.tsv").read_text(encoding="utf-8"), "aller de l'avant\tEchar para adelante\n")
        # The native side alone (D1), and what the reducer measured of it (D9).
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith(("kaikki-", "fr_gsd-")))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        measures = json.loads((work / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(
            measures,
            {
                "top": 10000,
                "glossed": 9,
                "share": 55.6,
                "direct": ["et", "intérêt", "main-d'œuvre"],
                "inverted": ["condom", "travers"],
            },
        )
        self.assertIn(
            "reduced fr-es: lemmas=10 (French's committed tables) glosses=9 (Spanish Wiktionary 4, French "
            "Wiktionary's Spanish translations 3, Spanish Wiktionary's French translations read backwards 2; "
            "44.4 % from a definition; 1 definitions repeating their headword set aside) expressions=1",
            err,
        )
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("fr", "es"))
        self.assertEqual(manifest["meta"]["pack_version"], "test")
        self.assertEqual(manifest["meta"]["analyzer_version"], fr_en.analyser_version(), "French's analyser version")
        self.assertIs(manifest["meta"]["levels_estimated"], True, "French's levels are fr-en's estimate")
        self.assertEqual([s["name"] for s in manifest["sources"]], ["kaikki", "wordfreq", "UD French-GSD"])
        notice = (work / "NOTICE").read_text(encoding="utf-8")
        for credit in ("enwiktionary", "eswiktionary", "frwiktionary", "wordfreq", "UD French-GSD"):
            self.assertIn(credit, notice)
        self.assertIn("The levels are estimated, not taken from a CEFR list", notice)
        for list_ in ("CEFR-J", "Octanove", "FLELex"):
            self.assertNotIn(list_, notice)

    def test_reduce_fr_es_runs_the_passes_in_order(self):
        # Its apostrophes read as French's, its letters left out, then the edition's pre-pass, then
        # fr-es's, then the shared rules over the last one's file.
        work = self.dir / "work"
        studied = self.studied("maison\tmaison\n", "maison\t1\n")
        self.jsonl(MAISON, name="work/kaikki-es-Frances.jsonl")
        self.jsonl(name="work/kaikki-fr-traductions.jsonl")
        self.jsonl(name="work/kaikki-es-traductions.jsonl")
        self.gsd(work)
        calls = []

        def spy(name, real):
            def call(*args, **kwargs):
                calls.append((name, *(Path(arg).name for arg in args if isinstance(arg, str))))
                return real(*args, **kwargs)

            return call

        with (
            mock.patch.object(fr_es, "straight_apostrophes", spy("straight_apostrophes", fr_es.straight_apostrophes)),
            mock.patch.object(fr_es, "corrected_section", spy("corrected_section", fr_es.corrected_section)),
            mock.patch.object(common, "without_letter_senses", spy("without_letter_senses", common.without_letter_senses)),
            mock.patch.object(spanish, "read_as_meanings", spy("read_as_meanings", spanish.read_as_meanings)),
            mock.patch.object(fr_es, "french_entries", spy("french_entries", fr_es.french_entries)),
            mock.patch.object(fr_es, "treebank_order", spy("treebank_order", fr_es.treebank_order)),
            mock.patch.object(fr_es, "with_labels", spy("with_labels", fr_es.with_labels)),
            mock.patch.object(fr_es, "native_side", spy("native_side", fr_es.native_side)),
        ):
            self.reduce(work, studied)
        self.assertEqual(
            calls,
            [
                ("straight_apostrophes", "kaikki-es-Frances.jsonl", "kaikki-es-Frances-apostrophes.jsonl"),
                ("corrected_section", "kaikki-es-Frances-apostrophes.jsonl", "kaikki-es-Frances-corrected.jsonl"),
                ("without_letter_senses", "kaikki-es-Frances-corrected.jsonl", "kaikki-es-Frances-words.jsonl"),
                ("read_as_meanings", "kaikki-es-Frances-words.jsonl", "kaikki-es-Frances-meanings.jsonl"),
                ("french_entries", "kaikki-es-Frances-meanings.jsonl", "kaikki-es-Frances-glossing.jsonl"),
                ("treebank_order", "kaikki-es-Frances-glossing.jsonl", "kaikki-es-Frances-treebank.jsonl"),
                ("with_labels", "kaikki-es-Frances-treebank.jsonl", "kaikki-es-Frances-labels.jsonl"),
                ("native_side", "kaikki-es-Frances-labels.jsonl"),
            ],
        )
        self.assertEqual((work / "gloss.tsv").read_text(encoding="utf-8"), "maison\tCasa\n")

    def test_fr_es_s_own_rules_through_main(self):
        # refine-lingua-fr-es-glosses through `main`: UD French-GSD's two sections read from the work
        # folder, the corrections, the treebank's order, « etc. », the acronym rule; the summary names
        # the corrections that found nothing.
        work = self.dir / "work"
        studied = self.studied(
            "cochon\tcochon\nhall\thall\npas\tpas\nrien\trien\nus\tus\n", "pas\t1\nrien\t2\nus\t3\nhall\t4\ncochon\t5\n"
        )
        self.jsonl(COCHON, PAS_NOUN, PAS_ADV, RIEN_NOUN, name="work/kaikki-es-Frances.jsonl")
        self.jsonl(HALL_TRANSLATED, name="work/kaikki-fr-traductions.jsonl")
        self.jsonl(EEUU_LISTS, name="work/kaikki-es-traductions.jsonl")
        self.gsd(work, adverbs=10)
        err = self.reduce(work, studied)
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "cochon\tCerdo, marrano, guarro, cochino, etc.\nhall\tVestíbulo, recibidor\npas\tNo; Paso\nrien\tPequeña cantidad de algo\n",
        )
        self.assertIn(
            "fr-es's own rules: 0 expression senses labelled; 4 translations read backwards left out (names, acronyms, "
            "the language code, other senses), 0 of the direct table named as other senses; 2 of 6 corrections fired; "
            "corrections that found nothing, to remove: amie (section, noun), il y a (section, phrase), el (section, pron), "
            "el (direct, pron)",
            err,
        )
        # Too little evidence: the section's order.
        self.gsd(work, adverbs=9)
        self.reduce(work, studied)
        self.assertIn("pas\tPaso; No\n", (work / "gloss.tsv").read_text(encoding="utf-8"))

    def test_main_reads_the_treebank_from_the_work_folder(self):
        work = self.dir / "work"
        studied = self.studied("maison\tmaison\n", "maison\t1\n")
        self.jsonl(MAISON, name="work/kaikki-es-Frances.jsonl")
        self.jsonl(name="work/kaikki-fr-traductions.jsonl")
        self.jsonl(name="work/kaikki-es-traductions.jsonl")
        with self.assertRaisesRegex(SystemExit, r"fr_gsd-ud-train\.conllu, .*fr_gsd-ud-dev\.conllu missing"):
            self.reduce(work, studied)

    def test_native_side_gives_the_tables_native_tables_gives(self):
        # The steps written out keep each source's lemmas and change nothing of the tables, when no
        # definition repeats its headword.
        entries = self.section(MAISON, PIERRE, VENIR)
        ranks = {"maison": 1, "pierre": 2, "venir": 3, "intérêt": 4, "travers": 5, "rien": 6}
        direct = self.direct(INTERET_TRANSLATED, VENIR_TRANSLATED, ALLER_DE_L_AVANT_TRANSLATED)
        inverted = self.inverted(TRAVES_LISTS, CASA_LISTS, GOLPE_MILITAR_LISTS)
        sources = [(direct, list), (inverted, fr_es.by_spanish_frequency(spanish_zipf))]
        glosses, runs, expressions, steps = fr_es.native_side(entries, ranks, sources)
        shared = common.native_tables(
            entries, ranks, studied=fr_es.FR, edition=fr_es.EDITION, fallbacks=sources, locutions=fr_es.LOCUTIONS
        )
        self.assertEqual((glosses, runs, expressions), shared[:3])
        self.assertEqual(
            steps,
            {
                "entries": {"maison", "pierre", "venir"},
                "yielded": set(),
                "direct": {"intérêt"},
                "inverted": {"travers"},
                "labelled": 0,
            },
        )
        self.assertEqual(len(steps["entries"]), shared[3], "`primary`")

    def test_the_shares_count_what_they_say(self):
        steps = {"entries": {"a", "c", "z"}, "direct": {"b", "y"}, "inverted": {"d"}, "yielded": set()}
        ranks = {"a": 1, "b": 2, "c": 3, "d": 4, "y": 5, "z": 6}
        self.assertEqual(
            fr_es.translation_share(steps, ranks, top=4),
            {"top": 4, "glossed": 4, "share": 50.0, "direct": ["b"], "inverted": ["d"]},
        )
        self.assertEqual(fr_es.translation_share(steps, ranks)["share"], 50.0)
        self.assertEqual(fr_es.definition_share(steps), 50.0)
        empty = {"entries": set(), "direct": set(), "inverted": set()}
        self.assertEqual((fr_es.translation_share(empty, ranks)["share"], fr_es.definition_share(empty)), (0.0, 0.0))

    def test_fr_es_s_copy_of_french_is_fr_en_s(self):
        # reduce-fr-es.py repeats fr-en's description of French (the token, the coordinators, the
        # form-of target) because a reducer loads no other pair's: the two must not drift.
        self.assertEqual(fr_es.FR, fr_en.FR)
        self.assertEqual(
            (fr_es.FR.token.pattern, fr_es.FR.form_of_target.pattern, fr_es.FR.coordinators),
            (fr_en.FR.token.pattern, fr_en.FR.form_of_target.pattern, fr_en.FR.coordinators),
        )
        self.assertIs(fr_es.EDITION, ES)
        self.assertEqual(fr_es.LOCUTIONS, {})
        self.assertEqual(fr_es.analyser_version(), fr_en.analyser_version())

    def test_fr_es_reads_the_committed_studied_tables_as_fr_en_writes_them(self):
        # A folder whose forms and ranks disagree is refused, naming both; the lemmas are capped by
        # rank; a folder lacking one is named.
        studied = self.studied("maison\tmaison\nmaisons\tmaison\n", "maison\t1\ncasa\t2\n")
        with self.assertRaisesRegex(SystemExit, r"freq\.tsv and .*forms\.tsv disagree on the lemmas"):
            fr_es.read_studied(str(studied), 60000)
        studied = self.studied("et\tet\nmaison\tmaison\nmaisons\tmaison\n", "et\t1\nmaison\t2\n")
        self.assertEqual(fr_es.read_studied(str(studied), 60000), {"et": 1, "maison": 2})
        self.assertEqual(fr_es.read_studied(str(studied), 1), {"et": 1})
        (studied / "freq.tsv").unlink()
        with self.assertRaisesRegex(SystemExit, "freq.tsv is missing"):
            fr_es.read_studied(str(studied), 60000)
        # The committed folder: every lemma French commits within build.sh's cap.
        committed = Path(_HERE, "tables", "fr")
        ranks = fr_es.read_studied(str(committed), 60000)
        ranked = sum(1 for line in (committed / "freq.tsv").read_text(encoding="utf-8").splitlines() if "\t" in line)
        self.assertEqual(len(ranks), ranked)
        self.assertTrue(fr_es.read_readings(str(committed)), "French's committed readings")

    def test_the_committed_tables_read_as_the_scenarios_say(self):
        # tables/fr-es/ as lingua-pack-update reduced it from lingua-pack-sources-fr-es-2026.10.10:
        # each scenario's word as its recorded entries give it, and no one-letter lemma glossed by
        # the letter or its name.
        committed = Path(_HERE, "tables", "fr-es")
        glosses = dict(fr_es.read_table(committed / "gloss.tsv"))
        runs = dict(fr_es.read_table(committed / "senses.tsv"))
        expressions = dict(fr_es.read_table(committed / "mwe.tsv"))
        expected = {
            "maison": "Casa",
            "intérêt": "Interés",
            "travers": "Través",
            "et": "Y, e",
            "venir": "Venir",
            "pierre": "Piedra",
            "françois": "Nombre de pila de varón, equivalente del español Francisco",
            "mon": "Mi",
            "mes": "Mi",
            "ma": "Mi",
            "cet": "Este",
            "main-d'œuvre": "Mano de obra",
            "chapelet": "Rosario; Guirnalda",
        }
        self.assertEqual({lemma: glosses.get(lemma) for lemma in expected}, expected)
        self.assertEqual((runs["mon"], runs["mes"], runs["ma"]), ("DET:1", "DET:1", "DET:1"))
        self.assertEqual(runs["et"], "CCONJ:1")
        self.assertEqual(expressions["aller de l'avant"], "Echar para adelante")
        letters = {lemma: gloss for lemma, gloss in glosses.items() if len(lemma) == 1}
        self.assertEqual(sorted(letters), ["y", "à", "ô"])
        for lemma, gloss in letters.items():
            for sense in gloss.split("; "):
                self.assertNotEqual(sense.strip().lower(), lemma, f"{lemma!r} glossed by itself")
                self.assertNotIn("letra", sense.lower(), f"{lemma!r} glossed as a letter: {gloss!r}")
        self.assertTrue(all("’" not in lemma for lemma in [*glosses, *expressions]), "a typographic apostrophe")

    def test_the_committed_tables_read_as_refine_lingua_fr_es_glosses_says(self):
        # tables/fr-es/ reduced again from lingua-pack-sources-fr-es-2026.10.10 with fr-es's own rules:
        # each scenario's word as its recorded entries give it.
        committed = Path(_HERE, "tables", "fr-es")
        glosses = dict(fr_es.read_table(committed / "gloss.tsv"))
        runs = dict(fr_es.read_table(committed / "senses.tsv"))
        expressions = dict(fr_es.read_table(committed / "mwe.tsv"))
        expected = {
            "baiser": "(malsonante) Coger (sexualmente); (malsonante) Culear, follar, fornicar, joder o realizar el coito; "
            "(malsonante) Dominar o joder; (malsonante) Quebrar o romper; (jergal) Grapar; (anticuado, Canadá, Bélgica) "
            "Besar; Beso, besuqueo u ósculo",
            "maîtresse": "(anticuado) Amante (femenina)",
            "rien": "Nada; Poca cosa; (obsoleto) Algo; Pequeña cantidad de algo; (coloquial, irónico) Muy; Mucha, muchas, "
            "mucho o muchos",
            "être": "Ser; (être + participio) Haber; Estar; (être + participio) ser",
            "qui": "Quién; Que",
            "cochon": "Cerdo, marrano, guarro, cochino, etc.",
            "pas": "No; Paso",
            "pendant": "Durante; Pendiente; Juego (cosas relacionadas entre si)",
            "parti": "Partido",
            "russe": "Ruso, rusa",
            "jeune": "Joven, chaval, muchacho",
            "clair": "Claro, luminoso, límpido; Claramente",
            "retraite": "Jubilación, retiro, pensión",
            "diaporama": "Diaporama",
            "amie": "Amiga; Amia o lamia",
            "hall": "Vestíbulo, recibidor",
            "onu": "ONU",
            "pâques": "Pascua",
        }
        self.assertEqual({lemma: glosses.get(lemma) for lemma in expected}, expected)
        self.assertEqual((runs["être"], runs["des"], runs["pas"]), ("VERB:4", "DET:1\tADP:1", "ADV:1\tNOUN:1"))
        for lemma in ("us", "usa", "fr", "luc", "lorient", "rap", "pilote", "ds", "arnaque", "el"):
            self.assertNotIn(lemma, glosses)
        self.assertEqual(expressions["mal aux cheveux"], "(anticuado) Resaca, caña, chaqui, chuchaqui, cruda, go")
        self.assertEqual(expressions["il y a"], "Hay; Hace")
        self.assertNotIn("contrôle continu", expressions)
        # A labelled expression stays within the card's page.
        self.assertLessEqual(max(len(gloss) for gloss in expressions.values() if gloss.startswith("(")), 160)

    def test_spec_scenario_a_rule_of_the_spanish_edition(self):
        # fr-es's rules are its reducer, the shared rules, the Spanish edition's and UD French-GSD's
        # module, which fr-en imports too (refine-lingua-fr-es-glosses D8) — no other pair's reducer.
        import pack_sources as ps

        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-fr-es.py")],
            ["reduce-fr-es.py", "reduce_common.py", "reduce_edition_es.py", "reduce_french_treebank.py"],
        )


class Labels(Tables):
    """*fr-es's glosses show a sense's register, age and place in the Spanish Wiktionary's words*
    (refine-lingua-fr-es-glosses D2)."""

    def test_spec_scenario_a_vulgar_sense_and_an_outdated_one(self):
        # « Besar », filed outdated, stays after the verb's other senses (the edition's pre-pass);
        # each labelled sense carries its own labels; the noun's sense has none.
        glosses, runs, _, _ = self.tables(["baiser"], BAISER_TRANSITIVE, BAISER_INTRANSITIVE, BAISER_NOUN)
        self.assertEqual(
            glosses["baiser"],
            "(malsonante) Coger (sexualmente); (malsonante) Culear, follar, fornicar, joder o realizar el coito; "
            "(malsonante) Dominar o joder; (malsonante) Quebrar o romper; (jergal) Grapar; "
            "(anticuado, Canadá, Bélgica) Besar; Beso, besuqueo u ósculo",
        )
        self.assertEqual(runs["baiser"], [("VERB", 6), ("NOUN", 1)])

    def test_spec_scenario_a_place_inside_another(self):
        # « América » and « Quebec »: Quebec alone, inside Canada inside America.
        glosses, _, _, _ = self.tables(["oui"], OUI_INTJ)
        self.assertEqual(
            glosses["oui"], "Afirmativo, así es, bien, ciertamente, correcto; (Quebec) Ciertamente o en verdad (contra preguntas negativas); ¿Sí?"
        )
        self.assertEqual(fr_es.labels({"categories": ["FR:Europa", "FR:Francia", "FR:Provenza"]}), ["Provenza"])
        self.assertEqual(fr_es.labels({"categories": ["FR:Europa", "FR:Suiza", "FR:Bélgica"]}), ["Suiza", "Bélgica"])
        self.assertEqual(fr_es.labels({"categories": ["FR:África", "FR:Ruanda"]}), ["Ruanda"])

    def test_spec_scenario_a_label_the_english_tags_miss(self):
        # kaikki's tags say `Canada` and `Quebec`; the categories say Belgium too.
        glosses, _, _, _ = self.tables(["avec"], AVEC_ADV)
        self.assertEqual(glosses["avec"], "(Quebec, Bélgica) También; Con este, consigo, con ella etc")

    def test_spec_scenario_no_reordering_for_a_label(self):
        # The labelled senses keep their place: `cul` opens on « Culo », not on « Fundo (de un
        # objeto) », its one unlabelled sense — figurative, which is no label.
        glosses, _, _, _ = self.tables(["cul"], CUL_NOUN, CUL_ADJ)
        self.assertEqual(
            glosses["cul"],
            "(malsonante) Culo; (malsonante) Ano, ojete, recto (culo); (malsonante) Coito, sexo; "
            "(malsonante, jergal) Pornografía; Fundo (de un objeto); (malsonante, jergal, Francia) Suerte propicia "
            "(ojete); (jergal) Estúpido",
        )

    def test_spec_scenario_a_figurative_sense(self):
        self.assertEqual(fr_es.labels(CUL_NOUN["senses"][4]), [])
        self.assertEqual(fr_es.labels({"categories": ["FR:Términos infrecuentes"], "tags": ["rare"]}), [])
        self.assertEqual(fr_es.labels({"glosses": ["Algo."]}), [])

    def test_a_category_filed_under_the_wrong_language_is_read_alike(self):
        glosses, _, _, _ = self.tables(["maîtresse"], MAITRESSE)
        self.assertEqual(glosses["maîtresse"], "(anticuado) Amante (femenina)")
        # Only the section's two prefixes: another language's category is no label of a French sense.
        self.assertEqual(fr_es.labels({"categories": ["EN:Términos coloquiales", "Términos coloquiales"]}), [])

    def test_a_pointer_is_never_labelled(self):
        # Made up: a pointer filed as colloquial glosses nothing, and keeps its text.
        pointer = {
            "word": "copine",
            "pos": "noun",
            "senses": [
                {"glosses": ["Forma del femenino de copain."], "categories": ["FR:Términos coloquiales"], "form_of": [{"word": "copain"}]}
            ],
        }
        written = Path(self.section(pointer)).read_text(encoding="utf-8")
        self.assertEqual(json.loads(written)["senses"][0]["glosses"], ["Forma del femenino de copain."])

    def test_rien_shows_its_labels_and_its_correction(self):
        glosses, runs, _, _ = self.tables(["rien"], RIEN_PRON, RIEN_NOUN, RIEN_ADV)
        self.assertEqual(
            glosses["rien"],
            "Nada; Poca cosa; (obsoleto) Algo; Pequeña cantidad de algo; (coloquial, irónico) Muy; Mucha, muchas, mucho o muchos",
        )
        self.assertEqual(runs["rien"], [("PRON", 3), ("NOUN", 1), ("ADV", 2)])

    def test_spec_scenario_an_expression_s_label_outside_its_cut(self):
        _, _, expressions, steps = self.tables([], MAL_AUX_CHEVEUX)
        self.assertEqual(expressions["mal aux cheveux"], "(anticuado) Resaca, caña, chaqui, chuchaqui, cruda, go")
        self.assertEqual(len("Resaca, caña, chaqui, chuchaqui, cruda, go"), fr_es.EXPRESSION_SENSE)
        self.assertEqual(steps["labelled"], 1)

    def test_an_expression_s_cut_remnant_is_labelled(self):
        # « fils de pute »: three senses cut to 42 characters each and to 80 together; the third is
        # the remnant « Obje », labelled as its sense is.
        _, _, expressions, steps = self.tables([], FILS_DE_PUTE)
        self.assertEqual(
            expressions["fils de pute"],
            "(vulgar) Hijo de puta, hijoputa o máncer; (malsonante, despectivo) Bastardo, hijo de la chingada, hijo "
            "de la; (malsonante, despectivo) Obje",
        )
        self.assertEqual(steps["labelled"], 3)

    def test_every_labelled_expression_is_its_gloss_without_labels(self):
        # Its labels taken out, each expression reads as the shared rules cut it, character for
        # character; an expression with no labelled sense is left as it is.
        entries = self.section(MAL_AUX_CHEVEUX, FILS_DE_PUTE, NOUS_AUTRES, IL_Y_A)
        bare = common.reduce_expressions(entries, common.EXPRESSION_GLOSS_LEN, studied=fr_es.FR, edition=ES)
        labelled, count = fr_es.label_expressions(bare, entries)
        self.assertEqual(count, 4)
        self.assertEqual(labelled["il y a"], bare["il y a"])
        vocabulary = [*fr_es.REGISTER.values(), *fr_es.AGE.values(), *fr_es.PLACES]
        pattern = r"\((?:%s)(?:, (?:%s))*\) " % ("|".join(vocabulary), "|".join(vocabulary))
        for expression, gloss in labelled.items():
            self.assertEqual(re.sub(pattern, "", gloss), bare[expression], expression)
        self.assertTrue(len(labelled["fils de pute"]) > common.EXPRESSION_GLOSS_LEN, "the label is outside the cut")

    def test_a_gloss_the_shared_rules_did_not_cut_so_is_left(self):
        # An expression whose gloss is not what the shared rules give from the section (a locution, or
        # a rule of theirs changed) is not walked.
        entries = self.section(MAL_AUX_CHEVEUX)
        labelled, count = fr_es.label_expressions({"mal aux cheveux": "Resaca"}, entries)
        self.assertEqual((labelled, count), ({"mal aux cheveux": "Resaca"}, 0))

    def test_the_expression_budgets_are_the_shared_rules(self):
        defaults = inspect.signature(common.reduce_expressions).parameters
        self.assertEqual(
            (fr_es.EXPRESSION_SENSE, fr_es.EXPRESSION_SENSES), (defaults["per_sense"].default, defaults["max_senses"].default)
        )


class FrenchMeanings(Tables):
    """*fr-es's definitions read as a French word's meanings* (refine-lingua-fr-es-glosses D6)."""

    def test_spec_scenario_an_infinitive_s_noun_after_its_verb(self):
        glosses, runs, _, _ = self.tables(["être", "devoir"], ETRE_NOUN, ETRE_VERB, ETRE_AUXILIARY, DEVOIR_NOUN, DEVOIR_VERB, readings=READINGS)
        self.assertEqual(glosses["être"], "Ser; (être + participio) Haber; Estar; (être + participio) ser")
        self.assertEqual(runs["être"], [("VERB", 4)])
        self.assertEqual((glosses["devoir"], runs["devoir"]), ("Deber", [("VERB", 1)]))
        # Without French's readings naming the word a verb, the section's order.
        glosses, runs, _, _ = self.tables(["être"], ETRE_NOUN, ETRE_VERB, ETRE_AUXILIARY)
        self.assertEqual(runs["être"], [("NOUN", 1), ("VERB", 3)])
        # « jaune », a noun beside an adjective, keeps its order.
        glosses, runs, _, _ = self.tables(["jaune"], JAUNE_ADJ, JAUNE_NOUN, readings=READINGS)
        self.assertEqual((glosses["jaune"], runs["jaune"]), ("Amarillo; Amarillo (persona asiática)", [("ADJ", 1), ("NOUN", 1)]))

    def test_a_noun_sharing_some_senses_keeps_its_place(self):
        # Made up: a noun with a sense of its own stays before the verb.
        noun = {**ETRE_NOUN, "senses": [{"glosses": ["Ser."]}, {"glosses": ["Criatura."]}]}
        _, runs, _, _ = self.tables(["être"], noun, ETRE_VERB, readings=READINGS)
        self.assertEqual(runs["être"], [("NOUN", 2), ("VERB", 1)])
        self.assertIsNone(fr_es._verb_first("être", [ETRE_VERB, ETRE_NOUN], READINGS), "already after its verb")

    def test_spec_scenario_a_contraction_of_a_preposition(self):
        glosses, runs, _, _ = self.tables(["des", "du", "duquel"], DES_ARTICLE, DES_CONTRACTION, DU_CONTRACTION, DUQUEL_CONTRACTION)
        self.assertEqual(
            glosses["des"], "Algunos, algunas, unos o unas; Contracción de la preposición de y el artículo les, de las o de los"
        )
        self.assertEqual(runs["des"], [("DET", 1), ("ADP", 1)])
        self.assertEqual((runs["du"], runs["duquel"]), ([("ADP", 1)], [("ADP", 1)]))
        # « c'est » contracts a pronoun and a verb: as written.
        written = [json.loads(line) for line in Path(self.section(C_EST)).read_text(encoding="utf-8").splitlines()]
        self.assertEqual(written[0]["pos"], "contraction")

    def test_spec_scenario_the_part_of_speech_named_again(self):
        glosses, _, expressions, _ = self.tables(["qui", "quoi", "il"], QUI_INTERROGATIVE, QUI_RELATIVE, QUOI_INTERROGATIVE, IL_NEUTER, NOUS_AUTRES)
        self.assertEqual((glosses["qui"], glosses["quoi"]), ("Quién; Que", "Qué"))
        # The notes that say what the heading does not stay.
        self.assertEqual(glosses["il"], "Pronombre sujeto expletivo impersonal. (No tiene traducción al español. No existe en español.)")
        self.assertEqual(expressions["nous autres"], "Nosotros [y no tú o vosotros]. (Plural exc")

    def test_spec_scenario_etc_with_its_period(self):
        self.assertEqual(fr_es.with_etc_period("Cerdo, marrano, guarro, cochino, etc"), "Cerdo, marrano, guarro, cochino, etc.")
        self.assertEqual(fr_es.with_etc_period("La que, lo que (etc.) o cual"), "La que, lo que (etc.) o cual")
        self.assertEqual(fr_es.with_etc_period("Cual cosa, lo que etc; Algo"), "Cual cosa, lo que etc.; Algo")
        self.assertEqual(fr_es.with_etc_period("Etcétera"), "Etcétera")


class TranslationTables(Tables):
    """*fr-es's translation-table glosses list each Spanish word once, and never the French word*
    (refine-lingua-fr-es-glosses D3, D5)."""

    def gloss(self, word, *direct):
        return common.fallback_glosses({word}, {}, [(self.direct(*direct), list)], edition=ES).get(word)

    def test_spec_scenario_a_word_under_two_parts_of_speech(self):
        self.assertEqual(self.gloss("parti", PARTI_ADJ_TRANSLATED, PARTI_NOUN_TRANSLATED), ("Partido", [("ADJ", 1)]))

    def test_spec_scenario_a_run_inside_another(self):
        self.assertEqual(self.gloss("russe", RUSSE_ADJ_TRANSLATED, RUSSE_NOUN_TRANSLATED), ("Ruso, rusa", [("NOUN", 1)]))

    def test_spec_scenario_a_word_shown_twice_in_different_runs(self):
        self.assertEqual(
            self.gloss("clair", CLAIR_ADJ_TRANSLATED, CLAIR_ADV_TRANSLATED),
            ("Claro, luminoso, límpido; Claramente", [("ADJ", 1), ("ADV", 1)]),
        )

    def test_a_word_listed_fourth_is_not_shown(self):
        # `net`'s adjective lists « claro » fourth and « red » last: the gloss shows neither there.
        self.assertEqual(self.gloss("net", *NET_TRANSLATED), ("Puro, limpio, neto; Claro; Red", [("ADJ", 1), ("ADV", 1), ("NOUN", 1)]))

    def test_listed_once_direct_repeats_until_nothing_moves(self):
        table = {"x": {"ADJ": ["a", "b"], "NOUN": ["b", "c"], "ADV": ["c", "d"]}}
        self.assertEqual(fr_es.listed_once_direct(table), {"x": {"ADJ": ["a", "b"], "NOUN": ["c"], "ADV": ["d"]}})
        # Two equal runs: the later goes; a run left with no word goes, and so does a word.
        self.assertEqual(fr_es.listed_once_direct({"y": {"ADJ": ["a"], "NOUN": ["a"]}}), {"y": {"ADJ": ["a"]}})
        self.assertEqual(fr_es.listed_once_direct({"z": {"ADJ": []}}), {})

    def test_spec_scenario_the_french_word_given_as_spanish(self):
        self.assertIsNone(self.gloss("arnaque", ARNAQUE_TRANSLATED))
        self.assertEqual(self.gloss("retraite", RETRAITE_TRANSLATED), ("Jubilación, retiro, pensión", [("NOUN", 1)]))
        # The next source glosses a word the table no longer does (made up: `arnaque` read backwards).
        estafa = {"pos": "noun", "translations": [{"word": "arnaque"}], "word": "estafa"}
        glosses, _, _, steps = self.tables(["arnaque"], direct=[ARNAQUE_TRANSLATED], inverted=[estafa])
        self.assertEqual((glosses, steps["inverted"]), ({"arnaque": "Estafa"}, {"arnaque"}))

    def test_spec_scenario_a_loanword_spanish_writes_alike(self):
        self.assertIn("diaporama", fr_es.SPANISH_ALIKE)
        self.assertEqual(self.gloss("diaporama", DIAPORAMA_TRANSLATED), ("Diaporama", [("NOUN", 1)]))
        self.assertEqual(len(fr_es.SPANISH_ALIKE), 12)

    def test_spec_scenario_a_cognate(self):
        self.assertEqual(self.gloss("club", CLUB_TRANSLATED), ("Club", [("NOUN", 1)]))

    def test_an_expression_takes_no_french_gloss_from_the_table(self):
        _, _, expressions, _ = self.tables([], direct=[CONTROLE_CONTINU_TRANSLATED])
        self.assertEqual(expressions, {})
        self.assertFalse(fr_es.is_the_french_word("contrôle continu", "control continuo", zipf))


class Corrections(Tables):
    """*fr-es corrects, by name, the slips its sources write* (refine-lingua-fr-es-glosses D7)."""

    def test_spec_scenario_an_agreement_slip(self):
        fired = set()
        glosses, _, _, _ = self.tables(["rien"], RIEN_NOUN, fired=fired)
        self.assertEqual(glosses["rien"], "Pequeña cantidad de algo")
        self.assertEqual([c.headword for c in fired], ["rien"])
        # The sense's other fields are kept.
        corrected = fr_es._corrected(RIEN_NOUN, "section")
        self.assertEqual(corrected["senses"], [{"glosses": ["Pequeña cantidad de algo."], "sense_index": "1"}])

    def test_spec_scenario_a_meaning_the_section_gives_only_as_a_pointer(self):
        glosses, runs, _, _ = self.tables(["amie"], AMIE_ADJ, AMIE_NOUN)
        self.assertEqual((glosses["amie"], runs["amie"]), ("Amiga; Amia o lamia", [("NOUN", 2)]))
        # The fish keeps its sense as written; the friend is a new one.
        corrected = fr_es._corrected(AMIE_NOUN, "section")
        self.assertEqual(corrected["senses"], [{"glosses": ["Amiga."]}, AMIE_NOUN["senses"][0]])

    def test_spec_scenario_a_sense_the_section_misses(self):
        _, _, expressions, _ = self.tables([], IL_Y_A)
        self.assertEqual(expressions["il y a"], "Hay; Hace")

    def test_spec_scenario_a_word_left_with_no_gloss(self):
        fired = set()
        glosses, _, _, _ = self.tables(["el"], EL_PRON, direct=[EL_TRANSLATED], fired=fired)
        self.assertEqual(glosses, {})
        self.assertEqual(sorted((c.source, c.headword) for c in fired), [("direct", "el"), ("section", "el")])

    def test_the_direct_table_s_correction(self):
        self.assertEqual(self.direct(HALL_TRANSLATED)["hall"], {"NOUN": ["vestíbulo", "recibidor"]})
        glosses, _, _, _ = self.tables(["hall"], direct=[HALL_TRANSLATED])
        self.assertEqual(glosses["hall"], "Vestíbulo, recibidor")

    def test_spec_scenario_the_page_corrected_upstream(self):
        # Once the page reads « Pequeña cantidad de algo. », the correction fires on nothing, and the
        # summary names it — as it names every correction an update finds nothing for.
        fixed = {**RIEN_NOUN, "senses": [{"glosses": ["Pequeña cantidad de algo."]}]}
        fired = set()
        glosses, _, _, _ = self.tables(["rien"], fixed, fired=fired)
        self.assertEqual((glosses["rien"], fired), ("Pequeña cantidad de algo", set()))
        self.assertIn("rien (section, noun)", fr_es.unfired(fired))
        self.assertEqual(len(fr_es.unfired(fired)), len(fr_es.CORRECTIONS))
        # Another part of speech, or a text with another sense beside it, is not the source's either.
        adverb = {**RIEN_NOUN, "pos": "adv"}
        self.assertIs(fr_es._corrected(adverb, "section"), adverb)
        two = {**RIEN_NOUN, "senses": [*RIEN_NOUN["senses"], {"glosses": ["Nada."]}]}
        self.assertIs(fr_es._corrected(two, "section"), two)

    def test_every_correction_names_its_reason_and_its_page(self):
        for correction in fr_es.CORRECTIONS:
            self.assertIn(correction.source, ("section", "direct"))
            self.assertTrue(correction.reason)
            self.assertRegex(correction.report, r"^https://(es|fr)\.wiktionary\.org/wiki/")
            self.assertTrue(correction.report.startswith("https://es." if correction.source == "section" else "https://fr."))


class TreebankOrder(Tables):
    """*A function word's row in fr-es opens on the part of speech UD French-GSD reads it as*
    (refine-lingua-fr-es-glosses D8), fr-en's rule read from `reduce_french_treebank.py`."""

    def test_spec_scenario_a_negation_s_adverb_first(self):
        glosses, runs, _, _ = self.tables(["pas", "pendant"], PAS_NOUN, PAS_ADV, PENDANT_ADJ, PENDANT_NOUN, PENDANT_PREP, counts=TREEBANK)
        self.assertEqual((glosses["pas"], runs["pas"]), ("No; Paso", [("ADV", 1), ("NOUN", 1)]))
        self.assertEqual(glosses["pendant"], "Durante; Pendiente; Juego (cosas relacionadas entre si)")
        # The section's order without the treebank.
        glosses, _, _, _ = self.tables(["pas"], PAS_NOUN, PAS_ADV)
        self.assertEqual(glosses["pas"], "Paso; No")

    def test_spec_scenario_too_little_evidence(self):
        for counts in (
            collections.Counter({("pas", "ADV"): 9, ("pas", "NOUN"): 0}),  # fewer than 10
            collections.Counter({("pas", "ADV"): 30, ("pas", "NOUN"): 16}),  # less than twice as often
        ):
            glosses, _, _, _ = self.tables(["pas"], PAS_NOUN, PAS_ADV, counts=counts)
            self.assertEqual(glosses["pas"], "Paso; No", counts)

    def test_spec_scenario_a_content_word_s_row(self):
        # Made up: a noun first, the treebank reading the word as an adjective — kept.
        counts = collections.Counter({("jaune", "ADJ"): 160, ("jaune", "NOUN"): 4})
        glosses, _, _, _ = self.tables(["jaune"], JAUNE_NOUN, JAUNE_ADJ, counts=counts)
        self.assertEqual(glosses["jaune"], "Amarillo; Amarillo (persona asiática)")
        self.assertIsNone(fr_es._treebank_first("jaune", [JAUNE_NOUN, JAUNE_ADJ], counts))

    def test_a_proper_noun_is_never_first(self):
        name = {"word": "Pas", "pos": "name", "senses": [{"glosses": ["Paso de Calais."]}]}
        counts = collections.Counter({("pas", "PROPN"): 500, ("pas", "NOUN"): 8})
        self.assertIsNone(fr_es._treebank_first("pas", [PAS_NOUN, name], counts))
        # A row opening on a name moves its common word first.
        counts = collections.Counter({("pas", "PROPN"): 1, ("pas", "ADV"): 981})
        self.assertEqual(fr_es._treebank_first("pas", [name, PAS_ADV], counts), [1, 0])

    def test_an_acronym_s_entry_is_passed_over(self):
        # Made up: the page opens on an acronym; the part of speech it opens on is the next entry's.
        acronym = {"word": "PAS", "pos": "noun", "senses": [{"glosses": ["Sigla."]}]}
        self.assertEqual(fr_es._treebank_first("pas", [acronym, PAS_ADV, PAS_NOUN], TREEBANK), None)
        self.assertEqual(fr_es._treebank_first("pas", [acronym, PAS_NOUN, PAS_ADV], TREEBANK), [2, 0, 1])

    def test_the_rule_is_fr_en_s(self):
        self.assertIs(fr_es.treebank, fr_en.treebank)
        self.assertEqual(fr_es.GSD_FILES, ("fr_gsd-ud-train.conllu", "fr_gsd-ud-dev.conllu"))
        with self.assertRaisesRegex(SystemExit, "fr-es reads UD French-GSD's two sections"):
            fr_es.read_treebank(str(self.dir))


class NamesAndOtherSenses(Tables):
    """*fr-es's inverted table glosses no French word through a name, an acronym or another sense*
    (refine-lingua-fr-es-glosses D4)."""

    def test_spec_scenario_an_acronym_s_other_word(self):
        inverted = self.inverted(EEUU_LISTS, ONU_LISTS, AEC_LISTS)
        self.assertEqual(inverted, {"onu": {"X": ["ONU"]}, "aec": {"X": ["a. e. c."]}})
        self.assertNotIn("us", inverted)
        self.assertNotIn("usa", inverted)

    def test_spec_scenario_the_language_code(self):
        self.assertEqual(
            self.inverted(CALABAZA_LISTS), {"citrouille": {"NOUN": ["calabaza"]}, "potiron": {"NOUN": ["calabaza"]}, "courge": {"NOUN": ["calabaza"]}}
        )

    def test_spec_scenario_a_saint_for_a_first_name(self):
        inverted = self.inverted(SAN_LUCAS_LISTS, PASCUA_LISTS, SANTO_TOME_LISTS)
        self.assertEqual(inverted, {"pâques": {"NOUN": ["Pascua"]}, "sao tomé-et-principe": {"X": ["Santo Tomé y Príncipe"]}})

    def test_an_elided_article_read_into_a_word(self):
        self.assertEqual(self.inverted(ORIENTE_LISTS), {})
        self.assertFalse(fr_es.read_backwards("lOrient", "Oriente"))

    def test_spec_scenario_a_word_the_list_names(self):
        inverted = self.inverted(SECUESTRO_LISTS)
        self.assertNotIn("rap", inverted)
        self.assertIn("enlèvement", inverted)
        self.assertIn("détournement illégal de véhicule", inverted, "an expression of capitalised words read as before")
        # The direct table's « DS »: `ds` takes no gloss from it.
        self.assertEqual(self.direct(DS_TRANSLATED), {})
        glosses, _, _, _ = self.tables(["ds", "rap"], direct=[DS_TRANSLATED], inverted=[SECUESTRO_LISTS])
        self.assertEqual(glosses, {})
        self.assertEqual(len(fr_es.OTHER_SENSES), 6)
        self.assertTrue(all(fr_es.OTHER_SENSES.values()), "each pair with its reason")

    def test_an_expression_of_capitalised_words_reads_as_before(self):
        _, _, expressions, _ = self.tables([], inverted=[MAR_ROJO_LISTS])
        self.assertEqual(expressions, {"mer rouge": "Mar Rojo"})


if __name__ == "__main__":
    unittest.main()
