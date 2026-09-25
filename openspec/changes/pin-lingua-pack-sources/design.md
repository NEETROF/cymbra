## Context

The `en→fr` pack is built in two stages (`scripts/lingua-data/build.sh en-fr <out>`):

1. **Reduce** (Python, `reduce-en-fr.py`): the raw sources — kaikki frwiktionary "Anglais" JSONL
   (~200 MB), AGID `infl.txt`, CEFR-J 1.5 and Octanove C1/C2 CSVs, and the `wordfreq` package —
   become a handful of sorted tables: `forms.tsv`, `freq.tsv`, `gloss.tsv`, `level.tsv`, `mwe.tsv`,
   `NOTICE`, `manifest.json`. About 3.4 MB of text.
2. **Build** (Rust, `crates/lingua-pack`, `lingua-pack-build`): the tables become `pack.lingua`
   (~1.5 MB). The builder is already deterministic and tested so
   (`spec_rebuilding_is_byte_for_byte_identical`; zstd level 19, stable for a given zstd version).

Stage 1 runs at every release, against live upstream, in two workflows (`lingua-extension-release`,
`lingua-apple-release`), with no version and no hash on any source — see the proposal for what that
costs. Stage 2 is already reproducible. Stage 1 is where the dictionary drifts and where a vanished
source blocks a release.

Decided with the product owner on 2026-09-25: the reduced tables are **committed**; the update
workflow pushes a branch and never opens a pull request; only kaikki gets a raw snapshot of ours;
drift is detected monthly and adopted on demand.

## Goals / Non-Goals

**Goals:**
- No release reads kaikki, GitHub raw or PyPI; every lane builds the same pack, byte for byte.
- Changing the dictionary is a pull request whose diff shows what changed.
- A change to the reduction rules is reviewed apart from upstream drift.
- A broken upstream is known before anyone needs an update.
- Pull requests build the real pack; the reviewer rebuilds it offline.
- A package's `pack_version` says which dictionary it carries.

**Non-Goals:**
- Changing the dictionary's content, the reducer's rules, the pack format or the builder.
- The testdata pack (`yarn gen:pack`), already committed.
- Other language pairs (the layout is keyed by pair; a second pair is a second folder, later).
- `apps/lingua-agent`, which builds from test data.

## Decisions

### D1 — The reduced tables are committed

They go to `scripts/lingua-data/tables/en-fr/`, exactly the files stage 2 reads, plus a `README.md`
stating their licences (the repository is Apache-2.0; these files are CC BY-SA 4.0 / GFDL / AGID /
CEFR-J with citation, as the pack's NOTICE already says).

- The pull request diff **is** the review: which glosses, levels or expressions change, line by line.
- Git is the pin: no asset to fetch, no hash to check for the tables themselves.
- The AMO source archive already includes `scripts/lingua-data`, so it carries them unchanged.
- The check lane can build the real pack on every pull request (D5).

*Cost:* ~3.4 MB of text (roughly 1 MB after git's compression) in a 229 MB repository, plus each
update's delta. *Rejected — the tables as a release asset pinned by sha256:* the same guarantees,
but a review needs a separate diff report, and every lane needs a fetch step.
*Rejected — the raw sources as the pin:* 200 MB per release, Python in every release lane, and a
republication question per source. *Rejected — only the built pack:* no one can rebuild it or read
its diff.

### D2 — The record beside the tables

`scripts/lingua-data/tables/en-fr/pin.json`:

```json
{
  "snapshot": "2026.09.25",
  "pack": { "sha256": "…", "size": 1543992 },
  "reducer": { "sha256": "…" },
  "sources": {
    "kaikki": { "release": "lingua-pack-sources-en-fr-2026.09.25", "asset": "kaikki-Anglais.jsonl.zst",
                "sha256": "…", "size": 201084572, "fetched": "2026-09-25", "last_modified": "…" },
    "agid": { "url": "https://raw.githubusercontent.com/en-wl/wordlist/464bea8cca4f606d9e271600b7718818fdd6507c/agid/infl.txt", "sha256": "…" },
    "cefrj": { "url": "https://raw.githubusercontent.com/openlanguageprofiles/olp-en-cefrj/d4e45b75b38f27b30dfc5c44d8c571aec7e7092f/cefrj-vocabulary-profile-1.5.csv", "sha256": "…" },
    "octanove": { "url": "…/d4e45b75b38f27b30dfc5c44d8c571aec7e7092f/octanove-vocabulary-profile-c1c2-1.0.csv", "sha256": "…" },
    "wordfreq": { "version": "3.1.1" }
  }
}
```

- `pack` is what every lane must obtain from the tables — it catches builder drift (a zstd update
  in `Cargo.lock`) as a failed check rather than a silent change.
- `reducer` is the hash of the `reduce-en-fr.py` the tables came from; `sources` are the raw bytes
  they came from. Together they make the tables reproducible from pinned inputs (D3).

### D3 — Raw sources: pinned where they live, kaikki kept by us

- **AGID, CEFR-J, Octanove** are fetched at a **commit** of their own repository, never a branch,
  and checked by sha256. Measured: the current AGID URL names `master`, a branch the repository no
  longer has (its default is `v2`; the file exists only on `v1`, last changed in 2009) and works
  through a leftover redirect. At a commit, a URL means the same bytes for as long as the repository
  exists. They are not republished, which leaves CEFR-J's terms — commercial use with citation,
  nothing said of redistribution — out of the question.
- **wordfreq** is installed as `wordfreq==3.1.1` from a hash-pinned requirements file, with its
  Python version fixed. The project is frozen and PyPI keeps its releases: the version is the
  snapshot.
- **kaikki** is the one source regenerated daily, so its bytes on the snapshot day exist nowhere
  else afterwards. The update workflow keeps them as a GitHub Release asset, zstd-compressed
  (197 MB → **13.6 MB** measured at level 19), one release per snapshot, never replaced. Its
  licence (CC BY-SA 4.0 / GFDL) allows republication with attribution; the release notes carry it.

### D4 — `build.sh` has three modes

- **Build** (default; `yarn gen:pack:real`): `lingua-pack-build` on `tables/en-fr/`, then check the
  pack's sha256 against `pin.json`. Offline, no Python.
- **Re-reduce** (`--reduce`): fetch the pinned raw sources (D3), check their sha256, run the
  reducer, write `tables/en-fr/`, update `pack` and `reducer` in `pin.json`. Used when the reduction
  rules change: the diff is the rule change alone.
- **Update** (`--update`): fetch the live sources, record them in `pin.json` (new kaikki snapshot,
  newer commits if asked), reduce, write the tables. Used to take in upstream changes.

`yarn gen:pack` (testdata) is unchanged.

### D5 — What each lane runs

- **Release lanes** (`lingua-extension-release`, `lingua-apple-release`): Build mode. The
  raw-source `actions/cache`, `pip install wordfreq` and the "at least 1 MB" guard are removed; the
  pack's sha256 replaces the guard.
- **Check lane** (`lingua-extension-check`, every pull request touching the extension or the
  pipeline): Build mode, so a pull request that breaks the pack, its budget or its sha256 fails
  before release day. It also fails when `reduce-en-fr.py` no longer matches `reducer.sha256`,
  asking for a re-reduce. It keeps building the testdata pack for the extension's own tests.
- **The reviewer's archive**: `REVIEWERS.md` rebuilds with Build mode, offline, and states the
  expected sha256. The archive's rebuild in the check lane does exactly that and compares.

### D6 — The update workflow pushes a branch; the maintainer opens the pull request

`lingua-pack-update`, manual, with a `mode` input (`update` or `reduce`): runs the chosen mode in a
pinned environment, publishes the kaikki snapshot release when there is a new one, commits the new
tables and `pin.json` to a branch `lingua-pack/<snapshot>`, and writes to the run summary a report —
lemmas, glosses, levels and expressions added, removed and changed (counts and samples) and the pack
size against its budget — with the link that opens the pull request.

It does not open the pull request itself. The repository setting that would allow it is one
checkbox for creating **and approving** pull requests, which in a public repository lets any
workflow approve; and a pull request opened with the workflow's token triggers no workflow, so its
required checks would never run. A maintainer opening it from the link has neither problem.

### D7 — Drift is detected monthly, adopted on demand

The same workflow runs on a monthly schedule in **dry** mode: Update mode into a scratch folder, the
report against the committed tables in the run summary, nothing committed or published. It fails —
visibly — when a source cannot be fetched, when the reducer fails, or when the result loses more
than a set share of a table (a format change upstream shows as a collapse, not an error). An update
itself is decided by a person, when the report or a reader's bug calls for it.

### D8 — `pack_version` is the snapshot

The reducer receives `--pack-version <snapshot>` (`2026.09.25`) instead of the constant `1.0.0`.
The core reads `pack_version` as an opaque string — it gates only on `analyzer_version` — and its own
tests already use a date-shaped value (`2026.09.1`).

## Risks / Trade-offs

- **[The dictionary stops improving on its own]** → That is the point; the monthly report says what
  an update would bring, and an update is one dispatch plus a review.
- **[An update diff too large to read]** → The report counts and samples; the diff is still there
  line by line; an update can be declined and retried later.
- **[A pinned repository disappears]** (AGID, CEFR-J) → The committed tables still build every
  release; only a future re-reduce or update is affected, and the monthly run reports it.
- **[wordfreq no longer installs]** → Only re-reduce and update need it, not releases; the pinned
  Python version keeps it installable for as long as that Python runs.
- **[zstd drift in the builder]** → `Cargo.lock` pins it; a lock update that changes the pack fails
  the check lane on the pull request that makes it, and the fix is a reviewed update of
  `pack.sha256`.
- **[Repository growth]** → ~1 MB compressed at first, then each update's delta; at a few updates a
  year, negligible against 229 MB.

## Migration Plan

1. Land the three modes, the update workflow and the checks, with the release lanes still on the
   live pipeline.
2. Dispatch the update once: it records the sources, keeps the first kaikki snapshot and pushes the
   first tables (what the current pipeline produces that day). Open and merge that pull request.
3. Switch both release lanes and the check lane to Build mode (same pull request as step 2, or the
   next).

Rollback: point the release lanes back at the live pipeline; the tables and `pin.json` are inert.

## Open Questions

- The share of a table whose loss makes the monthly dry run fail (D7) — to set from the first runs.
