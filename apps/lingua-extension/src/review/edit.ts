import type { ReviewEditError } from "../analyzer/port.ts";
import { withoutEdgePunctuation } from "../reading/selection.ts";

// The checks on an edited card (add-lingua-card-edit D3), the page's copy of the engine's: the
// engine's `reviewEdit` is the authority, this shows the same refusal before any round trip. A
// form is NFC, its whitespace collapsed, trimmed of its edge punctuation by the rule capture uses
// (`withoutEdgePunctuation`), then at most FORM_MAX_UNITS UTF-16 units — capture's own limit; a
// translation is collapsed and at most GLOSS_MAX_UNITS, empty meaning « no translation of mine ».

/** The longest form, in UTF-16 units: `MAX_SELECTION_LENGTH`, the engine's `FORM_MAX_UNITS`. */
export const FORM_MAX_UNITS = 120;
/** The longest translation, in UTF-16 units: the engine's `GLOSS_MAX_UNITS`. */
export const GLOSS_MAX_UNITS = 200;

/** Which field an error belongs to. */
export type EditField = "form" | "gloss";

/** A refused edit, tied to its field. */
export interface EditRefusal {
  field: EditField;
  error: ReviewEditError;
}

const collapsed = (text: string): string => text.normalize("NFC").replace(/\s+/gu, " ").trim();

/** The form as the engine saves it: collapsed and without its edge punctuation. */
export function editedForm(text: string): string {
  return withoutEdgePunctuation(collapsed(text));
}

/** The field an engine refusal belongs to. */
export function refusalOf(error: ReviewEditError): EditRefusal {
  return { field: error === "gloss_too_long" ? "gloss" : "form", error };
}

/** Why the engine would refuse `form` and `gloss`, the form's first, or null when it would not. */
export function checkEdit(form: string, gloss: string): EditRefusal | null {
  const saved = editedForm(form);
  if (saved.length === 0) return refusalOf("empty");
  if (saved.length > FORM_MAX_UNITS) return refusalOf("form_too_long");
  if (collapsed(gloss).length > GLOSS_MAX_UNITS) return refusalOf("gloss_too_long");
  return null;
}
