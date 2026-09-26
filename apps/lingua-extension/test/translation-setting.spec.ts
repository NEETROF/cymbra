import { describe, expect, it, vi } from "vitest";
import { mountSettings } from "@/reading/settings-view.ts";
import {
  COPY,
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

const OFF: ModelStatus = { offered: true, host: "none", state: { phase: "absent" } };
const on = (state: ModelState): ModelStatus => ({ offered: true, host: "local", state });

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
    v.push({ host: "local", state: { phase: "ready" } });
    expect(v.line.textContent).toBe(COPY.ready);
    expect(v.bar.hidden).toBe(true);
  });

  it("ready: says so, with nothing to press but the checkbox", async () => {
    const v = await mount(on({ phase: "ready" }));
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
    const v = await mount(on({ phase: "ready" }), { disable: OFF });
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
    fake.push({ host: "local", state: { phase: "ready" } });
    expect(block.hidden).toBe(true);
  });

  describe("keeping the download's host awake (D4)", () => {
    it("pings while a download runs and the view is on screen, and stops when it ends", async () => {
      const v = await mount(on({ phase: "downloading", received: 0, total: 1 }));
      expect(v.c.keepAwake).toHaveBeenCalledOnce();
      expect(v.whens[0]!()).toBe(true);
      v.block.hidden = true; // the drawer switched to another view, say
      expect(v.whens[0]!()).toBe(false);
      v.push({ host: "local", state: { phase: "ready" } });
      expect(v.stops[0]).toHaveBeenCalledOnce();
    });

    it("pings for nothing else", async () => {
      const v = await mount(on({ phase: "ready" }));
      expect(v.c.keepAwake).not.toHaveBeenCalled();
    });
  });
});

describe("helpers", () => {
  it("writes sizes as the reader reads them", () => {
    expect(megabytes(25_752_472)).toBe("25,8 Mo");
    expect(megabytes(0)).toBe("0,0 Mo");
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
      storage: { local, onChanged: { addListener: (l: (typeof listeners)[number]) => listeners.push(l) } },
    });
    vi.useFakeTimers();
    try {
      const c = runtimeTranslationControls();
      await expect(c.status()).resolves.toEqual(OFF);
      await c.command("enable");
      expect(sendMessage).toHaveBeenCalledWith({ type: "lingua-model", op: "enable" });

      const seen: TranslationSetting[] = [];
      c.watch((s) => seen.push(s));
      listeners[0]!({ "unrelated-key": {} }, "local");
      listeners[0]!({ "cymbra-lingua-model-state": {} }, "sync");
      listeners[0]!({ "cymbra-lingua-model-state": {} }, "local");
      await vi.runAllTicks();
      await Promise.resolve();
      await Promise.resolve();
      expect(local.get).toHaveBeenCalledOnce();
      expect(seen).toEqual([{ host: "local", state: { phase: "absent" } }]);

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
