import type { LinguaPort } from "../analyzer/port.ts";

type LevelPort = Pick<LinguaPort, "hasLevels" | "declaredLevel" | "exportDeclaredLevels">;

/**
 * Whether the reader still has to choose a level: the studied language has CEFR data and no
 * level decision was ever made. « Débutant » IS a decision — the engine keeps no declared
 * level but stamps the choice (an exported row with an empty level and a timestamp) — so an
 * absent level alone does not mean the reader was never asked.
 */
export async function needsLevelChoice(port: LevelPort): Promise<boolean> {
  if (!(await port.hasLevels())) return false;
  if ((await port.declaredLevel()) !== null) return false;
  return !(await port.exportDeclaredLevels()).some((decision) => decision.updated_at > 0);
}
