// The offscreen document (Chromium). It exists for one reason — an MV3 service worker cannot
// construct a Worker — and does no work of its own: it owns the engine's worker and relays.

import { EngineChannel, type WorkerLike } from "./channel.ts";
import { isOffscreenMessage } from "./offscreen-engine.ts";

const channel = new EngineChannel(
  () => new Worker(chrome.runtime.getURL("engine-worker.js")) as unknown as WorkerLike,
);

chrome.runtime.onMessage.addListener((message: unknown, _sender, sendResponse) => {
  if (!isOffscreenMessage(message)) return undefined;
  void channel.translate(message.markup).then(sendResponse);
  return true; // answered asynchronously
});
