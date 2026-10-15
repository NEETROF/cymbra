# add-lingua-card-edit — design

## Context

See proposal.md for the why. What exists:

- **The card** (`crates/lingua-core/src/decks/card.rs`): `lemma` (the dictionary form; a lemma
  with a space is an expression), `encountered_form`, `provenance { sentence, source,
  captured_at }`, `gloss`, `gloss_language` (left out of the backup when `fr`), `media`
  (reserved), `review` (FSRS), `updated_at` (seconds). The deck (`decks/review.rs`) is
  `BTreeMap<StudiedLanguage, BTreeMap<lemma, Card>>`: a card's identity is its lemma. Nothing
  removes a card today; `retire` keeps it and stops it coming due.
- **Synchronisation.** `exportCardOps` emits the whole deck as `CardOp`-shaped JSON, `client_id`
  = lemma, `deleted: false`, `client_ts` = `updated_at` × 1000, `gloss_language` only when it is
  not `fr`. `applyCardOps` applies each op through `Deck::apply_card_lww` (whole card, newer or
  equal wins, the local `captured_at` and page address kept) and **skips** `deleted` ops: "card
  deletion — and the tombstones it would need — is deferred to the change that actually adds
  it". The server (`backend/lingua/src/pg_deck.rs`) already stores a tombstone as a row with
  `deleted = true`, keyed by (user, language, client id), LWW by `updated_at` then `device_id`,
  and returns it on pull; a later upsert of the same key with a newer time overwrites it, so
  the server already allows a removed card to come back. The sync client pushes the whole
  export each time (`src/sync/sync.ts`).
- **The Cymbra Lingua app** reviews a restored copy of the backup the Safari extension shared,
  and after every save writes `appOpsSince(port, since)` — the engine's own `exportCardOps` and
  `exportStatusOps` filtered by time — to the App Group; the extension applies them with
  `applyStatusChanges` then `applyCardOps` (`src/state/app-relay.ts`). Whatever the engine
  exports and applies, the relay carries.
- **The gloss shown in review** (`review_current` → `readable_gloss` in `crates/lingua-wasm`):
  the card's own gloss when its `gloss_language` is the engine's native language; otherwise the
  pack's gloss for the card's lemma (word or expression table), the card's text only where the
  pack has none (*Review shows a gloss the reader can read*). So in that second case the pack
  wins over the card's text.
- **Capture** (`addCard`) upserts the card and sets the lemma `learning`, stamped. A cleared
  status already synchronises as `cleared` (*Withdrawals synchronise as cleared statuses*).
- **The review UI** (`src/review/view.ts`): the head, the source, the headword when the
  sentence does not hold the word (`markWord`: the form met, else the dictionary form, a whole
  word, letter case aside), the sentence, then the answer space reserved from the start, the
  action zone (« Afficher la réponse », then « Pas su » / « Su ») and two links (« Je connais »,
  « Ne plus me le montrer »). One action zone the same size before and after the reveal, no
  transition — e-ink repaints (refine-lingua-review-session D6, D7). Keys are read on the
  review's root: Space/Enter reveal, ←/→ answer. Copy lives in `src/i18n/{fr,en,es}/review.ts`.
- **Capture's limit**: a selection is at most `MAX_SELECTION_LENGTH` = 120 characters
  (`src/reading/selection.ts`).

## Goals / Non-Goals

**Goals:**
- The reader corrects a card's form and its translation from the review, on every host, in one
  repaint to open the form and one to close it.
- A rename converges on every device of the reader — the extension, the app, other signed-in
  devices — without resurrecting the old card and without forbidding a later re-capture.
- A translation the reader wrote is never silently replaced.

**Non-Goals:**
- Editing the sentence, the language, the encountered form directly, or the schedule.
- Editing cards outside a review (a deck list, the agent plugin): no such surface edits cards
  today; the core binding is reusable when one does.
- Showing the reader's translation in the reading word card, which shows the pack's: a
  follow-up if the dogfood asks for it.
- Moving exposure counters from the old form to the new one: they count what was read, under
  the form that was read.
- Deleting a card as a reader action (a « Supprimer »): the tombstones make it cheap later, but
  it is not asked for.

## Decisions

### D1 — « Modifier la carte » is always on the card; choosing it reveals the answer

A third link, « Modifier la carte » (French UI copy; « Edit card », « Editar la tarjeta » — renamed
from « Modifier » at implementation, see A2), sits in the links row with
« Je connais » and « Ne plus me le montrer », before and after the reveal. Before the reveal,
choosing it reveals the card — exactly as « Afficher la réponse » does, the reveal recorded in
the session — and opens the form in the same repaint.

The translation field necessarily shows the answer, so editing before the reveal would spoil
the card; revealing first is honest about that, and the reader then grades as usual. Keeping the
link in both states keeps the action zone the same size before and after the reveal, which is
the rule e-ink depends on. The junk-card case (« four-poster bed, ») is seen before the reveal:
one tap, not two.

*Alternative.* The link only after the reveal: the links row changes between the two states (a
reflow on e-ink), and a junk card costs a reveal first. *Alternative.* Editing the form alone
before the reveal, the translation after: two modes of one form for a rare action.

### D2 — An inline form in place of the answer and the actions

The form replaces the answer space and the action bar; the head, the source and the sentence
stay above it, so the reader edits with the sentence in view. Two labelled fields:

- « Mot ou expression » (French UI copy), prefilled with the card's dictionary form; `lang` =
  the card's studied language, `autocapitalize="none"`, `autocorrect="off"`,
  `spellcheck="false"` (a form is not a native-language sentence), `maxlength` 120.
- « Traduction » (French UI copy), prefilled with the gloss the review shows for the card — the
  card's own, or the pack's where the review substitutes it — and empty when the card has none;
  `lang` = the engine's native language, `maxlength` 200. A machine translation shown by
  `add-lingua-review-translation` is **not** prefilled: `lingua-translation` holds that a
  machine translation is never stored, and storing an accepted one is
  `add-lingua-review-translation-fallback`'s decision, not this change's.

Then « Enregistrer » (primary) and « Annuler ». The word field takes the focus. Enter in a field
saves, Escape cancels; the review's own keys (Space/Enter reveal, ←/→ answer) are ignored while
the focus is in the form — the root's key handler returns for events from an `input`. Each field
has a visible `<label>`; an invalid field gets `aria-invalid` and an error line tied to it by
`aria-describedby`, in the interface language (French UI copy: « Écrivez un mot ou une
expression. », « 120 caractères au plus. », « 200 caractères au plus. »). On a phone the
focused field is scrolled into view above the keyboard (`enterkeyhint="done"`). Opening and
closing are one repaint each, no transition; styles from `tokens.css`, sizes from
`--cymbra-lingua-ui-scale`. Read-aloud is stopped when the form opens.

A save that changes nothing closes the form and writes nothing. After a save the card repaints
revealed, with its new form and gloss; the front marks the new form in the sentence when the
sentence holds it, and otherwise shows the form above the sentence, as for any card whose
sentence does not hold its word. The reader then grades the card; the edit itself is not an
answer and counts nothing.

*Alternative.* A separate dialog: a layer and a focus trap in four hosts and the app, and a
second repaint on e-ink. *Alternative.* Editing in place by tapping the headword: invisible to
the reader and impossible before the reveal.

### D3 — The engine checks the form; the page shows the same rule early

`reviewEdit(form, gloss, now)` in the WASM build is the authority; the page applies the same
checks to show errors without a round trip. A form is: NFC, trimmed, inner whitespace runs
collapsed to one space, edge punctuation stripped by the rule capture uses
(`fix-lingua-selection-card-punctuation` — one rule, one function: whichever change lands first
introduces it and the other calls it, so a form typed here and a selection captured on a page
are trimmed alike), then non-empty and at most 120 characters
(capture's own limit). Letter case is kept as typed. A translation is trimmed and collapsed,
at most 200 characters; empty means "no translation of mine". The card keeps its studied
language: there is no language field, and the form is not checked against the pack — an
expression or a word the dictionary does not know is a legitimate card.

The binding answers `{ ok: true, merged?: lemma }` or `{ ok: false, error: "empty" |
"form_too_long" | "gloss_too_long" }`, never a raw message.

### D4 — Which gloss an edit leaves, and the reader-written mark

A card gains `gloss_by_reader: bool` (serde default `false`, left out of the backup and of the
card op when `false`). On save, in this order:

1. **The translation field was changed to some text** → that text, `gloss_by_reader = true`,
   `gloss_language` = the engine's native language.
2. **The translation field was emptied** → the reader withdraws their translation: the pack's
   gloss for the (new) form if the pack has one, else no gloss; `gloss_by_reader = false`.
3. **The field was left as prefilled and the form changed** → a gloss already written by the
   reader stays (and keeps its mark); otherwise the pack's gloss for the new form, when the pack
   of the card's language has one — read as `readable_gloss` reads a lemma: a word's gloss, or
   an expression's from the expression table — labelled with the engine's native language;
   otherwise the card's previous text stays, as the reader saw it in the field and kept it.
4. **Nothing changed** → nothing is written.

The precedence is reader-written > pack > previous text > none. The reader's text wins because
it is the only one they chose: the pack's gloss for « referred to as » is a guess about their
sense, theirs is the sense they meant. The pack wins over a previous text the reader merely left
in place, because that text was the old form's.

In review, `readable_gloss` returns a reader-written gloss as it is, whatever its label: the
reader wrote it, so they can read it. That is the one change to *Review shows a gloss the
reader can read*; a card without the mark is shown as before.

**Keeping the mark across synchronisation.** `apply_card_lww` keeps the local mark when the
incoming card carries none and its gloss is the same text as the local one. The mark is about
*that text*: an echo from a server that predates the field, or a card pushed back by an older
build after a grade, carries the same gloss and must not clear it; a different gloss that wins
LWW correctly arrives unmarked.

*Alternative.* No mark, the edit only relabelling the gloss in the native language: the pack
would still replace it on any engine whose native language differs, and nothing would say a
human wrote it. *Alternative.* Storing the reader's gloss in a second field beside the pack's:
two glosses to reconcile on every surface, for a precedence a boolean states.

### D5 — The mark travels: backup, relay, and one additive server field

- **Backup**: the field, left out when `false`; no schema version change, as for
  `gloss_language` (add-lingua-card-gloss-language). A build released before this change reads
  a backup carrying it — `Card` denies no unknown field — and drops it at its next save
  (rollback only).
- **Relay**: the app relays `exportCardOps` as JSON; the op gains `gloss_by_reader: true` when
  set, which the extension's `applyCardOps` reads. Nothing changes in the relay's shape.
- **Server**: `CardOp` gains `bool gloss_by_reader = 13`; `lingua.cards` gains
  `gloss_by_reader BOOLEAN NOT NULL DEFAULT false` (migration `0008`, idempotent,
  schema-qualified like 0001–0007); the server stores it and returns it as stored, no logic.
  Added field: `buf breaking` (FILE) passes, installed clients ignore it.

The server change is justified because a card's other devices are otherwise told nothing: a
second device would show the pack's gloss over the reader's wherever its native language
differs, and the mark would be lost from every device the first time a card is restored from
the server. It is one pass-through field; no capability gate is needed, because a server that
predates it degrades to today's behaviour and the keep rule of D4 protects the local mark.

*Alternative.* The mark local only (like the page address): one device knows it, the others
never do.

### D6 — Merging into a card the deck already holds

The target is the card of the same studied language whose lemma equals the new form, letter
case aside, other than the edited card itself. (A form that differs from the card's own lemma by
letter case alone is a plain rename to the new spelling.) The surviving card is under the
**target's** lemma, as spelled in the deck; the edited card is removed (D7). Its fields:

- **Review state**: the one with more reviews (`reps`); a tie goes to the more recent last
  review; then to the target. A longer history is the better evidence of what the reader
  remembers, and FSRS stability built over more answers is the more reliable. Keeping the more
  fragile state would bring back a word the reader has been recalling for months as if new.
- **Gloss**: the edited card's gloss as D4 settles it, when reader-written; else the target's,
  when reader-written; else the edited card's settled gloss when it has one; else the
  target's. The reader's own words survive a merge from either side.
- **Sentence, encountered form, source**: the edited card's — the encounter on screen, which
  the reader just corrected; the encountered form is recomputed (D8).
- **`captured_at`**: the earlier of the two: when the word was first met.
- **`media`**: the target's when it has some, else the edited card's (unpopulated today).
- **`updated_at`**: the edit's time.

In the session, the merged card stays the current card, revealed, and any later place of the
target in the session's queue is dropped, so the word does not come twice. If the target was
already answered earlier in this session, the merged card counts as answered: its next answer
moves no schedule (*A missed card comes back in the same session*: only a card's first answer
counts). The edit answers `merged: <target lemma>`, and the card says once, in its answer space,
that it was merged (French UI copy: « Réunie avec la carte « referred to as » déjà dans vos
révisions. »).

*Alternative.* Keeping the target's state always: the edited card's months of answers would be
thrown away whenever the duplicate happened to be the newer card. *Alternative.* Refusing the
edit: the reader would have to hide one card and keep two histories apart.

### D7 — A rename is an addition and a removal: tombstones in the deck

The deck gains `tombstones: BTreeMap<StudiedLanguage, BTreeMap<lemma, i64>>` (seconds; serde
default, left out when empty, so every deck without an edit keeps its bytes). A rename at time
`t`: the card is moved to its new key with `updated_at = t`, and the old key gets a tombstone at
`t`.

- **Export**: every tombstone is a card op `{ client_id: lemma, language, lemma, deleted: true,
  client_ts: t × 1000 }`, the other fields empty.
- **Apply a deleted op** at `t`: a local card of that key whose `updated_at` is later is kept (a
  re-capture after the removal wins); otherwise the card is removed and the tombstone recorded
  at the later of the two times. With no local card, the tombstone is recorded all the same, so
  a stale copy arriving afterwards is refused.
- **Apply a card op** at `u` for a tombstoned key: dropped when `u` ≤ the tombstone, applied
  (and the tombstone cleared) when `u` is later.
- **A local addition** of a tombstoned key — capture, seeding, a rename onto it — clears the
  tombstone: it is a newer decision. Seeding already skips a lemma with a card or an explicit
  status; a removed lemma has neither, so it may be seeded again, as a never-met word may.
- **Retention**: tombstones are kept. They are as many as edits, a few bytes each; pruning would
  need an acknowledgement from every device, which the protocol does not have.

The server needs nothing: it already applies and returns tombstones by LWW and lets a newer
upsert bring a key back. The device rule (tombstone wins a tie) and the server's (`device_id`
breaks a tie) differ only for an edit and a re-capture of the same form in the same second on
two devices.

*Alternative.* Renaming by retiring the old card instead of removing it: no sync change, but
« four-poster bed, » would stay in the deck's counts and come back to life on any device that
puts the word back to learn. *Alternative.* A `renamed_to` pointer: a second LWW object to
reconcile, for what a removal plus an addition already says.

### D8 — What the renamed card shows on its front

The encountered form becomes the new form's occurrence in the sentence — found as the front
finds it (`markWord`: a whole word, letter case aside), taken as the sentence writes it — and
the new form itself when the sentence does not hold it. The front then marks it, or shows the
form above the sentence. « colloquially referred to as » → « referred to as » marks
« referred to as » in the same sentence. The sentence never changes.

If `add-lingua-card-sentence-translation` has landed, a kept sentence translation stays with
the card: the sentence it translates has not changed.

### D9 — The word's status follows the rename

At the edit's time `t`, stamped like every decision:
- the **old form**'s status is cleared when it is `learning` — capture set it, and the reader
  has just said that form is not the one to learn; any other status of the old form is left;
- the **new form** becomes `learning` when it holds no explicit status; a `learning` one is left;
- a new form the reader made **known or ignored** keeps that status, and the card is retired
  and leaves the session, as when the reader marks a word known in review: a card never keeps
  coming due for a word the reader no longer learns.

The cleared status synchronises as `cleared`, the new one as any status; the app relays both.
`lingua-knowledge-model` is consumed as it stands.

### D10 — One binding in the engine; the page only paints

`reviewEdit` acts on the session's current card: validates (D3), settles the gloss (D4), merges
or renames (D6, D7), moves the statuses (D9), re-keys the session — its queue, its current key,
its snapshot — and returns the outcome. The page (`review-page.ts`) calls it through the port,
persists with `saveBackup` as after an answer, so the Safari extension's store and the app's
App Group write happen exactly as they do after a grade, and repaints. `ReviewController` gains
`edit(form, gloss)`; `ReviewActions` gains `edit` and the view tracks the form's open state and
errors.

## Risks / Trade-offs

- [An installed extension built before this change skips deleted ops: the old card stays on
  it, and its next whole-deck push brings it back to the server with a newer time after a
  grade] → the extension updates through the stores within days and the app ships the same
  bundle; the window is the release's rollout. Measured in the dogfood with an old build on a
  second device.
- [A merge picks one review history and drops the other's] → the rule keeps the longer one, and
  merges are rare (a duplicate is the exception); the dropped card's history is not otherwise
  shown anywhere.
- [Editing to a form the reader marked known or ignored makes the card leave the session at
  once] → consistent with « Je connais »; rare.
- [The mobile keyboard covers the form in the app] → the focused field scrolled into view, the
  buttons right under the fields; checked on iPhone.
- [The server ships after the clients] → an old server ignores field 13 and the keep rule of D4
  holds the local mark through the echo; the migration plan deploys the server first anyway.
- [Tombstones grow without bound] → one per edit; a pruning protocol is a follow-up if a deck
  ever holds thousands.

## Migration Plan

1. Backend first: migration `0008` and field 13, deployed with `backend-deploy`; nothing reads
   the column until clients send it.
2. Extension release (Chromium, Firefox, Safari), then the Apple app built on it.
3. No data migration: existing cards are not reader-written and no deck holds a tombstone.

Rollback: an older extension reads backups and ops carrying the new fields and ignores them; it
re-shows nothing removed on its own device, and drops the mark at its next save. The column can
stay.

## Adaptations at implementation (2026-10-15)

What landed on `main` between the proposal (#942) and its implementation, and what the
implementation settled; the decisions above stand except where a line here says otherwise.

- **A1 — The reader's mark already existed.** `add-lingua-review-translation-fallback` (#948, #944)
  landed `Card.gloss_by_reader` (serde default, left out when `false`), `Deck::keep_gloss`, the echo
  rule of D4 in `apply_card_lww`, `exportCardOps`/`applyCardOps` carrying `gloss_by_reader`,
  `CardOp.gloss_by_reader = 13` in `deck.proto`, migration `0008_lingua_card_gloss_by_reader.sql`
  (deployed in production) with `pg_deck.rs` storing and returning it, and the review precedence of
  D4 (`review_gloss`: the reader's gloss, else the pack's at the lemma trimmed of its edge
  punctuation, else the stored text). This change reuses all of it: no second field, no second
  migration, no proto change. D5's server bullet and step 1 of the migration plan are done.
- **A2 — The link says « Modifier la carte ».** The review card already carries a « Modifier » since
  #948: the keep action's, beside « Garder « … » », which opens the field to edit the translation to
  keep. Two « Modifier » on one card would edit two different things. The card edit's link is
  « Modifier la carte » (« Edit card », « Editar la tarjeta »): it names its object, and the keep
  action keeps its own « Modifier » for the candidate. Making the keep row's « Modifier » open the
  card form instead was weighed and left out: it would change a shipped, dogfooded gesture and mix
  a machine translation into the form D2 keeps it out of.
- **A3 — No server change.** The server already stores a `deleted` card op as a tombstone row
  keyed by (user, language, client id), last-write-wins by `updated_at` then `device_id`
  (`pg_deck.rs`), returns it on pull, and lets a newer upsert bring the key back
  (`backend/lingua/src/deck.rs` tests). Clients only had to send and apply it.
- **A4 — The extension's sync client sends the mark.** #948 left the TypeScript sync client
  (`src/sync/sync.ts`) without `glossByReader`: it now pushes `gloss_by_reader` as field 13 and maps
  it back on pull, and pulled `deleted` ops reach `applyCardOps`, which applies them through
  `Deck::apply_card_removal_lww` instead of skipping them.
- **A5 — The app's own path.** Since #957 the Cymbra Lingua app synchronises directly
  (`src/app/deck-sync.ts`, `SyncEngine` with `ops` = the app's pending ops). Those ops are
  `appOpsSince`, i.e. the engine's `exportCardOps` and `exportStatusOps` filtered by time: the removal
  (a `deleted` op stamped with the edit's time), the renamed card with its mark and the cleared and
  new statuses reach the server through the app's sync and the extension through the App Group
  relay, without any change to either path's shape.
- **A6 — Edge punctuation, one rule.** `fix-lingua-selection-card-punctuation` landed first:
  `without_edge_punctuation` (lingua-core) and `withoutEdgePunctuation` (`src/reading/selection.ts`)
  are called, not reintroduced. Lengths are counted in UTF-16 code units — JavaScript's `length`, as
  capture's `MAX_SELECTION_LENGTH` and the keep action's bound count —, in the engine and the page.
- **A7 — The binding's answer.** `reviewEdit` answers `{ ok: true, changed, merged?, retired? }` or
  `{ ok: false, error }`: `changed: false` for a save that changes nothing, so the page persists only
  a change, and `retired` when the new form is known or ignored. The prefill and the "left as it was"
  test both read the gloss `reviewCurrent` shows (`review_gloss`), computed by the binding.
- **A8 — D4 step 3, made precise.** "The card's previous text stays" is the text the reader saw in
  the field: when the review showed the pack's gloss of the old form over a stored text in another
  language, that shown text is kept, labelled with the native language; when it is the stored text,
  the card keeps it with its label.
- **A9 — Where the code lives.** The edit is `ReviewSession::edit` in
  `crates/lingua-core/src/decks/review/edit.rs` (a child module of `review`, so it re-keys the session
  in place), the pack lookup handed in as a closure; `lingua-wasm`'s `readable_gloss` is split so
  `pack_gloss` reads a pack at any lemma. The deck's removals serialise as `tombstones`, left out when
  empty; a removal's language counts among the backup's record languages, so a removal in French
  writes version 3 as a French card does.
- **A10 — The form while open.** It takes the place of the answer *and* of the whole action bar,
  links included (D2). The translation field's `lang` is the engine's native language, read once
  from `LinguaPort.nativeLanguage()`. The fields carry `maxlength` 120 and 200 as D2 says; the
  page's check (`src/review/edit.ts`) and the engine's still refuse a longer value, which a refused
  save shows again. The catalogue lint that keeps Réglages' block titles out of other modules
  (`test/lint-settings-hosts.spec.ts`) exempts `review.editGloss`, the field label « Traduction », by
  name.
