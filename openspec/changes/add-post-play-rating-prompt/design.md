## Context

Rating already exists end to end: `SubmitScoreRating` upserts one rating per
(user, score) and returns the fresh aggregate ([score_rating.rs](backend/music/src/score_rating.rs)),
`ListRatingDeck` sources the caller's un-rated pending/accepted scores, and the app
exposes it through the swipe deck ([rating_deck_notifier.dart](apps/music/lib/state/rating_deck_notifier.dart))
plus a library banner throttled by [rating_activity_notifier.dart](apps/music/lib/state/rating_activity_notifier.dart).

What is missing is a rating surface at the moment the user is best informed: right
after playing the piece. Three facts shape the design:

- **The player already knows the score.** `SelectedScore` holds a `CatalogEntry`
  whose `catalogId` is non-null exactly for public-catalog scores — the only
  rateable kind ([score_catalog.dart:129](apps/music/lib/state/score_catalog.dart:129)).
- **There is no "did I rate this one?" read.** The deck source answers in bulk
  ("my un-rated scores"), never for a single score, so the player cannot tell
  whether it should prompt.
- **Engagement is recorded only by the deck preview.** `rating_preview_bytes` is
  the sole caller of `record_engagement` ([module.rs:862](backend/music/src/module.rs:862)),
  so a rating submitted from the player would be coverage-ineligible and silently
  earn zero points.

The end-of-run surface must also coexist with an existing hard constraint: the
summary modal *requires* an explicit see-mistakes / retry / quit choice and refuses
to auto-dismiss (`PopScope(canPop: false)`, `barrierDismissible: false` in
[session_summary_modal.dart:36](apps/music/lib/widgets/session_summary_modal.dart:36)).

## Goals / Non-Goals

**Goals:**
- Offer the rating exactly where the user has an opinion: at the end of a run, and
  on the way out of an abandoned one.
- Never cost the user anything: no blocked exit, no extra mandatory choice, no
  confirmation dialog, no listening gate.
- Suppress precisely: per score, once, using server truth for "already rated".
- Keep the rating semantics identical to the deck — same operation, same
  star→verdict derivation, same rewards path.

**Non-Goals:**
- Changing the rating model (verdict, stars, aggregate, `needs_review`).
- Touching the swipe deck, its listening lock, or the library banner's thresholds.
- A new reward kind, or changing point values / caps.
- Rating bundled or user-contributed scores (they are not in the catalog).
- Server-side enforcement of "prompt at most once" — that is a client UX concern.

## Decisions

### D1 — The end-of-run affordance is inline in the summary modal, not a second dialog

A chained dialog after the summary would fight the modal's own contract (an
explicit choice, no auto-dismiss) and would read as a second gate between the
player and the library. Instead a compact star row sits **inside** the modal,
between the scrollable stats and the pinned action buttons, so it inherits the
"actions stay reachable on a short viewport" guarantee. Rating it mutates the row
in place (stars → thanks state) and never pops the dialog.

*Alternatives considered:* (a) a chained dialog — rejected, doubles the number of
dismissals the user must perform; (b) inside the scrollable stats area — rejected,
too easy to never see on a phone-landscape viewport.

### D2 — The early-exit prompt is a bottom sheet on a single exit funnel

Today there are two exit paths: the back `IconButton` in the top bar
([player_screen.dart:595](apps/music/lib/screens/player_screen.dart:595)) and the
system back gesture. Both are funnelled into one `_requestExit()`: if the prompt is
eligible it awaits a `showModalBottomSheet` (dismissible every way — barrier, back,
close, or rating), then pops unconditionally. The player's `PopScope` sets
`canPop: false` **only while a prompt is pending**, so the sheet gets a chance to
show once and the exit then always completes. A re-entrancy guard makes a second
exit intent pop immediately.

The sheet is *not* an "are you sure?" — it never offers a "stay" action, and every
outcome leaves. This is the whole point of the non-blocking choice.

*Alternatives considered:* prompting from the library after the pop (a snackbar) —
rejected: it detaches the question from the piece, and the library is where the
existing banner already lives.

### D3 — A dedicated `GetMyScoreRating` read, resolved once per player open

New RPC on `ScoreService`: `GetMyScoreRating(catalog_id) → { rated, verdict?, stars? }`,
identity-scoped, reporting *not rated* for an unknown or `rejected` id so it cannot
be used as an existence oracle (same fail-closed shape as the soundfont entitlement
read). The app resolves it once when the player opens a catalog score, so both
prompt moments decide instantly and offline (unresolved → no prompt).

*Alternatives considered:* adding `viewer_rated` to `CatalogHit` — rejected: it
forces a per-row join on every search page for a field only the player needs, and
the player is often reached without a fresh search (saved library, offline cache).

### D4 — Eligibility is a pure function; "played enough" is 25% of the piece's notes

`shouldPromptRating(...)` is a pure, host-testable predicate over
`(signedIn, catalogId, ratedState, alreadyOffered, playedNoteFraction)`, mirroring
`shouldInviteToRate`. The end-of-run case trivially satisfies the playback term.

Playback is measured **in notes**: the share of the score's notes the playhead has
passed, i.e. `count(n in notes where n.startMs <= furthestElapsedMs) / notes.length`,
with a threshold of **25%**. `PlayerData.notes` is already the piece flattened and
sorted by start ([player_data.dart:211](apps/music/lib/state/player_data.dart:211)),
so this is a binary search over a sorted list — a pure function of
`(notes, furthestElapsedMs)`. The player notifier only needs to keep a high-water
mark of `elapsedMs` (monotonic, so seeking back or pausing never lowers it); the
threshold is a named constant in the core, sited so it can later move to
feature-flag config without touching the predicate.

*Why notes and not time or position:* elapsed time is tempo-dependent — practising
at 50% tempo would double the time spent for the same music — and a position
fraction over-counts a piece that opens on long held notes or rests. Counting notes
measures how much *music* actually went by, independent of tempo and of the piece's
time layout.

*Counted over the whole score, not the selected hand:* the fraction uses `notes`,
not `visibleNotes`, so muting the left hand does not make a run look twice as
complete as it is.

*Interpretation:* "notes played" is read as **notes the playhead passed**, not notes
the user struck correctly. That keeps the criterion meaningful for an unscored
practice run and for a beginner who misses a lot; in Wait Mode the two coincide
anyway, since the gate does not advance until the note is played. The degenerate
case it admits — pressing play and walking away — is accepted: the prompt is
one-shot per score and dismissible in one tap. Note that a measure-range practice
run over a small slice of a long piece will not reach 25%, which is the intended
behaviour.

*Chords:* each note of a chord counts individually, since that is what `notes`
holds. A chordal texture therefore reaches 25% at roughly the same point in the
music as a monophonic one, so no per-onset de-duplication is worth the complexity.

### D5 — Per-score suppression: a bounded LRU of offered catalog ids in preferences

Per the product decision, suppression is **per score, not a global nag budget**:
each score is offered at most once, so the prompt volume is bounded by the number
of distinct scores the user actually plays — no global counter, no cross-score
snooze. The memory is one preferences key holding an ordered list of catalog ids,
capped (default 200, oldest dropped) so it cannot grow without bound; the
insert/trim logic is a pure function. The id is recorded when the prompt is
**shown**, not when it is answered, so a dismissal and a rating retire it alike.

This deliberately does **not** reuse `RatingActivity` (`lastAt` + `dismissals`),
which stays the library banner's own state. The two surfaces do not throttle each
other: the banner asks "rate some scores you haven't played", the prompt asks "rate
this one you just played", and a user who ignores one has not answered the other.

*Trade-off:* the memory is device-local, so a user on a second device can be
offered the same score once more — but only if they have not rated it, since D3's
server read still suppresses it. Rated scores never re-prompt anywhere.

### D6 — Playing records engagement, on both the bytes path and the session ingest

Coverage eligibility is widened at two seams so a post-play rating actually earns
points:

- **Player open** — a new module method mirroring `rating_preview_bytes`
  (`catalog_bytes_for_player(user_id, catalog_id)`) records engagement before
  delegating to the existing `get_catalog_bytes`, keeping that function's signature
  (it has no `user_id` today) and its other callers untouched.
- **Play-session ingest** — an offline-cached score never fetches bytes, so
  `RecordPlaySession` also records engagement when its `score_id` resolves to a
  catalog score. Wired at the gRPC composition seam rather than inside
  `play_module`, so the play module keeps no rewards dependency (the music module
  already holds `rewards` as an `Option`).

Both are best-effort and idempotent per (user, score) — `record_engagement` is
already an idempotent upsert, so double-recording a preview + a play is harmless.
Note this is a different notion from `catalog_limits`' engagement window (play
sessions + ratings for a *download allowance*): that stays untouched.

### D7 — Stars only, verdict derived; one shared widget for both surfaces

Both surfaces show the same 1–5 star row and derive the verdict the way the deck
already does (5 → love, 3–4 → like, ≤2 → dislike,
[rating_deck_notifier.dart:202](apps/music/lib/state/rating_deck_notifier.dart:202)).
Stars beat three swipe-verdict buttons here: the user has real nuance to express
after playing, and a star row is smaller than a button trio in a modal that is
already tight on a phone. The derivation function is extracted so the deck and the
prompt cannot drift apart.

### D8 — Layering: one notifier owns eligibility and submission

Per the repo's Riverpod rules, no widget touches `ratingServiceProvider`. A
`PostPlayRatingPrompt` notifier exposes the resolved eligibility and a
`submit(stars)` action; both widgets call the notifier and render its state. The
sheet is opened from the exit handler (a user gesture, not a state reaction), so it
needs no listener widget; the summary modal reads the notifier's state directly.
Submission failures live in the notifier's `AsyncValue` and surface at most a
localized message — never a gRPC string.

## Risks / Trade-offs

- **The summary modal grows on a short viewport** → the star row is pinned with the
  action buttons and is a single compact line; the existing "actions stay reachable"
  scenario is re-asserted in the `session-summary` delta and covered by a widget
  test at phone-landscape size.
- **The exit funnel is a behaviour change on a hot path** → a bug there could trap
  the user in the player. Mitigated by making `canPop: false` conditional on a
  pending prompt only, popping unconditionally after the sheet resolves whatever the
  outcome, and covering "dismiss leaves", "rate leaves", and "second exit intent
  pops immediately" in widget tests.
- **One extra RPC per player open** → a single indexed row read on the rating
  table, issued once per open and only for catalog scores; it is fired without
  awaiting on the play path, so a slow or failed read only means "no prompt".
- **Users may perceive the prompt as nagging** → bounded by construction: at most
  one prompt per distinct score played, never for an already-rated score, and never
  for bundled scores (which is what a new user mostly plays).
- **Rating quality could drop** (a frustrated abandoner rates the piece down for
  being hard, not bad) → the existing safeguards absorb this: ratings never change
  moderation status, `needs_review` needs a minimum rating count, and the honesty
  settlement scores a rater against community consensus. Worth watching in the
  aggregate after release rather than pre-empting with a gate.
- **Device-local prompt memory** → a reinstall or a second device can offer an
  un-rated score once more. Accepted: the server read already prevents the only
  outcome that matters (re-prompting a rated score), and syncing prompt history
  server-side is not worth an RPC.

## Migration Plan

No data migration: the rating and engagement tables already exist. The new RPC is
purely additive; a client older than the server simply never calls it, and a client
newer than the server gets an `UNIMPLEMENTED` that resolves to "unknown rated state"
and therefore no prompt — a fail-closed degradation, so the app can ship ahead of or
behind the backend. Rollback is removing the client prompt; the engagement widening
is idempotent and leaves no state to unwind.

## Open Questions

- Should the 25% played-notes threshold move to a runtime feature flag now, or stay
  a constant until real usage tells us it is wrong? Currently a constant in the core.
- Should a rating submitted from the prompt show the earned coverage points ("+N",
  as the deck does), or stay silent so the summary is not turned into a rewards
  surface? Currently silent.
