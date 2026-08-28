> **Dormant change.** Do not start groups 3–7 until a trigger in `proposal.md` fires.
> Groups 1 and 2 are corrections of current defects and should be lifted into whatever
> work is in flight — they are not split preparation and must not wait for it.
>
> Hard precondition for groups 3–7: `harden-module-boundaries` is applied.

## 1. Error semantics — do now, independently of the split

- [ ] 1.1 Add `Unavailable` and `DeadlineExceeded` to `AppError` (`backend/platform/src/error.rs`), with their gRPC status mappings both ways.
- [ ] 1.2 Add `From<tonic::Status> for AppError`, preserving the distinction between a domain outcome and a transport failure. Note that `Internal` currently flattens to `"internal error"` (`error.rs` ~:58) — a remote `Internal` must not be indistinguishable from a transport fault.
- [ ] 1.3 Test: a timeout maps to `DeadlineExceeded`, an unreachable callee to `Unavailable`, and a remote not-found stays a not-found.

## 2. Stop the silent seams — do now, independently of the split

- [ ] 2.1 `backend/music/src/module.rs:947` (`attach_review_attribution`): replace `let Ok(acct) =` with a match that logs on a non-domain failure. Visible behaviour unchanged.
- [ ] 2.2 `backend/music/src/module.rs:967` (`attach_public_credit`): same treatment.
- [ ] 2.3 `backend/music/src/grpc.rs:602` (`listable_profiles`): same treatment.
- [ ] 2.4 `backend/music/src/grpc.rs:792` (`get_account` in the admin SoundFont listing): same treatment.
- [ ] 2.5 `backend/music/src/leaderboard_module.rs:297` (`.ok()` on `get_player_profile`): same treatment.
- [ ] 2.6 `backend/music/src/global_leaderboard_module.rs:242` (`.ok()` on `get_player_profile`): same treatment.
- [ ] 2.7 Test: a dependency failure still yields a successful response with the field omitted, **and** emits a diagnostic record; a private profile yields the omission with no record.

## 3. CI hygiene — before a second binary exists

- [ ] 3.1 Reconcile the two `--ignore-filename-regex` lists (`.github/workflows/rust.yml`, `.github/workflows/sonar.yml`) into a single source; they have already diverged.
- [ ] 3.2 Decide and record whether coverage stays one workspace-wide run with one threshold after the split, or becomes per-binary. Prefer keeping one run.

## 4. Service identity

- [ ] 4.1 Extend `Claims` (`backend/platform/src/token.rs`) with a component identity distinct from `sub`, and decide the credential form (signed component token reusing existing key material, or mTLS — see design Open Question 2).
- [ ] 4.2 Add verification: a component credential is not accepted as an end-user identity, and an end-user token is not accepted as a component credential.
- [ ] 4.3 Support calls that carry no end-user token at all (the sign-in path provisions accounts before a user token exists).
- [ ] 4.4 Test: both substitution directions are refused; a pre-authentication internal call succeeds on the component credential alone.

## 5. Resolve the seams — still in-process, no process split yet

- [ ] 5.1 Inventory the 17 outbound seams (11 → `UserPort`, 4 → `PlanSource`, 1 → flags, 1 arriving with `harden-module-boundaries` from `soundfont.rs`) and classify each as **served call** or **granted read**, using the design D2 criterion. Record the reason per seam.
- [ ] 5.2 For the served ones: define them on an **internal** surface, never on the audience-facing contract. None of the four `UserPort` methods music calls has a usable RPC today — `GetAccountRequest` is empty, `GetPlayerProfile` derives the viewer from the token, `listable_profiles`/`age_eligible_profiles` have none.
- [ ] 5.3 For the granted ones: write the narrow read grants (named tables, read-only) into the role provisioning, each with its recorded reason. Model: the existing worker-path read at `backend/music/src/pg_streak.rs:194`.
- [ ] 5.4 Replace music's concrete `Arc<FlagService>` (`backend/music/src/module.rs:171`, `:303`) with its own read-only `FlagService`, modelled on `backend/worker/src/flags.rs:29-56` (`NoopResolver` + `NoopBus`, no user pool). Verified safe: the single evaluation site `caller_may_see_percussion` (`module.rs:319-343`) never consults the admin resolver.
- [ ] 5.5 Point every seam at its adapter, still resolved in-process. At the end of this task the code is split-ready and nothing has moved.
- [ ] 5.6 Test: each served seam behaves identically through its adapter; each granted read is refused when the grant is absent.

## 6. The second process

- [ ] 6.1 Create the music binary and its composition root: config, telemetry, interceptors, CORS, gRPC-web, pools, the Axum router relocated by `harden-module-boundaries`.
- [ ] 6.2 Move the six music trait adapters still in `backend/server/src/flags.rs` (~:98, :138, :187, :234, :383, :409) — `harden-module-boundaries` does not relocate them.
- [ ] 6.3 Remove the music wiring from `backend/server/src/main.rs` and mount nothing music-related there.
- [ ] 6.4 Add trace-context propagation across the process boundary (`backend/platform/src/telemetry.rs`); there is no propagator today because there is no outbound call.
- [ ] 6.5 Decide where `crates/score-crawler` lands — it is the third dependant of `cymbra-music` (design Open Question 3).
- [ ] 6.6 Test: every client-visible service is reachable at the same address with the same authentication as before the move.

## 7. Deployment

- [ ] 7.1 Add the `music` service to `backend/deploy/docker-compose.prod.yml`; the `SCORES_DIR` and `SOUNDFONTS_DIR` bind-mounts **migrate** to it rather than being duplicated.
- [ ] 7.2 Add the Caddy route for the music gRPC prefixes ahead of the catch-all. This file lives on the box and is deployed by `scp` + reload, outside CI — record it as a manual step per environment in `backend/deploy/DEPLOY.md`.
- [ ] 7.3 Make readiness per-component: `deploy.sh` must not report a deployment green while one process is dead.
- [ ] 7.4 Update the runbook: rollback is re-mounting the music services in the main binary — the crate boundary is unchanged, only the composition root differs.

## 8. Verification

- [ ] 8.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] 8.2 `cargo llvm-cov --workspace --fail-under-lines 80` passes with both binaries.
- [ ] 8.3 `openspec validate split-music-service --strict` passes.
- [ ] 8.4 Manual: stop the music process and confirm the app degrades on music surfaces only — sign-in, account and plans still work, and the deployment reports unhealthy.
- [ ] 8.5 Manual: delete a test account and confirm erasure is still one transaction covering every schema.
- [ ] 8.6 Manual: with the music process briefly unavailable, load the catalogue and confirm the missing contributor credits produce diagnostic records rather than a silent gap.
