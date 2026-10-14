## Context

See proposal.md for why. What the code does today, and what this design leans on:

- **Nothing measures a signed-out reader.** The Lingua console reads only synced daily statistics
  (`backend/lingua/proto/lingua_admin.proto`, "computed from the existing sync data (no new
  telemetry)"). The first-party pipeline `backend/analytics` is Music's: `ReportEvents` requires a
  signed-in caller and files each event under `user_bucket = HMAC(salt(month), user_id)`
  (`backend/analytics/README.md`), with raw events kept 180 days.
- **The background already hears every moment that matters.**
  - `chrome.runtime.onInstalled` runs with `details.reason` (`src/background.ts`, near the end).
  - Every content-script pass sends `{ type: "stats", pct, language }`, where `pct` is null when the
    page holds nothing analysable in a studied language (`src/reading/session.ts`, `pushBadge`).
    The message also carries the studied language beside the page; the counting code reads `pct`
    only.
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
- **Firefox already lists "technical and interaction data" as optional, and nothing reads it.**
  `tool/manifests.mjs` declares `data_collection_permissions` with `required: ["none"]` and
  `technicalAndInteraction` among the `optional` types; its comment says that type covers "daily
  stats, the random install id", but no code calls `permissions.getAll()` for `data_collection`.
  `strict_min_version` is `128.0` for desktop and Android.
- **Uninstall page: not used.** `runtime.setUninstallURL` is not called anywhere. MDN's compatibility
  data: Chrome and Firefox (desktop and Android) honour it (1023 characters since Chrome 115 and
  Firefox 116, 255 before); Safari and Safari iOS "API exists, but has no effect".
- **The Lingua backend is the place for Lingua's numbers.** It already keeps day totals with no
  account and no text for « Traduction en ligne » (`lingua.online_translation_daily`, migration
  `0007`, `src/pg_online_usage.rs`), reads runtime flags through a port it declares
  (`OnlineTranslationPolicy`, `src/online_translation.rs`) implemented in `backend/server/src/flags.rs`,
  and serves the console through `LinguaAdminService` (admin in the `lingua` scope). The next
  migration number is taken: `add-lingua-card-edit` and `add-lingua-review-translation-fallback`
  both plan `0008` (`gloss_by_reader`).
- **Privilege chain.** `CYMBRA_LINGUA_DATABASE_URL` → `db::connect` in `backend/server/src/main.rs`
  (the lingua block) → role `lingua_svc` → `search_path = lingua`
  (`backend/db/init/roles.sql.tpl`). Every Lingua service and the admin repo run on that pool; the
  existing queries are schema-qualified (`lingua.online_translation_daily`).
- **Routing and logs.**
  - Caddy sends `/.well-known/* /healthz /readyz /web/* /soundfonts/* /me/* /scores/* /billing/*` to
    Axum and everything else to tonic (`backend/deploy/Caddyfile`). The Caddyfile has no `log`
    directive, so Caddy writes **no access log**.
  - **But Caddy's error log does carry the address and the headers.** Whatever the `log` directive,
    Caddy's server logs a handler error through its `http.log.error` logger, at ERROR level for a
    status of 500 or more, with the request attached as a `request` object
    (`modules/caddyhttp/server.go`, `errLog := s.errorLogger.WithLazy(loggableReq)`); that object
    holds `remote_ip`, `client_ip` and `headers` (`modules/caddyhttp/marshalers.go`). So every
    request that meets a down or restarting backend — a 502 during a deploy — leaves the client's
    address, its `User-Agent` and its `Origin` in Caddy's output, which Docker keeps as json-file
    logs (`docker-compose.prod.yml`, 3 × 10 MB).
  - The server's only per-request span is `grpc.request` with `rpc.method`
    (`backend/platform/src/metrics.rs`, `ObserveLayer`); there is no HTTP trace layer. Its logs go to
    the console and, with the `observability` profile, as OTLP logs to Grafana Cloud
    (`backend/platform/src/telemetry.rs`, `backend/deploy/otel-collector.prod.yaml`).
  - Client addresses are read only for rate limits (`backend/platform/src/client_addr.rs`: Caddy's
    first `X-Forwarded-For` hop), and the shared limiter writes its subject into a Redis key
    (`rl:{scope}:{subject}`, `backend/platform/src/ratelimit.rs`) — for sign-in, that subject
    includes the address.
  - The privacy policy's general part already lists "technical logs (IP address, timestamps,
    errors)", kept 7 days (`apps/site/src/pages/confidentialite.md`, §2 and §5). The counts' promise
    — no address kept — is therefore an exception the annex states and Caddy must make true (D11).
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
    `apps/lingua-apple/STORE-LISTING.md`, « VOTRE LECTURE RESTE À VOUS ») — every text is listed in D14.
  - Firefox reviewer notes: "A reader who never signs in and never turns on « Traduction étendue »
    makes no request at all" (`apps/lingua-extension/REVIEWERS.md`).
  - App Review notes: "External services: none by default … no analytics" — 3976 of 4000
    characters (`apps/lingua-apple/STORE-LISTING.md`).
  - Chrome privacy practices: "no" to user activity (`apps/lingua-extension/STORE-LISTING.md`, *Data
    usage disclosures*).
  - Spec requirements: `lingua-browser-extension` *No network requests* and `lingua-sync` *Optional
    account, local-first preserved*.
  - The privacy policy has no Chrome Web Store Limited Use statement (`grep -rn -i "limited use"
    apps/site/src` finds none). That is a gap **today**, whatever this change becomes: Lingua already
    handles user data (the account's e-mail, synced decks, « Traduction en ligne »).

### What the stores require, read at the source on 2026-10-14

**Chrome Web Store** (Google's developer documentation is published under CC BY 4.0).
- *Disclosure Requirements* (https://developer.chrome.com/docs/webstore/program-policies/disclosure-requirements,
  last updated 2022-11-01) defines user data as "information provided by a user or collected about
  a user or a user's use of the Product or Chrome browser", and says: "If your Product handles any
  user data, then prior to installation, it must: Prominently disclose what user data will be
  collected and how it will be used. Obtain the user's affirmative and informed consent for such
  use."
- *User Data FAQ* (https://developer.chrome.com/docs/webstore/program-policies/user-data-faq),
  question 10, "How do I satisfy the prominent disclosure requirement?": "To obtain consent, the
  Product must ask the user to agree to the prominent disclosure in a manner that requires them to
  take a specific action clearly agreeing to the disclosure before collecting or handling user
  data. The prominent disclosure and consent must occur within the Product's user interface.
  Disclosures in the Chrome Web Store description or inline installation page do not satisfy this
  requirement."
- The same FAQ, *Limited uses of user data*, question 8: "Extensions may collect analytics or
  performance data where it is reasonably necessary to maintain, secure, or measure the performance
  and reliability of the extension's disclosed functionality", the developer having to "Clearly
  disclose the data collection to users". It does not say that this disclosure replaces the consent
  of question 10. Question 6 allows a human to read data that the developer "aggregates and
  anonymizes … and only uses … for internal operations (for example, reporting aggregate statistics
  in an internal dashboard)".
- The same FAQ, *Limited uses of user data*, question 1: "All extensions that request user data must
  show a Limited Use disclosure on your project's homepage or on a page one click away from the
  homepage; for example, in your privacy policy." The *Limited Use* policy
  (https://developer.chrome.com/docs/webstore/program-policies/limited-use, item 6) asks for "an
  affirmative statement that your use of the data complies with the Limited Use restrictions" on a
  website belonging to the extension. The Limited Use policy also applies to data that is
  "aggregated, anonymized, de-identified, or derived".
- The same FAQ, *Simplifying privacy practices*, question 3: discrepancies between the dashboard's
  disclosures, the privacy policy and the item's behaviour can lead to "the suspension of all the
  items owned by the publisher".
- The *privacy practices* categories ("User activity" among them) are listed only in the
  developer dashboard; Google's public pages name none
  (https://developer.chrome.com/docs/webstore/cws-dashboard-privacy, last updated 2020-06-12).

**Reading.** A count of "came back in its second week" is information collected about a user's use
of the Product, so it is user data in Chrome's own definition, anonymous or not. Chrome then asks
for an in-product, specific action of agreement **before** collecting. No sentence of Google's
exempts anonymous analytics from that consent. **On-by-default counting on Chromium is not
compliant as written**; the counts can only start after the reader presses a "yes" (D6, D7, Q1).

**Firefox / AMO** (Mozilla's site content is under a Creative Commons licence).
- *Firefox built-in consent for data collection and transmission*
  (https://extensionworkshop.com/documentation/develop/firefox-builtin-data-consent/, last update
  2026-03-12): supported "in Firefox for desktop 140 and later, and Firefox for Android 142 and
  later"; data permissions "can be required or optional, except for technicalAndInteraction that
  cannot be required"; and "the user can turn this permission on or off during the installation
  flow". Its examples of technical and interaction data include "extension usage and settings
  data". For older versions it offers three options, among them: "Turn off the data collection for
  old Firefox versions."
- *Add-on Policies* (https://extensionworkshop.com/documentation/publish/add-on-policies/, last
  update 2026-04-30), 6.2.1: an add-on that uses the built-in consent "must accurately state the data
  collection practices in the extension manifest"; 6.2.2.3, *Technical & User Interaction Data
  (opt-out)*: the add-on "must allow the user to disable that data transmission (opt-out) during the
  initial consent experience. The add-on functionality must not be restricted if the user declines
  transmission of this data."
- Firefox's own code ticks the box: `browser/modules/ExtensionsUI.sys.mjs` sets
  `grantTechnicalAndInteractionDataCollection = true` under the comment "This is an opt-out
  setting.", and Firefox for Android's `PermissionsDialogFragment.kt` sets
  `technicalAndInteraction.isChecked = true` (mozilla-firefox/firefox, `main`, read 2026-10-14).

**Reading.** On Firefox 140+ (desktop) and 142+ (Android), on by default is what Mozilla designed:
the install prompt is the consent, ticked, and the reader can untick it there or later in
`about:addons`. Below those versions nothing is counted.

**Apple** (Apple's texts are copyrighted; short quotes only).
- *App Review Guidelines* 5.1.1(ii) (https://developer.apple.com/app-store/review/guidelines/):
  apps that collect usage data must get consent "even if such data is considered to be anonymous";
  an app relying instead on a legitimate interest under the GDPR or a similar statute must comply
  with all terms of that law; withdrawing must be easy.
- *App privacy details* (https://developer.apple.com/app-store/app-privacy-details/): *Product
  Interaction* covers "app launches, taps, clicks"-style usage, *Analytics* is a purpose, and data is
  "collected" only when kept, readable, longer than it takes to serve the request — an IP address
  used and not retained need not be declared. Apple's page does not mention app extensions; the
  Safari extension ships inside the host app, whose label is the only one it has.
- The recorded answers (`apps/lingua-apple/README.md`, *App Store privacy*) already declare
  *Usage Data → Product Interaction*, linked to the user, for App Functionality and Analytics.
  App Store Connect asks once per data type whether it is linked, so anonymous counts — the same
  type, not linked — change no answer.

**Reading.** In Safari, on by default rests on the legitimate interest (audience measurement),
stated in the privacy policy, with the switch to object; the App Review notes must say it.

**France (CNIL).** Storing day 0 on the device and reading it back is access to the terminal under
article 82 of the French data-protection law. The CNIL exempts audience-measurement trackers from
consent when they serve only the publisher's own measurement, yield anonymous statistics, do not
follow people across sites or apps, and share nothing with third parties; it recommends a tracker
lifetime of 13 months at most and keeping the collected information 25 months at most
("Cookies : solutions pour les outils de mesure d'audience", cnil.fr; the page refused automated
reading on 2026-10-14, so these conditions are taken from its published 2020–2025 guidance as
summarised by search results, to be re-read by the owner). Day 0 lives 60 days (D4); counters are
deleted after 25 months (D9).

## Goals / Non-Goals

**Goals:**
- Six figures per installation week and browser, true enough to compare one week with the next.
- A report that cannot identify an installation, a person or what they read, and a server — reverse
  proxy included — that keeps nothing that could be turned into one.
- Counting that starts as each store requires and that the reader can turn off in one place, told
  before anything is sent.
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

### D2. Filed under the installation's week, with no identifier

A report says: the week of day 0 (the day number of the Monday that starts day 0's local week),
browser family, version, and the steps it carries. The server adds one to a counter per (week,
step, browser, version) and keeps nothing else. Filing a step under the installation's week — not
under the moment it happened — is what makes « 40 installations this week, 8 came back at D+7 » a
true sentence: the 8 belong to the 40. The device still computes the return windows from the exact
day 0; only the week leaves it.

Why this is not an identifier: many installations share each week/browser/version tuple, the tuple
is never stored with a report (there is no report row), and two reports from one installation look
exactly like reports from two installations made the same week.

What it still allows: at today's scale a cell can hold one installation ("one Firefox 1.10
installation of the week of 19 October came back in its second week"). Nobody can be identified from
the counters alone; but someone who already knows that a given person installed Lingua on Firefox
on a given day could, with a **day** grain, read whether that person came back. The week grain makes
that seven times harder and costs the funnel nothing, since the console reads weeks anyway (D13).

*Rejected:*
- **The installation's day** (the first draft of this change). It adds nothing the console shows,
  and the exact day is the most precise of the three dimensions — the one that turns a small cell
  into a person for someone who knows when they installed.
- **Suppressing small cells** (showing "< 5") in the console. At today's scale it would blank the
  whole section, and the only reader of the console is the owner; the week grain and the absence of
  any finer view (D13) do the same work without hiding the figures.
- **An installation id**, or reusing the sync's (`getOrCreateDeviceId`). Excluded by the owner's
  rule; reusing the sync's would also tie counts to accounts on the server.
- **`backend/analytics`'s period-salted `user_bucket`.** It is pseudonymous — the same person maps
  to the same bucket for a month — which is exactly what "no identifier, no hash of one" rules out.
  Its ingestion is authenticated and its raw events carry the bucket for 180 days.
- **Filing steps under the week they happened.** Simpler, but a week's "8 came back at D+7" would
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
steps waiting to be sent, whether the first-run sentence was shown (and, on Chromium, answered), and
the time of the last failed attempt. It is not in the backup, not in the exported file, not synced.

- **Lifetime.** It is deleted after day 59 or when the switch is turned off — far below the
  13 months the CNIL accepts for an audience-measurement tracker, and never extended.
- **A full reset** in Settings deletes it too, so the installation stops being counted; the switch
  keeps its position.
- **Installations made before the first counting release** receive `onInstalled` with `update`, so
  they have no day 0 and never report. The funnel starts clean with that release. They could be
  given a fake day 0, but their steps would be counted against a week in which they were not
  installed. (This also means Chrome's rule on practices introduced after installation never comes
  into play: nothing changes for an existing installation.)
- **A reinstall** is a new installation: Chrome and Firefox wipe extension storage on removal.
  In Safari, switching the extension off and on again in the browser's settings keeps its storage
  and fires no `install`, so it is not a new installation; deleting the app is.
- **Several devices** count as several installations, and so do several browser profiles. Counts
  never ride the sync.

### D5. Reports go when a step is recorded, and retry on later wake-ups

When the background records a step, it tries to send every waiting step in one report, while it is
already awake for that message. A failure keeps them waiting; the next wake-up tries again, at most
once an hour after a failure. The wake-ups are `onInstalled`, `onStartup`, a `stats` message and a
`sync:request`. No alarm and no timer are used. Offline readers' steps leave when they are back
online, still under the week of day 0.

The server answers "OK" to everything it can parse, counted or not: flag off, week out of range,
or a ceiling reached. A client therefore never retries a dropped report. An unparseable report gets
`INVALID_ARGUMENT`; the client also drops those steps, so a bug cannot loop. Only
`RESOURCE_EXHAUSTED` and transport errors are retried.

**Delivery is at least once.** If the browser suspends the service worker after the server counted
a report but before the answer arrived, the steps stay waiting and are counted again later. This is
rare and only inflates; preventing it would need a report identifier, which the rule excludes.

*Rejected:*
- **One batch a day.** A reader who installs, sets up, reads one page and never returns is the
  person the funnel exists for; waiting for tomorrow's batch would lose exactly them.
- **A random delay** before sending, so a report cannot be tied to an installation moment. The
  server keeps no time and no address, so there is nothing to tie it to, and a suspended service
  worker would lose delayed reports.
- **`chrome.alarms`.** A new permission, and against the product's line of no timers.

### D6. When counting starts, per browser, and the switch

« Comptages anonymes » / "Anonymous usage counts" / « Recuentos anónimos » is a checkbox in Settings
→ Data, with a note saying what is counted, that nothing read is sent, and a link to the privacy
policy in the interface language (`PRIVACY_POLICY_URLS`, `src/translate/online.ts`).

| Browser | Counting starts | The switch is |
|---|---|---|
| Chromium (Chrome, Edge…) | only after the reader presses "Yes, count" on the first-run sentence (D7), or ticks the switch | the key `cymbra-lingua-usage-counts-off`; until answered, the switch shows off |
| Safari | by default, once the first-run sentence has been shown | the same key (absent = on) |
| Firefox desktop 140+, Android 142+ | as the reader left the install prompt's "technical and interaction data" box, which Firefox ticks | the `technicalAndInteraction` grant: read with `permissions.getAll()`, asked with `permissions.request({ data_collection: [...] })` from the switch's click, withdrawn with `permissions.remove`; `permissions.onAdded` and `onRemoved` keep it in step with `about:addons` |
| Firefox before 140 / Android before 142 | never — those versions have no built-in consent, and Mozilla's own guidance allows simply turning collection off there | not shown |

**Off** (the switch, "No, thanks" on Chromium, or the Firefox grant withdrawn) deletes the waiting
steps and day 0, sends nothing more, and unregisters the leaving page (`setUninstallURL("")`).
**On again** sends nothing for this installation: it no longer has a day 0. This keeps every count
of a later step backed by a counted installation, so a percentage never goes above 100 %. Signed in
or not makes no difference.

The Firefox manifest keeps `required: ["none"]` with `technicalAndInteraction` optional: Firefox's
install prompt then says the extension requires no data collection and offers the ticked
technical-data box, which is exactly true.

*Rejected:*
- **On by default on Chromium too**, the owner's first choice. Chrome's policy, read at the source
  above, asks for a specific action of agreement inside the extension before any user data is
  collected, and names no exception for anonymous counts. A store that finds the practice
  non-compliant can suspend every item of the publisher. See Q1.
- **Our own consent screen on Firefox before 140.** It would duplicate Firefox's, for a shrinking
  share of readers; `strict_min_version` is a separate decision (Q3).

### D7. Told before anything is sent

No report leaves before the reader has been shown, once, one sentence: what is counted, that nothing
read leaves the device, and where to turn it off. It sits:
- on the welcome tab, which opens at install on Chromium and Firefox;
- in the popup, the first time it opens — Safari's way in, and Chromium's when the welcome tab did
  not open.

A build define, `__USAGE_COUNTS_DEFAULT__` = `on` | `ask`, set per variant, decides what the sentence
does:
- **`ask` — Chromium.** The sentence carries two buttons, « Oui, compter » / "Yes, count" /
  « Sí, contar » and « Non merci » / "No, thanks" / « No, gracias », equal in size and weight. The gate
  opens on "yes"; "no" is the switch turned off. Until the reader answers, steps are recorded and
  held, never sent. Unanswered, the popup asks again at its next openings — inside the popup only,
  never over a page.
- **`on` — Safari.** Rendering the sentence opens the gate (a `usage:notice-shown` message).
- **Firefox** builds with `on`, but its gate is the browser's grant (D6); the sentence is still shown,
  for information.

The sentence stays, permanently, as the switch's note in Settings → Data. It is not a notification,
a badge or a banner over a page. Moving a variant from `on` to `ask` (should App Review object, for
instance) is a define, not a redesign.

### D8. The transport: gRPC-web without a token, to a service with no authentication

- **Client.** The background calls `cymbra.lingua.v1.UsageCountsService/Report` through a second
  gRPC-web transport built with **no interceptor** — no bearer, no refresh, no session-expiry hook
  — in `src/usage/client.ts`, so no code path can attach the account.
- **Server.** The service is mounted in `backend/server/src/main.rs` **without** `strict` or
  `optional`: an `Authorization` header is never even verified, so a report cannot resolve an
  account.
- **Contract.** It is a `.proto` in `backend/lingua/proto/`, checked by `buf breaking`, because
  extensions keep sending the shape they shipped with for months.
- **No CORS or routing change.** The Chrome origin is already allowed for gRPC-web, Firefox and
  Safari call from the background with the backend in `host_permissions`, and the path falls through
  to tonic.

One finding matters here. On Firefox the extension's origin is `moz-extension://<uuid>`, a value
"randomly generated for every browser instance" (MDN); Safari's `safari-web-extension://<uuid>` is
likewise not a shared public id. A request from the background may carry it as `Origin`, and that
would work as an installation identifier if anyone stored it. Hence D11.

*Rejected:* an Axum JSON route for the extension. It would mean a second client stack in the
extension, and no `buf` gate on a contract that installed clients cannot update.

### D9. A module in `backend/lingua`, two counter tables in the `lingua` schema

`usage_counts_core.rs` holds the pure rules: validation, bounds, the closed sets, the week check.
`usage_counts.rs` holds the module and the ports it declares — the consumer declares them, as
`plans/src/ports.rs` and `OnlineTranslationPolicy` do:
- `UsageCountsRepo`;
- `UsageCountsPolicy` — the flag and the ceiling, implemented in `backend/server/src/flags.rs` like
  `OnlineTranslationPolicy`;
- `FloodGuard` — see D10.

`usage_counts_grpc.rs` and `pg_usage_counts.rs` are thin glue, excluded from the coverage gate like
`online_translation_grpc.rs` and `pg_online_usage.rs` (`.github/coverage-ignore-regex.txt`). The
migration takes the next free number at implementation time (`0008` is planned by
`add-lingua-card-edit`), named `00NN_lingua_usage_counts.sql`:

```sql
CREATE TABLE IF NOT EXISTS lingua.usage_counts_weekly (
  install_week INTEGER NOT NULL,  -- day number (days since 1970-01-01) of the Monday of day 0's local week
  step         TEXT    NOT NULL,  -- installed | set_up | first_analysis | returned_day_1 | …
  browser      TEXT    NOT NULL,  -- chromium | firefox | safari
  version      TEXT    NOT NULL,  -- major.minor
  count        BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (install_week, step, browser, version)
);
CREATE TABLE IF NOT EXISTS lingua.leave_reasons_weekly (
  week    INTEGER NOT NULL,       -- Monday of the UTC week the answer arrived in
  reason  TEXT    NOT NULL,       -- one of the six fixed reasons (D12)
  browser TEXT    NOT NULL,       -- the three, or unknown
  version TEXT    NOT NULL,       -- major.minor, or unknown
  count   BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (week, reason, browser, version)
);
```

**No column for a time of day, an address, a header or a report.** Counters are upserted with
`count = count + 1 WHERE count < ceiling`.

**Retention: 25 months.** The counters are anonymous aggregates, but the CNIL's figure for
audience-measurement data is 25 months, and a funnel older than two years answers no question. The
module deletes rows whose week is more than 25 months old, at most once a day, from the report path
(an in-memory "last pruned" day; no job, no timer). See Q6.

**Privilege chain.** Writes and the console's reads run on the `lingua_svc` pool
(`CYMBRA_LINGUA_DATABASE_URL`), whose `search_path` is `lingua`, and the queries are
schema-qualified like `pg_online_usage.rs`. Nothing reads another module's schema.

**Inert without its environment.** Without that URL the service and the route are not mounted, like
every Lingua service. The runtime flag `lingua.usage_counts.enabled` (app `lingua`, default on) stops
counting without a release: the server answers OK and records nothing. Its name says what it
switches and nothing else, like `lingua.sync.enabled`. It is administered in the existing flags
console.

*Rejected:* the `analytics` schema. Its model is pseudonymous and authenticated, its tables carry a
bucket, its role is `analytics_svc`, and the Lingua console (admin in `lingua`) would have to read
another module's schema. Reusing it would also tie the change to the still-open
`add-feature-usage-analytics`.

### D10. Abuse: bounded values, an address limiter that writes nothing, a ceiling

An unauthenticated counter can always be inflated by someone determined. The aim is to make that
costly and visible, not impossible.

- **Bounded values.** A step is one of six, a browser one of three, and `version` matches
  `^\d{1,3}\.\d{1,3}$`. A report holds one to six distinct steps. The week must be a Monday
  (`install_week % 7 == 4`); one that is not is `INVALID_ARGUMENT`. A Monday more than 18 weeks
  before the server's UTC day (day 59, and time offline), or later than the day after it (time
  zones ahead of UTC), is dropped and answered OK.
- **Floods from one address.** At most 60 reports an hour per address, enforced by an in-memory
  `FloodGuard` in the server process:
  - **Key.** An IPv4 address, or an IPv6 address's /64 prefix — one home or one phone holds a whole
    /64, so keying full IPv6 addresses would let one machine rotate past the limit. The key is
    hashed with HMAC under a key drawn at start-up and never written.
  - **Bounded.** The table holds at most 50 000 entries (a few megabytes). When it is full, a new
    address is refused with `RESOURCE_EXHAUSTED` until entries expire; the client keeps its steps
    and retries an hour later, so a botnet cannot push the guard open, and an honest reader loses
    nothing.
  - **Forgetting.** An entry is forgotten when its hour ends; a restart forgets everything. The
    server runs as one process (`docker-compose.prod.yml`), so one guard is enough.
  - **What the hash protects.** The table never holds an address in clear, so it can never leak one
    into a log, a panic message or the disk. It does not protect against someone who can read the
    server's memory: the key is there too, and 2³² IPv4 addresses can be tried. That attacker could
    read live requests anyway.
  - One installation sends at most six reports in its whole life, so 60 an hour leaves room for a
    school, an office or a mobile carrier's shared address. The address is resolved with
    `client_addr::resolve` and passed to the guard alone. The platform limiter is not used, because
    it writes its subject into a Redis key.
- **A ceiling per counter.** At most `lingua.usage_counts.ceiling` per (week, step, browser,
  version), 5 000 by default (runtime config, `cfg` in the flag registry). A capped week shows as
  such in the console.

The same flag, guard, ceiling and retention apply to the leaving route (D12). Its bounds are the
reason (one of six), and the browser family and version as above, where `unknown` is also accepted:
the page may be opened without its parameters.

*Rejected:* no limit per address (one script inflates everything); the platform's Redis limiter
(writes the address); keying full IPv6 addresses (trivially rotated); an unbounded table (memory
growth under a spread-out flood); a proof-of-work or a signed token (an anonymous client can still
mint either, at real cost to the code).

### D11. Nothing that identifies a request is kept — the reverse proxy included

- **The handler** reads the message fields and nothing else: no header, no remote address (only the
  flood guard sees the address, as D10 describes). It logs nothing about a report. No metric labels
  go beyond the method that `ObserveLayer` already records.
- **Caddy.** It keeps its no-access-log configuration, and a comment in the Caddyfile says why, so
  nobody adds a `log` directive without seeing it. Its **error** log, which today carries the
  client address and every header on any 5xx (Context), gets a global `log` option whose `filter`
  encoder deletes `request>remote_ip`, `request>client_ip`, `request>remote_port` and
  `request>headers`. That covers every Cymbra route behind Caddy, not only the counts — a plain
  privacy gain: nothing reads those fields today. The message, the status, the method, the path
  and the duration stay, so a 502 can still be diagnosed.
- **Tests.** A server test captures tracing output across a counted report, a dropped one, a refused
  one and a leaving reason, with `x-forwarded-for`, `Origin: moz-extension://…` and `User-Agent` set,
  and asserts that none of their values appears. The Caddy filter is checked on the deployed stack:
  with the server stopped, one request through Caddy, then Caddy's output read for the address and
  the headers (owner task).

### D12. The leaving page

- **Registration.** While counting is on, the background calls
  `chrome.runtime.setUninstallURL(url)` at start-up and whenever the switch or the interface
  language changes:

  | Interface language | Leaving page |
  |---|---|
  | fr | `https://cymbra.app/lingua/au-revoir/` |
  | en | `https://cymbra.app/en/lingua/goodbye/` |
  | es | `https://cymbra.app/es/lingua/adios/` |

  The URL carries `?b=<browser>&v=<major.minor>` and nothing else. With the switch off — or, on
  Chromium, before the reader agreed — the URL is `""`. The call is compiled out of the Safari bundle,
  where it does nothing.
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
  credentials. It delegates to the same module and goes through the same flag, ceiling, retention
  and guard, with the bounds D10 gives it. The answer is filed under the UTC week it arrived in.
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
- `AdminGetUsageFunnel(window, browser)` — for each installation week in the window: installed, set
  up, first analysis and the three returns, and whether a counter hit the ceiling;
- `AdminGetLeaveReasons(window, browser)` — the total for each reason.

Both answer per week and per browser family at the finest; versions are summed. `/lingua/overview`
gains an "Installations" section:
- a table by installation week, with each step as a count and as a share of "installed";
- a browser filter;
- a return cell reads "not yet" until its window has closed for every day of the week (day 2, 14
  and 60 after the week's last day), so a young week never looks like a bad week;
- the leaving reasons as a list of totals;
- a note that these are anonymous counts from installations that count — on Chromium, only those
  whose reader said yes, so Chromium's absolute numbers are not comparable with Firefox's and
  Safari's, while its percentages within a week are — and that reinstalls and several devices each
  count as one.

The store keeps the existing `Async<T>` union; strings are in en and fr. The section reuses the
console's window. Its one chart is the existing series component, plotting installations per week.

### D14. Disclosures and store forms, per browser — every text that changes

| Where | What changes | Evidence |
|---|---|---|
| Privacy policy, Annex B (fr/en/es) | a paragraph on the counts and the leaving page: what is counted, the three things sent with it (week, browser family, version), no identifier, no address kept — **not even in the technical logs of §2**, which otherwise keep IP addresses 7 days —, day 0 kept on the device for at most 60 days, counters deleted after 25 months, the switch, how each browser asks; legal basis legitimate interest (audience measurement) where no consent is asked, objection through the switch | `apps/site/src/pages/confidentialite.md`, `en/privacy.md`, `es/privacidad.md` |
| Privacy policy, general part (fr/en/es) | the Chrome Web Store Limited Use statement | User Data FAQ, *Limited uses*, question 1; Limited Use policy, item 6 |
| Firefox manifest | `technicalAndInteraction` stays in `optional` with `required: ["none"]`, and the comment names the counts | Extension Workshop, built-in consent; Add-on Policies 6.2.1 |
| `REVIEWERS.md` | "Where the add-on reaches the network" names the report, the leaving page, Firefox's consent and the versions below 140/142 that count nothing | — |
| Chrome Web Store privacy practices | "User activity" ticked, with the counts described, and "only after the reader agrees" said; the three certifications unchanged | Q7 |
| Store descriptions | the "no network request" sentence replaced (Q5) in eight texts: `apps/lingua-extension/STORE-LISTING.md` *Description* **FR**, **EN**, **ES with fr-es**, **ES without fr-es** — each pasted in both the Chrome Web Store and AMO dashboards — and `apps/lingua-apple/STORE-LISTING.md` **fr-FR**, **en-US/en-GB**, **es-ES/es-MX with fr-es**, **es-ES/es-MX without fr-es**, each with its stated character count re-measured | the files |
| Claims table | the row "Local analysis; no request without an account or translation" (`apps/lingua-extension/STORE-LISTING.md`) | — |
| App Review notes | the "External services" paragraph of the two live blocks (**with fr-es**, 3976/4000; **without fr-es**, 3935/4000) — "none by default … no analytics" replaced by the counts and the switch — and each block trimmed back under 4000, re-measured. The "As submitted with 1.5.0" block is history and stays | `apps/lingua-apple/STORE-LISTING.md` |
| App Store privacy answers | the Product Interaction row names the anonymous counts too; App Store Connect needs no edit (*What the stores require*, Apple) | `apps/lingua-apple/README.md` |
| Site's Lingua page | « Sans compte, tout reste local » / "Signed out, everything stays local" / « Sin cuenta, todo se queda en tu dispositivo » made precise: your words, deck and statuses stay on the device | `apps/site/src/lib/lingua-text.ts` (the three locales) |
| Welcome tab | « Sans compte, tout reste sur cet appareil » and its en/es equivalents made precise the same way, next to the new sentence | `src/i18n/{fr,en,es}/onboarding.ts` |
| Extension README | "No network request unless the reader signs in or turns on « Traduction étendue »" | `apps/lingua-extension/README.md` |

Reviewed and unchanged: the manifest summaries (`_locales/*/messages.json`, « Hors ligne et privé »
/ "Offline and private"), which stay true; the account page's « Sans compte, l'extension fonctionne
entièrement sur cet appareil », which speaks of the features; the Chrome test-instructions field,
which promises nothing about the network.

### D15. Order

1. **Server.** Migration, service, route, flag, the Caddy log filter. Deployed, then checked from
   outside: a report sent with `buf curl` increments one counter, the 61st report from one address
   within an hour is refused, and a forced 502 leaves no address in Caddy's output.
2. **Site.** Annex, Limited Use statement and leaving page, deployed.
3. **Stores.** Text and forms updated in their consoles, ready to go with the build.
4. **Extension.** The release that sends. An extension that sends before step 1 is deployed loses
   nothing: its reports wait and retry.

The back office ships with step 1.

## Risks / Trade-offs

- **Store review.** The per-browser rules above are read from the stores' own texts; enforcement
  depends on reviewers. Chromium asks, so Chrome's consent rule is met as written. Safari relies on
  the legitimate-interest route that Apple's guideline allows; if App Review objects, the Safari
  variant moves to `ask` with a define (D7).
- **Inflation.** Anyone can add counts within the limits of D10, and a lost answer can count a step
  twice (D5), so the figures are a trend, not an audit. The console says "anonymous counts" and marks
  capped weeks.
- **Undercounting.** Readers who turn counting off, Chromium readers who do not say yes, old Firefox
  versions, Safari readers who never open the popup, and installations made before the release are
  all missing. Percentages within a counted week stay meaningful; absolute numbers are a floor, and
  Chromium's floor is much lower than the others'.
- **One person in a cell.** At today's scale a weekly Firefox cell may hold one installation. No
  identity can be drawn from it alone; the week grain (D2) and the absence of any finer view in the
  console limit what side knowledge can add.
- **Caddy's filter applies to every route.** A future debugging session will not find client
  addresses in Caddy's error log. That is intended; the policy's general "technical logs" sentence
  may be narrowed accordingly in a later change.
- **Device clock.** Day 0 comes from the device clock; a clock far off produces a week the server
  drops.
- **Archive order.** `add-lingua-youtube-captions` has no task done yet; this change's archive waits
  for it, for its planned split, and for `add-lingua-review-translation-fallback` (Q8).

## Migration Plan

New tables only (the next free migration number, idempotent, `IF NOT EXISTS`); no backfill.
Rollback: set `lingua.usage_counts.enabled` off (clients keep working, nothing is recorded), or ship
an extension whose switch defaults off. Dropping the two tables loses only counts. The Caddy filter
is one global option; removing it restores today's error lines.

**TODO before implementation (task 0.2).** The `No network requests` block in
`specs/lingua-browser-extension/spec.md` is written on top of `add-lingua-youtube-captions` as it
stands on `main` on 2026-10-14. That change is being amended and split into
`add-lingua-youtube-captions` (A) and `add-lingua-youtube-caption-translation` (B). Once that lands,
this block is rebased on the amended text — its counts bullet and scenarios added to whatever A and B
say — and both stay in `archiveAfter`.

## Open Questions

Each has a recommended answer; the design above assumes it. Q1 comes first because it changes
something you decided.

- **Q1. Chrome does not allow counting by default. Do you accept asking Chrome readers once?**
  You chose "on by default, with an off switch". Firefox and Safari allow that; Chrome's written
  rules do not. Google says that before an extension collects any data about how someone uses it,
  the extension itself must show what it collects and get a clear "yes" — a button the reader
  presses — and that a store description or privacy policy is not enough. Anonymous counts are
  still "data about how someone uses it" in Google's definition, and Google's text names no
  exception for them. The penalty Google states for a mismatch is the suspension of every extension
  of the publisher. *Recommended, per browser:*
  - **Chrome and other Chromium browsers: ask once.** One sentence on the welcome tab (or in the
    popup) with "Yes, count" and "No, thanks", equal buttons. Nothing is sent until "yes". Expect
    fewer Chrome figures; the percentages stay meaningful.
  - **Firefox: on by default, as you decided.** Firefox's own install window shows a ticked box for
    exactly this kind of data, and Mozilla's rules call it opt-out.
  - **Safari: on by default, as you decided.** Apple allows it when the developer relies on a
    "legitimate interest" under European law and offers an easy way to say no; the privacy policy
    says so and the switch is the way to say no. If Apple's review objects, Safari moves to "ask"
    with one build setting.
- **Q2. A week, not a day, for "when was this installed".** The first draft sent the exact day.
  With a handful of readers, the exact day could let someone who knows when a friend installed
  Lingua see whether that friend came back. The back office shows weeks anyway. *Recommended:* send
  the week only.
- **Q3. Firefox before 140 (desktop) and 142 (Android):** no built-in consent there.
  *Recommended:* count nothing on those versions. Do not build a consent screen of our own. Whether
  to raise `strict_min_version` is a separate decision.
- **Q4. The return windows.** *Recommended:* day 1; days 7–13; days 30–59 (D1). The alternative is
  exact days 1, 7 and 30.
- **Q5. The sentence that replaces "no network request" in the store descriptions.**
  *Recommended* (it holds for every browser, whichever way it asks):
  - fr: « Ce que vous lisez ne quitte jamais votre appareil. Lingua peut envoyer des comptages
    anonymes — installation, premiers pas, retours —, sans aucun identifiant ; un interrupteur dans
    les Réglages les coupe. »
  - en: "What you read never leaves your device. Lingua may send anonymous counts — install, first
    steps, returns — with no identifier of any kind; one switch in Settings turns them off."
  - es: « Lo que lees nunca sale de tu dispositivo. Lingua puede enviar recuentos anónimos
    — instalación, primeros pasos, regresos —, sin ningún identificador; un interruptor en los
    Ajustes los desactiva. »
- **Q6. How long the server keeps the weekly counters.** They hold no identifier, but France's data
  regulator recommends keeping audience-measurement data 25 months at most. *Recommended:* delete
  them after 25 months, automatically.
- **Q7. Chrome privacy practices: tick "User activity"?** The dashboard's description of the
  category is not published in Google's documentation (read 2026-10-14), so the owner reads it in
  the dashboard when filling the form. Counted steps are interaction with the extension.
  *Recommended:* tick it, with the counts described. Declaring too much is safe; a declaration that
  does not match what the extension does can get it removed.
- **Q8. Archive order.** *No network requests* is also modified by `add-lingua-translation-delivery`
  (one task left), `add-lingua-youtube-captions` (no task done; being split into A and B), and
  *Lingua's privacy disclosures match what it collects* by `add-lingua-review-translation-fallback`.
  All four are in `archiveAfter`. *Recommended:* keep that. If YouTube captions are parked for good,
  say so and the requirement will be rebased on translation-delivery's text alone.
- **Q9. Firefox's "technical and interaction data" today claims to cover the signed-in daily stats
  and the sync's installation id, and nothing checks it.** *Recommended:* it gates the counts only.
  The comment is corrected: those synced items belong to the account the reader chose to sign in
  to. Whether declining it should also stop syncing statistics is a separate change, if you want
  one.
- **Q10. The six leaving answers** (D12), with no free text. *Recommended:* as listed; you can reword
  them before the site task starts.
- **Q11. Installations made before the first counting release.** *Recommended:* not counted; the
  funnel starts with that release (D4).
- **Q12. The Limited Use statement is missing from the privacy policy today.** Chrome asks for it
  from every extension that handles user data, which Lingua already does (accounts, sync, online
  translation). *Recommended:* add it now, in its own small change, rather than wait for this one.
