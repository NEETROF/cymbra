import { estimatedLevelsNote, languageName } from "../analyzer/language-labels.ts";
import { acceptedLanguages } from "../analyzer/pairs.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import { type CefrLevel, CEFR_LEVELS, type SeedOrder, type StudiedLanguage } from "../analyzer/types.ts";
import { DEFAULT_INTERFACE_LANGUAGE, formatCount, type InterfaceLanguage, plural } from "../i18n/index.ts";
import { countsOf, loadDailyStats, utcDay } from "../state/dailystats.ts";
import { type AsyncStorageArea, saveBackup } from "../state/storage.ts";
import { barChartElement } from "./chart.ts";
import { ladderView, type StatsCopy, statsCopy, vocabularyView } from "./ladder.ts";
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
// a port. Excluded from coverage (DOM wiring; model + chart are unit-tested). Its words are the
// catalogue's `stats` module in the interface language its host hands it
// (localise-lingua-review-stats D1).

export { statsCopy } from "./ladder.ts";

/** The daily counters, each labelled by the copy's entry `label`. */
const METRICS = [
  { key: "exposures", label: "wordsRead", color: "var(--cymbra-lingua-teal)" },
  { key: "wordsLearned", label: "wordsLearned", color: "var(--cymbra-lingua-green)" },
  { key: "reviews", label: "reviews", color: "var(--cymbra-lingua-amber)" },
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

async function fetchCounts(
  area: AsyncStorageArea,
  range: Range,
  language: StudiedLanguage,
): Promise<{ byDay: CountsByDay; scope: "allDevices" | "thisDevice" }> {
  const account = (await sendRuntime({ type: "account:state" })) as { state?: { signedIn?: boolean } } | null;
  if (account?.state?.signedIn) {
    const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
    const res = (await sendRuntime({ type: "stats:get", fromDay, toDay, language })) as {
      ok?: boolean;
      rows?: ConsolidatedRow[];
    } | null;
    if (res?.ok && res.rows) return { byDay: consolidatedToMap(res.rows), scope: "allDevices" };
  }
  return { byDay: countsOf(await loadDailyStats(area), language), scope: "thisDevice" };
}

/** Max cards a single "Renforcer un niveau" action may seed (matches the engine cap). */
const SEED_CAP = 50;

/** The "Mots marqués" sections: the reader's decisions open, the automatic ones folded; each
 *  labelled, and noted, by the copy's entries it names. */
const MARKED_SECTIONS: {
  origin: MarkedOrigin;
  label: "myDecisions" | "confirmedByReading" | "validatedInReview";
  note: "confirmedByReadingNote" | "validatedInReviewNote" | null;
  open: boolean;
}[] = [
  { origin: "decision", label: "myDecisions", note: null, open: true },
  { origin: "reading", label: "confirmedByReading", note: "confirmedByReadingNote", open: false },
  { origin: "review", label: "validatedInReview", note: "validatedInReviewNote", open: false },
];

/** The control, and — for levels estimated from frequency — the note saying so. */
function buildSeedControl(estimatedNote: string | null, copy: StatsCopy): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "seed";

  const label = document.createElement("div");
  label.className = "mlabel";
  label.textContent = copy.seedTitle;
  wrap.append(label);

  const note = document.createElement("div");
  note.className = "seed-note";
  note.textContent = copy.seedNote;
  wrap.append(note);
  if (estimatedNote) {
    const why = document.createElement("div");
    why.className = "seed-note seed-estimate";
    why.textContent = estimatedNote;
    wrap.append(why);
  }

  const controls = document.createElement("div");
  controls.className = "seed-controls";
  wrap.append(controls);

  const levelSel = document.createElement("select");
  levelSel.className = "seed-sel";
  levelSel.id = "seed-level";
  levelSel.setAttribute("aria-label", copy.level);
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
  countInput.setAttribute("aria-label", copy.wordCount);
  controls.append(countInput);

  const orderSel = document.createElement("select");
  orderSel.className = "seed-sel";
  orderSel.id = "seed-order";
  orderSel.setAttribute("aria-label", copy.order);
  const commonOpt = document.createElement("option");
  commonOpt.value = "common";
  commonOpt.textContent = copy.commonFirst;
  const rareOpt = document.createElement("option");
  rareOpt.value = "rare";
  rareOpt.textContent = copy.rareFirst;
  orderSel.append(commonOpt, rareOpt);
  controls.append(orderSel);

  const btn = document.createElement("button");
  btn.className = "seed-btn";
  btn.id = "seed-go";
  btn.textContent = copy.addToDeck;
  wrap.append(btn);

  const result = document.createElement("div");
  result.className = "note seed-result";
  result.id = "seed-result";
  result.hidden = true;
  wrap.append(result);

  return wrap;
}

/** The choice of language, one segment per accepted language (add-lingua-language-stats-review D4). */
function languagePicker(
  languages: StudiedLanguage[],
  current: StudiedLanguage,
  choose: (language: StudiedLanguage) => void,
  copy: StatsCopy,
): HTMLElement {
  const picker = document.createElement("div");
  picker.className = "ranges stats-languages";
  picker.setAttribute("role", "group");
  picker.setAttribute("aria-label", copy.language);
  for (const language of languages) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.dataset.language = language;
    btn.textContent = languageName(language);
    if (language === current) btn.className = "active";
    btn.addEventListener("click", () => {
      if (language !== current) choose(language);
    });
    picker.append(btn);
  }
  return picker;
}

/** The language each root shows: the hosts mount the view afresh after every change (a reading
 *  gesture, a sync pull), and that remount keeps the reader's choice. */
const shownLanguage = new WeakMap<HTMLElement, StudiedLanguage>();

/**
 * Render the whole stats view (ladder + seed control + daily cards) into `root`, in the interface
 * language its host read (French when not given — localise-lingua-review-stats D1).
 */
export async function mountStats(
  root: HTMLElement,
  port: LinguaPort,
  area: AsyncStorageArea,
  chosen?: StudiedLanguage,
  interfaceLanguage: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
): Promise<void> {
  const copy = statsCopy(interfaceLanguage);
  const ui = { interfaceLanguage };
  let range: Range = 30;
  root.classList.add("stats");
  // One of the reader's languages: the one chosen, else the one this root showed, else the first;
  // with several, a selector picks another and mounts the page afresh (add-lingua-language-stats-review D4).
  const languages = await acceptedLanguages(port);
  const wanted = chosen ?? shownLanguage.get(root);
  const language = wanted && languages.includes(wanted) ? wanted : languages[0];
  shownLanguage.set(root, language);
  const lang = port.for(language);
  const picker =
    languages.length > 1
      ? languagePicker(languages, language, (next) => void mountStats(root, port, area, next, interfaceLanguage), copy)
      : null;

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
  scopeSpan.textContent = copy.loading;
  const ranges = document.createElement("div");
  ranges.className = "ranges";
  for (const r of RANGES) {
    const btn = document.createElement("button");
    btn.dataset.range = String(r);
    if (r === range) btn.className = "active";
    btn.textContent = copy.days(formatCount(interfaceLanguage, r));
    ranges.append(btn);
  }
  topline.append(scopeSpan, ranges);

  const cards = document.createElement("div");
  cards.className = "cards";

  const trailingNote = document.createElement("p");
  trailingNote.className = "note";
  trailingNote.textContent = copy.agentNote;

  root.replaceChildren(
    ...(picker ? [picker] : []),
    vocabSlot,
    ladderSlot,
    seedSlot,
    markedSlot,
    topline,
    cards,
    trailingNote,
  );

  const pick = <T extends HTMLElement>(sel: string): T => {
    const el = root.querySelector<T>(sel);
    if (!el) throw new Error(`missing ${sel}`);
    return el;
  };

  // The ladder and the estimate do not depend on the range; re-rendered after a seed.
  const renderLadder = async (): Promise<void> => {
    const hasLevels = await lang.hasLevels();
    const vocab = vocabularyView(await lang.vocabularyEstimate(), hasLevels, ui);
    pick(".vocab-slot").replaceChildren(...(vocab ? [vocab] : []));
    if (hasLevels) {
      const [rows, declared, estimated] = [
        await lang.levelLadder(),
        await lang.declaredLevel(),
        await lang.levelsEstimated(),
      ];
      pick(".ladder-slot").replaceChildren(ladderView(rows, declared, language, estimated, ui));
    } else {
      const note = document.createElement("div");
      note.className = "note";
      note.textContent = copy.noLevels;
      pick(".ladder-slot").replaceChildren(note);
    }
  };
  await renderLadder();

  // "Renforcer un niveau" — only meaningful with CEFR data. Rendered once (stable
  // listener); a seed persists, reports, and refreshes the ladder.
  if (await lang.hasLevels()) {
    const estimated = await lang.levelsEstimated();
    pick(".seed-slot").replaceChildren(buildSeedControl(estimated ? estimatedLevelsNote(language) : null, copy));
    const declared = await lang.declaredLevel();
    if (declared) pick<HTMLSelectElement>("#seed-level").value = declared;
    pick<HTMLButtonElement>("#seed-go").addEventListener("click", async () => {
      const level = pick<HTMLSelectElement>("#seed-level").value as CefrLevel;
      const raw = Number(pick<HTMLInputElement>("#seed-count").value);
      const count = Math.max(1, Math.min(SEED_CAP, Number.isFinite(raw) ? Math.floor(raw) : 20));
      const order = pick<HTMLSelectElement>("#seed-order").value as SeedOrder;
      const added = await lang.seedLevel(level, count, order, Math.floor(Date.now() / 1000));
      await saveBackup(area, await port.backup());
      const result = pick("#seed-result");
      result.hidden = false;
      result.textContent = added > 0 ? plural(interfaceLanguage, added, copy.cardsAdded(level)) : copy.noCardsAdded;
      await renderLadder();
    });
  }

  // "Mots marqués" — the discoverable way to undo a "connu"/"ignoré" decision (the
  // in-page counterpart is Alt-clicking the word). Split by origin: the reader's own
  // decisions stay open and short; the automatic confirmations (reading, review), which
  // grow with use, sit folded behind a count and only build their rows when opened.
  // Re-rendered after an undo, keeping each section's open state; the ladder counts
  // change too. Built with DOM APIs so a lemma is never interpolated into HTML. The selected
  // language's words only (add-lingua-language-stats-review): the export holds every language's,
  // and a word is put back in the selected one.
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
      badge.textContent = w.status === "known" ? copy.known : copy.ignored;
      li.append(badge);
    }
    const undo = document.createElement("button");
    undo.className = "marked-undo";
    undo.textContent = copy.relearn;
    undo.addEventListener("click", async () => {
      undo.disabled = true;
      await lang.setStatusAt(w.lemma, null, Date.now());
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
    label.textContent = copy[section.label];
    const count = document.createElement("span");
    count.className = "marked-count";
    count.textContent = formatCount(interfaceLanguage, words.length);
    summary.append(label, count);
    details.append(summary);
    if (section.note) {
      const note = document.createElement("div");
      note.className = "marked-group-note";
      note.textContent = copy[section.note];
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
    const groups = groupMarkedWords(await port.exportStatusOps(), language);
    const slot = pick(".marked-slot");
    slot.replaceChildren();
    const wrap = document.createElement("div");
    wrap.className = "marked";
    const title = document.createElement("div");
    title.className = "mlabel";
    title.textContent = copy.markedTitle;
    wrap.append(title);
    if (MARKED_ORIGINS.every((origin) => groups[origin].length === 0)) {
      const empty = document.createElement("div");
      empty.className = "note";
      empty.textContent = copy.noMarked;
      wrap.append(empty);
      slot.append(wrap);
      return;
    }
    const note = document.createElement("div");
    note.className = "seed-note";
    note.textContent = copy.markedNote;
    wrap.append(note);
    for (const section of MARKED_SECTIONS) {
      const words = groups[section.origin];
      if (words.length > 0) wrap.append(markedSection(section, words));
    }
    slot.append(wrap);
  };
  await renderMarked();

  const renderCards = async (): Promise<void> => {
    const { byDay, scope } = await fetchCounts(area, range, language);
    const { fromDay, toDay } = dayWindow(utcDay(Date.now()), range);
    const series = buildSeries(byDay, fromDay, toDay);
    pick(".scope").textContent = copy[scope];
    const cardsEl = pick(".cards");
    cardsEl.replaceChildren();
    for (const m of METRICS) {
      const card = document.createElement("div");
      card.className = "card";

      const metric = document.createElement("div");
      metric.className = "metric";
      const label = document.createElement("span");
      label.className = "mlabel";
      label.textContent = copy[m.label];
      const total = document.createElement("b");
      total.className = "mtotal";
      total.textContent = formatCount(interfaceLanguage, series.totals[m.key]);
      metric.append(label, total);
      card.append(metric);

      const chart = document.createElement("div");
      chart.className = "chart";
      chart.append(barChartElement(series[m.key], m.color, copy[m.label]));
      card.append(chart);

      cardsEl.append(card);
    }
  };

  // The range's own buttons: the language selector looks the same but is not a range.
  for (const btn of ranges.querySelectorAll<HTMLButtonElement>("button")) {
    btn.addEventListener("click", () => {
      range = Number(btn.dataset.range) as Range;
      for (const b of ranges.querySelectorAll("button")) b.classList.toggle("active", b === btn);
      void renderCards();
    });
  }

  await renderCards();
}
