# limit-lingua-to-one-language — one studied language at a time

## Why

The settings let a reader study several languages at once: « Langues étudiées » ticks a box per
language the package ships (`add-lingua-language-choice`). The product owner decided that a reader
studies one language at a time for now, and may change it at most once every 30 days.

Nobody has studied two languages yet: every package ships en-fr alone, so the choice is hidden.
That ends with `enable-lingua-spanish` (change 28), which ships es-fr. This change settles the
choice first, so the first package carrying Spanish already offers one language at a time.

## What Changes

- **One language at a time.** « Langue étudiée » offers the languages the package ships as a single
  choice, in the settings and at onboarding, from the same builder. The profile holds the chosen
  language alone, and every surface follows it as today.
- **A change waits 30 days.** A change in the settings is confirmed first, with the date from which
  the reader may change again. Until then, the other languages show as unavailable, with that
  date. A choice made at onboarding starts no wait. The wait is kept on the device, outside the
  backup.
- **Nothing is deleted.** The statuses, cards, levels and daily statistics of a language no longer
  studied stay in the backup and on the server. They come back when the reader studies that
  language again.
- **The review follows the language studied.** With no language filtered, a review session and its
  count of due cards cover the reader's studied language. Today they cover every language the
  backup holds. The cards of a language no longer studied wait, unchanged.
- **The wording is neutral.** It says one language at a time, the date of the next change, and that
  the other languages' data is kept.

Unchanged: the engine, the profile's format and the backup, the sync, and the per-language
controls of R3. Those controls stay hidden while one language is accepted.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: « The reader chooses the languages they study » becomes a single
  choice, with a confirmed change at most once every 30 days and nothing deleted.
- `lingua-decks-review`: a requirement is added. The cards of a language the reader no longer
  studies wait outside the review and its counts.

## Impact

- **Products:**
  - Cymbra Lingua (the browser extension, and Safari through the same bundle) gets the new choice.
  - Nothing is consumed from ID or the platform.
  - The backend, the back office and the site are unchanged.
- **Code:**
  - `apps/lingua-extension/src/reading/studied-languages-view.ts`: the single choice, the
    confirmation and the wait;
  - `reading/settings-view.ts` and `onboarding/onboarding.ts` mount it;
  - `review/review-page.ts`: the review's language scope;
  - their tests.
- **Data:**
  - one device-local mark in `chrome.storage.local`, the time of the last change;
  - no backup or engine change, so the English baseline does not move.
- **Order:** this change merges before `enable-lingua-spanish`. Until a package ships two pairs,
  nothing a reader sees changes.
