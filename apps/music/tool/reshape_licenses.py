#!/usr/bin/env python3
"""Reshape `cargo about generate --format json` output into the compact asset
the app bundles and parses at startup (change: add-oss-license-attributions).

`cargo about`'s raw JSON repeats full cargo metadata (absolute manifest
paths, full dependency graphs, ...) per crate, which is both far larger than
needed and leaks build-machine file paths. This keeps only what the license
page renders: for each distinct license text, the SPDX id/name and the list
of "name version" packages that use it.
"""

import json
import sys


def reshape(raw):
    licenses = []
    for entry in raw["licenses"]:
        packages = sorted(
            f"{used['crate']['name']} {used['crate']['version']}"
            for used in entry["used_by"]
        )
        licenses.append(
            {
                "id": entry["id"],
                "name": entry["name"],
                "text": entry["text"],
                "packages": packages,
            }
        )
    licenses.sort(key=lambda lic: lic["id"])
    return {"licenses": licenses}


def main():
    raw_path, out_path = sys.argv[1], sys.argv[2]
    with open(raw_path, encoding="utf-8") as f:
        raw = json.load(f)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(reshape(raw), f, ensure_ascii=False, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
