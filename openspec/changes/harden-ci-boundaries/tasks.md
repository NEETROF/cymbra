> Independent of `harden-module-boundaries` — nothing here touches `backend/`. Do it
> **before lingua**, because lingua v1 is local-only and CI is the only thing it hits.
>
> Groups 1 and 2 are what actually block lingua. Group 3 is legibility and can land
> separately.

## 1. No buildable unit ships without CI

- [ ] 1.1 Inventory what each workflow selects today, and list every unit under `apps/`, `packages/` and `crates/` that no workflow covers. Expected today: none — the gap is structural, not yet realised.
- [ ] 1.2 Choose the mechanism and record why in `design.md`: a job that fails when a changed path matches no declared workflow, or a checked-in manifest of declared units verified in CI. Prefer whichever fails on the pull request rather than after merge.
- [ ] 1.3 Implement it, running on `pull_request`.
- [ ] 1.4 Test: a branch adding `apps/<new>/` with no workflow fails, and the message names the path.
- [ ] 1.5 Test: the same branch with a workflow selecting it passes.
- [ ] 1.6 Document the rule where someone adding an app will read it — `CLAUDE.md`, next to the existing CI conventions.

## 2. A trigger matches its job

- [ ] 2.1 Fix `frb-codegen`: it triggers on `apps/*/rust/**`, `apps/*/lib/src/rust/**` and `apps/*/pubspec.yaml` (~:26-28) but its job is hardcoded to music (`working-directory: apps/music` ~:53, `git diff -- apps/music/…` ~:64). Either narrow the trigger to `apps/music/**`, or make the job iterate the apps the glob selects. Record which and why — narrowing is honest today, iterating anticipates a second frb app.
- [ ] 2.2 Audit the other workflows for the same shape — a wildcard in the trigger, a fixed path in the body. Start with `rust` (`apps/*/rust/**`) and `sonar` (`apps/*/rust/**`).
- [ ] 2.3 Test: a change under a non-music `apps/*/rust/**` does not start `frb-codegen` (or starts it and verifies that app, per the choice in 2.1).

## 3. Names on one axis

- [ ] 3.1 Split `rust` into `backend-check` (`backend/**`) and `engine-check` (`crates/**`, plus `apps/*/rust/**` if it stays there) — one name currently covers the monolith, the shared engine crates and app FFI.
- [ ] 3.2 Rename `flutter` → `music-check` and `build` → `music-build`. Both cover only `apps/music/**` today, and neither name says which app or which check.
- [ ] 3.3 Rename `release-build` → `music-release`, `back-office` → `back-office-check`, `site` → `site-check`.
- [ ] 3.4 Leave `commitlint`, `codeql`, `sonar`, `release-please`, `openspec-archive` unprefixed — that absence becomes the signal for "repo-wide".
- [ ] 3.5 Update every reference to a renamed workflow: branch protection required checks, badges, and any `needs:`/`workflow_call` between workflows. **A renamed required check silently stops gating** — verify in the repo settings after merging, not before.
- [ ] 3.6 Reconcile the two `--ignore-filename-regex` lists (`.github/workflows/rust.yml` ~:78, `sonar.yml` ~:93) into one source; they have already diverged. Do not anchor or split the regex, and keep the coverage gate as a single workspace-wide run.
- [ ] 3.7 Record the naming rule in `CLAUDE.md` alongside 1.6.

## 4. Verification

- [ ] 4.1 Every workflow still parses and its triggers are unchanged except where intended: `python3 -c "import yaml,glob;[yaml.safe_load(open(f)) for f in glob.glob('.github/workflows/*.yml')]"`.
- [ ] 4.2 Open a throwaway pull request touching one file per product and confirm exactly the expected workflows start — no more, no fewer.
- [ ] 4.3 `openspec validate harden-ci-boundaries --strict` passes.
- [ ] 4.4 After merge: confirm in Settings → Branches that no required check still names a removed workflow.
