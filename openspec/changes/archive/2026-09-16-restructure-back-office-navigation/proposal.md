## Why

The back-office sidebar is one flat list that mixes three products — Music moderation,
Lingua, and cross-product administration — in the order the pages happened to ship, so an
operator cannot tell what a page belongs to. It also names things misleadingly ("Takedowns"
reads as "delete a catalog score" when it removes a user's *private* score), and three
entries are shown to admins the server then refuses: *Sound fonts*, *Campaigns* and
*Usage* appear for any admin, yet their RPCs require `admin` in the `music` scope, so a
`lingua`-only admin gets three links that fail. URLs follow no rule either: two Music pages
live under `/music/`, four others at the root.

## What Changes

- The sidebar is grouped by product under three headed sections — **Music**, **Lingua**,
  **Administration** — and a section with nothing the operator can open is not shown.
- An entry is shown **exactly when its page would open**: the navigation and the route guard
  read the same per-page access rule (role + scope), so a link can no longer lead to a
  refusal. This fixes *Sound fonts*, *Campaigns* and *Usage* (now `music`-admin only)
  and scopes the Music moderation pages to the `music` scope.
- Every page path starts with its section: `/music/…`, `/lingua/…`, `/admin/…`.
  **BREAKING (URLs only)**: `/takedowns`, `/soundfonts`, `/campaigns`, `/usage`, `/lingua`,
  `/users`, `/users/{id}`, `/flags`, `/notifications` and `/jobs` move; every former path —
  and the older `/roles` and `/plans` — redirects to the new one, keeping its id and query,
  so bookmarks and shared links keep working.
- The console lands an operator on the first page they can open (instead of always the Music
  review queue), and a signed-in account with no page to open gets the access-denied state.
- "Takedowns" becomes **Private scores** (page title says it is a takedown-on-notice tool for
  scores that are never in the catalog); the lone Lingua entry becomes **Overview** under the
  Lingua heading.
- An account's detail page offers music-scope admins a link to **that account's private
  scores** (the lookup opens already filtered on the owner), and each owner id in the lookup
  results links back to the account page. The lookup criteria ride in the URL.

Products impacted:

- **Back office** — new: grouped navigation, shared access rule, path prefixes, redirects,
  landing, the account ↔ private scores links. Consumed unchanged: every existing admin RPC
  and its server-side gate (still authoritative).
- **Music / ID / Live / site / backend** — none. No proto, no RPC, no database change.

## Capabilities

### New Capabilities

- `admin-console-navigation`: how the back-office navigation is organised — product
  sections, entry visibility tied to page access, path prefixes, former-path redirects, and
  the landing page.

### Modified Capabilities

- `admin-account-directory`: the Users page and the account detail page move to
  `/admin/users` and `/admin/users/{user_id}`; former paths redirect.
- `admin-plan-console`: the Campaigns page moves to `/music/campaigns`; the account plan
  sections follow the account page to `/admin/users/{user_id}`.
- `admin-jobs-console`: the Jobs page moves to `/admin/jobs`.
- `admin-lingua-console`: the Lingua page moves to `/lingua/overview` and is the Lingua
  section's *Overview* entry; the flags console it relies on is at `/admin/flags`.
- `music-user-score-takedown`: the takedown surface is the *Private scores* page at
  `/music/private-scores`; it is reachable from an account's detail page, pre-filtered on
  that account, and its results link back to the owner's account.

## Impact

- `apps/back-office/src/router.ts` (route table, access meta, redirects, guard, landing),
  `src/App.vue` (grouped sidebar), a new `src/lib/navigation.ts` holding the navigation
  model and the access rule shared by guard and sidebar.
- Every in-app link that names a moved route (`RouterLink`/`router.push` by name), notably
  `UserDetailView`, `UsersView`, `CampaignsView`, `TakedownsView`, `AccessDeniedView`,
  `SignInView`.
- `src/i18n/locales/{en,fr}.json`: section headings, renamed entries, private-scores page
  title/intro, access-denied copy.
- Tests: unit (`test/`) for the access rule, navigation model and redirects; Playwright
  (`e2e/`) paths and sidebar assertions updated.
- No backend, proto, database or deployment change. Cloudflare Pages already serves
  `index.html` for every path (`public/_redirects`), so the new paths need no hosting change.
