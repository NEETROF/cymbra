# Vendored code

## foliate-js

The book reader (`src/reader/`, change `add-lingua-reader`) renders EPUB with
[foliate-js](https://github.com/johnfactotum/foliate-js), the engine behind Readest.

- **Commit**: `78914aef4466eb960965702401634c2cb348e9b1` (2026-05-01)
- **Licence**: MIT — `foliate-js/LICENSE`, © 2022 John Factotum
- **Refresh**: `tool/vendor_foliate.sh [<commit>]`, then update the commit above, rebuild, and
  re-run the reader's device passes (tasks 1.3 and 6.2 of `add-lingua-reader`)

It has no release and says its API may change at any time, so it is vendored at a pinned commit
rather than taken as a git submodule (every CI checkout would need `submodules: true`) or a git
dependency (an unbuilt package Yarn would fetch on every install).

**What is here.** Only the modules an EPUB needs, unmodified: `view.js`, `epub.js`,
`epubcfi.js`, `paginator.js`, `fixed-layout.js`, `progress.js`, `overlayer.js`,
`text-walker.js`. The other formats and features `view.js` can load on demand — PDF, MOBI,
FB2, CBZ, search, text-to-speech — are not vendored: `build.mjs` resolves each of those
imports to a module that refuses, so none of them reaches the bundle.

**The one file that is ours.** `foliate-js/vendor/zip.js` stands where foliate keeps a
minified rollup build of [@zip.js/zip.js](https://github.com/gildas-lormeau/zip.js). It is
foliate's own rollup input (`rollup/zip.js`), pointed at the npm package — pinned to the
version foliate locks, `2.8.22` (BSD-3-Clause), in `package.json` — so the bundle is built
from source like every other dependency. `build.mjs` maps `@zip.js/zip.js` to that package's
`lib/zip-core.js`, the same entry foliate bundles: no worker, no WebAssembly, no codec of its
own (the browser's `DecompressionStream` inflates).

**Tooling.** `vendor/` is excluded from lint, format and coverage (`eslint.config.js`,
`.prettierignore`, `vitest.config.ts`): it is third-party code, kept byte-for-byte. It is
committed, so the AMO source archive (`tool/make_source_archive.sh`, which archives
`apps/lingua-extension` from git) carries it as source.
