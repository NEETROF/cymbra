## 1. The port

- [ ] 1.1 Declare a narrow trait in `cymbra-feature-flags` answering "does a campaign with this key exist?" — existence only, closed campaigns included; the crate declares it and never implements it, so it acquires no dependency on `cymbra-plans`
- [ ] 1.2 Give the trait a `mockall` generated mock (`#[automock]`), per the repo's rust-testing default, so the validation is testable without plans
- [ ] 1.3 Wire the port into the flag service as an optional dependency, following how the service already receives its other collaborators

## 2. The adapter

- [ ] 2.1 Implement the port in `backend/server/src/flags.rs`, beside the existing plans → flags bridge at `flags.rs:461` — same seam, same direction; the dependency arrow stays `server → {flags, plans}` with no edge between them
- [ ] 2.2 Assert at the composition root (startup check or a wiring test) that the port is present, since an unwired port under the fail-closed rule refuses every beta-scoped write

## 3. Validation at the write boundary

- [ ] 3.1 `grpc.rs:140` (`SetFlag`) and `grpc.rs:164` (`SetConfig`): after `parse_rollout_opt` yields a `Beta(key)`, verify the campaign exists before storing
- [ ] 3.2 Reject with a typed error distinguishable from a malformed scope, so the console can explain the cause rather than restate the transport status
- [ ] 3.3 Fail closed: when existence cannot be determined (plans read fails, port unwired), refuse the write rather than store it unverified
- [ ] 3.4 Accept a **closed** campaign's key — the check is existence, not openness; invalidating it would break withdrawing a feature by closing its campaign, which is a scenario in the `runtime-feature-flags` spec
- [ ] 3.5 Leave evaluation untouched: a stored override resolves to exactly the audience it did before

## 4. Backend tests

- [ ] 4.1 A `beta:<key>` naming an existing open campaign is accepted
- [ ] 4.2 A `beta:<key>` naming an existing **closed** campaign is accepted
- [ ] 4.3 A `beta:<key>` naming no campaign is refused, and no override row is written
- [ ] 4.4 An unavailable campaign directory refuses the write (fail-closed), and no override row is written
- [ ] 4.5 `global`, `staff_only` and `premium_only` writes are unaffected — no campaign lookup happens for them
- [ ] 4.6 Evaluation of an already-stored dangling scope is unchanged (it matches staff only, exactly as today), proving the change is write-only

## 5. Console

- [ ] 5.1 `stores/flags.ts`: surface the typed refusal as its own case in the existing `Async<T>` union rather than a generic error
- [ ] 5.2 `FlagDrawer.vue:35`: split today's single `staleScope` branch into "campaign closed" (legitimate, still gating — present unremarkably) and "campaign does not exist" (a defect — present distinctly, naming the cause)
- [ ] 5.3 Localise both the dangling marker and the refusal message (fr/en); never a raw transport status in the UI
- [ ] 5.4 Component test for the two stale cases and the refusal path
- [ ] 5.5 Confirm the selector itself is unchanged — it already offers only open campaigns and already forbids free text, per its spec

## 6. Gates

- [ ] 6.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 6.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [ ] 6.3 BO — `yarn test` and the Playwright e2e (pass `BO_E2E_PORT` to avoid colliding with another worktree's dev server)
- [ ] 6.4 `openspec validate add-flag-campaign-integrity --strict`

## 7. Manual verification

- [ ] 7.1 In the flags panel, scope a flag to an open campaign and save — accepted and audited as before
- [ ] 7.2 Close that campaign, reopen the flag, and confirm its scope is still selectable, saveable, and not flagged as a defect
- [ ] 7.3 Write a `beta:<key>` naming no campaign through a direct RPC call (bypassing the console, which cannot produce one) and confirm it is refused with an explicit reason and stores nothing
- [ ] 7.4 Seed a dangling scope directly in the database, open its flag, and confirm the console marks it distinctly and names the cause
- [ ] 7.5 Audit the existing beta-scoped flags once after deploying, and record whether any were already dangling — this decides the open question about a one-off report
