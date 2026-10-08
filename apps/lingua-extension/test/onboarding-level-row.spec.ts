import { describe, expect, it } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import { levelRow } from "@/onboarding/level-row.ts";

/** A level row over a view that records nothing; `estimated` as the pack says; handed French as the page hands it, unless told otherwise. */
function mount(language: "en" | "es", estimated: boolean, interfaceLanguage: InterfaceLanguage = "fr"): HTMLElement {
  const view = { setDeclaredLevelAt: async () => {}, setCalibration: async () => {} };
  const row = levelRow(language, view, null, estimated, async () => {}, interfaceLanguage);
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

  it("asks, names an estimated level and notes it in the interface language (add-lingua-native-language-labels)", async () => {
    // The confirmation's own sentence is still the page's French until localise-lingua-account-onboarding
    // (change 17) hands it the catalogue's; the level it quotes is already named in the interface language.
    const row = mount("es", true, "en");
    expect(row.querySelector("h2")?.textContent).toBe("What's your Spanish level?");
    expect(row.querySelector(".note")?.textContent).toBe(
      "Levels estimated from word frequency, as no freely licensed CEFR list exists for Spanish.",
    );
    row.querySelector<HTMLButtonElement>('button[data-lvl="B1"]')!.click();
    await settle();
    expect(row.querySelector(".confirm")?.textContent).toContain("B1 (estimated).");

    const spanish = mount("en", true, "es");
    expect(spanish.querySelector("h2")?.textContent).toBe("¿Cuál es tu nivel de inglés?");
    expect(spanish.querySelector(".note")?.textContent).toBe(
      "Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el inglés.",
    );
  });
});
