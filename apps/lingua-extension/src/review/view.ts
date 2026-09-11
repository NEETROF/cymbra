import type { Rating } from "../analyzer/port.ts";
import type { ReviewView } from "./session.ts";

// The review render — one function both surfaces (side panel and injected drawer) call
// to paint a ReviewView, wiring its controls to the given actions. Pure DOM, no styling
// of its own (the host provides the token sheet); no user-visible "lemma".

export interface ReviewActions {
  start(): void;
  reveal(): void;
  grade(rating: Rating): void;
  markKnown(): void;
}

const GRADES: { rating: Rating; label: string }[] = [
  { rating: "again", label: "À revoir" },
  { rating: "hard", label: "Difficile" },
  { rating: "good", label: "Correct" },
  { rating: "easy", label: "Facile" },
];

/** Render `view` into `root`, replacing its contents. */
export function renderReview(root: HTMLElement, view: ReviewView, actions: ReviewActions): void {
  root.replaceChildren();

  if (view.phase === "idle") {
    root.append(button("Réviser", () => actions.start(), true));
    return;
  }

  if (view.phase === "done" || !view.card) {
    root.append(note("Rien à réviser pour l'instant."));
    return;
  }

  const card = view.card;
  root.append(note(`${card.remaining} carte(s) à revoir`, "remaining"), headword(card.headword));

  if (!card.revealed) {
    root.append(button("Afficher la réponse", () => actions.reveal(), true));
    return;
  }

  if (card.gloss) root.append(line("gloss", card.gloss));
  if (card.sentence) root.append(line("sentence", `« ${card.sentence} »`));

  const grades = document.createElement("div");
  grades.className = "review-grades";
  for (const { rating, label } of GRADES) grades.append(button(label, () => actions.grade(rating), rating === "good"));
  root.append(
    grades,
    button("Je connais ✓", () => actions.markKnown(), false),
  );
}

function button(label: string, onClick: () => void, primary: boolean): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = primary ? "review-btn primary" : "review-btn";
  b.textContent = label;
  b.addEventListener("click", onClick);
  return b;
}

function note(text: string, extra = ""): HTMLElement {
  const d = document.createElement("div");
  d.className = extra ? `review-note ${extra}` : "review-note";
  d.textContent = text;
  return d;
}

function headword(text: string): HTMLElement {
  const d = document.createElement("div");
  d.className = "review-headword";
  d.textContent = text;
  return d;
}

function line(kind: string, text: string): HTMLElement {
  const d = document.createElement("div");
  d.className = `review-${kind}`;
  d.textContent = text;
  return d;
}
