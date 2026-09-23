import type { LinguaPort } from "../analyzer/port.ts";
import { type CefrLevel, CEFR_LEVELS, type SeedOrder } from "../analyzer/types.ts";
import { loadDailyStats, utcDay } from "../state/dailystats.ts";
import { type AsyncStorageArea, saveBackup } from "../state/storage.ts";
import { barChartElement } from "./chart.ts";
import { ladderView, vocabularyView } from "./ladder.ts";
import {
  buildSeries,
  consolidatedToMap,
  type CountsByDay,
  dayWindow,
  groupMarkedWords,
  MARKED_ORIGINS,
  type MarkedOrigin,
  type MarkedWord,
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
  const account = (await sendRuntime({ type: "account:state" })) as { state?: { signedIn?: boolean } } | null;
  if (account?.state?.signedIn) {
    const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
    const res = (await sendRuntime({ type: "stats:get", fromDay, toDay })) as {
      ok?: boolean;
      rows?: ConsolidatedRow[];
    } | null;
    if (res?.ok && res.rows) return { byDay: consolidatedToMap(res.rows), scope: "Tous tes appareils" };
  }
  return { byDay: await loadDailyStats(area), scope: "Cet appareil" };
}

/** Max cards a single "Renforcer un niveau" action may seed (matches the engine cap). */
const SEED_CAP = 50;

/** The "Mots marqués" sections: the reader's decisions open, the automatic ones folded. */
const MARKED_SECTIONS: { origin: MarkedOrigin; label: string; note: string | null; open: boolean }[] = [
  { origin: "decision", label: "Mes décisions", note: null, open: true },
  {
    origin: "reading",
    label: "Confirmés par la lecture",
    note: "Sous ton niveau et lus plusieurs jours différents : passés « connu » automatiquement.",
    open: false,
  },
  {
    origin: "review",
    label: "Validés en révision",
    note: "Marqués « je connais » pendant une révision.",
    open: false,
  },
];

function buildSeedControl(): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "seed";

  const label = document.createElement("div");
  label.className = "mlabel";
  label.textContent = "Renforcer un niveau";
  wrap.append(label);

  const note = document.createElement("div");
  note.className = "seed-note";
  note.textContent = "Ajoute des mots d'un niveau à ton deck de révision, sans attendre de les croiser en lisant.";
  wrap.append(note);

  const controls = document.createElement("div");
  controls.className = "seed-controls";
  wrap.append(controls);

  const levelSel = document.createElement("select");
  levelSel.className = "seed-sel";
  levelSel.id = "seed-level";
  levelSel.setAttribute("aria-label", "Niveau");
  for (const l of CEFR_LEVELS) {
    const opt = document.createElement("option");
    opt.value = l;
    opt.textContent = l;
    levelSel.append(opt);
  }
  controls.append(levelSel);

  const countInput = document.createElement("input");
  countInput.className = "seed-num";
  countInput.id = "seed-count";
  countInput.type = "number";
  countInput.min = "1";
  countInput.max = String(SEED_CAP);
  countInput.step = "1";
  countInput.value = "20";
  countInput.setAttribute("aria-label", "Nombre de mots");
  controls.append(countInput);

  const orderSel = document.createElement("select");
  orderSel.className = "seed-sel";
  orderSel.id = "seed-order";
  orderSel.setAttribute("aria-label", "Ordre");
  const commonOpt = document.createElement("option");
  commonOpt.value = "common";
  commonOpt.textContent = "courants d'abord";
  const rareOpt = document.createElement("option");
  rareOpt.value = "rare";
  rareOpt.textContent = "rares d'abord";
  orderSel.append(commonOpt, rareOpt);
  controls.append(orderSel);

  const btn = document.createElement("button");
  btn.className = "seed-btn";
  btn.id = "seed-go";
  btn.textContent = "Ajouter au deck";
  wrap.append(btn);

  const result = document.createElement("div");
  result.className = "note seed-result";
  result.id = "seed-result";
  result.hidden = true;
  wrap.append(result);

  return wrap;
}

/** Render the whole stats view (ladder + seed control + daily cards) into `root`. */
export async function mountStats(root: HTMLElement, port: LinguaPort, area: AsyncStorageArea): Promise<void> {
  let range: Range = 30;
  root.classList.add("stats");

  const vocabSlot = document.createElement("div");
  vocabSlot.className = "vocab-slot";
  const ladderSlot = document.createElement("div");
  ladderSlot.className = "ladder-slot";
  const seedSlot = document.createElement("div");
  seedSlot.className = "seed-slot";
  const markedSlot = document.createElement("div");
  markedSlot.className = "marked-slot";

  const topline = document.createElement("div");
  topline.className = "topline";
  const scopeSpan = document.createElement("span");
  scopeSpan.className = "scope";
  scopeSpan.textContent = "…";
  const ranges = document.createElement("div");
  ranges.className = "ranges";
  for (const r of RANGES) {
    const btn = document.createElement("button");
    btn.dataset.range = String(r);
    if (r === range) btn.className = "active";
    btn.textContent = `${r} j`;
    ranges.append(btn);
  }
  topline.append(scopeSpan, ranges);

  const cards = document.createElement("div");
  cards.className = "cards";

  const trailingNote = document.createElement("p");
  trailingNote.className = "note";
  trailingNote.textContent = "Les sessions d'agent IA (plugin Claude Code) ne sont pas comptées ici.";

  root.replaceChildren(vocabSlot, ladderSlot, seedSlot, markedSlot, topline, cards, trailingNote);

  const pick = <T extends HTMLElement>(sel: string): T => {
    const el = root.querySelector<T>(sel);
    if (!el) throw new Error(`missing ${sel}`);
    return el;
  };

  // The ladder and the estimate do not depend on the range; re-rendered after a seed.
  const renderLadder = async (): Promise<void> => {
    const hasLevels = await port.hasLevels();
    const vocab = vocabularyView(await port.vocabularyEstimate(), hasLevels);
    pick(".vocab-slot").replaceChildren(...(vocab ? [vocab] : []));
    if (hasLevels) {
      const [rows, declared] = [await port.levelLadder(), await port.declaredLevel()];
      pick(".ladder-slot").replaceChildren(ladderView(rows, declared));
    } else {
      const note = document.createElement("div");
      note.className = "note";
      note.textContent = "Niveaux CEFR indisponibles pour cette langue (pack sans données CEFR).";
      pick(".ladder-slot").replaceChildren(note);
    }
  };
  await renderLadder();

  // "Renforcer un niveau" — only meaningful with CEFR data. Rendered once (stable
  // listener); a seed persists, reports, and refreshes the ladder.
  if (await port.hasLevels()) {
    pick(".seed-slot").replaceChildren(buildSeedControl());
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

  // "Mots marqués" — the discoverable way to undo a "connu"/"ignoré" decision (the
  // in-page counterpart is Alt-clicking the word). Split by origin: the reader's own
  // decisions stay open and short; the automatic confirmations (reading, review), which
  // grow with use, sit folded behind a count and only build their rows when opened.
  // Re-rendered after an undo, keeping each section's open state; the ladder counts
  // change too. Built with DOM APIs so a lemma is never interpolated into HTML.
  const markedOpen = Object.fromEntries(MARKED_SECTIONS.map((s) => [s.origin, s.open])) as Record<
    MarkedOrigin,
    boolean
  >;

  const markedRow = (w: MarkedWord, showBadge: boolean): HTMLLIElement => {
    const li = document.createElement("li");
    li.className = "marked-row";
    const word = document.createElement("span");
    word.className = "marked-word";
    word.textContent = w.lemma;
    li.append(word);
    if (showBadge) {
      const badge = document.createElement("span");
      badge.className = `marked-badge marked-badge--${w.status}`;
      badge.textContent = w.status === "known" ? "connu" : "ignoré";
      li.append(badge);
    }
    const undo = document.createElement("button");
    undo.className = "marked-undo";
    undo.textContent = "Remettre à apprendre";
    undo.addEventListener("click", async () => {
      undo.disabled = true;
      await port.setStatusAt(w.lemma, null, Date.now());
      await saveBackup(area, await port.backup());
      await renderMarked();
      await renderLadder();
    });
    li.append(undo);
    return li;
  };

  const markedSection = (section: (typeof MARKED_SECTIONS)[number], words: MarkedWord[]): HTMLDetailsElement => {
    const details = document.createElement("details");
    details.className = "marked-group";
    const summary = document.createElement("summary");
    summary.className = "marked-summary";
    const label = document.createElement("span");
    label.className = "marked-summary-label";
    label.textContent = section.label;
    const count = document.createElement("span");
    count.className = "marked-count";
    count.textContent = String(words.length);
    summary.append(label, count);
    details.append(summary);
    if (section.note) {
      const note = document.createElement("div");
      note.className = "marked-group-note";
      note.textContent = section.note;
      details.append(note);
    }
    const list = document.createElement("ul");
    list.className = "marked-list";
    details.append(list);
    const fill = (): void => {
      if (list.childElementCount > 0) return;
      for (const w of words) list.append(markedRow(w, section.origin === "decision"));
    };
    details.open = markedOpen[section.origin];
    if (details.open) fill();
    details.addEventListener("toggle", () => {
      markedOpen[section.origin] = details.open;
      if (details.open) fill();
    });
    return details;
  };

  const renderMarked = async (): Promise<void> => {
    const groups = groupMarkedWords(await port.exportStatusOps());
    const slot = pick(".marked-slot");
    slot.replaceChildren();
    const wrap = document.createElement("div");
    wrap.className = "marked";
    const title = document.createElement("div");
    title.className = "mlabel";
    title.textContent = "Mots marqués";
    wrap.append(title);
    if (MARKED_ORIGINS.every((origin) => groups[origin].length === 0)) {
      const empty = document.createElement("div");
      empty.className = "note";
      empty.textContent = "Aucun mot marqué « connu » ou « ignoré » pour l'instant.";
      wrap.append(empty);
      slot.append(wrap);
      return;
    }
    const note = document.createElement("div");
    note.className = "seed-note";
    note.textContent =
      "Marqués « connu » ou « ignoré » (donc plus surlignés). Remets-en un « à apprendre » pour qu'il soit de nouveau signalé. " +
      "En lecture : Alt/Option-clic (ou appui long sur tactile) sur un mot pour le rouvrir.";
    wrap.append(note);
    for (const section of MARKED_SECTIONS) {
      const words = groups[section.origin];
      if (words.length > 0) wrap.append(markedSection(section, words));
    }
    slot.append(wrap);
  };
  await renderMarked();

  const renderCards = async (): Promise<void> => {
    const { byDay, scope } = await fetchCounts(area, range);
    const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
    const series = buildSeries(byDay, fromDay, toDay);
    pick(".scope").textContent = scope;
    const cardsEl = pick(".cards");
    cardsEl.replaceChildren();
    for (const m of METRICS) {
      const card = document.createElement("div");
      card.className = "card";

      const metric = document.createElement("div");
      metric.className = "metric";
      const label = document.createElement("span");
      label.className = "mlabel";
      label.textContent = m.label;
      const total = document.createElement("b");
      total.className = "mtotal";
      total.textContent = String(series.totals[m.key]);
      metric.append(label, total);
      card.append(metric);

      const chart = document.createElement("div");
      chart.className = "chart";
      chart.append(barChartElement(series[m.key], m.color, m.label));
      card.append(chart);

      cardsEl.append(card);
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
