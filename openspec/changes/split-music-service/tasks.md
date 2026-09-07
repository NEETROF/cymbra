> **Dormant change.** Do not start it until a trigger in `proposal.md` fires.
>
> Hard preconditions, both in `harden-module-boundaries`:
> - **groups 9 and 10** — `AppError` can express a transport failure, and the six
>   outbound seams stop discarding it. Moved there because they are corrections of
>   current defects, not split preparation: a dormant change is the wrong home for work
>   that pays off today.
> - **group 5** — `backend/server` no longer carries music code, so this change starts
>   from wiring rather than from untangling 301 interleaved lines of `main.rs`.

## 1. CI hygiene — before a second binary exists

- [ ] 1.1 Reconcile the two `--ignore-filename-regex` lists (`.github/workflows/rust.yml`, `.github/workflows/sonar.yml`) into a single source; they have already diverged.
- [ ] 1.2 Decide and record whether coverage stays one workspace-wide run with one threshold after the split, or becomes per-binary. Prefer keeping one run.

## 2. Service identity

- [ ] 2.1 Extend `Claims` (`backend/platform/src/token.rs`) with a component identity distinct from `sub`, and decide the credential form (signed component token reusing existing key material, or mTLS — see design Open Question 2).
- [ ] 2.2 Add verification: a component credential is not accepted as an end-user identity, and an end-user token is not accepted as a component credential.
- [ ] 2.3 Support calls that carry no end-user token at all (the sign-in path provisions accounts before a user token exists).
- [ ] 2.4 Test: both substitution directions are refused; a pre-authentication internal call succeeds on the component credential alone.

## 3. Resolve the seams — still in-process, no process split yet

- [ ] 3.1 Inventory the 17 outbound seams (11 → `UserPort`, 4 → `PlanSource`, 1 → flags, 1 arriving with `harden-module-boundaries` from `soundfont.rs`) and classify each as **served call** or **granted read**, using the design D2 criterion. Record the reason per seam.
- [ ] 3.2 For the served ones: define them on an **internal** surface, never on the audience-facing contract. None of the four `UserPort` methods music calls has a usable RPC today — `GetAccountRequest` is empty, `GetPlayerProfile` derives the viewer from the token, `listable_profiles`/`age_eligible_profiles` have none.
- [ ] 3.3 For the granted ones: write the narrow read grants (named tables, read-only) into the role provisioning, each with its recorded reason. Model: the existing worker-path read at `backend/music/src/pg_streak.rs:194`.
- [ ] 3.4 Replace music's concrete `Arc<FlagService>` (`backend/music/src/module.rs:171`, `:303`) with its own read-only `FlagService`, modelled on `backend/worker/src/flags.rs:29-56` (`NoopResolver` + `NoopBus`, no user pool). Verified safe: the single evaluation site `caller_may_see_percussion` (`module.rs:319-343`) never consults the admin resolver.
- [ ] 3.5 Point every seam at its adapter, still resolved in-process. At the end of this task the code is split-ready and nothing has moved.
- [ ] 3.6 Test: each served seam behaves identically through its adapter; each granted read is refused when the grant is absent.

## 4. The second process

- [ ] 4.1 Create the music binary and its composition root: config, telemetry, interceptors, CORS, gRPC-web, pools, the Axum router relocated by `harden-module-boundaries`.
- [ ] 4.2 Move the six music trait adapters still in `backend/server/src/flags.rs` (~:98, :138, :187, :234, :383, :409) — `harden-module-boundaries` does not relocate them.
- [ ] 4.3 Remove the music wiring from `backend/server/src/main.rs` and mount nothing music-related there.
- [ ] 4.4 Add trace-context propagation across the process boundary (`backend/platform/src/telemetry.rs`); there is no propagator today because there is no outbound call.
- [ ] 4.5 Decide where `crates/score-crawler` lands — it is the third dependant of `cymbra-music` (design Open Question 3).
- [ ] 4.6 Test: every client-visible service is reachable at the same address with the same authentication as before the move.

## 5. Deployment

- [ ] 5.1 Add the `music` service to `backend/deploy/docker-compose.prod.yml`; the `SCORES_DIR` and `SOUNDFONTS_DIR` bind-mounts **migrate** to it rather than being duplicated.
- [ ] 5.2 Add the Caddy route for the music gRPC prefixes ahead of the catch-all. This file lives on the box and is deployed by `scp` + reload, outside CI — record it as a manual step per environment in `backend/deploy/DEPLOY.md`.
- [ ] 5.3 Make readiness per-component: `deploy.sh` must not report a deployment green while one process is dead.
- [ ] 5.4 Update the runbook: rollback is re-mounting the music services in the main binary — the crate boundary is unchanged, only the composition root differs.

## 6. Verification

- [ ] 6.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] 6.2 `cargo llvm-cov --workspace --fail-under-lines 80` passes with both binaries.
- [ ] 6.3 `openspec validate split-music-service --strict` passes.
- [ ] 6.4 Manual: stop the music process and confirm the app degrades on music surfaces only — sign-in, account and plans still work, and the deployment reports unhealthy.
- [ ] 6.5 Manual: delete a test account and confirm erasure is still one transaction covering every schema.
- [ ] 6.6 Manual: with the music process briefly unavailable, load the catalogue and confirm the missing contributor credits produce diagnostic records rather than a silent gap.
