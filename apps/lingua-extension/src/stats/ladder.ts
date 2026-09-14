import type { CefrLevel, LevelRow } from "../analyzer/types.ts";
import { cumulativeTotals, estimatedPosition } from "./model.ts";

// The CEFR progression ladder's markup, shared by every stats host through mountStats.
// Pure (rows in, HTML out), so it is unit-tested without a DOM or the engine.

/** A count in French notation (thin space between thousands). */
const fmt = (n: number): string => n.toLocaleString("fr-FR");

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
    `<div class="note ladder-scope">Chaque niveau compte les mots qu'il introduit ; « cumulé » ajoute ceux des niveaux précédents. ` +
    `Ce sont les mots de base de chaque niveau : un lecteur de ce niveau en connaît en général bien davantage.</div></div>`
  );
}
