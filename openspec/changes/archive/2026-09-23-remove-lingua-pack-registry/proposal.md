## Why

The Lingua console shows a registry of data packs that is not true. Production displays
`0.0.0-testdata`: the 1 142-byte test fixture, never the ~1.5 MB pack readers actually have (`main`
has since moved to `0.0.1-testdata`, 1 326 bytes — the fixture still). That
is structural, not a missed refresh — the registry is a manifest committed with the repo, and the
test that keeps it fresh rebuilds the **testdata** pack, because CI does not download the ~200 MB
of real sources. The real pack exists only inside the release workflows (`gen:pack:real`), and
never reaches the manifest.

It also drives nothing: distribution stays bundled in the extension, and the over-the-air update
it was laid down for does not exist. Meanwhile it costs — every change to a pack's content forces
`emit-manifest` and a commit, or `cargo test -p lingua-pack` fails, to protect a figure that is
false. It surfaced exactly that way during the expression-table work.

## What Changes

- **BREAKING (wire):** remove `rpc AdminListDataPacks` and its messages from
  `backend/lingua/proto/lingua_admin.proto`. The only client is the back office, changed in the
  same pull request; the title carries the `!` marker so `buf breaking` reports the break rather
  than failing on it.
- Remove the backend registry: `pack_registry.rs`, the embedded `packs-manifest.json`, and the
  registry's slice of the admin module and its gRPC adapter.
- Remove the pipeline's registry tooling: `manifest.rs`, the `lingua-pack-manifest` binary, the
  `emit-manifest` and `check-manifest` modes of `scripts/lingua-data/build.sh`, and the staleness
  test that pinned the manifest to the fixture.
- Remove the "Packs de données" section of the Lingua console, its store slice, its e2e fake and
  its locale keys.
- **Re-source the studied-language filter.** The section was not its only use: the console's
  studied-language filter listed the languages of the registered packs. It now lists the languages
  of the usage report's per-language breakdown — the report is not itself filtered by language, so
  this is not circular — and always keeps the language currently selected.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `admin-lingua-console`: **Registry of data-pack versions** is removed. **Localised async states
  on the screen** no longer lists packs among the screen's asynchronous resources. A requirement is
  added for where the studied-language filter takes its languages, which the registry used to
  supply implicitly.

## Impact

**Products.** Back office only, plus the Lingua backend module and the pack pipeline behind it.
Nothing reaches readers: licence attribution reaches them through the extension's own credits
(`notice()` / `licences()` in the review page), never through the console. Cymbra ID, Music,
Live and the site are untouched, and nothing is consumed from them.

**Removed, not replaced.** The registry and everything that existed only to keep it current. When
over-the-air pack updates are built, the registry returns in the shape that work needs — fed by
wherever the packs are actually stored, not by a committed file that can only ever describe the
fixture.

**Contract.** `AdminListDataPacks` leaves `lingua_admin.proto`. The back office deploys on merge
and stops calling it at once; the backend drops it at its next release. The one pairing that
could meet the missing RPC is a back office older than this change against a backend newer than
it — in practice, a tab left open across both deploys — and it shows the section's localised
error state, not a crash.

**Release.** The `!` marker makes release-please record the break in the backend's changelog.
