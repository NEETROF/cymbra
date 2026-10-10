import { describe, expect, it, vi } from "vitest";
import { SHIPPED_PAIRS } from "@/analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import { studiedLanguages as frStudiedLanguages } from "@/i18n/fr/studied-languages.ts";
import { settingsCopy } from "@/reading/settings-copy.ts";
import { mountStudiedLanguages, shippedLanguages } from "@/reading/studied-languages-view.ts";
import { makeFakePort } from "./helpers.ts";

// « Langues étudiées » (add-lingua-language-choice D2): a box per shipped language, ticked from the
// reader's profile; the last language cannot be removed; hidden when one language ships. Mounted in
// French, handed explicitly: the view has no default language (add-lingua-native-language-labels).

const BOTH = ["en-fr", "es-fr"];
/** The six pairs, French last: packs.json since enable-lingua-french. */
const SIX = ["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"];
/** fr-es under its floor: the list ending with fr-en (enable-lingua-french D2). */
const FR_EN_ALONE = ["en-fr", "es-fr", "es-en", "en-es", "fr-en"];
const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

async function mount(
  studied: StudiedLanguage[],
  pairs: readonly string[] = BOTH,
  native: NativeLanguage = "fr",
  language: InterfaceLanguage = "fr",
) {
  const { port } = makeFakePort();
  port.nativeLanguage = async () => native;
  await port.setStudiedLanguages(studied);
  const block = document.createElement("div");
  const persist = vi.fn(async () => {});
  const copy = language === "fr" ? frStudiedLanguages : settingsCopy(language).studiedLanguages;
  const view = mountStudiedLanguages(block, port, persist, pairs, copy, language);
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

  // enable-lingua-french D3: French offered after a native language's default, never to a French speaker.
  const offered = (block: HTMLElement) =>
    [...block.querySelectorAll<HTMLInputElement>("input")].map((b) => [b.dataset.language, b.nextSibling?.textContent]);

  it("offers an English speaker Spanish then French, and ticking French adds it after Spanish (enable-lingua-french)", async () => {
    const s = await mount(["es"], SIX, "en", "en");
    expect(s.block.hidden).toBe(false);
    expect(offered(s.block)).toEqual([
      ["es", "Spanish"],
      ["fr", "French"],
    ]);
    expect(s.box("es").checked).toBe(true);
    expect(s.box("fr").checked).toBe(false);
    s.box("fr").checked = true;
    s.box("fr").dispatchEvent(new Event("change"));
    await settle();
    expect(await s.port.studiedLanguages()).toEqual(["es", "fr"]);
    expect(s.persist).toHaveBeenCalledOnce();
  });

  it("offers a Spanish speaker English then French, and English alone, hidden, without fr-es (enable-lingua-french D2)", async () => {
    const s = await mount(["en"], SIX, "es", "es");
    expect(offered(s.block)).toEqual([
      ["en", "Inglés"],
      ["fr", "Francés"],
    ]);
    s.box("fr").checked = true;
    s.box("fr").dispatchEvent(new Event("change"));
    await settle();
    expect(await s.port.studiedLanguages()).toEqual(["en", "fr"]);
    const under = await mount(["en"], FR_EN_ALONE, "es", "es");
    expect(offered(under.block)).toEqual([["en", "Inglés"]]);
    expect(under.block.hidden).toBe(true);
  });

  it("offers a French speaker English and Spanish, never French (enable-lingua-french)", async () => {
    const s = await mount(["en", "es"], SIX);
    expect(offered(s.block)).toEqual([
      ["en", "Anglais"],
      ["es", "Espagnol"],
    ]);
    expect(s.block.querySelector('input[data-language="fr"]')).toBeNull();
  });

  it("offers on the bundle's list what each native language's pairs study (enable-lingua-french)", async () => {
    expect(SHIPPED_PAIRS).toEqual(SIX);
    const languages = async (native: NativeLanguage) =>
      offered((await mount(["en"], SHIPPED_PAIRS, native)).block).map(([language]) => language);
    expect(await languages("fr")).toEqual(["en", "es"]);
    expect(await languages("en")).toEqual(["es", "fr"]);
    expect(await languages("es")).toEqual(["en", "fr"]);
  });

  it("shows a French reader's boxes before the port answers, as before", () => {
    const { port } = makeFakePort();
    const block = document.createElement("div");
    mountStudiedLanguages(block, port, async () => {}, BOTH, frStudiedLanguages, "fr");
    expect(block.hidden).toBe(false);
    expect(block.querySelectorAll("input")).toHaveLength(2);
  });

  // The choice says nothing of price (remove-lingua-several-languages-offer D1): the boxes, then
  // their one note, in every interface language, whether one language is studied or both.
  it.each(["fr", "en", "es"] as const)("says nothing of price, in its one note (%s)", async (language) => {
    const { port } = makeFakePort();
    port.nativeLanguage = async () => "fr";
    await port.setStudiedLanguages(["en"]);
    const block = document.createElement("div");
    const copy = settingsCopy(language).studiedLanguages;
    const view = mountStudiedLanguages(block, port, async () => {}, BOTH, copy, language);
    await view.refresh();
    const notes = (): (string | null)[] => [...block.querySelectorAll(".set-note")].map((n) => n.textContent);
    expect(block.hidden).toBe(false);
    expect([...block.children].map((child) => child.className)).toEqual(["set-languages", "set-note"]);
    expect(notes()).toEqual([copy.studiedNote]);
    const spanish = block.querySelector<HTMLInputElement>('input[data-language="es"]')!;
    spanish.checked = true;
    spanish.dispatchEvent(new Event("change"));
    await settle();
    expect(await port.studiedLanguages()).toEqual(["en", "es"]);
    expect(notes()).toEqual([copy.studiedNote]);
  });
});
