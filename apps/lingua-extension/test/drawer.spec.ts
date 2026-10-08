import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { drawer as enDrawer } from "@/i18n/en/drawer.ts";
import { Drawer, type DrawerOptions } from "@/reading/drawer.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { makeFakePort } from "./helpers.ts";

// The in-page drawer's own copy (localise-lingua-reading-surfaces D1, D3, D5): its tabs, its close
// control and the session-expired line — byte for byte what the module held before this change
// when built without a language, the English catalogue's with one — and the host's `lang`.
// What the drawer hosts (Révision, Statistiques, Réglages) is the shared mounts', tested with them.

function areaWith(items: Record<string, unknown>): AsyncStorageArea {
  return {
    async get(keys) {
      const list = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in items).map((k) => [k, items[k]]));
    },
    async set(more) {
      Object.assign(items, more);
    },
  };
}

/** A drawer whose closed shadow root is opened, so the test can look inside. */
function drawer(over: Partial<DrawerOptions> = {}): { drawer: Drawer; root: ShadowRoot } {
  const attach = HTMLElement.prototype.attachShadow;
  let root: ShadowRoot | null = null;
  const spy = vi.spyOn(HTMLElement.prototype, "attachShadow").mockImplementation(function (this: HTMLElement) {
    root = attach.call(this, { mode: "open" });
    return root;
  });
  const d = new Drawer({
    css: "",
    port: makeFakePort().port,
    area: areaWith({}),
    store: areaWith({}),
    now: () => 0,
    onChange: async () => {},
    ...over,
  });
  spy.mockRestore();
  return { drawer: d, root: root! };
}

const tabs = (root: ShadowRoot): string[] =>
  [...root.querySelectorAll<HTMLButtonElement>(".drawer-views button")].map((b) => b.textContent ?? "");

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

describe("the drawer's copy", () => {
  it("A surface without a spec today: built without a language, it is the French it was", async () => {
    const { drawer: d, root } = drawer();
    expect(tabs(root)).toEqual(["Révision", "Stats", "Réglages"]);
    const close = root.querySelector<HTMLButtonElement>(".drawer-close")!;
    expect(close.textContent).toBe("×");
    expect(close.getAttribute("aria-label")).toBe("Fermer");
    const lost = root.querySelector<HTMLElement>(".drawer-lost")!;
    expect(lost.textContent).toBe("Session expirée — reconnecte-toi dans Réglages › Données pour synchroniser.");
    expect(lost.hidden).toBe(true);
    d.setSessionLost(true);
    expect(lost.hidden).toBe(false);

    await d.openOn("review");
    const host = document.getElementById("cymbra-lingua-drawer-host")!;
    expect(host.getAttribute("lang")).toBe("fr");
    expect(host.getAttribute("data-cymbra-lingua-skip")).toBe("");
  });

  it("An English-native reader: the tabs, the close control and the line are the English catalogue's", async () => {
    const { drawer: d, root } = drawer({ language: "en", copy: enDrawer });
    expect(tabs(root)).toEqual(["Review", "Stats", "Settings"]);
    expect(root.querySelector(".drawer-close")?.getAttribute("aria-label")).toBe("Close");
    expect(root.querySelector(".drawer-lost")?.textContent).toBe(
      "Session expired — sign in again in Settings › Data to sync.",
    );
    await d.openOn("review");
    expect(document.getElementById("cymbra-lingua-drawer-host")?.getAttribute("lang")).toBe("en");
  });

  it("switches the active tab with the view, whatever the language", async () => {
    const { drawer: d, root } = drawer({ language: "en", copy: enDrawer });
    await d.openOn("stats");
    const active = [...root.querySelectorAll<HTMLButtonElement>(".drawer-views button")].filter((b) =>
      b.classList.contains("active"),
    );
    expect(active.map((b) => b.textContent)).toEqual(["Stats"]);
  });
});
