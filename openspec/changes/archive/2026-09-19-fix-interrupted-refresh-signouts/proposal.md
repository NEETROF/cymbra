## Why

Dogfooding Cymbra Lingua on Safari (TestFlight 70/71): the reader is signed out several times a day, on both the iPhone and the Mac. The background console shows the sequence:

```
[Cymbra Lingua] sync failed: ConnectError: [unauthenticated] token rejected: expired
```

— the access token expired, the refresh that should have renewed it was refused, and the session was purged.

The cause is a race between rotation and an interrupted client, not a bug in either side:

1. The refresh token is **rotated on use**: the server issues a new one and kills the old one in the same transaction.
2. Safari suspends an extension's background page at any moment — including **between the server's answer and the client's write** of the new token.
3. The device wakes with the token it already had, presents it in good faith, and the server sees a replay of a rotated token: **theft**, so it revokes the whole session family.

Nothing distinguishes "my answer never arrived" from "someone stole this token", so an honest client is treated as an attacker. The extension makes it frequent — its background is killed constantly, and every wake without a cached access token refreshes — but the window exists for every client: Music, the site and the back office are all one badly-timed kill away from the same sign-out.

## What Changes

- **A short grace on the replaced token.** The session remembers the token it just replaced and when. A token presented within the grace (default 60 s) of being replaced is **not** treated as theft: the family is rotated again and the caller gets a usable pair, which is what the interrupted client was owed. Outside the window — or for anything older than the immediately previous token — reuse detection is unchanged and still revokes the family.
- **The extension refreshes only when it must.** The access token and its expiry are kept where a suspended background page finds them again, and a refresh happens when the access token is actually expired, not on every wake. Fewer rotations mean fewer chances to be interrupted mid-rotation.

## Capabilities

### Modified Capabilities
- `backend-auth`: refresh-token rotation and reuse detection gain the grace window, and the concurrent-refresh scenario follows from it.

The extension's half (refresh when the token is expired, not when the page wakes) moves no contract: it is when a client calls `Refresh`, which the capability already allows. It is tracked in the tasks.

## Impact

- **Backend:** `backend/auth` session store (Postgres + the in-memory fake), `rotate`, a migration adding the previous-token hash and its timestamp, one configuration value (`CYMBRA_REFRESH_REUSE_GRACE`, default 60s).
- **Clients:** every app benefits without changing; only the Lingua extension changes, to refresh less often.
- **Security:** the trade-off is stated in the design — a stolen refresh token replayed **within** the grace window is honoured. It is bounded to one minute, applies only to the immediately previous token, and is the standard leeway for clients that can be killed mid-call.
- **Rollout:** backend first (the grace is inert for clients that never replay), then the extension build.
- **Out of scope:** binding sessions to a device, and shortening the refresh token's 30-day sliding life — both are separate questions.
