import { describe, expect, it } from "vitest";
import { INDISTINCT_WARNING, mountColourSettings } from "@/reading/colour-settings-view.ts";
import { type AsyncStorageArea, COLOURS_KEY, type ColourPreference } from "@/state/storage.ts";
import { customColours, tokenValues } from "./colour-fixtures.ts";

// The Couleurs block of Réglages (add-lingua-colour-settings D5): a preset or every colour by
// hand, a preview, a warning when the two statuses would look alike, a way back to Cymbra.

function area(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in store).map((k) => [k, store[k]]));
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

async function mount(preference?: ColourPreference) {
  const storage = area(preference ? { [COLOURS_KEY]: preference } : {});
  const container = document.createElement("div");
  document.body.append(container);
  const view = mountColourSettings(container, storage, { read: tokenValues() });
  await view.refresh();
  const q = <T extends HTMLElement>(sel: string): T => container.querySelector<T>(sel)!;
  const labelled = <T extends HTMLElement>(label: string): T => q<T>(`[aria-label="${label}"]`);
  const preset = (name: string): HTMLButtonElement =>
    [...container.querySelectorAll<HTMLButtonElement>(".set-segment")].find((b) => b.textContent === name)!;
  const stored = (): ColourPreference => storage.store[COLOURS_KEY] as ColourPreference;
  const pick = (label: string, value: string): void => {
    const input = labelled<HTMLInputElement | HTMLSelectElement>(label);
    input.value = value;
    input.dispatchEvent(new Event("change"));
  };
  return { storage, view, container, q, labelled, preset, stored, pick };
}

const settle = () => new Promise((r) => setTimeout(r, 0));

describe("the Couleurs block", () => {
  it("shows Cymbra by default, with its colours in the controls and the preview", async () => {
    const { preset, labelled, q } = await mount();
    expect(preset("Cymbra").getAttribute("aria-pressed")).toBe("true");
    expect(preset("Perso").getAttribute("aria-pressed")).toBe("false");
    expect(labelled<HTMLInputElement>("Mots inconnus : couleur du fond").value).toBe("#ffb4ab");
    expect(labelled<HTMLSelectElement>("Mots en cours : soulignement").value).toBe("dotted");
    expect(labelled<HTMLSelectElement>("Mots inconnus : couleur du texte").value).toBe("page");
    expect(labelled<HTMLInputElement>("Mots inconnus : couleur du texte choisie").disabled).toBe(true);
    expect(labelled<HTMLSelectElement>("Page papier : texte").value).toBe("book");
    const [unknownWord] = q(".set-colour-preview").querySelectorAll("span");
    expect(unknownWord.style.textDecoration).toContain("solid");
    expect(q(".set-colour-warning").hidden).toBe(true);
  });

  it("saves a preset when the reader picks one", async () => {
    const { preset, stored, labelled } = await mount();
    preset("E-ink contrasté").click();
    await settle();
    expect(stored()).toEqual({ preset: "eink-mono" });
    expect(preset("E-ink contrasté").getAttribute("aria-pressed")).toBe("true");
    expect(labelled<HTMLSelectElement>("Mots inconnus : épaisseur").value).toBe("thick");
    expect(labelled<HTMLSelectElement>("Page papier : texte").value).toBe("chosen");
  });

  it("starts from the preset on screen when one colour is changed, and keeps the rest", async () => {
    const { pick, stored, preset } = await mount({ preset: "eink-colour" });
    pick("Mots inconnus : épaisseur", "thin");
    await settle();
    const saved = stored();
    expect(saved.preset).toBe("custom");
    if (saved.preset !== "custom") return;
    expect(saved.colours.unknown.underline).toEqual({ colour: "#b0001e", style: "solid", thickness: "thin" });
    expect(saved.colours.unknown.text).toBe("#b0001e");
    expect(saved.colours.learning.underline.style).toBe("dotted");
    expect(preset("Perso").getAttribute("aria-pressed")).toBe("true");
  });

  it("sets each part of a status and of the page", async () => {
    const { pick, stored } = await mount({ preset: "custom", colours: customColours() });
    pick("Mots en cours : couleur du fond", "#333333");
    pick("Mots en cours : fond", "light");
    pick("Mots en cours : couleur du soulignement", "#444444");
    pick("Mots en cours : soulignement", "wavy");
    pick("Mots en cours : couleur du texte choisie", "#555555");
    pick("Page papier : fond", "#666666");
    pick("Page papier : texte choisi", "#777777");
    pick("Page sombre : fond", "#888888");
    pick("Page sombre : texte", "#999999");
    await settle();
    const saved = stored();
    if (saved.preset !== "custom") throw new Error("not custom");
    expect(saved.colours.learning).toEqual({
      fill: { colour: "#333333", intensity: "light" },
      underline: { colour: "#444444", style: "wavy", thickness: "thin" },
      text: "#555555",
    });
    expect(saved.colours.paper).toEqual({ background: "#666666", text: "#777777" });
    expect(saved.colours.dark).toEqual({ background: "#888888", text: "#999999" });
  });

  it("switches a text colour between the page's own and a chosen one", async () => {
    const { pick, stored, labelled } = await mount({ preset: "custom", colours: customColours() });
    pick("Mots inconnus : couleur du texte", "page");
    await settle();
    let saved = stored();
    if (saved.preset !== "custom") throw new Error("not custom");
    expect(saved.colours.unknown.text).toBeNull();
    expect(labelled<HTMLInputElement>("Mots inconnus : couleur du texte choisie").disabled).toBe(true);
    pick("Mots inconnus : couleur du texte", "chosen");
    pick("Page papier : texte", "chosen");
    await settle();
    saved = stored();
    if (saved.preset !== "custom") throw new Error("not custom");
    expect(saved.colours.unknown.text).toBe("#0000ff");
    expect(saved.colours.paper.text).toMatch(/^#[0-9a-f]{6}$/);
    pick("Page papier : texte", "book");
    await settle();
    saved = stored();
    if (saved.preset !== "custom") throw new Error("not custom");
    expect(saved.colours.paper.text).toBeNull();
  });

  it("warns when the two statuses get the same underline, and still applies it", async () => {
    const { pick, q, stored } = await mount();
    pick("Mots en cours : soulignement", "solid");
    await settle();
    expect(q(".set-colour-warning").hidden).toBe(false);
    expect(q(".set-colour-warning").textContent).toBe(INDISTINCT_WARNING);
    expect(stored().preset).toBe("custom");
  });

  it("offers no thickness for an underline set to none", async () => {
    const { pick, labelled } = await mount();
    pick("Mots inconnus : soulignement", "none");
    await settle();
    expect(labelled<HTMLSelectElement>("Mots inconnus : épaisseur").disabled).toBe(true);
  });

  it("turns the colours on screen into the reader's own set from Perso", async () => {
    const { preset, stored } = await mount({ preset: "eink-mono" });
    preset("Perso").click();
    await settle();
    const saved = stored();
    expect(saved.preset).toBe("custom");
    if (saved.preset === "custom") expect(saved.colours.paper).toEqual({ background: "#ffffff", text: "#000000" });
  });

  it("returns to Cymbra", async () => {
    const { container, stored, preset } = await mount({ preset: "custom", colours: customColours() });
    [...container.querySelectorAll<HTMLButtonElement>(".set-reset")]
      .find((b) => b.textContent?.includes("Cymbra"))!
      .click();
    await settle();
    expect(stored()).toEqual({ preset: "cymbra" });
    expect(preset("Cymbra").getAttribute("aria-pressed")).toBe("true");
  });

  it("shows a choice made in another surface on refresh", async () => {
    const { storage, view, preset } = await mount();
    storage.store[COLOURS_KEY] = { preset: "eink-colour" };
    await view.refresh();
    expect(preset("E-ink couleur").getAttribute("aria-pressed")).toBe("true");
  });

  it("reads the preset colours off the tokens it inherits by default", async () => {
    const storage = area();
    const container = document.createElement("div");
    document.body.append(container);
    const view = mountColourSettings(container, storage);
    // jsdom does not inherit custom properties: set the token on the block the view reads.
    container.querySelector<HTMLElement>(".set-colours")!.style.setProperty("--cymbra-lingua-coral", "#010203");
    await view.refresh();
    expect(container.querySelector<HTMLInputElement>('[aria-label="Mots inconnus : couleur du fond"]')!.value).toBe(
      "#010203",
    );
  });
});
