## Context

`RolloutScope::Beta(key)` grants its audience by string membership:
`self.staff || self.betas.contains(key)` (`feature-flags/src/context.rs:148`). The
key is validated for shape only — `RolloutScope::parse` accepts `beta:[a-z0-9-]+`
and asks nothing else (`context.rs:41`). The admin RPCs run that same parse
(`grpc.rs:140,164`) and store the result.

The console is not the problem. `FlagDrawer.vue:28` populates the selector from the
open campaigns and its own comment says "never free text"; the `feature-flags-admin`
spec requires exactly that. The problem is that this discipline lives only in one
client, while the write boundary accepts anything shape-valid from anyone.

Two properties make the resulting failure unusually hard to notice:

- **Staff match every beta scope.** An operator checking their own work from an
  admin session gets the feature whether or not the key names a real campaign. The
  only observation that distinguishes the cases requires a non-staff account.
- **A dangling scope is a legitimate-looking stored value.** It is audited like any
  other change, and the console re-displays it as an ordinary selected option
  (`FlagDrawer.vue:33` keeps it selectable so the selector never rewrites it).

`cymbra-feature-flags` and `cymbra-plans` are siblings: both depend on
`cymbra-platform`, neither on the other. That separation is worth keeping.

## Goals / Non-Goals

**Goals:**

- Make "a beta scope names a real campaign" an invariant of the write boundary
  rather than a convention of one client.
- Keep `feature-flags` free of any dependency on `plans`.
- Make an already-dangling stored scope visible as such.

**Non-Goals:**

- Changing how scopes are evaluated, or the audience a `beta:<key>` grants. This
  change is inert for every correctly-configured flag.
- Changing the console's selector, which already meets its spec.
- Validating campaign membership, staleness, or anything beyond existence.
- Retro-fixing dangling scopes automatically. Surfacing one is in scope; deciding
  what it should have been is an operator's judgement, not a migration's.

## Decisions

### A port declared by `feature-flags`, implemented by the composition root

`cymbra-feature-flags` declares a narrow trait ("does a campaign with this key
exist?") and never implements it. `backend/server/src/flags.rs` implements it
against plans, beside the **existing** plans → flags bridge at `flags.rs:461`,
which already resolves a caller's plan and beta keys into the evaluation context.

Today the two crates do not know each other: each lists only `cymbra-platform` in
its `Cargo.toml`, and `backend/server` depends on both. This keeps that true.

*Rationale, in order of weight:*

1. **`plans` already solved this exact problem in the opposite direction, and chose
   this shape.** It needs flag-backed settings (`plans.enabled`, `plans.grace_days`)
   and does **not** import the flags crate: it declares `PlanConfigSource`
   (`plans/src/ports.rs:47`) and the server fills it with `FlagPlanConfig`
   (`server/src/flags.rs:356`). Keeping the two crates unlinked was a deliberate
   choice; linking them in the other direction would contradict it for no gain.
2. **Testing.** With a declared need, testing "a scope naming no campaign is
   refused" plugs in a three-line stub. With a direct dependency it requires standing
   up a plans service and its database — turning a unit test into an integration
   test, for a rule that is pure logic.
3. **It keeps a door open.** Cargo rejects two crates that depend on each other. No
   cycle exists today, because plans routes around it with its own trait. But a
   direct `flags → plans` edge would permanently forbid the reverse, so anyone later
   finding that indirection unnecessary and wanting to simplify it would be stuck.

*Alternative rejected:* `feature-flags` depends on `plans` directly — fewer moving
parts and a more direct read, at the cost of all three points above. The observable
behaviour is identical either way; this is internal wiring, and reversible.

*Alternative rejected:* validate in the console only. That is today's state, and
today's state is what the change exists to fix.

### The check lives in `FlagService::set_value`, on the effective scope

The verification runs inside the service's write path, not in the gRPC handlers.
`set_value` is where every sibling write invariant already lives — declared-only
keys, type match, admin/app scoping, sensitive-key confirmation — and it is the
one place that resolves the rollout actually written: the request's scope, else
the previous override's (preserved), else the registry default
(`service.rs:411-419`).

*Rationale:* a check in `grpc.rs` after `parse_rollout_opt` would guard exactly
one door and see only the request's scope. A `SetFlag` carrying an empty
`rollout_scope`, or a future beta-scoped registry default, resolves its written
scope from sources the handler never sees; and a future non-gRPC adapter would
bypass the handler entirely — a real pattern in this repo, since `cymbra-plans`
already fronts one service through both `grpc.rs` and a bearer-JSON `web.rs`.
Putting the check anywhere but the service would recreate, one layer down, the
"discipline lives in one client" defect this change exists to fix.

### Only a write that sets or changes the scope is checked

The lookup runs when the effective scope about to be stored differs from the
stored one, or none is stored yet. A write that leaves the stored scope
untouched — disabling a flag, editing an unrelated field, a console save that
re-submits the same scope (which the console does on *every* save) — skips it
entirely.

*Rationale:* such a write cannot introduce a dangling scope, so checking it buys
nothing — and refusing it during a campaign-directory outage would cost a lot.
Because the console re-submits the stored scope with every save, a
scope-blind reading of fail-closed would refuse turning a beta-gated feature
**off** mid-incident, precisely the write an operator needs then. An incident
must never be un-mitigatable because an unrelated directory is down. Fail-closed
applies to scope *changes*; everything else stays writable.

### Existence, not openness

The port answers whether the campaign exists at all, including closed ones.

*Rationale:* closing a campaign is the documented mechanism for withdrawing a
feature from its members **without a flag edit** — it is a scenario in the
`runtime-feature-flags` spec. If a closed campaign made its scope invalid, the
stored override would become unwritable, and any later edit to that flag (turning
it off, say) would be refused until the operator changed the scope too. That is the
opposite of the intended ergonomics, at the exact moment the operator is trying to
wind something down.

### The check fails closed, and says so truthfully

If campaign existence cannot be determined — the plans read fails, the port is not
wired — a write that would set or change a beta scope is refused, not stored.

*Rationale:* this inverts the usual "degraded dependency must not break the hot
path" reflex, and deliberately. An override write is a rare, interactive, retryable
operator action, not a request path; refusing it costs a retry. Storing an
unverified scope costs the silent, months-long failure this whole change exists to
prevent. The asymmetry is decisive.

The refusal is its own typed error, distinct from "names no campaign": the port
answers *exists*, *does not exist*, or *cannot determine* — a fallible `Result`,
never an error-swallowing default — and the console maps the last to "retry
later" rather than "fix the scope". The existing `PlanContextSource` bridge is
deliberately **not** the model here: its `plan_context` returns infallibly and
documents that errors are treated as free (`context.rs:74`), the right shape for
evaluation and exactly the wrong one for this port, where swallowing an error
would tell the operator a campaign does not exist when the truth is that nobody
could ask — an invitation to the destructive scope edit the whole change is
meant to prevent.

Note the contrast with evaluation, which stays fail-open in the sense that it never
starts refusing traffic: an unreadable flag store still resolves to code defaults.
Only the *write* is strict — and, per the previous decision, only the write that
sets or changes a scope.

### Dangling and closed are presented differently

The console distinguishes "scope not in the offered list because its campaign is
closed" (legitimate, still gating) from "scope names no campaign at all"
(a defect). Today both fall into the same `staleScope` branch.

*Rationale:* collapsing them trains operators to ignore the marker, since the
common case — a closed campaign — is not a problem.

## Risks / Trade-offs

**A flag whose campaign was deleted cannot be re-scoped blindly** → if a campaign
row is ever removed rather than closed, unrelated edits to its flag — disabling
it, changing its value — still save fine, since a write that leaves the stored
scope untouched skips the check; only **re-scoping** the flag requires naming a
valid campaign (or a non-beta scope). Mitigation: the console's dangling
presentation names the cause, so the one refused action explains itself.
Accepted, and rare: the plans surface offers `CloseCampaign`, not deletion.

**A new port to wire, and a wiring that can be forgotten** → an unwired port under
the fail-closed rule would refuse *every* beta-scoped write, which is a loud,
immediate failure rather than a silent one. That is the intended direction of
breakage, but it must be caught before production. Mitigation: a startup-time
assertion or a test at the composition root that the port is present.

**Existing dangling scopes are not repaired** → the change prevents new ones and
surfaces old ones; it does not fix them. Mitigation: an explicit operator task to
review beta-scoped flags once, after deploying.

## Migration Plan

No schema change, no backfill, no flag. Existing overrides are untouched and keep
evaluating exactly as before — the validation applies to writes only, so a stored
dangling scope keeps behaving as it does today (matching staff only) until an
operator edits it.

Deploy order: the port and its adapter land together; the console change can follow
independently, since a refused write is already surfaced as an error before it is
surfaced as a *good* error.

Rollback is a revert, with nothing persisted to unwind.

## Open Questions

- **Should a one-off audit report existing dangling scopes?** The change surfaces
  them in the console when an operator opens the flag. A list — "these beta-scoped
  flags name no campaign" — would find them without anyone looking. Cheap, but it
  is a new read surface; left out unless the first audit finds any.
- **Should `plans` deletion be prevented outright?** The fail-closed refusal
  assumes campaigns are closed, not deleted. If deletion exists anywhere (a manual
  SQL path, a future admin action), guarding it would be more robust than handling
  the aftermath.
