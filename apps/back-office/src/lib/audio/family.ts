// The instrument-family vocabulary of the SoundFont catalog (change:
// add-drum-audio-channel). Fonts speak the same two families as the scores —
// `keyboard`/`percussion` — so every comparison site (the preview picker's
// filter, the drawer) is a plain equality, never a mapping.

/** The two families a stored font can be. Scores also know `unknown`, but a
 *  font's family is verified against its preset banks at upload, so "we could
 *  not tell" is not a state a stored font can be in. */
export type SoundFontFamily = "keyboard" | "percussion";

/** Normalises a wire `instrument` value onto the score vocabulary: the legacy
 *  `piano` spelling — and an absent value — reads as `keyboard`, permanently,
 *  mirroring the backend's upload-boundary normalisation. */
export function familyOf(instrument: string | null | undefined): SoundFontFamily {
  return instrument === "percussion" ? "percussion" : "keyboard";
}
