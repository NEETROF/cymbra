// Opening a tab from a surface that cannot (change: fix a dead link in the in-page drawer).
// `chrome.tabs` does not exist in a content script — it runs in the visited page's origin —
// so the drawer's links did nothing at all. The background opens the tab on its behalf; an
// extension page (popup, side panel) can send the same message rather than know which it is.

export interface OpenPageMessage {
  type: "openPage";
  /** An extension path ("account.html#data") or a browser page ("about:addons"). */
  url: string;
}

export function isOpenPageMessage(m: unknown): m is OpenPageMessage {
  const message = m as Partial<OpenPageMessage> | null;
  return message?.type === "openPage" && typeof message.url === "string";
}

export type OpenPage = (url: string) => void;

/** Ask the background to open it. Never throws: a dead link is not worth a broken panel. */
export const openPageViaBackground: OpenPage = (url) => {
  void chrome.runtime.sendMessage({ type: "openPage", url } satisfies OpenPageMessage).catch(() => {});
};
