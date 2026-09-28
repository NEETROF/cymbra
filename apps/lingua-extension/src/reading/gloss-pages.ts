import type { GrammarTag, SenseGroup } from "../analyzer/types.ts";

// A word's gloss, cut into the pages its card shows one at a time (add-lingua-word-grammar). The
// pack keeps up to eight whole senses of a word; a page holds what a card showed before — about
// 160 characters — so the most common senses come first and the rest wait behind « Suivant ».
// A page holds whole senses only, and at least one however long it is.

/** What a page holds, as a card showed a whole gloss before the pack kept more senses. */
export const PAGE_CHARS = 160;

/** The pack separates a gloss's senses with this, and nothing else (the reducer ensures it). */
const SEPARATOR = "; ";

/** Senses of one part of speech on a page; `tagged` is false for a gloss with no grammar. */
export interface PageGroup {
  tagged: boolean;
  tag?: GrammarTag;
  senses: string[];
}

/** One page: its groups, in the gloss's order. */
export type GlossPage = PageGroup[];

interface Sense {
  text: string;
  group: number;
  tag?: GrammarTag;
}

/**
 * The senses of `gloss`, each with the group it belongs to. The grammar's groups are used only
 * when they make up exactly the gloss the card holds; otherwise the gloss is one untagged group.
 */
function sensesOf(gloss: string, groups: readonly SenseGroup[]): { senses: Sense[]; tagged: boolean } {
  if (groups.length > 0 && groups.map((g) => g.text).join(SEPARATOR) === gloss) {
    const senses = groups.flatMap((g, group) => g.text.split(SEPARATOR).map((text) => ({ text, group, tag: g.tag })));
    return { senses, tagged: true };
  }
  return { senses: gloss.split(SEPARATOR).map((text) => ({ text, group: 0 })), tagged: false };
}

/** `gloss` in pages of whole senses, each within `PAGE_CHARS` unless one sense is longer. */
export function glossPages(gloss: string, groups: readonly SenseGroup[] = [], pageChars = PAGE_CHARS): GlossPage[] {
  const { senses, tagged } = sensesOf(gloss, groups);
  const pages: Sense[][] = [];
  let page: Sense[] = [];
  let length = 0;
  for (const sense of senses) {
    if (page.length > 0 && length + SEPARATOR.length + sense.text.length > pageChars) {
      pages.push(page);
      page = [];
    }
    length = page.length > 0 ? length + SEPARATOR.length + sense.text.length : sense.text.length;
    page.push(sense);
  }
  if (page.length > 0) pages.push(page);
  return pages.map((p) => {
    const out: GlossPage = [];
    let last = -1;
    for (const sense of p) {
      if (sense.group !== last) {
        out.push(tagged ? { tagged, tag: sense.tag, senses: [] } : { tagged, senses: [] });
        last = sense.group;
      }
      out.at(-1)!.senses.push(sense.text);
    }
    return out;
  });
}

/** A page as one text, as the pack writes a gloss: what a card created from it stores. */
export function pageText(page: GlossPage): string {
  return page.flatMap((g) => g.senses).join(SEPARATOR);
}
