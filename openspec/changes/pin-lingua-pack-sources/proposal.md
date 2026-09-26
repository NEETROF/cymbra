## Why

Every Lingua release rebuilds the dictionary — the `en→fr` pack bundled in the Chromium, Firefox
and Safari packages — from whatever its upstream sources hold that day, with no version and no
hash. A source that moves, changes format or vanishes blocks every release of the extension and
of the Safari app; and when nothing breaks, the dictionary still changes between two releases
without anyone having decided it. The requirement the pack already carries — **Reproducible
offline build**, "from dated sources, two runs over the same sources give identical packs" — is
therefore not met.

Measured on 2026-09-25, `scripts/lingua-data/build.sh` fetches, at every run of
`lingua-extension-release` and of `lingua-apple-release`:

| Source | Where from | Pinned? |
|---|---|---|
| kaikki frwiktionary "Anglais" (glosses, expressions) | `kaikki.org`, 201 MB, **regenerated daily** (Last-Modified: the day before) | no |
| AGID `infl.txt` (inflections) | GitHub raw, `master` — a branch **that no longer exists**: the repository's default is `v2`, the file lives only on `v1` (2009), and the URL works through a leftover redirect | no |
| CEFR-J 1.5, Octanove C1/C2 (levels) | GitHub raw, `master` | no |
| wordfreq (frequencies) | PyPI, unversioned — the project is sunset, last release 3.1.1 (2023-11-21) | no |

The `actions/cache` meant to hold them is evicted after seven idle days, and no entry survives
between releases. So also: the Chrome/Firefox and Safari packs, built by two workflows at
different times, can hold different dictionaries; Mozilla's reviewer, told to rebuild the pack from
`REVIEWERS.md`, gets that day's sources and different bytes; pull requests never build the real
pack, so a change that breaks it or its 5 MB budget is found on release day; and `pack_version` is
the constant `1.0.0` whatever the pack contains.

The translation engine had the same exposure and was pinned in `add-lingua-translation-delivery`.
The dictionary is the last input of a Lingua package still taken live.

## What Changes

- **The reduced tables are committed.** The tables the pack is built from — forms, frequencies,
  glosses, levels, expressions, NOTICE, manifest; about 3.4 MB of sorted text — live in the
  repository, under their own licences. Git is their pin, and a change of dictionary is a diff
  anyone can read in a pull request. The raw sources stay out of git, as today; the binary pack
  too.
- **Releases build from the committed tables, never from upstream.** `lingua-extension-release`
  and `lingua-apple-release` run the pack builder offline on them and refuse a pack whose sha256 is
  not the recorded one. The Chrome, Firefox and Safari packages of a release carry the same
  dictionary, byte for byte. No release reads kaikki, GitHub raw or PyPI.
- **Pull requests build the real pack.** The extension's check lane builds it from the tables on
  every pull request, checking its sha256 and its budget.
- **The raw sources are pinned for re-reduction.** GitHub-hosted sources by commit and sha256;
  wordfreq by version and hash; kaikki — the one that moves daily — kept as a compressed snapshot
  in a GitHub Release (about 14 MB). A change to the reduction rules is applied to those same raw
  bytes, so its diff shows the rule change and nothing else.
- **Updating the dictionary is a decision.** A manual workflow fetches the live sources, reduces
  them and pushes the result to a branch, with a report of what changes; a maintainer opens the pull
  request. It never opens or approves one itself.
- **Upstream breakage is noticed before it is needed.** The same workflow runs monthly in a dry
  mode: it reports how far upstream has drifted and fails loudly if a source is gone or its format
  changed. It commits and publishes nothing.
- **`pack_version` names the snapshot**, so a package says which dictionary it carries.
- **The reviewer rebuilds offline**: the AMO source archive already contains `scripts/lingua-data`,
  so it carries the tables; the pack rebuilds from them with no download, byte for byte.

## Capabilities

### New Capabilities

None — building and versioning packs belong to `lingua-data-packs`.

### Modified Capabilities

- `lingua-data-packs`: **modified** — **Reproducible offline build** (the reduced tables are
  committed; releases and pull requests build only from them; one pack per release; raw sources
  pinned, still never committed). **Added** — **A dictionary update is a reviewed decision**,
  **Upstream breakage is detected before it is needed** and **A pack says which dictionary it is**.

## Impact

**Products.** Cymbra Lingua only: the extension's Chromium, Firefox and Safari packages (the
Safari package ships inside `apps/lingua-apple`), their two release workflows, the extension's
check lane and the AMO source archive. ID, Music, Live, the back office and the site are untouched.
`apps/lingua-agent` builds its pack from test data and is not affected.

**Consumed, not redeclared.** The pack format and builder (`crates/lingua-pack`), the reducer
(`scripts/lingua-data/reduce-en-fr.py`), the licence rules of **Licence hygiene** and the size
budget, all unchanged.

**Repository.** About 3.4 MB of TSV (roughly 1 MB once git compresses it) in a 229 MB repository,
plus each update's delta. The rule "generated data is never committed" is narrowed on purpose: raw
sources and the binary pack still never are; the reduced tables now are, because they are the
pack's human-readable source and its review surface.

**Code.** `scripts/lingua-data/build.sh` (build from the committed tables; a re-reduce mode; an
update mode), the tables and their record under `scripts/lingua-data/tables/en-fr/`, a
`lingua-pack-update` workflow, the two release workflows (their raw-source cache and `pip install`
removed), the extension's check lane, `REVIEWERS.md` and `SOURCES.md`.

**Licences.** The tables are what every pack already redistributes, under the notices it embeds;
the kaikki snapshot is CC BY-SA 4.0 / GFDL. AGID, CEFR-J and Octanove are not republished: they are
fetched from their own repositories at a pinned commit — which also keeps CEFR-J, whose terms allow
commercial use with citation but say nothing of redistribution, out of the question.

**Not changed.** The dictionary's content on the day the tables are first committed: it is what the
current pipeline produces then.
