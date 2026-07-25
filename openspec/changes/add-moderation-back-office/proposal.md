## Why

Change #1 makes every catalog score `pending` until validated and adds a privileged,
back-office-only status filter — but there is **no tool to validate anything** and no
`moderator` role to do it with. This change delivers the **moderation back office**
(`bo.cymbra.app`): a web app where authorized moderators/admins review scores and
accept or reject them, plus the role model and the evaluate/role-grant operations it
needs. Without it, #1's migration leaves the hub empty with no way to fill it.

## What Changes

- **Moderator role (music scope)** — add a `moderator` role in the existing scoped RBAC
  (`user_roles`, scope `music`). Back-office access = `music/admin` **or**
  `music/moderator` (with `global/admin` as break-glass). No cross-app transversality: a
  role in one module never grants another module's powers.
- **Evaluate action** — an authorized moderator/admin sets a score's moderation status
  to `accepted` or `rejected`, recording **who** and **when** (the `reviewed_by` /
  `reviewed_at` audit columns from #1), so rejections are traceable.
- **Back-office console (Vue 3 SPA)** — a client-rendered single-page app (Vite, no
  SSR) at `bo.cymbra.app` showing **all scores in a simple table** with the **same
  filters as the app hub** plus a **back-office-only status filter** (never exposed in
  the Flutter app). Clicking a row opens the score, previews it read-only, and offers
  accept/reject. The re-review queue from #2 is surfaced here.
- **Role administration** — an admin grants/revokes `moderator` (and `admin`) within the
  `music` scope from the back office; guarded by `require_admin`. The **first admin is
  seeded out-of-band** (ops CLI/DB), matching how admin is granted today.
- **Browser transport** — expose a browser-reachable **gRPC-web + CORS** surface for the
  back office. This keeps the "no REST" invariant (gRPC-web is still gRPC); the Flutter
  app keeps using native gRPC unchanged.

Out of scope: the app-side rating UI (#2, though its `needs_review` flag is consumed
here); analytics dashboards; bulk moderation actions (single-score accept/reject first);
a full moderation event-history table (the single reviewer+timestamp from #1 suffices —
revisit if history is needed).

## Capabilities

### New Capabilities
- `moderation-access-control`: The authorization model for moderation — the `music`-scoped
  `moderator` role, the rule that moderators/admins may read and evaluate non-`accepted`
  scores, admin-only role granting/revocation within a scope, and first-admin bootstrapping.
- `moderation-console`: The back-office review tool — the evaluate (accept/reject)
  operation with reviewer audit, the tabular catalog view with the app's filters plus a
  privileged status filter and the re-review queue, the read-only score preview, and the
  Vue SPA delivery.

### Modified Capabilities
- `backend-service`: the gRPC foundation additionally exposes a browser-reachable
  gRPC-web endpoint with CORS restricted to the back-office origin, without introducing a
  REST API.

## Impact

- **Backend**: new `moderator` role value + `effective_roles` inclusion; a `SetModerationStatus`
  RPC (writes status + `reviewed_by`/`reviewed_at`); a `GrantRole`/`RevokeRole` RPC guarded
  by `require_admin`; widen #1's privileged status-filter + non-`accepted` fetch-bytes guard
  from admin-only to **admin-or-moderator**; `tonic-web` + `tower-http` CORS layer; a new
  back-office OIDC audience/client and `CYMBRA_ALLOWED_AUDIENCES` entry; an ops seed for the
  first `music/admin` (`backend/scripts/` + provision).
- **New web app**: a Vue 3 + Vite SPA (new package/repo `bo.cymbra.app`) using a gRPC-web
  client + the Cymbra OIDC sign-in; deployed like the marketing site (Cloudflare Pages or
  equivalent). To be challenged vs. plain Vue during design.
- **DB**: no new score columns (reuses #1's `moderation_status` + audit); possibly a
  `user_roles` grant audit later (out of scope now).
- **Depends on #1** (status + audit columns, privileged filter). Consumes #2's
  `needs_review` flag if present; degrades gracefully if #2 not yet shipped.
- **Coverage**: Rust ≥ 80% for the evaluate/role/guard logic; the Vue app carries its own
  lint/test setup (outside the Flutter/Rust gates).
