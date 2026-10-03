import { describe, expect, it } from "vitest";
import { levelRow } from "@/onboarding/level-row.ts";

/** A level row over a view that records nothing; `estimated` as the pack says. */
function mount(language: "en" | "es", estimated: boolean): HTMLElement {
  const view = { setDeclaredLevelAt: async () => {}, setCalibration: async () => {} };
  const row = levelRow(language, view, null, estimated, async () => {});
  document.body.replaceChildren(row);
  return row;
}

const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe("onboarding — a language's level (add-lingua-spanish-levels)", () => {
  it("says estimated levels are estimated, under the chips and in the confirmation", async () => {
    const row = mount("es", true);
    expect(row.querySelector("h2")?.textContent).toBe("Quel est ton niveau d'espagnol ?");
    expect(row.querySelector(".note")?.textContent).toContain("Niveaux estimés d'après la fréquence des mots");
    row.querySelector<HTMLButtonElement>('button[data-lvl="B1"]')!.click();
    await settle();
    expect(row.querySelector(".confirm")?.textContent).toBe(
      "Niveau enregistré : B1 (estimé). Tu peux fermer cet onglet et commencer à lire.",
    );
  });

  it("reads as before for levels from a CEFR list", async () => {
    const row = mount("en", false);
    expect(row.querySelector(".note")).toBeNull();
    row.querySelector<HTMLButtonElement>('button[data-lvl="B1"]')!.click();
    await settle();
    expect(row.querySelector(".confirm")?.textContent).toBe(
      "Niveau enregistré : B1. Tu peux fermer cet onglet et commencer à lire.",
    );
  });
});
