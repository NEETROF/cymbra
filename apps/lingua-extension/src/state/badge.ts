import { isNamedLanguage, listedLanguageName } from "../analyzer/language-labels.ts";
import { badge as enBadge } from "../i18n/en/badge.ts";
import { badge as esBadge } from "../i18n/es/badge.ts";
import { badge as frBadge } from "../i18n/fr/badge.ts";
import { formatCount, type InterfaceLanguage, plural } from "../i18n/index.ts";
import type { LanguageCount } from "./review-outlook.ts";
import type { ColourPreference } from "./storage.ts";

// What the toolbar icon shows (add-lingua-comeback-hints D4–D6): the background's badge decisions,
// pure, so every state is tested without a browser. The GLOBAL badge — no tab — carries the count
// of cards a review would hold today, across the studied languages, with a title that says it in
// words; it shows on every tab that has nothing of its own. A tab whose page reports a percentage
// gets its own text; a tab with nothing to count, or whose reader is off, gets none (`null`) and so
// inherits the count. Safari before 18 keeps one badge for every tab and refuses `null`: there the
// count is not shown and the badge keeps its behaviour from before (D4).

// The badge's colours go through the chrome.action API, not CSS, so they cannot be token vars: they
// mirror the palette — the percentage violet, the count teal (white text 5.5:1), both black on the
// e-ink presets, where the « % » tells them apart (D5). Amber and coral mean "learning" and
// "unknown" on the page.
export const BADGE_PERCENT_COLOUR = "#7c3aed"; // --cymbra-lingua-accent-strong
export const BADGE_COUNT_COLOUR = "#0f766e"; // --cymbra-lingua-teal, the light sheet's
export const BADGE_EINK_COLOUR = "#000000"; // --cymbra-lingua-eink-black
export const BADGE_TEXT_COLOUR = "#ffffff";
/** The dash's slate, kept where the count is not shown (Safari before 18). */
export const BADGE_NEUTRAL_COLOUR = "#6a7091"; // --cymbra-lingua-faint

/** The first Safari whose badge text is kept per tab and takes `null` (D4). */
const SAFARI_PER_TAB_BADGE = 18;

/** The icon's copy by interface language. */
const BADGE_COPY = { fr: frBadge, en: enBadge, es: esBadge } satisfies Record<InterfaceLanguage, typeof frBadge>;

/** What the badge applies: its text (`null` inherits the global one), colour and, globally, title. */
export interface BadgeLook {
  text: string | null;
  colour: string;
}

export interface GlobalBadge extends BadgeLook {
  text: string;
  title: string;
}

/** A page's report, the reading session's `stats` message. */
export interface PageReport {
  pct: number | null;
  /** The reader is switched off on this tab. */
  disabled?: boolean;
}

/** The count as the badge writes it: nothing at 0, the number up to 99, « 99+ » above (D5). */
export function formatBadgeCount(count: number): string {
  if (count <= 0) return "";
  return count > 99 ? "99+" : String(count);
}

/** Whether the reader chose an e-ink colour preset: both figures are then drawn black (D5). */
export function einkColours(preference: ColourPreference): boolean {
  return preference.preset === "eink-mono" || preference.preset === "eink-colour";
}

/**
 * Whether this browser can show the count beside page percentages: everywhere but the Safari
 * build before Safari 18, read once from the user agent's `Version/` (D4). A Safari user agent
 * that names no version is taken for an old one.
 */
export function countSupported(userAgent: string, target: string = __TARGET__): boolean {
  if (target !== "safari") return true;
  const version = /\bVersion\/(\d+)/.exec(userAgent);
  return version !== null && Number(version[1]) >= SAFARI_PER_TAB_BADGE;
}

/**
 * The global badge for each studied language's count: the sum as text, in the count's colour, and
 * a title naming each language when several hold cards (D6) — the extension's name alone at 0, or
 * when the count is turned off (`shown` false).
 */
export function globalBadge(
  counts: readonly LanguageCount[],
  opts: { shown: boolean; eink: boolean; interfaceLanguage: InterfaceLanguage },
): GlobalBadge {
  const copy = BADGE_COPY[opts.interfaceLanguage];
  const colour = opts.eink ? BADGE_EINK_COLOUR : BADGE_COUNT_COLOUR;
  const holding = opts.shown ? counts.filter(({ count }) => count > 0) : [];
  const total = holding.reduce((sum, { count }) => sum + count, 0);
  if (total === 0) return { text: "", colour, title: copy.name };
  const title =
    holding.length === 1
      ? plural(opts.interfaceLanguage, total, copy.cards)
      : copy.perLanguage(
          holding
            .map(({ language, count }) =>
              copy.languageCount(
                isNamedLanguage(language) ? listedLanguageName(opts.interfaceLanguage, language) : language,
                formatCount(opts.interfaceLanguage, count),
              ),
            )
            .join(copy.listSeparator),
        );
  return { text: formatBadgeCount(total), colour, title };
}

/**
 * A tab's own badge from its page's report. A percentage is the tab's own, in violet (black on
 * e-ink). Nothing to count, or the reader off: `null`, so the tab shows the global count — in the
 * count's colour, set explicitly since Chrome has no `null` colour. Where the count is not
 * supported (`counting` false: Safari before 18), the badge as it was: the dash on slate, nothing
 * when off.
 */
export function tabBadge(report: PageReport, opts: { eink: boolean; counting: boolean }): BadgeLook {
  const pct = report.disabled ? null : report.pct;
  if (pct != null) return { text: `${pct}%`, colour: opts.eink ? BADGE_EINK_COLOUR : BADGE_PERCENT_COLOUR };
  if (!opts.counting) {
    return report.disabled ? { text: "", colour: BADGE_NEUTRAL_COLOUR } : { text: "—", colour: BADGE_NEUTRAL_COLOUR };
  }
  return { text: null, colour: opts.eink ? BADGE_EINK_COLOUR : BADGE_COUNT_COLOUR };
}
