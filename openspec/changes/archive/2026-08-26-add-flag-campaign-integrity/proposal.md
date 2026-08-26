## Why

A flag's rollout scope can be `beta:<campaign>`, and the audience it grants is
resolved by a plain string lookup: `RolloutScope::Beta(key) => self.staff ||
self.betas.contains(key)`. Nothing anywhere checks that `<key>` names a campaign
that exists.

The back-office already does its part — the rollout selector is populated from the
open campaigns and is explicitly "not typed free-hand"
(`FlagDrawer.vue:28`, and the `feature-flags-admin` spec requires it). But that is a
**console convention, not an invariant**: `SetFlag` and `SetConfig` accept any
caller's scope string after `RolloutScope::parse`, which validates the *shape*
(`beta:[a-z0-9-]+`) and nothing else. A script, a `grpcurl` call, a seeding fixture
or a future client can store `beta:mididrums` — it parses, it persists, it is
audited as a legitimate change, and it matches nobody for ever.

The failure is silent in the worst way. Staff match **every** beta scope
(`self.staff || …`), so an operator verifying their own work from an admin session
sees the feature working regardless of whether the key is right. The people it
actually fails for are the testers, who simply see nothing and have no way to
attribute it.

## What Changes

**Referential integrity at the write boundary**

- `cymbra-feature-flags` gains a **port** for campaign existence — a trait it
  declares and does not implement — so it acquires no dependency on
  `cymbra-plans`. Both crates depend only on `cymbra-platform` today and that stays
  true.
- The composition root implements the port beside the **existing** plans → flags
  bridge (`backend/server/src/flags.rs:461`), which already injects the caller's
  plan and beta keys into the evaluation context. Same seam, same placement —
  but not the same error contract: the bridge treats errors as free, which is
  right for evaluation and wrong here, so the port is fallible (see the design).
- A write that **sets or changes** a rollout scope to a `beta:<key>` naming no
  campaign is rejected inside `FlagService::set_value` — the write boundary where
  every sibling invariant already lives — with a typed error the console surfaces
  as a message rather than a raw status. The check sees the **effective** scope
  about to be stored, not merely the request's, so a scope arriving via
  preservation or a registry default is covered, and so is any future non-gRPC
  adapter over the same service.

**Existence, not openness**

- The check asks whether the campaign **exists**, never whether it is open. A closed
  campaign's scope must stay valid: closing a campaign is the documented way to
  withdraw a feature from its members *without a flag edit*, so invalidating the
  stored scope would break that behaviour and force an edit at exactly the moment
  the operator wants none.

**A dangling scope is surfaced as dangling**

- The console already keeps an unrecognised stored scope selectable so the selector
  "never silently rewrites it" (`FlagDrawer.vue:33`). It is shown as an ordinary
  option, which is indistinguishable from a valid one. It gains a distinct
  presentation naming the cause — the campaign no longer exists, or never did.

**Not included:** any change to how scopes are evaluated, to the audience
`beta:<key>` grants, or to the console's selector, which already satisfies its spec.

## Capabilities

### Modified Capabilities

- `runtime-feature-flags`: a beta-scoped rollout SHALL name an existing campaign,
  verified at the service write boundary whenever a write sets or changes the
  stored scope; the verification fails closed, with "names no campaign" and
  "cannot verify" kept distinct; and it tests existence rather than openness so
  closing a campaign keeps withdrawing access without a flag edit.
- `feature-flags-admin`: a stored scope naming no existing campaign is presented
  distinctly from a valid one, and a rejected write surfaces a localised reason
  that distinguishes a bad scope from an unverifiable check.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Platform** (feature flags) | the existing composition-root bridge to plans | one declared port; a validation in the service write path behind both admin RPCs |
| **Back-office** | the same flags RPCs | a dangling-scope presentation and a refusal message |
| **Plans** | — | a campaign-existence read behind the port; no new coupling |
| **Music / ID / Live / Site** | — | untouched |

**Code**

- `backend/feature-flags/src/`: the port trait, its wiring into the service, and the
  validation inside `FlagService::set_value` (`service.rs`), where the effective
  rollout is resolved and every sibling write invariant already lives — not in
  `grpc.rs`, which would see only the request's scope and guard only one adapter.
- `backend/server/src/flags.rs`: the adapter, beside the existing plans bridge.
- `apps/back-office/src/components/FlagDrawer.vue` + `stores/flags.ts`: the dangling
  presentation and the refusal path.

**Why this is worth its own change.** It is a platform-socle correctness fix with no
relation to any product feature. It surfaced while specifying the drum rollout
(`add-drums-access`), which is the first change to depend on a `beta:<campaign>`
scope in earnest, but it is not part of it and should not wait for it: every future
beta rollout carries the same footgun.
