// Chromium only. An MV3 service worker cannot construct a Worker — measured, `typeof Worker`
// is undefined there — so the engine's worker, and the model download's, are owned by an
// offscreen document the service worker creates when one of them is needed. The document does
// no work itself; this class is the service worker's handle on it, and `serveOffscreen` is the
// document's side of the same conversation.
//
// The document reports back on its own (a download's progress, the engine gone idle) with
// OFFSCREEN_EVENT messages: a download outlives any single request, and a service worker can be
// stopped and restarted while it runs — the next report wakes it.

import type { DownloadEvent } from "./downloads.ts";
import { type EngineAccess, type EngineReply, isEngineReply } from "./engine.ts";

export const OFFSCREEN_TYPE = "lingua-translate-offscreen";
export const OFFSCREEN_EVENT = "lingua-translate-offscreen-event";
export const OFFSCREEN_URL = "offscreen.html";
export const OFFSCREEN_JUSTIFICATION =
  "Runs the translation engine, and downloads its model, off every thread that paints.";

export type OffscreenRequest =
  | { type: typeof OFFSCREEN_TYPE; op: "translate"; markup: string; language: string }
  | { type: typeof OFFSCREEN_TYPE; op: "warm"; language: string }
  | { type: typeof OFFSCREEN_TYPE; op: "download"; models: string[] }
  | { type: typeof OFFSCREEN_TYPE; op: "cancel" | "downloading" };

export type OffscreenEvent =
  { type: typeof OFFSCREEN_EVENT; event: DownloadEvent } | { type: typeof OFFSCREEN_EVENT; idle: true };

const OPS = ["translate", "warm", "download", "cancel", "downloading"];

export function isOffscreenRequest(message: unknown): message is OffscreenRequest {
  const m = message as { type?: unknown; op?: unknown; markup?: unknown; language?: unknown; models?: unknown } | null;
  if (m?.type !== OFFSCREEN_TYPE || !OPS.includes(m.op as string)) return false;
  switch (m.op) {
    case "translate":
      return typeof m.markup === "string" && typeof m.language === "string";
    case "warm":
      return typeof m.language === "string";
    case "download":
      return Array.isArray(m.models) && m.models.every((id) => typeof id === "string");
    default:
      return true;
  }
}

export function isOffscreenEvent(message: unknown): message is OffscreenEvent {
  const m = message as { type?: unknown; event?: { kind?: unknown }; idle?: unknown } | null;
  return m?.type === OFFSCREEN_EVENT && (m.idle === true || typeof m.event?.kind === "string");
}

/** The slice of chrome.offscreen this needs — a test hands in a fake. */
export interface OffscreenApi {
  hasDocument(): Promise<boolean>;
  createDocument(parameters: { url: string; reasons: string[]; justification: string }): Promise<void>;
  closeDocument(): Promise<void>;
}

export type OffscreenSend = (message: OffscreenRequest) => Promise<unknown>;

export class OffscreenEngine implements EngineAccess {
  private ready: Promise<void> | null = null;

  constructor(
    private readonly api: OffscreenApi,
    private readonly send: OffscreenSend,
  ) {}

  async translate(markup: string, language: string): Promise<EngineReply> {
    try {
      await this.ensure();
    } catch {
      return { ok: false, reason: "the offscreen document could not be created" };
    }
    try {
      const reply = await this.send({ type: OFFSCREEN_TYPE, op: "translate", markup, language });
      return isEngineReply(reply) ? reply : { ok: false, reason: "the offscreen document gave no answer" };
    } catch {
      // The document went away (Chrome may close it). Forget it, so the next request makes one.
      this.ready = null;
      return { ok: false, reason: "the offscreen document is gone" };
    }
  }

  /** Load the engine and `language`'s route in the document, creating it when needed; translate nothing. */
  async warm(language: string): Promise<boolean> {
    try {
      await this.ensure();
    } catch {
      return false;
    }
    try {
      return (await this.send({ type: OFFSCREEN_TYPE, op: "warm", language })) === true;
    } catch {
      this.ready = null; // gone: the next request makes another
      return false;
    }
  }

  /** Start downloading `models` (catalogue ids) in the document; it reports back with OFFSCREEN_EVENT. */
  async startDownload(models: string[]): Promise<void> {
    await this.ensure();
    await this.send({ type: OFFSCREEN_TYPE, op: "download", models });
  }

  /** Stop a download in progress. With no document there is none to stop. */
  async cancelDownload(): Promise<void> {
    if (!(await this.api.hasDocument())) return;
    await this.send({ type: OFFSCREEN_TYPE, op: "cancel" }).catch(() => undefined);
  }

  /** Whether a download is running — which only a live document can say. */
  async downloading(): Promise<boolean> {
    if (!(await this.api.hasDocument())) return false;
    try {
      return (await this.send({ type: OFFSCREEN_TYPE, op: "downloading" })) === true;
    } catch {
      return false;
    }
  }

  /** Close the document, and with it every worker it owns: the engine's memory is given back. */
  async close(): Promise<void> {
    this.ready = null;
    if (!(await this.api.hasDocument())) return;
    await this.api.closeDocument().catch(() => undefined);
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

/** What the document owns: the engine's channel and the download's host. */
export interface OffscreenParts {
  channel: {
    translate(markup: string, language: string): Promise<EngineReply>;
    warm(language: string): Promise<boolean>;
    running(): boolean;
  };
  downloads: { start(models: string[]): void; cancel(): void; running(): boolean };
  /** Ask the browser to keep the model's storage: only a document can (navigator.storage.persist). */
  persist?: () => void;
}

/**
 * The document's side: answer one request. Returns true when the answer comes later, as a
 * chrome.runtime.onMessage listener must.
 */
export function serveOffscreen(
  message: OffscreenRequest,
  parts: OffscreenParts,
  sendResponse: (reply: unknown) => void,
): boolean {
  switch (message.op) {
    case "translate":
      void parts.channel.translate(message.markup, message.language).then(sendResponse);
      return true;
    case "warm":
      void parts.channel.warm(message.language).then(sendResponse);
      return true;
    case "download":
      parts.persist?.();
      parts.downloads.start(message.models);
      sendResponse(true);
      return false;
    case "cancel":
      parts.downloads.cancel();
      sendResponse(true);
      return false;
    case "downloading":
      sendResponse(parts.downloads.running());
      return false;
  }
}

/** Whether the document holds nothing worth keeping it open for. */
export function offscreenIdle(parts: OffscreenParts): boolean {
  return !parts.channel.running() && !parts.downloads.running();
}
