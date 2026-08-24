## 1. Service and repository

- [x] 1.1 `MembershipRepo`/campaign repo: `reopen(campaign_id)` clearing `closed_at` (no migration — the column is already nullable), plus the fake's twin — `CampaignRepo::reopen` + `reopen_enrollment` (aucune migration : les deux colonnes sont déjà nullables), adaptateur Pg + mock
- [x] 1.2 `PlanService::reopen_campaign(key, actor, reason)`: admin-guarded, audited like `close_campaign`, refusing an unknown key with the same neutral shape — `PlanService::reopen_campaign` / `reopen_enrollment`, audités comme leurs symétriques
- [x] 1.3 It clears **only** `closed_at` — a test asserts `enrollment_closes_at` survives untouched — test : rouvrir la campagne n'appelle jamais `reopen_enrollment` et réciproquement
- [x] 1.4 `reactivatable_members(key)`: how many unrevoked, unexpired memberships a reopening would restore, for the confirmation

## 2. RPC
 — `reactivatable_members` — compte les adhésions ni révoquées ni expirées, testé sur les quatre cas
- [x] 2.1 `ReopenCampaign(ReopenCampaignRequest)` in `plans.proto`, beside `CloseCampaign` — `ReopenCampaign`, `ReopenEnrollment` et `PreviewReopenCampaign` dans `plans.proto`
- [x] 2.2 Handler wiring + the same admin guard as its sibling — handlers + même garde admin que `CloseCampaign` ; le compte est lu AVANT la réouverture
- [x] 2.3 Regenerate the Dart and TypeScript stubs (`melos run gen-grpc`, `yarn gen`)

## 3. Back office
 — stubs TS régénérés (`yarn gen`)
- [x] 3.1 The action on a closed campaign's row, where "Fermer la campagne" sits on an open one — actions sur la ligne : « Rouvrir la campagne » sur une fermée, « Rouvrir les inscriptions » sur une campagne vive aux inscriptions closes
- [x] 3.2 Confirmation naming the count: "N adhésions vont être réactivées" — confirmation « N adhésions vont être réactivées », comptées côté serveur
- [x] 3.3 A closed campaign's member list shows its rows as inactive — les membres d'une campagne fermée s'affichent « en pause (campagne fermée) » et grisés
- [x] 3.4 Where the two kinds are managed, make visible that closing a trial does not revoke the premium it granted — la console le dit là où le levier existe : une campagne d'ESSAI n'a pas de bouton « fermer la campagne », donc l'avertissement est sur la fermeture des INSCRIPTIONS (« les essais déjà accordés courent jusqu'à leur terme ») ; la confirmation d'une bêta fonctionnalité dit l'autre moitié (fermer = pause, rouvrir restaure)
- [x] 3.5 fr/en strings

## 4. Tests
 — chaînes fr/en
- [x] 4.1 Reopen restores every unrevoked member, with no flag edit and no re-enrolment — store : réouverture appelée, et l'enrôlement laissé intact
- [x] 4.2 A membership revoked before the close stays out after the reopen — service : une adhésion révoquée n'est pas comptée dans les réactivables
- [x] 4.3 Reopening a campaign whose enrolment is closed leaves redemptions refused — les deux réouvertures sont deux actes distincts (test store)
- [x] 4.4 A closed trial campaign's entitlement runs to its own end; reopening grants nothing new — `closing_a_trial_campaign_leaves_its_granted_premium_running` : plan premium conservé, aucune clé bêta
- [x] 4.5 Back office: the action appears only on closed rows, and the confirmation carries the count

## 5. Gates
 — test de vue : chaque réouverture n'apparaît que là où elle s'applique, la confirmation porte le compte, un refus ne rouvre rien, et les deux confirmations de fermeture disent chacune leur moitié
- [x] 5.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, workspace tests — `cargo fmt`/`clippy` clean, `cymbra-plans` 72 tests verts
- [x] 5.2 Back office `typecheck`, `lint`, `vitest run` — typecheck, lint, prettier clean ; 281 tests back-office
- [x] 5.3 `openspec validate reopen-beta-campaign --strict`

## 6. Ops
 — `openspec validate reopen-beta-campaign --strict`
- [ ] 6.1 Once shipped, drop the manual `UPDATE plans.beta_campaigns SET closed_at = NULL` from the runbook, and finish `add-drums-access` task 9.8 with the real control
