import { loadDailyStats, utcDay } from "../state/dailystats.ts";
import { barChartSvg } from "./chart.ts";
import { buildSeries, consolidatedToMap, type CountsByDay, dayWindow, type Range } from "./model.ts";

// Stats screen controller (a surface the extension owns). Signed out it shows this
// device's local daily counters; signed in it asks the background (which holds the
// session + transport) for consolidated GetStats across the account's devices. Charts
// are dependency-free SVG. Excluded from coverage (DOM wiring; model + chart are tested).

const area = {
  get: (keys: string | string[] | null) => chrome.storage.local.get(keys),
  set: (items: Record<string, unknown>) => chrome.storage.local.set(items),
};

const METRICS = [
  { key: "exposures", label: "Mots rencontrés", color: "var(--cymbra-lingua-teal)" },
  { key: "wordsLearned", label: "Mots appris", color: "var(--cymbra-lingua-green)" },
  { key: "reviews", label: "Révisions", color: "var(--cymbra-lingua-amber)" },
] as const;

let range: Range = 30;

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function sendRuntime(message: unknown): Promise<unknown> {
  try {
    return await chrome.runtime.sendMessage(message);
  } catch {
    return null;
  }
}

interface ConsolidatedRow {
  day: number;
  exposures: number;
  wordsLearned: number;
  reviews: number;
}

async function fetchCounts(): Promise<{ byDay: CountsByDay; scope: string }> {
  const account = (await sendRuntime({ type: "account:state" })) as { signedIn?: boolean } | null;
  if (account?.signedIn) {
    const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
    const res = (await sendRuntime({ type: "stats:get", fromDay, toDay })) as {
      ok?: boolean;
      rows?: ConsolidatedRow[];
    } | null;
    if (res?.ok && res.rows) return { byDay: consolidatedToMap(res.rows), scope: "Tous tes appareils" };
  }
  return { byDay: await loadDailyStats(area), scope: "Cet appareil" };
}

function render(byDay: CountsByDay, scope: string): void {
  const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
  const series = buildSeries(byDay, fromDay, toDay);
  const values = { exposures: series.exposures, wordsLearned: series.wordsLearned, reviews: series.reviews };
  $("scope").textContent = scope;
  const cards = $("cards");
  cards.replaceChildren();
  for (const m of METRICS) {
    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML =
      `<div class="metric"><span class="mlabel">${m.label}</span>` +
      `<b class="mtotal">${series.totals[m.key]}</b></div>` +
      `<div class="chart">${barChartSvg(values[m.key], m.color, m.label)}</div>`;
    cards.append(card);
  }
}

async function refresh(): Promise<void> {
  const { byDay, scope } = await fetchCounts();
  render(byDay, scope);
}

function main(): void {
  for (const btn of document.querySelectorAll<HTMLButtonElement>("#ranges button")) {
    btn.addEventListener("click", () => {
      range = Number(btn.dataset.range) as Range;
      for (const b of document.querySelectorAll("#ranges button")) b.classList.toggle("active", b === btn);
      void refresh();
    });
  }
  void refresh();
}

main();
