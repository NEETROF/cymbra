import { grammar } from "../i18n/fr/grammar.ts";

// The words a card uses for a form's grammar, in French (add-lingua-word-grammar, design D7): the
// French renderer of the catalogue (generalise-lingua-card-wording D2), under the API its spec
// (`test/word-grammar.spec.ts`) has always used. The pack speaks Universal Dependencies codes; the
// reader never sees one: `grammar-description.ts` describes the form in no language,
// `src/i18n/fr/grammar.ts` says it in French — a code it cannot name gives no text at all, so a pack
// from a later vocabulary can only ever show less, never a code. The card itself reads the renderer
// of the interface language beside its copy (`reading-copy.ts`), not through this module.

export type { GrammarLine, LineSegment, Named, StudiedLanguageCode } from "../i18n/index.ts";

/** A line as plain text — what a test or an accessibility label reads; the same in every language. */
export { lineText } from "./grammar-description.ts";

/** The lines a card shows about the form it was opened on, or none, in French. */
export const grammarLines = grammar.grammarLines;

/** What a reading makes of a form, in French words; null when the card says nothing of it. */
export const readingName = grammar.readingName;

/** The heading of a group of senses, in French: its part of speech, with the word's gender. */
export const senseHeading = grammar.senseHeading;

/** "a", "a et b", "a, b et c". */
export const joinFrench = grammar.join;
