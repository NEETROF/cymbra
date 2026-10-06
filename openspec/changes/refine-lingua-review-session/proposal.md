# refine-lingua-review-session — a short, honest review that reads like the page it came from

## Why

The deck fills while reading, but reviewing it is a chore the reader puts off. A session
takes every due card at once (« 142 carte(s) à revoir »), in alphabetical order when the
reader studies one language. The front of a card is the bare headword: the sentence where
the word was met appears only with the answer, small and grey. Every card asks for one of
five buttons. A missed word is gone until the next day, so a session often ends on a
failure, and it ends with « Rien à réviser pour l'instant. » without saying what it did.

On an e-ink reader — the reader's main device for books — every tap is a repaint and
greys wash out. On a phone the answers must sit under the thumb. The review has to be
short, honest about what the reader recalled, and look like the page it came from.

## What Changes

- **Short sessions, most fragile first.** A session holds at most 10 cards, ordered by
  predicted recall, lowest first, with new cards interleaved; « Encore 10 » starts another
  one over what is still due. This replaces "a single language keeps its usual order".
- **New words enter review at a daily pace.** At most 10 never-reviewed cards a day in
  each studied language by default, a reader setting offering 5, 10 or 20. Capturing a word
  is unchanged and unlimited; the cards beyond the day's allowance wait.
- **A missed card comes back in the same session**, three cards later, until it is
  recalled once or has been asked three times. Only the session's first answer updates
  the card's FSRS state.
- **Two answers instead of four grades** on every review surface: « Pas su » and « Su »,
  left and right like turning a page. « Je connais » stays. **New:** « Ne plus me le
  montrer » marks the word ignored and retires its card without deleting it. The engine
  keeps accepting the four FSRS ratings, so its other callers are unaffected.
- **The front of the card is the sentence** where the word was met, with the word marked.
  The answer appears in a space reserved from the start, so revealing it repaints one zone.
  The layout follows the screen: answers at the bottom on a phone held upright, on the
  sides when it is held sideways, a centred column on a tablet or a wide panel.
- **A session ends by saying what it did**: cards reviewed, missed words recovered, cards
  now held for more than a month; it offers to continue only when cards remain due.
- **Backup, restore and the pack's sources move to Réglages › Données**, rendered by the
  one settings builder in every host; the toolbar popup offers the backup and points to the
  panel for a restore. The review page keeps the language filter, the due count and the card.
- Unchanged: the FSRS formulas and parameters, the card schema, the backup format, the
  sync protocol, and the absence of any network call.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-decks-review`:
  - *Review present right next to the reading* — the sentence shows on the front; the
    gloss stays hidden until the reveal.
  - *One review queue across languages, with a language filter* (added by
    `add-lingua-language-stats-review`, not yet archived) — a session, in the one language
    the review is in since `refine-lingua-review-language`, is ordered by predicted recall;
    the text no longer offers every language at once. Its title is left to
    `refine-lingua-language-wording`, which reconciles that requirement.
  - Added: short sessions, a daily pace for new words, a missed card coming back in the
    session, two answers, hiding a word from review, and the end-of-session summary.
- `lingua-browser-extension`:
  - *Two review surfaces* — backup and restore are reached from Réglages › Données in
    every host that shows Réglages, the side panel included.
  - Added: the review card fits the screen it is on.

This change archives after `add-lingua-language-stats-review` (`archiveAfter` in
`.openspec.yaml`): one of its deltas modifies a requirement that change introduces.

## Impact

**Products.** Cymbra Lingua only: the core (`crates/lingua-core`), its WASM build
(`crates/lingua-wasm`), the browser extension (`apps/lingua-extension`) on Chromium,
Firefox and Safari, and therefore the Apple container app (`apps/lingua-apple`), which
ships the extension bundle unchanged. Cymbra ID, Music, Live, the back office and the
public site are untouched. Nothing new is consumed from `id-*` or `platform-*`: no new
flag, no new RPC, no `.proto` change, no backend change. The agent plugin
(`apps/lingua-agent`) keeps its own store and its four ratings. « Ne plus me le montrer »
syncs through the existing status and card operations (an `ignored` status and a retired
card).

**Code.**
- `crates/lingua-core/src/decks/review.rs` — session size, order by predicted recall,
  the daily allowance, the in-session return of a missed card, `ignore`, the summary.
- `crates/lingua-wasm/src/lib.rs` — `startReview` options, `reviewIgnore`,
  `reviewSummary`.
- `apps/lingua-extension/src/analyzer/port.ts` and the engine ports, `src/review/`
  (`session.ts`, `view.ts`, `review-page.ts`), `src/styles/review.css`,
  `src/reading/settings-view.ts` (the allowance setting and the two Données blocks),
  `src/state/storage.ts` (the setting's own key).

**Data.** No migration. The review state and the backup format do not change; the new
setting lives under its own preference key and is absent until the reader changes it.

**CI.** `rust` and `lingua-extension-check` already watch these paths.
