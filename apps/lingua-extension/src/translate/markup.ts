// The selection travels to the engine as markup, and that is the whole trick: Bergamot, asked
// to preserve HTML, repositions a tag onto the target span the tagged source corresponds to.
// Translating the selection in its sentence — rather than alone — is what gives it the form
// the context imposes: `<b>gave up</b>` comes back `<b>a abandonné</b>`, conjugated and
// agreed with its subject, where the fragment alone would give an infinitive.
//
// The consequence is that page text becomes markup input. Everything taken from the page is
// escaped before the tag is placed, so the only tag the engine ever sees is ours.

/** The tag that marks the selection. Page text is escaped, so no other tag reaches the engine. */
const OPEN = "<b>";
const CLOSE = "</b>";

/** A span of plain text, as [start, end) offsets. */
export interface Span {
  start: number;
  end: number;
}

/** Escape text so that it reads as text, never as markup. */
export function escapeText(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/**
 * The sentence as markup, with `selection` tagged. Offsets are into `sentence` as given; an
 * empty or out-of-range selection is clamped, and one that covers nothing gives no tag at
 * all — the sentence is still worth translating.
 */
export function markSelection(sentence: string, selection: Span | null): string {
  const span = clamp(sentence, selection);
  if (!span) return escapeText(sentence);
  return (
    escapeText(sentence.slice(0, span.start)) +
    OPEN +
    escapeText(sentence.slice(span.start, span.end)) +
    CLOSE +
    escapeText(sentence.slice(span.end))
  );
}

/** The selected text, clamped exactly as `markSelection` clamps it; null when it covers nothing. */
export function selectedText(sentence: string, selection: Span | null): string | null {
  const span = clamp(sentence, selection);
  return span ? sentence.slice(span.start, span.end) : null;
}

function clamp(sentence: string, selection: Span | null): Span | null {
  if (!selection) return null;
  const start = Math.max(0, Math.min(selection.start, sentence.length));
  const end = Math.max(start, Math.min(selection.end, sentence.length));
  return end > start ? { start, end } : null;
}

const NAMED: Readonly<Record<string, string>> = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: " " };

/** One character reference, decoded; an unknown one is left exactly as written. */
function decodeEntity(entity: string): string {
  const body = entity.slice(1, -1);
  if (body.startsWith("#x") || body.startsWith("#X")) {
    const code = Number.parseInt(body.slice(2), 16);
    return Number.isFinite(code) ? String.fromCodePoint(code) : entity;
  }
  if (body.startsWith("#")) {
    const code = Number.parseInt(body.slice(1), 10);
    return Number.isFinite(code) ? String.fromCodePoint(code) : entity;
  }
  return NAMED[body] ?? entity;
}

/** What the engine answered, read back out of its markup. */
export interface MarkedTranslation {
  /** The whole translated sentence, as plain text. */
  sentence: string;
  /**
   * Where the selection landed in `sentence`. Usually one span; more when the engine split the
   * tag across a reordering; none when it dropped the tag, which it may.
   */
  marks: Span[];
}

/**
 * Read the engine's markup answer: decode it to plain text, and record where our tag landed.
 * Written without a DOM parser because a worker has none, and so the rule is the same in
 * every context that reads it. A tag other than ours is treated as zero-width — it cannot have
 * come from the page, whose text was escaped, but the answer is read defensively all the same.
 */
export function readMarked(html: string): MarkedTranslation {
  let text = "";
  const marks: Span[] = [];
  let openAt: number | null = null;
  const token = /<\/?[a-zA-Z][^>]*>|&(?:#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z]+);/g;
  let last = 0;
  for (let m = token.exec(html); m !== null; m = token.exec(html)) {
    text += html.slice(last, m.index);
    last = m.index + m[0].length;
    const t = m[0];
    if (t.startsWith("&")) {
      text += decodeEntity(t);
    } else if (/^<b\b/i.test(t)) {
      openAt ??= text.length;
    } else if (/^<\/b\b/i.test(t)) {
      if (openAt !== null && text.length > openAt) marks.push({ start: openAt, end: text.length });
      openAt = null;
    }
  }
  text += html.slice(last);
  return trimMarks({ sentence: text, marks });
}

/** Keep each mark on text, not on the spaces around it, so a highlight never starts on a blank. */
function trimMarks({ sentence, marks }: MarkedTranslation): MarkedTranslation {
  const out: Span[] = [];
  for (let { start, end } of marks) {
    while (start < end && /\s/.test(sentence.charAt(start))) start++;
    while (end > start && /\s/.test(sentence.charAt(end - 1))) end--;
    if (end > start) out.push({ start, end });
  }
  return { sentence, marks: out };
}
