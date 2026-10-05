import { describe, expect, it } from "vitest";
import { rarityText } from "@/reading/selection-card.ts";

// The word card's frequency line (add-lingua-card-frequency): the band of the dictionary form's rank
// in the pack, whatever the reader's level. `toLocaleString("fr-FR")` groups with a narrow no-break
// space (U+202F), spelled out here so a future edit cannot quietly change what a reader sees.

describe("what a word card says of how common its word is", () => {
  it("names the band of the word's rank, at each bound", () => {
    const bands: Array<[number, string]> = [
      [1, "Très courant — parmi les 100 mots les plus fréquents."],
      [100, "Très courant — parmi les 100 mots les plus fréquents."],
      [101, "Courant — parmi les 1\u202f000 mots les plus fréquents."],
      [1_000, "Courant — parmi les 1\u202f000 mots les plus fréquents."],
      [1_001, "Assez courant — parmi les 5\u202f000 mots les plus fréquents."],
      [5_000, "Assez courant — parmi les 5\u202f000 mots les plus fréquents."],
      [5_001, "Peu fréquent — au-delà des 5\u202f000 mots les plus fréquents."],
      [20_000, "Peu fréquent — au-delà des 5\u202f000 mots les plus fréquents."],
      [20_001, "Rare — au-delà des 20\u202f000 mots les plus fréquents."],
    ];
    for (const [rank, line] of bands) expect(rarityText("Unknown", rank), String(rank)).toBe(line);
  });

  it("calls a word the pack does not rank rare", () => {
    expect(rarityText("Unknown", null)).toBe("Rare — au-delà des 20\u202f000 mots les plus fréquents.");
  });

  it("says the same of a known word: the line is about the word, not the reader", () => {
    expect(rarityText("Known", 22)).toBe(rarityText("Unknown", 22));
  });

  it("says nothing before the rank is there", () => {
    expect(rarityText("Unknown", undefined)).toBe("");
  });

  it("says a word in the deck is being learnt, whatever its rank", () => {
    expect(rarityText("Learning", 22)).toBe("Dans ton deck — en cours d'apprentissage.");
    expect(rarityText("Learning", null)).toBe("Dans ton deck — en cours d'apprentissage.");
    expect(rarityText("Learning", undefined)).toBe("Dans ton deck — en cours d'apprentissage.");
  });
});
