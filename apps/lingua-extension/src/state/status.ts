import type { LemmaStatus } from "../analyzer/types.ts";
import { type Card, defaultState, type LinguaState } from "./storage.ts";

// Pure state transitions over LinguaState (no I/O — the caller persists). The three
// reading gestures map here: "Je connais" → known, "+ Deck" → learning (+ a card),
// "Ignorer" → ignored. Clearing (null) removes the status and any card.

/** Context captured when a form is added to the deck via "+ Deck". */
export interface CardContext {
  surface: string;
  sentence: string;
  /** Epoch ms — the caller supplies the clock (determinism / testability). */
  createdAt: number;
}

/**
 * Apply a status change for one form, returning a new state.
 *
 * - `learning` adds the form to the deck and, when `card` context is given and no
 *   card exists yet, creates the card with its source sentence.
 * - `known` / `ignored` drop any deck card (the word is no longer being learned).
 * - `null` clears the explicit status and any card (back to calibration-only).
 */
export function applyStatus(
  state: LinguaState,
  lemma: string,
  status: LemmaStatus | null,
  card?: CardContext,
): LinguaState {
  const statuses = { ...state.statuses };
  const cards = { ...state.cards };

  if (status === null) {
    delete statuses[lemma];
    delete cards[lemma];
  } else {
    statuses[lemma] = status;
    if (status === "learning") {
      if (card && !cards[lemma]) {
        cards[lemma] = { lemma, surface: card.surface, sentence: card.sentence, createdAt: card.createdAt };
      }
    } else {
      delete cards[lemma];
    }
  }

  return { ...state, statuses, cards };
}

/** The deck: the forms currently being learned, with their cards. */
export function deck(state: LinguaState): Card[] {
  return Object.values(state.cards)
    .filter((c) => state.statuses[c.lemma] === "learning")
    .sort((a, b) => a.createdAt - b.createdAt);
}

/** How many forms are marked known (Known + the calibration is separate). */
export function knownCount(state: LinguaState): number {
  return Object.values(state.statuses).filter((s) => s === "known").length;
}

/** A full reset: statuses, cards, calibration and preferences back to defaults. */
export function resetState(): LinguaState {
  return defaultState();
}
