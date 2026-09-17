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
  const view = mountSettings(container, makeFakePort().port, fakeArea(), { persist: async () => {}, sync });
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
  beforeEach(() => document.body.replaceChildren());

  it("shows when this device last synced", async () => {
    const s = mount();
    await settle();

    expect(s.block.hidden).toBe(false);
    expect(s.block.textContent).toContain("Synchronisé il y a 3 min.");
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
