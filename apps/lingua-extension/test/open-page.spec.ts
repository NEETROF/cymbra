import { afterEach, describe, expect, it, vi } from "vitest";
import { isOpenPageMessage, openPageViaBackground } from "@/state/open-page.ts";

afterEach(() => vi.unstubAllGlobals());

/** Stub `chrome.runtime.sendMessage` with an answer of our choosing. */
function stubRuntime(answer: () => Promise<unknown>) {
  const sendMessage = vi.fn(answer);
  vi.stubGlobal("chrome", { runtime: { sendMessage } });
  return sendMessage;
}

describe("isOpenPageMessage", () => {
  it("recognises an openPage message", () => {
    expect(isOpenPageMessage({ type: "openPage", url: "account.html#data" })).toBe(true);
    expect(isOpenPageMessage({ type: "openPage", url: "about:addons" })).toBe(true);
  });

  it("rejects anything else the background may receive", () => {
    // The background reads every runtime message: a guard that says yes to the wrong
    // shape opens a tab at `undefined`.
    expect(isOpenPageMessage({ type: "stats", pct: 40 })).toBe(false);
    expect(isOpenPageMessage({ type: "openPage" })).toBe(false); // no url
    expect(isOpenPageMessage({ type: "openPage", url: 42 })).toBe(false);
    expect(isOpenPageMessage(null)).toBe(false);
    expect(isOpenPageMessage(undefined)).toBe(false);
    expect(isOpenPageMessage("openPage")).toBe(false);
  });
});

describe("openPageViaBackground", () => {
  it("asks the background to open the page", () => {
    // A content script has no chrome.tabs at all — the message IS the whole mechanism.
    const sendMessage = stubRuntime(async () => undefined);
    openPageViaBackground("account.html#data");
    expect(sendMessage).toHaveBeenCalledWith({ type: "openPage", url: "account.html#data" });
  });

  it("swallows a rejected message rather than breaking the panel around it", async () => {
    const sendMessage = stubRuntime(async () => {
      throw new Error("Receiving end does not exist");
    });
    expect(() => openPageViaBackground("about:addons")).not.toThrow();
    for (let i = 0; i < 5; i++) await Promise.resolve(); // an unhandled rejection would fail the run
    expect(sendMessage).toHaveBeenCalledOnce();
  });
});
