import type { CefrLevel, LevelRow, VocabularyEstimate } from "../analyzer/types.ts";
import { cumulativeTotals, estimatedPosition, roughCount } from "./model.ts";

// The CEFR progression ladder's markup, shared by every stats host through mountStats.
// Pure (rows in, HTML out), so it is unit-tested without a DOM or the engine.

/** A count in French notation (thin space between thousands). */
const fmt = (n: number): string => n.toLocaleString("fr-FR");

/**
 * The estimated vocabulary size: one rounded figure (an extrapolation, not a count), how
 * it was obtained, and how many words are confirmed. Empty when the pack has no
 * dictionary words to estimate over.
 */
export function vocabularyHtml(est: VocabularyEstimate): string {
  if (est.universe === 0) return "";
  const label = `<span class="mlabel">Vocabulaire estimé</span>`;
  if (est.estimated === 0) {
    return (
      `<div class="vocab"><div class="vocab-head">${label}</div>` +
      `<div class="note">Pas encore d'estimation&nbsp;: déclare ton niveau dans les réglages ou marque des mots que tu connais.</div></div>`
    );
  }
  const confirmed = est.confirmed ? ` (dont ${fmt(est.confirmed)} confirmés)` : "";
  return (
    `<div class="vocab"><div class="vocab-head">${label}<b class="vocab-n">≈&nbsp;${fmt(roughCount(est.estimated))} mots</b></div>` +
    `<div class="note">Extrapolé de ton niveau et de tes mots marqués, tranche de fréquence par tranche, ` +
    `sur les ${fmt(est.universe)} mots du dictionnaire${confirmed}.</div></div>`
  );
}

/**
 * One row per level: how many of that level's own words are known (confirmed +
 * presumed), and the running word count up to that level. A level only counts the
 * words it introduces, so the running total is the figure comparable with the usual
 * "about 1,500 words at A2" vocabulary-size estimates.
 */
export function ladderHtml(rows: LevelRow[], declared: CefrLevel | null): string {
  const pos = estimatedPosition(rows);
  const cumulative = cumulativeTotals(rows);
  const head =
    `<div class="ladder-head"><span class="mlabel">Mon niveau d'anglais</span>` +
    (pos ? `<span class="ladder-pos">niveau estimé <b>${pos}</b></span>` : "") +
    `</div>`;
  const cols =
    `<div class="ladder-row ladder-cols"><span class="ladder-lvl"></span>` +
    `<span class="ladder-spacer"></span><span class="ladder-frac">ce niveau</span>` +
    `<span class="ladder-cum">cumulé</span></div>`;
  const bars = rows
    .map((r, i) => {
      const known = r.confirmed + r.presumed;
      const conf = r.total ? (r.confirmed / r.total) * 100 : 0;
      const pres = r.total ? (r.presumed / r.total) * 100 : 0;
      const here = r.level === declared ? " ladder-row--here" : "";
      return (
        `<div class="ladder-row${here}"><span class="ladder-lvl">${r.level}</span>` +
        `<span class="ladder-track">` +
        `<span class="ladder-conf" style="width:${conf}%"></span>` +
        `<span class="ladder-pres" style="width:${pres}%"></span></span>` +
        `<span class="ladder-frac">${fmt(known)} / ${fmt(r.total)}</span>` +
        `<span class="ladder-cum">${fmt(cumulative[i])}</span></div>`
      );
    })
    .join("");
  return (
    `<div class="ladder">${head}${cols}${bars}` +
    `<div class="note ladder-legend">Confirmés (lus / appris), présumés (sous ton niveau), à apprendre.</div>` +
    // French punctuation keeps its spaces unbreakable (narrow and regular no-break
    // entities), so « cumulé » never wraps away from its guillemets in the narrow drawer.
    `<div class="note ladder-scope">Chaque niveau compte les mots qu'il introduit&#8239;; «&#8239;cumulé&#8239;» ajoute ceux des niveaux précédents. ` +
    `Ce sont les mots de base de chaque niveau&nbsp;: un lecteur de ce niveau en connaît en général bien davantage.</div></div>`
  );
}
