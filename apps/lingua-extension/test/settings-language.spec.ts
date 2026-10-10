import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AccountState } from "@/account/messages.ts";
import { drawer as esDrawer } from "@/i18n/es/drawer.ts";
import { sync as esSync } from "@/i18n/es/sync.ts";
import { sync as enSync } from "@/i18n/en/sync.ts";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import { type AccountControls, mountAccountSetting } from "@/reading/account-setting.ts";
import { mountBookDisplay } from "@/reading/book-display-view.ts";
import { mountColourSettings } from "@/reading/colour-settings-view.ts";
import { Drawer } from "@/reading/drawer.ts";
import { settingsCopy } from "@/reading/settings-copy.ts";
import { mountSettings, type SettingsOptions, type SyncControls } from "@/reading/settings-view.ts";
import { createSpeaker, type VoiceInfo, voiceLabel } from "@/reading/speech.ts";
import { mountStudiedLanguages } from "@/reading/studied-languages-view.ts";
import {
  costText,
  megabytes,
  mountTranslationSetting,
  stateText,
  type TranslationControls,
} from "@/reading/translation-setting.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { lastSyncLabel, syncErrorCopy } from "@/sync/status.ts";
import type { ModelStatus } from "@/translate/model-messages.ts";
import type { ModelState } from "@/translate/setting.ts";
import { makeFakePort, makeFakeSpeech, voiceFixture } from "./helpers.ts";

// Réglages speak the interface language (localise-lingua-settings): the view, its blocks and the
// formats they write, handed the language by their host — what the settings-view, colour, display,
// translation, account, sync and studied-languages specs assert in French, unchanged (no language
// is French), asserted here in English and Spanish. The names of languages — the level blocks'
// titles and notes, the studied languages' boxes, the voices' note and install help — are the labels
// module's in the interface language `mountSettings` hands its blocks (add-lingua-native-language-labels
// D2), asserted here through the mounted Réglages, the French pinned beside them.

const NNBSP = "\u202F";
/** Réglages' modules in English and in Spanish: a block handed a language is handed its module too. */
const FR = settingsCopy("fr");
const EN = settingsCopy("en");
const ES = settingsCopy("es");
const NOW = Date.UTC(2026, 8, 17, 12, 0, 0);
const MINUTE = 60_000;
const DAY = 86_400_000;
/** What the en-fr model costs, as the background reads it from the catalogue. */
const COST = { download: 25_752_472, stored: 36_749_127 };
/** A language the reader added, whose model is still to download. */
const missing = (total: number): ModelState => ({
  phase: "missing",
  models: ["en-fr/base-memory/2.0"],
  pairs: ["en-fr"],
  total,
});

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

const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

function syncControls(over: Partial<SyncControls> = {}): SyncControls {
  return {
    available: async () => true,
    syncNow: async () => ({ ok: true }),
    lastSync: async () => NOW - 3 * MINUTE,
    now: () => NOW,
    watch: () => {},
    ...over,
  };
}

function accountControls(state: AccountState | null, handle: string | null | undefined = "lea"): AccountControls {
  return {
    state: async () => state,
    providers: async () => ({ google: true, apple: false }),
    handle: async () => handle,
    signInWith: async () => null,
    signInLocal: async () => null,
    signOut: async () => null,
    takeSignInError: async () => null,
    rememberPendingEmail: async () => {},
  };
}

function translationControls(status: ModelStatus): TranslationControls {
  return {
    status: async () => status,
    command: async () => status,
    watch: () => {},
    keepAwake: () => () => {},
  };
}

async function mountReglages(
  interfaceLanguage: InterfaceLanguage | undefined,
  over: Partial<SettingsOptions> = {},
  fake = makeFakePort(),
) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  mountSettings(container, fake.port, fakeArea(), {
    persist: async () => {},
    store: fakeArea(),
    sync: syncControls(),
    account: accountControls({ signedIn: true }),
    openPage: () => {},
    interfaceLanguage,
    ...over,
  });
  await settle();
  const titles = [...container.querySelectorAll<HTMLElement>(".set-block > .set-label")].map((l) => l.textContent);
  const tabs = [...container.querySelectorAll<HTMLButtonElement>(".set-tab")].map((t) => t.textContent);
  const block = (title: string): HTMLElement =>
    [...container.querySelectorAll<HTMLElement>(".set-block")].find(
      (b) => b.querySelector(".set-label")?.textContent === title,
    )!;
  return { container, titles, tabs, block };
}

afterEach(() => {
  document.body.replaceChildren();
});

describe("Réglages", () => {
  it("An English-native reader: the tabs, the titles and the blocks are the English catalogue's", async () => {
    const r = await mountReglages("en");
    expect(r.tabs).toEqual(["Language", "Appearance", "Pages & books", "Data"]);
    expect(r.container.querySelector(".set-tabs")?.getAttribute("aria-label")).toBe("Settings");
    expect(r.titles).toEqual(
      expect.arrayContaining(["Bar on the page", "Books", "Display", "Colors", "Shortcuts & gestures", "Account"]),
    );
    expect(r.titles).toEqual(expect.arrayContaining(["Sync", "Reset"]));
    expect(r.block("Bar on the page").textContent).toContain("Show the percentage pill");
    expect(r.block("Sync").textContent).toContain("Synced 3 min ago.");
    expect(r.block("Reset").textContent).toContain("Start over from the server");

    // The count in bold, as English writes a count, the sentence around it the English one's.
    const calibration = r.container.querySelector<HTMLLabelElement>(".calib label")!;
    expect(calibration.textContent).toBe("I know the 3,000 most common words");
    expect(calibration.querySelector("b")?.textContent).toBe("3,000");
    // The slider moved: the count follows, in the same form.
    const slider = r.container.querySelector<HTMLInputElement>('.calib input[type="range"]')!;
    slider.value = "10000";
    slider.dispatchEvent(new Event("input"));
    expect(calibration.textContent).toBe("I know the 10,000 most common words");

    // The keys rendered as keys, where the English line puts them.
    const lines = [...r.block("Shortcuts & gestures").querySelectorAll("li")];
    expect(lines.map((li) => li.textContent)).toEqual([
      "Alt+Shift+S — side panel",
      "Alt+Shift+D — review panel on the page",
      "Alt+L — capture the selection",
      "Alt/Option-click (or long-press) a word — reclassify it",
    ]);
    expect([...lines[0].querySelectorAll("kbd")].map((k) => k.textContent)).toEqual(["Alt", "Shift", "S"]);
  });

  it("Every reader today: no language is French, its sentence and keys as they were", async () => {
    const r = await mountReglages(undefined);
    expect(r.tabs).toEqual(["Langue", "Apparence", "Pages & livres", "Données"]);
    const calibration = r.container.querySelector<HTMLLabelElement>(".calib label")!;
    expect([...calibration.childNodes].map((n) => n.textContent)).toEqual([
      "Je connais les ",
      "3000",
      " mots les plus courants",
    ]);
    const line = r.block("Raccourcis & gestes").querySelector("li")!;
    expect([...line.childNodes].map((n) => n.textContent)).toEqual(["Alt", "+", "Maj", "+", "S", " — panneau latéral"]);
  });

  it("A Spanish-native reader: the calibration's count as the RAE writes it", async () => {
    const fake = makeFakePort();
    fake.calls.setCalibration.push(10_000);
    const r = await mountReglages("es", {}, fake);
    const calibration = r.container.querySelector<HTMLLabelElement>(".calib label")!;
    expect(calibration.querySelector("b")?.textContent).toBe(`10${NNBSP}000`);
    expect(calibration.textContent).toBe(`Conozco las 10${NNBSP}000 palabras más corrientes`);
  });

  it("A Spanish-native reader: the tabs, the titles, the translation's cost and the last sync's date are Spanish", async () => {
    const r = await mountReglages("es", {
      sync: syncControls({ lastSync: async () => NOW - 3 * DAY }),
      translation: translationControls({ offered: true, host: "none", state: { phase: "absent" }, cost: COST }),
    });
    expect(r.tabs).toEqual(["Idioma", "Apariencia", "Páginas y libros", "Datos"]);
    expect(r.titles).toEqual(expect.arrayContaining(["Traducción", "Sincronización", "Restablecimiento"]));
    const translation = r.block("Traducción");
    expect(translation.textContent).toContain("Traducción ampliada");
    expect(translation.textContent).toContain("Descarga 25,8 MB una vez y luego usa unos 200 MB de memoria");
    // Day before month, as Spanish writes a date.
    expect(r.block("Sincronización").textContent).toMatch(/Última sincronización el 1[45]\/9\/2026\./);
  });

  /** Réglages whose speaker reads English on a French Windows, its only English voices Google's, allowed. */
  async function mountEnglishVoices(interfaceLanguage: InterfaceLanguage | undefined, readAloud: string) {
    const fake = makeFakeSpeech(voiceFixture("chrome-windows"), { remoteVoices: true });
    const r = await mountReglages(interfaceLanguage, { speaker: createSpeaker(fake.engine, "en", fake.preference) });
    const block = r.block(readAloud);
    const note = [...block.querySelectorAll<HTMLElement>(".set-note")].find((n) => n.querySelector(".set-info"))!;
    expect(note.hidden).toBe(false);
    return {
      fake,
      noVoice: note.querySelector("span")?.textContent,
      help: note.querySelector<HTMLElement>(".set-info")!.title,
      listen: [...block.querySelectorAll("button")][0] as HTMLButtonElement,
    };
  }

  it("An English-native reader: no English voice is installed, how to install one, the preview in English", async () => {
    const v = await mountEnglishVoices("en", "Read aloud");
    expect(v.noVoice).toBe("No English voice is installed on this device. ");
    expect(v.help).toContain("Add a language › English (United States), without setting it as the display language");
    expect(v.help).toContain("Manage Voices › English. Then restart the browser.");
    expect(v.help).toMatch(/turn on the online voices below\.$/);
    v.listen.click();
    expect(v.fake.spoken.map((u) => u.text)).toEqual([
      "This is how your pages will sound when Lingua reads them aloud.",
    ]);
  });

  it("A Spanish-native reader: the voice's absence and the install path name English in Spanish", async () => {
    const v = await mountEnglishVoices("es", ES.settings.readAloud);
    expect(v.noVoice).toBe("No hay ninguna voz inglesa instalada en este dispositivo. ");
    expect(v.help).toContain("Agregar un idioma › Inglés (Estados Unidos), sin definirlo como idioma de visualización");
    expect(v.help).toContain("Gestionar voces › Inglés. Después, reinicia el navegador.");
    v.listen.click();
    expect(v.fake.spoken.map((u) => u.text)).toEqual([
      "This is how your pages will sound when Lingua reads them aloud.",
    ]);
  });

  it("Every reader today: the voice's absence and the install path in French, byte for byte", async () => {
    const v = await mountEnglishVoices(undefined, "Lecture à voix haute");
    expect(v.noVoice).toBe("Aucune voix anglaise n'est installée sur cet appareil. ");
    expect(v.help).toContain("Ajouter une langue › Anglais (États-Unis), sans la définir comme langue d'affichage");
    expect(v.help).toContain("Gérer les voix › Anglais. Relance ensuite le navigateur.");
    v.listen.click();
    expect(v.fake.spoken.map((u) => u.text)).toEqual([
      "This is how your pages will sound when Lingua reads them aloud.",
    ]);
  });

  it("names the studied languages' boxes in the interface language, through mountSettings", async () => {
    const names = (block: HTMLElement): (string | null)[] =>
      [...block.querySelectorAll("label span")].map((s) => s.textContent);
    const en = await mountReglages("en", { pairs: ["en-fr", "es-fr"] });
    expect(names(en.block("Languages studied"))).toEqual(["English", "Spanish"]);
    const es = await mountReglages("es", { pairs: ["en-fr", "es-fr"] });
    expect(names(es.block("Idiomas estudiados"))).toEqual(["Inglés", "Español"]);
    // Every reader today: the French, byte for byte.
    const fr = await mountReglages(undefined, { pairs: ["en-fr", "es-fr"] });
    expect(names(fr.block("Langues étudiées"))).toEqual(["Anglais", "Espagnol"]);
  });

  it("names a voice and the automatic choice in the interface language", async () => {
    const samantha: VoiceInfo = {
      name: "Samantha",
      lang: "en-US",
      localService: true,
      default: false,
      voiceURI: "Samantha",
    };
    const fake = makeFakeSpeech([samantha]);
    const r = await mountReglages("en", { speaker: createSpeaker(fake.engine, "en", fake.preference) });
    const options = [...r.block("Read aloud").querySelectorAll("option")].map((o) => o.textContent);
    expect(options).toEqual(["Automatic (Samantha)", "Samantha — United States"]);
    expect(r.block("Read aloud").querySelector("button")?.textContent).toBe("▶ Listen");
  });
});

describe("Réglages for a speaker reading French (add-lingua-french-read-aloud)", () => {
  const PREVIEW = "Voici comment sonneront tes pages quand Lingua les lira à voix haute.";

  /** Réglages whose speaker reads `lang` over `voices`; the read-aloud block and its parts. */
  async function mountVoices(
    interfaceLanguage: InterfaceLanguage | undefined,
    readAloud: string,
    lang: string,
    voices: VoiceInfo[],
  ) {
    const fake = makeFakeSpeech(voices);
    const r = await mountReglages(interfaceLanguage, { speaker: createSpeaker(fake.engine, lang, fake.preference) });
    const block = r.block(readAloud);
    const note = [...block.querySelectorAll<HTMLElement>(".set-note")].find((n) => n.querySelector(".set-info"))!;
    const toggles = [...block.querySelectorAll<HTMLLabelElement>("label.set-toggle")];
    return {
      fake,
      block,
      note,
      select: block.querySelector("select")!,
      options: [...block.querySelectorAll("option")].map((o) => o.textContent),
      remote: (label: string) => toggles.find((t) => t.textContent === label)!,
      listen: [...block.querySelectorAll("button")][0] as HTMLButtonElement,
    };
  }

  /** Chrome on a French Windows without its three French voices: derived, not captured (D7). */
  const noLocalFrench = (): VoiceInfo[] =>
    voiceFixture("chrome-windows").filter((v) => !(v.lang.startsWith("fr") && v.localService));

  it("An English-native reader on Safari for macOS: Thomas automatic, the regions in English", async () => {
    const v = await mountVoices("en", "Read aloud", "fr", voiceFixture("safari-macos"));
    expect(v.options).toEqual(["Automatic (Thomas)", "Thomas — France", "Amélie — Canada"]);
    expect(v.note.hidden).toBe(true);
  });

  it("A Spanish-native reader on Safari for macOS: the regions in Spanish", async () => {
    const v = await mountVoices("es", ES.settings.readAloud, "fr", voiceFixture("safari-macos"));
    expect(v.options).toEqual(["Automática (Thomas)", "Thomas — Francia", "Amélie — Canadá"]);
  });

  it("the preview speaks the French sentence with the selected voice, whatever the interface", async () => {
    for (const [language, readAloud] of [
      ["en", "Read aloud"],
      ["es", ES.settings.readAloud],
    ] as const) {
      const v = await mountVoices(language, readAloud, "fr", voiceFixture("safari-macos"));
      v.select.value = "com.apple.voice.super-compact.fr-CA.Amelie";
      v.listen.click();
      expect(v.fake.spoken.map((u) => [u.text, u.voice.name])).toEqual([[PREVIEW, "Amélie"]]);
    }
  });

  it("An English-native reader, no French voice on the device: the sentence, the tooltip, the online voices off", async () => {
    const v = await mountVoices("en", "Read aloud", "fr", noLocalFrench());
    expect(v.note.hidden).toBe(false);
    expect(v.note.querySelector("span")?.textContent).toBe("No French voice is installed on this device. ");
    const help = v.note.querySelector<HTMLElement>(".set-info")!.title;
    expect(help).toContain("Add a language › French (France), without setting it as the display language");
    expect(help).toContain("Manage Voices › French. Then restart the browser.");
    expect(help).toMatch(/turn on the online voices below\.$/);
    const remote = v.remote("Use the browser's online voices");
    expect(remote.hidden).toBe(false);
    expect(remote.querySelector("input")!.checked).toBe(false);
    v.listen.click();
    expect(v.fake.spoken).toEqual([]);
    // Once allowed, Google's French voice stands in, and the preview speaks French with it.
    v.fake.prefer({ remoteVoices: true });
    expect([...v.block.querySelectorAll("option")].map((o) => o.textContent)).toEqual([
      "Automatic (Google français)",
      "Google français — France",
    ]);
    v.listen.click();
    expect(v.fake.spoken.map((u) => [u.text, u.voice.name])).toEqual([[PREVIEW, "Google français"]]);
  });

  it("A Spanish-native reader, no French voice on the device: the sentence and the tooltip in Spanish", async () => {
    const v = await mountVoices("es", ES.settings.readAloud, "fr", noLocalFrench());
    expect(v.note.querySelector("span")?.textContent).toBe(
      "No hay ninguna voz francesa instalada en este dispositivo. ",
    );
    const help = v.note.querySelector<HTMLElement>(".set-info")!.title;
    expect(help).toContain("Agregar un idioma › Francés (Francia), sin definirlo como idioma de visualización");
    expect(help).toContain("Gestionar voces › Francés. Después, reinicia el navegador.");
    const remote = v.remote("Usar las voces en línea del navegador");
    expect(remote.hidden).toBe(false);
    expect(remote.querySelector("input")!.checked).toBe(false);
  });

  it("a tag the catalogue does not name: the block without the sentences naming it, and no throw", async () => {
    // German on a French Windows: only Google's voice, remote — the case that names the language.
    const windows = await mountVoices("en", "Read aloud", "de", voiceFixture("chrome-windows"));
    expect(windows.block.hidden).toBe(false);
    expect(windows.note.hidden).toBe(true);
    expect(windows.note.textContent).toBe("ⓘ");
    expect(windows.note.querySelector<HTMLElement>(".set-info")!.title).toBe("");
    expect(windows.remote("Use the browser's online voices").hidden).toBe(false);
    expect(windows.listen.hidden).toBe(true);
    // German on a Mac: its voices listed, the automatic choice named, nothing that names the language.
    const mac = await mountVoices("en", "Read aloud", "de", voiceFixture("chrome-macos"));
    expect(mac.options[0]).toBe("Automatic (Anna)");
    expect(mac.listen.hidden).toBe(true);
    mac.listen.click();
    expect(mac.fake.spoken).toEqual([]);
  });
});

describe("the drawer's Réglages", () => {
  beforeEach(() => {
    vi.stubGlobal("chrome", {
      runtime: { sendMessage: vi.fn(async () => undefined), onMessage: { addListener: () => {} } },
      storage: { local: { get: async () => ({}), set: async () => {} }, onChanged: { addListener: () => {} } },
    });
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
  });

  it("speak the language the reading session handed the drawer", async () => {
    const attach = HTMLElement.prototype.attachShadow;
    let root: ShadowRoot | null = null;
    const spy = vi.spyOn(HTMLElement.prototype, "attachShadow").mockImplementation(function (this: HTMLElement) {
      root = attach.call(this, { mode: "open" });
      return root;
    });
    const drawer = new Drawer({
      css: "",
      port: makeFakePort().port,
      area: fakeArea(),
      store: fakeArea(),
      now: () => 0,
      onChange: async () => {},
      language: "es",
      copy: esDrawer,
    });
    spy.mockRestore();
    await drawer.openOn("settings");
    await settle();
    const tabs = [...root!.querySelectorAll<HTMLButtonElement>(".set-tab")].map((t) => t.textContent);
    expect(tabs).toEqual(["Idioma", "Apariencia", "Páginas y libros", "Datos"]);
  });
});

describe("the colour block", () => {
  it("An English-native reader: the presets, the controls' names and the preview are English", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    mountColourSettings(container, fakeArea(), { copy: EN.colours });
    await settle();
    const presets = [...container.querySelectorAll(".set-segment")].map((b) => b.textContent);
    expect(presets).toEqual(["Cymbra", "High-contrast e-ink", "Color e-ink", "Custom"]);
    expect(container.querySelector('[aria-label="Unknown words: fill color"]')).not.toBeNull();
    expect(container.querySelector('[aria-label="Paper page: background"]')).not.toBeNull();
    const preview = container.querySelector(".set-colour-preview")!;
    expect(preview.textContent).toBe("An unknown word, a word being learned and a known word.");
    // The two painted words are the message's parts, each its own span.
    expect([...preview.querySelectorAll("span")].map((s) => s.textContent)).toEqual(["unknown", "being learned"]);
  });

  it("A Spanish-native reader: the preview's words where Spanish puts them", async () => {
    const container = document.createElement("div");
    mountColourSettings(container, fakeArea(), { copy: ES.colours });
    await settle();
    const preview = container.querySelector(".set-colour-preview")!;
    expect(preview.textContent).toBe("Una palabra desconocida, una palabra en aprendizaje y una palabra conocida.");
  });
});

describe("the display block", () => {
  async function mount(language: InterfaceLanguage) {
    const container = document.createElement("div");
    mountBookDisplay(container, fakeArea({ "cymbra-lingua-reader-display": { textScale: 120 } }), {
      language,
      copy: settingsCopy(language).display,
    });
    await settle();
    return container;
  }

  it("An English-native reader: the size as English writes a percentage, the controls in English", async () => {
    const container = await mount("en");
    expect(container.querySelector(".set-step-value")?.textContent).toBe("120%");
    expect(container.querySelector(".set-step")?.getAttribute("aria-label")).toBe("Smaller text");
    expect(container.textContent).toMatch(/Text size.*Theme.*Paper.*Dark.*Page turn.*Instant.*Slide/);
  });

  it("A Spanish-native reader: the sign parted by a narrow no-break space, the copy following the language", async () => {
    const container = await mount("es");
    expect(container.querySelector(".set-step-value")?.textContent).toBe(`120${NNBSP}%`);
    expect([...container.querySelectorAll(".set-segment")].map((b) => b.textContent)).toEqual([
      "Papel",
      "Oscuro",
      "Directo",
      "Deslizado",
    ]);
  });
});

describe("the translation setting", () => {
  it("A Spanish-native reader: the cost, its sizes as Spanish writes them", () => {
    expect(costText(COST, "es", ES.translation)).toBe(
      "Traduce tus frases en este dispositivo, sin enviar nada. Descarga 25,8 MB una vez y luego usa " +
        "unos 200 MB de memoria durante la traducción. Ajuste propio de este dispositivo.",
    );
    expect(costText({ ...COST, pivot: true }, "es", ES.translation)).toContain("unos 340 MB de memoria");
    expect(costText(undefined, "es", ES.translation)).toContain("Descarga el modelo una vez");
    expect(megabytes(12_345_600_000, "es", ES.translation)).toBe(`12${NNBSP}345,6 MB`);
    expect(stateText({ phase: "failed", reason: "storage" }, COST, "es", ES.translation)).toBe(
      "No hay espacio suficiente en este dispositivo para el modelo (36,7 MB).",
    );
  });

  it("An English-native reader: the sizes, the states and the row are English", async () => {
    expect(megabytes(25_752_472, "en", EN.translation)).toBe("25.8 MB");
    const downloading = { phase: "downloading", received: 5_000_000, total: COST.download } as const;
    expect(stateText(downloading, undefined, "en", EN.translation)).toBe("Downloading the model… 5.0 MB of 25.8 MB");
    expect(stateText({ ...downloading, phase: "interrupted" }, undefined, "en", EN.translation)).toBe(
      "Download interrupted. 5.0 MB of 25.8 MB",
    );
    expect(stateText({ phase: "interrupted", received: 0, total: 0 }, undefined, "en", EN.translation)).toBe(
      "Download interrupted.",
    );
    expect(stateText(missing(COST.download), undefined, "en", EN.translation)).toBe(
      "A model is missing for one of your languages. 25.8 MB to download.",
    );
    const block = document.createElement("div");
    const view = mountTranslationSetting(
      block,
      translationControls({ offered: true, host: "local", state: missing(0), cost: COST }),
      { language: "en", copy: EN.translation },
    );
    await view.refresh();
    expect(block.querySelector("label")?.textContent).toBe("Extended translation");
    expect(block.querySelector("button")?.textContent).toBe("Download");
    expect(block.textContent).toContain("Downloads 25.8 MB once");
  });
});

describe("the account block", () => {
  it("An English-native reader: the sign-in offers, the fields and the account are English", async () => {
    const block = document.createElement("div");
    const out = mountAccountSetting(block, accountControls({ signedIn: false }), {
      openPage: () => {},
      onChange: () => {},
      copy: EN.accountSetting,
    });
    await out.refresh();
    const buttons = [...block.querySelectorAll("button")].map((b) => b.textContent);
    expect(buttons).toEqual(expect.arrayContaining(["Continue with Google", "Sign in", "Create an account"]));
    expect(block.querySelector('input[type="password"]')?.getAttribute("placeholder")).toBe("Password");
    expect(block.querySelector('input[type="password"]')?.getAttribute("aria-label")).toBe("Password");

    const inside = document.createElement("div");
    const view = mountAccountSetting(inside, accountControls({ signedIn: true }, null), {
      openPage: () => {},
      onChange: () => {},
      copy: EN.accountSetting,
    });
    await view.refresh();
    expect(inside.querySelector(".set-account-row")?.textContent).toBe("Choose a username");
    const named = document.createElement("div");
    await mountAccountSetting(named, accountControls({ signedIn: true }, "lea"), {
      openPage: () => {},
      onChange: () => {},
      copy: EN.accountSetting,
    }).refresh();
    expect(named.querySelector(".set-account-row")?.textContent).toBe("@lea");
  });
});

describe("the sync block's copy", () => {
  it("A Spanish-native reader: the last sync's age, and its date as Spanish writes it", () => {
    expect(lastSyncLabel(null, NOW, "es", esSync)).toBe("Aún no sincronizado en este dispositivo.");
    expect(lastSyncLabel(NOW - 5_000, NOW, "es", esSync)).toBe("Sincronizado ahora mismo.");
    expect(lastSyncLabel(NOW - 3 * MINUTE, NOW, "es", esSync)).toBe("Sincronizado hace 3 min.");
    expect(lastSyncLabel(NOW - 5 * 3_600_000, NOW, "es", esSync)).toBe("Sincronizado hace 5 h.");
    const at = new Date(2026, 8, 14, 12, 0, 0).getTime();
    expect(lastSyncLabel(at, at + 3 * DAY, "es", esSync)).toBe("Última sincronización el 14/9/2026.");
    expect(lastSyncLabel(at, at + 3 * DAY, "en", enSync)).toBe("Last synced on 9/14/2026.");
  });

  it("An English-native reader: a failure explained by its category, in English", () => {
    expect(syncErrorCopy("unavailable", enSync)).toBe("Server unreachable — try again later.");
    expect(syncErrorCopy("unauthenticated", enSync)).toBe("Session expired — sign in again above, under Account.");
    expect(syncErrorCopy(undefined, esSync)).toBe("La sincronización ha fallado: reintenta más tarde.");
  });
});

describe("the studied languages' block", () => {
  it("An English-native reader: its note is English, and so are the languages' names", async () => {
    const { port } = makeFakePort();
    port.nativeLanguage = async () => "fr";
    const block = document.createElement("div");
    const view = mountStudiedLanguages(block, port, async () => {}, ["en-fr", "es-fr"], EN.studiedLanguages, "en");
    await view.refresh();
    expect(block.textContent).toContain("Each page is read in whichever of your languages it holds.");
    expect([...block.querySelectorAll("label span")].map((s) => s.textContent)).toEqual(["English", "Spanish"]);
  });

  it("A Spanish-native reader: the languages' names are Spanish; French when handed French", async () => {
    const { port } = makeFakePort();
    port.nativeLanguage = async () => "fr";
    const names = (block: HTMLElement): (string | null)[] =>
      [...block.querySelectorAll("label span")].map((s) => s.textContent);
    const spanish = document.createElement("div");
    await mountStudiedLanguages(spanish, port, async () => {}, ["en-fr", "es-fr"], ES.studiedLanguages, "es").refresh();
    expect(names(spanish)).toEqual(["Inglés", "Español"]);
    const french = document.createElement("div");
    await mountStudiedLanguages(french, port, async () => {}, ["en-fr", "es-fr"], FR.studiedLanguages, "fr").refresh();
    expect(names(french)).toEqual(["Anglais", "Espagnol"]);
  });
});

describe("the level blocks name the language in the interface language", () => {
  /** Réglages with English and Spanish studied, the Spanish levels estimated. */
  async function mountLevels(interfaceLanguage: InterfaceLanguage | undefined) {
    const fake = makeFakePort();
    await fake.port.setStudiedLanguages(["en", "es"]);
    fake.port.hasLevels = async () => true;
    fake.port.levelsEstimated = async function (this: { language: string }) {
      return this.language === "es";
    };
    const r = await mountReglages(interfaceLanguage, { pairs: ["en-fr", "es-fr"] }, fake);
    await vi.waitFor(() => expect(r.container.querySelectorAll(".set-levels .set-block").length).toBe(2));
    await settle();
    const titles = [...r.container.querySelectorAll<HTMLElement>(".set-levels .set-block > .set-label")].map(
      (l) => l.textContent,
    );
    return { ...r, levelTitles: titles };
  }

  it("An English-native reader: the titles, the estimated note and the CEFR", async () => {
    const r = await mountLevels("en");
    expect(r.levelTitles).toEqual(["English level", "Estimated Spanish level"]);
    expect(r.block("Estimated Spanish level").querySelector(".set-estimate")?.textContent).toBe(
      "Levels estimated from word frequency, as no freely licensed CEFR list exists for Spanish.",
    );
  });

  it("A Spanish-native reader: the titles, the estimated note and the MCER", async () => {
    const r = await mountLevels("es");
    expect(r.levelTitles).toEqual(["Nivel de inglés", "Nivel de español estimado"]);
    expect(r.block("Nivel de español estimado").querySelector(".set-estimate")?.textContent).toBe(
      "Niveles estimados según la frecuencia de las palabras, a falta de una lista MCER de uso libre para el español.",
    );
  });

  it("Every reader today: the French titles and note, byte for byte", async () => {
    const r = await mountLevels(undefined);
    expect(r.levelTitles).toEqual(["Niveau d'anglais", "Niveau d'espagnol estimé"]);
    expect(r.block("Niveau d'espagnol estimé").querySelector(".set-estimate")?.textContent).toBe(
      "Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour l'espagnol.",
    );
  });
});

describe("a voice's label", () => {
  const voice = (name: string, lang: string): VoiceInfo => ({
    name,
    lang,
    localService: true,
    default: false,
    voiceURI: name,
  });

  it("A Spanish-native reader: the region named in Spanish", () => {
    expect(voiceLabel(voice("Samantha", "en-US"), "es", ES.settings)).toBe("Samantha — Estados Unidos");
    expect(voiceLabel(voice("Daniel", "en_GB"), "es", ES.settings)).toBe("Daniel — Reino Unido");
  });

  it("names it in English, keeps the code the runtime cannot name, and the name alone without a region", () => {
    expect(voiceLabel(voice("Daniel", "en-GB"), "en", EN.settings)).toBe("Daniel — United Kingdom");
    expect(voiceLabel(voice("Odd", "en-Latn"), "en", EN.settings)).toBe("Odd — LATN");
    expect(voiceLabel(voice("Plain", "en"), "es", ES.settings)).toBe("Plain");
    expect(voiceLabel(voice("Samantha", "en-US"))).toBe("Samantha — États-Unis"); // no language: French
  });
});
