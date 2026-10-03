# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The reduction rules every <studied>->FR pair shares (generalise-lingua-pack-reducer).

A pair's reducer (`reduce-<pair>.py`) reads its own inflection and level sources; what it does
with the French Wiktionary — cleaning a gloss, grouping its senses by part of speech, the
expressions — and what it does with forms once it has them — one lemma per form, dense ranks,
the words a level list adds — is the same for every studied language whose glosses are in
French, and lives here.

What IS the studied language's is passed in as a `Studied`: which strings are words of it, how a
French form-of gloss names its target, which conjunctions coordinate, and wordfreq's code for it.
The rules themselves do not change with the language, and moving them here changed no English
table (the en-fr tables reduce byte for byte as before).

This module is part of every pair's reduction rules: its sha256 enters the rule set pinned in
tables/<pair>/pin.json (pack_sources.py `rules`), so editing it asks for every pair to be
reduced again.
"""

import bisect
import itertools
import json
import os
import re
from dataclasses import dataclass


@dataclass(frozen=True)
class Studied:
    """What the shared rules need to know about the studied language."""

    code: str  # wordfreq's language code: "en"
    token: re.Pattern  # a word of the language, whole: `token.fullmatch(w)`
    form_of_target: re.Pattern  # the word a French form-of gloss points at ("Pluriel de datum.")
    coordinators: frozenset  # the conjunctions kaikki's `conj` means as CCONJ, not SCONJ


# French frwiktionary "form-of" gloss templates — these mark an entry that is an
# inflected form, not a word with a meaning of its own; never a useful translation.
# kaikki also tags most such senses `form-of` (see `_is_form_of`); the regex catches the
# untagged ones.
_FORM_OF = re.compile(
    r"^(pluriel|singulier|f[ée]minin|masculin|participe|pr[ée]t[ée]rit|pass[ée]|imparfait|"
    r"comparatif|superlatif|g[ée]rondif|(troisi[èe]me|deuxi[èe]me|premi[èe]re) personne|"
    r"variante|autre graphie|forme (de|du|d'|verbale|fl[ée]chie)|genre|orthographe)\b",
    re.IGNORECASE,
)


# Four more pointer wordings, applied in the MULTI-WORD path ONLY: "Présent progressif.",
# "Graphie alternative de douchebag." name a tense or a spelling, not a meaning. They are
# not in `_FORM_OF` because that regex also feeds forms.tsv, freq.tsv and gloss.tsv, which
# this pair's single-word tables must keep producing byte for byte; the eight senses it
# would cost there are not worth the risk.
_MWE_FORM_OF = re.compile(r"^(pr[ée]sent|futur|conjugaison|graphie)\b", re.IGNORECASE)


def _is_form_of(sense, gloss):
    """Whether a kaikki sense only points at another word ("Pluriel de …", "Passé de …")."""
    return "form-of" in (sense.get("tags") or ()) or bool(sense.get("form_of")) or bool(_FORM_OF.match(gloss))


def wiktionary_signals(path, words, *, studied):
    """For the given words: the parts of speech they have a meaning under, and their form-of targets.

    A sense is a meaning when its gloss says what the word means ("Beurre."), not which
    word it is a form of ("Comparatif de numb."); the targets are the words such pointers
    name (kaikki's `form_of` field, or the word the gloss ends on).
    """
    meanings, targets = {}, {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            word = (d.get("word") or "").strip().lower()
            if word not in words:
                continue
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                gloss = gg[0].strip() if gg else ""
                if not gloss:
                    continue
                if _is_form_of(sense, gloss):
                    found = {t.lower() for t in studied.form_of_target.findall(gloss)}
                    found |= {
                        (ref.get("word") or "").strip().lower()
                        for ref in sense.get("form_of") or ()
                        if isinstance(ref, dict)
                    }
                    found.discard("")
                    targets.setdefault(word, set()).update(found)
                else:
                    meanings.setdefault(word, set()).add(d.get("pos") or "")
    return meanings, targets


def canonical_ranks(inflected, want, *, studied):
    """Dense ranks over the top canonical lemmas (wordfreq order, inflected forms skipped)."""
    from wordfreq import top_n_list

    ranks = {}
    for w in top_n_list(studied.code, want * 4):
        w = w.strip().lower()
        if w in ranks or not studied.token.fullmatch(w) or w in inflected:
            continue
        ranks[w] = len(ranks) + 1
        if len(ranks) >= want:
            break
    return ranks


def orphaned_forms(inflected, pairs, ranks):
    """Inflected forms with no kept lemma behind them: read as words of their own.

    A form is left out of the lemmas because its base stands for it; when no base is kept — nor
    any base of a base — nothing does, and the word drops out of the pack. ESDB knows rare bases
    AGID did not ("grandkid", "policymaker", "uprise", "gree", "crowdfund"), and wordfreq ranks
    their forms far above them: without this, "grandkids", "policymakers", "uprising", "greed" and
    "crowdfunding" would vanish. A form whose base is itself a form of a kept lemma ("buildings",
    of "building", of "build") is not orphaned: it reads as that chain always did.
    """
    bases = {}
    for form, lemma in pairs:
        if form != lemma:
            bases.setdefault(form, set()).add(lemma)
    kept = ranks.keys()

    def behind(form):
        direct = bases.get(form, set())
        return direct | set().union(*(bases.get(base, set()) for base in direct))

    return {form for form in inflected if not behind(form) & kept}


def append_level_extras(ranks, cefr, inflected, frequency, lemma_of, *, studied):
    """Ranks extended with the CEFR headwords the pack would otherwise not hold.

    A CEFR word is added when it has no rank and does not already resolve to a kept lemma
    through `lemma_of` — an inflection whose base was dropped ("boring" from "bore") would
    otherwise vanish. So a declared level and the ladder see (nearly) the whole list.

    A hyphenated compound whose parts are all ranked takes its rarest part's rank
    ("well-known" ranks as "known"): its rank-based presumption matches the analyser's
    weakest-part judgement of it when unlisted, although statuses set on its parts no
    longer reach it. Any other word ranks with the last ranked lemma at least as frequent
    (`frequency`: wordfreq's Zipf value in the real build), or after the whole list, the
    commonest first, when it is rarer than all of them.
    """
    order = sorted(ranks, key=ranks.get)
    # -Zipf in rank order, forced non-decreasing so it can be bisected.
    scale = list(itertools.accumulate((-frequency(lemma) for lemma in order), max))
    out = dict(ranks)
    rarest = []
    for w in sorted(cefr):
        if w in ranks or not studied.token.fullmatch(w) or (w in inflected and lemma_of(w) is not None):
            continue
        if "-" in w:
            parts = [ranks.get(lemma_of(part)) for part in w.split("-")]
            if all(parts):
                out[w] = max(parts)
                continue
        else:
            at = bisect.bisect_right(scale, -frequency(w))
            if at < len(order):
                out[w] = ranks[order[max(at, 1) - 1]]
                continue
        rarest.append(w)
    last = max(ranks.values(), default=0)
    for i, w in enumerate(sorted(rarest, key=lambda w: (-frequency(w), w)), start=1):
        out[w] = last + i
    return out


def compound_inflections(lemmas, pairs, readings=None):
    """(form, compound) pairs inflecting each hyphenated lemma: "t-shirts", "mothers-in-law".

    AGID has no hyphenated headwords and the analyser looks a compound up whole, so without
    these a listed compound's plural would still be judged part by part. The first and the
    last part each take their AGID forms; the odd nonsense combination is never read. A compound
    form is what its inflected part is: `readings`, when given, gets the part's tags.
    """
    forms_of = {}
    for form, lemma in pairs:
        if form != lemma:
            forms_of.setdefault(lemma, set()).add(form)
    out = set()
    for lemma in lemmas:
        parts = lemma.split("-")
        if len(parts) < 2:
            continue
        for i in {0, len(parts) - 1}:
            for form in forms_of.get(parts[i], ()):
                whole = "-".join(parts[:i] + [form] + parts[i + 1 :])
                out.add((whole, lemma))
                if readings is not None and (form, parts[i]) in readings:
                    readings.setdefault((whole, lemma), set()).update(readings[(form, parts[i])])
    return out


def resolve_forms(pairs, ranks, targets=None, glossed=frozenset()):
    """The single lemma of every form whose lemma is kept.

    A kept lemma always maps to itself: the pack builder finds a lemma's id through this
    table, so a lemma read as another word ("bored" as "bore") would hand its rank and level
    to that word. That is also what keeps a word of its own away from the base AGID wrongly
    gave it (one too rare to keep still resolves as before). Any other form maps to one of
    its kept bases: the one its Wiktionary form-of gloss names ("uses" is "use", not "us"),
    then one with a gloss, then the most frequent, then alphabetically.
    """
    targets = targets or {}
    candidates = {}
    for form, lemma in pairs:
        if lemma in ranks and form not in ranks:
            candidates.setdefault(form, set()).add(lemma)
    for lemma in ranks:
        candidates[lemma] = {lemma}
    return {
        form: min(
            lemmas,
            key=lambda lemma: (lemma not in targets.get(form, ()), lemma not in glossed, ranks[lemma], lemma),
        )
        for form, lemmas in candidates.items()
    }


# A sense left hanging on a coordinator: the Wiktionary line read "(Vieilli) ou Pluie" and
# the parenthetical went, so the gloss opens on "ou". LOWERCASE only — `etcetera` is glossed
# "Et cetera", where the coordinator IS the translation, and a capital is what tells them
# apart across the 15 entries the corpus holds.
_DANGLING_COORDINATOR = re.compile(r"^(?:ou|et)\s+")


# What the Wiktionary writes for its own readers, not a translation: a pointer to another page
# ("Y avoir. → voir there be", "(→ voir bone marrow)", "(→ Comparer avec -ative)") — a link on the
# wiki, dead text on a card — and the placeholders of an unfinished page ("Définition manquante ou
# à compléter. (Ajouter)", an invitation to contributors), wherever they sit in the sense.
_WIKI_NOTES = re.compile(
    r"\s*\(→[^)]*\)"
    r"|\s*→\s*(?:voir|comparer)\b[^;]*"
    r"|\s*\(?Définition manquante ou à co.*?(?:\(Ajouter\)\)?|$)[.…]*"
    r"|\s*Étymologie manquante ou incomplète.*?(?:cliquant ici\.|$)",
    re.IGNORECASE,
)


def strip_wiki_notes(text):
    """`text` without the Wiktionary's pointers and placeholders (`_WIKI_NOTES`)."""
    return _WIKI_NOTES.sub("", text)


def clean_gloss(text, maxlen, whole_words=False):
    """A sense, tidied and held within `maxlen` characters.

    The expressions keep the plain cut. A word's senses (`whole_words`) are cut at a word
    boundary and end with an ellipsis (`cut_at_word`), since the word card shows them in full.
    A sense that is nothing but a pointer or a placeholder comes out empty, and is left out.
    """
    g = re.sub(r"\s+", " ", strip_wiki_notes(text)).strip(" ;,").rstrip(".:").strip()
    g = _DANGLING_COORDINATOR.sub("", g)
    if len(g) > maxlen:
        g = cut_at_word(g, maxlen) if whole_words else g[:maxlen].rstrip()
    return g


# What a cut sense may not end on before its ellipsis: a separator or an opening mark.
_CUT_TRAIL = " ,;:(«[\"'’-–—/"


def cut_at_word(text, maxlen):
    """`text` within `maxlen` characters, the ellipsis included, never ending mid-word.

    The cut falls on the last space the room allows; a text with no space in the second half of
    that room — one long word, a URL — is cut where the room ends. Separators and opening marks
    left dangling before the ellipsis go ("former le passif (…" → "former le passif…").
    """
    if len(text) <= maxlen:
        return text
    room = maxlen - 1  # the ellipsis
    space = text.rfind(" ", 0, room + 1)
    cut = text[:space] if space > room // 2 else text[:room]
    return cut.rstrip(_CUT_TRAIL) + "…"


def _join_senses(per_entry, maxlen, max_senses):
    """One gloss out of a headword's per-entry sense lists, cut to `maxlen`.

    Senses are taken one-per-entry, round-robin ACROSS the headword's POS entries, so a
    verb meaning appears beside the noun (run -> "Course; Courir") rather than being
    crowded out by the first entry's three.
    """
    picked = []
    depth = 0
    deepest = max(len(e) for e in per_entry)
    while depth < deepest and len(picked) < max_senses:
        for e in per_entry:
            if depth < len(e) and e[depth] not in picked:
                picked.append(e[depth])
                if len(picked) >= max_senses:
                    break
        depth += 1
    joined = "; ".join(picked)
    if len(joined) > maxlen:
        joined = joined[:maxlen].rstrip().rstrip(";").strip()
    return joined


# kaikki's `pos` in Universal Dependencies parts of speech (design D4). A conjunction is
# coordinating when it is one of the seven coordinators, subordinating otherwise; anything that is
# no part of speech of a word (an affix, a phrase, a typographic variant) is `X`.
_KAIKKI_UPOS = {
    "noun": "NOUN",
    "verb": "VERB",
    "adj": "ADJ",
    "adv": "ADV",
    "name": "PROPN",
    "pron": "PRON",
    "prep": "ADP",
    "postp": "ADP",
    "det": "DET",
    "article": "DET",
    "particle": "PART",
    "intj": "INTJ",
    "onomatopoeia": "INTJ",
    "num": "NUM",
    "character": "SYM",
    "symbol": "SYM",
}


# A `;` inside a sense, with the spaces French typography puts around it ("Indigène ; qui…").
_INNER_SEPARATOR = re.compile(r"\s*;\s*")


def kaikki_upos(pos, word, *, studied):
    """The UD part of speech of a kaikki entry of `word`."""
    if pos == "conj":
        return "CCONJ" if word in studied.coordinators else "SCONJ"
    return _KAIKKI_UPOS.get(pos, "X")


# The fewest characters a sense cut to fit a word's gloss may keep: below, it says nothing.
_MIN_CUT_SENSE = 20


def _join_senses_by_pos(per_entry, poses, maxlen, max_senses):
    """`_join_senses`, with the senses grouped by part of speech (add-lingua-word-grammar, D4).

    The same senses are picked, by the same round-robin across the headword's entries; they are
    then grouped by the part of speech of the entry each came from, stably, in the order the parts
    of speech first appear, so that the senses of one part of speech are adjacent. A `;` inside a
    sense becomes `,`, so that "; " only ever separates senses. The gloss keeps whole senses while
    they fit in `maxlen`; the next is cut at a word boundary with an ellipsis when at least
    `_MIN_CUT_SENSE` characters of room are left, and left out otherwise — never cut mid-word.
    The runs — [(part of speech, senses)] — count the senses the gloss holds.
    """
    picked = []  # (sense, part of speech)
    depth = 0
    deepest = max(len(e) for e in per_entry)
    while depth < deepest and len(picked) < max_senses:
        for senses, pos in zip(per_entry, poses):
            if depth < len(senses) and senses[depth] not in [sense for sense, _ in picked]:
                picked.append((senses[depth], pos))
                if len(picked) >= max_senses:
                    break
        depth += 1
    order = list(dict.fromkeys(pos for _, pos in picked))
    grouped = [
        (_INNER_SEPARATOR.sub(", ", sense).strip(" ,"), pos) for first in order for sense, pos in picked if pos == first
    ]
    # Whole senses while they fit; the next one cut at a word boundary when enough room is left
    # for it to say something, left out otherwise. The gloss never ends mid-word.
    kept = []
    length = 0
    for sense, _ in grouped:
        gap = 2 if kept else 0
        if length + gap + len(sense) <= maxlen:
            kept.append(sense)
            length += gap + len(sense)
            continue
        room = maxlen - length - gap
        if room >= _MIN_CUT_SENSE:
            kept.append(cut_at_word(sense, room))
        break
    joined = "; ".join(kept)
    runs = []
    for _, pos in grouped[: len(kept)]:
        if runs and runs[-1][0] == pos:
            runs[-1][1] += 1
        else:
            runs.append([pos, 1])
    return joined, [tuple(run) for run in runs]


def _acronym(headword):
    """Whether a headword is written all in capitals, as an acronym is ("AND", "WHO", "US")."""
    return len(headword) > 1 and headword.isupper()


def _read_entries(path, words, per_sense, pointers=None, *, studied):
    """The glossed kaikki entries of `words`: word -> [(an acronym's entry, part of speech, [gloss])].

    With `pointers`, also the words a word's form-of senses point at, with the part of speech of
    the entry that points: word -> [(base, part of speech)], in source order.
    """
    entries = {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            headword = (d.get("word") or "").strip()
            word = headword.lower()
            if not word or word not in words:
                continue
            pos = kaikki_upos(d.get("pos") or "", word, studied=studied)
            senses = []
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                if not gg:
                    continue
                if not _is_form_of(sense, gg[0].strip()):
                    g = clean_gloss(gg[0], per_sense, whole_words=True)
                    if g:
                        senses.append(g)
                elif pointers is not None:
                    for target in sense.get("form_of") or []:
                        base = (target.get("word") or "").strip().lower()
                        if len(base) >= _MIN_BASE and base != word and (base, pos) not in pointers.get(word, []):
                            pointers.setdefault(word, []).append((base, pos))
            if senses:
                entries.setdefault(word, []).append((_acronym(headword), pos, senses))
    return entries


# The shortest base a form-of sense may lend its gloss from: "fs" is the plural of the letter "f",
# and a letter's gloss says nothing about the word.
_MIN_BASE = 3


def reduce_gloss(path, lemmas, maxlen, per_sense=80, max_senses=3, runs=None, *, studied):
    """Up to `max_senses` short French glosses per canonical lemma, joined by "; ".

    Form-of senses ("Pluriel de …") are skipped — they are not meanings. An acronym's
    entries do not gloss the common word spelled like it in lower case when that word has an
    entry of its own: "AND", the logic operator, gave "and" a noun "ET" and a verb "Faire le
    ET de"; "WHO" gave "who" « OMS », "FOR" gave "for" « Franco wagon ». An acronym with no
    such word keeps its gloss: "NATO" still glosses "nato". The senses that make the cut are
    picked by `_join_senses_by_pos`, which groups them by part of speech; `runs`, when given,
    gets each gloss's runs: lemma -> [(part of speech, senses)].

    A lemma whose only senses are form-of senses borrows the gloss of the base they name, in the
    same part of speech: the word list keeps "catacombs" as a word of its own, and the Wiktionary
    only says it is the plural of "catacomb", so its card had no translation; it now reads
    « Catacombe ». The first base, in source order, that has senses in that part of speech lends
    them; a base in another part of speech lends nothing ("hearted" is no form of the noun "heart").
    """
    pointers = {}
    entries = _read_entries(path, lemmas, per_sense, pointers, studied=studied)
    glosses = {}

    def gloss(word, found):
        kept = [entry for entry in found if not entry[0]] or found
        per_entry = [senses for _, _, senses in kept]
        joined, word_runs = _join_senses_by_pos(per_entry, [pos for _, pos, _ in kept], maxlen, max_senses)
        if joined:
            glosses[word] = joined
            if runs is not None:
                runs[word] = word_runs

    for word, found in entries.items():
        gloss(word, found)
    borrowing = {word: bases for word, bases in pointers.items() if word not in glosses}
    lenders = _read_entries(
        path, {base for bases in borrowing.values() for base, _ in bases}, per_sense, studied=studied
    )
    for word, bases in borrowing.items():
        for base, pos in bases:
            lent = [entry for entry in lenders.get(base, []) if entry[1] == pos and not entry[0]]
            if lent:
                gloss(word, lent)
                break
    return glosses


def reduce_expressions(path, maxlen, per_sense=42, max_senses=3, *, studied):
    """Up to `max_senses` short French glosses per MULTI-WORD headword ("give up").

    `reduce_gloss`'s reduction over the entries it can never reach: it is scoped to the
    kept lemmas, a lemma is one word, so the 33 404 multi-word entries of the source were
    dropped whole. Left out here: an entry whose part of speech is `name` (a proper noun,
    which the card refuses to gloss anyway), a headword outside the studied language's word pattern,
    and an entry no sense survives.

    NOT left out: an expression holding a word the pack's lexicon does not hold. That test
    needs the lexicon, and only the builder has one.
    """
    entries = {}  # headword -> [per-entry [gloss, ...]]
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            word = re.sub(r"\s+", " ", (d.get("word") or "").strip().lower())
            if " " not in word or not all(studied.token.fullmatch(w) for w in word.split(" ")):
                continue
            if (d.get("pos") or "") == "name":
                continue
            senses = []
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                gloss = gg[0].strip() if gg else ""
                if not gloss or _is_form_of(sense, gloss) or _MWE_FORM_OF.match(gloss):
                    continue
                g = clean_gloss(gloss, per_sense)
                if g:
                    senses.append(g)
            if senses:
                entries.setdefault(word, []).append(senses)

    expressions = {}
    for word, per_entry in entries.items():
        joined = _join_senses(per_entry, maxlen, max_senses)
        if joined:
            expressions[word] = joined
    return expressions


# CEFR level order (A1 lowest). A lemma listed at several levels/POS takes the LOWEST
# (earliest-taught) level — the collapse rule from the design.
_LEVEL_RANK = {"A1": 1, "A2": 2, "B1": 3, "B2": 4, "C1": 5, "C2": 6}


def reduce_levels(cefr, lemmas):
    """Lowest CEFR level per kept lemma. Only lemmas we actually keep in the pack are emitted."""
    return {
        w: min((level for _, level in entries), key=_LEVEL_RANK.__getitem__)
        for w, entries in cefr.items()
        if w in lemmas
    }


def write(work, name, text):
    with open(os.path.join(work, name), "w", encoding="utf-8") as f:
        f.write(text)
