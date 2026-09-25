## Context

The `en→fr` pack is built in two stages (`scripts/lingua-data/build.sh en-fr <out>`):

1. **Reduce** (Python, `reduce-en-fr.py`): the raw sources — kaikki frwiktionary "Anglais" JSONL
   (~200 MB), AGID `infl.txt`, CEFR-J 1.5 and Octanove C1/C2 CSVs, and the `wordfreq` package —
   become a handful of sorted tables: `forms.tsv`, `freq.tsv`, `gloss.tsv`, `level.tsv`, `mwe.tsv`,
   `NOTICE`, `manifest.json`. About 3 MB; about **1 MB** gzipped (measured on a 2026-09-21
   snapshot).
2. **Build** (Rust, `crates/lingua-pack`, `lingua-pack-build`): the tables become `pack.lingua`
   (~1.5 MB). The builder is already deterministic and tested so
   (`spec_rebuilding_is_byte_for_byte_identical`; zstd level 19, stable for a given zstd version).

Stage 1 runs at every release, against live upstream, in two workflows (`lingua-extension-release`,
`lingua-apple-release`), with no version and no hash on any source — see the proposal for what that
costs. Stage 2 is already reproducible; stage 1 is where the dictionary drifts and where a vanished
source blocks a release.

`add-lingua-translation-delivery` solved the same problem for the translation engine: a pin file in
the repository (`engine-pin.json`, sha256 per file), the bytes in a GitHub Release that never expires,
a fetch script that refuses anything else, and a rebuild that only proves the recipe. This design
applies that pattern to the dictionary.

## Goals / Non-Goals

**Goals:**
- No release reads kaikki, GitHub raw or PyPI; a release builds the pinned pack, byte for byte, on
  every lane.
- Changing the dictionary is a pull request that shows what changed.
- A package's `pack_version` says which dictionary it carries.
- Mozilla's reviewer rebuilds the submitted pack offline and gets the same bytes.
- A pin whose snapshot cannot be fetched fails a pull request, not a release.

**Non-Goals:**
- Changing the dictionary's content, the reducer's rules, the pack format or the builder.
- Pinning the testdata pack (`yarn gen:pack`), already committed.
- Other language pairs (the pin is keyed by pair, so a second pair is a second entry, later).
- `apps/lingua-agent`, which builds from test data.

## Decisions

### D1 — Pin the reduced tables, not the raw sources and not only the built pack

Three things could be pinned:

| | Size | Release needs | Reviewer rebuilds | Republishing rights |
|---|---|---|---|---|
| Raw sources | ~205 MB | Python, pinned wordfreq, the reducer | from 205 MB, with Python | **per source** (CEFR-J's terms unclear) |
| **Reduced tables** | **~1 MB** gz | the Rust builder (already there) | offline, from the archive | same as the pack, which already ships them |
| Built pack only | ~1.5 MB | nothing | cannot: data with no source | same as the pack |

The tables are the pack's human-readable source: every row the pack holds, sorted, diffable, and
already redistributed inside every package under the pack's notices. Pinning them leaves the
already-deterministic stage 2 in every release and takes the drifting stage 1 out of it.

*Rejected — raw sources as the pin:* 200 MB per release and per review, a Python stack in every
release lane, and a republication question for each source that the tables do not raise.
*Rejected — the built pack alone:* a reviewer could not rebuild it, and a diff of two binary packs
says nothing about what changed in the dictionary.

### D2 — The pin file

`scripts/lingua-data/pack-pin.json`, one entry per pair:

```json
{
  "en-fr": {
    "snapshot": "2026.09.25",
    "tables": { "release": "lingua-pack-en-fr-2026.09.25", "asset": "tables.tgz", "sha256": "…" },
    "pack": { "sha256": "…", "size": 1543992 },
    "reducer": { "sha256": "…" },
    "sources": [
      { "name": "kaikki-Anglais.jsonl", "url": "https://kaikki.org/…", "fetched": "2026-09-25",
        "last_modified": "…", "sha256": "…", "size": 201084572 }
    ]
  }
}
```

- `tables` is what releases use; `pack` is what they must obtain from it.
- `reducer` records the hash of `reduce-en-fr.py` the tables came from. The check lane fails when
  the reducer changes without a new snapshot: a reducer edit would otherwise reach no release
  silently — or, worse, the next refresh would mix a rule change with an upstream change in one
  unreviewable diff.
- `sources` is provenance only — which bytes of which upstream produced the snapshot. The raw files
  are neither committed (unchanged rule) nor needed by any release.

### D3 — `build.sh` has a pinned mode and a live mode

- **Pinned** (`build.sh en-fr <out>`, what `yarn gen:pack:real` runs): download the tables asset by
  its release and name, check its sha256, unpack, run the builder, check the pack's sha256. No
  network beyond github.com; no Python.
- **Live** (`build.sh --live en-fr <out> <work>`): today's behaviour — fetch every source, reduce,
  build — plus recording each source's sha256, size and Last-Modified. Only the refresh workflow and
  a developer experimenting run it.

`yarn gen:pack` (testdata) is unchanged.

### D4 — The refresh workflow proposes, a pull request decides

`lingua-pack-refresh`, manual: runs the live mode with `wordfreq==3.1.1` installed from a
hash-pinned requirements file, then compares the new tables with the pinned ones and writes a report
to the run summary — lemmas, glosses, levels and expressions added, removed and changed (counts and
a sample of each), pack size against the 5 MB budget. It publishes the new tables as a new release
(`lingua-pack-en-fr-<date>`, never replacing one) and opens a pull request that bumps
`pack-pin.json`, the report in its body. Merging it is the decision; not merging it leaves every
release as it was.

The workflow is also how a reducer change reaches readers: edit the reducer, run the refresh, review
the diff it causes.

### D5 — `pack_version` is the snapshot

The reducer receives `--pack-version <snapshot>` (`2026.09.25`) instead of the constant `1.0.0`, so
the version a reader's pack reports names its dictionary. The core already reads `pack_version` as
an opaque string (it gates only on `analyzer_version`), and its own tests already use a date-shaped
value (`2026.09.1`).

### D6 — Every lane that ships a pack uses the pin

- `lingua-extension-release` and `lingua-apple-release`: `yarn gen:pack:real` (pinned mode). The
  raw-source `actions/cache` and the `pip install wordfreq` step are removed; the "at least 1 MB"
  guard is replaced by the pack's sha256.
- `lingua-extension-check` (every pull request touching the extension or the pipeline): checks that
  the pinned asset exists and matches, and that `reduce-en-fr.py` matches `reducer.sha256`. It does
  not build the real pack (it keeps using testdata, as today).
- The AMO source archive (`make_source_archive.sh`) carries the pinned `tables.tgz`; `REVIEWERS.md`
  rebuilds the pack from it with the builder alone and states the expected sha256.

### D7 — Where the snapshot lives

A GitHub Release per snapshot, created by the refresh workflow, never replaced — the engine's
pattern. The repository is public; the tables are what every package already distributes, under the
notices the pack embeds (**Licence hygiene** unchanged). The first snapshot is taken from the
current pipeline, so the first pinned pack is the dictionary readers would have received anyway.

## Risks / Trade-offs

- **[The dictionary stops improving on its own]** — Wiktionary fixes no longer arrive by
  themselves. → That is the point; the refresh is one dispatch, and its report says what a refresh
  would change before anyone accepts it.
- **[A refresh diff too large to read]** — the first refresh after months could change thousands of
  glosses. → The report counts and samples; the pack is still built and budget-checked; a refresh
  can be declined and retried later.
- **[wordfreq pinned but unmaintained]** — 3.1.1 may stop installing on a future Python. → Only the
  refresh needs it; releases no longer do. The refresh workflow pins its Python version too.
- **[Release deletion]** — someone deletes a snapshot release. → The check lane fails on the next
  pull request, naming the missing release; the refresh workflow can re-create it from the pinned
  sources only if they still match their recorded hashes — otherwise a new snapshot is the fix.
- **[zstd version drift in the builder]** — the pack's sha256 depends on it. → `Cargo.lock` pins
  it; a lock update that changes the pack fails the release's sha256 check, and the fix is a new pin
  of `pack.sha256`, reviewed.

## Migration Plan

1. Land the pinned mode, the refresh workflow and the checks, with the pin empty and releases still
   on the live mode.
2. Dispatch the refresh once: it creates the first snapshot release and the pull request that
   fills `pack-pin.json`. Merge it.
3. Switch both release lanes to the pinned mode (same pull request as step 2, or the next).

Rollback: point the release lanes back at the live mode; the pin file and releases are inert.

## Open Questions

- Should the refresh open the pull request itself (needs "Allow GitHub Actions to create pull
  requests" in the repository settings), or only publish the release and print the pin to paste?
- Should a raw-source snapshot also be kept, for sources whose licence allows it, so an old snapshot
  can be re-reduced after a reducer change without its upstream? (Not needed by any release.)
- A refresh cadence, if any (quarterly?), or only on demand.
