// Where the reader page is open, so every entry point lands in the SAME tab (add-lingua-reader
// D8): the background asks before opening a second one. The page answers from its own tab —
// no `tabs` permission is needed to find it, and none was added for the reader.

/** The reader page, as `openPage` names it. */
export const READER_PAGE = "reader.html";

export interface ReaderWhereMessage {
  type: "reader:where";
}

/** The reader page's answer: the tab it is in. */
export interface ReaderWhereReply {
  tabId: number;
  windowId: number;
}

export function isReaderWhere(m: unknown): m is ReaderWhereMessage {
  return (m as { type?: unknown } | null)?.type === "reader:where";
}

/** Whether an address is the reader page of this extension (a book in the address or not). */
export function isReaderUrl(url: string | undefined, pageUrl: string): boolean {
  if (!url) return false;
  return url === pageUrl || url.startsWith(`${pageUrl}#`) || url.startsWith(`${pageUrl}?`);
}

/** The reply as the background receives it, or null when no reader page answered. */
export function readerWhereReply(reply: unknown): ReaderWhereReply | null {
  const r = reply as Partial<ReaderWhereReply> | null | undefined;
  return typeof r?.tabId === "number" && typeof r.windowId === "number"
    ? { tabId: r.tabId, windowId: r.windowId }
    : null;
}

/** What opening the reader needs of the browser — the background's `chrome.tabs`/`windows`. */
export interface ReaderOpener {
  /** Ask the open reader pages where they are (`runtime.sendMessage` of a `reader:where`). */
  ask: () => Promise<unknown>;
  focusTab: (tabId: number) => Promise<unknown>;
  focusWindow: (windowId: number) => Promise<unknown>;
  openTab: (url: string) => Promise<unknown>;
}

/**
 * Open the reader page at `url`, or bring forward the one already open: every entry point —
 * the popup, the settings in the drawer or the side panel — lands in the same tab.
 */
export async function openOrFocusReader(url: string, opener: ReaderOpener): Promise<"focused" | "opened"> {
  const where = readerWhereReply(await opener.ask().catch(() => null));
  if (where) {
    try {
      await opener.focusTab(where.tabId);
      await opener.focusWindow(where.windowId).catch(() => {});
      return "focused";
    } catch {
      // The tab closed between the answer and the focus: open a new one.
    }
  }
  await opener.openTab(url);
  return "opened";
}
