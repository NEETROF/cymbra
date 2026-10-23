#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""Italian's detection guard, measured (add-lingua-italian-detection-guard, task 1.1; D2, D3, D6).

    python3 -I it_guard.py fetch   LIST DATA            # the pinned samples, checked by sha256
    python3 -I it_guard.py measure LIST DATA [REPO]     # offline: the figures and the refused blocks
    python3 -I it_guard.py words   LIST DATA [REPO]     # offline: the candidate words, one by one

LIST is it-guard-corpus.json; DATA a directory the samples are fetched into, one new directory per
source (nothing in it is ever executed: the samples are read as text); REPO the repository root
(default: three levels above this file). `measure` reads each block with whichlang 0.1.1 and with
lingua-core's own gate, `block_is_studied(block, Italian)` — the guard included — through the
`italian_guard_measure` example, and explains every refusal with a port of the guard's reading
over the tables it parses out of `language.rs`; the port must agree with the gate on every block
read as Italian, or the run fails. `words` measures D3's candidate words one by one against the
same blocks, with the port alone. Neither the samples nor the figures are committed.
"""

import bz2
import hashlib
import html
import json
import os
import re
import subprocess
import sys
import time
import unicodedata
import urllib.parse
import urllib.request

UA = {"User-Agent": "CymbraLinguaMeasure/1.0 (https://cymbra.app; detection-guard measurement)"}
MIN_BLOCK_BYTES = 12
SOFT_HYPHEN = "\u00ad"
APOSTROPHES = ("'", "\u2019")


# ---------------------------------------------------------------------------------------------
# Fetching


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def http_get(url):
    for attempt in range(4):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=120) as r:
                return r.read()
        except Exception:  # noqa: BLE001 — a transient failure: wait and retry
            if attempt == 3:
                raise
            time.sleep(5 * (attempt + 1))


def store(path, data, want, snapshot=False):
    """Writes `data` to `path` when its sha256 is `want`; refuses it otherwise, unless the file is a
    snapshot that moves upstream (Tatoeba's weekly export), which is kept with a warning."""
    got = sha256(data)
    if got != want:
        if not snapshot:
            sys.exit(f"error: {path}: sha256 {got}, the list pins {want}")
        print(f"warning: {path} is not the measured snapshot (sha256 {got}); sampled by id", file=sys.stderr)
    with open(path + ".part", "wb") as f:
        f.write(data)
    os.replace(path + ".part", path)
    return got


def checked(path, want, snapshot=False):
    if not os.path.isfile(path):
        return False
    with open(path, "rb") as f:
        got = sha256(f.read())
    if got != want:
        if not snapshot:
            sys.exit(f"error: {path}: sha256 {got}, the list pins {want}")
        print(f"warning: {path} is not the measured snapshot (sha256 {got}); sampled by id", file=sys.stderr)
    return True


def fetch_wiki(source, folder):
    missing = [a for a in source["articles"] if not checked(os.path.join(folder, f"{a['revid']}.wikitext"), a["sha256"])]
    for i in range(0, len(missing), 20):
        batch = missing[i : i + 20]
        query = urllib.parse.urlencode(
            {"action": "query", "revids": "|".join(str(a["revid"]) for a in batch), "prop": "revisions",
             "rvprop": "ids|sha1|content", "rvslots": "main", "format": "json", "formatversion": 2, "maxlag": 5}
        )
        time.sleep(1.0)
        data = json.loads(http_get(f"https://{source['wiki']}.wikipedia.org/w/api.php?{query}").decode("utf-8"))
        revisions = {r["revid"]: r for p in data["query"]["pages"] for r in p.get("revisions", [])}
        for article in batch:
            rev = revisions.get(article["revid"])
            if rev is None:
                sys.exit(f"error: {source['wiki']} revision {article['revid']} ({article['title']}) not returned")
            content = rev["slots"]["main"]["content"].encode("utf-8")
            if hashlib.sha1(content).hexdigest() != article["sha1"]:
                sys.exit(f"error: {source['wiki']} revision {article['revid']}: sha1 differs from the list's")
            store(os.path.join(folder, f"{article['revid']}.wikitext"), content, article["sha256"])


def fetch(list_path, data):
    corpus = json.load(open(list_path, encoding="utf-8"))
    for source in corpus["sources"]:
        if source["format"] == "pages":
            continue
        folder = os.path.join(data, source["id"])
        os.makedirs(folder, exist_ok=True)
        if source["format"] == "wikitext":
            fetch_wiki(source, folder)
        else:
            for file in source["files"]:
                path = os.path.join(folder, file["name"])
                snapshot = file.get("snapshot", False)
                if not checked(path, file["sha256"], snapshot):
                    time.sleep(1.0)
                    store(path, http_get(file["url"]), file["sha256"], snapshot)
        print(f"{source['id']}: ok", file=sys.stderr)


# ---------------------------------------------------------------------------------------------
# Blocks


def clean(block):
    """A block as the gate reads it: without soft hyphens, whitespace runs as one space, trimmed."""
    return re.sub(r"\s+", " ", block.replace(SOFT_HYPHEN, "")).strip()


def keep(blocks):
    return [b for b in (clean(b) for b in blocks) if len(b.encode("utf-8")) >= MIN_BLOCK_BYTES]


def conllu_blocks(path):
    with open(path, encoding="utf-8") as f:
        return [line[len("# text = "):] for line in f if line.startswith("# text = ")]


def tatoeba_blocks(path, ids_path):
    """The sentences the committed ids name, in their order: the sample drawn once (seed 42) from
    the export of 2026-10-17. A later export may have edited or deleted some; deleted ones are
    counted and left out."""
    with bz2.open(path, "rt", encoding="utf-8") as f:
        texts = {int(r[0]): r[2] for r in (line.rstrip("\n").split("\t") for line in f) if len(r) >= 3}
    with open(ids_path, encoding="utf-8") as f:
        ids = [int(line) for line in f if line.strip() and not line.startswith("#")]
    missing = [i for i in ids if i not in texts]
    if missing:
        print(f"warning: {len(missing)} of {len(ids)} sampled Tatoeba sentences are gone from this export",
              file=sys.stderr)
    return [texts[i] for i in ids if i in texts]


def gutenberg_blocks(path):
    with open(path, encoding="utf-8-sig") as f:
        text = f.read().replace("\r\n", "\n")
    start = re.search(r"^\*\*\* ?START OF.*$", text, flags=re.M)
    end = re.search(r"^\*\*\* ?END OF.*$", text, flags=re.M)
    body = text[start.end() : end.start()]
    return [" ".join(p.split("\n")).replace("_", "") for p in re.split(r"\n\s*\n", body)]


def pages_blocks(path, foreign):
    """pages-it.txt's Italian blocks: every line but the foreign ones the list names by page."""
    blocks, page, index = [], None, 0
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if line.startswith("=== "):
                page, index = line[4:].strip(), 0
                continue
            if not line.strip():
                continue
            if index not in foreign.get(page, []):
                blocks.append(line)
            index += 1
    return blocks


# Wikitext as a reader sees it, approximately: templates gone but the text of a few inline ones
# (a quotation, a language span), tables, references, files and categories gone, links read as
# their labels, bold and italics as MediaWiki reads runs of apostrophes. Deterministic over the
# pinned revision, which is what makes the sample a sample.
KEPT_TEMPLATES = {"lang": 2, "citazione": 1, "quote": 1, "cquote": 1, "quotation": 1, "blockquote": 1,
                  "nowrap": 1, "nobr": 1}
DROPPED_ELEMENTS = ["ref", "gallery", "math", "chem", "syntaxhighlight", "source", "pre", "timeline",
                    "graph", "mapframe", "templatedata", "score", "references", "imagemap", "hiero",
                    "categorytree", "inputbox"]


def split_params(inner):
    params, depth, start = [], 0, 0
    i = 0
    while i < len(inner):
        if inner.startswith("[[", i):
            depth += 1
            i += 2
            continue
        if inner.startswith("]]", i) and depth:
            depth -= 1
            i += 2
            continue
        if inner[i] == "|" and depth == 0:
            params.append(inner[start:i])
            start = i + 1
        i += 1
    params.append(inner[start:])
    return params


def render_template(inner):
    params = split_params(inner)
    name = params[0].strip().lower().replace("_", " ")
    positional = [p for p in params[1:] if not re.match(r"^\s*[\w -]+\s*=", p)]
    if name.startswith("lang-"):
        return positional[0] if positional else ""
    index = KEPT_TEMPLATES.get(name)
    if index and len(positional) >= index:
        return positional[index - 1]
    return ""


def strip_templates(text):
    out, stack, i = [], [], 0
    buf = []
    while i < len(text):
        if text.startswith("{{", i):
            stack.append(buf)
            buf = []
            i += 2
            continue
        if text.startswith("}}", i) and stack:
            rendered = render_template("".join(buf))
            buf = stack.pop()
            buf.append(rendered)
            i += 2
            continue
        buf.append(text[i])
        i += 1
    while stack:  # an unclosed template: drop it
        buf = stack.pop()
    out.extend(buf)
    return "".join(out)


def strip_tables(text):
    lines, depth = [], 0
    for line in text.split("\n"):
        s = line.lstrip(" :")
        if s.startswith("{|"):
            depth += 1
            continue
        if depth and s.startswith("|}"):
            depth -= 1
            continue
        if not depth:
            lines.append(line)
    return "\n".join(lines)


def apostrophes(line):
    def run(m):
        n = len(m.group(0))
        return {2: "", 3: "", 4: "'", 5: ""}.get(n, "'" * (n - 5))
    return re.sub(r"'{2,}", run, line)


def links(text, namespaces):
    names = {n.lower() for n in namespaces}

    def link(m):
        inner = m.group(1)
        target, _, label = inner.partition("|")
        shown = target.startswith(":")
        prefix = target.lstrip(":").split(":", 1)[0].strip().lower() if ":" in target else ""
        if prefix in names and not shown:
            return ""
        if prefix and not label and re.fullmatch(r"[a-z]{2,3}(-[a-z]+)*", prefix):
            return ""
        return label.split("|")[-1] if label else target.lstrip(":")

    previous = None
    while previous != text:
        previous = text
        text = re.sub(r"\[\[([^\[\]]*)\]\]", link, text)
    return text


def wikitext_blocks(path, namespaces):
    with open(path, encoding="utf-8") as f:
        text = f.read().replace("\r\n", "\n")
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    for tag in DROPPED_ELEMENTS:
        text = re.sub(rf"<{tag}\b[^>]*/>", "", text, flags=re.I)
        text = re.sub(rf"<{tag}\b[^>]*>.*?</{tag}\s*>", "", text, flags=re.I | re.S)
    text = strip_templates(text)
    text = strip_tables(text)
    text = links(text, namespaces)
    text = re.sub(r"\[(?:https?:)?//[^\s\]]*\s*([^\]]*)\]", r"\1", text)
    text = re.sub(r"https?://\S+", "", text)
    text = re.sub(r"<br\s*/?>", " ", text, flags=re.I)
    text = re.sub(r"</?[a-zA-Z][^>]*>", "", text)
    text = re.sub(r"__[A-Z]+__", "", text)
    blocks, paragraph = [], []
    for line in text.split("\n"):
        line = html.unescape(apostrophes(line))
        heading = re.match(r"^=+\s*(.*?)\s*=+\s*$", line)
        item = re.match(r"^[*#:;]+\s*(.*)$", line)
        if heading or item or not line.strip() or line.startswith(("|", "!", "{", "}", "----")):
            if paragraph:
                blocks.append(" ".join(paragraph))
                paragraph = []
            if heading:
                blocks.append(heading.group(1))
            elif item:
                blocks.append(item.group(1))
            continue
        paragraph.append(line.strip())
    if paragraph:
        blocks.append(" ".join(paragraph))
    return blocks


def source_blocks(source, data, repo):
    folder = os.path.join(data, source["id"])
    fmt = source["format"]
    if fmt == "wikitext":
        blocks = []
        for article in source["articles"]:
            blocks += wikitext_blocks(os.path.join(folder, f"{article['revid']}.wikitext"), source["namespaces"])
        return blocks
    if fmt == "conllu":
        return [b for f in source["files"] for b in conllu_blocks(os.path.join(folder, f["name"]))]
    if fmt == "tatoeba":
        ids = os.path.join(repo, "scripts/lingua-data/measure", source["ids"])
        return tatoeba_blocks(os.path.join(folder, source["files"][0]["name"]), ids)
    if fmt == "gutenberg":
        return gutenberg_blocks(os.path.join(folder, source["files"][0]["name"]))
    if fmt == "pages":
        return pages_blocks(os.path.join(repo, source["path"]), source["foreign"])
    sys.exit(f"error: unknown format {fmt}")


def all_blocks(corpus, data, repo):
    """[(source, block)], each block as the gate reads it, of 12 bytes or more."""
    out = []
    for source in corpus["sources"]:
        out += [(source, b) for b in keep(source_blocks(source, data, repo))]
    return out


# ---------------------------------------------------------------------------------------------
# The guard's reading, ported (language.rs: guard_words, guard_words_opened, italic_neighbour)


def in_run(c):
    """Rust's `c.is_alphabetic() || (!c.is_ascii() && is_combining_mark(c)) || c.is_numeric()`."""
    return unicodedata.category(c)[0] in "LMN"


def alphabetic(c):
    cat = unicodedata.category(c)
    return cat[0] == "L" or cat == "Nl"


def guard_words(text):
    """(start, end) of each word as language.rs's guard_words reads it: a run of letters, marks and
    digits joined across `-`, `.`, U+00B7 and U+2011 before a letter; a run holding a digit is no
    word."""
    words, start, digit = [], None, False
    n = len(text)
    i = 0
    while i < n:
        c = text[i]
        if in_run(c):
            digit |= unicodedata.category(c)[0] == "N"
            if start is None:
                start = i
            i += 1
            continue
        joins = c in "-.\u00b7\u2011" and start is not None and i + 1 < n and alphabetic(text[i + 1])
        if joins:
            i += 1
            continue
        if start is not None and not digit:
            words.append((start, i))
        start, digit = None, False
        i += 1
    if start is not None and not digit:
        words.append((start, n))
    return words


def opened(text, start, end):
    """Whether an apostrophe opens the word at `start`..`end` (guard_words_opened): a straight one or
    U+2019 just before it, after no letter, mark or digit, and no quote closing it just after it
    (« la lettera 'a' » quotes a letter)."""
    if start == 0 or text[start - 1] not in APOSTROPHES:
        return False
    if end < len(text) and text[end] in APOSTROPHES:
        return False
    return start == 1 or not (text[start - 2].isalnum() or unicodedata.category(text[start - 2])[0] == "M")


LONGEST_MARKER_BYTES = 9


def tokens(text):
    """The words of `text` the guard looks up: (lowercase NFC form, written in lowercase, opened by
    an apostrophe), a word longer than the longest marker skipped unless it is decomposed."""
    out = []
    for start, end in guard_words(text):
        word = text[start:end]
        decomposed = not word.isascii() and any(unicodedata.category(c)[0] == "M" for c in word)
        if len(word.encode("utf-8")) > LONGEST_MARKER_BYTES and not decomposed:
            continue
        lowercase = not any(c.isupper() for c in word)
        out.append((unicodedata.normalize("NFC", word).lower(), lowercase, opened(text, start, end)))
    return out


def found(toks, neighbour_table, italian_table):
    """The neighbour and the Italian markers counted among `toks`, as italic_neighbour counts them."""
    neighbour, italian = [], []
    for lower, lowercase, opens in toks:
        if lowercase and opens and ("'" + lower) in neighbour_table:
            neighbour.append("'" + lower)
        elif lowercase and lower in neighbour_table:
            neighbour.append(lower)
        elif lower in italian_table:
            italian.append(lower)
    return neighbour, italian


def markers(text, neighbour_table, italian_table):
    return found(tokens(text), neighbour_table, italian_table)


def refused(text, neighbour_table, italian_table):
    neighbour, italian = markers(text, neighbour_table, italian_table)
    return len(neighbour) > len(italian)


def rust_tables(repo):
    with open(os.path.join(repo, "crates/lingua-core/src/analysis/language.rs"), encoding="utf-8") as f:
        source = f.read()

    def table(name):
        m = re.search(rf"const {name}: &\[&str\] = &\[(.*?)\];", source, flags=re.S)
        if not m:
            sys.exit(f"error: {name} not found in language.rs")
        return set(re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(1)))

    return table("ITALIC_NEIGHBOUR_MARKERS"), table("ITALIAN_MARKERS")


# ---------------------------------------------------------------------------------------------
# Reading the blocks with lingua-core


def read_with_core(blocks, repo, work):
    """[(whichlang's class, read as Italian by the gate)] for each block, by the Rust example."""
    path = os.path.join(work, "blocks.txt")
    with open(path, "w", encoding="utf-8") as f:
        for block in blocks:
            f.write(block.replace("\t", " ") + "\n")
    out = subprocess.run(
        ["cargo", "run", "--quiet", "--release", "-p", "lingua-core", "--example", "italian_guard_measure", "--", path],
        cwd=repo, check=True, capture_output=True, text=True,
    ).stdout.splitlines()
    if len(out) != len(blocks):
        sys.exit(f"error: {len(out)} answers for {len(blocks)} blocks")
    return [(line.split("\t")[0], line.split("\t")[1] == "1") for line in out]


def pct(part, whole):
    return f"{100 * part / whole:.1f} %" if whole else "—"


def measure(list_path, data, repo):
    corpus = json.load(open(list_path, encoding="utf-8"))
    blocks = all_blocks(corpus, data, repo)
    work = os.path.join(data, "_measure")
    os.makedirs(work, exist_ok=True)
    answers = read_with_core([b for _, b in blocks], repo, work)
    neighbour_table, italian_table = rust_tables(repo)
    disagreements = 0
    rows = {}
    refusals = []
    for (source, block), (cls, studied) in zip(blocks, answers):
        size = len(block.encode("utf-8"))
        row = rows.setdefault(source["id"], [source, 0, 0, 0, 0, 0, 0])
        row[1] += 1
        row[2] += size
        if cls == "ita":
            row[3] += 1
            row[4] += size
            if refused(block, neighbour_table, italian_table) == studied:
                disagreements += 1
                print(f"DISAGREE {source['id']}: {block}", file=sys.stderr)
            if studied:
                row[5] += 1
                row[6] += size
            elif source["kind"] == "italian":
                refusals.append((source["id"], block, *markers(block, neighbour_table, italian_table)))
    shares = neighbour_shares(corpus, blocks, answers)
    kept = set(corpus["candidates"].get("named_by_spec", []))
    kept |= {language for language, (n, _, ita, _) in shares.items() if ita * 10 >= n}
    print("## Neighbours: share of blocks (of bytes) read as Italian\n")
    print("| Neighbour | Wiki | Blocks | Without the guard | With the guard | D2's default | Kept |")
    print("|---|---|---:|---:|---:|---|---|")
    pooled = [0, 0, 0, 0, 0, 0]
    for row in rows.values():
        source = row[0]
        if source["kind"] != "neighbour":
            continue
        print(f"| {source['language']} | {source['wiki']} | {row[1]:,} | {pct(row[3], row[1])} ({pct(row[4], row[2])}) "
              f"| {pct(row[5], row[1])} ({pct(row[6], row[2])}) | {source['default']} "
              f"| {'yes' if source['language'] in kept else 'no'} |")
        if source["language"] in kept:
            for i in range(6):
                pooled[i] += row[i + 1]
    print(f"| Pooled, the kept | | {pooled[0]:,} | {pct(pooled[2], pooled[0])} ({pct(pooled[3], pooled[1])}) "
          f"| {pct(pooled[4], pooled[0])} ({pct(pooled[5], pooled[1])}) | | |")
    print("\n## Italian corpora: blocks refused among those read as Italian\n")
    print("| Corpus | Blocks | Read as Italian | Refused |")
    print("|---|---:|---:|---:|")
    total = [0, 0, 0]
    for row in rows.values():
        source = row[0]
        if source["kind"] != "italian":
            continue
        refused_here = row[3] - row[5]
        print(f"| {source['label']} | {row[1]:,} | {row[3]:,} ({pct(row[3], row[1])}) | {refused_here} |")
        total[0] += row[1]
        total[1] += row[3]
        total[2] += refused_here
    print(f"| All | {total[0]:,} | {total[1]:,} | {total[2]} ({100 * total[2] / max(total[1], 1):.3f} %) |")
    print("\n## Italian blocks refused, with their markers\n")
    for source_id, block, neighbour, italian in refusals:
        print(f"- [{source_id}] neighbour {neighbour}, Italian {italian}: {block}")
    print(f"\nThe port and the gate agree on every block read as Italian: {disagreements == 0} "
          f"({disagreements} disagreements)")
    pages_refused = sum(1 for s, *_ in refusals if s == "pages-it")
    gate = total[2] * 1000 <= total[1] and pages_refused == 0
    print(f"Recall gate (<= 1 in 1,000 refused, none of pages-it.txt): {'passes' if gate else 'FAILS'}")
    return 0 if gate and disagreements == 0 else 1


# ---------------------------------------------------------------------------------------------
# The candidate words, one by one (D3)


def neighbour_shares(corpus, blocks, answers):
    """Per neighbour: (blocks, bytes, blocks read as Italian, their bytes)."""
    shares = {}
    for (source, block), (cls, _) in zip(blocks, answers):
        if source["kind"] != "neighbour":
            continue
        row = shares.setdefault(source["language"], [0, 0, 0, 0])
        size = len(block.encode("utf-8"))
        row[0] += 1
        row[1] += size
        if cls == "ita":
            row[2] += 1
            row[3] += size
    return shares


def italian_rates(token_lists, table):
    words_read, counts = 0, {}
    for toks in token_lists:
        words_read += len(toks)
        for w in found(toks, set(), table)[1]:
            counts[w] = counts.get(w, 0) + 1
    return {w: 1000 * counts.get(w, 0) / max(words_read, 1) for w in table}


def words(list_path, data, repo):
    """D3's candidates, measured one by one (the design's Measurement, rules 1 to 5)."""
    corpus = json.load(open(list_path, encoding="utf-8"))
    candidates = corpus["candidates"]
    blocks = all_blocks(corpus, data, repo)
    work = os.path.join(data, "_measure")
    os.makedirs(work, exist_ok=True)
    answers = read_with_core([b for _, b in blocks], repo, work)
    # Rule 1: the neighbours whichlang reads as Italian on a tenth of their blocks or more; and
    # those the spec names.
    shares = neighbour_shares(corpus, blocks, answers)
    print("## Rule 1: share of each neighbour's Wikipedia blocks read as Italian, without the guard\n")
    kept = set(candidates.get("named_by_spec", []))
    for language, (n, size, ita, ita_size) in shares.items():
        by_rule = ita * 10 >= n
        if by_rule:
            kept.add(language)
        print(f"- {language}: {ita:,} of {n:,} blocks ({pct(ita, n)}; {pct(ita_size, size)} of the text)"
              f" -> {'kept' if by_rule else 'below a tenth'}")
    print(f"\nKept (rule 1, and the spec's {candidates.get('named_by_spec', [])}): {sorted(kept)}")
    read = [(s, b, tokens(b)) for (s, b), (cls, _) in zip(blocks, answers) if cls == "ita"]
    excluded = set(candidates["excluded"])
    own = {lang: set(ws) - excluded for lang, ws in candidates["neighbour"].items()}
    neighbour_table = set().union(*(own[lang] for lang in kept))
    italian_table = set(candidates["italian"]) - set(candidates["italian_excluded"])
    italian_toks = [t for s, _, t in read if s["kind"] == "italian"]
    by_neighbour = {}
    for s, _, t in read:
        if s["kind"] == "neighbour" and s["language"] in kept:
            by_neighbour.setdefault(s["language"], []).append(t)
    # Rule 4: Italian markers, as pre-written (every block read as Italian) and as amended (the
    # blocks the neighbour wrote: those holding one of its own candidate markers).
    it_rates = italian_rates(italian_toks, italian_table)
    out_written, out_amended = {}, {}
    print("\n## Rule 4: Italian candidates, per 1,000 words (Italian corpora; each neighbour, all | own)\n")
    for w in sorted(italian_table):
        cells = []
        for language in sorted(by_neighbour):
            all_toks = by_neighbour[language]
            own_toks = [t for t in all_toks if found(t, own[language], set())[0]]
            rate_all = italian_rates(all_toks, {w})[w]
            rate_own = italian_rates(own_toks, {w})[w]
            if rate_all > it_rates[w] / 10:
                out_written.setdefault(w, []).append(language)
            if rate_own > it_rates[w] / 10:
                out_amended.setdefault(w, []).append(language)
            cells.append(f"{language[:3]} {rate_all:.2f}|{rate_own:.2f}")
        print(f"- `{w}` {it_rates[w]:.2f}; " + ", ".join(cells))
    print(f"\nOut as pre-written: {dict(sorted(out_written.items()))}")
    print(f"Out as amended (the neighbour's own blocks): {dict(sorted(out_amended.items()))}")
    quoted = candidates.get("italian_read_as_quotation", {})
    out_read = {w: [n for n in langs if n not in quoted.get(w, {})] for w, langs in out_amended.items()}
    out_read = {w: langs for w, langs in out_read.items() if langs}
    print(f"Out once read (the neighbour writes it): {dict(sorted(out_read.items()))}")
    italian_final = italian_table - set(out_read)
    # Rule 3(a): a neighbour word counting in no kept neighbour's block read as Italian.
    neighbour_toks = [t for ts in by_neighbour.values() for t in ts]
    caught = {}
    for t in neighbour_toks:
        for w in set(found(t, neighbour_table, set())[0]):
            caught[w] = caught.get(w, 0) + 1
    named = set().union(*(own[lang] for lang in candidates.get("named_by_spec", [])))
    uncaught = sorted(w for w in neighbour_table if not caught.get(w) and w not in named)
    print(f"\n## Rule 3(a): out, catching no neighbour block read as Italian ({len(uncaught)}): {uncaught}")
    print(f"Kept by the spec though they catch none: {sorted(w for w in named if not caught.get(w))}")
    # Rule 3(b): the words a refused Italian block, read by hand as genuine Italian, drops.
    drops = candidates.get("misfire_drops", {})
    review = candidates.get("review_drops", {})
    table = neighbour_table - set(uncaught) - set(drops) - set(review)
    print(f"Rule 3(b), dropped after reading the refused blocks: {sorted(drops)}")
    print(f"Left out after the review: {sorted(review)}")

    def refusals(nt, it):
        return [(s, b, found(t, nt, it)) for s, b, t in read if s["kind"] == "italian"
                and len(found(t, nt, it)[0]) > len(found(t, nt, it)[1])]

    # Rule 5: the recall gate; while it fails, drop the word counting in the most refused blocks
    # (tie: the one catching fewer neighbour blocks, then alphabetical).
    italian_read = len(italian_toks)
    while True:
        now = refusals(table, italian_final)
        pages = sum(1 for s, _, _ in now if s["id"] == "pages-it")
        if len(now) * 1000 <= italian_read and pages == 0:
            break
        tally = {}
        for _, _, (n, _) in now:
            for w in set(n):
                tally[w] = tally.get(w, 0) + 1
        worst = min(tally, key=lambda w: (-tally[w], caught.get(w, 0), w))
        print(f"Rule 5: {len(now)} refused of {italian_read:,} ({pages} of pages-it.txt); drop `{worst}`, "
              f"in {tally[worst]} refused blocks, catching {caught.get(worst, 0)} neighbour blocks")
        table = table - {worst}

    def leaks(nt, it):
        return sum(1 for t in neighbour_toks if len(found(t, nt, it)[0]) <= len(found(t, nt, it)[1]))

    refused_now = refusals(table, italian_final)
    print(f"\n## Italian blocks refused: {len(refused_now)} of {italian_read:,} read as Italian\n")
    for s, b, (n, i) in refused_now:
        print(f"- [{s['id']}] neighbour {n}, Italian {i}: {b}")
    base_leak = leaks(table, italian_final)
    print(f"\n## Each word out alone (kept neighbours' blocks still read as Italian: {base_leak:,} of "
          f"{len(neighbour_toks):,}; Italian refused: {len(refused_now)})\n")
    for w in sorted(table):
        print(f"- `{w}` caught {caught.get(w, 0)}; without it {leaks(table - {w}, italian_final) - base_leak:+d} neighbour, "
              f"{len(refusals(table - {w}, italian_final)) - len(refused_now):+d} Italian")
    for w in sorted(italian_final):
        print(f"- Italian `{w}`; without it {leaks(table, italian_final - {w}) - base_leak:+d} neighbour, "
              f"{len(refusals(table, italian_final - {w})) - len(refused_now):+d} Italian")
    print(f"\nTABLES neighbour ({len(table)})={json.dumps(sorted(table), ensure_ascii=False)}")
    print(f"TABLES italian ({len(italian_final)})={json.dumps(sorted(italian_final), ensure_ascii=False)}")
    return 0


def main():
    args = sys.argv[1:]
    if len(args) < 3 or args[0] not in ("fetch", "measure", "words"):
        sys.exit(__doc__)
    command, list_path, data = args[:3]
    repo = args[3] if len(args) > 3 else os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../.."))
    if command == "fetch":
        fetch(list_path, data)
        return 0
    return measure(list_path, data, repo) if command == "measure" else words(list_path, data, repo)


if __name__ == "__main__":
    sys.exit(main())
