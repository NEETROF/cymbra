## 0. Before any of this is built

This change is proposed and parked. Nothing below starts until all four hold — the design says why.

- [ ] 0.1 A reader has asked for it, or a device that cannot run the local engine has been identified rather than supposed
- [ ] 0.2 The delivery change has shipped and the "Traduction étendue" setting exists
- [ ] 0.3 The new privacy text is drafted and reviewed — the annex, the three store listings, the App Store answers — **before** any code
- [ ] 0.4 A per-account limit and its cost ceiling are agreed

## 1. The promise, first

- [ ] 1.1 Rewrite the Lingua annex of the privacy policy (fr + en): page text stays on the device **unless** the reader turns on remote translation, in which case the selected sentence is sent to Cymbra, translated, and kept nowhere
- [ ] 1.2 Correct every store listing that states page text never leaves the device, naming remote translation as the one exception, off by default and chosen per device
- [ ] 1.3 Re-check the App Store privacy answers against the new behaviour and record them with the Apple app
- [ ] 1.4 Publish the policy before the first build that can send a sentence

## 2. The service

- [ ] 2.1 Add a translation RPC to the Lingua backend taking the sentence and the selection's offsets, answering the translated sentence and its marked spans — the same shape `TranslatorPort` already has
- [ ] 2.2 Run the same Bergamot model the local engine uses, from the artefact `lingua-engine-build` produces, so the answer is the one the device would have given
- [ ] 2.3 Mark the marks with the same rule as the extension (`translate/reconcile.ts`): translate the selection alone alongside the tagged sentence and reconcile, so both hosts answer identically
- [ ] 2.4 Require a signed-in account, and limit per account
- [ ] 2.5 Keep nothing: no sentence in a log, an error, a metric or a trace; assert it with a test that fails if a request's text reaches the logger
- [ ] 2.6 Measure latency and CPU per request on the deployment target, and record both here

## 3. The extension

- [ ] 3.1 Add a remote `TranslatorPort` implementation behind the existing seam, so no caller learns where the engine ran
- [ ] 3.2 Give the setting its third state (none / local / remote), per device, defaulting to none
- [ ] 3.3 Never cross: a failing local engine sends nothing to the network, a failing remote call downloads nothing; both answer as no engine does
- [ ] 3.4 Tell a signed-out reader who chooses remote that an account is needed, before anything is sent
- [ ] 3.5 Say, in the reader's language, when the account's limit is reached
- [ ] 3.6 Extend `check:variants` so a shipped build with no engine carries no trace of either host

## 4. Prove it

- [ ] 4.1 Translate the same selections through both hosts and assert the answers and their marks match
- [ ] 4.2 Verify with a proxy or a network log that a reader on **local** makes no request, and a reader on **none** makes none either
- [ ] 4.3 Verify on the device that showed the local engine's cost that remote answers without the download and without the resident memory
- [ ] 4.4 Verify the service keeps nothing, from its own logs after a translation

## 5. Gates

- [ ] 5.1 `yarn lint`, `format:check`, `typecheck`, `yarn test` clean from `apps/lingua-extension`, coverage at or above its gate
- [ ] 5.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo llvm-cov --workspace --fail-under-lines 80`
- [ ] 5.3 `yarn build` and `yarn check:variants` pass for all three variants
- [ ] 5.4 `openspec validate add-lingua-remote-translation --strict` passes
