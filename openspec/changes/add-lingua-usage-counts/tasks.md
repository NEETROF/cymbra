# Tasks

Size: about **13.5 ideal days** — server 4, extension 4, site 1.5, back office 1.5, disclosures and
store texts 1.5, integration 1. The owner's tasks (group 7) are not counted.

## 0. Decisions

- [ ] 0.1 The owner answers design.md's open questions Q1–Q9, or accepts the recommended answers. Verify: the answers are recorded in design.md, and any change to the windows (Q3), the leaving answers (Q5) or the listing sentence (Q6) is carried into the specs before group 2 starts.

## 1. The promise first (≈ 1.5 d)

- [ ] 1.1 Add the anonymous counts and the leaving page to Annex B of `apps/site/src/pages/confidentialite.md`, `apps/site/src/pages/en/privacy.md` and `apps/site/src/pages/es/privacidad.md`. Cover:
  - the six steps and the two dimensions;
  - no identifier, even for a signed-in reader; nothing about what is read;
  - no network address kept; the day of installation kept on the device for at most 60 days;
  - the leaving page's single answer;
  - on by default, the switch in Settings → Data, Firefox's own consent;
  - legitimate interest as the legal basis, the switch as the way to object.

  Add the Chrome Web Store Limited Use statement to the general part of each, and move `updated`. Verify: `yarn build` in `apps/site`, and the rendered pages read against the `lingua-privacy` delta.
- [ ] 1.2 Replace the "no network request without an account or extended translation" sentence in every store description, in French, English and Spanish. It appears in `apps/lingua-extension/STORE-LISTING.md` (Chrome Web Store and AMO) and in `apps/lingua-apple/STORE-LISTING.md`. Use the Q6 wording. Update the claims table's "Local analysis; no request without an account or translation" row. Verify: `grep -n -i "aucune requête\|no network request\|ninguna solicitud"` over both files returns only the new sentences.
- [ ] 1.3 Rewrite the *Data usage disclosures (Chrome Web Store)* section of `apps/lingua-extension/STORE-LISTING.md` with the counts and the Q9 answer ("User activity"). Verify: the section names the six steps, says "no identifier", and keeps the three certifications.
- [ ] 1.4 Name the count reports and the leaving page in `apps/lingua-extension/REVIEWERS.md` ("Where the add-on reaches the network"), and rewrite the `data_collection_permissions` comment in `apps/lingua-extension/tool/manifests.mjs` so `technicalAndInteraction` names the counts (Q7). `optional` stays as it is. Verify: `yarn check:variants`.
- [ ] 1.5 Rewrite "External services: none by default … no analytics" in each App Review notes block of `apps/lingua-apple/STORE-LISTING.md`, naming the counts and the switch. Trim each block back under 4000 characters, recounting the stated totals. Verify: each block's character count, measured, is ≤ 4000 and written in its heading.
- [ ] 1.6 Review the App Store privacy answers in `apps/lingua-apple/README.md` against the counts. Expected outcome: Usage Data → Product Interaction is already declared, linked, with Analytics; no new type or purpose. Record the date and the outcome like the « Traduction en ligne » review. Verify: the README states the review.

## 2. The server (≈ 4 d)

- [ ] 2.1 Add `backend/lingua/proto/usage_counts.proto`: `UsageCountsService.Report`, whose request carries `install_day` (int32), a `BrowserFamily` enum, `version` (string) and `repeated UsageStep steps`, and whose response is empty. Add `AdminGetUsageFunnel` and `AdminGetLeaveReasons` to `lingua_admin.proto`. List the new file in `backend/lingua/build.rs`, and add both protos to `backend/lingua/tests/privacy_allow_list.rs`. Verify: `cargo test -p cymbra-lingua --test privacy_allow_list`, `buf lint`, and `buf breaking` against `main` (additions only).
- [ ] 2.2 Write `usage_counts_core.rs`: the six steps, the three browser families (and `unknown` for leaving reasons), the version pattern, the six reasons, the day-0 window (−120/+1 days around the server's UTC day), and one to six distinct steps per report. Verify: unit tests for each bound — valid, malformed (invalid) and out of range (acknowledged, not counted).
- [ ] 2.3 Write `usage_counts.rs` with the ports this module declares: `UsageCountsRepo` (increment under a ceiling, read the funnel and the reasons) and `UsageCountsPolicy` (the flag and the ceiling). Add the in-memory `FloodGuard`: an HMAC of the address under a key drawn at start-up, 60 an hour, entries forgotten when their hour ends. Verify: mockall tests for counted, flag off (acknowledged, repo untouched), out-of-range day 0, ceiling reached, the 61st request refused, and an entry forgotten after its hour (an injected clock).
- [ ] 2.4 Add migration `backend/lingua/migrations/0008_lingua_usage_counts.sql` (`lingua.usage_counts_daily`, `lingua.leave_reasons_daily`, `IF NOT EXISTS`, schema-qualified) and `pg_usage_counts.rs`: an upsert with `count = count + 1 WHERE count < $ceiling`, and the window reads. Verify: a schema check that neither table has a column for an address, a header, a time of day or an account; and a run against a local database (`docker compose up`).
- [ ] 2.5 Write `usage_counts_grpc.rs`. It reads the message fields and nothing else, and resolves the address with `cymbra_platform::client_addr::resolve` for the guard alone. Mount it in `backend/server/src/main.rs` inside the lingua block, **without** `strict` or `optional`, only when `CYMBRA_LINGUA_DATABASE_URL` is set. Verify: a test that a request carrying a bearer resolves no identity, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 2.6 Declare `lingua.usage_counts.enabled` (default true) and `lingua.usage_counts.ceiling` (default 5000) in `backend/feature-flags/src/registry.rs` (app `lingua`), with the policy adapter in `backend/server/src/flags.rs`. Verify: the flags console lists both, and an adapter test reads each.
- [ ] 2.7 Add `backend/server/src/web_lingua.rs`: `POST /web/lingua/leave` `{ reason, browser, version }`, with CORS limited to `CYMBRA_WEB_ORIGINS` and no credentials, delegating to the module, merged into the HTTP router only when the lingua module is wired. Verify: route tests for a counted reason, free text (400), a disallowed origin, flag off, and the 61st request (429).
- [ ] 2.8 Prove nothing identifying is kept: capture tracing output across a counted, a dropped and a refused report and a leaving reason, with `x-forwarded-for`, `Origin: moz-extension://…` and `User-Agent` set. Assert none of their values appears. Add the comment to `backend/deploy/Caddyfile` that explains why it has no `log` directive. Verify: the test, and `cargo llvm-cov --workspace --fail-under-lines 80` (with `/pg_usage_counts\.rs` and `/usage_counts_grpc\.rs` added to `.github/coverage-ignore-regex.txt` as thin glue).
- [ ] 2.9 Implement the two admin RPCs in `admin.rs`/`admin_grpc.rs`/`pg_admin.rs`: the funnel per day 0 with a capped mark, and the reason totals, each filterable by browser. Verify: grpc tests for the window, the browser filter, a non-admin refused, and a music-only admin refused.

## 3. The back office (≈ 1.5 d)

- [ ] 3.1 Regenerate the stubs, and add the funnel and the reasons to `apps/back-office/src/stores/lingua.ts`, each as an `Async<T>` behind `api()`. Group days into weeks there, and compute the shares and the "not yet" cells (day 2, 14 and 60 after a week's last day). Verify: store unit tests for grouping, shares, "not yet", capped weeks and a failed RPC (a localised error).
- [ ] 3.2 Add the "Installations" section to `LinguaView.vue`: the table by week, the browser filter, the leaving reasons, the note on anonymous counts, and one series of installations per day through the existing chart. Strings in en and fr; the vocabulary lint passes. Verify: `yarn lint`, `yarn typecheck`, `yarn test`, and a Playwright e2e through the fake-client seam.

## 4. The extension (≈ 4 d)

- [ ] 4.1 Write `src/usage/counts.ts`, the pure state (key `cymbra-lingua-usage-counts` in `chrome.storage.local`): day 0 from the local date, the steps sent and waiting, the notice mark, the last failure. Record each step once with the D1 windows, forget everything after day 59, and drop all on "off" or a full reset. Verify: vitest for every scenario of *Six anonymous steps*, including a weekend reader, day 200, an update and a clock change around midnight.
- [ ] 4.2 Write `src/usage/client.ts`: a gRPC-web transport with **no interceptor**, and the send of waiting steps with the D5 outcomes. Accepted and invalid replies drop the steps; resource-exhausted and transport errors retry at most once an hour. Verify: tests with a mocked client for each outcome, and one asserting no `Authorization` header while a session holds a token.
- [ ] 4.3 Wire the background:
  - `onInstalled` (`install` only) records installed;
  - a `stats` message with a percentage records the first analysis and a use;
  - `sync:request` with `surface` records a use;
  - a new `usage:step` message records set up, sent by `src/onboarding/level-row.ts` and `setLevel` in `src/reading/settings-view.ts` after a level is saved;
  - `usage:notice-shown` marks the notice;
  - `onStartup` and each of these retry the waiting steps.

  Verify: background tests for each trigger, with nothing sent before the notice except on Firefox.
- [ ] 4.4 Add the switch to the Data tab of `src/reading/settings-view.ts`. Chromium and Safari read the key `cymbra-lingua-usage-counts-off`. Firefox maps it onto `permissions.getAll()` / `request` / `remove` with `data_collection: ["technicalAndInteraction"]`, follows `permissions.onAdded`/`onRemoved`, and hides it where `data_collection` is unsupported. Strings in `src/i18n/{fr,en,es}/settings.ts`, with the privacy-policy link in the interface language. Verify: settings tests for default on, off (forgets and unregisters), on again (nothing more), Firefox granted, Firefox declined, an old Firefox (hidden), and `test/i18n.spec.ts`.
- [ ] 4.5 Add the first-run sentence: to the welcome tab (`src/onboarding/onboarding.html`, `onboarding.ts`, `src/i18n/{fr,en,es}/onboarding.ts`), and to the popup on its first opening (`src/popup/popup.ts`, `src/i18n/{fr,en,es}/popup.ts`). Each sends `usage:notice-shown` once rendered. Not on any reading surface. Add the build define `__USAGE_COUNTS_DEFAULT__` (`on` for every variant; `ask` adds a yes/no pair to the sentence, holds the steps until yes, and turns counting off on no — design D7) to `build.mjs` and `env.d.ts`. Verify: onboarding and popup tests in both modes, and `test/lint-copy.spec.ts`.
- [ ] 4.6 Register the leaving page with `setUninstallURL` at start-up and on a change of switch or interface language: the D12 path per language, `?b=&v=` only, and `""` when off. Compile it out of the Safari bundle (`__TARGET__`). Verify: tests for fr/en/es, off, and the Safari bundle not containing the call (`yarn check:variants`).
- [ ] 4.7 Build every variant. Verify: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` with coverage, `yarn build`, `yarn check:variants`.

## 5. The site (≈ 1.5 d)

- [ ] 5.1 Add the leaving page at `/lingua/au-revoir/`, `/en/lingua/goodbye/` and `/es/lingua/adios/`: the question, six buttons, thanks with the support and community links, and copy in `src/lib/i18n.ts` for fr, en and es. The island goes through `src/composables/useLeaveReason.ts` and a client in `src/lib/`, with an `Async<T>` state; nothing is sent on load and at most one answer per visit. Verify: island tests (answer sent, failure thanked silently, missing parameters sent as `unknown`), `yarn typecheck`, `yarn test`, `yarn build`.
- [ ] 5.2 Add the three paths to `PINNED_ROUTES` (`apps/site/src/lib/pinned-routes.ts`), pinned by "Lingua — uninstall page, compiled into Chromium and Firefox builds". Verify: `yarn check:routes`.

## 6. Integration (≈ 1 d)

- [ ] 6.1 On the local stack, install fresh in Chrome, Firefox 140+ and Safari for macOS. Choose a level, read a Spanish page, then move the device clock to day 1 and day 8 and use Lingua. Verify: the counters for that day 0 read installed, set up, first analysis, day 1 and days 7–13 once each per browser; the requests carry no `Authorization` header; no report left before the first-run sentence on Chrome and Safari.
- [ ] 6.2 Turn the switch off with a step waiting, and remove Lingua from Chrome and Firefox both with the switch on and with it off. Verify: nothing more is counted after "off"; the leaving page opens only when on, and one tap adds one reason.
- [ ] 6.3 Read the server and Caddy output for the whole run. Verify: no address, `Origin` or `User-Agent` value appears.
- [ ] 6.4 Run `openspec validate add-lingua-usage-counts --strict`. Verify: it passes.

## 7. The owner's tasks

- [ ] 7.1 Deploy the server: migration `0008`, the service, the route and the two flags. Then check from outside: a report sent with `buf curl` to `https://api.cymbra.app` increments one counter (read in the back office), and the 61st request from one address within an hour is refused. Verify: the console shows the test counter. Then remove it with a one-off `DELETE` on the test day 0.
- [ ] 7.2 Deploy the site: the annex and the leaving page, before any build that registers it. Verify: the three leaving pages and the three annexes are live.
- [ ] 7.3 Chrome Web Store: update the Privacy practices tab from 1.3 and the description from 1.2. Verify: the dashboard matches the file.
- [ ] 7.4 AMO: update the description from 1.2 and the reviewer notes from 1.4. Verify: the listing matches the file.
- [ ] 7.5 App Store Connect: update the description and the App Review notes from 1.2 and 1.5. The privacy answers need no edit (1.6). Verify: the next submission carries them.
- [ ] 7.6 Release the extension that sends, only after 7.1 and 7.2. Verify: a week later, the console shows that week's installations per browser.
- [ ] 7.7 Rehearse the kill switch: set `lingua.usage_counts.enabled` off, send a report, set it back on. Verify: no counter moved while it was off.
