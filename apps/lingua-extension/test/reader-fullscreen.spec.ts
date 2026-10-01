import { describe, expect, it, vi } from "vitest";
import { documentFullscreen, type FullscreenDocument } from "@/reader/fullscreen.ts";

// The reader page's fullscreen over the document (add-lingua-reader-fullscreen, design D2): the
// standard API, the `webkit` one of iPadOS before 16.4, and none at all (Safari on iPhone).

function standardDoc(enabled = true) {
  const listeners = new Map<string, () => void>();
  const doc = {
    fullscreenEnabled: enabled,
    fullscreenElement: null as Element | null,
    documentElement: { requestFullscreen: vi.fn(async () => {}) },
    exitFullscreen: vi.fn(async () => {}),
    addEventListener: (type: string, l: () => void) => void listeners.set(type, l),
  };
  return { doc, listeners };
}

describe("documentFullscreen", () => {
  it("drives the standard API, and listens to fullscreenchange", async () => {
    const { doc, listeners } = standardDoc();
    const host = documentFullscreen(doc);
    expect(host.available).toBe(true);
    expect(host.active()).toBe(false);
    await host.enter();
    expect(doc.documentElement.requestFullscreen).toHaveBeenCalledOnce();
    doc.fullscreenElement = {} as Element;
    expect(host.active()).toBe(true);
    await host.exit();
    expect(doc.exitFullscreen).toHaveBeenCalledOnce();
    const changed = vi.fn();
    host.onChange(changed);
    listeners.get("fullscreenchange")!();
    expect(changed).toHaveBeenCalledOnce();
  });

  it("is unavailable where the browser says fullscreen is not enabled", () => {
    expect(documentFullscreen(standardDoc(false).doc).available).toBe(false);
  });

  it("passes a refusal on to the caller", async () => {
    const { doc } = standardDoc();
    doc.documentElement.requestFullscreen.mockRejectedValue(new TypeError("no gesture"));
    await expect(documentFullscreen(doc).enter()).rejects.toThrow("no gesture");
  });

  it("falls back to the webkit names", async () => {
    const listeners = new Map<string, () => void>();
    const doc: FullscreenDocument & { documentElement: { webkitRequestFullscreen: () => void } } = {
      webkitFullscreenEnabled: true,
      webkitFullscreenElement: null,
      documentElement: { webkitRequestFullscreen: vi.fn() },
      webkitExitFullscreen: vi.fn(),
      addEventListener: (type, l) => void listeners.set(type, l),
    };
    const host = documentFullscreen(doc);
    expect(host.available).toBe(true);
    await host.enter();
    expect(doc.documentElement.webkitRequestFullscreen).toHaveBeenCalledOnce();
    doc.webkitFullscreenElement = {} as Element;
    expect(host.active()).toBe(true);
    await host.exit();
    expect(doc.webkitExitFullscreen).toHaveBeenCalledOnce();
    const changed = vi.fn();
    host.onChange(changed);
    listeners.get("webkitfullscreenchange")!();
    expect(changed).toHaveBeenCalledOnce();
  });

  it("is unavailable, and does nothing, on a page without the API (Safari on iPhone)", async () => {
    const doc: FullscreenDocument = { documentElement: {}, addEventListener: () => {} };
    const host = documentFullscreen(doc);
    expect(host.available).toBe(false);
    expect(host.active()).toBe(false);
    await expect(host.enter()).resolves.toBeUndefined();
    await expect(host.exit()).resolves.toBeUndefined();
  });
});
