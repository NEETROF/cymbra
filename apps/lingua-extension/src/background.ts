import { type GlueLoader, WasmAnalyzerPort, type WasmModule } from "./analyzer/engine.ts";
import { handleRpc, isRpcRequest } from "./analyzer/rpc-host.ts";
import { type AsyncStorageArea, hydrateEngine } from "./state/storage.ts";
// Static import of the wasm-pack glue (esbuild bundles it into the background). The
// engine hosted here must NOT dynamic-import: a Chromium service worker forbids
// `import()` (HTML spec). A static loader sidesteps that; it is harmless on the Firefox
// event page too.
import * as wasmGlue from "./wasm/pkg/lingua_wasm.js";

// Background worker: orchestration for both variants — badge, keyboard commands, the
// activeTab-first permission posture (design D3). On Firefox (an event page, not a
// service worker) it ALSO hosts the WASM engine and answers the content script / side
// panel over the AnalyzerPort RPC, because Firefox's CSP blocks WASM in a content
// script. On Chromium the engine lives in each content script and this stays pure
// orchestration. "Always highlight" grants <all_urls>, registering the reader on every
// page; without it, the popup injects on demand via activeTab. No network.

// Badge colours are set through the chrome.action API (not CSS), so they cannot be a
// token var; they mirror the Cymbra palette: violet primaryContainer for an analysed
// page, faint slate for a page with nothing to count.
const BADGE_ACTIVE = "#7c3aed"; // --cymbra-lingua-accent-strong
const BADGE_NEUTRAL = "#6a7091"; // --cymbra-lingua-faint
const BADGE_TEXT = "#ffffff";

const READER_SCRIPT_ID = "lingua-reader";
const ALL_URLS = "<all_urls>";

interface StatsMessage {
  type: "stats";
  pct: number | null;
}

chrome.runtime.onMessage.addListener((message: unknown, sender) => {
  const msg = message as Partial<StatsMessage> | null;
  if (msg?.type !== "stats") return;
  const tabId = sender.tab?.id;
  if (tabId == null) return;
  const pct = msg.pct ?? null;
  void chrome.action.setBadgeText({ tabId, text: pct == null ? "—" : `${pct}%` });
  void chrome.action.setBadgeBackgroundColor({ tabId, color: pct == null ? BADGE_NEUTRAL : BADGE_ACTIVE });
  chrome.action.setBadgeTextColor?.({ tabId, color: BADGE_TEXT });
});

// Host the WASM engine here and serve the AnalyzerPort RPC. On Firefox (whose
// content-script CSP always blocks WASM) this is the primary engine; on Chromium it is
// the fallback for pages whose own CSP blocks the in-content engine (e.g. GitHub) — the
// content script's `resolveContentPort` routes to it over this RPC. Lazy: the engine
// instantiates on the first RPC, so a Chromium session that never hits a CSP-strict
// page pays nothing. It self-hydrates from storage on wake, so a restarted worker
// restores state before answering; the surfaces persist after their own mutations.
{
  const storage: AsyncStorageArea = {
    get: (keys) => chrome.storage.local.get(keys),
    set: (items) => chrome.storage.local.set(items),
  };
  const staticGlue: GlueLoader = async () => wasmGlue as unknown as WasmModule;
  const enginePort = new WasmAnalyzerPort(staticGlue);
  let hydrated: Promise<void> | null = null;
  const ensure = (): Promise<void> => (hydrated ??= hydrateEngine(enginePort, storage));
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!isRpcRequest(message)) return undefined;
    void handleRpc(enginePort, ensure, message).then(sendResponse);
    return true; // async response
  });
}

chrome.commands.onCommand.addListener((command) => {
  chrome.tabs.query({ active: true, currentWindow: true }, ([tab]) => {
    const tabId = tab?.id;
    if (tabId == null) return;
    if (command === "lingua-capture-selection") {
      void chrome.tabs.sendMessage(tabId, { type: "captureSelection" }).catch(() => {});
    } else if (command === "lingua-toggle-drawer") {
      void chrome.tabs.sendMessage(tabId, { type: "toggleDrawer" }).catch(() => {});
    } else if (command === "lingua-side-panel") {
      void chrome.sidePanel.open({ tabId }).catch(() => {});
    }
  });
});

/** Register the reader for every page when <all_urls> is granted; unregister otherwise. */
async function syncReaderRegistration(): Promise<void> {
  const granted = await chrome.permissions.contains({ origins: [ALL_URLS] });
  const registered = (await chrome.scripting.getRegisteredContentScripts({ ids: [READER_SCRIPT_ID] })).length > 0;
  if (granted && !registered) {
    await chrome.scripting.registerContentScripts([
      {
        id: READER_SCRIPT_ID,
        matches: [ALL_URLS],
        js: ["content.js"],
        runAt: "document_idle",
      },
    ]);
  } else if (!granted && registered) {
    await chrome.scripting.unregisterContentScripts({ ids: [READER_SCRIPT_ID] });
  }
}

chrome.runtime.onInstalled.addListener(() => void syncReaderRegistration());
chrome.runtime.onStartup.addListener(() => void syncReaderRegistration());

// When the user grants "always highlight", register the reader and light up the
// current tab immediately so the grant has a visible effect without a reload.
chrome.permissions.onAdded.addListener((perms) => {
  if (!perms.origins?.includes(ALL_URLS)) return;
  void syncReaderRegistration().then(async () => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab?.id != null)
      await chrome.scripting.executeScript({ target: { tabId: tab.id }, files: ["content.js"] }).catch(() => {});
  });
});
chrome.permissions.onRemoved.addListener((perms) => {
  if (perms.origins?.includes(ALL_URLS)) void syncReaderRegistration();
});
