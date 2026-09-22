import { describe, expect, it } from "vitest";
import { rarityText } from "@/reading/selection-card.ts";

describe("what a word popup says about rarity", () => {
  it("speaks of the level when a level is what the reader declared", () => {
    // `onSetLevel` pins the calibration to 0 on purpose — the level becomes the only source
    // of presumed-known. Rendering that 0 told every such reader they knew "0 mots les plus
    // courants", and declaring a level is the normal path (the pack ships CEFR levels).
    expect(rarityText("Unknown", 0)).toBe("Peu fréquent — au-delà de ton niveau.");
  });

  it("still names the count when the reader calibrated by frequency instead", () => {
    // `toLocaleString("fr-FR")` groups with a narrow no-break space (U+202F), not a plain
    // one — spelled out here so a future edit cannot quietly change what a reader sees.
    expect(rarityText("Unknown", 3000)).toBe("Peu fréquent — au-delà de tes 3\u202f000 mots les plus courants.");
  });

  it("says nothing about rarity for a word already in the deck", () => {
    expect(rarityText("Learning", 0)).toBe("Dans ton deck — en cours d'apprentissage.");
    expect(rarityText("Learning", 3000)).toBe("Dans ton deck — en cours d'apprentissage.");
  });
});
