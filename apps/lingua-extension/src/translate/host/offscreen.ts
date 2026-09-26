// The offscreen document (Chromium). It exists for one reason — an MV3 service worker cannot
// construct a Worker — and does no work of its own: it owns the engine's worker and the model
// download's, relays to them, and says when it holds neither, so the service worker can close it.

import { EngineChannel, type WorkerLike } from "./channel.ts";
import { DownloadHost, type DownloadWorkerLike } from "./downloads.ts";
import {
  isOffscreenRequest,
  OFFSCREEN_EVENT,
  type OffscreenEvent,
  offscreenIdle,
  type OffscreenParts,
  serveOffscreen,
} from "./offscreen-engine.ts";

const report = (message: OffscreenEvent): void => void chrome.runtime.sendMessage(message).catch(() => {});

/** Nothing left running: the service worker closes the document, giving its memory back. */
const reportIfIdle = (): void => {
  if (offscreenIdle(parts)) report({ type: OFFSCREEN_EVENT, idle: true });
};

const parts: OffscreenParts = {
  channel: new EngineChannel(() => new Worker(chrome.runtime.getURL("engine-worker.js")) as unknown as WorkerLike, {
    onIdle: reportIfIdle,
  }),
  downloads: new DownloadHost(
    () => new Worker(chrome.runtime.getURL("model-worker.js")) as unknown as DownloadWorkerLike,
    (event) => report({ type: OFFSCREEN_EVENT, event }),
    { onEnded: reportIfIdle },
  ),
  persist: () => void navigator.storage?.persist?.().catch(() => {}),
};

chrome.runtime.onMessage.addListener((message: unknown, _sender, sendResponse) => {
  if (!isOffscreenRequest(message)) return undefined;
  return serveOffscreen(message, parts, sendResponse);
});
