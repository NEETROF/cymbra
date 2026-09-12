import type { LinguaPort } from "../analyzer/port.ts";
import { type CefrLevel, CEFR_LEVELS, type LevelRow, type SeedOrder } from "../analyzer/types.ts";
import { loadDailyStats, utcDay } from "../state/dailystats.ts";
import { type AsyncStorageArea, saveBackup } from "../state/storage.ts";
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

/** Max cards a single "Renforcer un niveau" action may seed (matches the engine cap). */
const SEED_CAP = 50;

function seedControlHtml(): string {
  const levels = CEFR_LEVELS.map((l) => `<option value="${l}">${l}</option>`).join("");
  return (
    `<div class="seed"><div class="mlabel">Renforcer un niveau</div>` +
    `<div class="seed-note">Ajoute des mots d'un niveau à ton deck de révision, sans attendre de les croiser en lisant.</div>` +
    `<div class="seed-controls">` +
    `<select class="seed-sel" id="seed-level" aria-label="Niveau">${levels}</select>` +
    `<input class="seed-num" id="seed-count" type="number" min="1" max="${SEED_CAP}" step="1" value="20" aria-label="Nombre de mots" />` +
    `<select class="seed-sel" id="seed-order" aria-label="Ordre">` +
    `<option value="common">courants d'abord</option><option value="rare">rares d'abord</option>` +
    `</select></div>` +
    `<button class="seed-btn" id="seed-go">Ajouter au deck</button>` +
    `<div class="note seed-result" id="seed-result" hidden></div></div>`
  );
}

/** Render the whole stats view (ladder + seed control + daily cards) into `root`. */
export async function mountStats(root: HTMLElement, port: LinguaPort, area: AsyncStorageArea): Promise<void> {
  let range: Range = 30;
  root.classList.add("stats");
  root.innerHTML =
    `<div class="ladder-slot"></div>` +
    `<div class="seed-slot"></div>` +
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

  // The ladder does not depend on the range; re-rendered after a seed.
  const renderLadder = async (): Promise<void> => {
    if (await port.hasLevels()) {
      const [rows, declared] = [await port.levelLadder(), await port.declaredLevel()];
      pick(".ladder-slot").innerHTML = ladderHtml(rows, declared);
    } else {
      pick(".ladder-slot").innerHTML =
        `<div class="note">Niveaux CEFR indisponibles pour cette langue (pack sans données CEFR).</div>`;
    }
  };
  await renderLadder();

  // "Renforcer un niveau" — only meaningful with CEFR data. Rendered once (stable
  // listener); a seed persists, reports, and refreshes the ladder.
  if (await port.hasLevels()) {
    pick(".seed-slot").innerHTML = seedControlHtml();
    const declared = await port.declaredLevel();
    if (declared) pick<HTMLSelectElement>("#seed-level").value = declared;
    pick<HTMLButtonElement>("#seed-go").addEventListener("click", async () => {
      const level = pick<HTMLSelectElement>("#seed-level").value as CefrLevel;
      const raw = Number(pick<HTMLInputElement>("#seed-count").value);
      const count = Math.max(1, Math.min(SEED_CAP, Number.isFinite(raw) ? Math.floor(raw) : 20));
      const order = pick<HTMLSelectElement>("#seed-order").value as SeedOrder;
      const added = await port.seedLevel(level, count, order, Math.floor(Date.now() / 1000));
      await saveBackup(area, await port.backup());
      const result = pick("#seed-result");
      result.hidden = false;
      result.textContent =
        added > 0
          ? `${added} carte${added > 1 ? "s" : ""} ajoutée${added > 1 ? "s" : ""} au deck (niveau ${level}).`
          : `Aucune carte ajoutée — ces mots sont déjà suivis ou dans ton deck.`;
      await renderLadder();
    });
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
