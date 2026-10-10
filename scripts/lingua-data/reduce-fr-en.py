#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the fr-en pack into its tables (add-lingua-french-forms-tables).

fr-en is French's reference pair (design D1): its reduction writes French's studied side, which
`pack_sources.py split` files into tables/fr/, and every other pair studying French reads it as
committed. French's studied rules live here, as Spanish's live in reduce-es-fr.py: a reducer named
with a hyphen cannot be imported, so no other pair loads them, and no shared module is edited.

Inputs, in `--work`:
- `kaikki-French.jsonl`: the English Wiktionary's French section, derived from the English
  edition's dump (pack_sources.py EDITIONS). A lemma's entry lists its inflections, each with tags;
  a form's own entry points at what it is a form of (`form_of`). Its senses are fr-en's glosses
  too (add-lingua-pack-fr-en): the English glosses of French words and expressions, written by
  people for French words.
- `fr_gsd-ud-train.conllu`, `fr_gsd-ud-dev.conllu`: UD French-GSD, read for how often each form
  stands for each lemma (design D5) and how often each hyphenated lemma occurs (D6). Its test
  section is never read: the measurement holds it out (D9).
- wordfreq `fr` (the installed, pinned package): the 60,000 commonest lemmas and which forms are
  attested at all (D6).

Outputs, in `--work`: `forms.tsv`, `freq.tsv`, the readings `grammar.tsv`
(add-lingua-french-grammar-tables), `level.tsv` (French's estimated levels, add-lingua-french-levels),
fr-en's native side — `gloss.tsv`, `senses.tsv` and `mwe.tsv` (add-lingua-pack-fr-en) —, `NOTICE`
and `manifest.json`.

The tables serve French's tokenisation as add-lingua-french-tokenisation writes it (M21, design
D4): the pre-pass hands the lookup the word an elided piece stands for (`l'` is read `le`), splits
`au` and `aux` into `à` + `le`/`les` and an inversion into its words (`dit-il` → `dit` + `il`),
keeps `du` and `des` whole, and keeps whole a hyphenated run the pack lists.

The native side (add-lingua-pack-fr-en D1–D3, D11) is read from the same section as the forms, after
the studied side, as es-en's is from the Spanish section: the section cut to what the native side
reads, a headword's typographic apostrophe read as `'` (`native_fields`); the English edition's
pre-passes in es-en's order — its letters left out (`reduce_common.without_letter_senses`,
`english.without_letter_headwords`), its senses read as meanings and in one English typography
(`english.read_as_meanings`), a word's etymologies merged as the edition's setting says
(`english.merge_same_pos_etymologies`) —; French's expressions (`expression_senses`, `split_words`);
then the rules every pair shares (`reduce_common.native_tables`) over the lemmas just ranked. The
English edition's rules are es-en's and fr-en's alike: editing `reduce_edition_en.py` re-pins both.

No translation table glosses a French word or expression (D3). The section already holds French's
words: measured, what the French Wiktionary's English translations and the English Wiktionary's
French translations read backwards would add is 1,988 lemmas, 1,895 of them words the section has no
entry for — English words (« in », « end »), names, initialisms, unaccented misspellings, 1,193
listing the word itself as its translation —, and fr-en is French's reference pair, so every lemma it
glosses becomes a dictionary word of French. Read backwards, an English entry makes French's
commonest bigrams expressions (« il est » "he's"). A word or an expression the section does not gloss
has no gloss.
"""

import argparse
import collections
import itertools
import json
import math
import os
import re
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_en as english  # noqa: E402 — the English Wiktionary's rules: fr-en's glosses are English

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")

# French letters, lowercased.
_LETTERS = "a-zàâäçéèêëîïôöùûüÿœæ"
# A French form (D3): letters, words joined by hyphens or by an inner apostrophe (`aujourd'hui`,
# `presqu'île`), and the elided pieces' final apostrophe (`l'`).
_TOKEN = re.compile(rf"[{_LETTERS}]+(?:['-][{_LETTERS}]+)*'?")

# The bookkeeping of kaikki's inflection tables, never forms: the class, the template, the table's
# own tags, the headword repeated.
_BOOKKEEPING = frozenset({"table-tags", "inflection-template", "class", "romanization", "canonical"})
# An inflection a source marks archaic, rarer or doubtful is no inflection (D3): an alternative or
# a misspelling of the form, an obsolete or rare one, an abbreviation or a clipping, a
# pronunciation spelling, what kaikki could not tag — and a multi-word construction
# (`avoir dirigé`, the compound tenses), no form of one word.
_DOUBTFUL = frozenset(
    {
        "alternative",
        "obsolete",
        "archaic",
        "rare",
        "dated",
        "uncommon",
        "misspelling",
        "nonstandard",
        "proscribed",
        "misconstruction",
        "abbreviation",
        "clipping",
        "pronunciation-spelling",
        "error-unknown-tag",
        "multiword-construction",
    }
)

# The gender and number markers a head template writes (`m`, `f`, `p` for plural, …). The dump
# leaves a few among an entry's forms, untagged or tagged by number: `m` under *Paris*, `f` under
# *Angora* and *Chambord*, `p` under *Socceroos* and *Saintes* — and `m` → *paris* would read « M. »
# (Monsieur) as Paris. A marker is no form of another word; a letter's entry listing its own other
# case (`M` lists `m`) is the word itself.
_MARKERS = frozenset({"m", "f", "n", "c", "p", "s", "mf", "pl", "sg", "mpl", "fpl", "inv"})

# The elided pieces (D4), each a form of the word the French pre-pass reads it as outside its
# special cases: piece → (lemma, reason). The pre-pass never looks a piece up — it hands the lookup
# the word — but a treebank writes the piece as the word (UD: `l'`, lemma `le`), and a card can be
# opened on one alone. The dictionary cannot decide them: it lists `l'` as an alternative of *le*
# and *la*, `s'` and `n'` with no pointer, `qu'` under *que* and *qui*. Editing this table changes
# the rules' sha256, as for any rule.
ELISIONS = {
    "l'": ("le", "the article and the pronoun; `la`, which elides to it too, is a form of le"),
    "d'": ("de", "the preposition, and the article it also is"),
    "j'": ("je", "the pronoun"),
    "m'": ("me", "the pronoun; the pre-pass reads `moi` right after a hyphen alone (donne-m'en)"),
    "t'": ("te", "the pronoun; the pre-pass reads `toi` right after a hyphen alone (va-t'en)"),
    "s'": ("se", "the pronoun; the pre-pass reads `si` before il and ils alone (s'il)"),
    "n'": ("ne", "the negation"),
    "c'": ("ce", "the pronoun"),
    "ç'": ("ça", "the pronoun"),
    "qu'": ("que", "the conjunction and the pronoun; the dictionary's reading as qui is Louisiana French's"),
    "jusqu'": ("jusque", "the preposition"),
    "lorsqu'": ("lorsque", "the conjunction"),
    "puisqu'": ("puisque", "the conjunction"),
    "quoiqu'": ("quoique", "the conjunction"),
}
# A plain word beginning with a piece (`c'est`, `d'abord`, `l'on`, `jusqu'à`): the pre-pass always
# splits it there, so the whole string is never looked up. Longest piece first: `qu'` is the end
# of `jusqu'`.
_ELIDED_START = re.compile(
    "(?:" + "|".join(re.escape(piece) for piece in sorted(ELISIONS, key=len, reverse=True)) + rf")[{_LETTERS}]"
)
# wordfreq's tokeniser splits `l'homme` into `l` and `homme`: its list counts each elided piece as
# the bare letters before the apostrophe (`l`, `d`, `qu`, `jusqu`, `ç`, …), at the pieces'
# frequency — ranked, the letter `l` would be French's fourth commonest word (D6). They are no
# words; the pieces are forms of theirs. Every piece's stem, so that no piece's frequency is ranked
# as a word.
WORDFREQ_STEMS = frozenset(piece[:-1] for piece in ELISIONS)
# M21: the pre-pass always splits these into `à` + `le` and `à` + `les`, so no page token is ever
# one: neither a form nor a rank.
SPLIT_CONTRACTIONS = frozenset({"au", "aux"})
# M21 keeps these whole, each a word of its own: `du` (the partitive article's own entry) and `des`
# (every sense of which points elsewhere — the plural of un, une and du, de + les — and which GSD's
# counts would read as un). A reader meeting « des » meets « des ». The one place the counts are
# overridden (D4).
OWN_WORDS = ("du", "des")
# The pronouns of an inversion, as the pre-pass reads them (add-lingua-french-tokenisation D5): a
# hyphenated run whose pieces after the first are among them is split into words, unless the pack
# lists the run whole.
INVERSION_PRONOUNS = frozenset(
    {"je", "tu", "il", "elle", "on", "nous", "vous", "ils", "elles", "ce", "le", "la", "les", "lui", "leur"}
    | {"moi", "toi", "y", "en"}
)
# The pieces a verb form is joined to by hyphens in a reflexive or imperative conjugation
# (`souviens-toi`, `allons-nous-en`, `sois-t'en`): the pronouns, the euphonic `t` (`a-t-il`), and
# an elided pronoun before `en` or `y`.
_CLITIC_PIECES = INVERSION_PRONOUNS | {"t"} | {f"{p}{w}" for p in ("m'", "t'", "l'") for w in ("en", "y")}
# The parts of speech of a word ending in a pronoun that the dictionary lists as a word of its own
# (`rendez-vous`, `qu'en-dira-t-on`), which the tables keep whole so the inversion rule never splits
# it. A verb, phrase or interjection made of a verb and its pronouns (`est-il`, `allez-y`,
# `excusez-moi`) is read by that rule as words, and stays out (D4).
_WORD_POS = frozenset({"noun", "adj", "adv", "pron", "prep"})
# How far a form of a form is followed (D3): `dirigée` → `dirigé` → *diriger* is one step. The
# dictionary's chains take two at most; a pair of entries pointing at each other (`pourparler` and
# `pourparlers`) stops here.
_FOLLOW_STEPS = 3
# wordfreq's list is read this many times the lemmas kept: enough past the cut for its words of
# the cut's frequency, and for the inflected forms it skips.
_WORDFREQ_DEPTH = 6

# Homographs a person decided (design D5), checked before any count: form → (lemma, reason). The
# reason is part of the rule: a row without one is not a decision. Editing this table changes the
# rules' sha256, so the tables must be reduced again, as for any rule.
#
# A row costs the other lemma its place in the pack: a lemma's own form always reads as itself, so
# keeping the noun `porte` would read « il porte » as the door. No homograph has a row, as in
# Spanish's; M8's cost — the dictionary nouns a verb's form takes — is listed in
# tables/fr-en/README.md. The rows below correct the source's copy errors, which no rule can tell
# from a meaning.
OVERRIDES = {
    "fatiguée": (
        "fatiguer",
        "a copy error: the English Wiktionary's verb entry reads « feminine singular of parlé »; its "
        "adjective entry and fatigué's tables make it fatigué's feminine, which reads as fatiguer",
    ),
    "bridée": (
        "bridé",
        "a copy error: the noun's form-of targets are « bridé » and « female slant », whose first word "
        "is English (« female equivalent of bridé, female slant »)",
    ),
    "quis": (
        "quérir",
        "a copy error: a sense of the verb entry reads « masculine plural of qui », a word with no verb "
        "entry; its other senses make quis quérir's past participle and past historic",
    ),
}


def nfc_lower(text):
    """A word lowercased and in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip().lower()).replace("’", "'")


def is_form(word):
    """Whether a word may be a form of the tables (D3, D4): a French token, and no plain word
    beginning with an elided piece — the pre-pass splits it whatever the pack lists. A hyphenated
    run beginning with one is read whole first, so it may be (`c'est-à-dire`)."""
    if not _TOKEN.fullmatch(word):
        return False
    return "-" in word or not _ELIDED_START.match(word)


def _form_targets(sense):
    """The words a sense makes its entry a form of: each `form_of` target's first word (kaikki
    writes `bel`'s as « beau used before a masculine noun… »)."""
    out = []
    for ref in sense.get("form_of") or ():
        words = (ref.get("word") if isinstance(ref, dict) else "") or ""
        if words.split():
            out.append(words.split()[0])
    return out


def _is_form_of(sense):
    return "form-of" in (sense.get("tags") or ()) or bool(_form_targets(sense))


_POST_1990 = re.compile(r"^post-1990 spelling of (\S+)")


def spelling_target(sense, word):
    """The word a sense says its entry only spells (D7), or None: a post-1990 spelling (`connait`,
    « post-1990 spelling of connaît »), or an ASCII spelling of a ligature (`coeur`, whose
    `alt_of` is *cœur*)."""
    glosses = sense.get("glosses") or [""]
    m = _POST_1990.match(glosses[0] or "")
    if m:
        return m.group(1).strip(".,;")
    alt = sense.get("alt_of") or ()
    target = (alt[0].get("word") or "") if alt and isinstance(alt[0], dict) else ""
    if target and target.lower() != word and target.lower().replace("œ", "oe").replace("æ", "ae") == word:
        return target
    return None


class Lexicon:
    """What the French section says of its words.

    - `candidates`: form → the words it may be a form of (its own word when it is a lemma);
    - `links`: form → candidate → the parts of speech of the entries linking them, which a form of
      a form follows (D3);
    - `lemmas`: the words with an entry that is not only a form of another;
    - `poses`: lemma → the parts of speech of its lemma entries;
    - `names`: the words whose only lemma entries are proper names.
    """

    def __init__(self):
        self.candidates = collections.defaultdict(set)
        self.links = collections.defaultdict(lambda: collections.defaultdict(set))
        self.lemmas = set()
        self.poses = collections.defaultdict(set)
        self._kinds = collections.defaultdict(set)

    @property
    def names(self):
        return {word for word, kinds in self._kinds.items() if kinds == {True}}

    def link(self, form, word, pos):
        self.candidates[form].add(word)
        self.links[form][word].add(pos)

    def read(self, entry):
        """One kaikki entry (D3, D4, D7).

        An elided piece is read by `ELISIONS` alone, and a gender or number marker is no inflection
        (`_MARKERS`). A word that is no form (`is_form`) is skipped whole. An entry whose every sense only spells another word is a form of that word, its
        inflections forms of it. Otherwise: an entry with a sense that is not a form-of is a lemma
        and its own candidate; the inflections it lists are its forms, but the bookkeeping, the
        doubtful ones and the multi-word constructions; its form-of senses make it a form of
        their targets.
        """
        word = nfc_lower(entry.get("word"))
        if word in ELISIONS:
            self.candidates[word].add(ELISIONS[word][0])
            return
        if not is_form(word):
            return
        pos = entry.get("pos")
        senses = entry.get("senses") or []
        spelt = [spelling_target(sense, word) for sense in senses]
        if spelt and all(spelt):
            for target in spelt:
                self.link(word, nfc_lower(target), pos)
            self._inflections(entry, word, pos)
            return
        if not (senses and all(_is_form_of(sense) for sense in senses)):
            self.lemmas.add(word)
            self.link(word, word, pos)
            self.poses[word].add(pos)
            self._kinds[word].add(pos == "name")
        self._inflections(entry, word, pos)
        for sense in senses:
            for target in _form_targets(sense):
                target = nfc_lower(target)
                if is_form(target):
                    self.link(word, target, pos)

    def _inflections(self, entry, word, pos):
        for inflection in entry.get("forms") or ():
            tags = set(inflection.get("tags") or ())
            form = nfc_lower(inflection.get("form"))
            if tags & _BOOKKEEPING or tags & _DOUBTFUL or not is_form(form) or form in ELISIONS:
                continue
            if form in _MARKERS and form != word:
                continue
            self.link(form, word, pos)


def read_kaikki(path, readings=None):
    """The French section, read entry by entry into a `Lexicon` — and, with `readings` (a
    `Readings`), each entry's grammar in the same pass (`read_readings`)."""
    lexicon = Lexicon()
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            lexicon.read(entry)
            if readings is not None:
                read_readings(entry, readings)
    return lexicon


def read_gsd_counts(paths):
    """How often each (form, lemma) pair stands in the treebank, lowercased, in NFC, the typographic
    apostrophe read as `'`."""
    counts = collections.Counter()
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for line in f:
                if not line.strip() or line.startswith("#"):
                    continue
                cols = line.rstrip("\n").split("\t")
                # Multi-word tokens (`du` = 1-2) and empty nodes (8.1) carry no lemma of their own.
                if len(cols) < 4 or "-" in cols[0] or "." in cols[0]:
                    continue
                counts[(nfc_lower(cols[1]), nfc_lower(cols[2]))] += 1
    return counts


def gsd_zipf(counts):
    """Each hyphenated lemma the treebank attests, by its own frequency there, as a Zipf value (its
    count per billion words, in log10) — the evidence a compound is ranked by (D6)."""
    total = sum(counts.values())
    per_lemma = collections.Counter()
    for (_, lemma), n in counts.items():
        if "-" in lemma:
            per_lemma[lemma] += n
    return {lemma: math.log10(n / total * 1e9) for lemma, n in per_lemma.items()}


def name_or_word(candidates, names, frequency):
    """The candidates, a form that is a proper name and also another word's keeping only the commoner
    reading, as Spanish's (fix-lingua-spanish-card-noise D1): `cette` reads as *ce*, not the town,
    `claire` as *clair*, while `paris` stays the city (5.71 against *pari*'s 4.12)."""
    for form, opts in candidates.items():
        words = opts - names
        if form in opts & names and words:
            opts.intersection_update({form} if frequency(form) > max(frequency(w) for w in words) else words)
    return candidates


def own_words(lexicon):
    """`du` and `des`, each a lemma of its own whatever its entries point at (M21, D4)."""
    for word in OWN_WORDS:
        lexicon.candidates[word] = {word}
        lexicon.lemmas.add(word)


def choose_lemma(form, options, counts, lemmas, frequency, overrides=OVERRIDES):
    """The one lemma a form maps to (D5, M8), among `options`.

    An override first, then the treebank's counts, then the form's own entry, then the lemma's
    frequency, then alphabetical order — so the result never depends on the source's order.
    """
    if not options:
        return None
    override = overrides.get(form)
    if override and override[0] in options:
        return override[0]
    return min(
        options,
        key=lambda lemma: (
            -counts.get((form, lemma), 0),
            not (lemma == form and form in lemmas),
            -frequency(lemma),
            lemma,
        ),
    )


def ends_in_pronoun(word, poses):
    """A hyphenated word of the dictionary whose last piece is a pronoun of the inversion rule, and
    which is a noun, an adjective, an adverb, a pronoun or a preposition (D4): `rendez-vous`,
    `qu'en-dira-t-on` — not `est-il` or `allez-y`, a verb and its pronouns."""
    return "-" in word and word.rsplit("-", 1)[1] in INVERSION_PRONOUNS and bool(poses.get(word, set()) & _WORD_POS)


def clitic_compound(form, lemma):
    """A verb form and the pronouns it is joined to by hyphens (`souviens-toi`, `allons-nous-en`,
    `sois-t'en`), of a lemma without a hyphen: the pre-pass reads them as words, which listing the
    run whole would stop (D4)."""
    if "-" not in form or "-" in lemma:
        return False
    return all(piece in _CLITIC_PIECES for piece in form.split("-")[1:])


def follow(word, pos, links, poses, choose, overrides=OVERRIDES):
    """What `word` reads as along one part of speech (D3): an override's lemma when a person decided
    the word (`fatiguée`, whose verb entry is a copy error, reads as *fatiguer* wherever a chain
    passes through it); otherwise the first choice (D5) among itself, when one of its lemma entries
    is of that part of speech, and the words its entries of that part of speech link it to — but a
    word whose entries are all of other parts of speech, which a chain along this one cannot reach.
    `word` itself when nothing of that part of speech is left: a participle's own inflections
    (`compromis`, invariable, lists itself) are no lemma entry."""
    if word in overrides:
        return overrides[word][0]
    along = {
        w
        for w, of in links.get(word, {}).items()
        if pos in of and w != word and (not links.get(w) or any(pos in p for p in links[w].values()))
    }
    if pos in poses.get(word, ()):
        along.add(word)
    return choose(word, along) if along else word


def reach(form, options, links, step):
    """A form's lemmas (D3): each candidate followed along the part of speech of the entry linking
    the form to it. A candidate that part of speech reads as a form of another word (`follow`)
    stands for that word instead, and the chain goes on along the same part of speech; one it reads
    as itself is a lemma the form reaches. `dirigée`, the feminine of the participle `dirigé` (a
    verb entry), itself the past participle of *diriger*, reaches *diriger*; `citée`, the feminine
    of the participle `cité`, reaches *citer*, not the noun *cité* (the city) the participle is
    spelt like; `étés`, the plural of the noun `été`, reaches the noun, not *être*, which `été`
    reads as through a verb entry. The form's own entry, and a candidate no entry links it to (an
    elided piece, `du`, `des`), stand as they are. `step(word, pos)` is `follow` over the lexicon."""
    reached = set()
    for candidate in options:
        linked = links.get(form, {}).get(candidate, set())
        if candidate == form or not linked:
            reached.add(candidate)
            continue
        for pos in linked:
            word = candidate
            for _ in range(_FOLLOW_STEPS):
                nxt = step(word, pos)
                if nxt == word:
                    break
                word = nxt
            reached.add(word)
    return reached


def ranks_for(inflected, want, lemmas, poses, compounds_zipf, frequency, top_n):
    """`lemma → rank`, dense, the `want` first kept (D6).

    wordfreq's order over its words that are French forms, neither its elision stems nor the split
    contractions nor a word `inflected` reads as another's. A hyphenated lemma wordfreq cannot rank
    — it answers a combination of the parts — is ranked when GSD attests it (`compounds_zipf`), at
    the lower of wordfreq's estimate and GSD's own frequency, after wordfreq's words of the same
    frequency and alphabetically among compounds; a word ending in a pronoun (`ends_in_pronoun`)
    GSD never meets takes the cut's last ranks, so the inversion rule never splits it.
    """
    order, seen = [], set()
    for word in top_n(want * _WORDFREQ_DEPTH):
        word = word.strip().lower()
        if word in WORDFREQ_STEMS or word in SPLIT_CONTRACTIONS or word in seen:
            continue
        if not is_form(word) or word in inflected:
            continue
        seen.add(word)
        order.append((frequency(word), word))
    floor = order[min(len(order), want) - 1][0] if order else 0
    tail = []
    for lemma in sorted(lemmas):
        if "-" not in lemma or lemma in seen or lemma in inflected:
            continue
        if lemma in compounds_zipf:
            estimate = min(frequency(lemma), compounds_zipf[lemma])
            if estimate >= floor:
                order.append((estimate, lemma))
        elif ends_in_pronoun(lemma, poses):
            tail.append(lemma)
    # A stable sort: wordfreq's own order among its words, the compounds after them at a tie.
    order.sort(key=lambda kept: -kept[0])
    kept = [word for _, word in order[: want - len(tail)]] + tail
    return {word: rank for rank, word in enumerate(kept, 1)}


def forms_for(lexicon, step, ranks, counts, frequency, overrides=OVERRIDES):
    """`form → lemma` over the ranked lemmas (D3–D6): each form's choice among the ranked lemmas it
    reaches; the split contractions and a verb joined to its pronouns left out; a form kept when it
    is its lemma's own, an elided piece, or attested by wordfreq; and each ranked lemma's own form."""
    forms = {}
    for form, options in lexicon.candidates.items():
        options = reach(form, options, lexicon.links, step)
        lemma = choose_lemma(form, {o for o in options if o in ranks}, counts, lexicon.lemmas, frequency, overrides)
        if lemma is None or form in SPLIT_CONTRACTIONS or clitic_compound(form, lemma):
            continue
        if lemma == form or form in ELISIONS or frequency(form) > 0:
            forms[form] = lemma
    for lemma in ranks:
        forms.setdefault(lemma, lemma)
    return forms


def reduce_forms(lexicon, counts, compounds_zipf, frequency, top_n, max_lemmas, overrides=OVERRIDES):
    """The forms table and the ranks: `form → lemma` over the kept lemmas, and `lemma → rank`.

    A first choice among every candidate says which forms are only inflected: they are never
    ranked. The ranks follow; a ranked word whose own form reads as another word once the forms of
    forms are followed (`reach`, whose chains choose as the first choice does) gives its rank to the
    next, until every ranked lemma's own form reads as itself (D6). A pack finds a lemma by its own
    form — the builder keys its rank, and later its gloss and level, by looking the lemma up as a
    form — so a lemma whose form reads elsewhere would lend its rank to that word. Every form of the
    noun *tenue*, `tenue` and `tenues`, reads as *tenir*; `donnée` reads as *donner*, and the noun
    *donnée*, which `données` alone still reaches, leaves the pack with it, as M8's nouns do (D5).
    """
    first = {
        form: choose_lemma(form, opts, counts, lexicon.lemmas, frequency, overrides)
        for form, opts in lexicon.candidates.items()
    }
    inflected = {form for form, lemma in first.items() if lemma != form}
    steps = {}

    def choose(word, options):
        return choose_lemma(word, options, counts, lexicon.lemmas, frequency, overrides)

    def step(word, pos):
        if (word, pos) not in steps:
            steps[(word, pos)] = follow(word, pos, lexicon.links, lexicon.poses, choose, overrides)
        return steps[(word, pos)]

    unreached = set()
    while True:
        ranks = ranks_for(inflected | unreached, max_lemmas, lexicon.lemmas, lexicon.poses, compounds_zipf, frequency, top_n)
        forms = forms_for(lexicon, step, ranks, counts, frequency, overrides)
        lost = {lemma for lemma in ranks if forms.get(lemma) != lemma}
        if not lost:
            return forms, ranks
        unreached |= lost


def unlisted_pronoun_tails(lexicon, forms):
    """The dictionary's words ending in a pronoun (`ends_in_pronoun`) that the forms table does not
    list: the inversion rule would split them. A reduction that leaves one out fails (D4)."""
    return sorted(word for word in lexicon.lemmas if ends_in_pronoun(word, lexicon.poses) and word not in forms)


def analyser_version(path=_ANALYSIS_RS):
    """French's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const FRENCH_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no FRENCH_ANALYZER_VERSION in {path}")
    return m.group(1)


# — The estimated levels (add-lingua-french-levels) —
#
# No French CEFR list can be shipped (FLELex is non-commercial), so French's levels are estimated
# from frequency, as Spanish's are (the programme's M7), and the pack says so. The commonest lemmas,
# in rank order, take the sizes of English's CEFR levels: measured on English, giving its 8,302 CEFR
# lemmas their levels this way by their own ranks agrees with the lists for 39.8 % of them, and
# within one level for 82.6 % (design D1). The sizes are es-fr's (`reduce-es-fr.py`, measured on
# en-fr's 2026-09-26 tables), kept here: an English update must not move French's levels
# unannounced, and moving them into reduce_common.py would move every pair's rule digest.
ENGLISH_BANDS = (("A1", 1020), ("A2", 1158), ("B1", 2015), ("B2", 2347), ("C1", 886), ("C2", 876))

# Which lemmas a CEFR list would hold is read from the English Wiktionary's French section, the
# source of the forms, never from a pair's glosses (design D2): the table does not wait for fr-en's
# glosses, and does not move when they land.
#
# A single character takes a level only as a word (D2, rule 2): a sense of another part of speech
# than a letter's, a symbol's or a name's that is no abbreviation — `à`, a preposition, and `y`, a
# pronoun; not `b` or `e`, a letter's name, a symbol or an abbreviation.
_LETTER_POS = frozenset({"character", "symbol", "name"})
_NOT_A_WORD_TAGS = frozenset({"abbreviation", "initialism", "acronym", "letter"})
# A sense that only spells another word (D2, rule 3), as the section's glosses open: a reader learns
# the word it spells (`etre` → *être*, `parceque`, `hazard` → *hasard*).
_SPELLING_OF = (
    "alternative spelling of",
    "obsolete spelling of",
    "archaic spelling of",
    "dated spelling of",
    "misspelling of",
    "alternative letter-case form of",
    "pronunciation spelling of",
    "nonstandard spelling of",
    "eye dialect spelling of",
    "obsolete form of",
    "archaic form of",
    "dated form of",
    "rare spelling of",
    "uncommon spelling of",
    "informal spelling of",
)
# Why a ranked lemma takes no level, in the order the rules are read (D2, D3): the section does not
# know it as a word, or knows it only as a name; its own form reads as another lemma; it is a letter
# that is no word; it only spells another word.
LEVEL_RULES = ("unknown", "name", "elsewhere", "letter", "spelling")


def word_senses(entry):
    """`(word, [(part of speech, first gloss, tags)])`: the senses of a kaikki entry that are not a
    form of another word (`_is_form_of`), its word as the tables write it."""
    pos = entry.get("pos")
    return nfc_lower(entry.get("word")), [
        (pos, (sense.get("glosses") or [""])[0] or "", frozenset(sense.get("tags") or ()))
        for sense in entry.get("senses") or ()
        if not _is_form_of(sense)
    ]


def read_level_senses(path):
    """`word → senses`, over the French section: each word the section gives a sense that is not a
    form of another word, with those senses (`word_senses`). Its own pass, so the forms' reading is
    not edited for it."""
    senses = collections.defaultdict(list)
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            word, own = word_senses(entry)
            if own:
                senses[word].extend(own)
    return senses


def _a_word(word, senses):
    """Whether a single character is a word (rule 2); a longer word always is."""
    if len(word) != 1:
        return True
    return any(
        pos not in _LETTER_POS and not tags & _NOT_A_WORD_TAGS and not gloss.lower().startswith("abbreviation of")
        for pos, gloss, tags in senses
    )


def _only_spells_another(senses):
    """Whether every sense but a name's only spells another word (rule 3)."""
    own = [(gloss, tags) for pos, gloss, tags in senses if pos != "name"]
    return bool(own) and all(gloss.lower().startswith(_SPELLING_OF) or "misspelling" in tags for gloss, tags in own)


def no_level(lemma, forms, senses):
    """Why a ranked lemma takes no level (`LEVEL_RULES`), or None when a CEFR list would hold it
    (design D2):
    1. the section gives it no sense that is not a form of another word (`the`, `etc`, `km`), or
       only a name's (`paris`, `france`) — but `du` and `des`, words of their own (M21);
    4. its own form reads as another lemma in the forms table: the builder keys a level by looking
       the lemma up as a form, so `donnée`, read as *donner*, would give *donner* its level (D3);
    2. it is a single character the section gives no word's sense (`b`, `e`);
    3. every sense it is given only spells another word (`etre`, `parceque`)."""
    own = senses.get(lemma, ())
    if lemma not in OWN_WORDS:
        if not own:
            return "unknown"
        if {pos for pos, _, _ in own} == {"name"}:
            return "name"
    if forms.get(lemma, lemma) != lemma:
        return "elsewhere"
    if not _a_word(lemma, own):
        return "letter"
    if _only_spells_another(own):
        return "spelling"
    return None


def estimated_levels(ranks, forms, senses, bands=ENGLISH_BANDS):
    """`(lemma → level, rule → lemmas)`: in rank order then lemma, each band's size to the ranked
    lemmas a CEFR list would hold (`no_level`), and the lemmas each rule left out before the last
    level was given."""
    slots = [level for level, size in bands for _ in range(size)]
    levels, left_out = {}, {rule: [] for rule in LEVEL_RULES}
    for lemma in sorted(ranks, key=lambda lemma: (ranks[lemma], lemma)):
        if len(levels) == len(slots):
            break
        why = no_level(lemma, forms, senses)
        if why:
            left_out[why].append(lemma)
        else:
            levels[lemma] = slots[len(levels)]
    return levels, left_out


def levels_report(levels, left_out, ranks):
    """The reduction's line on the levels: each level's rank span, and how many lemmas each rule
    left out."""
    spans = []
    for level, _ in ENGLISH_BANDS:
        at = [ranks[lemma] for lemma, lvl in levels.items() if lvl == level]
        if at:
            spans.append(f"{level} {len(at)} ({min(at)}–{max(at)})")
    out = ", ".join(f"{rule} {len(lemmas)}" for rule, lemmas in left_out.items())
    return f"levels={len(levels)} estimated: {'; '.join(spans)}; left out: {out}"


# — French's word grammar (add-lingua-french-grammar-tables) —
#
# Every form kaikki lists carries its grammar as tags (`parlions`: first-person, imperfect,
# indicative, plural). They become Universal Dependencies tags, the vocabulary the pack's grammar
# sections read (add-lingua-word-grammar D1):
# `VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin`. The rules read the same entries as
# the forms, in the same pass (`read_kaikki`), and the forms and ranks once chosen (design D1, D2):
# they choose neither.

# kaikki's parts of speech whose forms carry readings, as UPOS: an article is a determiner, as UD
# writes it (D4); every verb is `VERB`, `AUX` being a role in a sentence a card cannot see (D3).
_UPOS = {"verb": "VERB", "noun": "NOUN", "adj": "ADJ", "det": "DET", "article": "DET", "pron": "PRON", "num": "NUM"}
# kaikki's moods and tenses, in the order they are read; the passé simple is `historic` (D3).
_MOODS = (("conditional", "Cnd"), ("imperative", "Imp"), ("subjunctive", "Sub"))
_TENSES = (("historic", "Past"), ("imperfect", "Imp"), ("future", "Fut"), ("present", "Pres"))
_PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
_GENDER = {"masculine": "Masc", "feminine": "Fem"}
# `fr-noun`'s gender argument: the genders a noun takes, and whether its headword is a plural.
_NOUN_GENDER = {
    "m": (("Masc",), False),
    "f": (("Fem",), False),
    "mf": (("Masc", "Fem"), False),
    "mfbysense": (("Masc", "Fem"), False),
    "mfequiv": (("Masc", "Fem"), False),
    "m,f": (("Masc", "Fem"), False),
    "f,m": (("Masc", "Fem"), False),
    "m-p": (("Masc",), True),
    "f-p": (("Fem",), True),
    "mf-p": (("Masc", "Fem"), True),
}
# A determiner's, pronoun's or numeral's row reads only when it says nothing but gender, number and
# degree: a personal pronoun's head lists its other persons and cases (`il`: dative `lui`, possessive
# `son`), which are other words, not inflections (D4).
_NOMINAL_ONLY = frozenset(
    {"masculine", "feminine", "singular", "plural", "superlative", "comparative", "before-vowel", "form-of"}
    | {"invariable"}
)
# A row or sense that gives no reading (D5): the tables' bookkeeping, the tags change 43 leaves out
# of the forms (a form a source marks doubtful is no inflection, so no reading either), and what
# kaikki could not parse.
_NOT_A_READING = _BOOKKEEPING | _DOUBTFUL | {"error-unrecognized-form"}
# A form's own sense marked as a region's or a register's reads nothing (D5): `été` is Louisiana's
# past participle of *aller*, and a standard card would otherwise say so.
_REGISTERS = frozenset(
    {
        "Louisiana", "Quebec", "Canada", "Belgium", "Switzerland", "Africa", "Acadia", "Cajun", "France",
        "Haiti", "North-America", "Canadian", "regional", "dialectal", "Lorraine", "Provence", "Marseille",
        "Normandy", "Picardy", "slang", "colloquial", "informal", "Internet", "vulgar", "Jersey", "Guernsey",
        "Réunion", "Louisiana-French",
    }
)
# The regions a sense may be marked with: a word whose every sense is one region's is named on no
# standard card (D6: *vader*, a Louisiana verb, would be named on `va`).
_REGIONS = frozenset(
    {
        "Louisiana", "Quebec", "Canada", "Belgium", "Switzerland", "North-America", "regional", "dialectal",
        "Africa", "Rwanda", "Morocco", "Canadian", "Congo", "Antilles", "Lyon", "Luxembourg", "Vietnam",
        "Lorraine", "Alsace", "Montreal", "Normandy", "New-England", "Acadia", "Cajun", "Haiti", "Réunion",
        "Provence", "Marseille", "Picardy", "Jersey", "Guernsey", "Senegal", "Ivory-Coast", "Cameroon",
        "Algeria", "Tunisia", "Lebanon", "Belgian", "Swiss", "Québec", "Ontario", "Manitoba", "New-Brunswick",
        "Burundi",
    }
)
# The head template of a past participle's own entry (`dirigé`), whose rows are its agreed forms.
_PARTICIPLE_HEAD = "fr-past participle"
# A determiner's or pronoun's head naming a plural headword (`tes`), whose table lists the singular.
_PLURAL_HEADS = frozenset({"p", "m-p", "f-p", "mf-p"})
_LETTER_GLOSS = re.compile(r"(?:the )?name of the (?:[\w-]+ )?(?:script )?(?:letter|digraph)\b", re.IGNORECASE)
# A pronominal verb's row, as its table writes it: the pronoun before (`m'évanouis`, `nous
# évanouissons`) or after a hyphen in the imperative (`évanouis-toi`).
_PRONOUN_BEFORE = re.compile(r"^(?:(?:me|te|se|nous|vous) |[mts]')(.+)$")
_PRONOUN_AFTER = re.compile(r"^(.+?)-(?:toi|nous|vous)$")


def ud_tag(upos, features):
    """A Universal Dependencies tag: the part of speech, then the features sorted by name."""
    return "|".join([upos, *(f"{name}={value}" for name, value in sorted(features.items()))])


def parse_tag(tag):
    """`ud_tag`'s inverse: the part of speech and the features."""
    upos, *features = tag.split("|")
    return upos, dict(feature.split("=", 1) for feature in features)


def _agreement(tags):
    """Gender and number, as a participle's or an adjective's tags state them."""
    out = {}
    genders = [_GENDER[tag] for tag in tags if tag in _GENDER]
    if len(genders) == 1:
        out["Gender"] = genders[0]
    if "plural" in tags:
        out["Number"] = "Plur"
    elif "singular" in tags:
        out["Number"] = "Sing"
    return out


def _past_participle(tags):
    """A past participle's features: in French's tables the bare row is the masculine singular,
    not a repeat of it (D3)."""
    agreement = _agreement(tags)
    return {
        "VerbForm": "Part",
        "Tense": "Past",
        "Gender": agreement.get("Gender", "Masc"),
        "Number": agreement.get("Number", "Sing"),
    }


def verb_features(tags):
    """Every reading a verb's row or sense names (D3), as feature sets, none when it names none.

    - The passé simple (`historic past`) is the indicative past; the conditional and the
      imperative take no tense, as Spanish's tables write them (D7: the moods' merge of the card).
    - The present participle (`gerund participle present`) is `VerbForm=Part|Tense=Pres`, as UD
      French writes it; a past participle reads with its agreement, the bare row and a sense that
      names an agreement without `past` (`feminine singular of dirigé`) as the past one.
    - A sense merging persons, numbers or moods (`first/third-person singular present
      indicative/subjunctive`) reads as each of them; a table's row names one.
    - The imperative has no third person; the subjunctive no future or passé simple.
    """
    if "participle" in tags:
        return [{"VerbForm": "Part", "Tense": "Pres"}] if "present" in tags else [_past_participle(tags)]
    if "infinitive" in tags:
        return [{"VerbForm": "Inf"}]
    moods = [mood for kaikki, mood in _MOODS if kaikki in tags]
    tense = next((ud for kaikki, ud in _TENSES if kaikki in tags), None)
    if "indicative" in tags or (not moods and tense):
        moods.append("Ind")
    persons = [_PERSON[tag] for tag in tags if tag in _PERSON]
    numbers = [number for kaikki, number in (("singular", "Sing"), ("plural", "Plur")) if kaikki in tags]
    out = []
    for mood, person, number in itertools.product(moods, persons, numbers):
        features = {"VerbForm": "Fin", "Mood": mood, "Person": person, "Number": number}
        if mood in ("Ind", "Sub"):
            if tense is None or (mood == "Sub" and tense not in ("Pres", "Imp")):
                continue
            features["Tense"] = tense
        elif mood == "Imp" and person == "3":
            continue
        out.append(features)
    return out


def nominal_features(tags, genders=()):
    """The features of a noun's, adjective's, determiner's, pronoun's or numeral's form, one set per
    gender: its number (singular unless kaikki says plural), its degree, and its gender — the form's
    own, none when it names both, or each of the lemma's for a form kaikki gives none (a noun's
    plural)."""
    base = {"Number": "Plur" if "plural" in tags else "Sing"}
    if "superlative" in tags:
        base["Degree"] = "Sup"
    elif "comparative" in tags:
        base["Degree"] = "Cmp"
    own = [_GENDER[tag] for tag in tags if tag in _GENDER]
    if len(own) == 1:
        return [{**base, "Gender": own[0]}]
    if own or not genders:
        return [base]
    return [{**base, "Gender": gender} for gender in genders]


def reading_tags(upos, tags, genders=(), participle=False):
    """The UD tags kaikki's tags name for a form of `upos`; none when they name no reading.

    A past participle's own entry (`participle`) lists its agreed forms (`dirigée`: feminine), which
    read as the participle with that agreement. A determiner's, pronoun's or numeral's row reads only
    when it names agreement alone, and a pronoun's only with a gender (D4)."""
    if upos == "VERB":
        if participle and not ({"participle", "infinitive"} & tags) and not (set(_PERSON) & tags):
            return [ud_tag(upos, _past_participle(tags | ({"singular"} if "plural" not in tags else set())))]
        return [ud_tag(upos, features) for features in verb_features(tags)]
    if upos in ("PRON", "DET", "NUM") and not tags <= _NOMINAL_ONLY:
        return []
    if upos == "PRON" and not tags & set(_GENDER):
        return []
    return [ud_tag(upos, f) for f in nominal_features(tags, genders) if "Degree" not in f or upos == "ADJ"]


def noun_genders(entry):
    """A noun's genders, and whether its headword is a plural: `fr-noun`'s argument, else the
    genders its senses are tagged with (D4)."""
    for head in entry.get("head_templates") or ():
        if head.get("name") == "fr-noun":
            known = _NOUN_GENDER.get((head.get("args") or {}).get("1"))
            if known:
                return known
    tags = set()
    for sense in entry.get("senses") or ():
        tags.update(sense.get("tags") or ())
    return tuple(_GENDER[tag] for tag in ("masculine", "feminine") if tag in tags), False


def invariable_noun(entry):
    """A noun the dictionary gives one form for (D4): `fr-noun`'s plural `#`, or a sense tagged
    `invariable` and no plural listed (`temps`, `fois`, `bras`, `vis`)."""
    args = {}
    for head in entry.get("head_templates") or ():
        if head.get("name") == "fr-noun":
            args = head.get("args") or {}
    if args.get("2") == "#":
        return True
    senses = entry.get("senses") or []
    return any("invariable" in (sense.get("tags") or ()) for sense in senses) and not any(
        "plural" in (inflection.get("tags") or ()) for inflection in entry.get("forms") or ()
    )


def adjective_agrees(entry):
    """Whether an adjective has a feminine of its own (`grande`), unlike `rapide`."""
    for inflection in entry.get("forms") or ():
        tags = set(inflection.get("tags") or ())
        if "feminine" in tags and "masculine" not in tags:
            return True
    return False


def _names_a_letter(sense):
    """Whether `sense` is a letter's name (`elle`, the letter L, whose plural `elles` a card would
    name beside the pronoun's): kaikki says so by its tags, its category or its gloss."""
    if {"letter", "name"} <= set(sense.get("tags") or ()):
        return True
    for category in sense.get("categories") or ():
        name = category if isinstance(category, str) else category.get("name", "")
        if "letter names" in str(name):
            return True
    return any(_LETTER_GLOSS.search(gloss) for gloss in sense.get("glosses") or ())


def _alternative_or_neologism(sense):
    """A sense that is only an alternative form of another word, or a neologism (D5)."""
    tags = set(sense.get("tags") or ())
    return "alt-of" in tags or bool(sense.get("alt_of")) or "neologism" in tags


def without_pronoun(form):
    """A pronominal verb's row as the bare form French's pre-pass leaves (D3): `s'évanouit` →
    `évanouit`, `nous évanouissons` → `évanouissons`, `évanouis-toi` → `évanouis`; else None."""
    m = _PRONOUN_BEFORE.match(form) or _PRONOUN_AFTER.match(form)
    if m and _TOKEN.fullmatch(m.group(1)) and " " not in m.group(1):
        return m.group(1)
    return None


class Readings:
    """The readings kaikki states, by (form, lemma), and what the rules need of its entries.

    - `table`: a lemma's table — its own form, its inflections —, and a past participle's own
      entry's agreed forms; `senses`: a form's own entry's senses, for the pairs no table lists
      (D5);
    - `poses`: lemma → the parts of speech of its lemma entries, among those read;
    - `standard`: the words with a lemma entry that is not only regional (D6).
    """

    def __init__(self):
        self.table = collections.defaultdict(set)
        self.senses = collections.defaultdict(set)
        self.poses = collections.defaultdict(set)
        self.standard = set()

    def pairs(self):
        """`(form, lemma) → tags`: a table's reading of a pair wins over a form entry's senses."""
        out = {pair: set(tags) for pair, tags in self.senses.items()}
        out.update((pair, set(tags)) for pair, tags in self.table.items())
        return out


def read_readings(entry, readings):
    """One entry's readings into `readings`, word and form lowercased and in NFC (D3–D5).

    Never from a capitalised headword (`CE`, `LE` are other words), nor from an entry whose every
    sense is an alternative form of another word or a neologism (`estre`, archaic spelling of
    *être*, whose table would make `est` its form). A spelling variant reads as the word it spells
    (`coeurs` → *cœur*). A form's own entry gives each sense toward its target, but a doubtful,
    regional or register-marked one; a past participle's entry gives its agreed forms too. A lemma's
    entry gives a noun's or an adjective's own form, and its table's rows: a pronominal verb's
    without their pronoun, never a compound tense (a multi-word construction, doubtful), a letter's
    plural, a feminine noun's masculine (`déesse`: `dieu`), nor a plural-headed determiner's or
    pronoun's table (`tes`)."""
    upos = _UPOS.get(entry.get("pos"))
    word = nfc_lower(entry.get("word"))
    if upos is None or not _TOKEN.fullmatch(word):
        return
    raw = unicodedata.normalize("NFC", (entry.get("word") or "").strip())
    if raw != raw.lower():
        return
    senses = entry.get("senses") or []
    spelt = [spelling_target(sense, word) for sense in senses]
    spelt = nfc_lower(spelt[0]) if spelt and all(spelt) and len({nfc_lower(t) for t in spelt}) == 1 else None
    if spelt is None and senses and all(_alternative_or_neologism(sense) for sense in senses):
        return
    heads = entry.get("head_templates") or ()
    participle = upos == "VERB" and any(head.get("name") == _PARTICIPLE_HEAD for head in heads)
    home = spelt or word
    if spelt is None and senses and all(_is_form_of(sense) for sense in senses):
        for sense in senses:
            tags = set(sense.get("tags") or ())
            if tags & _NOT_A_READING or tags & _REGISTERS:
                continue
            for target in _form_targets(sense):
                target = nfc_lower(target)
                if _TOKEN.fullmatch(target):
                    readings.senses[(word, target)].update(reading_tags(upos, tags))
        if not participle:
            return
    elif spelt is None:
        readings.poses[word].add(upos)
        if not all(set(sense.get("tags") or ()) & _REGIONS for sense in senses):
            readings.standard.add(word)
    genders, plural = noun_genders(entry) if upos == "NOUN" else ((), False)
    if upos == "NOUN":
        own = {"Number": "Plur" if plural else "Sing"}
        for features in [{**own, "Gender": gender} for gender in genders] or [own]:
            readings.table[(word, home)].add(ud_tag(upos, features))
        # A letter's name keeps its own form, but none of its inflections: `elles` is the pronoun's,
        # not the plural of the letter L (add-lingua-spanish-word-card D7).
        if senses and all(_names_a_letter(sense) for sense in senses):
            return
        if not plural and invariable_noun(entry):
            for features in [{"Number": "Plur", "Gender": gender} for gender in genders] or [{"Number": "Plur"}]:
                readings.table[(word, home)].add(ud_tag(upos, features))
    elif upos == "ADJ":
        plural_only = all({"plural", "plural-only"} & set(sense.get("tags") or ()) for sense in senses)
        own = {"Number": "Plur" if plural_only else "Sing", **({"Gender": "Masc"} if adjective_agrees(entry) else {})}
        readings.table[(word, home)].add(ud_tag(upos, own))
    if upos in ("DET", "PRON") and any((head.get("args") or {}).get("g") in _PLURAL_HEADS for head in heads):
        return
    inflections = entry.get("forms") or ()
    pronominal = upos == "VERB" and any(
        "infinitive" in (inflection.get("tags") or ()) and without_pronoun(nfc_lower(inflection.get("form")))
        for inflection in inflections
    )
    for inflection in inflections:
        tags = set(inflection.get("tags") or ())
        form = nfc_lower(inflection.get("form"))
        if tags & _NOT_A_READING:
            continue
        bare = without_pronoun(form) if upos == "VERB" and ("reflexive" in tags or pronominal) else None
        if bare is not None:
            form, tags = bare, tags - {"reflexive"}
        elif not _TOKEN.fullmatch(form) or "reflexive" in tags:
            continue
        if upos == "NOUN" and "masculine" in tags and "plural" not in tags:
            continue
        readings.table[(form, home)].update(reading_tags(upos, tags, genders, participle))


def _kind(tag):
    """A reading's part of speech and verb form: what a form of a form is not added over (D5)."""
    upos, features = parse_tag(tag)
    return upos, features.get("VerbForm")


def compose(outer, inner):
    """A form of a form (D5): the outer reading — of the word the form is a form of — with the inner
    one's agreement, along one part of speech, a verb's only through a participle; None otherwise."""
    upos, features = parse_tag(outer)
    inner_upos, inner_features = parse_tag(inner)
    if upos != inner_upos:
        return None
    if upos == "VERB" and (features.get("VerbForm") != "Part" or inner_features.get("VerbForm") not in (None, "Part")):
        return None
    for name in ("Gender", "Number"):
        if name in inner_features:
            features[name] = inner_features[name]
    return ud_tag(upos, features)


def set_aside(readings, overrides=OVERRIDES):
    """The links a reviewed override row sets aside as the source's copy error (D5): an overridden
    form's own entry linking it to another word than the row's, which no lemma's table lists —
    `fatiguée`'s verb entry, « feminine singular of parlé », gives no reading toward *parlé*, and so
    none toward *parler* through it."""
    return {
        (form, target)
        for (form, target) in readings.senses
        if form in overrides and target != overrides[form][0] and (form, target) not in readings.table
    }


def grammar_rows(readings, forms, ranks, overrides=OVERRIDES):
    """`grammar.tsv`'s rows, sorted: the readings of the forms `forms` holds, under the lemmas
    `ranks` keeps (D2), a form's of its own lemma marked `-`, another's `other` (D6).

    - A form of a form along one part of speech reads as the word it is a form of reads, with its
      own agreement (`dirigée` → `dirigé` → *diriger*), through the form's own lemma too — unless
      the form already reads as that word in that part of speech and verb form (`les` is no
      feminine through `la`) (D5).
    - A reading's part of speech is one the dictionary holds its lemma as, when it holds the lemma
      as one of those read: a participle filed under a noun's spelling names its verb instead
      (D5), and `venait` names no *came*, which the French section holds as a noun.
    - `other` only toward an entry of the dictionary that is not only regional (D6): `irait` names
      no *would*, `va` no *vader*.
    - Never through a link an override row sets aside (`set_aside`)."""
    excluded = set_aside(readings, overrides)
    by_form = collections.defaultdict(dict)
    for (form, lemma), tags in readings.pairs().items():
        if tags and (form, lemma) not in excluded:
            by_form[form][lemma] = tags
    rows = set()
    for form, own in forms.items():
        direct = by_form.get(form, {})
        options = {lemma: set(tags) for lemma, tags in direct.items()}
        for inner_word, inner_tags in direct.items():
            if inner_word == form:
                continue
            for lemma, outer_tags in by_form.get(inner_word, {}).items():
                if lemma in (inner_word, form):
                    continue
                kinds = {_kind(tag) for tag in direct.get(lemma, ())}
                made = {compose(outer, inner) for outer in outer_tags for inner in inner_tags} - {None}
                made = {tag for tag in made if _kind(tag) not in kinds}
                if made:
                    options.setdefault(lemma, set()).update(made)
        for lemma, tags in options.items():
            if lemma not in ranks:
                continue
            held = readings.poses.get(lemma)
            if held:
                tags = {tag for tag in tags if parse_tag(tag)[0] in held}
            mark = "-" if lemma == own else "other"
            if mark == "other" and (lemma not in readings.poses or lemma not in readings.standard):
                continue
            rows.update((form, lemma, tag, mark) for tag in tags)
    return sorted(f"{form}\t{lemma}\t{tag}\t{mark}\n" for form, lemma, tag, mark in rows)


# — fr-en's native side (add-lingua-pack-fr-en) —
#
# French glossed in English, from the section the forms come from (D1): its senses, read through
# the English edition's rules as es-en's are (D2), then the rules every pair shares. No translation
# table (D3, the module's doc). Nothing of the studied side moves: the glosses are keyed by the
# lemmas the reduction has just ranked, each its own form's lemma (`reduce_forms`), so that the
# builder, which keys a gloss by looking its lemma up as a form, files no gloss under another word.

# French as the shared native rules read it (D1): a French word, whole — change 43's token pattern
# — and French's seven coordinating conjunctions, which make kaikki's `conj` a CCONJ in a sense run.
FR = common.Studied(
    code="fr",
    token=_TOKEN,
    # Not read by the native side; kept for the shared rules' interface.
    form_of_target=re.compile(rf"([{_LETTERS}]+)\s*\.?$"),
    coordinators=frozenset({"et", "ou", "mais", "ni", "or", "car", "donc"}),
)

EDITION = english.EN

# What the native side reads of an entry (`reduce_common._read_entries`, `reduce_expressions`,
# `without_letter_senses`, `english.read_as_meanings`): the rest — the inflection tables, most of
# the section's bytes — is the studied side's, read in its own pass.
_ENTRY_FIELDS = ("word", "pos", "senses")
_SENSE_FIELDS = ("glosses", "tags", *EDITION.pointer_fields)


def native_fields(src, dst):
    """The section cut down to what the native side reads (`_ENTRY_FIELDS`, `_SENSE_FIELDS`),
    written to `dst`: the shared rules read the file several times, and the inflection tables are
    most of its 510 MB. An entry's `pos` is kept as written, and its `word` too but for a
    typographic apostrophe, read as `'` as French's forms are and as the core reads a page
    (« nombre d’oxydation »); its case is kept, which the shared rules read an acronym by and the
    English edition a place's name. A line that is no JSON object is left out, and counted:
    `(dst, dropped)`."""
    dropped = 0
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if not isinstance(entry, dict):
                dropped += 1
                continue
            cut = {field: entry[field] for field in _ENTRY_FIELDS if field in entry}
            if isinstance(cut.get("word"), str):
                cut["word"] = cut["word"].replace("’", "'")
            cut["senses"] = [
                {field: sense[field] for field in _SENSE_FIELDS if field in sense}
                for sense in entry.get("senses") or ()
                if isinstance(sense, dict)
            ]
            out.write(json.dumps(cut, ensure_ascii=False) + "\n")
    return dst, dropped


# French's expressions (D11): what add-lingua-french-expression-keys hands this reducer.
#
# An expression's sense that only points (D11, 2): it names another spelling (« post-1990 spelling of
# crème fraîche »), an inverted form (« subject-inverted form of il y a ») or the pieces the
# expression is written with (« que + elle », « contraction of que + il »). Pointers the dump tags
# are the shared rules' already (`reduce_common._is_form_of`).
_POINTS = re.compile(
    r"^(?:(?:post-1990 spelling|subject-inverted form) of\b|(?:contraction of )?[^\s+;]+(?: \([^)]*\))? \+ )",
    re.IGNORECASE,
)
# The meaning a spelling or an inversion writes after its target, past a semicolon or a colon
# (« subject-inverted form of il y a; is there? are there? »): kept.
_POINTS_THEN = re.compile(
    r"^(?:post-1990 spelling|subject-inverted form) of [^;:]+[;:]\s*(.+)$", re.IGNORECASE | re.DOTALL
)
# Expressions left out by name, each with its reason: a person's decision, as an override row is.
# Editing this table changes the rules' sha256.
LEFT_OUT = {
    "à la": (
        "its one sense, « in the style or manner of », is met only before a word that completes it, "
        "and the section writes those uses as entries of their own (« à la carte », « à la maison »); "
        "as a key it is French's commonest preposition and article, and every « au » before "
        "add-lingua-french-expression-keys keyed it as `à le`"
    ),
}


def expression_word(headword):
    """A headword as the shared rules key an expression: lowercased, single spaces."""
    return re.sub(r"\s+", " ", (headword or "").strip().lower())


def split_word(word, forms):
    """Whether `word` is a single word French's tokenisation splits (D11, 1): no space, an apostrophe
    or a hyphen (`d'abord`, `c'est`, `allez-y`), a French token, and no form of the tables — a form
    is looked up whole (`aujourd'hui`). The builder keys those that read as two tokens or more."""
    return " " not in word and ("'" in word or "-" in word) and bool(_TOKEN.fullmatch(word)) and word not in forms


def is_expression(word, forms):
    """Whether a headword is one of fr-en's expressions: words of French with a space between them,
    as the shared rules read them (`reduce_common.reduce_expressions`), or a word the tokenisation
    splits (`split_word`)."""
    if " " in word:
        return all(_TOKEN.fullmatch(part) for part in word.split(" "))
    return split_word(word, forms)


def _meaning_of_pointer(sense):
    """`sense` with its pointer read out (D11, 2): None when it only points, the sense read as the
    meaning written after its target when it has one, the sense itself when it does not point."""
    glosses = sense.get("glosses")
    if not isinstance(glosses, list) or not glosses or not isinstance(glosses[0], str):
        return sense
    gloss = glosses[0].strip()
    if not gloss or common._is_form_of(sense, gloss, edition=EDITION) or not _POINTS.match(gloss):
        return sense
    after = _POINTS_THEN.match(gloss)
    return {**sense, "glosses": [after.group(1), *glosses[1:]]} if after else None


def expression_senses(src, dst, forms):
    """The section's expressions as fr-en reads them, written to `dst` — a pass before the shared
    rules read the file (D11, 2 and 3), over the expressions alone (`is_expression`): no lemma's
    entry is a headword with a space or a word that is no form, so `gloss.tsv` and `senses.tsv` do
    not move with it.

    - A sense that only points at another spelling, an inverted form or the pieces it is written
      with goes; a meaning written after its target stays, in its place (« y a-t-il » « is there?
      are there? »). An expression no sense is left to is none: the traditional spelling, keyed
      alike, stands alone (« crème fraîche »), and « jusqu'au soir » meets `jusqu'à` « until », not
      « jusque + au ».
    - The expressions of `LEFT_OUT` go, each with its reason (`à la`).

    A line this pass cannot read is written as it is, and so is an entry it does not change."""
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            headword = entry.get("word") if isinstance(entry, dict) else None
            word = expression_word(headword) if isinstance(headword, str) else None
            if word is None or not is_expression(word, forms):
                out.write(line if line.endswith("\n") else line + "\n")
                continue
            if word in LEFT_OUT:
                continue
            senses = entry.get("senses")
            if isinstance(senses, list):
                read = [_meaning_of_pointer(s) if isinstance(s, dict) else s for s in senses]
                read = [s for s in read if s is not None]
                if read != senses:
                    line = json.dumps({**entry, "senses": read}, ensure_ascii=False) + "\n"
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def split_words(path, forms, maxlen=common.EXPRESSION_GLOSS_LEN, per_sense=42, max_senses=3):
    """The words French's tokenisation splits, glossed as expressions (D11, 1): `word → gloss`, read
    with `reduce_common.reduce_expressions`' sense rules — a name's entry, a pointer and a sense
    nothing survives of left out, each sense cleaned to `per_sense` characters, `max_senses` joined
    across the word's entries. The reducer offers every such word; the builder keys those that read
    as two to seven tokens (add-lingua-french-expression-keys)."""
    entries = {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(entry, dict) or (entry.get("pos") or "") == "name":
                continue
            word = expression_word(entry.get("word") if isinstance(entry.get("word"), str) else "")
            if not split_word(word, forms):
                continue
            senses = []
            for sense in entry.get("senses") or ():
                glosses = sense.get("glosses") if isinstance(sense, dict) else None
                texts = isinstance(glosses, list) and glosses and isinstance(glosses[0], str)
                gloss = glosses[0].strip() if texts else ""
                if not gloss or common._is_form_of(sense, gloss, edition=EDITION):
                    continue
                cleaned = common.clean_gloss(gloss, per_sense, edition=EDITION)
                if cleaned:
                    senses.append(cleaned)
            if senses:
                entries.setdefault(word, []).append(senses)
    out = {}
    for word, per_entry in entries.items():
        joined = common._join_senses(per_entry, maxlen, max_senses)
        if joined:
            out[word] = joined
    return out


def native_side(work, kaikki, ranks, forms):
    """fr-en's native side over the section in `kaikki` (D1, D2, D11): `(glosses, runs, expressions,
    stats)`, the glosses and runs keyed by the ranked lemmas (`ranks`), the expressions the section's
    headwords with a space and the words the tokenisation splits. The intermediate files are written
    in `work`."""
    entries, dropped = native_fields(kaikki, os.path.join(work, "kaikki-French-senses.jsonl"))
    entries = common.without_letter_senses(entries, os.path.join(work, "kaikki-French-words.jsonl"), edition=EDITION)
    entries = english.without_letter_headwords(entries, os.path.join(work, "kaikki-French-headwords.jsonl"))
    entries = english.read_as_meanings(entries, os.path.join(work, "kaikki-French-meanings.jsonl"))
    entries = english.merge_same_pos_etymologies(entries, os.path.join(work, "kaikki-French-merged.jsonl"))
    entries = expression_senses(entries, os.path.join(work, "kaikki-French-expressions.jsonl"), forms)
    # No fallback: no translation table glosses a French word or expression (D3).
    glosses, runs, expressions, primary = common.native_tables(
        entries, ranks, studied=FR, edition=EDITION, fallbacks=[]
    )
    split = split_words(entries, forms)
    expressions.update(split)
    return glosses, runs, expressions, {"primary": primary, "split": len(split), "dropped": dropped}


NOTICE = """Cymbra Lingua data pack — FR->EN attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), French section: CC BY-SA 4.0 + GFDL —
the forms and their lemmas, and the English glosses of French words and expressions, from its
senses.

wordfreq (French frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 — the commonest lemmas and which forms are attested.

UD French-GSD, the Universal Dependencies French-GSD treebank
(https://github.com/UniversalDependencies/UD_French-GSD): CC BY-SA 4.0 — how often a form stands
for each of its lemmas, to choose one, and how often a hyphenated word occurs, to rank it.

The levels are estimated, not taken from a CEFR list: the commonest lemmas by wordfreq's ranks take
the sizes of English's CEFR levels, the English Wiktionary's French section saying which lemmas take
one, and every pack studying French reads them as committed.
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--max-lemmas", type=int, default=60000)
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    from wordfreq import top_n_list, zipf_frequency

    zipf = {}

    def frequency(word):
        if word not in zipf:
            zipf[word] = zipf_frequency(word, "fr")
        return zipf[word]

    readings = Readings()
    lexicon = read_kaikki(os.path.join(a.work, "kaikki-French.jsonl"), readings)
    counts = read_gsd_counts(
        [os.path.join(a.work, "fr_gsd-ud-train.conllu"), os.path.join(a.work, "fr_gsd-ud-dev.conllu")]
    )
    name_or_word(lexicon.candidates, lexicon.names, frequency)
    own_words(lexicon)
    forms, ranks = reduce_forms(
        lexicon, counts, gsd_zipf(counts), frequency, lambda n: top_n_list("fr", n), a.max_lemmas
    )
    unlisted = unlisted_pronoun_tails(lexicon, forms)
    if unlisted:
        raise SystemExit(
            f"fr-en: {len(unlisted)} word(s) ending in a pronoun are no form, and the inversion rule would split "
            f"them: {', '.join(unlisted[:10])}"
        )

    common.write(a.work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())))
    common.write(
        a.work,
        "freq.tsv",
        "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: (kv[1], kv[0]))),
    )
    # The readings of the forms and ranks just chosen (add-lingua-french-grammar-tables D1).
    grammar = grammar_rows(readings, forms, ranks)
    common.write(a.work, "grammar.tsv", "".join(grammar))
    # French's estimated levels (add-lingua-french-levels): from the ranks and the section's senses,
    # never from the glosses.
    levels, left_out = estimated_levels(ranks, forms, read_level_senses(os.path.join(a.work, "kaikki-French.jsonl")))
    common.write(a.work, "level.tsv", "".join(f"{l}\t{lvl}\n" for l, lvl in sorted(levels.items())))
    # fr-en's native side (add-lingua-pack-fr-en), after the studied side, keyed by the lemmas just
    # ranked. Its glossed lemmas are French's dictionary words (`pack_sources.py split` writes
    # tables/fr/lexical.tsv from gloss.tsv).
    glosses, runs, expressions, native = native_side(a.work, os.path.join(a.work, "kaikki-French.jsonl"), ranks, forms)
    common.write(a.work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    common.write(
        a.work,
        "senses.tsv",
        "".join(
            f"{w}\t" + "\t".join(f"{pos}:{n}" for pos, n in r) + "\n" for w, r in sorted(runs.items()) if w in glosses
        ),
    )
    common.write(a.work, "mwe.tsv", "".join(f"{w}\t{g}\n" for w, g in sorted(expressions.items())))
    common.write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "fr",
            "native": "en",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
            # Derived from frequency, not taken from a CEFR list: the extension says so.
            "levels_estimated": True,
            "licences": [
                "kaikki / enwiktionary (CC BY-SA 4.0 + GFDL)",
                "wordfreq (CC BY-SA 4.0)",
                "UD French-GSD (CC BY-SA 4.0)",
            ],
        },
        "sources": [
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "UD French-GSD", "licence": "CcBySa"},
        ],
    }
    common.write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    print(f"reduced fr-en: forms={len(forms)} lemmas={len(ranks)} readings={len(grammar)}", file=sys.stderr)
    print(f"reduced fr-en: {levels_report(levels, left_out, ranks)}", file=sys.stderr)
    print(
        f"reduced fr-en: glosses={len(glosses)} (English Wiktionary {native['primary']}) "
        f"expressions={len(expressions)} ({native['split']} words the tokenisation splits) "
        f"dropped={native['dropped']} (section lines that are no JSON object)",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
