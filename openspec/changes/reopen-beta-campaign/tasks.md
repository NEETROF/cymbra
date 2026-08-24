## 1. Service and repository

- [ ] 1.1 `MembershipRepo`/campaign repo: `reopen(campaign_id)` clearing `closed_at` (no migration — the column is already nullable), plus the fake's twin
- [ ] 1.2 `PlanService::reopen_campaign(key, actor, reason)`: admin-guarded, audited like `close_campaign`, refusing an unknown key with the same neutral shape
- [ ] 1.3 It clears **only** `closed_at` — a test asserts `enrollment_closes_at` survives untouched
- [ ] 1.4 `reactivatable_members(key)`: how many unrevoked, unexpired memberships a reopening would restore, for the confirmation

## 2. RPC

- [ ] 2.1 `ReopenCampaign(ReopenCampaignRequest)` in `plans.proto`, beside `CloseCampaign`
- [ ] 2.2 Handler wiring + the same admin guard as its sibling
- [ ] 2.3 Regenerate the Dart and TypeScript stubs (`melos run gen-grpc`, `yarn gen`)

## 3. Back office

- [ ] 3.1 The action on a closed campaign's row, where "Fermer la campagne" sits on an open one
- [ ] 3.2 Confirmation naming the count: "N adhésions vont être réactivées"
- [ ] 3.3 A closed campaign's member list shows its rows as inactive
- [ ] 3.4 Where the two kinds are managed, make visible that closing a trial does not revoke the premium it granted
- [ ] 3.5 fr/en strings

## 4. Tests

- [ ] 4.1 Reopen restores every unrevoked member, with no flag edit and no re-enrolment
- [ ] 4.2 A membership revoked before the close stays out after the reopen
- [ ] 4.3 Reopening a campaign whose enrolment is closed leaves redemptions refused
- [ ] 4.4 A closed trial campaign's entitlement runs to its own end; reopening grants nothing new
- [ ] 4.5 Back office: the action appears only on closed rows, and the confirmation carries the count

## 5. Gates

- [ ] 5.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, workspace tests
- [ ] 5.2 Back office `typecheck`, `lint`, `vitest run`
- [ ] 5.3 `openspec validate reopen-beta-campaign --strict`

## 6. Ops

- [ ] 6.1 Once shipped, drop the manual `UPDATE plans.beta_campaigns SET closed_at = NULL` from the runbook, and finish `add-drums-access` task 9.8 with the real control
