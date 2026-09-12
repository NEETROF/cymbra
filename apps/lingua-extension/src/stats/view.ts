import type { LinguaPort } from "../analyzer/port.ts";
import type { CefrLevel, LevelRow } from "../analyzer/types.ts";
import { loadDailyStats, utcDay } from "../state/dailystats.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import { barChartSvg } from "./chart.ts";
import {
  buildSeries,
  consolidatedToMap,
  type CountsByDay,
  dayWindow,
  estimatedPosition,
  type Range,
  RANGES,
} from "./model.ts";

// The stats view, shared by the side panel (primary) and the standalone tab. It renders
// the CEFR progression ladder (from the hydrated engine) plus the daily counters
// (local when signed out, consolidated GetStats when signed in). Self-contained: it
// builds its own DOM into a host element, so both surfaces just hand it a container +
// a port. Excluded from coverage (DOM wiring; model + chart are unit-tested).

const METRICS = [
  { key: "exposures", label: "Mots rencontrés", color: "var(--cymbra-lingua-teal)" },
  { key: "wordsLearned", label: "Mots appris", color: "var(--cymbra-lingua-green)" },
  { key: "reviews", label: "Révisions", color: "var(--cymbra-lingua-amber)" },
] as const;

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

async function fetchCounts(area: AsyncStorageArea, range: Range): Promise<{ byDay: CountsByDay; scope: string }> {
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

function ladderHtml(rows: LevelRow[], declared: CefrLevel | null): string {
  const pos = estimatedPosition(rows);
  const head =
    `<div class="ladder-head"><span class="mlabel">Mon niveau d'anglais</span>` +
    (pos ? `<span class="ladder-pos">niveau estimé <b>${pos}</b></span>` : "") +
    `</div>`;
  const bars = rows
    .map((r) => {
      const known = r.confirmed + r.presumed;
      const conf = r.total ? (r.confirmed / r.total) * 100 : 0;
      const pres = r.total ? (r.presumed / r.total) * 100 : 0;
      const here = r.level === declared ? " ladder-row--here" : "";
      return (
        `<div class="ladder-row${here}"><span class="ladder-lvl">${r.level}</span>` +
        `<span class="ladder-track">` +
        `<span class="ladder-conf" style="width:${conf}%"></span>` +
        `<span class="ladder-pres" style="width:${pres}%"></span></span>` +
        `<span class="ladder-frac">${known} / ${r.total}</span></div>`
      );
    })
    .join("");
  return (
    `<div class="ladder">${head}${bars}` +
    `<div class="note ladder-legend">Confirmés (lus / appris), présumés (sous ton niveau), à apprendre.</div></div>`
  );
}

/** Render the whole stats view (ladder + daily cards) into `root`, driven by `port`. */
export async function mountStats(root: HTMLElement, port: LinguaPort, area: AsyncStorageArea): Promise<void> {
  let range: Range = 30;
  root.classList.add("stats");
  root.innerHTML =
    `<div class="ladder-slot"></div>` +
    `<div class="topline"><span class="scope">…</span>` +
    `<div class="ranges">` +
    RANGES.map((r) => `<button data-range="${r}"${r === range ? ' class="active"' : ""}>${r} j</button>`).join("") +
    `</div></div>` +
    `<div class="cards"></div>` +
    `<p class="note">Les sessions d'agent IA (plugin Claude Code) ne sont pas comptées ici.</p>`;

  const pick = <T extends HTMLElement>(sel: string): T => {
    const el = root.querySelector<T>(sel);
    if (!el) throw new Error(`missing ${sel}`);
    return el;
  };

  // The ladder does not depend on the range, so render it once.
  if (await port.hasLevels()) {
    const [rows, declared] = [await port.levelLadder(), await port.declaredLevel()];
    pick(".ladder-slot").innerHTML = ladderHtml(rows, declared);
  } else {
    pick(".ladder-slot").innerHTML =
      `<div class="note">Niveaux CEFR indisponibles pour cette langue (pack sans données CEFR).</div>`;
  }

  const renderCards = async (): Promise<void> => {
    const { byDay, scope } = await fetchCounts(area, range);
    const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
    const series = buildSeries(byDay, fromDay, toDay);
    pick(".scope").textContent = scope;
    const cards = pick(".cards");
    cards.replaceChildren();
    for (const m of METRICS) {
      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML =
        `<div class="metric"><span class="mlabel">${m.label}</span>` +
        `<b class="mtotal">${series.totals[m.key]}</b></div>` +
        `<div class="chart">${barChartSvg(series[m.key], m.color, m.label)}</div>`;
      cards.append(card);
    }
  };

  for (const btn of root.querySelectorAll<HTMLButtonElement>(".ranges button")) {
    btn.addEventListener("click", () => {
      range = Number(btn.dataset.range) as Range;
      for (const b of root.querySelectorAll(".ranges button")) b.classList.toggle("active", b === btn);
      void renderCards();
    });
  }

  await renderCards();
}
