import type { LinguaPort } from "../analyzer/port.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";

type LevelPort = Pick<LinguaPort, "for" | "exportDeclaredLevels">;

/**
 * Whether the reader still has to choose a level: the studied language has CEFR data and no
 * level decision was ever made. « Débutant » IS a decision — the engine keeps no declared
 * level but stamps the choice (an exported row with an empty level and a timestamp) — so an
 * absent level alone does not mean the reader was never asked. The decisions are read across
 * languages, as before; telling them apart is add-lingua-language-choice's.
 */
export async function needsLevelChoice(port: LevelPort, language: StudiedLanguage): Promise<boolean> {
  const view = port.for(language);
  if (!(await view.hasLevels())) return false;
  if ((await view.declaredLevel()) !== null) return false;
  return !(await port.exportDeclaredLevels()).some((decision) => decision.updated_at > 0);
}
