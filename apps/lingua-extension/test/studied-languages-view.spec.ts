import { describe, expect, it, vi } from "vitest";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import { studiedLanguages as frStudiedLanguages } from "@/i18n/fr/studied-languages.ts";
import { mountStudiedLanguages, shippedLanguages } from "@/reading/studied-languages-view.ts";
import { makeFakePort } from "./helpers.ts";

// « Langues étudiées » (add-lingua-language-choice D2): a box per shipped language, ticked from the
// reader's profile; the last language cannot be removed; hidden when one language ships. Mounted in
// French, handed explicitly: the view has no default language (add-lingua-native-language-labels).

const BOTH = ["en-fr", "es-fr"];
const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

async function mount(studied: StudiedLanguage[], pairs: readonly string[] = BOTH, native: NativeLanguage = "fr") {
  const { port } = makeFakePort();
  port.nativeLanguage = async () => native;
  await port.setStudiedLanguages(studied);
  const block = document.createElement("div");
  const persist = vi.fn(async () => {});
  const view = mountStudiedLanguages(block, port, persist, pairs, frStudiedLanguages, "fr");
  await view.refresh();
  const box = (language: string): HTMLInputElement =>
    block.querySelector<HTMLInputElement>(`input[data-language="${language}"]`)!;
  return { port, block, persist, view, box };
}

describe("the shipped languages", () => {
  it("are each pair's studied side, once, in the pairs' order", () => {
    expect(shippedLanguages(["en-fr", "en-de", "es-fr"])).toEqual(["en", "es"]);
    expect(shippedLanguages(["en-fr"])).toEqual(["en"]);
  });
});

describe("« Langues étudiées »", () => {
  it("ticks the reader's languages, and keeps the last one", async () => {
    const s = await mount(["en"]);
    expect(s.block.hidden).toBe(false);
    expect(s.block.textContent).toContain("Anglais");
    expect(s.block.textContent).toContain("Espagnol");
    expect(s.box("en").checked).toBe(true);
    expect(s.box("en").disabled).toBe(true); // the only one
    expect(s.box("es").checked).toBe(false);
  });

  it("adds a language after the others, and saves the profile", async () => {
    const s = await mount(["en"]);
    s.box("es").checked = true;
    s.box("es").dispatchEvent(new Event("change"));
    await settle();
    expect(await s.port.studiedLanguages()).toEqual(["en", "es"]);
    expect(s.persist).toHaveBeenCalledOnce();
    expect(s.box("en").disabled).toBe(false);
  });

  it("removes a language, and refuses to remove the last one", async () => {
    const s = await mount(["es", "en"]);
    s.box("es").checked = false;
    s.box("es").dispatchEvent(new Event("change"));
    await settle();
    expect(await s.port.studiedLanguages()).toEqual(["en"]);
    expect(s.box("en").disabled).toBe(true);
    // A forced untick of the last box is refused: the box ticks again, nothing is saved.
    s.box("en").checked = false;
    s.box("en").dispatchEvent(new Event("change"));
    await settle();
    expect(s.box("en").checked).toBe(true);
    expect(await s.port.studiedLanguages()).toEqual(["en"]);
    expect(s.persist).toHaveBeenCalledOnce();
  });

  it("follows a choice made in another context on refresh", async () => {
    const s = await mount(["en"]);
    await s.port.setStudiedLanguages(["es", "en"]);
    await s.view.refresh();
    expect(s.box("es").checked).toBe(true);
  });

  it("hides when the package ships a single language", async () => {
    const s = await mount(["en"], ["en-fr"]);
    expect(s.block.hidden).toBe(true);
  });

  it("offers a French reader the languages glossed in French alone (generalise-lingua-native-language)", async () => {
    const s = await mount(["en"], ["en-fr", "es-en", "es-fr"]);
    expect(s.block.hidden).toBe(false);
    expect([...s.block.querySelectorAll<HTMLInputElement>("input")].map((b) => b.dataset.language)).toEqual([
      "en",
      "es",
    ]);
    // Without es-fr, Spanish glossed in English is not a choice for them.
    const without = await mount(["en"], ["en-fr", "es-en"]);
    expect(without.block.hidden).toBe(true);
  });

  it("offers an English reader the languages glossed in English alone", async () => {
    const s = await mount(["es"], ["en-fr", "es-en", "es-fr"], "en");
    expect([...s.block.querySelectorAll<HTMLInputElement>("input")].map((b) => b.dataset.language)).toEqual(["es"]);
    // One language glossed in English: nothing to choose, though two are listed in French.
    expect(s.block.hidden).toBe(true);
  });

  it("shows a French reader's boxes before the port answers, as before", () => {
    const { port } = makeFakePort();
    const block = document.createElement("div");
    mountStudiedLanguages(block, port, async () => {}, BOTH, frStudiedLanguages, "fr");
    expect(block.hidden).toBe(false);
    expect(block.querySelectorAll("input")).toHaveLength(2);
  });

  it("says that several languages at once are free for now", async () => {
    const s = await mount(["en"]);
    expect(s.block.textContent).toContain("Plusieurs langues à la fois : gratuit pour l'instant.");
  });
});
