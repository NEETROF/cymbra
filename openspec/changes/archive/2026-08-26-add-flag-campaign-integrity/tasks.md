## 1. The port

- [x] 1.1 Declare a narrow trait in `cymbra-feature-flags` answering "does a campaign with this key exist?" — **fallibly**: the answer is exists, does not exist, or cannot determine (a `Result`, never an error-swallowing default), because "names no campaign" and "cannot verify" must stay distinguishable all the way to the console; existence only, closed campaigns included, **any campaign kind accepted, trials included**; the crate declares it and never implements it, so it acquires no dependency on `cymbra-plans`
- [x] 1.2 Give the trait a `mockall` generated mock (`#[automock]`), per the repo's rust-testing default, so the validation is testable without plans
- [x] 1.3 Wire the port into the flag service as an optional dependency, following how the service already receives its other collaborators

## 2. The adapter

- [x] 2.1 Implement the port in `backend/server/src/flags.rs`, beside the existing plans → flags bridge at `flags.rs:461` — same seam, same direction; the dependency arrow stays `server → {flags, plans}` with no edge between them
- [x] 2.2 Assert at the composition root (startup check or a wiring test) that the port is present, since an unwired port under the fail-closed rule refuses every beta-scoped write

## 3. Validation at the write boundary

- [x] 3.1 `FlagService::set_value` (`service.rs`): after the effective rollout is resolved — request scope, else the previous override's, else the registry default — and only when it would **set or change** the stored scope, verify a `Beta(key)` names an existing campaign before storing; not in `grpc.rs` after `parse_rollout_opt`, which sees only the request's scope and guards only one adapter
- [x] 3.2 Reject with typed errors — one for a scope naming no campaign, a **distinct** one for existence that could not be determined — both distinguishable from a malformed scope, so the console can say "fix the scope" or "retry later" rather than restate the transport status
- [x] 3.3 Fail closed: when existence cannot be determined (plans read fails, port unwired), refuse a write that would set or change the scope rather than store it unverified
- [x] 3.4 Accept a **closed** campaign's key — the check is existence, not openness; invalidating it would break withdrawing a feature by closing its campaign, which is a scenario in the `runtime-feature-flags` spec
- [x] 3.5 Leave evaluation untouched: a stored override resolves to exactly the audience it did before
- [x] 3.6 Skip the lookup when the write leaves the stored scope untouched — a value-only save or an unrelated edit must succeed even when the directory is down or the stored scope dangles, so a beta-gated flag stays disablable mid-incident

## 4. Backend tests

- [x] 4.1 A `beta:<key>` naming an existing open campaign is accepted
- [x] 4.2 A `beta:<key>` naming an existing **closed** campaign is accepted
- [x] 4.3 A `beta:<key>` naming no campaign is refused, and no override row is written
- [x] 4.4 An unavailable campaign directory refuses a scope-setting write (fail-closed) with a typed error **distinct** from names-no-campaign, and no override row is written
- [x] 4.5 `global`, `staff_only` and `premium_only` writes are unaffected — no campaign lookup happens for them
- [x] 4.6 Evaluation of an already-stored dangling scope is unchanged (it matches staff only, exactly as today), proving the change is write-only
- [x] 4.7 A write with an empty `rollout_scope` over a stored beta scope resolves the effective scope, sees it unchanged, and skips the lookup — it succeeds with the directory down and with a dangling stored scope alike, and stores the same scope as before
- [x] 4.8 A first write against a key whose registry default is beta-scoped is checked even when the request carries no scope — the effective scope is the default's, and it must name a real campaign

## 5. Console

- [x] 5.1 `stores/flags.ts`: surface **both** typed refusals — names no campaign, and could not verify — as their own cases in the existing `Async<T>` union rather than a generic error
- [x] 5.2 `FlagDrawer.vue:35`: split today's single `staleScope` branch into "campaign closed" (legitimate, still gating — present unremarkably) and "campaign does not exist" (a defect — present distinctly, naming the cause)
- [x] 5.3 Localise the dangling marker and **both** refusal messages (fr/en) — "fix the scope" vs "retry later"; an outage message must never claim the campaign does not exist; never a raw transport status in the UI
- [x] 5.4 Component test for the two stale cases and both refusal paths
- [x] 5.5 Confirm the selector itself is unchanged — it already offers only open campaigns and already forbids free text, per its spec

## 6. Gates

- [x] 6.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 6.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [x] 6.3 BO — `yarn test` and the Playwright e2e (pass `BO_E2E_PORT` to avoid colliding with another worktree's dev server)
- [x] 6.4 `openspec validate add-flag-campaign-integrity --strict`

## 7. Manual verification

- [x] 7.1 In the flags panel, scope a flag to an open campaign and save — accepted and audited as before — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 7.2 Close that campaign, reopen the flag, and confirm its scope is still selectable, saveable, and not flagged as a defect — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 7.3 Write a `beta:<key>` naming no campaign through a direct RPC call (bypassing the console, which cannot produce one) and confirm it is refused with an explicit reason and stores nothing — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 7.4 Seed a dangling scope directly in the database, open its flag, and confirm the console marks it distinctly and names the cause — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 7.5 Audit the existing beta-scoped flags once after deploying, and record whether any were already dangling — this decides the open question about a one-off report — VALIDÉ 2026-08-25 en production (backend 0.21.1)
