# add-lingua-comeback-hints — design

## Context

**The badge today.** `apps/lingua-extension/src/background.ts` listens for the reading session's
`stats` message and sets, per tab, `"${pct}%"` on violet (`BADGE_ACTIVE`, `#7c3aed`), `"—"` on
slate (`BADGE_NEUTRAL`) when the page has nothing to count, and `""` when the reader is switched
off. The reading session sends it after every paint (`ReadingSession.pushBadge`,
`src/reading/session.ts`). A tab with no reading session sends nothing, so its badge is empty: on
Chromium (activeTab-first) that is every tab the reader did not analyse; on Firefox and Safari the
reader is injected statically on every web page, so the dash shows on every page that is not in a
studied language, and nothing shows on `about:` pages and the start page. The badge colours are hex
in the background because `chrome.action` takes no CSS token.

**The popup.** `renderStats` (`src/popup/render.ts`) hides `#controls` — which holds « Deck (en
cours) » and « Réviser (N) » — when the tab answers no `getStats`. N is the page's
`dueCount(now, [language])` from the content script (`ReadingSession.statsMessage`). « Réviser »
opens the side panel on Chromium and sends `openDrawer` to the tab on Firefox and Safari
(`openReviewSurface`, `src/popup/popup.ts`), which does nothing on a tab without the reader.

**Due counts.** `Deck::due_count_for(now, languages)` (`crates/lingua-core/src/decks/review.rs`)
counts every card whose `ReviewState::is_due(now)` holds — `due <= now`, or `due` unset, which is
every never-reviewed card. A retired card has `due = i64::MAX`. A session lets in only the day's
allowance of never-reviewed cards per language (`within_allowance`, `NewCardAllowance`,
`Deck::introduced_since`; 10 by default, 5 or 20 in Réglages — `NEW_WORDS_PER_DAY_KEY` in
`chrome.storage.local`). The WASM engine exposes `dueCount` (`crates/lingua-wasm/src/lib.rs`), and
the review page's summary « N carte(s) · M à revoir » uses it (`src/review/review-page.ts`,
`refreshSummary`).

**What it costs to ask from the background.** A `LinguaEngine` is built from a pack
(`WasmAnalyzerPort.build` fetches the default pair's pack, then `restore`s the backup). On
Chromium the background is a service worker that hosts two such engines lazily (the CSP fallback
and the sync engine), so asking `dueCount` there means loading the WASM module, a pack and the
whole backup on a worker the browser stops after ~30 s idle. On Firefox and Safari the event page
already hosts the reading engine. A path without a pack exists: `reprofileBackup` is a standalone WASM
function that parses a backup with `LinguaState::from_backup` and needs no pack.

**Where the data changes.** Every write of the reader's data goes through the background's
`ownedStore`, which calls `announceStoreChange`; a change of `ROOT_KEY` (the backup) is what
schedules a sync today (`onReaderDataChanged`). Answers, captures, sync pulls, restores, a native
language change, an erasure and the Apple app's changes collected by `reviewRelay` all land there.

**The background's wake.** Each start of the background runs the account block at the top level of
`background.ts`: `session.resume()` (`src/state/session.ts` — no request while the stored access
token is usable, a token refresh otherwise) and, for a signed-in reader, `flags.refresh()` (at most
every 5 min, `FLAG_REFRESH_MS`) and `scheduler.onOpen(SURFACE_INTERVAL_MS)` — a sync if none ran in
the last 10 s. A Chromium service worker is stopped after about 30 s without events, so every start
after a quiet spell syncs a signed-in reader. Pages and surfaces also ask for their own syncs
(`requestSync("page")`, 60 s; `requestSync("surface")`, 10 s).

**The review end.** `renderReview` → `ending()` (`src/review/view.ts`) shows « Séance terminée »
and the summary when the session answered at least one card, else « Rien à réviser pour
l'instant », then « Encore 10 » or « S'entraîner quand même ». One `mountReview` serves the
Chromium side panel (`src/sidepanel/panel.ts`), the in-page drawer on Firefox and Safari
(`src/reading/drawer.ts`) and the Cymbra Lingua app's page (`src/app/app.ts`, built with
`__APP__`, `__TARGET__ = "safari"`), which talks to `ViewController.swift` through the `lingua`
message handler (`src/app/bridge.ts`, `AppBridgeMessage` in `LinguaSignIn`).

**Store addresses.** `apps/site/src/lib/stores.ts`: Chrome Web Store
`chromewebstore.google.com/detail/cymbra-lingua/lodgdmkjlbpieomelpdkfaifdbipfncd`, Firefox Add-ons
`addons.mozilla.org/firefox/addon/cymbra-lingua/`, App Store `apps.apple.com/app/id6813053825`. The
extension holds none of them.

**Browser support, checked on 2026-10-13** (MDN's browser-compat-data `webextensions/api/action`,
Chrome's `chrome.action` reference, MDN's `action.setBadgeText`):

| | Chrome (min 116) | Firefox (min 128, desktop and Android) | Safari macOS / iOS |
|---|---|---|---|
| `setBadgeText`, per tab | yes | yes | yes; per-tab text fixed in Safari 18 (Apple forum, FB15231333) |
| `text: null` → the tab inherits the global text | 116 | 109 | 18 |
| tab's own text cleared on navigation | yes (Chromium clears a tab's action values on a cross-document navigation) | yes, documented | not documented |
| `setBadgeBackgroundColor` | yes | yes | "API exists, but has no effect" |
| `setBadgeTextColor` | 110 | 109 | no |
| `setTitle` | yes | yes | yes; on iOS "the title is not visible in the UI" |

Where Safari on iPhone and iPad shows a badge's text is not documented by Apple; it is checked in
the dogfood (task 8.3).

**Store rules, read on 2026-10-13, checked again on 2026-10-14.**
- Chrome Web Store program policies: *Best Practices and Guidelines* forbid "manipulating your
  extension's reviews or ratings"; *Spam and Abuse* forbids "fraudulent or incentivized downloads,
  reviews and ratings" and abusing notifications "by sending spam, ads, promotions". Nothing forbids
  asking. The Chrome Web Store lets a reader rate an item they have installed, from its *Reviews*
  tab (`…/detail/<slug>/<id>/reviews`).
- Firefox Add-ons: the Add-on Policies say nothing of reviews; Extension Workshop's *Create an
  appealing listing* › *Gently ask for a review* says not to make a review a requirement of any
  feature, not to "repeatedly request the user to provide a review", to give the user time to use
  most features first, and not to pre-populate the comment nor select five stars. Rating on AMO needs
  the reader signed in to their Mozilla account.
- Apple App Review Guidelines: 3.2.2(x) forbids forcing a rating "to access functionality, content,
  or use of the app"; the introduction to section 3 forbids "incentivized, filtered, or fake
  feedback"; 5.6.1 says "Use the provided API to prompt users to review your app … we will disallow
  custom review prompts." Apple's *Requesting App Store reviews* asks for the prompt "at the end of a
  sequence of events that they successfully complete", never at launch nor "as the result of a user
  action"; the system shows it at most three times in 365 days; its sample asks after four completed
  processes, once per app version, two seconds after the end shows, and opens
  `https://apps.apple.com/app/id…?action=write-review` for a review the reader starts from the
  interface. `AppStore.requestReview(in:)` exists from iOS 16 (a `UIWindowScene`) and macOS 13 (an
  `NSViewController`); `SKStoreReviewController.requestReview()` covers macOS 10.14 and later.
- Asking only the readers who seem satisfied is what Apple calls "filtered" feedback and what the
  Chrome Web Store's rule on manipulating ratings covers: that is why nothing here depends on how a
  session went or on a question asked first.

## Goals / Non-Goals

**Goals:**
- A reader away from Lingua sees, on the icon they already have, that cards wait — and how many a
  review would actually give them today.
- The first ratings, asked for rarely — once, and once more only if the reader said « Plus tard » or
  did not answer — at the moment a session ends, within each store's rules.
- Nothing new leaves the device; nothing wakes the extension on a schedule.
- The synchronisation keeps its current triggers: this change adds none and removes none, though on
  Chromium one of them — the background's start — now happens more often (D9, *Follow-up*).

**Non-Goals:**
- Notifications, alarms, timers, `idle` detection, streaks, any badge or count on the page while
  reading — the owner's standing non-goals
  (`openspec/changes/archive/2026-10-12-refine-lingua-review-session/design.md`).
- A rating request in the popup, the word card, the page bar or the book reader.
- A store listing on Microsoft Edge Add-ons (Edge readers install from the Chrome Web Store).
- Counting, on any server, who was asked or who rated.

## Decisions

### D1 — One figure: the cards a review would hold today

`lingua-core` gains `Deck::reviewable_count_for(now, languages, allowance: Option<NewCardAllowance>)`:
the reviewed cards due at `now` plus, per language, the never-reviewed cards due within that
language's allowance left today — exactly the length of the queue `ReviewSession::start_with`
builds with no `limit` (a test holds the two equal). The engine exposes it as
`reviewableCount(now, languages, newPerDay, dayStart)`. The badge, the popup's « Réviser (N) » and
the review page's « M à revoir » all show it.

*Rejected.* Keep `dueCount`: a reader who captures many words sees a figure that never reaches 0 —
« 99+ » for weeks on the icon — and the popup promises 60 when a session gives 10. Cap at 10 (one
session): hides how much is waiting; « Encore 10 » already exists for that.

### D2 — Worked out once per data change, read on every event

The count changes in two ways: when the data changes, and when time passes (a card comes due, a new
day refills the allowance). So the background computes, after each change of the backup, a small
**outlook** per studied language, and the events only read it:

- `due`: the reviewed cards due at the time of computing;
- `upcoming`: the due times of the next reviewed cards to come due, ascending — at most 100, enough
  to tell 99 from more;
- `fresh`: the never-reviewed cards (due, not retired);
- `introduced`: the cards first answered since the local midnight of `dayStart`;
- `cards`: every card of the language — the « Deck (en cours) » the popup shows away from a page
  (D8; added during implementation, a count like the others).

Count at `t` = `due` + `upcoming` ≤ `t` + min(`fresh`, `perDay` − (`introduced` if `t` is still on
`dayStart`'s day, else 0)). The outlook is computed by `reviewOutlook(backup, now, dayStart)`, a
standalone WASM function that needs no pack — the deck read with `LinguaState::from_backup`,
counted in the core next to `reviewable_count_for`, as `reprofileBackup` reads a backup — and kept
in `chrome.storage.local` under one key with its version. It holds counts and timestamps only: no
word, no page. The popup reads the same key. Reading it repeats one line of the allowance rule (the
smaller of the new words waiting and the allowance left); a test checks the outlook's count at
several times — before and after upcoming due times, and on the next day — against the engine's
`reviewableCount` on the same deck, so the two cannot drift apart unnoticed.

The computation runs after every write that touches `ROOT_KEY` (hooked where `announceStoreChange`
already calls `onReaderDataChanged`), without a timer: a write arriving while one runs marks it
dirty, and one more run follows. It also runs at the background's start when the key is missing or
of another version, and at an event that finds a language's `upcoming` list used up (it held 100
times and the last has passed): the 101st card may then be due, and only a new outlook knows — rare,
since any answer or sync recomputes it first. It does not run while the setting is off.

*Rejected.* Ask a hydrated engine on each event: on Chromium every tab switch after 30 s idle would
load the WASM module, a pack and the whole backup. Count in TypeScript from the backup JSON: a
second reading of the engine's private card format (due dates, repetitions, retirement), which would
drift with it.
Compute in each surface that writes: there are five writers (pages, panel, drawer, sync, the Apple
relay), and only the background sees them all.

### D3 — Events, never a timer

The badge is re-evaluated (outlook read, count worked out, action API touched only when the text,
colour or title differ from what was last applied, kept in `chrome.storage.session`):

- when an outlook is written (an answer, a capture, a sync applied, a restore, a native language
  change, an erasure, a review made in the Cymbra Lingua app);
- `runtime.onStartup` and `runtime.onInstalled` (browser start, install, update);
- `tabs.onActivated` (another tab) and `windows.onFocusChanged` to a window (back to the browser —
  `chrome.windows` is guarded, Firefox for Android has none);
- a change of the daily allowance, of the setting, of the colour preset (`chrome.storage.onChanged`);
- any other message that already wakes the background (a page's `stats`, a surface's `sync`).

**The count can be behind, by design.** Between two events nothing runs, so the badge shows the
count as of the last event:
- *Too low*: a card that comes due at 14:00 while the reader is away shows at the next of these
  events, not at 14:00; the day's new words show at the first event after midnight.
- *Too high*: cards the reader answered on another device (or in the app) still count until this
  browser applies them — at its next sync, which a surface opening, a page loading, the browser
  starting or, on Chromium, a wake of the background after a quiet spell already triggers (D9).

That is the price of "no timer", and the events above are frequent while the browser is in use;
opening the popup is itself one of them (it asks for a sync), and the review counts from the engine,
never from the outlook (D1). None of the events
needs a permission (`tabs.onActivated` gives ids only), so the update asks for no new consent. Not
used: `alarms`, `idle`, `setInterval`, `tabs.onUpdated` (it fires several times per page load and
for background tabs; a navigation already clears a tab's own badge — D4).

### D4 — The global badge says the count, a tab's own badge says its percentage

- The **global** badge (no `tabId`) carries the count, its colour and its title; it shows on every
  tab that has nothing of its own.
- A tab whose page reports a percentage gets its own text `"94%"` on violet, as today.
- A tab whose page reports nothing to count, or whose reader is switched off, gets
  `setBadgeText({ tabId, text: null })`: it inherits the global count. Its colour is set to the
  count's colour explicitly, Chrome having no `null` colour.
- A navigation clears a tab's own values, so a percentage never outlives its page (Firefox
  documents it; Chromium clears a tab's action values on a cross-document navigation — checked in
  task 8.2, with a reset on `tabs.onUpdated` `loading` as the fallback if it does not).

The dash « — » goes: the badge shows one thing at a time, and the popup already says why a page
has no figure (« Pas de texte anglais détecté sur cette page. »).

*Rejected.* A per-tab count kept by the background on every tab: the service worker loses any map
of tabs when it stops, and rewriting every tab's badge on each change wakes it more. Keeping the
dash when nothing is due: either the dash on every tab of the browser (the global text), or a
per-tab state as above.

**Safari before 18 shows no count.** There `text: null` is refused and per-tab texts behave as one
text for every tab, so a global count and per-tab percentages would overwrite each other and the
badge would flip between "12" and "94%" from tab to tab. On those versions the count is not shown and
its setting is not offered; the badge keeps today's behaviour (the last percentage set). The version
is read once from the user agent's `Version/` in the Safari build only (a pure, unit-tested
function); the popup's « Réviser (N) » remains the hint there. Safari 18 comes with iOS 18 and is
offered on macOS 13 and later; the app's minimum (iOS 17.2, macOS 12) stays where it is.

*Rejected for Safari before 18.* Showing the count anyway: the reader could not tell which figure the
icon carries, and a percentage would sit on pages it does not describe.

### D5 — Telling a count from a percentage

- **Text**: the number, no « % »; nothing at 0; « 99+ » above 99 (four characters fit; « 100% »
  already uses them).
- **Colour** where the browser lets the extension choose: teal `#0f766e` (the light sheet's
  `--cymbra-lingua-teal`, white text 5.5:1) for the count, violet for the percentage. On the
  `eink-mono` and `eink-colour` presets (`src/reading/colours.ts`), both are black
  (`--cymbra-lingua-eink-black`) with white text: a monochrome panel renders violet and teal as two
  similar greys, and the « % » is what tells them apart. Safari draws its own badge colour; there the
  « % » alone does.
- **Title**: « Cymbra Lingua — 12 carte(s) à revoir », or, with several languages holding cards,
  « Cymbra Lingua — à revoir : anglais 8, espagnol 4 ». Restored to the extension's name at 0 or
  when off.

*Rejected.* A glyph before the number (« ↻12 »): « ↻99+ » no longer fits, and glyphs render
differently per platform. Amber or coral: they mean "learning" and "unknown" on the page.

### D6 — Every studied language, summed

The badge counts all the reader's studied languages together; the title names each. The icon is a
sign that something waits, not a session: a reader of English and Spanish who last reviewed
Spanish must still learn that 30 English cards wait. The popup and the review keep their rule —
one language at a time (*A review is in the language being read*).

*Rejected.* The language the review last opened in: consistent with the popup's figure, but it
hides a whole language's backlog, which is what the hint is for. Most readers study one language,
for whom the two are the same.

### D7 — The switch

In Réglages › Langue › « Rythme de révision », under « Nouveaux mots par jour »: « Afficher les
cartes à réviser sur l'icône », with a note saying it counts what a review would hold today, on the
pages where the icon shows no percentage, computed on this device. On by default, for new and
existing readers; a device preference in `chrome.storage.local`, not synced, not in the backup file.
Off: no outlook is computed, the global badge and title are cleared, page percentages stay. Not
offered where the count cannot show: Safari before 18 (D4), and iOS and iPadOS if task 8.3 finds
that Safari shows no badge text there.

### D8 — The popup offers the review on every tab

« Deck (en cours) » and « Réviser (N) » move out of `#controls` into a block shown on every tab,
highlighting on or off. On an analysed tab N is the page's language's count, from the page; away
from one, it is the count of the language the review would open in away from a page (the last one
chosen in the review on this device, else the first studied), from the outlook — the popup needs no
engine. Where no outlook is kept — the count turned off (D7), or none worked out yet — the popup
works one out itself with the same standalone function over the stored backup, still with no engine
and no pack. « Réviser » opens the side panel on Chromium; on Firefox and Safari it opens the drawer when
the tab has the reader, and otherwise `sidepanel.html` in a tab, as the background's `openPanel`
already does when the side panel cannot open.

### D9 — The background's start is left as it is

With D3, a tab switch or a window focus can start the background where nothing started it before —
on Chromium, whose service worker stops after about 30 s without events. Each start already resumes
the session and, for a signed-in reader, syncs if none ran in the last 10 s and refreshes the flags
(*The background's wake*). So a signed-in Chromium reader who comes back to the browser after a
quiet spell now gets one exchange with the server at the first tab switch or window focus, where
today it comes with the next page analysed or surface opened. A signed-out reader — the majority —
gets none: signed out, the background makes no request.

This change keeps that start as it is. It is what keeps the *too high* count of D3 short: a review
made on the phone reaches this browser, and its badge, at the first tab switch after the reader
comes back. How many exchanges it adds is measured in the dogfood (task 8.2: exchanges in the network
log over ten minutes of ordinary browsing, signed in, on Chrome, against the same ten minutes on the
previous version) and written in the pull request.

*Rejected for this change.* Move the start-time sync and flag refresh to `runtime.onStartup` and
`runtime.onInstalled` (the first version of this proposal): it changes when every reader's data is
pulled, not only the badge's — a wake from a content script's store request or a `stats` message no
longer pulls, a page load waits for its own 60 s throttle instead of the start's 10 s, the count
stays too high after a review on another device until a page or surface asks, and Safari's
`runtime.onStartup` on iOS, where the system unloads the background at will, was not checked. It
did not remove every request either: `session.resume()` refreshes an expired access token on any
start. If the measurement in task 8.2 shows too many exchanges, a change of its own revisits the
start-time sync (*Follow-up*).

### D10 — The rating request

- **Where**: the end of a session that shows « Séance terminée » (a session that answered at least
  one card, practice included), in the side panel and the in-page drawer, under the summary and
  below « Encore 10 » or « S'entraîner quand même », which stay first. Never at « Rien à réviser
  pour l'instant », never in the popup, the word card, the page bar or the book reader.
- **When**: the fifth finished session, on the third distinct day of review or later — Apple's
  sample waits for four completions; Firefox's guidance asks for time with the add-on first; a
  single evening's binge does not qualify.
- **How often**: once shown, the request counts as made. « Plus tard », or leaving without
  answering, asks again after ten more finished sessions; after the second showing, never again.
  « Noter Cymbra Lingua » and « Ne plus demander » end it.
- **What it says**: « Une note sur la boutique de ton navigateur aide d'autres lecteurs à trouver
  Cymbra Lingua. » and three buttons, « Noter Cymbra Lingua », « Plus tard », « Ne plus demander ».
  No stars, no pre-filled text, nothing offered, no question about satisfaction first, no condition
  on the session's results — any of which reads as gating or incentive to the stores.
- **Where it leads**: the store of the build — Chromium: the listing's reviews
  (`…/lodgdmkjlbpieomelpdkfaifdbipfncd/reviews`); Firefox: the listing, whose « Rate your
  experience » section takes the rating (AMO redirects to the reader's locale) — opened by the
  background (`openPage`), so no `Referer` carries the page being read, from the drawer as from the
  panel.
- **What it keeps**: one device preference in `chrome.storage.local` — finished sessions, distinct
  days, the last day, times shown, the session count at the last showing, the answer. Not synced
  (each browser has its own store, and one rating per store is the point), not in the backup file,
  not erased by « Effacer mes données Lingua » (it describes no reading).
- A pure module decides (`ratingDue(state, day)`, `afterSession`, `answer`), the review page counts
  and asks, the view renders; no network request is made until the reader presses « Noter ».

*Rejected.* A star widget in the end screen (pre-selecting a rating is what Firefox's guidance
forbids, and it invites gating). Asking after a session that went well only (sentiment gating).
Syncing the state (the same reader on Chrome and Firefox rates two different listings).

### D11 — Safari and the Cymbra Lingua app

- **The app** uses StoreKit's own request: the page, built with `__APP__`, shows no card and posts
  `review.sessionFinished` at each « Séance terminée »; `ViewController` hands it to a
  `ReviewRequestPolicy` in `LinguaSignIn` (counts in the app's own `UserDefaults`, not the App
  Group), which asks from the fifth finished session in the app, at most once per
  `CFBundleShortVersionString`; the request is made two seconds after the end shows, as Apple's
  sample does, through `AppStore.requestReview(in:)` (a `UIWindowScene` on iOS, the view
  controller on macOS), falling back to `SKStoreReviewController.requestReview()` where the
  platform's minimum (macOS 12) lacks it. The system decides whether it shows (three times a year
  at most; never when the reader turned review requests off).
- **The Safari extension** shows no request at the end of a session. It ships inside the app that
  App Review examines, and 5.6.1 disallows custom review prompts; an extension cannot present
  StoreKit's prompt.
- **A plain link** stays possible: Apple's own sample opens `action=write-review` from an interface
  action, which is not a prompt. Réglages › Données carry « Noter Cymbra Lingua » in every browser
  (D13); in Safari it opens `https://apps.apple.com/app/id6813053825?action=write-review`.

*Rejected.* The Chrome/Firefox card in Safari, linking to the App Store: a reviewer exercising the
extension meets a custom review prompt, and the rejection would hold back the app's release.
Nothing at all on Apple: the App Store is where Safari readers decide.

### D12 — The store addresses in the extension

`src/state/store-links.ts` holds the three listing addresses and the review page of each build
target; a vitest reads `apps/site/src/lib/stores.ts` and checks the Chrome Web Store id, the AMO
slug and the App Store id are the same in both, so a listing moved on the site fails the
extension's tests.

*Rejected.* Import the site's module: the extension's build does not reach `apps/site`, and a shared
package for three strings costs more than the test.

### D13 — A link to rate in Réglages

Réglages › Données gain, under « Sources et confidentialité », a line « Noter Cymbra Lingua » with
« Sur la boutique où tu as installé Cymbra Lingua. », opening the build's store page through the
background. Always there, in every browser: a reader who answered « Ne plus demander » or who
reviews only in Safari can still rate.

## Risks / Trade-offs

- [More starts of the background on Chromium, and with them more syncs] → each tab switch or window
  focus after about 30 s of quiet starts the service worker; for a signed-in reader that start runs
  the existing start-time sync (D9). Measured in task 8.2 — the time from a tab switch to the badge
  on a cold worker, and the exchanges over ten minutes of browsing against the previous version —
  and recorded in the pull request; too many → the *Follow-up*.
- [The outlook's cost on a large deck] → `from_backup` reads the whole backup after each answer.
  Measured on a 2,000-card backup in task 3.1; if it exceeds 50 ms on the Boox (Firefox for
  Android), the run is coalesced per session end instead of per answer.
- [A count behind the truth] → by design (D3): too low until the next event after a card comes due,
  too high until the next sync after a review on another device. The next tab switch, window focus
  or popup opening corrects it; the review itself always counts from the engine.
- [Safari before 18, iOS] → per-tab badges behave as one there and `null` is refused: no count is
  shown on those versions (D4). iOS may not show badge text at all; then the switch is hidden on iOS
  and iPadOS (task 8.3), and the popup's « Réviser (N) » is the hint there.
- [The Chrome Web Store's reviews address changes] → pinned by the drift test only for the id; the
  `/reviews` suffix is clicked through in the dogfood.
- [A rating request read as a nag] → at most twice, ever, after real use, below the session's own
  actions, with a permanent « Ne plus demander ».
- [Edge, Brave, Opera readers] → they installed from the Chrome Web Store and land on it, which is
  right.

## Migration Plan

Nothing to migrate: the outlook is computed at the first start after the update; the switch and the
request's state start at their defaults. Rollback is the previous version: the keys it leaves in
`chrome.storage.local` are ignored (and added to `dropRetiredKeys` if a later version retires them).

## Open Questions

**Answered by the owner on 2026-10-14: every question below takes its recommended answer.** The
count is on by default with its switch (1); summed across the studied languages, the title naming
each (2); the dash « — » goes (3); the count shows where highlighting is off (4); one figure
everywhere — the popup's « Réviser (N) » and the review's summary count what a review would hold
today (5); the rating request comes after the fifth finished session on the third distinct day or
later, « Plus tard » asks again after ten more sessions, at most twice (6); Safari shows no request
in the extension, the app uses StoreKit, Réglages carry a plain write-review link (7); the request's
state is not synced (8); the background's start-time sync stays as it is (9, D9).

1. **Is the count on by default, for everyone?** *Recommended: yes*, for new and existing readers,
   with the switch in « Rythme de révision ». Off by default, almost nobody would discover it, and
   the icon already carries a figure.
2. **All studied languages summed, or only the language last reviewed?** *Recommended: summed*, the
   title naming each language (D6). The popup and the review keep one language at a time.
3. **Does the dash « — » go?** *Recommended: yes* — on a page with nothing to count the icon shows
   the cards waiting, or nothing (D4).
4. **On a tab where highlighting is switched off, does the icon still count?** *Recommended: yes* —
   the switch turns off highlighting, not the review; the count has its own switch.
5. **The popup's « Réviser (N) » and the review's « M à revoir » become "what a review would hold
   today" (new words beyond today's allowance no longer counted).** A reader who captured 60 words
   and reviewed none sees « Réviser (10) » where they see « Réviser (60) » today. *Recommended: yes*,
   one figure everywhere (D1): otherwise the icon says 10 and the popup, one click later, says 60.
6. **Which session asks for a rating, and how many times?** You asked for a one-time request; this
   proposal shows it once, and a second time only to a reader who pressed « Plus tard » or closed
   the review without answering — never a third time. *Recommended: the fifth finished session, on
   the third day of review or later; once more after ten further sessions if « Plus tard »; twice at
   most* (D10). A reader who presses « Plus tard » has asked to be asked again, and Firefox's
   guidance forbids asking *repeatedly*, not twice. If you want it strictly once, « Plus tard »
   becomes « Pas maintenant » and the request never comes back.
7. **Safari: no request in the extension, StoreKit's prompt in the app, a plain link in Réglages?**
   *Recommended: yes* (D11, D13).
8. **Does the request's state sync across devices?** *Recommended: no* — each browser rates its own
   listing; the cost is that a reader of Chrome and Firefox may be asked once in each.
9. **May a tab switch, once Chrome has been quiet for half a minute, start a sync for a signed-in
   reader?** It does so because the icon now listens to tab switches, and the extension's start
   already syncs. *Recommended: yes, for now* — it is also what clears the count after a review made
   on the phone — and measure it in the dogfood (task 8.2). If the extra exchanges are too many, a
   separate change decides when the extension's start syncs (*Follow-up*); this one does not.

## Follow-up (not in this change)

- **When the background's start syncs.** Today every start of the background syncs a signed-in
  reader if no sync ran in the last 10 s, although the comment above that code speaks of a minute
  (`background.ts`, `scheduler.onOpen(SURFACE_INTERVAL_MS)`). If task 8.2 measures too many
  exchanges from the icon's wakes, a change of its own chooses between moving that sync to the
  browser's start (`runtime.onStartup`), raising its throttle to `PAGE_INTERVAL_MS`, or leaving it —
  with Safari's `runtime.onStartup` on iOS checked first, and the effect on a review made on another
  device stated. It touches *when every reader's data is pulled*, which is the synchronisation's
  business (`lingua-sync`), not the icon's.
