## Why

Lingua ships next, and its MVP is local-only — `crates/lingua-core`,
`apps/lingua-extension`, `apps/lingua-agent`, no backend. So **the first thing a third
product hits is not the backend, it is CI**, and CI is not ready for it:

- **A new `apps/*` gets no CI at all.** Workflows name their apps one by one
  (`apps/back-office/**`, `apps/site/**`, `apps/music/**`); there is no catch-all and no
  failure when an app matches nothing. `apps/lingua-extension` would merge with zero
  checks and nothing would say so.
- **`frb-codegen` would fire on it wrongly.** It triggers on the glob `apps/*/rust/**` but
  its job is hardcoded to music (`working-directory: apps/music`, then
  `git diff -- apps/music/lib/src/rust`). The glob promises a genericity the body does not
  have.
- **The names no longer say what they cover.** `flutter` covers only `apps/music/**` —
  and so does `build`, with a name that says neither which app nor which check. `rust`
  covers the backend, the shared engine crates and app FFI under one name. Workflows are
  named on three different axes at once (deployable, stack, verb), so you cannot tell from
  a name whether one is product-scoped or repo-wide.

None of this blocks a backend change, which is why it is separate from
`harden-module-boundaries`. It blocks the product that ships first.

## What Changes

**Coverage — no app ships without CI**
- Adding an app under `apps/` (or a package under `packages/`) either matches a workflow
  or **fails loudly**. Silence stops being a valid outcome.
- Decide and record the mechanism: a catch-all job that fails when a changed path matches
  no declared workflow, or a manifest of declared apps checked in CI.

**Correctness — a trigger matches its job**
- Fix the `frb-codegen` contradiction: either narrow the trigger to the app the job
  actually verifies, or make the job iterate over the apps the glob selects.
- Audit the other globs for the same shape (a `*` in the trigger, a hardcoded path in the
  body).

**Naming — one primary axis**
- Rename on `<target>-<verb>`, so a name without a target prefix means repo-wide:
  `rust` → `backend-check` + `engine-check` (the backend and the shared crates are not the
  same scope), `flutter` → `music-check`, `build` → `music-build`, `release-build` →
  `music-release`, `back-office` → `back-office-check`, `site` → `site-check`.
- `commitlint`, `codeql`, `sonar`, `release-please`, `openspec-archive` keep bare
  verb names — they genuinely have no target, and that becomes the readable signal.

**Already landed** (done while scoping this change, recorded here so the naming rule is
not re-litigated): `deploy.yml` → `backend-deploy.yml`; a new `deploy` orchestrator that
takes a target list plus optional version pins, resolves each version from
`.release-please-manifest.json`, and reports requested / resolved / outcome in the run
summary; `back-office-deploy` and `site-deploy` made manual-only and callable.

## Capabilities

### New Capabilities
- `platform-ci-boundaries`: every buildable unit in the repo is covered by CI, a
  workflow's trigger matches what its job actually does, and a workflow's name says
  whether it is product-scoped or repo-wide.

## Impact

**Products**

| Product | Consumed (unchanged) | New / changed |
|---|---|---|
| **Lingua** | nothing yet | its three units get CI on arrival instead of shipping unchecked |
| **Music** | its checks and builds | same jobs, renamed (`music-check`, `music-build`, `music-release`) |
| **Cymbra ID** | backend checks | `rust` split into `backend-check` + `engine-check` |
| **Back-office / Site** | their checks and deploys | renamed for consistency; deploys already done |
| **Live** | nothing | inherits the rule when it arrives |

**Code**: `.github/workflows/*` (17 files), and any branch-protection rule or badge that
names a renamed workflow.

**Not in scope**
- Splitting the coverage gate. `cargo llvm-cov --workspace` stays one run with one
  threshold; the two `--ignore-filename-regex` lists (`rust.yml`, `sonar.yml`) have already
  diverged and are reconciled into one source, but not anchored or split.
- Anything under `backend/` — that is `harden-module-boundaries`.
- Per-app deploy pipelines beyond what already exists.
