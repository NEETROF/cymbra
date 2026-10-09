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
  a form's own entry points at what it is a form of (`form_of`).
- `fr_gsd-ud-train.conllu`, `fr_gsd-ud-dev.conllu`: UD French-GSD, read for how often each form
  stands for each lemma (design D5) and how often each hyphenated lemma occurs (D6). Its test
  section is never read: the measurement holds it out (D9).
- wordfreq `fr` (the installed, pinned package): the 60,000 commonest lemmas and which forms are
  attested at all (D6).

Outputs, in `--work`: `forms.tsv`, `freq.tsv`, an empty `gloss.tsv` (fr-en's glosses come with
add-lingua-pack-fr-en), `NOTICE` and `manifest.json`. The readings (`grammar.tsv`) and the levels
(`level.tsv`) are later changes' of the same reducer.

The tables serve French's tokenisation as add-lingua-french-tokenisation writes it (M21, design
D4): the pre-pass hands the lookup the word an elided piece stands for (`l'` is read `le`), splits
`au` and `aux` into `à` + `le`/`les` and an inversion into its words (`dit-il` → `dit` + `il`),
keeps `du` and `des` whole, and keeps whole a hyphenated run the pack lists.
"""

import argparse
import collections
import json
import math
import os
import re
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares

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


def read_kaikki(path):
    """The French section, read entry by entry into a `Lexicon`."""
    lexicon = Lexicon()
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            lexicon.read(entry)
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


NOTICE = """Cymbra Lingua data pack — FR->EN attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), French section: CC BY-SA 4.0 + GFDL —
the forms and their lemmas.

wordfreq (French frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 — the commonest lemmas and which forms are attested.

UD French-GSD, the Universal Dependencies French-GSD treebank
(https://github.com/UniversalDependencies/UD_French-GSD): CC BY-SA 4.0 — how often a form stands
for each of its lemmas, to choose one, and how often a hyphenated word occurs, to rank it.
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

    lexicon = read_kaikki(os.path.join(a.work, "kaikki-French.jsonl"))
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
    # fr-en glosses nothing yet: the file every pair's folder holds, empty, so French's dictionary
    # words (tables/fr/lexical.tsv, its glossed lemmas) are none.
    common.write(a.work, "gloss.tsv", "")
    common.write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "fr",
            "native": "en",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
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
    print(f"reduced fr-en: forms={len(forms)} lemmas={len(ranks)}", file=sys.stderr)


if __name__ == "__main__":
    main()
