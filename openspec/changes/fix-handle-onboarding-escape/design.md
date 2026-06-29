## Context

`add-music-account-access` routes every post-auth user with a null handle to a
blocking `HandleOnboardingScreen` (decision D4: uniform onboarding). The screen uses
`AuthScaffold`, which provides no app-bar actions; the sign-out/delete menu lives only
on `LibraryScreen`. The account is created at `SignInOidc` time via
`resolve_or_provision` (keyed by `(provider, subject)`), before any handle. Net effect:
a taken handle traps the user, and abandoning leaves a handle-less orphan that the same
user can never reclaim (it's keyed by subject, not email).

## Goals / Non-Goals

**Goals:**
- Guarantee the handle gate is always escapable.
- Avoid leaving orphan handle-less accounts (client cleanup + backend safety net).

**Non-Goals:**
- The "sign in to an existing account and link this identity" flow (lives in
  `add-account-identity-linking`).
- Revealing which sign-in method an existing account uses (needs email persistence +
  lookup; out of scope).

## Decisions

### D1 — Escape action on the handle screen, reusing `SessionNotifier.signOut`
Add a low-emphasis action ("Use a different account") to the handle screen. For an
existing user (handle already set) it calls the existing `signOut`. The screen is the
only place that currently lacks an exit, so the fix is localized.

### D2 — Delete-on-abandon for brand-new accounts via existing `DeleteAccount`
When the abandoning account has no handle, the app calls `DeleteAccount` instead of a
plain sign-out, removing the orphan. OIDC delete re-auth reuses the fresh `id_token`
already obtained this session (no second provider round-trip); for a just-created local
account the password entered moments ago is reused. *Alternative:* leave cleanup to the
reaper only — rejected; immediate deletion keeps the common path clean and the DB tidy.

### D3 — Backend reaper as the safety net for hard kills
Client cleanup can't run if the app is force-quit on the handle screen, so a backend
maintenance task purges accounts with a null handle older than a configurable grace
period (e.g. 24h). The selection predicate (null handle AND `created_at < now - grace`)
is kept in a pure, host-testable function; the scheduler/DB glue is thin. *Alternative:*
a DB TTL/partial index job — rejected; keeping the predicate in Rust keeps it testable
and within the existing module boundaries.

## Risks / Trade-offs

- **Delete-on-abandon needs re-auth** → DeleteAccount requires re-auth; mitigated by
  reusing the in-hand OIDC token / just-entered password so abandon stays one tap.
- **Reaper deleting an account a user still wants** → Only null-handle accounts past the
  grace period are eligible; any account with a handle is never touched. Grace period is
  configurable to tune aggressiveness.
- **Race between client delete and reaper** → Both are idempotent deletes of the same
  row; a not-found on the second is benign.

## Migration Plan

Additive. App change ships independently; the reaper is a new maintenance task gated by
config (grace period). No schema change beyond what already exists (`created_at`,
`handle`). Rollback = revert app code / disable the task.
