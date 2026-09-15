import { afterEach, describe, expect, it, vi } from "vitest";
import { hasShortcutEditor, hasWebAuthFlow, isTouchPrimary } from "@/state/platform.ts";

describe("hasWebAuthFlow", () => {
  it("is true where the identity API can launch a web auth flow", () => {
    expect(hasWebAuthFlow({ launchWebAuthFlow: () => Promise.resolve("") })).toBe(true);
  });

  it("is false without the identity API (Safari)", () => {
    expect(hasWebAuthFlow(undefined)).toBe(false);
    expect(hasWebAuthFlow({})).toBe(false);
  });
});

/** Stub matchMedia so `(pointer: coarse)` matches only on a touch-primary device. */
function pointer(coarse: boolean): void {
  vi.stubGlobal("matchMedia", (query: string) => ({ matches: coarse && query === "(pointer: coarse)" }));
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("isTouchPrimary", () => {
  it("is true only when the primary pointer is coarse", () => {
    pointer(true);
    expect(isTouchPrimary()).toBe(true);
    pointer(false);
    expect(isTouchPrimary()).toBe(false);
  });

  it("is false where matchMedia does not exist", () => {
    vi.stubGlobal("matchMedia", undefined);
    expect(isTouchPrimary()).toBe(false);
  });
});

describe("hasShortcutEditor", () => {
  it("links the shortcut editor on desktop Chromium and Firefox", () => {
    pointer(false);
    expect(hasShortcutEditor("chromium")).toBe(true);
    expect(hasShortcutEditor("firefox")).toBe(true);
  });

  it("never links one on Safari, which has no shortcut editor", () => {
    pointer(false);
    expect(hasShortcutEditor("safari")).toBe(false);
  });

  it("does not link one on a touch-primary device (Firefox for Android)", () => {
    pointer(true);
    expect(hasShortcutEditor("firefox")).toBe(false);
  });
});
