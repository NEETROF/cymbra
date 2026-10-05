// How a surface drives « Traduction étendue »: it asks the background, which alone downloads,
// deletes and records where the model stands (translate/host/model-controller.ts). Every answer is
// the whole status, so a surface never assembles it from pieces. Surfaces may import this module.

import { ABSENT, type ModelState, parseHost, parseModelState, type TranslationHost } from "./setting.ts";

export const MODEL_MESSAGE = "lingua-model";

/** status: where things stand. enable / disable: the checkbox. resume: try again, resume, download again. */
export type ModelCommand = "status" | "enable" | "disable" | "resume";

const COMMANDS: readonly ModelCommand[] = ["status", "enable", "disable", "resume"];

export interface ModelMessage {
  type: typeof MODEL_MESSAGE;
  op: ModelCommand;
}

/** What the setting costs, in bytes, from the package's catalogue (generalise-lingua-translation-catalogue D4). */
export interface ModelCost {
  /** Downloaded once: the model's files as served. */
  download: number;
  /** Kept on the device: its files decompressed. */
  stored: number;
  /** A needed route goes through another language, so two models translate at once (add-lingua-spanish-translation-pivot D4). */
  pivot?: boolean;
}

export interface ModelStatus {
  /** Whether this browser offers the setting at all (not on Firefox for Android — D8). */
  offered: boolean;
  host: TranslationHost;
  state: ModelState;
  /** Absent when the background could not read the catalogue: the setting then states no size. */
  cost?: ModelCost;
}

export const NOT_OFFERED: ModelStatus = { offered: false, host: "none", state: ABSENT };

export function isModelMessage(message: unknown): message is ModelMessage {
  const m = message as Partial<ModelMessage> | null;
  return m?.type === MODEL_MESSAGE && COMMANDS.includes(m.op as ModelCommand);
}

const positive = (n: unknown): n is number => typeof n === "number" && Number.isFinite(n) && n > 0;

/** A cost, or none when either size is missing or not a size. */
function asCost(raw: unknown): ModelCost | undefined {
  const c = raw as Partial<ModelCost> | null | undefined;
  if (!c || !positive(c.download) || !positive(c.stored)) return undefined;
  return { download: c.download, stored: c.stored, ...(c.pivot === true ? { pivot: true } : {}) };
}

/** A reply, read defensively: anything unreadable means the setting is not offered here. */
export function asModelStatus(reply: unknown): ModelStatus {
  const r = reply as Partial<ModelStatus> | null | undefined;
  if (!r || typeof r.offered !== "boolean") return NOT_OFFERED;
  const cost = asCost(r.cost);
  return { offered: r.offered, host: parseHost(r.host), state: parseModelState(r.state), ...(cost ? { cost } : {}) };
}

export type ModelSend = (message: ModelMessage) => Promise<unknown>;

const runtimeSend: ModelSend = (message) => chrome.runtime.sendMessage(message);

/** Ask the background. A background that cannot answer offers nothing. */
export async function askModel(op: ModelCommand, send: ModelSend = runtimeSend): Promise<ModelStatus> {
  try {
    return asModelStatus(await send({ type: MODEL_MESSAGE, op }));
  } catch {
    return NOT_OFFERED;
  }
}
