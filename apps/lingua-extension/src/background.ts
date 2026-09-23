import { type GlueLoader, WasmAnalyzerPort, type WasmModule } from "./analyzer/engine.ts";
import { handleRpc, isRpcRequest } from "./analyzer/rpc-host.ts";
import { type AccountHostDeps, handleAccountMessage } from "./account/host.ts";
import { userServicePort } from "./account/profile.ts";
import { type AccountReply, isAccountMessage } from "./account/messages.ts";
import { api, initApi } from "./net/api.ts";
import { setTokenRefresher, setUnauthenticatedHandler } from "./net/transport.ts";
import {
  hostAppSignInUrl,
  NATIVE_APP_ID,
  type NativeSend,
  nativeProviders,
  takeHandedIdToken,
} from "./state/native-signin.ts";
import {
  appleAuthorizeRequest,
  availableProviders,
  googleAuthorizeRequest,
  type Provider,
  runAuthFlow,
} from "./state/oidc.ts";
import { isOpenPageMessage } from "./state/open-page.ts";
import { EngineChannel, type WorkerLike } from "./translate/host/channel.ts";
import type { EngineAccess } from "./translate/host/engine.ts";
import { OffscreenEngine } from "./translate/host/offscreen-engine.ts";
import { KEEPALIVE_PING } from "./translate/keepalive.ts";
import { relayTranslation } from "./translate/host/relay.ts";
import { isTranslateMessage } from "./translate/wire.ts";
import { Session } from "./state/session.ts";
import { type AsyncStorageArea, hydrateEngine, ROOT_KEY, SESSION_LOST_KEY } from "./state/storage.ts";
import {
  idbArea,
  isStoreMessage,
  migrateStore,
  openStore,
  ownerArea,
  STORE_CHANGED_KEY,
  type StoreChange,
  type StoreReply,
} from "./state/store.ts";
import { isSyncMessage, LAST_SYNC_KEY, loadLastSync, type SyncReply } from "./sync/messages.ts";
import { PAGE_INTERVAL_MS, SURFACE_INTERVAL_MS, SyncScheduler } from "./sync/scheduler.ts";
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

// A session this device held was refused by the server: the toolbar icon carries an alert
// dot until the reader signs in again, so "not syncing any more" is visible at a glance
// (the badge keeps showing the page's percentage).
const PLAIN_ICON = { 16: "icons/icon-16.png", 32: "icons/icon-32.png", 48: "icons/icon-48.png" };
const ALERT_ICON = { 16: "icons/icon-alert-16.png", 32: "icons/icon-alert-32.png", 48: "icons/icon-alert-48.png" };

function showSessionLost(lost: boolean): void {
  // setIcon is unavailable on some builds; the popup and drawer still say it.
  void chrome.action.setIcon?.({ path: lost ? ALERT_ICON : PLAIN_ICON })?.catch(() => {});
}

void chrome.storage.local.get(SESSION_LOST_KEY).then((got) => showSessionLost(got[SESSION_LOST_KEY] === true));
chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes[SESSION_LOST_KEY]) showSessionLost(changes[SESSION_LOST_KEY].newValue === true);
});

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

// The in-page HUD asks the background to open the Chromium Side Panel on a given view. The
// open is issued SYNCHRONOUSLY inside the message listener, using sender.tab.id, so the
// content-script click's user activation is still live (sidePanel.open requires it); a tab
// of the same page is the rare fallback. Firefox never sends this — it stays in the page via
// the in-page drawer (content.ts routes to the drawer there), so this is Chromium-only.
const PANEL_VIEW_KEY = "cymbra-lingua-panel-view";

chrome.runtime.onMessage.addListener((message: unknown, sender) => {
  const m = message as { type?: string; view?: string } | null;
  if (m?.type !== "openPanel") return;
  const view = m.view === "settings" || m.view === "review" ? m.view : "stats";
  // Fire-and-forget: the panel page reads this on init; awaiting it would spend the gesture.
  void chrome.storage.session.set({ [PANEL_VIEW_KEY]: view }).catch(() => {});
  const openTab = (): void => void chrome.tabs.create({ url: chrome.runtime.getURL("sidepanel.html") });
  const tabId = sender.tab?.id;
  // Gated on __REVIEW_IN_PAGE__ (not just the runtime chrome.sidePanel?.open check) so esbuild
  // drops this call from the Firefox/Safari bundle: the API is unimplemented there, and the
  // AMO linter flags any reference to it in the bundle text even when it is unreachable.
  if (!__REVIEW_IN_PAGE__ && chrome.sidePanel?.open && tabId != null) chrome.sidePanel.open({ tabId }).catch(openTab);
  else openTab();
});

// A surface that cannot open a tab asks here: a content script has no `chrome.tabs`, which
// is why the in-page drawer's links did nothing (dogfooding, build 100/101). An extension
// path is resolved; a browser page (about:addons, chrome://extensions/shortcuts) is opened
// as given.
chrome.runtime.onMessage.addListener((message: unknown) => {
  if (!isOpenPageMessage(message)) return;
  const url = /^[a-z-]+:/.test(message.url) ? message.url : chrome.runtime.getURL(message.url);
  void chrome.tabs.create({ url }).catch(() => {});
});

// The reader's data — engine backup, daily statistics, sync cursors — lives in IndexedDB,
// owned here (change: move-lingua-store-to-indexeddb). A content script cannot open the
// extension's database (it runs in the visited page's origin), so every other surface asks
// this listener, and follows the store through a marker in chrome.storage.local, the one
// change channel that reaches every context and survives a suspended background page.
const settingsArea: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

let storeRev = 0;

/**
 * Set by the sync block: a change to the reader's own data schedules an exchange. The
 * owner is what knows when that happens — the trigger used to watch the backup key in
 * chrome.storage.local, which stopped firing the moment the data moved to the store
 * (dogfooding: a level chosen right after an erasure never left the phone).
 */
let onReaderDataChanged: (() => void) | null = null;

/** Say which keys just changed; surfaces re-read what they care about. */
function announceStoreChange(keys: string[]): void {
  storeRev += 1;
  void chrome.storage.local.set({ [STORE_CHANGED_KEY]: { rev: storeRev, keys } satisfies StoreChange }).catch(() => {});
  if (keys.includes(ROOT_KEY)) onReaderDataChanged?.();
}

/**
 * The database, opened once, with the reader's data copied out of chrome.storage.local on
 * the first start after the update. A database that will not open is not fatal: the
 * settings area still holds a bounded state, so the reader keeps reading and reviewing.
 */
const storeArea: Promise<AsyncStorageArea> = (async () => {
  let area: AsyncStorageArea;
  try {
    area = idbArea(await openStore());
  } catch (e) {
    console.warn("[Cymbra Lingua] durable store unavailable, staying on storage.local:", e);
    return settingsArea;
  }
  // A migration that fails must not cost us the store: it is retried at the next start,
  // and copying only what the store lacks makes that safe.
  try {
    const moved = await migrateStore(settingsArea, area);
    if (moved.length > 0) console.info(`[Cymbra Lingua] moved ${moved.length} keys into the durable store`);
  } catch (e) {
    console.warn("[Cymbra Lingua] could not finish moving the previous state:", e);
  }
  void navigator.storage?.persist?.().catch(() => {});
  return area;
})();

/** The owner's own handle: writes announce themselves, like a surface's would. */
const ownedStore: AsyncStorageArea = ownerArea(
  {
    get: async (keys) => (await storeArea).get(keys),
    set: async (items) => (await storeArea).set(items),
  },
  announceStoreChange,
);

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!isStoreMessage(message)) return undefined;
  void (async () => {
    const area = await storeArea;
    if (message.type === "store:get") {
      sendResponse({ ok: true, items: await area.get(message.keys) } satisfies StoreReply);
      return;
    }
    await ownedStore.set(message.items); // announces, and schedules the sync
    sendResponse({ ok: true } satisfies StoreReply);
  })().catch((e: unknown) => {
    console.warn("[Cymbra Lingua] store request failed:", e);
    sendResponse({ ok: false } satisfies StoreReply);
  });
  return true; // async response
});

// Host the WASM engine here and serve the AnalyzerPort RPC. On Firefox (whose
// content-script CSP always blocks WASM) this is the primary engine; on Chromium it is
// the fallback for pages whose own CSP blocks the in-content engine (e.g. GitHub) — the
// content script's `resolveContentPort` routes to it over this RPC. Lazy: the engine
// instantiates on the first RPC, so a Chromium session that never hits a CSP-strict
// page pays nothing. It self-hydrates from storage on wake, so a restarted worker
// restores state before answering; the surfaces persist after their own mutations.
{
  const storage = ownedStore;
  const enginePort = new WasmAnalyzerPort(staticGlue);
  let hydrated: Promise<void> | null = null;
  const ensure = (): Promise<void> => (hydrated ??= hydrateEngine(enginePort, storage));
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!isRpcRequest(message)) return undefined;
    void handleRpc(enginePort, ensure, message).then(sendResponse);
    return true; // async response
  });
}

// The translation engine (add-lingua-translation-engine), built in only by a development
// build that side-loads a model — every shipped build folds this block away. The background
// relays and never translates itself: the engine runs in a worker of its own, so the analyser
// RPC above keeps answering while a sentence is being translated. On Chromium that worker is
// owned by an offscreen document, because a service worker cannot construct one.
if (__TRANSLATION_HOST__ !== "none") {
  const engine: EngineAccess =
    __TRANSLATION_HOST__ === "offscreen"
      ? new OffscreenEngine(chrome.offscreen, (message) => chrome.runtime.sendMessage(message))
      : new EngineChannel(() => new Worker(chrome.runtime.getURL("engine-worker.js")) as unknown as WorkerLike);
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!isTranslateMessage(message)) return undefined;
    void relayTranslation(engine, message.request).then(sendResponse);
    return true; // async response
  });
  // A reader page pings while it is being read (translate/keepalive.ts): answering is what
  // keeps this page loaded, and with it the worker and the model it has already read, so the
  // reader's next selection is not a cold start. An open port does not do it — measured.
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if ((message as { type?: unknown } | null)?.type !== KEEPALIVE_PING) return undefined;
    sendResponse(true);
    return false;
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

  // Provider sign-in (Google, Apple): an authorization request through launchWebAuthFlow.
  // The redirect is https://<ext-id>.chromiumapp.org/ (stable via the manifest key); the
  // id_token comes back in the URL fragment and is exchanged server-side by SignInOidc. A
  // closed window resolves to null (a cancel). Apple is asked for NO scope so it answers
  // in the fragment (add-lingua-account-parity, design D3).
  const launch = async (url: string): Promise<string | undefined> =>
    chrome.identity.launchWebAuthFlow({ url, interactive: true });
  const requestParams = (clientId: string) => ({
    clientId,
    redirectUri: chrome.identity.getRedirectURL(),
    state: crypto.randomUUID(),
    nonce: crypto.randomUUID(),
  });
  const getGoogleIdToken = async (): Promise<string | null> => {
    if (!__GOOGLE_CLIENT_ID__) throw new Error("Google sign-in is not configured in this build.");
    return runAuthFlow(launch, googleAuthorizeRequest(requestParams(__GOOGLE_CLIENT_ID__)));
  };
  const getAppleIdToken = async (): Promise<string | null> => {
    if (!__APPLE_CLIENT_ID__) throw new Error("Apple sign-in is not configured in this build.");
    return runAuthFlow(launch, appleAuthorizeRequest(requestParams(__APPLE_CLIENT_ID__)));
  };

  const session = new Session({
    auth: () => api().auth,
    sessionArea: sessionStore,
    localArea: localStore,
    getGoogleIdToken,
    getAppleIdToken,
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
  const getSyncEngine = async (): Promise<SyncEngine> => {
    deviceIdPromise ??= getOrCreateDeviceId(ownedStore);
    syncEngine ??= new SyncEngine({
      port: syncPort,
      storage: ownedStore,
      clients: () => ({ knownWords: api().knownWords, deck: api().deck, stats: api().stats, data: api().data }),
      deviceId: await deviceIdPromise,
    });
    return syncEngine;
  };
  const scheduler = new SyncScheduler({
    sync: async () => (await getSyncEngine()).sync().then(() => undefined),
    signedIn: () => session.state().signedIn,
    now: () => Date.now(),
    lastSynced: async () => (await loadLastSync(localStore)) ?? 0,
    onSynced: (at) => localStore.set({ [LAST_SYNC_KEY]: at }),
    onError: (e) => {
      // Reset the memoised device id + engine so a transient failure (e.g. a storage
      // read error) retries next time instead of wedging for the worker's lifetime.
      deviceIdPromise = null;
      syncEngine = null;
      console.warn("[Cymbra Lingua] sync failed:", e);
    },
  });
  // « Effacer mes données Lingua » (add-lingua-privacy-controls): no sync may push between
  // the server erasure and the local wipe, so runs are held and a running one awaited; the
  // held triggers then pull back anything created elsewhere since the erasure.
  const eraseLinguaData = (): Promise<void> => scheduler.exclusive(async () => (await getSyncEngine()).eraseAll());

  // Safari (add-lingua-connected-clients D6): Apple and Google run in the host app, which
  // hands the id_token back through this extension's native handler.
  // Everything native stays inside the define's branch, so the other variants fold it away.
  const native = __NATIVE_PROVIDERS__
    ? (() => {
        const send: NativeSend = (message) => chrome.runtime.sendNativeMessage(NATIVE_APP_ID, message);
        return {
          providers: () => nativeProviders(send),
          handOff: {
            open: async (provider: Provider): Promise<void> => {
              await chrome.tabs.create({ url: hostAppSignInUrl(provider) });
            },
            take: () => takeHandedIdToken(send),
          },
        };
      })()
    : undefined;
  const handOff = native?.handOff;
  const accountDeps: AccountHostDeps = {
    session,
    account: userServicePort(() => api().user),
    providers: () =>
      native
        ? native.providers()
        : availableProviders({ google: __GOOGLE_CLIENT_ID__, apple: __APPLE_CLIENT_ID__ }, chrome.identity),
    handOff,
    eraseLinguaData,
    onSignedIn: () => scheduler.schedule(0),
  };

  // One collection at a time: the wake below and a surface opening can ask together, and the
  // second must wait for the first's sign-in rather than read "nothing pending" and move on.
  let collecting: Promise<AccountReply> | null = null;
  const collectHandedToken = (): Promise<AccountReply> =>
    (collecting ??= handleAccountMessage({ type: "account:collectHandedToken" }, accountDeps).finally(() => {
      collecting = null;
    }));

  // Restore a session on wake and sync if one was resumed (unless one ran within the minute:
  // an event page restarts often). On Safari, also collect an id_token the host app handed
  // back while the event page was asleep.
  void session.resume().then(async (ok) => {
    if (ok) await scheduler.onOpen(SURFACE_INTERVAL_MS);
    if (handOff) await collectHandedToken();
  });
  // A mutation in any context persists the backup → debounced sync. The store's owner
  // reports it, whether the store is IndexedDB or the settings-area fallback.
  onReaderDataChanged = () => scheduler.schedule(2000);

  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    // Every account:* message (sign-in, sign-up, verification, reset, providers) goes
    // through the unit-tested host; replies carry categories, never error strings.
    if (isAccountMessage(message)) {
      const reply =
        message.type === "account:collectHandedToken"
          ? collectHandedToken()
          : handleAccountMessage(message, accountDeps);
      void reply.then(sendResponse);
      return true;
    }
    // A surface opened or a page loaded (throttled), or « Synchroniser maintenant » (forced).
    // Both answer only once the exchange is over: the pending response is what keeps Safari's
    // event page alive until then.
    if (isSyncMessage(message)) {
      if (!message.force) {
        const interval = message.reason === "page" ? PAGE_INTERVAL_MS : SURFACE_INTERVAL_MS;
        void scheduler.onOpen(interval).then(() => sendResponse({ ok: true } satisfies SyncReply));
        return true;
      }
      void scheduler
        .syncNow()
        .then((error) => sendResponse((error ? { ok: false, error } : { ok: true }) satisfies SyncReply));
      return true;
    }
    const msg = message as { type?: string } | null;
    switch (msg?.type) {
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

chrome.commands.onCommand.addListener((command) => {
  // Firefox and Safari have no native lateral panel — Alt+Shift+S opens the in-page drawer
  // (stay in the page), like everything else there. Chromium's lingua-side-panel opens the
  // Side Panel below (its sidePanel.open needs a tabId and tolerates the async tabs.query hop).
  if (command === "lingua-side-panel" && __REVIEW_IN_PAGE__) {
    chrome.tabs.query({ active: true, currentWindow: true }, ([tab]) => {
      if (tab?.id != null) void chrome.tabs.sendMessage(tab.id, { type: "openDrawer", view: "review" }).catch(() => {});
    });
    return;
  }
  chrome.tabs.query({ active: true, currentWindow: true }, ([tab]) => {
    const tabId = tab?.id;
    if (tabId == null) return;
    if (command === "lingua-capture-selection") {
      void chrome.tabs.sendMessage(tabId, { type: "captureSelection" }).catch(() => {});
    } else if (command === "lingua-toggle-drawer") {
      void chrome.tabs.sendMessage(tabId, { type: "toggleDrawer" }).catch(() => {});
    } else if (command === "lingua-side-panel" && !__REVIEW_IN_PAGE__) {
      // Chromium only — Firefox was handled synchronously above. The !__REVIEW_IN_PAGE__
      // check is redundant with that early return, but it lets esbuild drop this call from
      // the Firefox/Safari bundle so the AMO linter stops flagging the unimplemented API.
      void chrome.sidePanel.open({ tabId }).catch(() => {});
    }
  });
});

/** Register the reader for every page when <all_urls> is granted; unregister otherwise.
 *  No-op on Firefox and Safari: they ship a STATIC content script on every page (build.mjs),
 *  because dynamic registration is unreliable on GeckoView and in Safari — so there is nothing
 *  to register here, and doing so would double-inject. Chromium (activeTab-first) uses the
 *  dynamic path. */
async function syncReaderRegistration(): Promise<void> {
  if (__STATIC_READER__) return;
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

chrome.runtime.onInstalled.addListener((details) => {
  void syncReaderRegistration();
  // Best-effort first-run welcome (Chromium/Firefox). The popup's level CTA is the
  // portable equivalent, so failures here are swallowed (e.g. Safari, where opening a
  // tab from install is unreliable).
  if (details.reason === "install") {
    try {
      void chrome.tabs.create({ url: chrome.runtime.getURL("onboarding.html") });
    } catch {
      /* onboarding tab is optional; ignore where unsupported */
    }
  }
});
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
