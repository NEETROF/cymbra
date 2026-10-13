import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import { mountSettings, type OnlineTranslationControls, type SyncControls } from "@/reading/settings-view.ts";
import type { TranslationControls } from "@/reading/translation-setting.ts";
import { type AsyncStorageArea, ONLINE_TRANSLATION_KEY } from "@/state/storage.ts";
import { FLAGS_KEY } from "@/flags/store.ts";
import { ONLINE_TRANSLATION_FLAG } from "@/translate/online.ts";
import { makeFakePort } from "./helpers.ts";

// « Traduction en ligne » in Réglages (add-lingua-online-translation, the lingua-translation delta's
// first requirement): offered only to a signed-in reader whose account has access, unticked until
// ticked, on this device alone, with a note that never names the service and a link to the policy.

function fakeArea(store: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

interface Options {
  signedIn?: boolean;
  access?: boolean;
  area?: AsyncStorageArea & { store: Record<string, unknown> };
  translation?: TranslationControls | null;
  language?: InterfaceLanguage;
}

function mount(opts: Options = {}) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  let signedIn = opts.signedIn ?? true;
  let access = opts.access ?? true;
  const accountWatchers: (() => void)[] = [];
  const flagWatchers: (() => void)[] = [];
  const sync: SyncControls = {
    available: async () => signedIn,
    syncNow: async () => ({ ok: true }),
    lastSync: async () => null,
    now: () => 0,
    watch: (onChange) => void accountWatchers.push(onChange),
  };
  const asked = vi.fn(async () => access);
  const online: OnlineTranslationControls = {
    access: asked,
    watch: (onChange) => {
      flagWatchers.push(onChange);
      return () => flagWatchers.splice(flagWatchers.indexOf(onChange), 1);
    },
  };
  const opened: string[] = [];
  const area = opts.area ?? fakeArea();
  const view = mountSettings(container, makeFakePort().port, area, {
    persist: async () => {},
    store: fakeArea(),
    sync,
    onlineTranslation: online,
    translation: opts.translation === undefined ? null : opts.translation,
    openPage: (url) => opened.push(url),
    interfaceLanguage: opts.language ?? "fr",
  });
  const label = { fr: "Traduction en ligne", en: "Online translation", es: "Traducción en línea" }[
    opts.language ?? "fr"
  ];
  const row = (): HTMLLabelElement | undefined =>
    [...container.querySelectorAll<HTMLLabelElement>("label.set-toggle")].find((l) => l.textContent === label);
  const part = (): HTMLElement => row()!.parentElement!;
  const block = (): HTMLElement => part().closest(".set-block") as HTMLElement;
  const visible = (): boolean => !!row() && !part().hidden && !block().hidden;
  const box = (): HTMLInputElement => row()!.querySelector("input")!;
  return {
    view,
    area,
    opened,
    asked,
    container,
    row,
    part,
    block,
    visible,
    box,
    setSignedIn: (v: boolean) => (signedIn = v),
    setAccess: (v: boolean) => (access = v),
    flagsChanged: () => flagWatchers.forEach((w) => w()),
    flagWatchers,
  };
}

describe("Online translation is chosen in the settings, off by default", () => {
  beforeEach(() => document.body.replaceChildren());

  it("A reader who was given access: « Traduction en ligne » shown unticked, with its note", async () => {
    const s = mount();
    await settle();
    expect(s.visible()).toBe(true);
    expect(s.box().checked).toBe(false);
    const note = s.part().querySelector(".set-note")!.textContent!;
    expect(note).toContain("envoyée à un service de traduction externe");
    expect(note).toContain("n'est conservée par personne");
    // It never names the service.
    expect(s.part().textContent).not.toMatch(/Google|Gemini|Anthropic|Claude/);
    // In the Traduction block, under « Traduction étendue »'s part.
    expect(s.block().querySelector(".set-label")?.textContent).toBe("Traduction");
  });

  it("A signed-in reader without access: not shown, and the block with it when nothing else is offered", async () => {
    const s = mount({ access: false });
    await settle();
    expect(s.part().hidden).toBe(true);
    expect(s.block().hidden).toBe(true);
  });

  it("A signed-out reader: not shown, and the flags are not even asked", async () => {
    const s = mount({ signedIn: false, access: true });
    await settle();
    expect(s.visible()).toBe(false);
    expect(s.asked).not.toHaveBeenCalled();
  });

  it("An access that cannot be read: not shown", async () => {
    const s = mount();
    s.asked.mockImplementation(async () => {
      throw new Error("no background");
    });
    await s.view.refresh();
    expect(s.visible()).toBe(false);
  });

  it("ticked: stored on this device under its own key, and shown ticked the next time", async () => {
    const area = fakeArea();
    const s = mount({ area });
    await settle();
    s.box().checked = true;
    s.box().dispatchEvent(new Event("change"));
    await settle();
    expect(area.store[ONLINE_TRANSLATION_KEY]).toBe(true);
    const again = mount({ area });
    await settle();
    expect(again.box().checked).toBe(true);
  });

  it("Another device: ticked on one, unticked on the other — the key is never synced", async () => {
    const mac = mount();
    await settle();
    mac.box().checked = true;
    mac.box().dispatchEvent(new Event("change"));
    await settle();
    const tablet = mount({ area: fakeArea() });
    await settle();
    expect(tablet.box().checked).toBe(false);
  });

  it("Access withdrawn: hidden at the next flag read; granted: shown", async () => {
    const s = mount({ access: false });
    await settle();
    expect(s.visible()).toBe(false);
    s.setAccess(true);
    s.flagsChanged();
    await settle();
    expect(s.visible()).toBe(true);
    s.setAccess(false);
    s.flagsChanged();
    await settle();
    expect(s.visible()).toBe(false);
  });

  it("Signed out from Réglages: hidden once the view is refreshed", async () => {
    const s = mount();
    await settle();
    expect(s.visible()).toBe(true);
    s.setSignedIn(false);
    await s.view.refresh();
    expect(s.visible()).toBe(false);
  });

  it("links to the privacy policy in the interface language", async () => {
    for (const [language, url] of [
      ["fr", "https://cymbra.app/confidentialite/"],
      ["en", "https://cymbra.app/en/privacy/"],
      ["es", "https://cymbra.app/es/privacidad/"],
    ] as const) {
      const s = mount({ language });
      await settle();
      const link = s.part().querySelector<HTMLButtonElement>("button.linklike")!;
      link.click();
      expect(s.opened).toEqual([url]);
    }
  });

  it("is independent of « Traduction étendue »: shown when the engine is offered too, and either may be ticked alone", async () => {
    const translation: TranslationControls = {
      status: async () => ({ offered: true, host: "none", state: { phase: "absent" } }),
      command: async () => ({ offered: true, host: "local", state: { phase: "absent" } }),
      watch: () => () => {},
      keepAwake: () => () => {},
    } as unknown as TranslationControls;
    const area = fakeArea();
    const s = mount({ translation, area });
    await settle();
    expect(s.visible()).toBe(true);
    const extended = [...s.block().querySelectorAll<HTMLLabelElement>("label.set-toggle")].find(
      (l) => l.textContent === "Traduction étendue",
    )!;
    expect(extended.closest("div")!.hidden).toBe(false);
    s.box().checked = true;
    s.box().dispatchEvent(new Event("change"));
    await settle();
    expect(area.store[ONLINE_TRANSLATION_KEY]).toBe(true);
    expect(extended.querySelector("input")!.checked).toBe(false);
    // The engine's part still shows the block alone once online translation is withdrawn.
    s.setAccess(false);
    s.flagsChanged();
    await settle();
    expect(s.part().hidden).toBe(true);
    expect(s.block().hidden).toBe(false);
  });

  it("stops hearing the flags when the view is destroyed", async () => {
    const s = mount();
    await settle();
    expect(s.flagWatchers).toHaveLength(1);
    s.view.destroy();
    expect(s.flagWatchers).toHaveLength(0);
  });
});

describe("Réglages' online translation, by default from the background's flags", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("reads the access flag through the background and follows the flags in storage", async () => {
    const listeners = new Set<(changes: Record<string, unknown>, area: string) => void>();
    let flag = false;
    const sendMessage = vi.fn(async (m: { type: string }) =>
      m.type === "flags:get" ? { values: { [ONLINE_TRANSLATION_FLAG]: flag } } : undefined,
    );
    vi.stubGlobal("chrome", {
      runtime: { sendMessage },
      storage: {
        onChanged: {
          addListener: (fn: (changes: Record<string, unknown>, area: string) => void) => listeners.add(fn),
          removeListener: (fn: (changes: Record<string, unknown>, area: string) => void) => listeners.delete(fn),
        },
      },
    });
    const container = document.createElement("div");
    document.body.replaceChildren(container);
    const sync: SyncControls = {
      available: async () => true,
      syncNow: async () => ({ ok: true }),
      lastSync: async () => null,
      now: () => 0,
      watch: () => {},
    };
    const view = mountSettings(container, makeFakePort().port, fakeArea(), {
      persist: async () => {},
      store: fakeArea(),
      sync,
      translation: null,
      openPage: () => {},
      interfaceLanguage: "fr",
    });
    await settle();
    const row = () =>
      [...container.querySelectorAll<HTMLLabelElement>("label.set-toggle")].find(
        (l) => l.textContent === "Traduction en ligne",
      )!;
    expect(row().parentElement!.hidden).toBe(true);
    flag = true;
    for (const l of listeners) l({ [FLAGS_KEY]: { newValue: {} } }, "local");
    await settle();
    expect(row().parentElement!.hidden).toBe(false);
    view.destroy();
    expect(listeners.size).toBe(0);
  });
});
