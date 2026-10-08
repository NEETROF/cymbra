import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import { mountSettings, type SettingsOptions } from "@/reading/settings-view.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import { makeFakePort } from "./helpers.ts";

// Réglages › Langue › « Langue maternelle » (add-lingua-native-language-choice D4): a block above the
// studied languages, its title the catalogue's, built only when two native languages ship — so a
// reader's Réglages today are what they were — and a choice confirmed there sent to the background,
// the host's port then rebuilt from the store (D3).

const TODAY = ["en-fr", "es-fr"];
const MIXED = ["en-fr", "es-fr", "es-en"];

const settle = async (): Promise<void> => {
  for (let i = 0; i < 4; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea {
  const store: Record<string, unknown> = { ...seed };
  return {
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in store).map((k) => [k, store[k]]));
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

let sent: unknown[];

beforeEach(() => {
  sent = [];
  vi.stubGlobal("chrome", {
    runtime: {
      sendMessage: async (message: unknown) => {
        sent.push(message);
        return { ok: true, changed: true };
      },
    },
    storage: { local: { get: async () => ({}), set: async () => {} }, onChanged: { addListener: () => {} } },
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.replaceChildren();
});

async function mountReglages(pairs: readonly string[], interfaceLanguage?: InterfaceLanguage, studied = ["en", "es"]) {
  const fake = makeFakePort();
  await fake.port.setStudiedLanguages(studied as ("en" | "es")[]);
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const store = fakeArea({ [ROOT_KEY]: { v: 2, backup: "STORED" } });
  const options: SettingsOptions = {
    persist: async () => {},
    store,
    sync: {
      available: async () => false,
      syncNow: async () => ({ ok: true }),
      lastSync: async () => null,
      now: () => 0,
      watch: () => {},
    },
    account: {
      state: async () => null,
      providers: async () => ({ google: false, apple: false }),
      handle: async () => null,
      signInWith: async () => null,
      signInLocal: async () => null,
      signOut: async () => null,
      takeSignInError: async () => null,
      rememberPendingEmail: async () => {},
    },
    openPage: () => {},
    pairs,
    interfaceLanguage,
  };
  mountSettings(container, fake.port, fakeArea(), options);
  await settle();
  const languagePanel = container.querySelector<HTMLElement>('.set-panel[data-tab="language"]')!;
  const titles = [...languagePanel.querySelectorAll(":scope > .set-block > .set-label")].map((l) => l.textContent);
  return { container, languagePanel, titles, fake };
}

describe("Réglages › Langue maternelle", () => {
  it("Every reader today: no block is built, the language tab holds what it held", async () => {
    const r = await mountReglages(TODAY);
    expect(r.titles).toEqual(["Langues étudiées", "Traduction", "Lecture à voix haute"]);
    expect(r.languagePanel.querySelector('input[type="radio"]')).toBeNull();
    expect(r.container.textContent).not.toContain("Langue maternelle");
  });

  it.each([
    ["fr", "Langue maternelle", "Langues étudiées"],
    ["en", "Native language", "Languages studied"],
    ["es", "Idioma materno", "Idiomas estudiados"],
  ] as const)(
    "in %s: the block above the studied languages, titled from the catalogue",
    async (language, title, studied) => {
      const r = await mountReglages(MIXED, language);
      expect(r.titles.slice(0, 2)).toEqual([title, studied]);
      const radios = [...r.languagePanel.querySelectorAll<HTMLInputElement>('input[type="radio"]')];
      expect(radios.map((radio) => [radio.value, radio.checked, radio.parentElement?.textContent])).toEqual([
        ["fr", true, "Français"],
        ["en", false, "English"],
      ]);
    },
  );

  it("Changing the native language: confirmed in Réglages, sent to the background, the host's port rebuilt from the store", async () => {
    const r = await mountReglages(MIXED, "fr");
    const english = r.languagePanel.querySelector<HTMLInputElement>('input[value="en"]')!;
    english.checked = true;
    english.dispatchEvent(new Event("change"));
    const block = english.closest<HTMLElement>(".set-block")!;
    expect(
      [...block.querySelectorAll<HTMLElement>(".set-note")].filter((n) => !n.hidden).map((n) => n.textContent),
    ).toEqual(["La langue de l'interface et des traductions de mots.", "Tu étudieras ensuite : Espagnol."]);

    block.querySelector<HTMLButtonElement>("button")!.click();
    await settle();

    expect(sent).toEqual([{ type: "lingua-native-language", native: "en" }]);
    // The stored backup restored into the port, which rebuilds its engine for its native language (D3).
    expect(r.fake.calls.restored).toEqual(["STORED"]);
  });
});
