#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw EN->FR sources (AGID + wordfreq + kaikki) into the pack tables.

Inputs (downloaded into <work>/ by build.sh, git-ignored):
  - agid-infl.txt          AGID inflection database  (lemma <POS>: forms)
  - kaikki-Anglais.jsonl   kaikki frwiktionary "Anglais" extract (FR glosses of EN words)
  - wordfreq (pip)         English frequency ranks

Outputs (into <work>/, consumed by lingua-pack-build):
  forms.tsv  (form<TAB>lemma) / freq.tsv (lemma<TAB>rank) / gloss.tsv (lemma<TAB>gloss)
  / NOTICE / manifest.json

Key rule: only CANONICAL LEMMAS (base forms) are ever treated as lemmas. An inflected
form (e.g. "targets", "gives") is kept in forms.tsv so it lemmatises to its base, but is
NEVER given a rank or a gloss of its own — otherwise it becomes a spurious pool lemma
that (a) carries a useless "Pluriel de …" form-of gloss and (b) collides with its base
lemma's entry. Pack is scoped to the top-N canonical lemmas to fit the 5 MB budget.
Output is sorted + date-stamped so a rebuild from the same snapshots is byte-identical.
"""

import argparse
import json
import os
import re

_TOKEN = re.compile(r"[A-Za-z][A-Za-z'\-]*")

# French frwiktionary "form-of" gloss templates — these mark an entry that is an
# inflected form, not a word with a meaning of its own; never a useful translation.
_FORM_OF = re.compile(
    r"^(pluriel|singulier|f[ée]minin|masculin|participe|pr[ée]t[ée]rit|"
    r"(troisi[èe]me|deuxi[èe]me|premi[èe]re) personne|variante|autre graphie|"
    r"forme (de|du|d'|verbale|fl[ée]chie)|genre|orthographe)\b",
    re.IGNORECASE,
)


def agid_forms(inflections):
    """Cleaned inflected forms from an AGID right-hand side."""
    cleaned = re.sub(r"\{[^}]*\}", " ", inflections)
    out = []
    for tok in re.split(r"[|,]", cleaned):
        tok = tok.strip().strip("?!~").strip()
        if _TOKEN.fullmatch(tok):
            out.append(tok.lower())
    return out


def parse_agid(path):
    """All (form, lemma) pairs from AGID, skipping questionable-POS lines.

    A `?` on the part of speech (e.g. `gif N?: gives`) marks a doubtful headword; such
    lines carry bogus inflections (AGID really does claim the noun "gif" pluralises to
    "gives"), so they are dropped.
    """
    pairs = set()
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if ":" not in line:
                continue
            left, right = line.split(":", 1)
            head = left.split()
            if not head:
                continue
            lemma = head[0].lower()
            if not _TOKEN.fullmatch(lemma):
                continue
            if any("?" in flag for flag in head[1:]):  # questionable POS -> skip
                continue
            pairs.add((lemma, lemma))
            for form in agid_forms(right):
                pairs.add((form, lemma))
    return pairs


def canonical_ranks(inflected, want):
    """Dense ranks over the top canonical lemmas (wordfreq order, inflected forms skipped)."""
    from wordfreq import top_n_list

    ranks = {}
    for w in top_n_list("en", want * 4):
        w = w.strip().lower()
        if w in ranks or not _TOKEN.fullmatch(w) or w in inflected:
            continue
        ranks[w] = len(ranks) + 1
        if len(ranks) >= want:
            break
    return ranks


def clean_gloss(text, maxlen):
    g = re.sub(r"\s+", " ", text).strip().rstrip(".").strip()
    if len(g) > maxlen:
        g = g[:maxlen].rstrip()
    return g


def reduce_gloss(path, lemmas, maxlen, per_sense=42, max_senses=3):
    """Up to `max_senses` short French glosses per canonical lemma, joined by "; ".

    Glosses are gathered one-per-sense, round-robin ACROSS a word's POS entries, so a
    verb meaning appears beside the noun (run -> "Course; Courir") rather than being
    crowded out. Form-of senses ("Pluriel de …") are skipped — they are not meanings.
    """
    entries = {}  # word -> [per-entry [gloss,...]]
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            word = (d.get("word") or "").strip().lower()
            if not word or word not in lemmas:
                continue
            senses = []
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                if gg and not _FORM_OF.match(gg[0].strip()):
                    g = clean_gloss(gg[0], per_sense)
                    if g:
                        senses.append(g)
            if senses:
                entries.setdefault(word, []).append(senses)

    glosses = {}
    for word, per_entry in entries.items():
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
        if joined:
            glosses[word] = joined
    return glosses


NOTICE = """\
Cymbra Lingua data pack — EN->FR attributions.

AGID (Automatically Generated Inflection Database), from the SCOWL / aspell family
(en-wl/wordlist): permission to use, copy, modify, distribute and sell, with the
upstream notices retained (WordNet, 2of12id, ENABLE).

wordfreq (English frequency list): CC BY-SA 4.0 (includes SUBTLEX with Brysbaert's
permission).

kaikki.org extract of the French Wiktionary (frwiktionary): CC BY-SA 4.0 + GFDL.
"""


def write(work, name, text):
    with open(os.path.join(work, name), "w", encoding="utf-8") as f:
        f.write(text)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--max-lemmas", type=int, default=40000)
    ap.add_argument("--max-gloss-len", type=int, default=80)
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    pairs = parse_agid(os.path.join(a.work, "agid-infl.txt"))
    inflected = {form for form, lemma in pairs if form != lemma}
    ranks = canonical_ranks(inflected, a.max_lemmas)
    lemmas = set(ranks)

    # forms.tsv: every AGID mapping whose lemma we keep, plus each lemma's self-map so a
    # lemma with no listed inflection is still recognised.
    forms = {(f, l) for f, l in pairs if l in lemmas}
    forms |= {(l, l) for l in lemmas}
    glosses = reduce_gloss(os.path.join(a.work, "kaikki-Anglais.jsonl"), lemmas, a.max_gloss_len)

    write(a.work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(forms)))
    write(
        a.work,
        "freq.tsv",
        "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: kv[1])),
    )
    write(a.work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "en",
            "native": "fr",
            "pack_version": a.pack_version,
            "analyzer_version": "1.0.0",
            "licences": [
                "AGID (permissive, commercial use allowed)",
                "wordfreq (CC BY-SA 4.0)",
                "kaikki / frwiktionary (CC BY-SA 4.0 + GFDL)",
            ],
        },
        "sources": [
            {"name": "AGID", "licence": "Permissive"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "kaikki", "licence": "CcBySa"},
        ],
    }
    write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")

    print(f"reduced en-fr: forms={len(forms)} lemmas={len(ranks)} gloss={len(glosses)}")


if __name__ == "__main__":
    main()
