## 1. Access rule and navigation model

- [ ] 1.1 Add `src/lib/navigation.ts`: the `Access` type (`role`, optional `scope`), a pure `canOpen(access, rolesByScope)` built on `hasRoleInScope` (global counts everywhere; missing access = not openable)
- [ ] 1.2 In the same module, declare the sidebar model: the three sections (id, heading i18n key) and their ordered entries (route name, label key, icon), plus `landing(router, rolesByScope)` returning the first openable entry or `denied`
- [ ] 1.3 Unit-test `canOpen` for moderator/admin rules, scoped and unscoped, global break-glass, and a missing rule; test `landing` for a music moderator, a music admin, a lingua-only admin, a global admin and a live-only moderator

## 2. Router

- [ ] 2.1 Rewrite the route table with the prefixed paths and `<section>-<page>` names, each with `meta.access` per the design table (`/music/*`, `/lingua/overview`, `/admin/*`)
- [ ] 2.2 Add the former-path redirects (`/takedowns`, `/soundfonts`, `/campaigns`, `/plans`, `/usage`, `/lingua`, `/users`, `/roles`, `/users/:userId`, `/flags`, `/notifications`, `/jobs`) forwarding params and query, and the section-root redirects (`/music`, `/lingua`, `/admin`)
- [ ] 2.3 Rewrite the guard on `canOpen`: refused or unknown pages and `/` go to `landing`; an already-signed-in operator on `/signin` goes to `landing`; no loop
- [ ] 2.4 Unit-test the router: every non-public, non-redirect route declares `access`; every sidebar entry resolves to a route; each former path redirects with params and query; a former path does not bypass access

## 3. Sidebar

- [ ] 3.1 Render the sidebar in `App.vue` from the model, filtered by `canOpen`, as headed `role="group"` sections with spacing (no divider), empty sections hidden
- [ ] 3.2 Add the section heading keys and the renamed entries (Private scores, Overview) in `en.json` and `fr.json`; keep the locales aligned
- [ ] 3.3 Test the sidebar per profile (global admin, music admin, music moderator, lingua-only admin): sections, entries and order

## 4. Moved pages and links

- [ ] 4.1 Update every navigation by route name or path in views and components (`UserDetailView`, `UsersView`, `CampaignsView`, `ScoreDetailView`, `QueueView`, `ReviewView`, `CatalogView`, `SignInView`, `AccessDeniedView`, …) to the new names; leave backend HTTP paths (`/soundfonts/{id}` on the API host) untouched
- [ ] 4.2 Send the post-sign-in redirect to `landing` instead of the review queue
- [ ] 4.3 Replace the access-denied copy (en + fr) with the "no page in this console, ask an administrator" message

## 5. Private scores

- [ ] 5.1 Rename `TakedownsView` to `PrivateScoresView`, retitle it (Private scores, takedown on notice, never in the catalog) in en + fr
- [ ] 5.2 Make the lookup addressable: read `owner`/`title` from the query on arrival and on change and run the store search; submitting the form replaces the query
- [ ] 5.3 Link each result's owner id to `admin-user-detail`
- [ ] 5.4 On the account detail page, show music-scope admins a "Private scores" link to `music-private-scores` with `owner` set to the account id
- [ ] 5.5 Tests: query-driven search, form → query, owner link, account link shown only with the `music` scope

## 6. End-to-end

- [ ] 6.1 Update every Playwright spec to the new paths and names
- [ ] 6.2 Add e2e coverage: grouped sidebar for a global admin, a lingua-only admin without Music entries, a former path with a query redirected, the account → private scores → account round trip

## 7. Verification

- [ ] 7.1 `yarn lint`, `yarn typecheck`, `yarn format:check` in `apps/back-office` (Prettier gate through `rtk proxy`)
- [ ] 7.2 `yarn test` (coverage ≥ 80 %) and `yarn e2e` green
- [ ] 7.3 Visual check on the dev server: sidebar screenshots for a global admin and a music moderator; old bookmarks redirect
- [ ] 7.4 Manual check on `bo.cymbra.app` after deploy: sections match the account's scopes, old links redirect, account ↔ private scores links work
