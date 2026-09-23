// Chromium only. An MV3 service worker cannot construct a Worker — measured, `typeof Worker`
// is undefined there — so the engine's worker is owned by an offscreen document the service
// worker creates on first use. The document does no work itself; this class is the service
// worker's handle on it.

import { type EngineAccess, type EngineReply, isEngineReply } from "./engine.ts";

export const OFFSCREEN_TYPE = "lingua-translate-offscreen";
export const OFFSCREEN_URL = "offscreen.html";
export const OFFSCREEN_JUSTIFICATION = "Runs the translation engine off every thread that paints.";

export interface OffscreenMessage {
  type: typeof OFFSCREEN_TYPE;
  markup: string;
}

export function isOffscreenMessage(message: unknown): message is OffscreenMessage {
  const m = message as Partial<OffscreenMessage> | null;
  return m?.type === OFFSCREEN_TYPE && typeof m.markup === "string";
}

/** The slice of chrome.offscreen this needs — a test hands in a fake. */
export interface OffscreenApi {
  hasDocument(): Promise<boolean>;
  createDocument(parameters: { url: string; reasons: string[]; justification: string }): Promise<void>;
}

export type OffscreenSend = (message: OffscreenMessage) => Promise<unknown>;

export class OffscreenEngine implements EngineAccess {
  private ready: Promise<void> | null = null;

  constructor(
    private readonly api: OffscreenApi,
    private readonly send: OffscreenSend,
  ) {}

  async translate(markup: string): Promise<EngineReply> {
    try {
      await this.ensure();
    } catch {
      return { ok: false, reason: "the offscreen document could not be created" };
    }
    try {
      const reply = await this.send({ type: OFFSCREEN_TYPE, markup });
      return isEngineReply(reply) ? reply : { ok: false, reason: "the offscreen document gave no answer" };
    } catch {
      // The document went away (Chrome may close it). Forget it, so the next request makes one.
      this.ready = null;
      return { ok: false, reason: "the offscreen document is gone" };
    }
  }

  /**
   * One document, created once. It can outlive the service worker, so a restarted worker asks
   * before creating: a second `createDocument` is refused while the first one exists.
   */
  private ensure(): Promise<void> {
    this.ready ??= (async () => {
      if (await this.api.hasDocument()) return;
      await this.api.createDocument({
        url: OFFSCREEN_URL,
        reasons: ["WORKERS"],
        justification: OFFSCREEN_JUSTIFICATION,
      });
    })().catch((e: unknown) => {
      this.ready = null;
      throw e;
    });
    return this.ready;
  }
}
