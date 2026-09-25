import { describe, expect, it } from "vitest";
import { mountBookDisplay } from "@/reading/book-display-view.ts";
import { type AsyncStorageArea, READER_DISPLAY_KEY } from "@/state/storage.ts";

// The text size and the page of a book (add-lingua-reader D10): one builder, in the reader's
// "Aa" panel and in Réglages → Livres. Each tap is one step, saved.

function area(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
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

async function mount(seed: Record<string, unknown> = {}) {
  const storage = area(seed);
  const container = document.createElement("div");
  const view = mountBookDisplay(container, storage);
  await view.refresh();
  const q = <T extends HTMLElement>(sel: string): T => container.querySelector<T>(sel)!;
  const page = (name: string): HTMLButtonElement =>
    [...container.querySelectorAll<HTMLButtonElement>(".set-segment")].find((b) => b.textContent === name)!;
  return {
    storage,
    view,
    smaller: q<HTMLButtonElement>("[aria-label='Réduire le texte']"),
    larger: q<HTMLButtonElement>("[aria-label='Agrandir le texte']"),
    size: () => q(".set-step-value").textContent,
    page,
  };
}

const settle = () => new Promise((r) => setTimeout(r, 0));

describe("mountBookDisplay", () => {
  it("shows the stored size and page", async () => {
    const v = await mount({ [READER_DISPLAY_KEY]: { textScale: 120, theme: "dark" } });
    expect(v.size()).toBe("120 %");
    expect(v.page("Sombre").getAttribute("aria-pressed")).toBe("true");
    expect(v.page("Papier").getAttribute("aria-pressed")).toBe("false");
  });

  it("steps the size by one notch a tap, and saves it", async () => {
    const v = await mount();
    v.larger.click();
    await settle();
    v.larger.click();
    await settle();
    expect(v.size()).toBe("120 %");
    expect(v.storage.store[READER_DISPLAY_KEY]).toEqual({ textScale: 120, theme: "paper" });
    v.smaller.click();
    await settle();
    expect(v.storage.store[READER_DISPLAY_KEY]).toEqual({ textScale: 110, theme: "paper" });
  });

  it("stops at either end", async () => {
    const small = await mount({ [READER_DISPLAY_KEY]: { textScale: 80, theme: "paper" } });
    expect(small.smaller.disabled).toBe(true);
    expect(small.larger.disabled).toBe(false);
    const large = await mount({ [READER_DISPLAY_KEY]: { textScale: 200, theme: "paper" } });
    expect(large.larger.disabled).toBe(true);
  });

  it("switches the page, keeping the size", async () => {
    const v = await mount({ [READER_DISPLAY_KEY]: { textScale: 140, theme: "paper" } });
    v.page("Sombre").click();
    await settle();
    expect(v.storage.store[READER_DISPLAY_KEY]).toEqual({ textScale: 140, theme: "dark" });
    expect(v.page("Sombre").getAttribute("aria-pressed")).toBe("true");
  });

  it("follows a change made in the other place once refreshed", async () => {
    const v = await mount();
    v.storage.store[READER_DISPLAY_KEY] = { textScale: 90, theme: "dark" };
    await v.view.refresh();
    expect(v.size()).toBe("90 %");
    expect(v.page("Sombre").getAttribute("aria-pressed")).toBe("true");
  });
});
