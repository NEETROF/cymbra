import type { ReaderCopy } from "./copy.ts";
import type { LookupRecord } from "./library.ts";

// The recap of the words looked up (add-lingua-chapter-recap D5): a sheet over the reader page, one
// row per word — the form met, its gloss, its sentence with the word marked — and three actions.
// Built whole, then shown: on e-ink the sheet paints once, and a row sorted leaves without an
// animation. Text nodes only: a sentence comes from the book, never parsed as markup.

/** A word to sort, with the gloss the pack gives it now. */
export interface RecapRow {
  lookup: LookupRecord;
  gloss: string | null;
}

/** The rows met in one chapter. */
export interface RecapGroup {
  chapter: string;
  rows: RecapRow[];
}

export interface RecapActions {
  /** « + Deck »: the card created as from the word card, with the row's sentence and chapter. */
  add(row: RecapRow): Promise<void>;
  /** « Je connais »: the word's status set to known. */
  know(row: RecapRow): Promise<void>;
  /** Close the sheet: « Plus tard » at the end of a chapter, « Fermer » for the whole book. */
  close(): void;
}

export interface RecapView {
  /** The sheet's title: the chapter just left, or the whole book's list. */
  title: string;
  intro: string;
  groups: RecapGroup[];
  /** Whether each group is headed by its chapter: the whole book's list, when it spans several. */
  headed: boolean;
  /** The closing action's label. */
  closeLabel: string;
}

/** Rows in the order they were met, gathered under their chapter in the order the chapters were. */
export function groupByChapter(rows: readonly RecapRow[]): RecapGroup[] {
  const groups = new Map<string, RecapRow[]>();
  for (const row of rows) {
    const group = groups.get(row.lookup.chapter);
    if (group) group.push(row);
    else groups.set(row.lookup.chapter, [row]);
  }
  return [...groups].map(([chapter, grouped]) => ({ chapter, rows: grouped }));
}

/**
 * Where the word met sits in its sentence: the text before it, the word, the text after — the first
 * place it is written, whatever its case. Null when the sentence does not hold it as written.
 */
export function markedSentence(sentence: string, form: string): [string, string, string] | null {
  const at = form ? sentence.toLocaleLowerCase().indexOf(form.toLocaleLowerCase()) : -1;
  if (at < 0) return null;
  return [sentence.slice(0, at), sentence.slice(at, at + form.length), sentence.slice(at + form.length)];
}

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

function button(className: string, text: string, onClick: () => void): HTMLButtonElement {
  const b = el("button", className, text);
  b.type = "button";
  b.addEventListener("click", onClick);
  return b;
}

/** Draw the sheet into `container`, in one go; a row sorted or left is removed, the last closes it. */
export function renderRecap(container: HTMLElement, view: RecapView, copy: ReaderCopy, actions: RecapActions): void {
  const head = el("div", "recap-head");
  const title = el("h2", "recap-title", view.title);
  title.id = "reading-recap-title";
  head.append(
    title,
    button("reading-action recap-close", view.closeLabel, () => actions.close()),
  );
  const body = el("div", "recap-body");
  let left = 0;

  const leave = (item: HTMLElement): void => {
    const group = item.closest(".recap-group");
    item.remove();
    if (group && !group.querySelector(".recap-row")) group.remove();
    left--;
    if (left === 0) actions.close();
  };

  const row = (entry: RecapRow): HTMLLIElement => {
    const item = el("li", "recap-row");
    const word = el("div", "recap-word");
    word.append(el("b", "recap-form", entry.lookup.surface));
    if (entry.gloss) word.append(el("span", "recap-gloss", entry.gloss));
    const sentence = el("p", "recap-sentence");
    const marked = markedSentence(entry.lookup.sentence, entry.lookup.surface);
    if (marked) sentence.append(marked[0], el("mark", undefined, marked[1]), marked[2]);
    else sentence.textContent = entry.lookup.sentence;
    const acts = el("div", "recap-actions");
    // A row's buttons answer once: a second tap while the engine writes changes nothing.
    const act = (className: string, label: string, run: () => Promise<void>): HTMLButtonElement =>
      button(className, label, () => {
        if (item.dataset.busy) return;
        item.dataset.busy = "1";
        void run()
          .then(() => leave(item))
          .catch(() => delete item.dataset.busy);
      });
    acts.append(
      act("reading-action recap-add", copy.recapAdd, () => actions.add(entry)),
      act("reading-action recap-known", copy.recapKnown, () => actions.know(entry)),
      button("reading-action recap-leave", copy.recapLeave, () => leave(item)),
    );
    item.append(word, sentence, acts);
    return item;
  };

  for (const group of view.groups) {
    const section = el("section", "recap-group");
    if (view.headed) section.append(el("h3", "recap-chapter", group.chapter || copy.recapNoChapter));
    const list = el("ul", "recap-rows");
    list.append(...group.rows.map(row));
    left += group.rows.length;
    section.append(list);
    body.append(section);
  }
  container.setAttribute("role", "dialog");
  container.setAttribute("aria-labelledby", title.id);
  container.replaceChildren(head, el("p", "recap-intro", view.intro), body);
}
