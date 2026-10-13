# add-lingua-review-practice — design

## Context

- `ReviewSession::start_with` (`crates/lingua-core/src/decks/review.rs`) queues the cards due at `now`:
  reviewed ones by predicted recall, never-reviewed ones within the day's allowance. `grade` updates
  a card's FSRS state on its first answer in the session and stamps it for sync; a missed card comes
  back after three others, at most three asks.
- A retired card ("Je connais", « Ne plus me le montrer ») keeps its memory with `due = i64::MAX`.
- The WASM `startReview` takes the session's options as positional arguments; the extension's
  `ReviewOptions` carries them; `ReviewController` starts sessions and records "review" in the day's
  counts when `reviewGrade` resolves true.
- On Safari the session is saved between two answers (`to_snapshot`), the background page being
  stopped by the browser.

## Goals / Non-Goals

**Goals:** a reader can practise when nothing is due, without moving any card's schedule; every host.

**Non-Goals:** reviewing ahead with FSRS (the product owner chose practice on 2026-10-13); more new
words beyond the day's allowance; practice while cards are due.

## Decisions

### D1 — A practice option on the session, in the core
`SessionOptions` gains `practice: bool`. A practice session queues the deck's reviewed cards
(`memory` set) that are not retired (`due != i64::MAX`), in the languages asked, due or not, lowest
predicted recall first, up to `limit`; no never-reviewed card, no allowance. The session remembers
it is practice, in its snapshot too (`#[serde(default)]`, so a snapshot an older engine wrote resumes
as a review, as it was).

*Alternative.* Build the practice queue in the extension: it has no API listing the deck's cards with
their recall, and the session's requeue and summary would be written twice.

### D2 — A practice answer changes no card
In a practice session `grade` never calls `ReviewState::grade` nor stamps `updated_at`, so no card
changes, nothing syncs, the backup is unchanged by an answer. The requeue of a missed card and the
summary's counts work as in a review; `grade` returns true on a card's first answer — what the
extension reads as "count a review" — and `holding` stays 0. `mark_known` and `ignore` apply as in a
review: they are the reader's decisions, not answers.

### D3 — The binding and the port
The WASM `startReview` gains a trailing `practice: Option<bool>`; `ReviewOptions` gains
`practice?: boolean`, passed through by the in-page port and over messaging.

### D4 — Where practice is offered
`ReviewController` measures, when a session ends with nothing more due and when a start finds nothing,
whether a practice session would hold cards (it starts one and reads its count, as it already measures
« Encore 10 »), and exposes `practiceAvailable` and `practice` (whether the session under way is
practice). The view shows « S'entraîner quand même » under « Rien à réviser pour l'instant » and at the
end of a session when `practiceAvailable`; a practice card's head says « Entraînement · N carte(s) »
in place of « N carte(s) à revoir ». The end of a practice session offers practice again, never
« Encore 10 » over due cards it does not have.

## Risks / Trade-offs

- [The same few cards practised again and again] → the queue starts with the lowest recall, which a
  practice answer does not change: a reader practising twice in a row sees the same cards. Accepted:
  practice is a bonus, and a short deck has few cards anyway.
- [An older extension with a newer engine, or the reverse] → the option is trailing and optional on
  both sides: absent, a session is a review.
