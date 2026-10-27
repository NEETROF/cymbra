import { describe, expect, it, vi } from "vitest";
import { translation as COPY } from "@/i18n/fr/translation.ts";
import { settingsCopy } from "@/reading/settings-copy.ts";
import { mountSettings } from "@/reading/settings-view.ts";
import {
  costText,
  megabytes,
  mountTranslationSetting,
  runtimeTranslationControls,
  shown,
  stateText,
  type TranslationControls,
} from "@/reading/translation-setting.ts";
import type { ModelStatus } from "@/translate/model-messages.ts";
import type { ModelState, TranslationSetting } from "@/translate/setting.ts";
import { makeFakePort } from "./helpers.ts";

// « Traduction étendue » in Réglages (add-lingua-translation-delivery D2, D4, D8), one test per
// state the setting can be in, driven through a fake of the background.

const flush = async () => {
  for (let i = 0; i < 10; i++) await Promise.resolve();
};

/** What the en-fr model costs, as the background reads it from the catalogue. */
const COST = { download: 25_752_472, stored: 36_749_127 };
const OFF: ModelStatus = { offered: true, host: "none", state: { phase: "absent" }, cost: COST };
const on = (state: ModelState): ModelStatus => ({ offered: true, host: "local", state, cost: COST });

function controls(initial: ModelStatus, next: Partial<Record<string, ModelStatus>> = {}) {
  let push: ((s: TranslationSetting) => void) | null = null;
  const stops: Array<ReturnType<typeof vi.fn>> = [];
  const whens: Array<() => boolean> = [];
  const c = {
    status: vi.fn(async () => initial),
    command: vi.fn(async (op: string) => next[op] ?? initial),
    watch: (onChange: (s: TranslationSetting) => void) => void (push = onChange),
    keepAwake: vi.fn((when: () => boolean) => {
      whens.push(when);
      const stop = vi.fn();
      stops.push(stop);
      return stop;
    }),
  } satisfies TranslationControls;
  return { c, push: (s: TranslationSetting) => push?.(s), stops, whens };
}

async function mount(initial: ModelStatus, next: Partial<Record<string, ModelStatus>> = {}) {
  const fake = controls(initial, next);
  const block = document.createElement("div");
  document.body.append(block);
  const view = mountTranslationSetting(block, fake.c);
  await view.refresh();
  const q = <T extends Element>(sel: string) => block.querySelector<T>(sel)!;
  return {
    ...fake,
    block,
    view,
    box: q<HTMLInputElement>("input[type=checkbox]"),
    bar: q<HTMLProgressElement>("progress"),
    line: q<HTMLElement>("[role=status]"),
    action: q<HTMLButtonElement>("button"),
    text: () => block.textContent ?? "",
  };
}

describe("the Traduction étendue setting", () => {
  it("off: unticked, its cost stated before it is ticked, and its source credited", async () => {
    const v = await mount(OFF);
    expect(v.block.hidden).toBe(false);
    expect(v.box.checked).toBe(false);
    expect(v.text()).toContain("Traduction étendue");
    expect(v.text()).toContain("sans rien envoyer");
    expect(v.text()).toContain("25,8 Mo une fois");
    expect(v.text()).toContain("environ 200 Mo de mémoire");
    expect(v.text()).toContain("propre à cet appareil");
    expect(v.text()).toContain("MPL 2.0");
    expect(v.line.hidden).toBe(true);
    expect(v.action.hidden).toBe(true);
    expect(v.bar.hidden).toBe(true);
  });

  it("states its cost without a size when the background could not read the catalogue", async () => {
    const v = await mount({ offered: true, host: "none", state: { phase: "absent" } });
    expect(v.text()).toContain("Télécharge le modèle une fois");
    expect(v.text()).not.toContain("Mo une fois");
  });

  it("missing: says a language needs a model, its size, and offers to download it (model-state D3)", async () => {
    const v = await mount(
      on({ phase: "missing", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"], total: 26_200_000 }),
      {
        resume: on({ phase: "downloading", received: 0, total: 26_200_000 }),
      },
    );
    expect(v.box.checked).toBe(true);
    expect(v.line.textContent).toBe("Il manque un modèle pour une de tes langues. 26,2 Mo à télécharger.");
    expect(v.action.hidden).toBe(false);
    expect(v.action.textContent).toBe(COPY.download);
    v.action.click();
    await flush();
    expect(v.c.command).toHaveBeenCalledWith("resume");
  });

  it("sizes a storage failure from the catalogue", async () => {
    const v = await mount(on({ phase: "failed", reason: "storage" }));
    expect(v.line.textContent).toBe("Pas assez de place sur cet appareil pour le modèle (36,7 Mo).");
  });

  it("ticking it asks the background to turn it on", async () => {
    const v = await mount(OFF, { enable: on({ phase: "downloading", received: 0, total: 25_752_472 }) });
    v.box.checked = true;
    v.box.dispatchEvent(new Event("change"));
    expect(v.box.disabled).toBe(true); // one command at a time
    await flush();
    expect(v.c.command).toHaveBeenCalledWith("enable");
    expect(v.box.disabled).toBe(false);
    expect(v.line.textContent).toContain("0,0 Mo sur 25,8 Mo");
  });

  it("downloading: progress shown, and a cancel that turns it off", async () => {
    const v = await mount(on({ phase: "downloading", received: 12_300_000, total: 25_752_472 }), { disable: OFF });
    expect(v.box.checked).toBe(true);
    expect(v.bar.hidden).toBe(false);
    expect(v.bar.value).toBe(12_300_000);
    expect(v.bar.max).toBe(25_752_472);
    expect(v.line.textContent).toBe("Téléchargement du modèle… 12,3 Mo sur 25,8 Mo");
    expect(v.action.textContent).toBe(COPY.cancel);
    v.action.click();
    await flush();
    expect(v.c.command).toHaveBeenCalledWith("disable");
    expect(v.box.checked).toBe(false);
  });

  it("follows the download as the background reports it, in every open copy of the view", async () => {
    const v = await mount(on({ phase: "downloading", received: 0, total: 1_000_000 }));
    v.push({ host: "local", state: { phase: "downloading", received: 500_000, total: 1_000_000 } });
    expect(v.bar.value).toBe(500_000);
    v.push({ host: "local", state: { phase: "ready", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"] } });
    expect(v.line.textContent).toBe(COPY.ready);
    expect(v.bar.hidden).toBe(true);
  });

  it("ready: says so, with nothing to press but the checkbox", async () => {
    const v = await mount(on({ phase: "ready", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"] }));
    expect(v.line.textContent).toBe(COPY.ready);
    expect(v.action.hidden).toBe(true);
  });

  for (const [reason, words] of [
    ["network", "pas de connexion"],
    ["unavailable", "le serveur ne répond pas"],
    ["not-the-model", "pas le bon modèle"],
    ["storage", "Pas assez de place"],
    ["unknown", "Le téléchargement a échoué"],
  ] as const) {
    it(`failed (${reason}): explained in the reader's words, with a retry`, async () => {
      const v = await mount(on({ phase: "failed", reason }), {
        resume: on({ phase: "downloading", received: 0, total: 1 }),
      });
      expect(v.box.checked).toBe(true); // a failure leaves the setting on
      expect(v.line.textContent).toContain(words);
      expect(v.line.textContent).not.toMatch(/Error|TypeError|sha256|HTTP/);
      expect(v.action.textContent).toBe(COPY.retry);
      v.action.click();
      await flush();
      expect(v.c.command).toHaveBeenCalledWith("resume");
    });
  }

  it("interrupted: says how far it got, and offers to resume", async () => {
    const v = await mount(on({ phase: "interrupted", received: 12_300_000, total: 25_752_472 }));
    expect(v.line.textContent).toBe("Téléchargement interrompu. 12,3 Mo sur 25,8 Mo");
    expect(v.action.textContent).toBe(COPY.resume);
    v.action.click();
    await flush();
    expect(v.c.command).toHaveBeenCalledWith("resume");
  });

  it("removed: says the browser removed the model, and offers to download it again", async () => {
    const v = await mount(on({ phase: "removed" }));
    expect(v.line.textContent).toBe(COPY.removed);
    expect(v.action.textContent).toBe(COPY.again);
  });

  it("unticking it turns it off, whatever state it was in", async () => {
    const v = await mount(on({ phase: "ready", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"] }), {
      disable: OFF,
    });
    v.box.checked = false;
    v.box.dispatchEvent(new Event("change"));
    await flush();
    expect(v.c.command).toHaveBeenCalledWith("disable");
    expect(v.line.hidden).toBe(true);
  });

  it("is hidden where it is not offered — Firefox for Android (D8)", async () => {
    const v = await mount({ offered: false, host: "none", state: { phase: "absent" } });
    expect(v.block.hidden).toBe(true);
  });

  it("stays hidden until the background has answered", () => {
    const block = document.createElement("div");
    mountTranslationSetting(block, controls(OFF).c);
    expect(block.hidden).toBe(true);
  });

  it("ignores a change reported before it has read where things stand", () => {
    const fake = controls(OFF);
    const block = document.createElement("div");
    mountTranslationSetting(block, fake.c);
    fake.push({ host: "local", state: { phase: "ready", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"] } });
    expect(block.hidden).toBe(true);
  });

  describe("keeping the download's host awake (D4)", () => {
    it("pings while a download runs and the view is on screen, and stops when it ends", async () => {
      const v = await mount(on({ phase: "downloading", received: 0, total: 1 }));
      expect(v.c.keepAwake).toHaveBeenCalledOnce();
      expect(v.whens[0]!()).toBe(true);
      v.block.hidden = true; // the drawer switched to another view, say
      expect(v.whens[0]!()).toBe(false);
      v.push({ host: "local", state: { phase: "ready", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"] } });
      expect(v.stops[0]).toHaveBeenCalledOnce();
    });

    it("stops pinging and watching once its host is taken down (add-lingua-native-language-choice D3)", async () => {
      const unwatch = vi.fn();
      const fake = controls(on({ phase: "downloading", received: 0, total: 1 }));
      const view = mountTranslationSetting(document.createElement("div"), { ...fake.c, watch: () => unwatch });
      await view.refresh();
      expect(fake.c.keepAwake).toHaveBeenCalledOnce();

      view.destroy();

      expect(fake.stops[0]).toHaveBeenCalledOnce();
      expect(unwatch).toHaveBeenCalledOnce();
    });

    it("pings for nothing else", async () => {
      const v = await mount(on({ phase: "ready", models: ["en-fr/base-memory/2.0"], pairs: ["en-fr"] }));
      expect(v.c.keepAwake).not.toHaveBeenCalled();
    });
  });
});

describe("helpers", () => {
  it("writes sizes as the reader reads them", () => {
    expect(megabytes(25_752_472)).toBe("25,8 Mo");
    expect(megabytes(0)).toBe("0,0 Mo");
  });

  it("states the cost from the catalogue's sizes, or none without them", () => {
    expect(costText(COST)).toContain("Télécharge 25,8 Mo une fois");
    expect(costText(COST)).toContain("environ 200 Mo de mémoire");
    expect(costText()).toContain("Télécharge le modèle une fois");
    // A reader of Spanish: both models of its route, two models' memory (add-lingua-spanish-translation-pivot D4,
    // enable-lingua-french D6).
    const spanish = { download: 51_993_524, stored: 74_762_516, twoModels: true };
    expect(costText(spanish)).toContain("Télécharge 52,0 Mo une fois");
    expect(costText(spanish)).toContain("environ 340 Mo de mémoire");
    // A reader of Italian alone: it-fr's route, it-en then en-fr, summed from the catalogue — two models'
    // memory (enable-lingua-italian D6).
    const italian = { download: 51_994_773, stored: 73_837_254, twoModels: true };
    expect(costText(italian)).toBe(
      "Traduit tes phrases sur cet appareil, sans rien envoyer. Télécharge 52,0 Mo une fois, puis utilise " +
        "environ 340 Mo de mémoire pendant la traduction. Réglage propre à cet appareil.",
    );
    // A reader of English, Spanish and Italian: en-fr, es-en and it-en, three models held two at a time —
    // the memory of two, the download of three.
    const three = { download: 78_235_825, stored: 110_851_343, twoModels: true };
    expect(costText(three)).toContain("Télécharge 78,2 Mo une fois");
    expect(costText(three)).toContain("environ 340 Mo de mémoire");
    expect(costText(three)).not.toContain("200 Mo");
    // An English reader of Italian alone: it-en's one model — one model's memory
    // (enable-lingua-italian-for-english-speakers D5).
    const english = settingsCopy("en").translation;
    const italianAlone = costText({ download: 26_242_301, stored: 37_088_127 }, "en", english);
    expect(italianAlone).toContain("Downloads 26.2 MB once");
    expect(italianAlone).toContain("about 200 MB of memory");
    // An English reader of Spanish, French and Italian: es-en, fr-en and it-en, three one-model routes
    // held two at a time — the memory of two, the download of three.
    const threeEnglish = costText({ download: 78_718_068, stored: 111_302_527, twoModels: true }, "en", english);
    expect(threeEnglish).toContain("Downloads 78.7 MB once");
    expect(threeEnglish).toContain("about 340 MB of memory");
    expect(threeEnglish).not.toContain("about 200 MB");
    // A Spanish reader of Italian alone: it-es's route, it-en then en-es — two models' memory
    // (enable-lingua-italian-for-spanish-speakers D5).
    const spanishCopy = settingsCopy("es").translation;
    const italianSpanish = costText({ download: 51_615_655, stored: 73_664_404, twoModels: true }, "es", spanishCopy);
    expect(italianSpanish).toBe(
      "Traduce tus frases en este dispositivo, sin enviar nada. Descarga 51,6 MB una vez y luego usa " +
        "unos 340 MB de memoria durante la traducción. Ajuste propio de este dispositivo.",
    );
    // A Spanish reader of English, French and Italian: en-es, fr-en and it-en, held two at a time —
    // the memory of two, the download of three.
    const threeSpanish = costText({ download: 77_850_370, stored: 110_864_715, twoModels: true }, "es", spanishCopy);
    expect(threeSpanish).toContain("Descarga 77,9 MB una vez");
    expect(threeSpanish).toContain("unos 340 MB de memoria");
    expect(threeSpanish).not.toContain("unos 200 MB");
    // An Italian reader of English: en-it's one model — one model's memory, in the Italian interface
    // (enable-lingua-italian-speakers D4).
    const italianCopy = settingsCopy("it").translation;
    const englishItalian = costText({ download: 26_509_835, stored: 36_507_703 }, "it", italianCopy);
    expect(englishItalian).toBe(
      "Traduce le tue frasi su questo dispositivo, senza inviare nulla. Richiede di scaricare 26,5 MB una sola " +
        "volta, poi circa 200 MB di memoria durante la traduzione. Impostazione valida solo su questo dispositivo.",
    );
    expect(englishItalian).not.toContain("circa 340 MB");
    expect(stateText({ phase: "failed", reason: "storage" }, COST)).toContain("(36,7 Mo)");
    expect(stateText({ phase: "failed", reason: "storage" })).toBe(
      "Pas assez de place sur cet appareil pour le modèle.",
    );
  });

  it("says nothing about an absent model, and leaves out an unknown total", () => {
    expect(stateText({ phase: "absent" })).toBe("");
    expect(stateText({ phase: "downloading", received: 5, total: 0 })).toBe("Téléchargement du modèle…");
  });

  it("knows whether a node is on screen, through a shadow root", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const root = host.attachShadow({ mode: "closed" });
    const inner = document.createElement("div");
    root.append(inner);
    expect(shown(inner)).toBe(true);
    host.hidden = true;
    expect(shown(inner)).toBe(false);
    expect(shown(document.createElement("div"))).toBe(false); // not attached
  });
});

describe("the runtime controls", () => {
  it("ask the background, follow the two keys, and ping through keep-warm", async () => {
    const listeners: Array<(changes: Record<string, unknown>, area: string) => void> = [];
    const sendMessage = vi.fn(async () => OFF);
    const local = { get: vi.fn(async () => ({ "cymbra-lingua-translation-host": "local" })) };
    vi.stubGlobal("chrome", {
      runtime: { sendMessage },
      storage: {
        local,
        onChanged: {
          addListener: (l: (typeof listeners)[number]) => listeners.push(l),
          removeListener: (l: (typeof listeners)[number]) => listeners.splice(listeners.indexOf(l), 1),
        },
      },
    });
    vi.useFakeTimers();
    try {
      const c = runtimeTranslationControls();
      await expect(c.status()).resolves.toEqual(OFF);
      await c.command("enable");
      expect(sendMessage).toHaveBeenCalledWith({ type: "lingua-model", op: "enable" });

      const seen: TranslationSetting[] = [];
      const unwatch = c.watch((s) => seen.push(s));
      listeners[0]!({ "unrelated-key": {} }, "local");
      listeners[0]!({ "cymbra-lingua-model-state": {} }, "sync");
      listeners[0]!({ "cymbra-lingua-model-state": {} }, "local");
      await vi.runAllTicks();
      await Promise.resolve();
      await Promise.resolve();
      expect(local.get).toHaveBeenCalledOnce();
      expect(seen).toEqual([{ host: "local", state: { phase: "absent" } }]);
      unwatch?.();
      expect(listeners).toHaveLength(0);

      const stop = c.keepAwake(() => true);
      vi.advanceTimersByTime(5_000);
      expect(sendMessage).toHaveBeenCalledWith({ type: "lingua-translate-keepalive" });
      stop();
    } finally {
      vi.useRealTimers();
      vi.unstubAllGlobals();
    }
  });
});

describe("the settings view", () => {
  function sync() {
    return {
      available: async () => false,
      syncNow: async () => ({ ok: true as const }),
      lastSync: async () => null,
      now: () => 0,
      watch: () => {},
    };
  }
  const area = { get: async () => ({}), set: async () => {} };

  it("shows the row in a variant that carries the engine", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const view = mountSettings(container, makeFakePort().port, area, {
      persist: async () => {},
      store: area,
      sync: sync(),
      translation: controls(OFF).c,
    });
    await view.refresh();
    expect(container.textContent).toContain("Traduction étendue");
    expect([...container.querySelectorAll(".set-label")].map((l) => l.textContent)).toContain("Traduction");
  });

  it("has no such row in a variant without the engine (Safari)", async () => {
    const container = document.createElement("div");
    const view = mountSettings(container, makeFakePort().port, area, {
      persist: async () => {},
      store: area,
      sync: sync(),
    });
    await view.refresh();
    expect(container.textContent).not.toContain("Traduction étendue");
  });
});
