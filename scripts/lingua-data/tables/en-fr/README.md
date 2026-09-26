# The en→fr dictionary tables

The reduced tables Cymbra Lingua's English→French data pack is built from, committed so that every
release builds the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it (pin-lingua-pack-sources).

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | AGID (permissive), with Wiktionary signals |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0) |
| `gloss.tsv` | lemma → French gloss | kaikki.org extract of the French Wiktionary (CC BY-SA 4.0 + GFDL) |
| `level.tsv` | lemma → CEFR level | CEFR-J Wordlist v1.5 (commercial use with citation) + Octanove Vocabulary Profile C1/C2 v1.0 (CC BY-SA 4.0) |
| `mwe.tsv` | expression → French gloss | kaikki.org extract of the French Wiktionary (CC BY-SA 4.0 + GFDL) |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata; `pack_version` is the snapshot | — |
| `pin.json` | the raw sources these tables came from, and the pack they build | — |

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences: the Wiktionary-derived tables (`gloss.tsv`, `mwe.tsv`, and the Wiktionary
signals in `forms.tsv`) under CC BY-SA 4.0 and the GFDL, `freq.tsv` under CC BY-SA 4.0, `level.tsv`
under CEFR-J's terms (commercial use allowed with citation) and CC BY-SA 4.0, AGID's relations under
its permissive licence. `NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `mode=update`. It reads today's
  sources, keeps kaikki's bytes as the release `lingua-pack-sources-en-fr-<snapshot>`, reduces,
  pushes the branch `lingua-pack/<snapshot>`, and writes a report of what changes. Open the pull
  request from the link in its summary; releases keep these tables until it is merged.
- **After editing `reduce-en-fr.py`**: the check lane fails until the tables are reduced again
  from the pinned sources — `scripts/lingua-data/build.sh --reduce en-fr <out>` (Python 3.12,
  `requirements-reduce.txt`), or `lingua-pack-update` with `mode=reduce`. The diff is then the
  rules' effect alone.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

A monthly dry run of the update reports how far upstream has drifted, and fails when a source moved
or a table collapsed.
