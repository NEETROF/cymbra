## Why

Today you cannot tell whether Cymbra Lingua works for the people who install it. The stores give
you a weekly install figure and, on Firefox, a daily-user figure (one, at the last look); the back
office's Lingua console counts only accounts that sync (`apps/back-office/src/views/LinguaView.vue`
says so on screen), and most readers never create an account. Between "someone clicked Install"
and "someone syncs a deck" you see nothing: not whether they chose a level, not whether a single
page was ever highlighted, not whether anyone came back the next day. Without that, every growth
idea — a better store page, a clearer welcome tab, a reminder of due cards — is a guess you cannot
check afterwards.

You decided on 2026-10-13 what to count and what never to count: six anonymous counts, on by
default, with an off switch in the extension's Settings — installed, setup finished, first page
analysed, came back the next day, came back at day 7, came back at day 30 — and **no identifier of
any kind** (no installation id, no device id, no account id even when signed in, no hash of one),
and nothing about what the reader reads. The goal is a sentence you can read in the back office:
« 40 installations this week, 25 analysed a page, 8 came back at D+7 ».

You also want to know why people leave. Chrome and Firefox can open a page of your choice when an
extension is removed; Lingua does not use it today (`runtime.setUninstallURL` appears nowhere in
`apps/lingua-extension/src`).

This change does that while keeping Lingua's promise that what a reader reads never leaves the
device. It does break one sentence you publish today: every store description says that, without
an account or extended translation, Lingua makes no network request at all
(`apps/lingua-extension/STORE-LISTING.md`, `apps/lingua-apple/STORE-LISTING.md`, in French, English
and Spanish). With counts on by default that sentence becomes false, so it is rewritten here, before
the first build that sends.

## What Changes

- **Six counts per installation, each sent once.** Installed; set up (a first level chosen, the
  only "setup done" every browser shares); first page analysed in a studied language; back on the
  next day; back during days 7–13; back during days 30–59. "Back" means a page analysed in a studied
  language or a Lingua surface opened. Each count is filed under the day the installation was made,
  so the back office can say how many of one week's installations reached each step.
- **No identifier, nothing read.** A report carries the installation's day, the browser family
  (Chromium, Firefox, Safari), the extension's version (major.minor) and the steps it reports —
  nothing else. It never carries the account's token, even for a signed-in reader, and the server
  service is mounted with no authentication at all, so it could not resolve an account if it tried.
  The device keeps the installation's day for at most 60 days, then forgets it.
- **On by default, one switch to turn it off** — « Comptages anonymes » in Settings → Data
  (fr/en/es). Off, the device forgets what it had not sent and stops for good for this installation.
  On Firefox the switch is Firefox's own "technical and interaction data" consent, which Firefox
  shows ticked at install and lets the reader untick.
- **Told before anything is sent.** One plain sentence on the welcome tab and in the popup the
  first time it opens, linking to the switch and the privacy policy. Nothing during reading: no
  notification, no badge, no timer.
- **A leaving page on the site.** On Chrome and Firefox, while counting is on, removing Lingua opens
  `cymbra.app/lingua/au-revoir/` (or `/en/lingua/goodbye/`, `/es/lingua/adios/`) with one question,
  "Why are you leaving?", and six fixed answers. Nothing is sent until the reader taps one; the
  answer is counted the same anonymous way. No free-text box. Safari ignores this API.
- **Counters only on the server.** A new unauthenticated `UsageCountsService.Report` in
  `backend/lingua` adds one to day counters in the `lingua` schema; the leaving page posts to a
  JSON route under `/web/` that does the same for reasons. No row per report, no IP address, no
  header kept. Values are bounded, floods from one address are refused without writing the address
  anywhere, and a runtime flag stops collection server-side without a release.
- **A funnel in the back office.** The Lingua console gains a section: installations by week and
  browser, how many reached each step, and why readers left. Aggregates only, like the rest of the
  console.
- **Disclosures.** The privacy policy's Lingua annex (fr/en/es) describes the counts, the leaving
  page and the switch, and the policy gains the Chrome Web Store "Limited Use" statement it lacks
  today. Store descriptions, Chrome's privacy-practices answers, Firefox reviewer notes, the App
  Review notes and the recorded App Store privacy answers are brought in line.
- **BREAKING (promise).** "No network request without an account or extended translation"
  becomes "nothing you read ever leaves your device; by default Lingua sends anonymous counts you
  can turn off". The `lingua-browser-extension` and `lingua-sync` requirements that promise no
  request for a signed-out reader are amended to name this one exception.

## Capabilities

### New Capabilities

- `lingua-usage-counts`: the six counts, what a report may carry, the switch, when reports are
  sent, the server's counters and abuse limits, and the leaving page's registration and count.
  No existing capability covers it: `lingua-stats` is the reader's own statistics, `lingua-sync`
  the signed-in account's data, and `feature-usage-analytics` (still in the open
  `add-feature-usage-analytics`) is Music's, authenticated and keyed by a pseudonymous user bucket —
  exactly what the "no identifier" rule excludes (design D2).

### Modified Capabilities

- `lingua-browser-extension`: **No network requests** names the anonymous counts as the one request
  made by default (written on top of the open `add-lingua-youtube-captions` text, itself on top of
  `add-lingua-translation-delivery`).
- `lingua-sync`: **Optional account, local-first preserved** and **Synchronisation keeps up with the
  reader** say "no synchronisation request" where they said "no request".
- `lingua-privacy`: **Lingua's privacy disclosures match what it collects** adds the counts, the
  leaving page, the switch and the Limited Use statement, and extends the store-listing rule to the
  "no request" sentence.
- `admin-lingua-console`: ADDED — the installation funnel and the leaving reasons.
- `site-lingua-page`: ADDED — the leaving page.

## Impact

**Products.**
- **Cymbra Lingua** (the extension's Chromium, Firefox and Safari variants; the Apple host app is
  untouched) — new: the counting state, the report client, the switch, the first-run sentence, the
  leaving-page registration, copy in fr/en/es. Consumed: the background's existing wake-ups
  (`sync:request`, the `stats` message, `onInstalled`), the gRPC-web transport (without its bearer).
- **Backend (`backend/lingua`, `backend/server`)** — new: a proto, a module with its core, port,
  gRPC service and Postgres adapter, one migration, one JSON route, one runtime flag and one config
  key. Consumed: the `lingua_svc` pool, the flag registry, `client_addr::resolve`. No Caddy change:
  gRPC paths already fall through to tonic and `/web/*` is already on the `@http` allow-list.
- **Site (`apps/site`)** — new: the leaving page in fr/en/es and its route pins; the privacy
  annex in fr/en/es.
- **Back office** — new: a funnel section in `/lingua/overview`, strings in en/fr.
- **ID, Music, Live** — untouched. `backend/analytics` is not used.

**Contract.** A new `backend/lingua/proto/usage_counts.proto` and two new RPCs in
`lingua_admin.proto`; nothing existing changes shape, so no installed client breaks. Both protos
join `backend/lingua/tests/privacy_allow_list.rs`.

**Order.** Server deployed and checked from outside → site (annex + leaving page) deployed →
store forms and listing text updated → the first extension release that sends.

**Size.** About 13.5 ideal days (tasks.md).
