import { borrowedTypicalNote, estimatedLevelsNote, myLevelTitle } from "../analyzer/language-labels.ts";
import { DEFAULT_LANGUAGE } from "../analyzer/pairs.ts";
import type { CefrLevel, LevelRow, StudiedLanguage, VocabularyEstimate } from "../analyzer/types.ts";
import { cumulativeTotals, estimatedPosition, roughCount } from "./model.ts";

// The CEFR progression ladder's markup, shared by every stats host through mountStats.
// Pure (rows in, DOM out), so it is unit-tested without the engine — jsdom (already the
// project's test environment) stands in for the real DOM.

/** A count in French notation (thin space between thousands). */
const fmt = (n: number): string => n.toLocaleString("fr-FR");

/**
 * The estimated vocabulary size, worded for what it rests on. Extrapolated from a
 * declared level or the frequency slider, it is a rounded figure that never reads below
 * the words confirmed; resting only on marked words, it is their exact count. `null` when
 * the pack has no dictionary words to estimate over.
 */
export function vocabularyView(est: VocabularyEstimate, hasLevels: boolean): HTMLElement | null {
  if (est.universe === 0) return null;
  const exact = est.basis === "marked";

  const wrap = document.createElement("div");
  wrap.className = "vocab";
  const head = document.createElement("div");
  head.className = "vocab-head";
  wrap.append(head);
  const label = document.createElement("span");
  label.className = "mlabel";
  label.textContent = exact ? "Vocabulaire connu" : "Vocabulaire estimé";
  head.append(label);

  if (est.estimated === 0) {
    const hint =
      est.basis === "level"
        ? "Ton niveau ne présume encore aucun mot\u00A0: marque ceux que tu connais pour lancer l'estimation."
        : `Pas encore d'estimation\u00A0: ${hasLevels ? "déclare ton niveau" : "règle les mots courants que tu connais"} dans les réglages, ou marque des mots que tu connais.`;
    const note = document.createElement("div");
    note.className = "note";
    note.textContent = hint;
    wrap.append(note);
    return wrap;
  }

  const dictionary = `sur les ${fmt(est.universe)} mots du dictionnaire`;
  const n = document.createElement("b");
  n.className = "vocab-n";
  head.append(n);
  const note = document.createElement("div");
  note.className = "note";
  wrap.append(note);

  if (exact) {
    n.textContent = `${fmt(est.estimated)} mots`;
    note.textContent = `Les mots que tu as marqués connus, ${dictionary}.`;
    return wrap;
  }

  const source = est.basis === "level" ? "ton niveau déclaré" : "ton réglage des mots les plus courants";
  const confirmed = est.confirmed ? ` (dont ${fmt(est.confirmed)} confirmés)` : "";
  const figure = Math.max(roughCount(est.estimated), est.confirmed);
  n.textContent = `≈\u00A0${fmt(figure)} mots`;
  note.textContent =
    `D'après ${source} et tes mots marqués, extrapolé tranche de fréquence par tranche, ` +
    `${dictionary}${confirmed}.`;
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
): HTMLElement {
  const pos = estimatedPosition(rows);
  const cumulative = cumulativeTotals(rows);

  const ladder = document.createElement("div");
  ladder.className = "ladder";

  const head = document.createElement("div");
  head.className = "ladder-head";
  const headLabel = document.createElement("span");
  headLabel.className = "mlabel";
  headLabel.textContent = myLevelTitle(language, estimated);
  head.append(headLabel);
  if (pos) {
    const posSpan = document.createElement("span");
    posSpan.className = "ladder-pos";
    posSpan.append("niveau estimé ");
    const b = document.createElement("b");
    b.textContent = pos;
    posSpan.append(b);
    head.append(posSpan);
  }
  ladder.append(head);
  if (estimated) {
    const why = document.createElement("div");
    why.className = "note ladder-estimate";
    why.textContent = estimatedLevelsNote(language);
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
  colsFrac.textContent = "ce niveau";
  const colsCum = document.createElement("span");
  colsCum.className = "ladder-cum";
  colsCum.textContent = estimated ? "courants" : "enseignés";
  const colsEst = document.createElement("span");
  colsEst.className = "ladder-est";
  colsEst.textContent = "estimés";
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
    frac.textContent = `${fmt(known)} / ${fmt(r.total)}`;
    row.append(frac);

    const cum = document.createElement("span");
    cum.className = "ladder-cum";
    cum.textContent = fmt(cumulative[i]);
    row.append(cum);

    const est = document.createElement("span");
    est.className = "ladder-est";
    est.textContent = r.typicalVocabulary ? `≈\u00A0${fmt(roughCount(r.typicalVocabulary))}` : "–";
    row.append(est);

    ladder.append(row);
  });

  const legend = document.createElement("div");
  legend.className = "note ladder-legend";
  legend.textContent = "Confirmés (lus / appris), présumés (sous ton niveau), à apprendre.";
  ladder.append(legend);

  // French punctuation keeps its spaces unbreakable (narrow no-break space, U+202F, and
  // regular no-break space, U+00A0), so « enseignés » never wraps away from its guillemets
  // in the narrow drawer.
  const scope = document.createElement("div");
  scope.className = "note ladder-scope";
  scope.textContent =
    (estimated
      ? "«\u202Fcourants\u202F»\u00A0: les mots les plus fréquents jusqu'à ce niveau. "
      : "«\u202Fenseignés\u202F»\u00A0: les mots de base introduits jusqu'à ce niveau par les listes d'enseignement. ") +
    "«\u202Festimés\u202F»\u00A0: le vocabulaire qu'a en général un lecteur de ce niveau, " +
    // A pack whose levels are estimated has no lists to extrapolate: it shows English's figures.
    (rows[0]?.typicalFrom
      ? borrowedTypicalNote(language, rows[0].typicalFrom)
      : "extrapolé des mots des niveaux inférieurs sur tout le dictionnaire.");
  ladder.append(scope);

  return ladder;
}
