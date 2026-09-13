## Context

`KnowledgeState` (`crates/lingua-core/src/knowledge/state.rs`) holds only *explicit*
statuses (`learning`, `known(provenance)`, `ignored`). Everything else is resolved on
the fly:
- A lemma below the declared CEFR level (or, without a level, ranked within the
  calibration threshold) is presumed `known(calibration)`.
- The rest is new.

Exposure promotion (`promote_by_exposure`) turns a presumed lemma read on N distinct
days into an explicit `known(exposure)`. The extension calls it on every exposure flush.

For cross-device last-write-wins, the state also keeps an `updated` map of per-lemma
timestamps. A clear leaves that timestamp behind as a tombstone, so a stale re-add cannot
win.

The undo gesture ("Remettre à apprendre") calls `setStatusAt(lemma, null, now)`, which
removed the explicit status without stamping. Three consequences:

1. The lemma fell back to calibration: presumed known, not highlighted.
2. An exposure-confirmed lemma was promoted again at the next flush.
3. `export_statuses` iterated present statuses only, so the clear was never pushed.

## Goals / Non-Goals

**Goals:**
- An undo makes the word highlighted again, whatever the declared level or calibration.
- An undo permanently blocks exposure re-promotion, like any other user interaction.
- An undo propagates to other devices and survives a wipe + re-pull.
- No proto, backend or migration change; older clients keep working.

**Non-Goals:**
- Reorganising the marked-words list (grouping by provenance, search, a dedicated tab,
  bulk revert of exposure promotions). Tracked separately.
- Card creation on undo: putting a word back does not add it to the deck.
- A user-facing "force unknown" action on a merely presumed word.

## Decisions

### D1 — The stamped tombstone *is* the withdrawal

A lemma with no explicit status but an entry in `updated` is **withdrawn**.
`resolve_lemma` returns `None` (new) for it before consulting the declared level or the
calibration. A new `clear_status_at(lang, lemma, at_ms)` clears and stamps; the wasm
`setStatusAt` clear path uses it.

Alternatives considered:
- **A new explicit status (`unknown` / `new`).** Honest in the type, but it is a wire
  change: a new `KnownWordsService` value, backend handling, a `buf breaking`-relevant
  vocabulary change. Older clients would have to cope with a kind they do not know. It
  also duplicates what the tombstone already records.
- **Undo = add to deck (`learning`).** Matches the label literally, but creates
  sentence-less review cards the reader did not ask for. It turns a correction into
  study work.
- **A separate `withdrawn` set.** Explicit, but it duplicates `updated` and must be kept
  in step with it on every set, clear and pull, including through backups.

The tombstone already exists, is serialised in backups (`#[serde(default)]`) and is
already written by pulled clears. Giving it meaning costs no schema change. The meaning
is documented on the `updated` field.

### D2 — Promotion promotes exactly the presumed lemmas

`promote_by_exposure` now promotes a lemma iff it resolves to `known(calibration)` (and
meets the day threshold), instead of re-checking "no explicit status" and "below level"
itself. One resolution rule means a withdrawal — or any future resolution nuance —
blocks promotion without a second guard drifting from it.

### D3 — Withdrawals export as `cleared`

`StatusRecord.status` becomes `Option<Status>`. `export_statuses` walks the union of
statuses and tombstones in deterministic order, and emits `None` for a withdrawal.
`exportStatusOps` maps `None` to `"cleared"` with provenance `manual`.

`cleared` is already a documented `KnownWordsService` value, and the server stores
`status` as free text under LWW. A pulled `cleared` goes through `apply_status_lww(None)`,
which already stamps.

### D4 — A first pulled withdrawal counts as a change

`apply_status_lww` reported a change only when the explicit status differed. A pulled
`cleared` for a lemma this device never decided on leaves the status absent, but it does
change how the lemma resolves. The sync only persists the backup when something changed,
so the function now also reports a change when the withdrawn flag flips.

### D5 — Unstamped clears stay plain "new"

`set_status` / `clear_status` without a timestamp (internal, tests, the v1 migration)
leave no tombstone, so they keep the old "back to calibration" behaviour. Only a decision
that carried a sync time can be withdrawn.

## Risks / Trade-offs

- **[Retroactive effect]** A word put back before this change already has a tombstone
  (from the earlier stamped decision), so it becomes highlighted after the update.
  → Intended: it is what the reader asked for. Noted in the proposal.
- **[Implicit meaning on a sync map]** `updated` now carries resolution semantics, not
  just ordering. → Documented on the field and on `clear_status`. The only writers of a
  status-less entry are clears (local stamped or pulled). `reset_statuses` replaces the
  whole `KnowledgeState`, so a partial reset drops tombstones with the statuses.
- **[Old clients]** An extension without this change applies a pulled `cleared` as a
  plain clear, and presumes the word known again locally. → Converges on update; no data
  is lost (the tombstone is written by LWW either way).
- **[Outbox growth]** Every withdrawn lemma stays in the full push. → Bounded by explicit
  user gestures, and pushes are idempotent under LWW.
- **[Wipe + re-pull]** A partial reset clears tombstones locally. → The server keeps the
  `cleared` op, so the re-pull restores the withdrawal.
