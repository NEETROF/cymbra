import { borrowedTypicalNote, estimatedLevelsNote, myLevelTitle } from "../analyzer/language-labels.ts";
import { DEFAULT_LANGUAGE } from "../analyzer/pairs.ts";
import type { CefrLevel, LevelRow, StudiedLanguage, VocabularyEstimate } from "../analyzer/types.ts";
import { stats as enStats } from "../i18n/en/stats.ts";
import { stats as esStats } from "../i18n/es/stats.ts";
import { stats as frStats } from "../i18n/fr/stats.ts";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  formatNumber,
  type InterfaceLanguage,
  NODE_SLOT,
  plural,
  renderAround,
} from "../i18n/index.ts";
import { cumulativeTotals, estimatedPosition, roughCount } from "./model.ts";

// The CEFR progression ladder's markup, shared by every stats host through mountStats.
// Pure (rows in, DOM out), so it is unit-tested without the engine — jsdom (already the
// project's test environment) stands in for the real DOM. Its words are the catalogue's `stats`
// module in the interface language (localise-lingua-review-stats D1), and the languages it names are
// the labels module's in that language (add-lingua-native-language-labels D2).

/** The statistics' copy: the catalogue's `stats` module, in the interface language (its French the default). */
export type StatsCopy = typeof frStats;

/** The statistics' copy by interface language: the one place that holds all three. */
const STATS_COPY: Record<InterfaceLanguage, StatsCopy> = { fr: frStats, en: enStats, es: esStats };

/** The statistics' module for the interface language — the ladder's, the view's, and the standalone tab's page. */
export function statsCopy(language: InterfaceLanguage): StatsCopy {
  return STATS_COPY[language];
}

/** What the ladder's functions speak: the interface language, whose module and figures they take —
 *  the copy derived from it, so the two cannot disagree. */
export interface StatsCopyOptions {
  /** The interface language; French when not given. */
  interfaceLanguage?: InterfaceLanguage;
}

/**
 * The copy, the language, and a figure as the language writes it (D2): French through
 * `toLocaleString("fr-FR")` as before — « 12 345 », a narrow no-break space between the groups —
 * English "12,345", Spanish as the RAE writes it (`formatNumber`).
 */
function speaking(opts: StatsCopyOptions): {
  copy: StatsCopy;
  language: InterfaceLanguage;
  fmt: (n: number) => string;
} {
  const language = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  return { copy: STATS_COPY[language], language, fmt: (n) => formatNumber(language, n) };
}

/**
 * The estimated vocabulary size, worded for what it rests on. Extrapolated from a
 * declared level or the frequency slider, it is a rounded figure that never reads below
 * the words confirmed; resting only on marked words, it is their exact count. `null` when
 * the pack has no dictionary words to estimate over.
 */
export function vocabularyView(
  est: VocabularyEstimate,
  hasLevels: boolean,
  opts: StatsCopyOptions = {},
): HTMLElement | null {
  if (est.universe === 0) return null;
  const { copy, language, fmt } = speaking(opts);
  const exact = est.basis === "marked";

  const wrap = document.createElement("div");
  wrap.className = "vocab";
  const head = document.createElement("div");
  head.className = "vocab-head";
  wrap.append(head);
  const label = document.createElement("span");
  label.className = "mlabel";
  label.textContent = exact ? copy.vocabularyKnown : copy.vocabularyEstimated;
  head.append(label);

  if (est.estimated === 0) {
    const hint =
      est.basis === "level"
        ? copy.noEstimateFromLevel
        : copy.noEstimateYet(hasLevels ? copy.declareYourLevel : copy.setCommonWords);
    const note = document.createElement("div");
    note.className = "note";
    note.textContent = hint;
    wrap.append(note);
    return wrap;
  }

  const dictionary = copy.ofDictionary(fmt(est.universe));
  const n = document.createElement("b");
  n.className = "vocab-n";
  head.append(n);
  const note = document.createElement("div");
  note.className = "note";
  wrap.append(note);

  if (exact) {
    n.textContent = plural(language, est.estimated, copy.words, fmt(est.estimated));
    note.textContent = copy.markedKnownWords(dictionary);
    return wrap;
  }

  const source = est.basis === "level" ? copy.fromDeclaredLevel : copy.fromCommonWordsSetting;
  const confirmed = est.confirmed ? plural(language, est.confirmed, copy.confirmedCount, fmt(est.confirmed)) : "";
  const figure = Math.max(roughCount(est.estimated), est.confirmed);
  n.textContent = plural(language, figure, copy.approxWords, fmt(figure));
  note.textContent = copy.estimateNote(source, dictionary, confirmed);
  return wrap;
}

/**
 * One row per level: how many of that level's own words are known (confirmed +
 * presumed), the words taught up to that level (running total of the teaching lists),
 * and the vocabulary a reader at that level typically has — the figure comparable
 * with the usual "about 16,000 words at C2" estimates, which the teaching lists are not.
 *
 * Levels estimated from word frequency (`estimated`, add-lingua-spanish-levels) say so: the
 * title and a note name them estimated, and no teaching list stands behind the running
 * total, so its column reads « courants », the commonest words up to the level.
 */
export function ladderView(
  rows: LevelRow[],
  declared: CefrLevel | null,
  language: StudiedLanguage = DEFAULT_LANGUAGE,
  estimated = false,
  opts: StatsCopyOptions = {},
): HTMLElement {
  const { copy, language: interfaceLanguage, fmt } = speaking(opts);
  const pos = estimatedPosition(rows);
  const cumulative = cumulativeTotals(rows);

  const ladder = document.createElement("div");
  ladder.className = "ladder";

  const head = document.createElement("div");
  head.className = "ladder-head";
  const headLabel = document.createElement("span");
  headLabel.className = "mlabel";
  headLabel.textContent = myLevelTitle(interfaceLanguage, language, estimated);
  head.append(headLabel);
  if (pos) {
    const posSpan = document.createElement("span");
    posSpan.className = "ladder-pos";
    const b = document.createElement("b");
    b.textContent = pos;
    renderAround(posSpan, copy.estimatedLevel(NODE_SLOT), b);
    head.append(posSpan);
  }
  ladder.append(head);
  if (estimated) {
    const why = document.createElement("div");
    why.className = "note ladder-estimate";
    why.textContent = estimatedLevelsNote(interfaceLanguage, language);
    ladder.append(why);
  }

  const cols = document.createElement("div");
  cols.className = "ladder-row ladder-cols";
  const colsLvl = document.createElement("span");
  colsLvl.className = "ladder-lvl";
  const colsSpacer = document.createElement("span");
  colsSpacer.className = "ladder-spacer";
  const colsFrac = document.createElement("span");
  colsFrac.className = "ladder-frac";
  colsFrac.textContent = copy.thisLevel;
  const colsCum = document.createElement("span");
  colsCum.className = "ladder-cum";
  colsCum.textContent = estimated ? copy.common : copy.taught;
  const colsEst = document.createElement("span");
  colsEst.className = "ladder-est";
  colsEst.textContent = copy.typical;
  cols.append(colsLvl, colsSpacer, colsFrac, colsCum, colsEst);
  ladder.append(cols);

  rows.forEach((r, i) => {
    const known = r.confirmed + r.presumed;
    const conf = r.total ? (r.confirmed / r.total) * 100 : 0;
    const pres = r.total ? (r.presumed / r.total) * 100 : 0;

    const row = document.createElement("div");
    row.className = r.level === declared ? "ladder-row ladder-row--here" : "ladder-row";

    const lvl = document.createElement("span");
    lvl.className = "ladder-lvl";
    lvl.textContent = r.level;
    row.append(lvl);

    const track = document.createElement("span");
    track.className = "ladder-track";
    const confBar = document.createElement("span");
    confBar.className = "ladder-conf";
    confBar.style.width = `${conf}%`;
    const presBar = document.createElement("span");
    presBar.className = "ladder-pres";
    presBar.style.width = `${pres}%`;
    track.append(confBar, presBar);
    row.append(track);

    const frac = document.createElement("span");
    frac.className = "ladder-frac";
    frac.textContent = copy.fraction(fmt(known), fmt(r.total));
    row.append(frac);

    const cum = document.createElement("span");
    cum.className = "ladder-cum";
    cum.textContent = fmt(cumulative[i]);
    row.append(cum);

    const est = document.createElement("span");
    est.className = "ladder-est";
    est.textContent = r.typicalVocabulary ? copy.approx(fmt(roughCount(r.typicalVocabulary))) : copy.noFigure;
    row.append(est);

    ladder.append(row);
  });

  const legend = document.createElement("div");
  legend.className = "note ladder-legend";
  legend.textContent = copy.legend;
  ladder.append(legend);

  // French punctuation keeps its spaces unbreakable (narrow no-break space, U+202F, and
  // regular no-break space, U+00A0), so « enseignés » never wraps away from its guillemets
  // in the narrow drawer: the French module writes them as this module did (D3).
  const scope = document.createElement("div");
  scope.className = "note ladder-scope";
  scope.textContent =
    (estimated ? copy.scopeCommon : copy.scopeTaught) +
    copy.scopeTypical(
      // A pack whose levels are estimated has no lists to extrapolate: it shows English's figures.
      rows[0]?.typicalFrom ? borrowedTypicalNote(interfaceLanguage, language, rows[0].typicalFrom) : copy.extrapolated,
    );
  ladder.append(scope);

  return ladder;
}
