# Design — harden-ci-boundaries

## Context

Seventeen workflows, named on three axes at once: by deployable (`back-office`, `site`,
`backend-image`, `crawler-image`), by stack (`rust`, `flutter`), and by verb (`build`,
`sonar`, `commitlint`, `release-please`, `frb-codegen`). With one product that reads fine.
With three you cannot tell from a name whether a workflow is product-scoped or repo-wide.

The trigger for doing this now is sequencing, not tidiness: lingua v1 is local-only
(`crates/lingua-core`, `apps/lingua-extension`, `apps/lingua-agent`), so it touches CI and
nothing else. Measured against the current filters:

| lingua v1 unit | covered by |
|---|---|
| `crates/lingua-core` | `rust`, `sonar` (glob `crates/**`) — silently correct |
| `apps/lingua-extension` | **nothing** |
| `apps/lingua-agent/rust/` | `rust` — and wrongly `frb-codegen` |

## Goals / Non-Goals

**Goals:**
- A unit that no workflow covers fails loudly instead of shipping unchecked.
- A workflow's trigger selects exactly what its job verifies.
- A workflow's name says whether it is product-scoped, without opening the file.

**Non-Goals:**
- Anything under `backend/` — that is `harden-module-boundaries`.
- Splitting the coverage gate: it stays one workspace-wide run with one threshold.
- Per-app deploy pipelines beyond what exists.

## Decisions

### D1 — Fail on an uncovered unit, rather than a catch-all that runs something

**Resolved in implementation: coverage is derived from the workflows, not from a
manifest.** A manifest is a second source of truth that drifts — it keeps claiming a unit
is covered after the workflow that covered it changed. Reading the filters cannot drift,
and adding a workflow that selects a unit makes the check pass with nothing else to
update. `scripts/check_ci_units.py`, run by the `ci-units` workflow on every pull request.

Two things the implementation had to get right, both found by testing rather than by
reasoning:

- **Both filter mechanisms count.** Four workflows filter at `on.*.paths`; seven always
  start and gate their real jobs with a `dorny/paths-filter` step. Reading only the
  top-level filter reported the second group as watching nothing.
- **Match against real files, never a probe.** A first version probed
  `<unit>/rust/src/probe.rs`, which made every app match the `apps/*/rust/**` glob of
  `rust`/`sonar` — so a brand-new TypeScript app was reported as watched and the guard
  passed on exactly the case it exists for.

**The gap was not hypothetical.** `packages/cymbra_flags` ships `test/flags_test.dart`
that no trigger ran: melos manages `packages/**`, so `melos run test` covers it, but
`flutter` and `sonar` filtered on `apps/music/**` only. `release-build.yml` already
carries comments about an earlier incident of the same shape — a step scoped to
`apps/music` "silently dropped cymbra_flags". Fixed by adding `packages/**` to both
filters.

Two shapes were considered. A **catch-all workflow** that runs a generic check on any
unmatched path is tempting, but a generic check on an unknown stack is either vacuous or
wrong, and a green vacuous job is worse than no job — it looks like coverage. So the rule
is: an unmatched unit **fails the build and names itself**. Adding an app becomes a
deliberate act of also declaring how it is checked.

The criterion that decided the mechanism: it must fail on the **pull request**. A check
that only fires after merge reports a gap that is already in `main`.

### D2 — `frb-codegen`: narrow the trigger, unless a second frb app is imminent

The contradiction is that the trigger spans `apps/*` while the job is hardcoded to music.
Narrowing the trigger to `apps/music/**` is honest today and is two lines. Making the job
iterate is more general, but it is generality with one consumer — the same speculative
move this repo has already paid for elsewhere.

Recommendation: narrow. Revisit if a second flutter_rust_bridge app is actually planned.

**Resolved: narrowed** to `apps/music/rust/**`, `apps/music/lib/src/rust/**` and
`apps/music/pubspec.yaml`. Verified by test 2.3: a Rust crate under another app is now
watched by `rust` and `sonar` — which handle it correctly, being `cargo --workspace`
commands — and no longer by `frb-codegen`.

The audit (task 2.2) found no second instance. `rust` and `sonar` also use
`apps/*/rust/**`, but their jobs are `cargo fmt --all`, `cargo clippy --workspace` and
`cargo llvm-cov --workspace`, which genuinely handle every workspace member. The
distinction to keep: a wildcard is honest when the job is generic over what it selects.

### D3 — The prefix is the target, never the stack

`flutter` is the cautionary case: the name was true when there was one Flutter app, and
becomes a lie at the second. A stack name describes how a unit is built, which is exactly
the property most likely to be shared by a future product. The deployable or app it serves
is the property that stays unique.

**Resolved: `rust` is NOT split, and NOT renamed.** The original reasoning — that
`backend/**` and `crates/**` are different targets sharing a toolchain — ignored what the
job does. `rust.yml` is a single workspace-wide run: `cargo fmt --all`,
`clippy --workspace`, `build --workspace`, and `llvm-cov --workspace --fail-under-lines
80`, which is the aggregate coverage gate CLAUDE.md mandates. Splitting it leaves two bad
options: run `--workspace` twice and compile everything twice, or scope with `-p` and lose
the single aggregate gate.

And the name breaks no rule. The convention forbids a *stack* name because it stops being
true when a second product uses that stack — but `rust` never claimed a product scope. It
has no target prefix, so it reads as repo-wide, and it genuinely is: every workspace
member, including a future `crates/lingua-core`. Same for `sonar`.

The names that were actually false are the ones renamed: `flutter` covered only
`apps/music/**` (and now `packages/**`), and `build` covered the same paths under a name
that said neither which app nor which check.

### D4 — Branch protection matches job names, not workflow names

Checked before renaming anything, and it contradicted the risk written in the Risks
section below: `required_status_checks` on `main` lists `rust`, `flutter`, `frb`, `sonar`,
`pr-title`, `android`, `linux`, `macos`, `windows` — all **job** identifiers. Renaming a
workflow file or its `name:` therefore changes nothing for protection; renaming or removing
a *job* is what silently stops a gate.

So the renames are safe as long as no job is touched, which is how they were done. Worth
knowing for the future: `music-build` and `music-release` both declare jobs named
`android`/`linux`/`macos`/`windows`, so those four required checks are satisfied by
whichever workflow ran.

## Risks / Trade-offs

- **[A renamed workflow silently stops gating]** → ~~the real risk of group 3~~. **Checked
  and wrong**: protection matches *job* names, not workflow names (D4). The renames touch
  no job, and all nine required checks are still produced. The risk is real for a *job*
  rename, which this change does not do.
- **[The uncovered-unit check becomes noise]** → it fires only on paths matching no
  workflow at all, which is rare by construction; if it fires often, the declaration
  mechanism is wrong, not the rule.
- **[Renaming churns open pull requests]** → land group 3 when few branches are in flight,
  or accept one round of re-runs.
- **[Splitting `rust` doubles backend CI time]** → moot: it is not split (D3). The
  concern was real and is the reason it is not.

## Migration Plan

1. **Groups 1 and 2** — coverage rule and the `frb-codegen` fix. Both are additive or
   two-line; neither renames anything, so nothing in branch protection moves.
2. **Group 3** — renames, in one commit so the required-check window is as short as
   possible, followed immediately by the settings check.

Rollback: every step is a workflow file, revertable per commit. The only step with state
outside the repo is 3.5 (branch protection), which is a manual settings edit either way.

## Open Questions

- Which mechanism for D1 — path-matching job or declared-units manifest? Decide in task
  1.2 against the pull-request criterion.
- Does `apps/*/rust/**` belong to `engine-check` or to each app's own check? It is FFI for
  one app, so arguably the latter; but there is only one such app today, so either is
  defensible. Decide in task 3.1.
- Should `crawler-image` become `crawler-build` for consistency, or does `-image` earn its
  place by naming the artifact kind? Both `backend-image` and `crawler-image` currently use
  it; keeping it is defensible if it is applied consistently.
