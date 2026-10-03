import {
  type AsyncStorageArea,
  COLOURS_KEY,
  type ColourPreference,
  colourPreferenceOf,
  DEFAULT_COLOUR_PREFERENCE,
  DEFAULT_READER_DISPLAY,
  loadColourPreference,
  loadReaderDisplay,
  READER_DISPLAY_KEY,
  type ReaderDisplay,
  readerDisplayOf,
} from "../state/storage.ts";

// How the extension's own surfaces look (add-lingua-colour-settings D8, D9): the word card, the
// drawer, the pill, the reader's toolbar and library, the popup, the side panel. Two of the
// reader's choices decide it, wherever they were made:
// - their colours: an e-ink preset turns every surface black on white, high contrast;
// - their display: the page theme (paper or dark) gives the other presets a light or a dark
//   surface, and the text size scales the surfaces with the book's text.
// A surface carries the result on its root — a shadow host or an extension page's <html> — as
// an attribute the token sheet themes from, and the scale every font size multiplies.

/** The surfaces' palettes, defined in tokens.css. */
export type SurfaceTheme = "dark" | "light" | "eink";

/** The attribute a surface's root carries; tokens.css themes `:host([…])` and `:root[…]`. */
export const SURFACE_THEME_ATTR = "data-cymbra-lingua-ui";

/** The custom property every surface font size multiplies (1 = the sizes as designed). */
export const UI_SCALE_VAR = "--cymbra-lingua-ui-scale";

/** The reader's preferences, where both choices live (and their change event reaches every context). */
const preferences: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

/** The palette for a choice: e-ink presets keep black on white; the others follow the page. */
export function surfaceTheme(colours: ColourPreference, display: ReaderDisplay): SurfaceTheme {
  if (colours.preset === "eink-mono" || colours.preset === "eink-colour") return "eink";
  return display.theme === "dark" ? "dark" : "light";
}

/** Paint one surface root with the reader's choice. */
export function applySurfaceLook(root: HTMLElement, colours: ColourPreference, display: ReaderDisplay): void {
  root.setAttribute(SURFACE_THEME_ATTR, surfaceTheme(colours, display));
  root.style.setProperty(UI_SCALE_VAR, String(display.textScale / 100));
}

/**
 * Paint `root` now, then again whenever the reader's colours or display change, from any
 * surface. Both preferences live in `chrome.storage.local`, whose change event reaches every
 * context; `area` reads them first.
 */
export function followSurfaceLook(root: HTMLElement, area: AsyncStorageArea = preferences): void {
  let colours = DEFAULT_COLOUR_PREFERENCE;
  let display = DEFAULT_READER_DISPLAY;
  const paint = (): void => applySurfaceLook(root, colours, display);
  paint();
  void Promise.all([loadColourPreference(area), loadReaderDisplay(area)]).then(
    ([storedColours, storedDisplay]) => {
      colours = storedColours;
      display = storedDisplay;
      paint();
    },
    () => {},
  );
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== "local") return;
    const changedColours = changes[COLOURS_KEY];
    const changedDisplay = changes[READER_DISPLAY_KEY];
    if (!changedColours && !changedDisplay) return;
    if (changedColours) colours = colourPreferenceOf(changedColours.newValue);
    if (changedDisplay) display = readerDisplayOf(changedDisplay.newValue);
    paint();
  });
}
