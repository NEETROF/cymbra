## Context

The back office (`apps/back-office`, Vue 3 + vue-router) decides what an operator sees in
two places that were written separately:

- `App.vue` builds the sidebar from `auth.isAdmin` plus three ad-hoc `adminScopes.includes()`
  checks, appended in shipping order.
- `router.ts` guards routes with `meta: { admin, adminScope }`, and falls back to
  `music-catalog` on refusal; `/` and unknown paths land on `music-queue`.

The two drifted. *Instrument sounds* (`/soundfonts`), *Campaigns* (`/campaigns`) and *Usage*
(`/usage`) carry only `meta.admin`, so any admin sees them and opens them, while their RPCs
check `admin` in `music` (`soundfont_http.rs`, `plans/src/grpc.rs`, `analytics/src/grpc.rs`).
The Music moderation pages carry no scope at all, so a `live` moderator or a `lingua` admin
lands on a review queue whose every call is refused. The shell itself is shown to anyone
holding `moderator` or `admin` in any scope (`isModerator`, flat) — that stays.

Scope semantics already exist client-side: `hasRoleInScope(rolesByScope, scope, role)` in
`src/lib/jwt.ts` mirrors the backend (a `global` role counts in every scope), and
`auth.adminScopes` is built on it. The server remains the only authority; everything here is
UX.

## Goals / Non-Goals

**Goals:**

- One place declares, per page, who may open it; the guard and the sidebar both read it.
- The sidebar is grouped into Music / Lingua / Administration, empty groups hidden.
- Paths are prefixed by group; every former path still resolves.
- An operator lands on a page they can open, whatever their scopes.
- The private-score takedown page is named for what it is and reachable from an account.

**Non-Goals:**

- Any server-side authorization change (the RPC gates are already right).
- A per-module view of the Jobs, Flags, Notifications or Users pages for scoped admins
  (they keep their current gates: Jobs `global` admin; the other three any admin).
- Collapsible groups, icons per group, or a new visual design beyond the section headings.
- Looking a private score up by owner handle (the lookup still takes an owner id or a title
  fragment; the account link removes the need to copy an id).

## Decisions

### D1 — Access is route metadata, and the sidebar is derived from the router

Each route carries `meta.access = { role: "moderator" | "admin", scope?: Scope }`:

- `role: "moderator", scope` → moderator **or** admin in `scope` (global counts);
- `role: "admin", scope` → admin in `scope` (global counts);
- `role: "admin"` without scope → admin in any scope (today's `meta.admin`).

A pure `canOpen(access, rolesByScope)` in a new `src/lib/access.ts` implements it on top of
`hasRoleInScope`. The guard calls it; the sidebar model lists route **names** per group and
keeps an entry only if `canOpen(router.resolve(name).meta.access, …)`. So the sidebar cannot
show a page the guard refuses — the drift above becomes unrepresentable rather than merely
fixed.

*Alternative rejected:* keep `meta.admin/adminScope` and duplicate the checks in `App.vue`
with the right scopes. That is today's design; it already drifted three times.

Access per page:

| Group | Page | Path | Access |
|---|---|---|---|
| Music | Review queue | `/music/queue` | moderator · music |
| Music | Catalog | `/music/catalog` | moderator · music |
| Music | *(review, score detail — not in the sidebar)* | `/music/review`, `/music/score/:id` | moderator · music |
| Music | Private scores | `/music/private-scores` | admin · music |
| Music | Instrument sounds | `/music/soundfonts` | admin · music |
| Music | Campaigns | `/music/campaigns` | admin · music |
| Music | Usage | `/music/usage` | admin · music |
| Lingua | Overview | `/lingua/overview` | admin · lingua |
| Administration | Users (+ `/admin/users/:userId`) | `/admin/users` | admin · any |
| Administration | Feature flags | `/admin/flags` | admin · any |
| Administration | Notifications | `/admin/notifications` | admin · any |
| Administration | Jobs | `/admin/jobs` | admin · global |

*Usage* is Music's: the analytics events carry no product dimension and come from the Music
app, and the RPCs are `music`-admin gated.

### D2 — Route names follow the prefix

Route names become `<group>-<page>` (`music-private-scores`, `lingua-overview`,
`admin-users`, `admin-user-detail`, …), matching the existing `music-*` convention the router
comment already states. Every in-app navigation is by name, so the rename is mechanical and
the type-checker plus the e2e suite catch a missed one.

### D3 — Former paths redirect, keeping params and query

A redirect record per former path: `/takedowns → music-private-scores`,
`/soundfonts → music-soundfonts`, `/campaigns` and `/plans → music-campaigns`,
`/usage → music-usage`, `/lingua → lingua-overview`, `/users` and `/roles → admin-users`,
`/users/:userId → admin-user-detail` (param carried), `/flags`, `/notifications`,
`/jobs → admin-*`. Redirects are function redirects that forward `to.query` (the user page
uses `?tab=`, the private-scores page gets `?owner=`). Group roots `/music`, `/lingua`,
`/admin` redirect to the group's first page; the guard then re-routes if that page is not
the operator's.

These redirects are permanent: they cost a few lines and protect every bookmark and pasted
link. Removing them later is a separate decision.

### D4 — Landing is the first page the operator can open

`landing(rolesByScope)` walks the sidebar model in order and returns the first page
`canOpen` admits, or the `denied` route when there is none. It serves `/`, unknown paths, a
refused page, and the post-sign-in redirect (today hard-coded to `music-queue` in the guard
and in `SignInView`). A music moderator still lands on the review queue; a `lingua`-only
admin lands on the Lingua overview instead of a refused queue. Because `landing` only ever
returns an admitted page or the public `denied` route, the guard cannot loop.

The access-denied copy stops saying "not a moderator or admin": a `live` moderator reaches
it too, so it says the account has no page in this console and to ask an admin.

### D5 — Section headings, not dividers

Each group renders as a `role="group"` element labelled by its heading (small, muted,
monospace, like the brand subtitle), with vertical spacing between groups and no rule line.
The heading already separates *and* names the group, and it is what a screen reader
announces; a divider would be a second signal for the same boundary, competing with the
sidebar edge and the active-entry highlight. Chosen by the product owner over "heading +
rule" and "rule only" from a mock-up.

### D6 — The private-scores lookup is addressable

`TakedownsView` (renamed `PrivateScoresView`) reads `owner` and `title` from the route query:
on arrival and on query change it fills the form and calls the store's `search`. Submitting
the form replaces the query instead of searching directly, so the URL is the single source
of the criteria. The account detail page links to
`{ name: "music-private-scores", query: { owner: userId } }`, shown only to music-scope
admins (the same `canOpen` rule). Owner ids in the results become links to
`admin-user-detail`. The store, its `Async` unions and the confirmation dialog are unchanged.

## Risks / Trade-offs

- [A link outside the console (docs, chat, a ticket) uses an old path] → D3 redirects every
  former path, params and query included.
- [A route rename is missed in a `RouterLink`] → names are string literals, so the type
  checker does not see them; the e2e suite exercises every page by its new path, and a unit
  test asserts every sidebar route name resolves.
- [Tightening the Music moderation pages to the `music` scope hides them from a `live`
  moderator] → intended: every call on those pages is already refused for them; they now get
  the access-denied state instead of a page of errors.
- [A future page is added without `meta.access`] → the guard treats a non-public route
  without `access` as **not openable** (fail closed), and a unit test asserts every
  non-public, non-redirect route declares it, so the omission fails CI instead of silently
  opening or hiding a page.

## Migration Plan

Front-end only, shipped with a normal back-office release (`back-office-deploy`). No backend
or data step. Rollback is redeploying the previous back-office version; links created under
the new paths would then 404 into the SPA fallback and land on the queue, which is
acceptable for a rollback window.

## Open Questions

None.
