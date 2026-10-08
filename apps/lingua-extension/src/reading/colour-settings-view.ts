import { colours as frColours } from "../i18n/fr/colours.ts";
import { fillSlots, slot } from "../i18n/index.ts";
import {
  type AsyncStorageArea,
  type ColourPreference,
  type ColourPresetId,
  type Colours,
  DEFAULT_COLOUR_PREFERENCE,
  FILL_INTENSITIES,
  type FillIntensity,
  loadColourPreference,
  saveColourPreference,
  type StatusColours,
  UNDERLINE_STYLES,
  UNDERLINE_THICKNESSES,
  type UnderlineStyle,
  type UnderlineThickness,
} from "../state/storage.ts";
import { coloursFor, indistinct, type MarkedStatus, statusStyle, type TokenReader, tokensOf } from "./colours.ts";
import type { ColoursCopy } from "./settings-copy.ts";

// The Couleurs block of Réglages (add-lingua-colour-settings D5): a preset, or every colour by
// hand — how unknown and learning words are marked, and the reader's paper and dark pages.
// One builder, mounted by the one settings builder, so it is the same in the popup, the side
// panel, the reader's drawer and the Safari app. Editing anything while a preset is on screen
// starts from that preset and stores the result as the reader's own set. The preference is
// saved; every read page and open book follows it through storage. Its copy is the catalogue's
// `colours` module, handed by the settings view in the interface language (localise-lingua-settings).

export interface ColourSettingsView {
  /** Show the stored choice (it may have changed in another surface). */
  refresh(): Promise<void>;
}

export interface ColourSettingsOptions {
  /** Where preset colours are read; default: the tokens the block itself inherits. */
  read?: TokenReader;
  /** The block's copy in the interface language; the French module when not given. */
  copy?: ColoursCopy;
}

/** Render the colour choice into `container`. */
export function mountColourSettings(
  container: HTMLElement,
  area: AsyncStorageArea,
  opts: ColourSettingsOptions = {},
): ColourSettingsView {
  const doc = container.ownerDocument;
  const copy = opts.copy ?? frColours;
  let preference: ColourPreference = DEFAULT_COLOUR_PREFERENCE;

  const presetLabels: { preset: ColourPresetId | "custom"; text: string }[] = [
    { preset: "cymbra", text: copy.presetCymbra },
    { preset: "eink-mono", text: copy.presetEinkMono },
    { preset: "eink-colour", text: copy.presetEinkColour },
    { preset: "custom", text: copy.presetCustom },
  ];
  const intensityLabels: Record<FillIntensity, string> = {
    none: copy.intensityNone,
    light: copy.intensityLight,
    strong: copy.intensityStrong,
  };
  const styleLabels: Record<UnderlineStyle, string> = {
    solid: copy.styleSolid,
    dotted: copy.styleDotted,
    dashed: copy.styleDashed,
    wavy: copy.styleWavy,
    double: copy.styleDouble,
    none: copy.styleNone,
  };
  const thicknessLabels: Record<UnderlineThickness, string> = { thin: copy.thicknessThin, thick: copy.thicknessThick };
  const statusLabels: Record<MarkedStatus, string> = { unknown: copy.unknownWords, learning: copy.learningWords };

  function el<K extends keyof HTMLElementTagNameMap>(
    tag: K,
    className?: string,
    text?: string,
  ): HTMLElementTagNameMap[K] {
    const node = doc.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function colourInput(label: string, onPick: (hex: string) => void): HTMLInputElement {
    const input = el("input", "set-colour-pick");
    input.type = "color";
    input.setAttribute("aria-label", label);
    input.title = label;
    // `change`, not `input`: one write when the picker closes, not one per step of a drag.
    input.addEventListener("change", () => onPick(input.value));
    return input;
  }

  function select<T extends string>(
    label: string,
    values: readonly T[],
    labels: Record<T, string>,
    onPick: (value: T) => void,
  ): HTMLSelectElement {
    const s = el("select", "set-colour-select");
    s.setAttribute("aria-label", label);
    for (const v of values) {
      const o = el("option", undefined, labels[v]);
      o.value = v;
      s.append(o);
    }
    s.addEventListener("change", () => onPick(s.value as T));
    return s;
  }

  function row(label: string, ...controls: HTMLElement[]): HTMLDivElement {
    const r = el("div", "set-colour-row");
    const controlsBox = el("div", "set-colour-controls");
    controlsBox.append(...controls);
    r.append(el("span", undefined, label), controlsBox);
    return r;
  }

  const read = (): TokenReader => opts.read ?? tokensOf(box);
  const current = (): Colours => coloursFor(preference, read());

  /** Change one thing, from what is on screen: a preset becomes the reader's own set. */
  async function edit(change: (colours: Colours) => void): Promise<void> {
    const colours = structuredClone(current());
    change(colours);
    await choose({ preset: "custom", colours });
  }

  async function choose(next: ColourPreference): Promise<void> {
    preference = next;
    render();
    await saveColourPreference(area, next);
  }

  // — Presets —
  const presets = el("div", "set-segmented set-colour-presets");
  presets.setAttribute("role", "group");
  presets.setAttribute("aria-label", copy.presets);
  const presetButtons = presetLabels.map(({ preset, text }) => {
    const b = el("button", "set-segment", text);
    b.type = "button";
    b.addEventListener("click", () => {
      if (preset === "custom") void choose({ preset: "custom", colours: current() });
      else void choose({ preset });
    });
    return { preset, b };
  });
  presets.append(...presetButtons.map(({ b }) => b));

  // — Preview: painted with the same values as the sheet, without the highlight registry —
  const preview = el("p", "set-colour-preview");
  const unknownWord = el("span", undefined, copy.previewUnknown);
  const learningWord = el("span", undefined, copy.previewLearning);
  // The two painted words are the message's parts, where the language puts them (D3).
  preview.append(...fillSlots(copy.preview(slot(0), slot(1)), [unknownWord, learningWord]));
  const warning = el("p", "set-note set-colour-warning", copy.indistinctWarning);
  warning.setAttribute("role", "status");

  // — One status, by hand —
  interface StatusControls {
    fill: HTMLInputElement;
    intensity: HTMLSelectElement;
    line: HTMLInputElement;
    style: HTMLSelectElement;
    thickness: HTMLSelectElement;
    textMode: HTMLSelectElement;
    text: HTMLInputElement;
  }
  const statusControls = {} as Record<MarkedStatus, StatusControls>;

  function statusFieldset(status: MarkedStatus): HTMLFieldSetElement {
    const name = statusLabels[status];
    const set = (change: (s: StatusColours) => void) => void edit((c) => change(c[status]));
    const controls: StatusControls = {
      fill: colourInput(copy.fillColour(name), (hex) => set((s) => void (s.fill.colour = hex))),
      intensity: select(copy.fill(name), FILL_INTENSITIES, intensityLabels, (v) =>
        set((s) => void (s.fill.intensity = v)),
      ),
      line: colourInput(copy.lineColour(name), (hex) => set((s) => void (s.underline.colour = hex))),
      style: select(copy.line(name), UNDERLINE_STYLES, styleLabels, (v) => set((s) => void (s.underline.style = v))),
      thickness: select(copy.thickness(name), UNDERLINE_THICKNESSES, thicknessLabels, (v) =>
        set((s) => void (s.underline.thickness = v)),
      ),
      textMode: select(
        copy.textColour(name),
        ["page", "chosen"] as const,
        { page: copy.textOfPage, chosen: copy.textChosen },
        (v) => set((s) => void (s.text = v === "page" ? null : controls.text.value)),
      ),
      text: colourInput(copy.chosenTextColour(name), (hex) => set((s) => void (s.text = hex))),
    };
    statusControls[status] = controls;
    const fieldset = el("fieldset", "set-colour-group");
    fieldset.append(
      el("legend", undefined, name),
      row(copy.background, controls.fill, controls.intensity),
      row(copy.underline, controls.line, controls.style, controls.thickness),
      row(copy.text, controls.textMode, controls.text),
    );
    return fieldset;
  }

  // — The reader's page —
  const paperBackground = colourInput(copy.paperBackground, (hex) => void edit((c) => void (c.paper.background = hex)));
  const paperText = colourInput(copy.paperChosenText, (hex) => void edit((c) => void (c.paper.text = hex)));
  const paperTextMode = select(
    copy.paperText,
    ["book", "chosen"] as const,
    { book: copy.textOfBook, chosen: copy.textChosen },
    (v) => void edit((c) => void (c.paper.text = v === "book" ? null : paperText.value)),
  );
  const darkBackground = colourInput(copy.darkBackground, (hex) => void edit((c) => void (c.dark.background = hex)));
  const darkText = colourInput(copy.darkText, (hex) => void edit((c) => void (c.dark.text = hex)));
  const pageFieldset = el("fieldset", "set-colour-group");
  pageFieldset.append(
    el("legend", undefined, copy.readerPage),
    row(copy.paperBackgroundRow, paperBackground),
    row(copy.paperTextRow, paperTextMode, paperText),
    row(copy.darkBackgroundRow, darkBackground),
    row(copy.darkTextRow, darkText),
  );

  const details = el("details", "set-colour-details");
  details.append(
    el("summary", undefined, copy.adjustEach),
    statusFieldset("unknown"),
    statusFieldset("learning"),
    pageFieldset,
  );

  const reset = el("button", "set-reset", copy.restoreDefaults);
  reset.type = "button";
  reset.addEventListener("click", () => void choose(DEFAULT_COLOUR_PREFERENCE));

  const box = el("div", "set-colours");
  box.append(presets, preview, warning, details, reset, el("div", "set-note", copy.scopeNote));
  container.append(box);

  function render(): void {
    const colours = current();
    for (const { preset, b } of presetButtons) b.setAttribute("aria-pressed", String(preference.preset === preset));
    Object.assign(unknownWord.style, statusStyle(colours.unknown));
    Object.assign(learningWord.style, statusStyle(colours.learning));
    preview.style.backgroundColor = colours.paper.background;
    preview.style.color = colours.paper.text ?? "";
    warning.hidden = !indistinct(colours);
    for (const status of ["unknown", "learning"] as const) {
      const c = statusControls[status];
      const s = colours[status];
      c.fill.value = s.fill.colour;
      c.intensity.value = s.fill.intensity;
      c.line.value = s.underline.colour;
      c.style.value = s.underline.style;
      c.thickness.value = s.underline.thickness;
      c.thickness.disabled = s.underline.style === "none";
      c.textMode.value = s.text === null ? "page" : "chosen";
      c.text.disabled = s.text === null;
      if (s.text !== null) c.text.value = s.text;
    }
    paperBackground.value = colours.paper.background;
    paperTextMode.value = colours.paper.text === null ? "book" : "chosen";
    paperText.disabled = colours.paper.text === null;
    if (colours.paper.text !== null) paperText.value = colours.paper.text;
    darkBackground.value = colours.dark.background;
    darkText.value = colours.dark.text;
  }

  async function refresh(): Promise<void> {
    preference = await loadColourPreference(area);
    render();
  }

  void refresh();
  return { refresh };
}
