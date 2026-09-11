#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw EN->FR sources into the four pack tables (spec: add-lingua-data-pack).

Inputs (already downloaded into <work>/ by build.sh, git-ignored):
  - agid-infl.txt          AGID inflection database  (form <- lemma)
  - kaikki-Anglais.jsonl   kaikki frwiktionary "Anglais" extract (FR glosses of EN words)
  - wordfreq (pip)         English frequency ranks

Outputs (into <work>/, consumed by lingua-pack-build):
  - forms.tsv   form<TAB>lemma
  - freq.tsv    lemma<TAB>rank      (1 = most frequent)
  - gloss.tsv   lemma<TAB>gloss     (one short French gloss)
  - NOTICE      the attribution stack (the builder fails if a source is missing)
  - manifest.json  PackMeta + sources

Everything is SCOPED to the top-N most frequent English lemmas (wordfreq): that is
what keeps the pack under the 5 MB budget and covers the vocabulary that matters —
rarer words simply read as "unknown", which is honest. Output is deterministic
(sorted, fixed build date) so a rebuild from the same snapshots is byte-identical.
"""

import argparse
import json
import os
import re

# A studied-language token we keep: starts with a letter, then letters/apostrophe/hyphen.
_TOKEN = re.compile(r"[A-Za-z][A-Za-z'\-]*")


def top_lemmas(n):
    """The top-n English words as a lemma->dense-rank map (1-based)."""
    from wordfreq import top_n_list

    ranks = {}
    for w in top_n_list("en", n):
        w = w.strip().lower()
        if w and _TOKEN.fullmatch(w) and w not in ranks:
            ranks[w] = len(ranks) + 1
    return ranks


def agid_forms(inflections):
    """Every inflected form on an AGID line's right-hand side, cleaned.

    Groups are `|`-separated, alternatives `,`-separated; `{...}` are usage notes and
    `?`/`!`/`~` and bare numbers are annotations — all dropped.
    """
    cleaned = re.sub(r"\{[^}]*\}", " ", inflections)
    out = []
    for tok in re.split(r"[|,]", cleaned):
        tok = tok.strip().strip("?!~").strip()
        m = _TOKEN.fullmatch(tok)
        if m:
            out.append(tok.lower())
    return out


def reduce_forms(path, lemmas):
    """(form, lemma) pairs for every AGID entry whose lemma is in the top-N set."""
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
            if lemma not in lemmas:
                continue
            pairs.add((lemma, lemma))
            for form in agid_forms(right):
                pairs.add((form, lemma))
    return pairs


def clean_gloss(text, maxlen):
    g = re.sub(r"\s+", " ", text).strip().rstrip(".").strip()
    if len(g) > maxlen:
        g = g[:maxlen].rstrip()
    return g


def reduce_gloss(path, lemmas, maxlen, per_sense=42, max_senses=3):
    """Up to `max_senses` short French glosses per top-N lemma, joined by "; ".

    English words are polysemous and frwiktionary's first sense is not always the
    common one (e.g. `run`'s first sense is a rare noun, "Liquide"), so a single
    `senses[0]` is often misleading. Gathering the first gloss of the first few senses
    ACROSS the word's POS entries surfaces the everyday meaning (…"Courir"…) and gives
    the reader the range, which is what a vocabulary popup wants.
    """
    # word -> list of per-entry (per-POS) gloss lists, e.g. run -> [[noun senses], [verb senses]].
    entries = {}
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
                if gg:
                    g = clean_gloss(gg[0], per_sense)
                    if g:
                        senses.append(g)
            if senses:
                entries.setdefault(word, []).append(senses)

    glosses = {}
    for word, per_entry in entries.items():
        # Round-robin by sense depth ACROSS entries: take each POS's first sense before
        # any POS's second, so a verb meaning appears next to the noun rather than being
        # crowded out by one entry's many senses.
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

    ranks = top_lemmas(a.max_lemmas)
    lemmas = set(ranks)
    forms = reduce_forms(os.path.join(a.work, "agid-infl.txt"), lemmas)
    glosses = reduce_gloss(os.path.join(a.work, "kaikki-Anglais.jsonl"), lemmas, a.max_gloss_len)

    write(
        a.work,
        "forms.tsv",
        "".join(f"{form}\t{lemma}\n" for form, lemma in sorted(forms)),
    )
    write(
        a.work,
        "freq.tsv",
        "".join(f"{lemma}\t{rank}\n" for lemma, rank in sorted(ranks.items(), key=lambda kv: kv[1])),
    )
    write(
        a.work,
        "gloss.tsv",
        "".join(f"{lemma}\t{g}\n" for lemma, g in sorted(glosses.items())),
    )
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

    print(f"reduced en-fr: forms={len(forms)} freq={len(ranks)} gloss={len(glosses)}")


if __name__ == "__main__":
    main()
