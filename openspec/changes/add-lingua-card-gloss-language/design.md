# Design — add-lingua-card-gloss-language

## Context

See proposal.md (Why). Where a gloss lives today:

| Where | What |
|---|---|
| `crates/lingua-core/src/decks/card.rs` `Card` | `gloss: Option<String>`, "in the user's native language"; `#[serde(default)]` on `updated_at` shows how a field is added without moving old backups |
| `crates/lingua-core/src/decks/backup.rs` | schema version 2; `profile` left out while default, so an English backup does not move |
| `crates/lingua-wasm/src/lib.rs` `add_card` | the gloss the surface hands in (the pack's for a word, the shown one for an expression) |
| `lib.rs` `export_card_ops` / `apply_card_ops` | the JSON shape the sync client sends and applies; `gloss` empty ↔ `None` |
| `lib.rs` `review_current` | `{headword, surface, sentence, source, gloss, revealed, remaining}`, pinned by the English baseline; the language is a call of its own (`reviewCurrentLanguage`) |
| `lib.rs` `nativeLanguage` | the engine's one native language (generalise-lingua-native-language) |
| `crates/lingua-core/src/packs/pack.rs` | `Pack::gloss(lemma)`, `Pack::expression(key)` |
| `apps/lingua-extension/src/analyzer/port.ts` | `CardOp` (export and apply shape), `ReviewCard` |
| `apps/lingua-agent/rust` | local cards, one native language followed from `pack.lingua` |
| `crates/lingua-wasm/tests/cross_native.rs` | a second pack over the same studied tables, glossed in another native language: the fixture a card "from another native" needs |

A card is keyed by (studied language, lemma); a multi-word expression is a lemma with spaces.
There is no flag saying whether a gloss came from the pack or from a person: every gloss today
is the pack's or none.

## Goals / Non-Goals

**Goals:**
- Every card says the language of its gloss, exactly `fr` for every card that exists today.
- Review never shows a gloss in a language the reader did not choose, when the pack has one.
- No backup, baseline or French gloss moves.

**Non-Goals:**
- The wire: change 12 sends the label, withholds non-French cards until the server stores it,
  and rewords *The backup records the reader's language profile* for the native language on
  statistics.
- An edited gloss, or a flag saying a person wrote it: no surface lets a reader write a gloss.
- A card's surface showing both glosses, or the label itself.

## Decisions

### D1 — `gloss_language: String`, `fr` by default, out of the backup when `fr`

`#[serde(default = "french", skip_serializing_if = "is_french")]`. Every backup written today
is byte for byte the same, as the profile's `skip_serializing_if` keeps an English backup
unchanged; the schema version stays 2, since the field is read by its default and ignored by a
build that predates it (`Card` does not deny unknown fields). `Card::new` and `Card::seeded`
take the gloss language beside the gloss; their callers are the engine's `add_card`,
`seed_level` and `apply_card_ops`, and the agent.

Alternative: `Option<String>`, `None` meaning French. A `None` that means something is what
`gloss: None` already is (no gloss); a language is always known.

### D2 — Created with the engine's native language

`add_card` labels the card with `nativeLanguage()`: the gloss it was handed is the pack's, or
the one the surface showed, both in the engine's native; so does a card seeded from the pack
(`seed_level`). The agent labels with its followed pack's native language. `apply_card_ops`
reads `gloss_language`, absent or empty meaning `fr` (the wire default, change 10 D1), and
`export_card_ops` emits it when it is not `fr`: the goldens pin the exported operations of a
French-glossed deck byte for byte, and a reader of French gains nothing from a `fr` on every
line.

### D3 — Review substitutes in the view model, by key

In `review_current`, when `card.gloss_language` differs from the engine's native language:
- a word (no space) → `Pack::gloss(lemma)` of the pack held for the card's studied language;
- an expression (a lemma with spaces) → `Pack::expression(lemma)`;
- when either answers a gloss, `gloss` is that; otherwise `gloss` is the card's own text.

The JSON keys are unchanged, so the English baseline pins the same bytes for every French card,
and no surface changes: `review/view.ts` shows `card.gloss` as it does. The card itself is not
rewritten: its label and text stay, and a later pull may replace them under last-write-wins.

Alternative: a second call (`reviewCurrentGloss`), as `reviewCurrentLanguage` was added. That
would put the choice in two surfaces (the drawer and the side panel) instead of one engine; the
baseline is kept by keeping the keys, not by keeping the call.

### D4 — A gloss the pack has not is kept

A card whose lemma the current pack does not gloss keeps its text: it is the only gloss there
is, and — when a surface lets a reader write one — what a free translation is (M4). It is shown
as it is, in its language.

## Risks / Trade-offs

- **A backup byte moves** → `backup_format` and S0 run on every pull request; the field is out of
  the backup when `fr`, and no shipped engine creates another label.
- **An expression key that is not the pack's key** → the expression table is keyed by the lemma
  the card was created from (`whole.key`), which is the card's lemma; a miss keeps the card's
  text (D4), never an error.
- **A review that reads the pack on every card** → one lookup per revealed card, on tables
  already in memory; measured by the baseline's timings.

## Migration Plan

One release, silent. A backup written by this build is read by the previous one (the field is
ignored) and by this one; a backup written by the previous one reads every card as `fr`.
