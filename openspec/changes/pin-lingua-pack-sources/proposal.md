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
| AGID `infl.txt` (inflections) | GitHub raw, `master` branch | no |
| CEFR-J 1.5, Octanove C1/C2 (levels) | GitHub raw, `master` branch | no |
| wordfreq (frequencies) | PyPI, unversioned — the project is sunset, last release 3.1.1 (2023-11-21) | no |

The `actions/cache` meant to hold them is evicted after seven idle days, and no entry survives
between releases. So also: the Chrome/Firefox and Safari packs, built by two workflows at
different times, can hold different dictionaries; Mozilla's reviewer, told to rebuild the pack from
`REVIEWERS.md`, gets that day's sources and different bytes; and `pack_version` is the constant
`1.0.0` whatever the pack contains, so no one — reader, support, reviewer — can tell which
dictionary a package carries.

The translation engine had the same exposure and was pinned in `add-lingua-translation-delivery`
(a hash in the repository, the bytes in a GitHub Release that never expires). The dictionary is
the last input of a Lingua package still taken live.

## What Changes

- **The dictionary is pinned.** A pin file in the repository names one dictionary snapshot: the
  reduced tables the pack is built from (forms, frequencies, glosses, levels, expressions, NOTICE,
  manifest), kept as a GitHub Release asset and checked by sha256, together with the sha256 of the
  pack those tables must build — plus, for provenance, the address, date and sha256 of each raw
  source they were reduced from.
- **Releases build from the pin, never from live upstream.** `lingua-extension-release` and
  `lingua-apple-release` fetch the pinned tables, run the pack builder offline, and refuse a pack
  whose sha256 is not the pinned one. The Chrome, Firefox and Safari packages of a release carry
  the same dictionary, byte for byte. No release downloads from kaikki, GitHub raw or PyPI.
- **Refreshing the dictionary becomes a decision.** A manual workflow downloads the live sources,
  runs the reducer with a pinned wordfreq, and reports what changed against the current pin (lemmas,
  glosses, levels and expressions added, removed or changed; pack size). Its result is a new
  snapshot and a pin change reviewed in a pull request like any other change.
- **`pack_version` names the snapshot**, so a package says which dictionary it carries.
- **The reviewer rebuilds offline.** The AMO source archive carries the pinned tables; its
  instructions rebuild the pack with no download and get the pinned bytes.
- **A missing snapshot is caught on pull requests**, not on release day: the extension's check lane
  verifies that the pinned asset exists and matches.

## Capabilities

### New Capabilities

None — building and versioning packs belong to `lingua-data-packs`.

### Modified Capabilities

- `lingua-data-packs`: **modified** — **Reproducible offline build** (a release builds only from a
  pinned snapshot; the same snapshot gives the same pack everywhere; the raw sources are recorded,
  still never committed). **Added** — **A dictionary refresh is a reviewed decision** (live sources
  are read only by the refresh workflow, which reports the difference) and **A pack says which
  dictionary it is** (`pack_version` identifies the snapshot).

## Impact

**Products.** Cymbra Lingua only: the extension's Chromium, Firefox and Safari packages (the
Safari package ships inside `apps/lingua-apple`), their two release workflows, the extension's
check lane and the AMO source archive. ID, Music, Live, the back office and the site are untouched.
`apps/lingua-agent` builds its pack from test data and is not affected.

**Consumed, not redeclared.** The pack format and builder (`crates/lingua-pack`), the reducer
(`scripts/lingua-data/reduce-en-fr.py`), the licence rules of **Licence hygiene** and the size
budget, all unchanged. The pinning pattern — a hash in the repository, bytes in a never-expiring
GitHub Release — is the one `add-lingua-translation-delivery` introduced for the engine.

**Code.** `scripts/lingua-data/build.sh` (a pinned mode that downloads nothing live, and the live
mode the refresh uses), a pin file beside it, a refresh workflow, the two release workflows (their
raw-source cache removed), `apps/lingua-extension/tool/make_source_archive.sh` and `REVIEWERS.md`.

**Licences.** The reduced tables are what every pack already redistributes, under the pack's own
notices. Republishing the **raw** sources is a separate question per source (CEFR-J's terms in
particular) and is not required by this change.

**Not changed.** The dictionary's content on the day the first snapshot is taken: it is whatever
the current pipeline produces then, and later changes go through the refresh.
