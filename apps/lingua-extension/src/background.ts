// Service worker: orchestration only (design D2). It paints the toolbar badge from
// the per-tab percentage the content script reports, routes the capture-selection
// keyboard command, and manages the activeTab-first permission posture (design D3):
// nothing runs until the user acts. "Always highlight" grants <all_urls>, which
// registers the reader on every page; without it, the popup injects on demand via
// activeTab. No analysis, no storage writes, no network.

// Badge colours are set through the chrome.action API (not CSS), so they cannot be a
// token var; they mirror the Cymbra palette: violet primaryContainer for an analysed
// page, faint slate for a page with nothing to count.
const BADGE_ACTIVE = "#7c3aed"; // --cymbra-lingua-accent-strong
const BADGE_NEUTRAL = "#6a7091"; // --cymbra-lingua-faint
const BADGE_TEXT = "#ffffff";

const READER_SCRIPT_ID = "lingua-reader";
const ALL_URLS = "<all_urls>";

interface StatsMessage {
  type: "stats";
  pct: number | null;
}

chrome.runtime.onMessage.addListener((message: unknown, sender) => {
  const msg = message as Partial<StatsMessage> | null;
  if (msg?.type !== "stats") return;
  const tabId = sender.tab?.id;
  if (tabId == null) return;
  const pct = msg.pct ?? null;
  void chrome.action.setBadgeText({ tabId, text: pct == null ? "—" : `${pct}%` });
  void chrome.action.setBadgeBackgroundColor({ tabId, color: pct == null ? BADGE_NEUTRAL : BADGE_ACTIVE });
  chrome.action.setBadgeTextColor?.({ tabId, color: BADGE_TEXT });
});

chrome.commands.onCommand.addListener((command) => {
  if (command !== "lingua-capture-selection") return;
  chrome.tabs.query({ active: true, currentWindow: true }, ([tab]) => {
    if (tab?.id != null) void chrome.tabs.sendMessage(tab.id, { type: "captureSelection" }).catch(() => {});
  });
});

/** Register the reader for every page when <all_urls> is granted; unregister otherwise. */
async function syncReaderRegistration(): Promise<void> {
  const granted = await chrome.permissions.contains({ origins: [ALL_URLS] });
  const registered = (await chrome.scripting.getRegisteredContentScripts({ ids: [READER_SCRIPT_ID] })).length > 0;
  if (granted && !registered) {
    await chrome.scripting.registerContentScripts([
      {
        id: READER_SCRIPT_ID,
        matches: [ALL_URLS],
        js: ["content.js"],
        runAt: "document_idle",
      },
    ]);
  } else if (!granted && registered) {
    await chrome.scripting.unregisterContentScripts({ ids: [READER_SCRIPT_ID] });
  }
}

chrome.runtime.onInstalled.addListener(() => void syncReaderRegistration());
chrome.runtime.onStartup.addListener(() => void syncReaderRegistration());

// When the user grants "always highlight", register the reader and light up the
// current tab immediately so the grant has a visible effect without a reload.
chrome.permissions.onAdded.addListener((perms) => {
  if (!perms.origins?.includes(ALL_URLS)) return;
  void syncReaderRegistration().then(async () => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab?.id != null)
      await chrome.scripting.executeScript({ target: { tabId: tab.id }, files: ["content.js"] }).catch(() => {});
  });
});
chrome.permissions.onRemoved.addListener((perms) => {
  if (perms.origins?.includes(ALL_URLS)) void syncReaderRegistration();
});
