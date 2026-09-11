// Icon-popup controller (a surface the extension owns). It holds no engine and no
// storage of its own: it asks the active tab's content script for stats and drives
// calibration / reset / review through messages, so the content script (which owns the
// engine and persists) stays the single writer. Excluded from coverage (DOM wiring).

interface PageStats {
  analysable: boolean;
  percent: number | null;
  counted: number;
  unknownOccurrences: number;
  distinctUnknown: number;
  calibration: number;
  trackedCount: number;
  deckCount: number;
  dueCount: number;
}

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function activeTabId(): Promise<number | undefined> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab?.id;
}

async function send(message: unknown): Promise<unknown> {
  const tabId = await activeTabId();
  if (tabId == null) return null;
  try {
    return await chrome.tabs.sendMessage(tabId, message);
  } catch {
    return null; // no content script on this tab
  }
}

function render(stats: PageStats | null): void {
  const present = stats !== null;
  $("setup").hidden = present;
  $("controls").hidden = !present;
  if (!stats) return;

  $("analysed").hidden = !stats.analysable;
  $("note").hidden = stats.analysable;
  if (stats.analysable) {
    const pct = stats.percent ?? 0;
    $("pct").textContent = stats.percent == null ? "—" : `${pct}%`;
    ($("bar") as HTMLElement).style.width = `${pct}%`;
    $("counted").textContent = String(stats.counted);
    $("unknown").textContent = String(stats.unknownOccurrences);
    $("distinct").textContent = String(stats.distinctUnknown);
  }

  $("tracked").textContent = String(stats.trackedCount);
  $("deck").textContent = String(stats.deckCount);
  $("due").textContent = String(stats.dueCount);
  ($("calib") as HTMLInputElement).value = String(stats.calibration);
  $("calibv").textContent = String(stats.calibration);
}

async function analyseCurrentPage(): Promise<void> {
  const tabId = await activeTabId();
  if (tabId == null) return;
  await chrome.scripting.executeScript({ target: { tabId }, files: ["content.js"] });
  setTimeout(() => void refresh(), 500);
}

async function refresh(): Promise<void> {
  render((await send({ type: "getStats" })) as PageStats | null);
}

async function main(): Promise<void> {
  const calib = $("calib") as HTMLInputElement;
  calib.addEventListener("input", () => {
    $("calibv").textContent = calib.value;
  });
  calib.addEventListener("change", () => void send({ type: "setCalibration", value: Number(calib.value) }));

  $("reset").addEventListener("click", async () => {
    await send({ type: "reset" });
    await refresh();
  });

  $("review").addEventListener("click", async () => {
    const tabId = await activeTabId();
    if (tabId != null) await chrome.sidePanel.open({ tabId });
    window.close();
  });

  $("analyse").addEventListener("click", () => void analyseCurrentPage());
  $("always").addEventListener("click", async () => {
    if (await chrome.permissions.request({ origins: ["<all_urls>"] })) window.close();
  });

  await refresh();
}

void main();
