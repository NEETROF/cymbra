## Context

See proposal.md for why. What the code does today, and what this design leans on:

- **Nothing measures a signed-out reader.** The Lingua console reads only synced daily statistics
  (`backend/lingua/proto/lingua_admin.proto`, "computed from the existing sync data (no new
  telemetry)"). The first-party pipeline `backend/analytics` is Music's: `ReportEvents` requires a
  signed-in caller and files each event under `user_bucket = HMAC(salt(month), user_id)`
  (`backend/analytics/README.md`), with raw events kept 180 days.
- **The background already hears every moment that matters.**
  - `chrome.runtime.onInstalled` runs with `details.reason` (`src/background.ts`, near the end).
  - Every content-script pass sends `{ type: "stats", pct }`, where `pct` is null when the page holds
    nothing analysable in a studied language (`src/reading/session.ts`, `pushBadge`).
  - Every Lingua surface sends `{ type: "sync:request", reason: "surface" }` when it opens
    (`src/popup/popup.ts`, `src/sidepanel/sidepanel.ts`, `src/reading/drawer.ts`), and pages send
    `reason: "page"` (`session.ts`); the background answers them whether or not anyone is signed in.
- **"Setup done" has one portable meaning: a level decision.** The welcome tab has no "finish"
  button and Safari may never open it (`src/onboarding/onboarding.ts`: "a best-effort bonus — the
  popup's level call-to-action is the portable equivalent"). Choosing « Débutant » is a decision
  too (`src/state/level-choice.ts`). Levels are set in two places: `src/onboarding/level-row.ts`
  and `setLevel` in `src/reading/settings-view.ts` (the popup's call-to-action opens the latter).
- **The extension has no timer permission** (`manifest.json`: `activeTab`, `storage`, `scripting`,
  `sidePanel`, `identity`) and schedules work on its own wake-ups (`src/sync/scheduler.ts`).
- **Per-device choices live in `chrome.storage.local`**, never in the backup and never synced —
  « Traduction en ligne » is the closest precedent (`ONLINE_TRANSLATION_KEY`, `src/state/storage.ts`).
  Days are counted as day numbers (`utcDay`, `src/state/dailystats.ts`).
- **The sync's installation id exists and must stay out of this.** `getOrCreateDeviceId`
  (`src/sync/sync.ts`) is a random UUID used to break ties between devices.
- **The gRPC-web transport attaches the account's bearer to every call** except refresh and logout
  (`src/net/transport.ts`, `WITHOUT_BEARER`).
- **Firefox already lists "technical and interaction data" as optional, and nothing reads it.** The
  `data_collection_permissions` comment in `tool/manifests.mjs` says it covers "daily stats, the
  random install id", but no code calls `permissions.getAll()` for `data_collection`.
- **Uninstall page: not used.** `runtime.setUninstallURL` is not called anywhere. MDN's compatibility
  data: Chrome and Firefox (desktop and Android) honour it (1023 characters since Chrome 115 and
  Firefox 116, 255 before); Safari and Safari iOS "API exists, but has no effect".
- **The Lingua backend is the place for Lingua's numbers.** It already keeps day totals with no
  account and no text for « Traduction en ligne » (`lingua.online_translation_daily`, migration
  `0007`, `src/pg_online_usage.rs`), reads runtime flags through a port it declares
  (`OnlineTranslationPolicy`, `src/online_translation.rs`) implemented in `backend/server/src/flags.rs`,
  and serves the console through `LinguaAdminService` (admin in the `lingua` scope).
- **Privilege chain.** `CYMBRA_LINGUA_DATABASE_URL` → `db::connect` in `backend/server/src/main.rs`
  (the lingua block) → role `lingua_svc` → `search_path = lingua`
  (`backend/db/init/roles.sql.tpl`). Every Lingua service and the admin repo run on that pool; the
  existing queries are schema-qualified (`lingua.online_translation_daily`).
- **Routing and logs.**
  - Caddy sends `/.well-known/* /healthz /readyz /web/* /soundfonts/* /me/* /scores/* /billing/*` to
    Axum and everything else to tonic (`backend/deploy/Caddyfile`). The Caddyfile has no `log`
    directive, so Caddy writes no access log.
  - The server's only per-request span is `grpc.request` with `rpc.method`
    (`backend/platform/src/metrics.rs`, `ObserveLayer`); there is no HTTP trace layer.
  - Client addresses are read only for rate limits (`backend/platform/src/client_addr.rs`), and the
    shared limiter writes its subject into a Redis key (`rl:{scope}:{subject}`,
    `backend/platform/src/ratelimit.rs`) — for sign-in, that subject includes the address.
- **Origins.**
  - gRPC-web CORS admits the back office and `CYMBRA_ALLOWED_WEB_ORIGINS` (the Chrome extension's
    origin); Firefox and Safari need no entry because the background holds the backend in
    `host_permissions` (`backend/deploy/.env.prod.example`).
  - The site's JSON routes under `/web/` admit `CYMBRA_WEB_ORIGINS`, which holds
    `https://cymbra.app` (`backend/server/src/web_plans.rs`).
  - The site calls the backend with plain `fetchJson`, not gRPC-web (`apps/site/src/lib/web-plans.ts`).
- **Published promises that this changes.**
  - Store descriptions: « Sans compte ni traduction étendue, elle ne fait aucune requête réseau »,
    with the English and Spanish equivalents (`apps/lingua-extension/STORE-LISTING.md`;
    `apps/lingua-apple/STORE-LISTING.md`, « VOTRE LECTURE RESTE À VOUS »).
  - Firefox reviewer notes: "A reader who never signs in and never turns on « Traduction étendue »
    makes no request at all" (`apps/lingua-extension/REVIEWERS.md`).
  - App Review notes: "External services: none by default … no analytics" — 3976 of 4000
    characters (`apps/lingua-apple/STORE-LISTING.md`).
  - Chrome privacy practices: "no" to user activity (`apps/lingua-extension/STORE-LISTING.md`, *Data
    usage disclosures*).
  - Spec requirements: `lingua-browser-extension` *No network requests* and `lingua-sync` *Optional
    account, local-first preserved*.
  - The privacy policy has no Chrome Web Store Limited Use statement (`grep` finds none in
    `apps/site/src`).

## Goals / Non-Goals

**Goals:**
- Six figures per installation week and browser, true enough to compare one week with the next.
- A report that cannot identify an installation, a person or what they read, and a server that
  keeps nothing that could be turned into one.
- Counting the reader can turn off in one place, told before anything is sent.
- One question to people who leave, with answers that cannot carry personal data.

**Non-Goals:**
- Counting anything about reading: pages, sites, words, languages studied, levels, time spent.
- Daily or monthly active users, sessions, feature usage, crash reports. If wanted later, they are
  a separate change against the same rule.
- Counting the Apple host app, or Music.
- Publishing these figures on Discord (the policy's published figures exclude "numbers of
  people"; installation counts are exactly that).
- Deduplicating reinstalls or several devices — impossible without an identifier, by design.

## Decisions

### D1. Six steps, each reported once, with these exact meanings

Day 0 is the local calendar day the installation was made on (one notion of day on the device:
"the next day" is the reader's next day, not a UTC rollover at 7 p.m. in New York). A **use** is a
`stats` message with a non-null percentage — a page analysed in a studied language — or a
`sync:request` with reason `surface` (popup, side panel, drawer, review, book reader). The welcome
tab opening by itself at install is not a use.

| Step (wire) | Recorded when |
|---|---|
| `INSTALLED` | `onInstalled` with reason `install` |
| `SET_UP` | the first level decision, any studied language, « Débutant » included |
| `FIRST_ANALYSIS` | the first `stats` message with a percentage |
| `RETURNED_DAY_1` | a use on day 1 |
| `RETURNED_DAYS_7_13` | the first use on a day from 7 to 13 |
| `RETURNED_DAYS_30_59` | the first use on a day from 30 to 59 |

After day 59 the device forgets day 0 and the step list; that installation never reports again.
`SET_UP` comes from a one-line message (`usage:step`) sent by `level-row.ts` and `settings-view.ts`
after a level is saved; the background records the rest from messages it already receives.

*Rejected:*
- **Exact day 7 and day 30.** A weekend reader who reads on day 6 and day 8 never counts; with
  today's numbers (one daily Firefox user) the figure would be mostly noise.
- **"On or after day N"** with no end. A return on day 200 would count as "back at day 7", so the
  figure would say nothing about the second week.
- **"Setup = welcome tab finished."** No such moment exists, and Safari rarely opens the tab.
- **"Use = any `sync:request`".** On Firefox and Safari the reader runs on every page
  (`tool/manifests.mjs`, static content script), so `page` requests fire on pages in no studied
  language: that would measure browser use, not Lingua use.

### D2. Filed under the installation's day, with no identifier

A report says: day 0, browser family, version, and the steps it carries. The server adds one to a
counter per (day 0, step, browser, version) and keeps nothing else. Filing a step under day 0 — not
under the day it happened — is what makes « 40 installations this week, 8 came back at D+7 » a true
sentence: the 8 belong to the 40.

Why this is not an identifier: many installations share each day-0/browser/version tuple, the tuple
is never stored with a report (there is no report row), and two reports from one installation look
exactly like reports from two installations made the same day.

What it does allow: with very small numbers, a counter can describe one person ("one Firefox
installation of 20 October came back in its second week"). The person is not identifiable from it,
and the console groups days into weeks by default to soften this.

*Rejected:*
- **An installation id**, or reusing the sync's (`getOrCreateDeviceId`). Excluded by the owner's
  rule; reusing the sync's would also tie counts to accounts on the server.
- **`backend/analytics`'s period-salted `user_bucket`.** It is pseudonymous — the same person maps
  to the same bucket for a month — which is exactly what "no identifier, no hash of one" rules out.
  Its ingestion is authenticated and its raw events carry the bucket for 180 days.
- **Filing steps under the day they happened.** Simpler, but a week's "8 came back at D+7" would
  then come from installations of the week before, so no percentage could be computed.

### D3. Two dimensions: browser family and version

- **Browser family** — `chromium`, `firefox` or `safari`, from the build target (`__TARGET__`), never
  from the User-Agent. Each family is a separate store with its own listing, so a funnel per family
  is a funnel per acquisition channel. Three values cannot single anyone out.
- **Version** — `major.minor` of the extension (`package.json`'s `1.9.0` is sent as `1.9`). It tells
  whether a release changed setup or returns; patch releases add granularity and no insight.

*Rejected:* desktop vs mobile, operating system, interface language, studied language, country.
Each multiplies the counter cells — and the chance a cell holds one person — for a question that
is not asked today. The studied language is also about what the reader reads, which the rule
excludes. If the mobile/desktop split is wanted, it is one more enum in a later change.

### D4. What the device keeps, and for how long

One `chrome.storage.local` key, `cymbra-lingua-usage-counts`: day 0, the steps already sent, the
steps waiting to be sent, whether the first-run sentence was shown, and the time of the last failed
attempt. It is not in the backup, not in the exported file, not synced.

- **Lifetime.** It is deleted after day 59 or when the switch is turned off. This stays well inside
  the CNIL's consent exemption for audience-measurement trackers (first party, anonymous statistics
  only, a lifetime of at most 13 months that is never extended).
- **A full reset** in Settings deletes it too, so the installation stops being counted; the switch
  keeps its position.
- **Installations made before the first counting release** receive `onInstalled` with `update`, so
  they have no day 0 and never report. The funnel starts clean with that release. They could be
  given a fake day 0, but their steps would be counted against a week in which they were not
  installed.
- **A reinstall** is a new installation: the browser wipes extension storage on removal.
- **Several devices** count as several installations. Counts never ride the sync.

### D5. Reports go when a step is recorded, and retry on later wake-ups

When the background records a step, it tries to send every waiting step in one report, while it is
already awake for that message. A failure keeps them waiting; the next wake-up tries again, at most
once an hour after a failure. The wake-ups are `onInstalled`, `onStartup`, a `stats` message and a
`sync:request`. No alarm and no timer are used. Offline readers' steps leave when they are back
online, still under day 0.

The server answers "OK" to everything it can parse, counted or not: flag off, day 0 out of range,
or a ceiling reached. A client therefore never retries a dropped report. An unparseable report gets
`INVALID_ARGUMENT`; the client also drops those steps, so a bug cannot loop. Only
`RESOURCE_EXHAUSTED` and transport errors are retried.

*Rejected:*
- **One batch a day.** A reader who installs, sets up, reads one page and never returns is the
  person the funnel exists for; waiting for tomorrow's batch would lose exactly them.
- **A random delay** before sending, so a report cannot be tied to an installation moment. The
  server keeps no time and no address, so there is nothing to tie it to, and a suspended service
  worker would lose delayed reports.
- **`chrome.alarms`.** A new permission, and against the product's line of no timers.

### D6. The switch, per browser

« Comptages anonymes » / "Anonymous usage counts" / « Recuentos anónimos » is a checkbox in Settings
→ Data, with a note saying what is counted, that nothing read is sent, and a link to the privacy
policy in the interface language (`PRIVACY_POLICY_URLS`, `src/translate/online.ts`).

| Browser | The switch is | Default |
|---|---|---|
| Chromium | the key `cymbra-lingua-usage-counts-off` (absent = on) | on |
| Safari | the same key | on |
| Firefox desktop 140+, Android 142+ | the `technicalAndInteraction` grant: read with `permissions.getAll()`, asked with `permissions.request({ data_collection: [...] })`, withdrawn with `permissions.remove`; `permissions.onAdded` and `onRemoved` keep it in step with `about:addons` | as the reader left the install prompt (Firefox ticks it) |
| Firefox before 140 / Android before 142 | absent — those versions have no data-collection consent, so nothing is counted | off |

**Off** deletes the waiting steps and day 0, sends nothing more, and unregisters the leaving page
(`setUninstallURL("")`). **On again** sends nothing for this installation: it no longer has a day 0.
This keeps every count of a later step backed by a counted installation, so a percentage never
goes above 100 %. Signed in or not makes no difference.

### D7. Told before anything is sent

No report leaves before the reader has been shown, once, one sentence: what is counted, that nothing
read leaves the device, and where to turn it off. It sits:
- on the welcome tab, which opens at install on Chromium and Firefox;
- in the popup, the first time it opens — Safari's way in, and Chromium's when the welcome tab did
  not open.

Either one rendering it marks it shown (a `usage:notice-shown` message). It stays, permanently, as
the switch's note in Settings → Data. On Firefox 140+ the install prompt is that moment, so reports
may leave at once. The sentence is not a notification, a badge or a banner over a page.

This is what Chrome asks for ("Prominently disclose what user data will be collected", Chrome Web
Store *Disclosure requirements*). See open question Q1 for the consent half.

The same gate serves an "ask" build, should Q1 require one. A build define,
`__USAGE_COUNTS_DEFAULT__` = `on` | `ask`, controls it; every variant ships `on`. In an `ask` build
the sentence carries two buttons, and the gate opens on "yes" instead of on display; "no" is the
switch turned off. Until the reader answers, steps are recorded and held, never sent. Changing the
default is then a define, not a redesign.

### D8. The transport: gRPC-web without a token, to a service with no authentication

- **Client.** The background calls `cymbra.lingua.v1.UsageCountsService/Report` through a second
  gRPC-web transport built with **no interceptor** — no bearer, no refresh, no session-expiry hook
  — in `src/usage/client.ts`, so no code path can attach the account.
- **Server.** The service is mounted in `backend/server/src/main.rs` **without** `strict` or
  `optional`: an `Authorization` header is never even verified, so a report cannot resolve an
  account.
- **Contract.** It is a `.proto` in `backend/lingua/proto/`, checked by `buf breaking`, because
  extensions keep sending the shape they shipped with for months.
- **No CORS or Caddy change.** The Chrome origin is already allowed for gRPC-web, Firefox and Safari
  call from the background with the backend in `host_permissions`, and the path falls through to
  tonic.

One finding matters here. On Firefox the extension's origin is `moz-extension://<uuid>`, a value
"randomly generated for every browser instance" (MDN). A request from the background may carry it
as `Origin`, and that would work as an installation identifier if anyone stored it. Hence D11.

*Rejected:* an Axum JSON route for the extension. It would mean a second client stack in the
extension, and no `buf` gate on a contract that installed clients cannot update.

### D9. A module in `backend/lingua`, two counter tables in the `lingua` schema

`usage_counts_core.rs` holds the pure rules: validation, bounds, the closed sets. `usage_counts.rs`
holds the module and the ports it declares:
- `UsageCountsRepo`;
- `UsageCountsPolicy` — the flag and the ceiling, implemented in `backend/server/src/flags.rs` like
  `OnlineTranslationPolicy`;
- `FloodGuard` — see D10.

`usage_counts_grpc.rs` and `pg_usage_counts.rs` are thin glue. Migration `0008`:

```sql
CREATE TABLE IF NOT EXISTS lingua.usage_counts_daily (
  install_day INTEGER NOT NULL,  -- day 0, days since the epoch of the device's local date
  step        TEXT    NOT NULL,  -- installed | set_up | first_analysis | returned_day_1 | …
  browser     TEXT    NOT NULL,  -- chromium | firefox | safari
  version     TEXT    NOT NULL,  -- major.minor
  count       BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (install_day, step, browser, version)
);
CREATE TABLE IF NOT EXISTS lingua.leave_reasons_daily (
  day     INTEGER NOT NULL,      -- UTC day the answer arrived
  reason  TEXT    NOT NULL,      -- one of the six fixed reasons (D12)
  browser TEXT    NOT NULL,
  version TEXT    NOT NULL,
  count   BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (day, reason, browser, version)
);
```

**No column for a time of day, an address, a header or a report.** Counters are upserted with
`count = count + 1 WHERE count < ceiling`. They are anonymous aggregates — not personal data — so
they have no expiry.

**Privilege chain.** Writes and the console's reads run on the `lingua_svc` pool
(`CYMBRA_LINGUA_DATABASE_URL`), whose `search_path` is `lingua`, and the queries are
schema-qualified like `pg_online_usage.rs`.

**Inert without its environment.** Without that URL the service and the route are not mounted, like
every Lingua service. The runtime flag `lingua.usage_counts.enabled` (app `lingua`, default on) stops
counting without a release: the server answers OK and records nothing. The flag is administered in
the existing flags console.

*Rejected:* the `analytics` schema. Its model is pseudonymous and authenticated, its tables carry a
bucket, its role is `analytics_svc`, and the Lingua console (admin in `lingua`) would have to read
another module's schema. Reusing it would also tie the change to the still-open
`add-feature-usage-analytics`.

### D10. Abuse: bounded values, an address limiter that writes nothing, a ceiling

An unauthenticated counter can always be inflated by someone determined. The aim is to make that
costly and visible, not impossible.

- **Bounded values.** A step is one of six, a browser one of three, and `version` matches
  `^\d{1,3}\.\d{1,3}$`. A report holds one to six distinct steps. Day 0 must lie within 120 days
  before the server's UTC day and one day after it, to allow time zones ahead of UTC. Out of range
  is dropped and answered OK; malformed is `INVALID_ARGUMENT`.
- **Floods from one address.** At most 60 reports an hour per address, enforced by an in-memory
  `FloodGuard`. It keys on an HMAC of the address under a key drawn at start-up and never written,
  so even a memory dump holds no address. An entry is forgotten when its hour ends. One
  installation sends at most six reports in its whole life, so 60 an hour leaves room for a school
  or an office behind one address. The address is resolved with `client_addr::resolve` and passed to
  the guard alone. The platform limiter is not used, because it writes its subject into a Redis key.
- **A ceiling per counter.** At most `lingua.usage_counts.ceiling` per (day 0, step, browser,
  version), 5 000 by default (runtime config). A capped day shows as such in the console.

The same flag, guard and ceiling apply to the leaving route (D12). Its bounds are the reason (one of
six), and the browser family and version as above, where `unknown` is also accepted: the page may be
opened without its parameters.

*Rejected:* no limit per address (one script inflates everything); the platform's Redis limiter
(writes the address); a proof-of-work or a signed token (an anonymous client can still mint either,
at real cost to the code).

### D11. Nothing that identifies a request is kept

The report handler reads the message fields and nothing else: no header, no remote address (only
the flood guard sees the address, as D10 describes). It logs nothing about a report. No metric
labels go beyond the method that `ObserveLayer` already records. Caddy keeps its no-access-log
configuration; a comment in the Caddyfile says why, so nobody adds a `log` directive without seeing
it.

A test captures tracing output across a counted report, a dropped one and a refused one. It
asserts that no address, no `Origin` value and no `User-Agent` value appears.

### D12. The leaving page

- **Registration.** While counting is on, the background calls
  `chrome.runtime.setUninstallURL(url)` at start-up and whenever the switch or the interface
  language changes:

  | Interface language | Leaving page |
  |---|---|
  | fr | `https://cymbra.app/lingua/au-revoir/` |
  | en | `https://cymbra.app/en/lingua/goodbye/` |
  | es | `https://cymbra.app/es/lingua/adios/` |

  The URL carries `?b=<browser>&v=<major.minor>` and nothing else. With the switch off the URL is
  `""`. The call is compiled out of the Safari bundle, where it does nothing.
- **The page.** A static Astro page in each language asks « Pourquoi quittez-vous Cymbra Lingua ? »
  and offers six buttons:
  1. « Je ne m'en servais pas » — `not_using`
  2. « Trop de mots surlignés, ou pas assez » — `highlighting`
  3. « Ma langue n'y est pas » — `missing_language`
  4. « Ça ralentissait mon navigateur » — `slow`
  5. « Je n'ai pas compris comment m'en servir » — `unclear`
  6. « Une autre raison » — `other`

  Loading the page sends nothing. Tapping a button sends one `POST /web/lingua/leave`
  `{ reason, browser, version }`, disables the buttons and thanks the reader, whatever the answer.
  The thanks offer the support page and the community link for anyone who wants to say more. The
  island follows the site's rules: a composable behind a client seam and an `Async<T>` union,
  never `fetch` in the component. A missing or malformed `b`/`v` is sent as `unknown` and counted
  under it.
- **The route.** `backend/server/src/web_lingua.rs` follows `web_plans.rs`: a JSON route under
  `/web/`, already on Caddy's `@http` list, with CORS limited to `CYMBRA_WEB_ORIGINS` and no
  credentials. It delegates to the same module and goes through the same flag, ceiling and guard,
  with the bounds D10 gives it.
- **Pinned routes.** The three paths join `PINNED_ROUTES` (`apps/site/src/lib/pinned-routes.ts`):
  shipped builds compile them in.

*Rejected:*
- **A free-text box.** People type e-mail addresses, names and URLs into free text. That would be
  personal data with no identifier to erase it by, to moderate and keep, behind an unauthenticated
  endpoint that spam would find. Anyone with more to say has the support page and Discord.
- **Counting the page's load as "an uninstall".** Anyone can open the URL, and the reader took no
  action.
- **Using gRPC-web from the site.** It would need a new dependency and a new CORS origin for one
  call.

### D13. The funnel in the back office

`LinguaAdminService` gains two RPCs, behind the same `admin`-in-`lingua` guard and the same
"aggregates only" rule:
- `AdminGetUsageFunnel(window, browser)` — for each day 0 in the window: installed, set up, first
  analysis and the three returns, and whether a counter hit the ceiling;
- `AdminGetLeaveReasons(window, browser)` — the total for each reason.

`/lingua/overview` gains an "Installations" section:
- a table by installation week, with each step as a count and as a share of "installed";
- a browser filter;
- a return cell reads "not yet" until its window has closed for every day of the week (day 2, 14
  and 60 after the week's last day), so a young week never looks like a bad week;
- the leaving reasons as a list of totals;
- a note that these are anonymous counts from installations that count, and that reinstalls and
  several devices each count as one.

The store keeps the existing `Async<T>` union; strings are in en and fr. The section reuses the
console's window. Its one chart is the existing day-series component, plotting installations per
day.

### D14. Disclosures and store forms, per browser

| Where | What changes | Evidence |
|---|---|---|
| Privacy policy, Annex B (fr/en/es) | a paragraph on the counts and the leaving page: what is counted, the two dimensions, no identifier, no address kept, day 0 kept on the device for at most 60 days, the switch, Firefox's consent; legal basis legitimate interest (audience measurement), objection through the switch | `apps/site/src/pages/confidentialite.md` and its two translations |
| Privacy policy, general part (fr/en/es) | the Chrome Web Store Limited Use statement | the Limited Use policy (item 6) asks for a compliance statement on a website of the extension, such as its privacy policy (developer.chrome.com/docs/webstore/program-policies/limited-use) |
| Firefox manifest | `technicalAndInteraction` stays in `optional` (it cannot be required), and the comment names the counts | Extension Workshop: the one exception is "`technicalAndInteraction` that cannot be required"; Firefox's own install code says it "is an opt-out setting" |
| `REVIEWERS.md` | "Where the add-on reaches the network" names the report and the leaving page | — |
| Chrome Web Store privacy practices | "User activity" ticked, with the counts described; the three certifications unchanged | Q9 |
| Store descriptions (fr/en/es, the three stores) | the "no network request" sentence replaced | Q6 |
| App Review notes | "External services: none by default … no analytics" rewritten, and the notes trimmed back under 4000 characters | — |
| App Store privacy answers (`apps/lingua-apple/README.md`) | reviewed: Usage Data → Product Interaction is already declared, linked, with Analytics as a purpose; anonymous counts add no type and no purpose, so App Store Connect needs no edit | the review is recorded with its date |

Apple's Guideline 5.1.1(ii) asks for consent "even if such data is considered to be anonymous",
unless a legitimate interest under the GDPR applies with an easy way to withdraw. The switch is that
way, and the policy states the legitimate interest.

Sources, read on 2026-10-13:
- extensionworkshop.com/documentation/develop/firefox-builtin-data-consent/ — Firefox 140 for
  desktop and 142 for Android show the consent;
- extensionworkshop.com/documentation/publish/add-on-policies/ — technical and interaction data may
  be opt-out, and declining it must not restrict functionality;
- developer.chrome.com/docs/webstore/program-policies/disclosure-requirements and
  …/user-data-faq;
- developer.apple.com/app-store/app-privacy-details/ and …/review/guidelines/ (5.1.1);
- cnil.fr, "Cookies : solutions pour les outils de mesure d'audience";
- MDN browser-compat-data, `runtime.setUninstallURL`.

### D15. Order

1. **Server.** Migration, service, route, flag. Deployed, then checked from outside: a report sent
   with `buf curl` increments one counter, and the 61st report from one address within an hour is
   refused.
2. **Site.** Annex and leaving page, deployed.
3. **Stores.** Text and forms updated in their consoles, ready to go with the build.
4. **Extension.** The release that sends. An extension that sends before step 1 is deployed loses
   nothing: its reports wait and retry.

The back office ships with step 1.

## Risks / Trade-offs

- **Inflation.** Anyone can add counts within the limits of D10, so the figures are a trend, not an
  audit. The console says "anonymous counts" and marks capped days.
- **Undercounting.** Readers who turn counting off, old Firefox versions, Safari readers who never
  open the popup, and installations made before the release are all missing. Percentages within a
  counted week stay meaningful; absolute numbers are a floor.
- **One person in a cell.** At today's scale a weekly Firefox cell may hold one installation. No
  identity can be drawn from it. Weeks and the browser filter are the only grains offered.
- **Chrome review.** Chrome's wording asks for "affirmative and informed consent". If a reviewer
  reads on-by-default as non-compliant, the Chromium build switches to asking once on the welcome
  tab (Q1) through `__USAGE_COUNTS_DEFAULT__ = ask` (D7).
- **Device clock.** Day 0 comes from the device clock; a clock far off produces a day 0 the server
  drops.
- **Archive order.** `add-lingua-youtube-captions` has no task done yet; this change's archive waits
  for it (Q8).

## Migration Plan

New tables only (migration `0008`, idempotent, `IF NOT EXISTS`); no backfill. Rollback: set
`lingua.usage_counts.enabled` off (clients keep working, nothing is recorded), or ship an extension
whose switch defaults off. Dropping the two tables loses only counts.

## Open Questions

Each has a recommended answer; the design above assumes it.

- **Q1. Chrome: on by default, or one click on the welcome tab?** Chrome's policy says that before
  handling "any user data" an extension must "prominently disclose" it and "obtain the user's
  affirmative and informed consent". It has no carve-out for anonymous counts. It also allows
  "analytics or performance data where it is reasonably necessary" with clear disclosure (User Data
  FAQ, Q8), and names "aggregate statistics in an internal dashboard" as acceptable internal use.
  *Recommended:* keep your decision — on by default with the D7 sentence and the Privacy practices
  declaration. Keep the "ask once" mode ready in the build, and switch to it only if a Chrome review
  objects. Firefox (the install prompt is the consent) and Safari (legitimate interest, the switch
  to withdraw) stay on by default either way.
- **Q2. Firefox before 140 (desktop) and 142 (Android):** no built-in consent there.
  *Recommended:* count nothing on those versions. Do not build a consent screen of our own. Whether
  to raise `strict_min_version` is a separate decision.
- **Q3. The return windows.** *Recommended:* day 1; days 7–13; days 30–59 (D1). The alternative is
  exact days 1, 7 and 30.
- **Q4. Installations made before the first counting release.** *Recommended:* not counted; the
  funnel starts with that release (D4).
- **Q5. The six leaving answers** (D12), with no free text. *Recommended:* as listed; you can reword
  them before the site task starts.
- **Q6. The sentence that replaces "no network request" in the store descriptions.**
  *Recommended:*
  - fr: « Ce que vous lisez ne quitte jamais votre appareil. Par défaut, Lingua envoie seulement des
    comptages anonymes — installation, premiers pas, retours —, sans aucun identifiant ; un
    interrupteur dans les Réglages les coupe. »
  - en: "What you read never leaves your device. By default Lingua sends only anonymous counts —
    install, first steps, returns — with no identifier of any kind; one switch in Settings turns
    them off."
  - es: « Lo que lees nunca sale de tu dispositivo. Por defecto, Lingua solo envía recuentos
    anónimos — instalación, primeros pasos, regresos —, sin ningún identificador; un interruptor en
    los Ajustes los desactiva. »
- **Q7. Firefox's "technical and interaction data" today claims to cover the signed-in daily stats
  and the sync's installation id, and nothing checks it.** *Recommended:* it gates the counts only.
  The comment is corrected: those synced items belong to the account the reader chose to sign in
  to. Whether declining it should also stop syncing statistics is a separate change, if you want
  one.
- **Q8. Archive order.** *No network requests* is also modified by `add-lingua-translation-delivery`
  (one task left) and `add-lingua-youtube-captions` (no task done), so both are in `archiveAfter`.
  *Recommended:* keep that. If YouTube captions are parked for good, say so and the requirement
  will be rebased on translation-delivery's text alone.
- **Q9. Chrome privacy practices: tick "User activity"?** The category is described elsewhere as
  "network monitoring, clicks, mouse position, scroll, or keystroke logging". That description was
  not found in Google's own documentation, which shows the list only in the dashboard. Counted steps
  are interaction with the extension. *Recommended:* tick it, with the counts described. Declaring
  too much is safe; a declaration that does not match what the extension does can get it removed.
