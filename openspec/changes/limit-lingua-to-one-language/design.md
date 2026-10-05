# Design

## Context

R2 and R3 of the Spanish programme taught every layer to hold several studied languages:
- the profile is an ordered list (`Profile.studied_languages`), the first one primary;
- `acceptedLanguages()` (`analyzer/pairs.ts`) is the reader's studied languages that the package
  ships;
- every surface reads that list: the reading session (one accepted language asks for no
  detection), the statistics' selector, the review's filter, the library's sections, the sync's
  pulls and onboarding's levels.

One builder writes the list: `mountStudiedLanguages` (`reading/studied-languages-view.ts`). It is
mounted by Réglages (`settings-view.ts`, every host) and by onboarding. Today it ticks a box per
shipped language; the last box cannot be unticked.

So narrowing to one language is a change to that builder alone. Every reader of the list follows,
because a profile that holds one language gives one accepted language.

## Goals / Non-Goals

**Goals:**
- One studied language at a time, chosen in the settings and at onboarding.
- A change in the settings is confirmed, then possible again 30 days later.
- Nothing is deleted on a change, and the review covers the language studied.
- Neutral wording.

**Non-Goals:**
- Several languages at once. The engine, the profile's list, the sync and R3's per-language
  controls keep supporting them; the controls stay hidden while one language is accepted.
- Syncing the choice or the wait across devices: the language choice is not synced (programme
  decision D9).
- The agent plugin (change 31).
- The deck's total count, which still counts every card.

## Decisions

### D1 — The limit lives in the builder that writes the choice

`mountStudiedLanguages` renders one option per shipped language, as a single choice (radio
buttons), the studied language checked. Choosing writes the profile as that language alone.

The profile keeps its list format, so neither the engine nor the backup changes:
- an English reader's backup stays byte for byte what it was, and the English baseline does not move;
- several languages at once remain a change to this builder, not to the core.

A profile holding several languages exists only in dogfood builds, since no package has shipped
two pairs. It is shown with its first language chosen. It is not rewritten until the reader
chooses.

*Alternative:* trim `acceptedLanguages()` to its first language. Rejected: it would contradict
`add-lingua-language-sync-client`'s requirement (a device accepts the reader's studied languages)
for those profiles, and hide the limit in a reader of the list instead of its writer.

### D2 — A change waits 30 days; onboarding's choice does not count

A change made in the settings records its time in `chrome.storage.local`, under
`cymbra-lingua-language-changed-at`, in epoch milliseconds. That storage holds the extension's
preferences and marks (the HUD toggle, the last-sync time), and every host passes it to Réglages
as `area`. Within 30 days of that time:
- the other options are disabled;
- the block says from which date a change is possible.

A choice made at onboarding records nothing: the builder takes an option for it. So a reader is
never held by their first choice, and a reader who never changed may change at once.

The mark stays outside the backup:
- a restore or a sync never carries it;
- the 30 days are a constant of the builder.

*Alternative:* the time in the engine's profile. Rejected: it would change the backup format and the
S0 baseline for a mark that is the device's, like the other marks.

### D3 — A change is confirmed in place

Choosing another language does not switch at once. The block shows, under the options:
- « Passer à l'espagnol ? Tu pourras changer à nouveau à partir du 4 novembre. »;
- two buttons, « Changer » and « Annuler ».

The checked option stays the studied language until the change is confirmed. The block is mounted
in the popup, the side panel and the in-page drawer, so the confirmation is part of it, not a modal.

### D4 — Nothing is deleted; the review follows the studied language

A change touches the profile only:
- the other language's statuses, cards, declared level and daily statistics stay in the backup;
- the sync keeps pushing them;
- pulls name the accepted language, and a newly accepted language pulls from the start
  (`add-lingua-language-sync-client`, « Widening… »).

Choosing a language again finds its data as it was.

The review page's « Toutes » scope passes no language today: with one accepted language, a session
and its due count take every language the backup holds. After a change, that would keep reviewing
the language left behind. It passes the accepted languages instead. For a reader of English alone,
whose cards are all English, the queue and its order do not change.

### D5 — Wording

All the wording is French, inline in the builder as today:
- the block title, in Réglages: « Langue étudiée »;
- the onboarding heading: « Quelle langue apprends-tu ? »;
- the note: « Tu étudies une langue à la fois. Tes mots, tes cartes et tes statistiques des autres
  langues sont gardés. »;
- while waiting: « Tu pourras changer de langue à partir du 4 novembre. », the date in the reader's
  locale, with no year unless it differs from the current one.

Nothing else is said about the limit.

## Risks / Trade-offs

- **The wait is the device's.** Another device, or the browser's data cleared, starts without one.
  Accepted for now: the choice itself is per device (D9).
- **Two devices, two languages.** A reader may study English on one device and Spanish on another,
  each device accepting its own. Accepted for now, for the same reason.
- **A reader who changes on day 1 waits 30 days.** Mitigations:
  - the confirmation names the date before anything changes;
  - onboarding's choice does not count.
