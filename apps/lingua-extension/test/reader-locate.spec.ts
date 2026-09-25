import { describe, expect, it, type Mock, vi } from "vitest";
import { isReaderUrl, isReaderWhere, openOrFocusReader, type ReaderOpener, readerWhereReply } from "@/reader/locate.ts";

// Every entry point to the library lands in the same reader tab (add-lingua-reader D8, 4.8).

const URL = "chrome-extension://abc/reader.html";

type MockOpener = { [K in keyof ReaderOpener]: Mock<ReaderOpener[K]> };

function opener(answer: unknown): MockOpener {
  return {
    ask: vi.fn(async () => answer),
    focusTab: vi.fn(async () => undefined),
    focusWindow: vi.fn(async () => undefined),
    openTab: vi.fn(async () => undefined),
  };
}

describe("openOrFocusReader", () => {
  it("focuses the reader tab that answered, in its window, and opens nothing", async () => {
    const o = opener({ tabId: 7, windowId: 3 });
    expect(await openOrFocusReader(URL, o)).toBe("focused");
    expect(o.focusTab).toHaveBeenCalledWith(7);
    expect(o.focusWindow).toHaveBeenCalledWith(3);
    expect(o.openTab).not.toHaveBeenCalled();
  });

  it("opens a tab when no reader page answers", async () => {
    const o = opener(undefined);
    expect(await openOrFocusReader(URL, o)).toBe("opened");
    expect(o.openTab).toHaveBeenCalledWith(URL);
  });

  it("opens a tab when asking fails (nobody listening)", async () => {
    const o = opener(null);
    o.ask.mockRejectedValue(new Error("Receiving end does not exist."));
    expect(await openOrFocusReader(URL, o)).toBe("opened");
  });

  it("opens a tab when the answering tab closed meanwhile", async () => {
    const o = opener({ tabId: 7, windowId: 3 });
    o.focusTab.mockRejectedValue(new Error("No tab with id: 7."));
    expect(await openOrFocusReader(URL, o)).toBe("opened");
  });

  it("still counts a focused tab whose window cannot be raised", async () => {
    const o = opener({ tabId: 7, windowId: 3 });
    o.focusWindow.mockRejectedValue(new Error("unsupported"));
    expect(await openOrFocusReader(URL, o)).toBe("focused");
  });
});

describe("reader messages and addresses", () => {
  it("recognises the question and a well-formed answer only", () => {
    expect(isReaderWhere({ type: "reader:where" })).toBe(true);
    expect(isReaderWhere({ type: "openPage" })).toBe(false);
    expect(isReaderWhere(null)).toBe(false);
    expect(readerWhereReply({ tabId: 1, windowId: 2 })).toEqual({ tabId: 1, windowId: 2 });
    expect(readerWhereReply({ tabId: "1" })).toBeNull();
    expect(readerWhereReply(undefined)).toBeNull();
  });

  it("knows the reader page with or without a book in its address", () => {
    expect(isReaderUrl(URL, URL)).toBe(true);
    expect(isReaderUrl(`${URL}#book=ab`, URL)).toBe(true);
    expect(isReaderUrl(`${URL}?x`, URL)).toBe(true);
    expect(isReaderUrl("chrome-extension://abc/reader.htmlx", URL)).toBe(false);
    expect(isReaderUrl("https://example.com/", URL)).toBe(false);
    expect(isReaderUrl(undefined, URL)).toBe(false);
  });
});
