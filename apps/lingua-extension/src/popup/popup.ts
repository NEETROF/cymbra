import { type AsyncStorageArea, defaultState, loadState, saveState } from "../state/storage.ts";
import { deck, knownCount } from "../state/status.ts";

// Icon-popup controller (a surface the extension owns). Reads the local state for the
// deck/known/calibration counts, asks the active tab's content script for the page
// stats, and writes calibration / reset back to storage — the content scripts repaint
// via storage.onChanged. Excluded from coverage (DOM wiring; exercised manually).

interface PageStats {
  analysable: boolean;
  percent: number | null;
  counted: number;
  unknownOccurrences: number;
  distinctUnknown: number;
}

const area: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function activeTabId(): Promise<number | undefined> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab?.id;
}

async function pageStats(): Promise<PageStats | null> {
  const tabId = await activeTabId();
  if (tabId == null) return null;
  try {
    return (await chrome.tabs.sendMessage(tabId, { type: "getStats" })) as PageStats;
  } catch {
    // No content script on this tab (system page, or opened before install).
    return null;
  }
}

function renderPageStats(stats: PageStats | null): void {
  const analysed = $("analysed");
  const setup = $("setup");
  const ok = stats?.analysable === true;
  analysed.hidden = !ok;
  setup.hidden = ok;
  if (!ok || !stats) return;
  const pct = stats.percent ?? 0;
  $("pct").textContent = stats.percent == null ? "—" : `${pct}%`;
  ($("bar") as HTMLElement).style.width = `${pct}%`;
  $("counted").textContent = String(stats.counted);
  $("unknown").textContent = String(stats.unknownOccurrences);
  $("distinct").textContent = String(stats.distinctUnknown);
}

/** activeTab: inject the reader into the current page once, on the user's click. */
async function analyseCurrentPage(): Promise<void> {
  const tabId = await activeTabId();
  if (tabId == null) return;
  await chrome.scripting.executeScript({ target: { tabId }, files: ["content.js"] });
  // Give the content script a moment to analyse, then refresh the popup stats.
  setTimeout(() => void pageStats().then(renderPageStats), 500);
}

/** "Always highlight": request the <all_urls> grant exactly once (D3). */
async function requestAlways(): Promise<void> {
  const granted = await chrome.permissions.request({ origins: ["<all_urls>"] });
  if (granted) window.close();
}

async function main(): Promise<void> {
  const state = await loadState(area);
  $("known").textContent = String(knownCount(state));
  $("deck").textContent = String(deck(state).length);

  const calib = $("calib") as HTMLInputElement;
  const calibv = $("calibv");
  calib.value = String(state.calibration);
  calibv.textContent = String(state.calibration);
  calib.addEventListener("input", () => {
    calibv.textContent = calib.value;
  });
  calib.addEventListener("change", async () => {
    const current = await loadState(area);
    await saveState(area, { ...current, calibration: Number(calib.value) });
  });

  $("reset").addEventListener("click", async () => {
    await saveState(area, defaultState());
    window.location.reload();
  });

  $("analyse").addEventListener("click", () => void analyseCurrentPage());
  $("always").addEventListener("click", () => void requestAlways());

  renderPageStats(await pageStats());
}

void main();
