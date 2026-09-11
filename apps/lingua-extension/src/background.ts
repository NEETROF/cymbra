import { type GlueLoader, WasmAnalyzerPort, type WasmModule } from "./analyzer/engine.ts";
import { handleRpc, isRpcRequest } from "./analyzer/rpc-host.ts";
import { api, initApi } from "./net/api.ts";
import { setTokenRefresher, setUnauthenticatedHandler } from "./net/transport.ts";
import { Session } from "./state/session.ts";
import { type AsyncStorageArea, hydrateEngine, ROOT_KEY } from "./state/storage.ts";
import { getOrCreateDeviceId, SyncEngine } from "./sync/sync.ts";
// Static import of the wasm-pack glue (esbuild bundles it into the background). The
// engine hosted here must NOT dynamic-import: a Chromium service worker forbids
// `import()` (HTML spec). A static loader sidesteps that; it is harmless on the Firefox
// event page too.
import * as wasmGlue from "./wasm/pkg/lingua_wasm.js";

// The wasm-pack glue as a STATIC loader: a Chromium service worker forbids dynamic
// import(), so both engines hosted here (the rpc-host reading engine and the sync
// engine) take this instead of the content script's dynamic loader.
const staticGlue: GlueLoader = async () => wasmGlue as unknown as WasmModule;

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
  /** When true the reader is switched off on this tab: clear the badge entirely. */
  disabled?: boolean;
}

chrome.runtime.onMessage.addListener((message: unknown, sender) => {
  const msg = message as Partial<StatsMessage> | null;
  if (msg?.type !== "stats") return;
  const tabId = sender.tab?.id;
  if (tabId == null) return;
  if (msg.disabled) {
    void chrome.action.setBadgeText({ tabId, text: "" });
    return;
  }
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
  const enginePort = new WasmAnalyzerPort(staticGlue);
  let hydrated: Promise<void> | null = null;
  const ensure = (): Promise<void> => (hydrated ??= hydrateEngine(enginePort, storage));
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!isRpcRequest(message)) return undefined;
    void handleRpc(enginePort, ensure, message).then(sendResponse);
    return true; // async response
  });
}

// Account & session (add-lingua-connected-clients §1). The background owns the single
// session: it wires the transport's token getter + single-flight refresher to it, and
// the popup/settings drive sign-in through messages. Signed out, no request is ever
// made. Google sign-in runs the OAuth flow here (chrome.identity works in the worker).
{
  const sessionStore: AsyncStorageArea = {
    get: (keys) => chrome.storage.session.get(keys),
    set: (items) => chrome.storage.session.set(items),
  };
  const localStore: AsyncStorageArea = {
    get: (keys) => chrome.storage.local.get(keys),
    set: (items) => chrome.storage.local.set(items),
  };

  // "Continue with Google": OpenID implicit flow via launchWebAuthFlow. The redirect is
  // https://<ext-id>.chromiumapp.org/ (stable for a published id); the id_token comes
  // back in the URL fragment and is exchanged server-side by SignInOidc.
  const getGoogleIdToken = async (): Promise<string> => {
    if (!__GOOGLE_CLIENT_ID__) {
      throw new Error("Google sign-in is not configured in this build (missing client id).");
    }
    const redirectUri = chrome.identity.getRedirectURL();
    const auth = new URL("https://accounts.google.com/o/oauth2/v2/auth");
    auth.searchParams.set("client_id", __GOOGLE_CLIENT_ID__);
    auth.searchParams.set("response_type", "id_token");
    auth.searchParams.set("redirect_uri", redirectUri);
    auth.searchParams.set("scope", "openid email");
    auth.searchParams.set("nonce", crypto.randomUUID());
    const redirect = await chrome.identity.launchWebAuthFlow({ url: auth.toString(), interactive: true });
    const idToken = redirect ? new URLSearchParams(new URL(redirect).hash.slice(1)).get("id_token") : null;
    if (!idToken) throw new Error("Google did not return an id_token.");
    return idToken;
  };

  const session = new Session({
    auth: () => api().auth,
    sessionArea: sessionStore,
    localArea: localStore,
    getGoogleIdToken,
  });
  initApi(() => session.token());
  setTokenRefresher(() => session.refresh());
  // A terminal 401 (refresh already failed and purged) needs no extra work here; the
  // popup re-reads state on open. Left as a hook for the §2 sync-state indicator.
  setUnauthenticatedHandler(() => {});

  // Sync (§2b): a DEDICATED engine (static glue — a service worker forbids dynamic
  // import()) that the SyncEngine hydrates from the backup each run, leaving the
  // rpc-host reading engine untouched. Runs only while signed in; debounced so a burst
  // of mutations (each persisting the backup) coalesces into one exchange.
  const syncPort = new WasmAnalyzerPort(staticGlue);
  let deviceIdPromise: Promise<string> | null = null;
  let syncEngine: SyncEngine | null = null;
  let syncing = false;
  let syncPending = false;
  let syncTimer: ReturnType<typeof setTimeout> | null = null;
  const scheduleSync = (delayMs: number): void => {
    if (!session.state().signedIn) return;
    // A trigger arriving mid-sync is remembered, not lost, and drained when the current
    // run finishes — so a mutation made during a slow sync still gets pushed.
    if (syncing) {
      syncPending = true;
      return;
    }
    if (syncTimer !== null) clearTimeout(syncTimer);
    syncTimer = setTimeout(() => {
      syncTimer = null;
      void runSync();
    }, delayMs);
  };
  const runSync = async (): Promise<void> => {
    if (syncing || !session.state().signedIn) return;
    syncing = true;
    syncPending = false;
    try {
      deviceIdPromise ??= getOrCreateDeviceId(localStore);
      syncEngine ??= new SyncEngine({
        port: syncPort,
        storage: localStore,
        clients: () => ({ knownWords: api().knownWords, deck: api().deck, stats: api().stats }),
        deviceId: await deviceIdPromise,
      });
      await syncEngine.sync();
    } catch (e) {
      // Reset the memoised device id + engine so a transient failure (e.g. a storage
      // read error) retries next time instead of wedging for the worker's lifetime.
      deviceIdPromise = null;
      syncEngine = null;
      console.warn("[Cymbra Lingua] sync failed:", e);
    } finally {
      syncing = false;
      if (syncPending) scheduleSync(0); // drain a trigger that arrived during the run
    }
  };

  // Restore a session on wake and sync once if one was resumed.
  void session.resume().then((ok) => {
    if (ok) scheduleSync(0);
  });
  // A mutation in any context persists the backup → debounced sync.
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === "local" && changes[ROOT_KEY]) scheduleSync(2000);
  });

  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    const msg = message as { type?: string; email?: string; password?: string } | null;
    switch (msg?.type) {
      case "account:state":
        sendResponse(session.state());
        return false;
      case "account:signInGoogle":
        session.signInWithGoogle().then(
          () => {
            scheduleSync(0);
            sendResponse({ ok: true, state: session.state() });
          },
          (e: unknown) => sendResponse({ ok: false, error: errorMessage(e) }),
        );
        return true;
      case "account:signInLocal":
        session.signInLocal(msg.email ?? "", msg.password ?? "").then(
          () => {
            scheduleSync(0);
            sendResponse({ ok: true, state: session.state() });
          },
          (e: unknown) => sendResponse({ ok: false, error: errorMessage(e) }),
        );
        return true;
      case "account:signOut":
        session.signOut().then(() => sendResponse({ ok: true, state: session.state() }));
        return true;
      case "stats:get": {
        // Consolidated stats for the stats screen (summed across the account's devices).
        const range = message as { fromDay?: number; toDay?: number };
        if (!session.state().signedIn) {
          sendResponse({ ok: false });
          return false;
        }
        api()
          .stats.getStats({ fromDay: range.fromDay ?? 0, toDay: range.toDay ?? 0, language: "" })
          .then(
            (res) =>
              sendResponse({
                ok: true,
                rows: res.stats.map((s) => ({
                  day: s.day,
                  exposures: s.exposures,
                  wordsLearned: s.wordsLearned,
                  reviews: s.reviewsDone,
                })),
              }),
            () => sendResponse({ ok: false }),
          );
        return true;
      }
      default:
        return undefined;
    }
  });
}

function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
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
