import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import { nativeLanguage as enCopy } from "@/i18n/en/native-language.ts";
import { nativeLanguage as esCopy } from "@/i18n/es/native-language.ts";
import { nativeLanguage as frCopy } from "@/i18n/fr/native-language.ts";
import { mountNativeStep, presetThenStart } from "@/onboarding/native-step.ts";
import { mountNativeCta } from "@/popup/native-cta.ts";
import { mountNativeLanguage, nativeLanguageCopy } from "@/reading/native-language-view.ts";
import {
  changeNativeLanguage,
  NATIVE_CHOSEN_KEY,
  NATIVE_LANGUAGE_MESSAGE,
  type NativeLanguageHost,
  type NativeLanguageReply,
  presetNativeLanguage,
} from "@/state/native-language.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import { ownerArea, type StoreChangeReason } from "@/state/store.ts";

// The reader's choice of native language, one view in three places (add-lingua-native-language-choice
// D4): Réglages' block is asserted with Réglages (settings-native-language.spec.ts); here the view,
// the onboarding's first question and the popup's first-run call to action.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
/** Today's package: French-native pairs alone. */
const TODAY = ["en-fr", "es-fr"];
/** es-en shipping beside them (change 34). */
const MIXED = ["en-fr", "es-fr", "es-en"];
/** The engine's names for the languages (state/profile.ts). */
const NAMES = { fr: "French", en: "English", es: "Spanish" } as const;

const settle = async (): Promise<void> => {
  for (let i = 0; i < 4; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
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

/** A store holding a backup of `native` studying `studied`, its profile as the engine writes it. */
function storeOf(native: "French" | "English", studied: ("English" | "Spanish")[]) {
  return fakeArea({
    [ROOT_KEY]: { v: 2, backup: JSON.stringify({ profile: { native_language: native, studied_languages: studied } }) },
  });
}

function profile(native: NativeLanguage, studied: StudiedLanguage[]) {
  return async () => ({ native, studied });
}

const radios = (container: ParentNode) =>
  [...container.querySelectorAll<HTMLInputElement>("input[type=radio]")].map((r) => ({
    native: r.value,
    checked: r.checked,
    name: r.parentElement?.textContent,
    lang: r.nextElementSibling?.getAttribute("lang"),
  }));

const visibleNotes = (container: ParentNode) =>
  [...container.querySelectorAll<HTMLElement>(".set-note, .set-warn")]
    .filter((n) => !n.hidden)
    .map((n) => n.textContent);

const button = (container: ParentNode) => container.querySelector<HTMLButtonElement>("button")!;

function pick(container: ParentNode, native: string): void {
  const radio = container.querySelector<HTMLInputElement>(`input[value="${native}"]`)!;
  radio.checked = true;
  radio.dispatchEvent(new Event("change"));
}

describe("the view (D4)", () => {
  it("Every reader today: one native language ships, nothing is mounted", () => {
    const container = document.createElement("div");
    container.innerHTML = "<p>before</p>";
    const view = mountNativeLanguage(container, {
      language: "fr",
      copy: frCopy,
      profile: profile("fr", ["en"]),
      pairs: TODAY,
    });
    expect(view).toBeNull();
    expect(container.innerHTML).toBe("<p>before</p>");
  });

  it("A native language with no pair: Spanish is not offered; each language in its own name", async () => {
    const container = document.createElement("div");
    const view = mountNativeLanguage(container, {
      language: "es",
      copy: esCopy,
      profile: profile("fr", ["en"]),
      pairs: MIXED,
    })!;
    await view.refresh();

    expect(radios(container)).toEqual([
      { native: "fr", checked: true, name: "Français", lang: "fr" },
      { native: "en", checked: false, name: "English", lang: "en" },
    ]);
    expect(container.querySelector("[role=radiogroup]")?.getAttribute("aria-label")).toBe("Leo en…");
    // The current language: nothing to confirm, nothing said but what the choice sets.
    expect(visibleNotes(container)).toEqual([esCopy.note]);
    expect(button(container).hidden).toBe(true);
  });

  it("A native language that was the only studied one: the choice says the reader will study Spanish, before confirming", async () => {
    const container = document.createElement("div");
    const choose = vi.fn(async (): Promise<NativeLanguageReply> => ({ ok: true, changed: true }));
    const chosen = vi.fn();
    const view = mountNativeLanguage(container, {
      language: "fr",
      copy: frCopy,
      profile: profile("fr", ["en"]),
      choose,
      onChosen: chosen,
      pairs: MIXED,
    })!;
    await view.refresh();

    pick(container, "en");
    expect(visibleNotes(container)).toEqual([frCopy.note, "Tu étudieras ensuite : Espagnol."]);
    expect(button(container).hidden).toBe(false);
    expect(button(container).textContent).toBe("Confirmer");
    expect(choose).not.toHaveBeenCalled();

    // Back on the current language: nothing to confirm again.
    pick(container, "fr");
    expect(visibleNotes(container)).toEqual([frCopy.note]);
    expect(button(container).hidden).toBe(true);

    pick(container, "en");
    button(container).click();
    await settle();
    expect(choose).toHaveBeenCalledWith("en");
    expect(chosen).toHaveBeenCalledWith("en", { ok: true, changed: true });
  });

  it("Changing the native language: a reader of English and Spanish is told they keep Spanish alone, in English", async () => {
    const container = document.createElement("div");
    const view = mountNativeLanguage(container, {
      language: "en",
      copy: enCopy,
      profile: profile("fr", ["en", "es"]),
      choose: async () => ({ ok: true, changed: true }),
      pairs: MIXED,
    })!;
    await view.refresh();
    pick(container, "en");
    expect(visibleNotes(container)).toEqual([enCopy.note, "You'll then study: Spanish."]);
  });

  it("says a change that could not be made, and keeps the reader's pick", async () => {
    const container = document.createElement("div");
    const view = mountNativeLanguage(container, {
      language: "fr",
      copy: frCopy,
      profile: profile("fr", ["en"]),
      choose: async () => ({ ok: false, error: "failed" }),
      pairs: MIXED,
    })!;
    await view.refresh();
    pick(container, "en");
    button(container).click();
    await settle();
    expect(visibleNotes(container)).toEqual([frCopy.note, "Tu étudieras ensuite : Espagnol.", frCopy.failed]);
    expect(radios(container).find((r) => r.checked)?.native).toBe("en");
    expect(button(container).disabled).toBe(false);
  });

  it("picks its copy by interface language", () => {
    expect(nativeLanguageCopy("fr")).toBe(frCopy);
    expect(nativeLanguageCopy("en")).toBe(enCopy);
    expect(nativeLanguageCopy("es")).toBe(esCopy);
  });
});

describe("the onboarding's first question (D4)", () => {
  const HTML = readFileSync(join(root, "src/onboarding/onboarding.html"), "utf8");
  const page = () => new DOMParser().parseFromString(HTML, "text/html");

  function watcher() {
    let listener: ((keys: string[], reason?: StoreChangeReason) => void) | null = null;
    return {
      watch: (fn: NonNullable<typeof listener>) => (listener = fn),
      fire: (native: NativeLanguage) => listener?.([ROOT_KEY], { type: "native-language", native }),
    };
  }

  it("Every reader today: the page holds no question", async () => {
    const doc = page();
    const before = doc.body.innerHTML;
    const step = mountNativeStep(doc.getElementById("languages-section")!, "fr", storeOf("French", ["English"]), {
      pairs: TODAY,
      watch: watcher().watch,
    });
    await settle();
    expect(step).toBeNull();
    expect(doc.body.innerHTML).toBe(before);
  });

  it("A new install when two native languages ship: the question comes first, in the language preset, English selected", async () => {
    const doc = page();
    const step = mountNativeStep(doc.getElementById("languages-section")!, "en", storeOf("English", ["Spanish"]), {
      pairs: MIXED,
      watch: watcher().watch,
    })!;
    await settle();
    expect(step.id).toBe("native-section");
    expect(step.tagName).toBe("SECTION");
    expect(step.nextElementSibling?.id).toBe("languages-section");
    expect(step.querySelector("h2")?.textContent).toBe("I read in…");
    expect(radios(step)).toEqual([
      { native: "fr", checked: false, name: "Français", lang: "fr" },
      { native: "en", checked: true, name: "English", lang: "en" },
    ]);
  });

  it("the preset confirmed in the same step is the reader's answer (M3)", async () => {
    const doc = page();
    const step = mountNativeStep(doc.getElementById("languages-section")!, "en", storeOf("English", ["Spanish"]), {
      pairs: MIXED,
      watch: watcher().watch,
    })!;
    await settle();
    // The view's default sends the choice to the background.
    const sendMessage = vi.fn(async () => ({ ok: true, changed: false }));
    vi.stubGlobal("chrome", { runtime: { sendMessage } });
    expect(button(step).hidden).toBe(false);
    button(step).click();
    await settle();
    expect(sendMessage).toHaveBeenCalledWith({ type: "lingua-native-language", native: "en" });
    expect(button(step).hidden).toBe(true); // answered: nothing more to confirm
    vi.unstubAllGlobals();
  });

  it("the page reloads when the native language changes, not for its own preset", async () => {
    const doc = page();
    const { watch, fire } = watcher();
    const reload = vi.fn();
    mountNativeStep(doc.getElementById("languages-section")!, "en", storeOf("English", ["Spanish"]), {
      pairs: MIXED,
      watch,
      reload,
    });
    fire("en");
    await settle();
    expect(reload).not.toHaveBeenCalled();
    fire("fr");
    await settle();
    expect(reload).toHaveBeenCalledOnce();
  });

  it("presets before the page starts, and starts whatever the preset did", async () => {
    const order: string[] = [];
    await presetThenStart(async () => void order.push("start"), {
      preferences: fakeArea(),
      browserLanguage: "de-DE",
      pairs: MIXED,
      choose: async (native, preset) => {
        order.push(`preset ${native} ${preset}`);
        throw new Error("background asleep");
      },
    });
    expect(order).toEqual(["preset en true", "start"]);
  });
});

describe("the popup's first run (D4)", () => {
  const HTML = readFileSync(join(root, "src/popup/popup.html"), "utf8");
  const page = () => new DOMParser().parseFromString(HTML, "text/html");

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("Every reader today: the preset reads nothing, and the popup holds no call to action", async () => {
    const preferences = fakeArea();
    const reads = vi.spyOn(preferences, "get");
    const choose = vi.fn();
    expect(await presetNativeLanguage({ preferences, browserLanguage: "en-US", choose })).toBe(false);
    expect(reads).not.toHaveBeenCalled();
    expect(choose).not.toHaveBeenCalled();

    const doc = page();
    const before = doc.body.innerHTML;
    expect(mountNativeCta(doc, "fr", storeOf("French", ["English"]), { pairs: TODAY })).toBeNull();
    expect(doc.body.innerHTML).toBe(before);
  });

  it("Safari without the onboarding: preset from the browser's language, asked before the level, asked no more once answered", async () => {
    // The background's side as it runs (state/native-language.ts): a page was read before the popup
    // opened, so the store holds a French backup with exposures and nothing else.
    const preferences = fakeArea();
    const backup = JSON.stringify({ schema_version: 2, exposure: { English: { the: 3 } } });
    const store = ownerArea(fakeArea({ [ROOT_KEY]: { v: 2, backup } }), () => {}, preferences, MIXED);
    const host: NativeLanguageHost = {
      store,
      preferences,
      ensureBackup: async () => {},
      reprofile: async (from, native, studied) =>
        JSON.stringify({
          ...(JSON.parse(from) as object),
          profile: { native_language: NAMES[native], studied_languages: studied.map((l) => NAMES[l]) },
        }),
      exclusive: (task) => task(),
      rehydrate: () => {},
      pairs: MIXED,
    };
    const sent: [NativeLanguage, boolean][] = [];
    const choose = (native: NativeLanguage, preset = false): Promise<NativeLanguageReply> => {
      sent.push([native, preset]);
      return changeNativeLanguage(host, { type: NATIVE_LANGUAGE_MESSAGE, native, ...(preset ? { preset } : {}) });
    };
    const popup = async () => {
      const asked = await presetNativeLanguage({ preferences, browserLanguage: "en-GB", choose, pairs: MIXED });
      const doc = page();
      const cta = asked ? mountNativeCta(doc, "en", store, { pairs: MIXED, choose: (native) => choose(native) }) : null;
      await settle();
      return { asked, doc, cta };
    };

    // The first popup of a new install whose onboarding never opened: preset, then asked.
    const first = await popup();
    expect(first.asked).toBe(true);
    expect(sent).toEqual([["en", true]]);
    const cta = first.cta!;
    // Above the page's figures, which show once a content script answers, and the level's call to action.
    expect(cta.nextElementSibling?.id).toBe("setup");
    expect(
      cta.compareDocumentPosition(first.doc.getElementById("level-cta")!) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(cta.querySelector(".native-question")?.textContent).toBe("I read in…");
    expect(radios(cta).find((r) => r.checked)?.native).toBe("en");
    expect(button(cta).hidden).toBe(false);

    // The popover closed before the reader answered: the next one asks again.
    const second = await popup();
    expect(second.asked).toBe(true);
    expect(second.cta).not.toBeNull();

    // The preset confirmed is the answer: the question goes, and no popup asks again.
    button(second.cta!).click();
    await settle();
    expect(second.doc.getElementById("native-cta")).toBeNull();
    expect(preferences.store[NATIVE_CHOSEN_KEY]).toBe(true);
    const third = await popup();
    expect(third.asked).toBe(false);
    expect(sent).toEqual([
      ["en", true],
      ["en", true],
      ["en", false],
    ]);
  });

  it("An installed extension is not asked: the update's marker keeps the popup from presetting", async () => {
    const preferences = fakeArea({ [NATIVE_CHOSEN_KEY]: true });
    const choose = vi.fn();
    expect(await presetNativeLanguage({ preferences, browserLanguage: "en-US", choose, pairs: MIXED })).toBe(false);
    expect(choose).not.toHaveBeenCalled();
  });

  it("a preset that cannot be read is skipped, and said", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const preferences: AsyncStorageArea = {
      get: async () => {
        throw new Error("storage unavailable");
      },
      set: async () => {},
    };
    expect(await presetNativeLanguage({ preferences, browserLanguage: "en", pairs: MIXED })).toBe(false);
    expect(warn).toHaveBeenCalledOnce();
  });
});
