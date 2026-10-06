# refine-lingua-review-session — design

## Context

See proposal.md for the why. The pieces this change reshapes:

- **The session** is `ReviewSession` (`crates/lingua-core/src/decks/review.rs`). `start_for`
  takes every due key of the chosen languages; a mixed-language queue is sorted by due date,
  a single language keeps the deck's order — alphabetical by lemma. The queue is fixed at
  the start: `grade` applies the rating and advances, so a card graded `again` is not seen
  again before its next due date (at least a day: `interval_days` clamps to 1).
- **FSRS** (`fsrs.rs`) implements the long-term path only. `FsrsParams::retrievability`
  exists but is called by `ReviewState::grade` alone, and the same-day weights `w[17]` and
  `w[18]` are unused.
- **The WASM engine** (`crates/lingua-wasm/src/lib.rs`, `LinguaEngine`) holds one session.
  `reviewCurrent` already returns the card's `sentence` and `surface` with its headword and
  gloss; the extension simply does not show the sentence before the reveal.
- **The extension** renders the session with one widget: `renderReview`
  (`src/review/view.ts`), driven by `ReviewController` (`src/review/session.ts`), inside
  `mountReview` (`src/review/review-page.ts`), which also holds backup/restore and the pack's
  sources. Its two hosts, the drawer and the side panel, already call `followSurfaceLook`:
  the theme (dark, light, or the e-ink theme of `add-lingua-colour-settings`) and the
  reader's text scale reach the review with no work here.
- **The review language** (`refine-lingua-review-language`, archived on 2026-10-06): a
  session and its counts hold one studied language — the page's or the book's, else the
  last chosen, else the first — and the filter offers no « Toutes ». `mountReview` follows
  the page through its host's `pageLanguage` and `pageChanged`. The core still accepts
  several languages in one session; nothing asks for that any more.
- **The port** crosses contexts method by method: `analyzer/port.ts` (interface),
  `analyzer/engine.ts` (WASM adapter), `analyzer/messaging-port.ts` (the Firefox and Safari
  proxy to the background's engine), and the test fake in `test/helpers.ts`.
- **Réglages** has four tabs since #660 (Langue, Apparence, Pages & livres, Données), all
  built by `mountSettings`, the one settings builder every host renders.
- **The reader** turns pages by thirds (`turnByZone`, `src/reader/app.ts`): left back,
  right forward. The review's two answers borrow that geography.

## Goals / Non-Goals

**Goals:**
- One queue order for every session, mixed or single-language, decided in the core.
- No schema change, no backup-format change, no sync change: everything new is computed
  from the card state the deck already keeps, or lives in the session.
- One widget for every host; layout chosen by the space the widget gets, not by the device.

**Non-Goals:**
- A four-level grading option (revisit after the e-ink dogfood if the two answers fall short).
- A translated sentence on the back of the card. A following change keeps on the card the
  sentence translation the word popup already shows when the word is captured; a later one
  translates the sentence at review time for older cards, where a translation model is
  installed. The card's answer space leaves room for it then.
- Anything that needs an index of the reader's books ("you will meet these words N times
  further on"): a later change.
- The opening warm-up, review inside the text, the end-of-chapter recap.
- The agent plugin's review (`apps/lingua-agent`, its own store and `answer_card`).
- The FSRS same-day model, parameter optimisation, card deletion.
- Any timer, notification or network call.

## Decisions

### D1 — The queue is ordered by predicted recall, then capped at 10 distinct cards

At the start, the core splits the due keys of the chosen languages into reviewed cards (with
a memory) and never-reviewed cards. Reviewed cards are sorted by retrievability at `now`,
lowest first (ties: due date, then language and lemma). Never-reviewed cards keep capture
order, oldest first. The session takes up to 10 cards: a new card at the 4th and 8th
places when both kinds are due and the day's allowance is not spent; otherwise whichever
kind remains, new cards never beyond the allowance. « Encore 10 » is simply a new
`startReview` over what is still due.

This rewrites the single-language rule of `add-lingua-language-stats-review` D1, which this
change therefore archives after (`archiveAfter`, checked by `scripts/openspec_archive_order.py`).

*Alternatives.* Due-date order (today's mixed queue): it ignores how much was forgotten — an
overdue card with high stability is less at risk than a fresh card with low stability, and
retrievability measures exactly that. A time budget ("five minutes") instead of a count: a
count is visible (« 3 / 10 »), deterministic under test, and does not drift with e-ink
repaint times.

### D2 — The daily allowance is computed from the deck, with the day start from the caller

A card was introduced today when `reps == 1` and `last_review >= day_start`. In-session
returns are not graded (D3), so a card first answered today still has one rep tonight; a
card first answered earlier has at least two reps when it is answered today. No new field,
no counter to keep in step, and a card introduced on another device counts once it has
synced. The extension passes `day_start` as the reader's local midnight in epoch seconds;
the daily statistics keep their UTC days, unchanged.

Each studied language has its own allowance: a card counts against its own language's, and a
session over several languages takes what is left of each one's. A session holds one language
since `refine-lingua-review-language`, so one allowance shared by every language would be
spent by the first language reviewed in a day, and the other language's session would then
bring no new word, with nothing to say why. Each language has its own, as each deck has its
own limit of new cards in Anki. A reader of two languages may thus take up to twice the
setting in a day; the 5 step is there for them, and Réglages says the number counts per
language. Decided by the product owner on 2026-10-06.

The allowance is a reader preference under its own `chrome.storage.local` key (steps 5, 10,
20; absent means 10), in a « Rythme de révision » block of the « Langue » tab — not
« Révision », which `test/lint-settings-hosts.spec.ts` would read as a copy of the drawer's
tab. Like the bar and highlight toggles, it is a comfort setting: never in the engine
backup, never synced.

*Alternatives.* A per-day counter in the store: a second source of truth that a restore or a
sync would contradict. One allowance shared by every language: what the setting means would
depend on which language the reader reviews first.

### D3 — A missed card's return is a learning step, not a review

The session marks a card missed on its first « Pas su », grades it `again` once, and puts
it back three places later (at the end if fewer remain). Later answers in the session do
not touch FSRS: « Su » takes the card out and counts it recovered; « Pas su » puts it back
again, until it has been asked three times. `reviewGrade` returns whether the answer
updated FSRS, and `ReviewController` records a daily « review » only then.

*Why not grade the returns.* With the long-term path only, a second `again` at an elapsed
time near zero applies the forgetting formula a second time to a single lapse; a `good` at
retrievability ≈ 1 leaves stability where it was. Grading would only penalise twice.
Implementing FSRS-5's same-day weights is a separate, larger change.

### D4 — Two answers in the widget; four ratings in the core

« Pas su » sends `again`, « Su » sends `good`. The core, the backup, the sync and the agent
plugin keep the four ratings: nothing in the contract narrows. The reasons are product
reasons: a recall is a yes-or-no fact where Difficile / Correct / Facile ask for a judgement
on every card; « Difficile » is commonly pressed after a failure, yet FSRS counts `hard` as
a pass and schedules the card later; one decision per card is half the taps on a screen
where each tap is a repaint; and D3 needs an unambiguous miss.

*Alternatives.* Four larger buttons: the cost is the decision, not the target size. A
setting for four levels: deferred until dogfooding shows it is missed.

### D5 — « Ne plus me le montrer » is `ReviewSession::ignore`, the twin of `mark_known`

It stamps `Ignored` (milliseconds, for last-write-wins), retires the card the way
`Deck::retire` does (due at the end of time, `updated_at` bumped), and advances. It reaches
other devices through the existing status and card operations, and is undone by « Remettre
à apprendre », as for any ignored word. The card is kept: deletion and its tombstones stay
deferred, as `apply_card_lww` documents. It is not counted as a word learned.

### D6 — The card puts the sentence first and reserves the answer's space

`renderReview` draws, top to bottom: the source label (a site, or a book and its chapter),
the sentence in a reading serif at `--cymbra-lingua-ui-scale`, the answer space, then the
action zone. The encountered form is marked in the sentence, matched case-insensitively at
word boundaries and built from text nodes only (the sentence came from a page). When the
form is not found, the headword is shown above the sentence; a card without a sentence
shows its headword alone, as today.

The answer space has its height from the start and is empty until the reveal, which fills
it with the gloss: on e-ink, revealing repaints that block and nothing moves. The action
zone has the same size before and after: one full-width « Afficher la réponse », then the
two answers side by side, with « Je connais · Ne plus me le montrer » as links below. The
review stylesheet has no transition.

### D7 — The layout follows the space the widget gets (container queries)

The same widget lives in a 380 px side panel, in a drawer up to 92 % of the viewport, and
across a phone's width; a drawer on a tablet can be narrow on a wide screen. The review root
is therefore a CSS container, and the layout keys on its size: below ~600 px wide, the
answers form a bar at the bottom, padded by `env(safe-area-inset-bottom)` — the card
fills the height below the page's header (an estimate, `100dvh - 180px`), so a short card still
puts them under the thumb and a long one keeps them in view (`position: sticky`); when the
viewport is landscape with little height, they become bands along both edges, the card
between them; from ~600 px, the card is a centred column of reading width with the answers
under it. Container queries are supported by every target (Chromium 105, Firefox 110,
Safari 16).

Keys are handled on the review root only, never on the page's `window`: Space or Enter
reveals, the left and right arrows answer. The focus moves into the card when a session
starts, so a drawer over a web page never takes the page's keys.

### D8 — The end of a session reads a summary kept by the core

The session counts as it goes: cards graded, cards recovered (D3), cards whose new due date
is at least 30 days away, cards marked known, cards hidden. `reviewSummary` returns them;
the end view shows them and offers « Encore 10 » only when another session would hold cards.
A due count would not tell: new words past today's allowance are due and still wait. So the
end prepares the next session, in the same language, and measures it; « Encore 10 » starts it
again. Closing stays each host's own control.

### D9 — Backup, restore and sources become two Données blocks

`mountSettings` gains « Fichier de sauvegarde » (Sauvegarder, Restaurer) and « Sources et
confidentialité » in the « Données » tab; `mountReview` drops them. Being in the one settings
builder, they reach the popup, the side panel, the drawer and the Safari app together, and
`test/lint-settings-hosts.spec.ts` keeps any other page from holding a copy. A restore made
in the page that is showing a review ends that session, since the engine drops its session
on restore.

The toolbar popup is the exception for the restore: Firefox closes a popup the moment a file
picker opens, so the change event never arrives. `mountSettings` takes `canPickFiles`; the popup
passes `false` and offers the download only, with a pointer to the panel. Found while
implementing; the spec says so.

### D10 — The engine contract grows, additively

- `startReview(now, languages?, limit?, newPerDay?, dayStart?)`: without the options there
  is no cap and no allowance, so older callers and tests keep their behaviour; the order of
  D1 always applies.
- `reviewGrade(rating, now)` returns whether FSRS was updated.
- New: `reviewIgnore(now)` and `reviewSummary()`.

Each is carried through the four places every port method lives: the WASM binding, the
`engine.ts` adapter, the `messaging-port.ts` proxy with the background's dispatch, and the
test fake.

## Risks / Trade-offs

- [The session opens on the hardest cards and feels like failure] → new words interleaved,
  missed words coming back to a success, 10 cards at most; measured in dogfood.
- [The sentence on the front makes recall easier, so intervals grow longer than for a bare
  word] → accepted: the skill trained is reading in context; the dogfood watches the real
  recall rate against the 90 % target.
- [New words pile up behind the allowance for a heavy reader] → the 20 step; the pile shows
  in the due count and in Stats.
- [A reader of two languages takes in up to twice the setting's new words a day, and reviews
  them later] → the allowance is per language by decision; the 5 step halves it.
- [`env(safe-area-inset-bottom)` is zero inside a drawer over a page without
  `viewport-fit=cover`, so the bar may meet the iPhone home indicator] → a minimum bottom
  padding, checked on an iPhone in the dogfood pass.
- [A restore in one surface while another surface is mid-session] → unchanged by this
  change: that surface keeps its session until it ends, as today.
- [An Anki habit of four buttons] → the core keeps four ratings, so an option costs one
  setting if it is ever wanted.
- [The numbers are first guesses: 10 cards, 10 new words a day, a return three cards later,
  three asks, 30 days for « more than a month »] → they are set for the first e-ink dogfood
  and named in the specs on purpose; changing one after use is a small follow-up change.

## Migration Plan

None. No stored state, backup field or sync message changes; the allowance key is absent
until the reader sets it. Rolling back the extension restores the previous review over the
same data.
