import { beforeEach, describe, expect, it, vi } from "vitest";
import { mountSettings, type SyncControls } from "@/reading/settings-view.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import type { SyncReply } from "@/sync/messages.ts";
import { makeFakePort } from "./helpers.ts";

function fakeArea(): AsyncStorageArea {
  const store: Record<string, unknown> = {};
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

const NOW = Date.UTC(2026, 8, 17, 12, 0, 0);

const opened: string[] = [];

function mount(overrides: Partial<SyncControls> = {}) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const watchers: (() => void)[] = [];
  const syncNow = vi.fn(async (): Promise<SyncReply> => ({ ok: true }));
  const sync: SyncControls = {
    available: async () => true,
    syncNow,
    lastSync: async () => NOW - 3 * 60_000,
    now: () => NOW,
    watch: (onChange) => void watchers.push(onChange),
    ...overrides,
  };
  const view = mountSettings(container, makeFakePort().port, fakeArea(), {
    persist: async () => {},
    store: fakeArea(),
    sync,
    openPage: (url) => opened.push(url),
  });
  const block = [...container.querySelectorAll<HTMLElement>(".set-block")].find((b) =>
    b.textContent?.startsWith("Synchronisation"),
  );
  if (!block) throw new Error("no Synchronisation block");
  const button = [...block.querySelectorAll("button")].find((b) => b.textContent === "Synchroniser maintenant");
  if (!button) throw new Error("no sync button");
  return { view, block, button, syncNow, notify: () => watchers.forEach((w) => w()) };
}

/** Let the view's floating promises settle. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe("Réglages — Synchronisation", () => {
  beforeEach(() => {
    document.body.replaceChildren();
    opened.length = 0;
  });

  it("shows when this device last synced", async () => {
    const s = mount();
    await settle();

    expect(s.block.hidden).toBe(false);
    expect(s.block.textContent).toContain("Synchronisé il y a 3 min.");
  });

  it("offers to start again from the server, instead of a local wipe the sync undoes", async () => {
    // Signed in, emptying this device erases nothing: the exchange pulls it all back. The
    // action is offered for what it does, and a real erasure is pointed at the account.
    const syncNow = vi.fn(async (): Promise<SyncReply> => ({ ok: true }));
    const s = mount({ syncNow });
    await settle();

    const restart = [...s.block.parentElement!.querySelectorAll("button")].find(
      (b) => b.textContent === "Repartir du serveur",
    );
    const wipe = [...s.block.parentElement!.querySelectorAll("button")].find((b) => b.textContent === "Réinitialiser…");
    expect(restart?.closest("div")?.hidden).toBe(false);
    expect(wipe?.closest("div")?.hidden).toBe(true); // the local wipe is not offered

    restart?.click();
    await settle();

    expect(syncNow).toHaveBeenCalledTimes(1); // emptied, then pulled back
    expect(s.block.parentElement?.textContent).toContain("Repris depuis le serveur.");
  });

  it("keeps the local reset for a reader with no account", async () => {
    const s = mount({ available: async () => false });
    await settle();

    const buttons = [...s.block.parentElement!.querySelectorAll("button")].map((b) => b.textContent);
    expect(buttons).toContain("Réinitialiser…");
    const restart = [...s.block.parentElement!.querySelectorAll("button")].find(
      (b) => b.textContent === "Repartir du serveur",
    );
    expect(restart?.closest("div")?.hidden).toBe(true);
  });

  it("opens the account page through the background, which the drawer cannot do itself", async () => {
    // In the drawer this code runs in the visited page, where `chrome.tabs` does not exist:
    // the link did nothing at all (dogfooding, build 100/101).
    const s = mount();
    await settle();

    const link = [...s.block.parentElement!.querySelectorAll("button")].find(
      (b) => b.textContent === "Gérer mes données",
    );
    link?.click();

    expect(opened).toEqual(["account.html#data"]);
  });

  it("stays hidden while signed out", async () => {
    const s = mount({ available: async () => false });
    await settle();

    expect(s.block.hidden).toBe(true);
  });

  it("syncs on demand and refreshes the label", async () => {
    let last = NOW - 3 * 60_000;
    const syncNow = vi.fn(async (): Promise<SyncReply> => {
      last = NOW;
      return { ok: true };
    });
    const s = mount({ lastSync: async () => last, syncNow });
    await settle();

    s.button.click();
    await settle();

    expect(syncNow).toHaveBeenCalledTimes(1);
    expect(s.block.textContent).toContain("Synchronisé à l'instant.");
  });

  it("explains a failure by category", async () => {
    const s = mount({ syncNow: async () => ({ ok: false, error: "unavailable" }) });
    await settle();

    s.button.click();
    await settle();

    expect(s.block.textContent).toContain("Serveur injoignable");
    expect(s.button.disabled).toBe(false);
  });

  it("follows a sync that finished in the background", async () => {
    let last = NOW - 3 * 60_000;
    const s = mount({ lastSync: async () => last });
    await settle();

    last = NOW;
    s.notify();
    await settle();

    expect(s.block.textContent).toContain("Synchronisé à l'instant.");
  });
});
