import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ReviewView } from "@/review/session.ts";
import { type ReviewActions, renderReview } from "@/review/view.ts";

let root: HTMLElement;
let actions: ReviewActions;

beforeEach(() => {
  document.body.innerHTML = "<div id='r'></div>";
  root = document.getElementById("r")!;
  actions = { start: vi.fn(), reveal: vi.fn(), grade: vi.fn(), markKnown: vi.fn() };
});

function button(label: string): HTMLButtonElement {
  const b = [...root.querySelectorAll("button")].find((el) => el.textContent === label);
  if (!b) throw new Error(`no button "${label}"`);
  return b as HTMLButtonElement;
}

const card = (over: Partial<NonNullable<ReviewView["card"]>> = {}) => ({
  headword: "seldom",
  surface: "seldom",
  sentence: "They seldom ship.",
  gloss: "rarement",
  revealed: false,
  remaining: 3,
  ...over,
});

describe("renderReview", () => {
  it("idle shows a start button wired to start()", () => {
    renderReview(root, { phase: "idle", card: null }, actions);
    button("Réviser").click();
    expect(actions.start).toHaveBeenCalledOnce();
  });

  it("done shows a 'nothing to review' note and no buttons", () => {
    renderReview(root, { phase: "done", card: null }, actions);
    expect(root.textContent).toContain("Rien à réviser");
    expect(root.querySelectorAll("button")).toHaveLength(0);
  });

  it("reviewing (hidden) shows the headword and a reveal button, no answer", () => {
    renderReview(root, { phase: "reviewing", card: card() }, actions);
    expect(root.querySelector(".review-headword")!.textContent).toBe("seldom");
    expect(root.textContent).not.toContain("rarement"); // answer hidden
    button("Afficher la réponse").click();
    expect(actions.reveal).toHaveBeenCalledOnce();
  });

  it("reviewing (revealed) shows gloss, sentence, four grades and mark-known", () => {
    renderReview(root, { phase: "reviewing", card: card({ revealed: true }) }, actions);
    expect(root.querySelector(".review-gloss")!.textContent).toBe("rarement");
    expect(root.textContent).toContain("They seldom ship.");
    button("Correct").click();
    expect(actions.grade).toHaveBeenCalledWith("good");
    button("À revoir").click();
    expect(actions.grade).toHaveBeenCalledWith("again");
    button("Je connais ✓").click();
    expect(actions.markKnown).toHaveBeenCalledOnce();
  });

  it("never renders the word 'lemma'/'lemme'", () => {
    renderReview(root, { phase: "reviewing", card: card({ revealed: true }) }, actions);
    expect(root.textContent ?? "").not.toMatch(/lemm/i);
  });
});
