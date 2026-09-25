import {
  type AsyncStorageArea,
  loadReaderDisplay,
  type ReaderDisplay,
  readerDisplayOf,
  type ReaderTheme,
  saveReaderDisplay,
  TEXT_SCALE_MAX,
  TEXT_SCALE_MIN,
  TEXT_SCALE_STEP,
} from "../state/storage.ts";

// How a book is shown in the reader — its text size and its page (add-lingua-reader D10). One
// builder, rendered in the two places the reader looks for it: the reader's own "Aa" panel, and
// the Livres block of Réglages. Buttons, not a slider: a tap is one step, which an e-ink screen
// redraws once. The choice is saved; the reader page follows the stored value.

export interface BookDisplayView {
  /** Show the stored choice (it may have changed in the other place). */
  refresh(): Promise<void>;
}

function el<K extends keyof HTMLElementTagNameMap>(
  doc: Document,
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = doc.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function button(doc: Document, className: string, text: string, label: string): HTMLButtonElement {
  const b = el(doc, "button", className, text);
  b.type = "button";
  b.setAttribute("aria-label", label);
  b.title = label;
  return b;
}

const THEMES: { theme: ReaderTheme; text: string }[] = [
  { theme: "paper", text: "Papier" },
  { theme: "dark", text: "Sombre" },
];

/** Render the text size and page choice into `container`. */
export function mountBookDisplay(container: HTMLElement, area: AsyncStorageArea): BookDisplayView {
  let display: ReaderDisplay | null = null;
  // The document the view is mounted in: the reader page's, a panel's or a drawer's shadow.
  const doc = container.ownerDocument;

  const smaller = button(doc, "set-step", "A−", "Réduire le texte");
  const larger = button(doc, "set-step", "A+", "Agrandir le texte");
  const size = el(doc, "output", "set-step-value");
  const sizeRow = el(doc, "div", "set-display-row");
  const stepper = el(doc, "div", "set-stepper");
  stepper.append(smaller, size, larger);
  sizeRow.append(el(doc, "span", undefined, "Taille du texte"), stepper);

  const pages = el(doc, "div", "set-segmented");
  pages.setAttribute("role", "group");
  pages.setAttribute("aria-label", "Page");
  const themeButtons = THEMES.map(({ theme, text }) => {
    const b = el(doc, "button", "set-segment", text);
    b.type = "button";
    b.addEventListener("click", () => void choose({ theme }));
    return { theme, b };
  });
  pages.append(...themeButtons.map(({ b }) => b));
  const pageRow = el(doc, "div", "set-display-row");
  pageRow.append(el(doc, "span", undefined, "Page"), pages);

  const box = el(doc, "div", "set-display");
  box.append(sizeRow, pageRow);
  container.append(box);

  function render(): void {
    if (!display) return;
    size.textContent = `${display.textScale} %`;
    smaller.disabled = display.textScale <= TEXT_SCALE_MIN;
    larger.disabled = display.textScale >= TEXT_SCALE_MAX;
    for (const { theme, b } of themeButtons) b.setAttribute("aria-pressed", String(display.theme === theme));
  }

  async function choose(change: Partial<ReaderDisplay>): Promise<void> {
    display = readerDisplayOf({ ...(display ?? (await loadReaderDisplay(area))), ...change });
    render();
    await saveReaderDisplay(area, display);
  }

  smaller.addEventListener("click", () => void choose({ textScale: (display?.textScale ?? 100) - TEXT_SCALE_STEP }));
  larger.addEventListener("click", () => void choose({ textScale: (display?.textScale ?? 100) + TEXT_SCALE_STEP }));

  async function refresh(): Promise<void> {
    display = await loadReaderDisplay(area);
    render();
  }

  void refresh();
  return { refresh };
}
