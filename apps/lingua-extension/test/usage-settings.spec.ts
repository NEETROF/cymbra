import { beforeEach, describe, expect, it, vi } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import { levelRow } from "@/onboarding/level-row.ts";
import { mountSettings, type OnlineTranslationControls, type SyncControls } from "@/reading/settings-view.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import type { SwitchView } from "@/usage/counts.ts";
import type { SwitchReply, UsageSwitchControls } from "@/usage/messages.ts";
import { makeFakePort } from "./helpers.ts";

// « Comptages anonymes » in Réglages → Données (add-lingua-usage-counts D6): one switch, wherever
// Réglages show, with a note saying what is counted, that nothing read is sent, and a link to the
// policy in the interface language. The background decides; the view moves the switch.

function fakeArea(store: Record<string, unknown> = {}): AsyncStorageArea {
  return {
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

/** The background's answers, as a switch moved in each browser would get them. */
function usageControls(
  initial: SwitchView,
  onSet: (on: boolean) => SwitchReply = (on) => ({ view: { shown: true, on } }),
) {
  let view = initial;
  const watchers: (() => void)[] = [];
  const c = {
    state: vi.fn(async () => view),
    set: vi.fn(async (on: boolean) => {
      const reply = onSet(on);
      view = reply.view;
      return reply;
    }),
    forget: vi.fn(async () => {}),
    watch: (onChange: () => void) => {
      watchers.push(onChange);
      return () => watchers.splice(watchers.indexOf(onChange), 1);
    },
  } satisfies UsageSwitchControls;
  return { c, watchers, setView: (v: SwitchView) => (view = v) };
}

function mount(usage: UsageSwitchControls, language: InterfaceLanguage = "fr", onLevelSaved = vi.fn()) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const sync: SyncControls = {
    available: async () => false,
    syncNow: async () => ({ ok: true }),
    lastSync: async () => null,
    now: () => 0,
    watch: () => undefined,
  };
  const online: OnlineTranslationControls = { access: async () => false, watch: () => undefined };
  const opened: string[] = [];
  const fake = makeFakePort();
  const view = mountSettings(container, fake.port, fakeArea(), {
    persist: async () => {},
    store: fakeArea(),
    sync,
    onlineTranslation: online,
    translation: null,
    openPage: (url) => opened.push(url),
    interfaceLanguage: language,
    usage,
    onLevelSaved,
  });
  const label = { fr: "Comptages anonymes", en: "Anonymous usage counts", es: "Recuentos anónimos" }[language];
  const block = (): HTMLElement =>
    [...container.querySelectorAll<HTMLElement>(".set-block")].find(
      (b) => b.querySelector(".set-label")?.textContent === label,
    )!;
  const box = (): HTMLInputElement => block().querySelector("input")!;
  return { container, view, opened, block, box, onLevelSaved };
}

describe("Each browser's consent rule decides, and one switch turns it off", () => {
  beforeEach(() => document.body.replaceChildren());

  it("Safari: on by default, in the Data tab, with its note and the policy in the interface language", async () => {
    const { c } = usageControls({ shown: true, on: true });
    const s = mount(c);
    await settle();
    expect(s.block().hidden).toBe(false);
    expect(s.box().checked).toBe(true);
    expect(s.block().closest<HTMLElement>('[role="tabpanel"]')!.dataset.tab).toBe("data");
    const note = s.block().querySelector(".set-note")!.textContent!;
    expect(note).toContain("aucun identifiant");
    expect(note).toContain("Rien de ce que tu lis ne quitte ton appareil.");
    [...s.block().querySelectorAll("button")].find((b) => b.textContent === "Politique de confidentialité")!.click();
    expect(s.opened).toEqual(["https://cymbra.app/confidentialite/"]);
  });

  it("Chromium, unanswered: the switch shows off, and ticking it is a yes", async () => {
    const { c } = usageControls({ shown: true, on: false });
    const s = mount(c, "en");
    await settle();
    expect(s.box().checked).toBe(false);
    s.box().click();
    await settle();
    expect(c.set).toHaveBeenCalledWith(true);
    expect(s.box().checked).toBe(true);
    [...s.block().querySelectorAll("button")].find((b) => b.textContent === "Privacy policy")!.click();
    expect(s.opened).toEqual(["https://cymbra.app/en/privacy/"]);
  });

  it("off: the background is told, and the switch stays off; on again changes nothing more here", async () => {
    const { c } = usageControls({ shown: true, on: true });
    const s = mount(c, "es");
    await settle();
    s.box().click();
    await settle();
    expect(c.set).toHaveBeenLastCalledWith(false);
    expect(s.box().checked).toBe(false);
    s.box().click();
    await settle();
    expect(c.set).toHaveBeenLastCalledWith(true);
    expect(s.box().checked).toBe(true);
  });

  it("Firefox, declined in its prompt: the switch shows off; withdrawn elsewhere, it follows", async () => {
    const usage = usageControls({ shown: true, on: false });
    const s = mount(usage.c);
    await settle();
    expect(s.box().checked).toBe(false);
    usage.setView({ shown: true, on: true });
    usage.watchers.forEach((w) => w());
    await settle();
    expect(s.box().checked).toBe(true);
  });

  it("Firefox, from the in-page drawer: turning it back on points at the toolbar", async () => {
    const { c } = usageControls({ shown: true, on: false }, () => ({
      view: { shown: true, on: false },
      fromToolbar: true,
    }));
    const s = mount(c);
    await settle();
    s.box().click();
    await settle();
    expect(s.box().checked).toBe(false);
    const msgs = [...s.block().querySelectorAll<HTMLElement>(".set-note")].filter((n) => !n.hidden);
    expect(msgs.at(-1)!.textContent).toBe(
      "Pour les rétablir, ouvre les Réglages depuis le bouton de Lingua dans la barre d'outils.",
    );
  });

  it("An older Firefox: the switch is not shown", async () => {
    const { c } = usageControls({ shown: false, on: false });
    const s = mount(c);
    await settle();
    expect(s.block().hidden).toBe(true);
  });

  it("a full reset forgets the counts, the switch left as it was", async () => {
    const { c } = usageControls({ shown: true, on: true });
    const s = mount(c);
    await settle();
    const button = (label: string): HTMLButtonElement =>
      [...s.container.querySelectorAll("button")].find((b) => b.textContent === label)!;
    button("Réinitialiser…").click();
    button("Complète — tout effacer").click();
    button("Oui, confirmer").click();
    await settle();
    expect(c.forget).toHaveBeenCalledOnce();
    expect(c.set).not.toHaveBeenCalled();
    // A partial reset keeps them.
    button("Réinitialiser…").click();
    button("Partielle — statuts + calibration (garde le deck)").click();
    await settle();
    expect(c.forget).toHaveBeenCalledOnce();
  });

  it("stops following the switch when the view is taken down", async () => {
    const usage = usageControls({ shown: true, on: true });
    const s = mount(usage.c);
    await settle();
    expect(usage.watchers).toHaveLength(1);
    s.view.destroy();
    expect(usage.watchers).toHaveLength(0);
  });
});

describe("A level saved is the counts' « set up »", () => {
  beforeEach(() => document.body.replaceChildren());

  it("in Réglages, « Débutant » included", async () => {
    const { c } = usageControls({ shown: true, on: true });
    const s = mount(c);
    await settle();
    const beginner = s.container.querySelector<HTMLButtonElement>('button.lvl[data-lvl=""]')!;
    beginner.click();
    await settle();
    expect(s.onLevelSaved).toHaveBeenCalledTimes(1);
    s.container.querySelector<HTMLButtonElement>('button.lvl[data-lvl="B1"]')!.click();
    await settle();
    expect(s.onLevelSaved).toHaveBeenCalledTimes(2);
  });

  it("on the welcome tab, once the level is saved", async () => {
    const saved = vi.fn();
    const order: string[] = [];
    const view = {
      setDeclaredLevelAt: async () => void order.push("level"),
      setCalibration: async () => {},
    };
    const row = levelRow(
      "en",
      view,
      null,
      false,
      async () => void order.push("persist"),
      "fr",
      () => {
        order.push("set up");
        saved();
      },
    );
    document.body.replaceChildren(row);
    row.querySelector<HTMLButtonElement>('button[data-lvl="A2"]')!.click();
    await settle();
    expect(saved).toHaveBeenCalledOnce();
    expect(order).toEqual(["level", "persist", "set up"]);
  });
});
