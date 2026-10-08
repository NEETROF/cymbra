import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import { COPY_PENDING_ATTR, fillPageInLanguage, INTERFACE_LANGUAGE_KEY } from "@/i18n/index.ts";
import { onboardingCopy } from "@/onboarding/copy.ts";
import { levelRow } from "@/onboarding/level-row.ts";
import { PENDING_RULE, pageArea, refusingArea, REVEAL_KEYFRAMES } from "./helpers.ts";

// The onboarding's page (localise-lingua-account-onboarding D1): its skeleton holds no text; opened
// as onboarding.ts opens it — `fillPageInLanguage` over the preferences area, before the engine, with
// the onboarding's modules — every node holds byte for byte what the page held before this change in
// French (the inventory below is that page's text, its line breaks collapsed as a browser shows
// them), the page says its language, and the mark that hid the body is gone, a storage that cannot be
// read included. onboarding.ts itself is an entry script with no exported render; the level rows it
// builds are `levelRow`'s, asserted here in English.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const HTML = readFileSync(join(root, "src/onboarding/onboarding.html"), "utf8");
const CSS = readFileSync(join(root, "src/onboarding/onboarding.css"), "utf8");

function page(): Document {
  return new DOMParser().parseFromString(HTML, "text/html");
}

/** Every `data-copy` node of the page, by selector, with the text the page held before. */
const TEXTS: Array<[string, string]> = [
  ["title", "Bienvenue — Cymbra Lingua"],
  ["h1", "Bienvenue dans Cymbra Lingua"],
  [
    "p.lead",
    "Surligne les mots que tu ne connais pas encore pendant que tu lis, puis révise-les au bon moment — sans quitter ta page.",
  ],
  ["#languages-section h2", "Quelles langues apprends-tu ?"],
  [
    "#level-section p.muted",
    "Les mots en dessous de ton niveau ne seront pas surlignés. Rien n'est présumé tant que tu n'as pas choisi — et tu pourras changer ça à tout moment dans les réglages.",
  ],
  [".start h2", "Pour commencer"],
  [".start li:nth-of-type(1)", "Épingle l'icône Cymbra Lingua dans la barre du navigateur."],
  [".start li:nth-of-type(2)", "Ouvre une page dans la langue que tu apprends et clique l'icône pour l'analyser."],
  [".start li:nth-of-type(3)", "Clique un mot surligné pour le traduire ou l'ajouter à ton deck."],
  ["#account-section h2", "Retrouve tes mots partout (facultatif)"],
  [
    "#account-section p.muted",
    "Avec un compte Cymbra — le même que dans Cymbra Music — tes mots, ton deck et tes statistiques se synchronisent entre tes appareils. Sans compte, tout reste sur cet appareil.",
  ],
  ["#account-create", "Créer un compte"],
  ["#account-later", "Plus tard"],
];

function expectFrench(doc: Document): void {
  for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
}

const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

afterEach(() => {
  vi.restoreAllMocks();
});

describe("the onboarding's page, filled from the catalogue", () => {
  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    for (const [selector] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe("");
    // Hidden while pending — and shown after a moment even if the script never fills it.
    expect(CSS).toMatch(PENDING_RULE);
    expect(CSS).toMatch(REVEAL_KEYFRAMES);
  });

  it("Every reader today: opened as onboarding.ts opens it, every node holds the text the page held", async () => {
    const doc = page();
    const { language, copy } = await fillPageInLanguage(doc, pageArea(), onboardingCopy);
    expect(language).toBe("fr");
    expect(copy).toBe(onboardingCopy("fr"));
    expectFrench(doc);
    expect(doc.querySelectorAll("[data-copy]").length).toBe(TEXTS.length);
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(doc.documentElement.lang).toBe("fr");
    // The skeleton is the one the script drives: the sections it shows, the buttons it wires.
    for (const id of ["languages-section", "level-section", "account-section"]) {
      expect(doc.getElementById(id)?.hidden, id).toBe(true);
    }
    expect(doc.getElementById("level-rows")?.childElementCount).toBe(0);
  });

  it("A storage that cannot be read: the page still shows, in French, and says so", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = page();
    await fillPageInLanguage(doc, refusingArea(), onboardingCopy);
    expectFrench(doc);
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("An English-native reader: the welcome, the steps and the account offer are the English catalogue's", async () => {
    const doc = page();
    await fillPageInLanguage(doc, pageArea({ [INTERFACE_LANGUAGE_KEY]: "en" }), onboardingCopy);
    expect(doc.title).toBe("Welcome — Cymbra Lingua");
    expect(doc.querySelector("h1")?.textContent).toBe("Welcome to Cymbra Lingua");
    expect(doc.querySelector("#languages-section h2")?.textContent).toBe("Which languages are you learning?");
    expect(doc.querySelector(".start h2")?.textContent).toBe("To get started");
    expect(doc.querySelector("#account-create")?.textContent).toBe("Create an account");
    expect(doc.querySelector("#account-later")?.textContent).toBe("Later");
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
  });

  it("A Spanish-native reader: the page is the Spanish catalogue's and says es", async () => {
    const doc = page();
    await fillPageInLanguage(doc, pageArea({ [INTERFACE_LANGUAGE_KEY]: "es" }), onboardingCopy);
    expect(doc.querySelector("#languages-section h2")?.textContent).toBe("¿Qué idiomas aprendes?");
    expect(doc.documentElement.lang).toBe("es");
  });
});

describe("the onboarding's level row, in the interface language", () => {
  it("A French reader: the module the page reads when none is stored", () => {
    expect(onboardingCopy()).toBe(onboardingCopy("fr"));
    const row = levelRow(
      "en",
      { setDeclaredLevelAt: async () => {}, setCalibration: async () => {} },
      null,
      false,
      async () => {},
      "fr",
    );
    expect(row.querySelector('button[data-lvl=""]')?.textContent).toBe("Débutant — je pars de zéro");
  });

  it("An English-native reader: the beginner's chip and both confirmations are English", async () => {
    const view = { setDeclaredLevelAt: async () => {}, setCalibration: async () => {} };
    const row = levelRow("es", view, null, false, async () => {}, "en");
    document.body.replaceChildren(row);
    const beginner = row.querySelector<HTMLButtonElement>('button[data-lvl=""]')!;
    expect(beginner.textContent).toBe("Beginner — starting from scratch");
    row.querySelector<HTMLButtonElement>('button[data-lvl="B1"]')!.click();
    await settle();
    expect(row.querySelector(".confirm")?.textContent).toBe(
      "Level saved: B1. You can close this tab and start reading.",
    );
    beginner.click();
    await settle();
    expect(row.querySelector(".confirm")?.textContent).toBe(
      "Noted — starting from scratch. You can close this tab and start reading.",
    );
  });
});
