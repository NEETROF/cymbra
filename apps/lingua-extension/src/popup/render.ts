import { chooseLevelPrompt, levelTitle, noTextDetected } from "../analyzer/language-labels.ts";
import { DEFAULT_LANGUAGE } from "../analyzer/pairs.ts";
import type { CefrLevel, StudiedLanguage } from "../analyzer/types.ts";
import type { popup as frPopup } from "../i18n/fr/popup.ts";
import { formatCount, formatPercent, type InterfaceLanguage, NODE_SLOT, renderAround } from "../i18n/index.ts";

// The popup's main panel, rendered from a tab's stats — apart from popup.ts, the entry that reads
// the tab and wires the page, so a test mounts popup.html and renders it in every interface
// language (add-lingua-native-language-labels): the sentences that name the page's language are
// the labels module's in the language the entry read with its preferences, never a default.

/** The popup's copy: the catalogue's `popup` module in the interface language. */
export type PopupCopy = typeof frPopup;

export interface PageStats {
  /** "book" when the tab is the extension's reader, showing a section of a book. */
  surface?: "page" | "book";
  analysable: boolean;
  percent: number | null;
  counted: number;
  unknownOccurrences: number;
  distinctUnknown: number;
  calibration: number;
  declaredLevel: CefrLevel | null;
  hasLevels: boolean;
  /** Levels estimated from word frequency (add-lingua-spanish-levels); absent from an older build. */
  levelsEstimated?: boolean;
  /** No level decision yet (« Débutant » is a decision): show the call to action. */
  needsLevel: boolean;
  trackedCount: number;
  deckCount: number;
  dueCount: number;
  /** The page's language and the reader's accepted ones; absent from a content script of an older build. */
  language?: StudiedLanguage;
  languages?: StudiedLanguage[];
}

/**
 * Write `stats` into the page's main panel: the setup when the tab has no content script, the
 * figures otherwise. `onReader` says the tab is the extension's reader, which is never analysed
 * from here (add-lingua-reader D8). `language` is the interface language the page was filled in,
 * `copy` its module.
 */
export function renderStats(
  doc: Document,
  language: InterfaceLanguage,
  copy: PopupCopy,
  stats: PageStats | null,
  onReader: boolean,
): void {
  const $ = (id: string): HTMLElement => {
    const el = doc.getElementById(id);
    if (!el) throw new Error(`missing #${id}`);
    return el;
  };
  const present = stats !== null;
  // A reader tab is never analysed from here: the reader page reads its books itself.
  $("setup").hidden = present || onReader;
  $("controls").hidden = !present;
  if (!stats) return;

  const book = stats.surface === "book" || onReader;
  $("pct-label").textContent = book ? copy.knownInChapter : copy.knownOnPage;
  $("analysed").hidden = !stats.analysable;
  $("note").hidden = stats.analysable;
  // Named from the language the page is read in (add-lingua-language-choice D1).
  const studied = stats.language ?? DEFAULT_LANGUAGE;
  $("note").textContent = book ? copy.openBookForFigures : noTextDetected(language, stats.languages ?? [studied]);
  if (stats.analysable) {
    const pct = stats.percent ?? 0;
    $("pct").textContent = stats.percent == null ? copy.noPercent : formatPercent(language, pct, "tight");
    $("bar").style.width = `${pct}%`;
    $("counted").textContent = formatCount(language, stats.counted);
    $("unknown").textContent = formatCount(language, stats.unknownOccurrences);
    $("distinct").textContent = formatCount(language, stats.distinctUnknown);
  }

  $("tracked").textContent = formatCount(language, stats.trackedCount);
  $("deck").textContent = formatCount(language, stats.deckCount);
  $("review").textContent = copy.review(formatCount(language, stats.dueCount));

  // With CEFR data, the reader declares a level in Réglages; the main panel gets a compact
  // reminder, or a call-to-action until a level has been chosen (asked at first use).
  // « Débutant » is a decision (no level, but chosen): only a missing decision asks again.
  $("level-cta").hidden = !stats.hasLevels || !stats.needsLevel;
  $("level-indicator").hidden = !stats.hasLevels || stats.needsLevel;
  $("level-cta").textContent = chooseLevelPrompt(language, studied);
  // « Niveau de … : B1 »: the line's message rendered around the bold level (D1).
  const level = $("level-current");
  level.textContent = stats.declaredLevel ?? copy.beginner;
  renderAround(
    $("level-line"),
    copy.levelLine(levelTitle(language, studied, stats.levelsEstimated ?? false), NODE_SLOT),
    level,
  );
}
