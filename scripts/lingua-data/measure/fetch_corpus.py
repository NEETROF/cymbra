#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""Fetch the text of the measurement corpus (corpus.json lists titles only).

    fetch_corpus.py corpus.json out.json

Wikipedia and Wikinews articles come as plain text from their APIs (as they are today: an article
edited since `fetched` reads differently); MDN pages as Markdown at the commit the manifest names,
with front matter, code, macros and link targets stripped. The result — [{title, kind, text}] — is
what compare_packs.mjs reads. It is never committed.
"""

import json
import re
import sys
import time
import urllib.parse
import urllib.request

UA = {"User-Agent": "CymbraLinguaMeasure/1.0 (https://cymbra.app; pack inflection measurement)"}
API = {"wikipedia": "https://en.wikipedia.org/w/api.php", "wikinews": "https://en.wikinews.org/w/api.php"}


def get(url):
    for attempt in range(4):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60) as r:
                return r.read().decode("utf-8")
        except Exception:  # noqa: BLE001 — a transient failure: wait and retry
            if attempt == 3:
                raise
            time.sleep(2 * (attempt + 1))


def wiki(source, title):
    query = urllib.parse.urlencode(
        {"action": "query", "prop": "extracts", "explaintext": 1, "redirects": 1, "titles": title, "format": "json"}
    )
    pages = json.loads(get(f"{API[source]}?{query}"))["query"]["pages"]
    return next(iter(pages.values())).get("extract", "")


def mdn(path, commit):
    text = get(f"https://raw.githubusercontent.com/mdn/content/{commit}/files/en-us/{path}/index.md")
    text = re.sub(r"\A---\n.*?\n---\n", "", text, flags=re.S)  # front matter
    text = re.sub(r"```.*?```", " ", text, flags=re.S)  # code blocks
    text = re.sub(r"\{\{.*?\}\}", " ", text)  # macros
    text = re.sub(r"`[^`]*`", " ", text)  # inline code
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)  # links: their words, not their targets
    text = re.sub(r"<[^>]+>", " ", text)  # inline HTML
    return re.sub(r"^[#>*\-|]+\s*", "", text, flags=re.M)


def main():
    manifest, out = sys.argv[1], sys.argv[2]
    docs = []
    for d in json.load(open(manifest, encoding="utf-8"))["documents"]:
        text = mdn(d["title"], d["commit"]) if d["source"] == "mdn" else wiki(d["source"], d["title"])
        docs.append({"title": d["title"], "kind": d["kind"], "text": text})
        time.sleep(0.2)
    json.dump(docs, open(out, "w", encoding="utf-8"), ensure_ascii=False)
    print(f"{len(docs)} documents, {sum(len(d['text']) for d in docs):,} characters -> {out}")


if __name__ == "__main__":
    main()
