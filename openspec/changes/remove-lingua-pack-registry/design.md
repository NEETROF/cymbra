## Context

`add-lingua-back-office` (D5) added a read-only registry of data packs to the Lingua console:
pair, pack version, analyzer version, build date, size, NOTICE. Its source is
`backend/lingua/packs-manifest.json`, embedded into the backend at compile time
(`pack_registry.rs`, `include_str!`), produced by `scripts/lingua-data/build.sh emit-manifest`,
and guarded by `the_committed_pack_manifest_matches_a_fresh_build` in `crates/lingua-pack`.

That guard rebuilds the **testdata** pack and requires the committed manifest to describe it.
CI never builds the real pack outside the release workflows, which download ~200 MB of sources
for `gen:pack:real`. So the registry can only ever describe the fixture: production shows
`0.0.0-testdata`, 1 142 bytes, and `main` holds `0.0.1-testdata`, 1 326 bytes — each with a NOTICE
that ends "(test fixture)".

The blast radius was measured with graphify (Rust graph, from `DataPack` and
`admin_list_data_packs`) and confirmed by grep across the back office, the pipeline and CI. It is
confined to the Lingua module, its pipeline and the Lingua console.

## Goals / Non-Goals

**Goals:**

- Stop showing a registry that describes the test fixture as though it were what readers have.
- Remove everything that existed only to keep that registry current, including the chore it
  imposed on every pack change.
- Keep the console's studied-language filter working, which the registry fed implicitly.

**Non-Goals:**

- Building a registry that describes the shipped pack. That belongs with over-the-air pack
  updates, when there is a real store of packs to read from.
- Changing how packs are built, versioned or released. `gen:pack:real` and the release workflows'
  size guard are untouched.
- Changing the NOTICE readers see. It comes from the pack through the extension's credits.

## Decisions

### Remove rather than correct

Making the registry true would mean feeding it from the release workflows — the only place the
real pack exists — into a manifest that is compiled into a backend released independently of
the extension. That couples two release trains to keep an informational table current, for an
over-the-air update that does not exist. Labelling it as the CI fixture would keep a table nobody
needs, and keep its maintenance.

The repository's own rule on internal transport applies here by analogy: infrastructure kept warm
for a future that has not been built costs maintenance and decays into documentation of something
that never happened. When over-the-air updates are built, their registry is written against the
store they read from, in the shape they need.

### The studied-language filter reads the usage report

The registry was not only a table. `LinguaView.vue` built the studied-language filter from the
distinct `studied` codes of the registered packs, chosen because that list does not move with the
usage window.

The replacement is the per-language breakdown of `AdminGetLinguaUsage`, which the screen already
loads. It is not circular: that report takes only the window, and the language filter applies to
the per-day series alone. The list is the languages present in the breakdown, plus the language
currently selected — so a window with no activity in the selected language never leaves the
filter on a value it no longer offers.

The behaviour that changes: a language with no activity in the window is not offered. Filtering the
series by it would show zeros throughout, so offering it offered nothing.

*Rejected — a constant list.* Lingua studies one language today, so `["en"]` would be true now
and silently wrong the day a second pack ships.

*Rejected — dropping the filter.* It is equivalent to "every language" while there is one, but
the series RPC already takes the parameter, and the filter costs nothing once its source is
honest.

### One pull request, with the breaking marker

`AdminListDataPacks` is part of the external contract. The `proto` workflow runs `buf breaking`
with the `FILE` rule against the target branch, which refuses a removed RPC. The pull request title
carries the Conventional Commits `!` marker, so the gate reports the break without failing and
release-please records it.

A single pull request is safe because the deployables go out in the right order on their own: the
back office deploys on merge and stops calling the RPC at once; the backend drops it at its next
release. The only pairing that can meet the missing RPC is a back office older than this change
against a backend newer than it — a tab left open across both deploys — and the section's `Async`
union renders its localised error state for that, not a crash.

## Risks / Trade-offs

**The registry's NOTICE was a place an admin could read the attributions** → the NOTICE lives in
the pack, is shown to readers in the extension's credits, and is in the repository under
`scripts/lingua-data`. No obligation was ever met through the console.

**A future over-the-air update will need a registry again** → it will need one fed by the packs it
distributes, which this one never was. Nothing here would have been reusable as it stood: its
source, its freshness guard and its build step were all tied to the fixture.

**The breaking marker bumps the backend's version** → intended: the contract loses an RPC, and the
changelog should say so.

## Migration Plan

1. Merge: the back office deploys and no longer calls `AdminListDataPacks`.
2. Next backend release: the RPC, the registry and the embedded manifest are gone.
3. Rollback: revert the pull request. The manifest can be regenerated with the reverted
   `emit-manifest`; nothing in it was ever data that could be lost.

## Open Questions

None.
