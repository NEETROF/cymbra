## Context

- **Rotation today** (`openspec/specs/backend-auth`, « Token model »): a refresh token is rotated on use, atomically (`UPDATE sessions SET current_rt_hash = … WHERE id = $1 AND current_rt_hash = $2 AND expires_at > now()`). If no row matches and the family is still live, the presented token is a replay of a rotated one → the family is revoked (`backend/auth/src/session_pg.rs`, `session.rs`).
- **Why that is right:** a refresh token is a bearer credential with a 30-day sliding life. Rotation plus reuse detection is what turns a stolen token into a dead one, and it is the reason a theft cannot outlive the victim's next refresh.
- **Why it misfires:** the rotation's result reaches the client over the network and is then written to storage. A client killed in between keeps the old token and, in good faith, replays it. The server cannot tell that apart from theft.
- **Who is killed most:** a Safari web extension's background page, suspended whenever it is idle — and, before this change's client half, refreshing on every wake that lost its cached access token.

## Goals / Non-Goals

**Goals:**
- An honest client interrupted mid-rotation keeps its session, with no re-authentication and nothing for the reader to do.
- A stolen token is still detected and still kills the family.
- Every client benefits without changing; the extension additionally stops refreshing when it does not need to.

**Non-Goals:**
- Device-bound or DPoP-style proof-of-possession tokens: a much larger change, and not what these sign-outs are about.
- Changing the access token's 15-minute life or the refresh token's 30-day sliding life.
- Making the extension's background page survive suspension, which is not ours to decide.

## Decisions

### D1 — The server carries the tolerance, not the client

A client cannot fix this on its own: whatever it writes, and whenever, it can be killed before the write lands. It can only make the window rarer (D4). The server is the only place that can tell "the token I just replaced" from "a token replaced long ago", so the tolerance belongs there — and then every client gets it, including the ones we are not changing.

### D2 — Remember the replaced token, for a minute

`sessions` gains `prev_rt_hash` and `prev_replaced_at`, both set by every rotation.

`rotate(token)` resolves in this order:
1. **The current token** → rotate as today: the new hash becomes current, the old one becomes `prev_rt_hash` with `prev_replaced_at = now()`, the expiry slides, and the new pair is returned.
2. **The previous token, replaced less than the grace ago** → the client never received the answer. Rotate again from the current hash: the caller gets a usable pair, `prev_rt_hash` becomes the token that was current, and `prev_replaced_at` is refreshed. Nothing is revoked.
3. **Anything else on a live family** — the previous token past the grace, a token from further back, or one already superseded twice — → reuse detection, unchanged: the family is revoked and the call fails `UNAUTHENTICATED`.

Both paths stay a single conditional `UPDATE … RETURNING`, so two concurrent callers still cannot both take the same branch: the check-and-rotate remains atomic.

**Consequence for concurrent refreshes.** Two refreshes of the same token no longer end with a revoked family: the loser lands in case 2 and gets its own pair. That is the correct outcome — they are the same client racing itself, which is exactly what an interrupted retry looks like — and it is why the spec's concurrency scenario changes.

### D3 — One minute, configurable, and what it costs

`CYMBRA_REFRESH_REUSE_GRACE`, default **60s**.

The window is what an interrupted client needs: it wakes, finds the old token, and retries within seconds. It is not a session-length knob and should stay in the tens of seconds.

**What it costs, stated plainly:** a refresh token stolen and replayed *within* the grace of its rotation is honoured, and the thief obtains a live pair. Before this change that replay revoked the family and both sides lost the session. So the change trades "a thief racing the victim inside a one-minute window is caught" for "an honest client killed mid-rotation is not thrown out". Everything else is unchanged: a theft that surfaces later — the normal case, since a stolen token is used on the thief's own schedule — still kills the family, and `Logout`, password reset and admin revocation still cut sessions immediately.

**Alternatives rejected.**
- *Return the same pair for a replay, rather than rotating again:* it would need the token itself in the store, and only hashes are stored. Keeping the token would make the database a credential store, which is precisely what hashing avoids.
- *No rotation at all:* a stolen token would then live for its whole 30-day sliding life, undetected.
- *An idempotency key supplied by the client:* every client would have to change, and a killed client has no more guarantee of having persisted the key than the token.

### D4 — The extension refreshes when the token is expired, not when the page wakes

Two changes in `apps/lingua-extension`:
- The access token is kept with its expiry in the area that survives a suspended background page, instead of the session-scoped one a wake often finds empty.
- A refresh happens when the access token is absent or within a few seconds of expiry, instead of on every wake that lost it.

**On keeping the access token on disk:** it is a 15-minute bearer, and it now sits beside the 30-day refresh token that already lives there. An attacker who can read the extension's storage already holds the more valuable of the two, so the exposure added is bounded and small — while the sign-outs it removes are constant. Web pages cannot read either: extension storage is not page storage.

### D5 — Rollout

1. **Backend first.** The grace is inert for a client that never replays, so deploying it early changes nothing and starts protecting every app.
2. **Then the extension build**, which reduces how often rotation happens at all.

The migration adds two nullable columns; sessions created before it simply have no previous token, so their first rotation fills them in.

## Risks / Trade-offs

- **[A thief racing inside the window]** → Accepted and bounded (D3): one minute, the immediately previous token only.
- **[The grace hides a genuinely broken client that refreshes in a loop]** → Bounded: each grace rotation moves the chain forward, so a client stuck on a token older than the previous one is still cut. The rotation count per session stays observable in the jobs/metrics the auth module already exposes.
- **[Two columns on a hot table]** → Two fixed-size columns written by the same `UPDATE` that already writes `current_rt_hash`; no extra statement, no extra index.
- **[The access token on disk]** → D4: bounded by its 15-minute life, next to a credential that is already there.

## Migration Plan

1. Migration: `ALTER TABLE sessions ADD COLUMN prev_rt_hash TEXT, ADD COLUMN prev_replaced_at TIMESTAMPTZ` (both nullable, no backfill).
2. Deploy the backend; the grace applies from the first rotation after it.
3. Ship the extension build.
4. **Rollback:** revert the module; the columns can stay — an older binary ignores them.

## Open Questions

None.
