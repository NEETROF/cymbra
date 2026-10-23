import { type GlueLoader, WasmAnalyzerPort, type WasmModule } from "../analyzer/engine.ts";
import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import { fillPage, setDocumentLanguage } from "../i18n/index.ts";
import { type Clients, createClients, createTransport, setTokenRefresher } from "../net/transport.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import { applySurfaceLook } from "../reading/surface-look.ts";
import {
  COLOURS_KEY,
  colourPreferenceOf,
  nativeLanguageOfStored,
  ONLINE_TRANSLATION_KEY,
  READER_DISPLAY_KEY,
  readerDisplayOf,
  STORAGE_VERSION,
} from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";
import { modelDb } from "../translate/host/model-db.ts";
import { loadBundledCatalogue } from "../translate/host/model-manifest.ts";
import { parseHost, TRANSLATION_HOST_KEY } from "../translate/setting.ts";
import * as wasmGlue from "../wasm/pkg/lingua_wasm.js";
import { appAccount, appAccountLocale } from "./account.ts";
import { webkitSend } from "./bridge.ts";
import { type EmailPanel, emailPanel, reloadsOnForeground } from "./email-panel.ts";
import { type AppDeckSync, appDeckSync } from "./deck-sync.ts";
import { openAppSession } from "./session.ts";
import { type AppTranslation, appPairs, openAppTranslation } from "./translation.ts";
import {
  appCopy,
  renderAccountLine,
  renderModelOffer,
  savedAtLine,
  setSyncOfferAside,
  syncOfferSetAside,
} from "./view.ts";
import { appDownload, appEngine } from "./workers.ts";

// The Cymbra Lingua app's page (add-lingua-apple-review D1): the extension's own Révision and
// Statistiques over the state the extension shared through the App Group, the engine in the page.
// Excluded from coverage (DOM wiring; session.ts and view.ts are tested, the views are the
// extension's, measured in their own specs).

/** The glue bundled into this page: it is served from the app, never fetched from elsewhere. */
const staticGlue: GlueLoader = async () => wasmGlue as unknown as WasmModule;

const now = (): number => Math.floor(Date.now() / 1000);

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

let review: ReviewPage | null = null;
let emailSignIn: EmailPanel | null = null;
/** The app's synchronisation of its reviews (add-lingua-app-deck-sync), once the page is open. */
let deckSync: AppDeckSync | null = null;

async function main(): Promise<void> {
  const send = webkitSend();
  if (!send) return;
  // The engine takes a few seconds to load the deck: the loading copy shows at once, in the reader's
  // language and colours, read from the shared state before the engine starts.
  const opened = (globalThis as { linguaLanguage?: string }).linguaLanguage;
  const session = await openAppSession({
    onState: (state, language) => {
      applySurfaceLook(
        document.documentElement,
        colourPreferenceOf(state.prefs[COLOURS_KEY]),
        readerDisplayOf(state.prefs[READER_DISPLAY_KEY]),
      );
      setDocumentLanguage(document, language);
      fillPage(document, appCopy(language));
    },
    send,
    port: (backup) =>
      new WasmAnalyzerPort(staticGlue, SHIPPED_PAIRS, async () =>
        nativeLanguageOfStored({ v: STORAGE_VERSION, backup }),
      ),
    now: () => Date.now(),
    local: (() => {
      try {
        return globalThis.localStorage;
      } catch {
        return null;
      }
    })(),
    fallbackLanguage: opened,
    onSaved: () => deckSync?.saved(),
  }).catch((e: unknown) => {
    console.warn("[Cymbra Lingua] could not open the shared state:", e);
    return null;
  });
  if (!session) {
    await send({ type: "review.unavailable" });
    return;
  }
  const { port, store, prefs, interfaceLanguage: language } = session;
  const saved = savedAtLine(language, session.savedAt);
  if (saved) {
    $("saved").textContent = saved;
    $("saved").hidden = false;
  }
  const local = (() => {
    try {
      return globalThis.localStorage;
    } catch {
      return null;
    }
  })();
  const chosen = await prefs.get([TRANSLATION_HOST_KEY, ONLINE_TRANSLATION_KEY]);

  // « Traduction étendue » in the app (add-lingua-review-translation-fallback D7): its own copy of the
  // models, offered once; the engine in a worker of this page.
  const offer = $("model-offer");
  let translation: AppTranslation | null = null;
  let reoffered = false;
  const offerActions = { accept: () => translation?.accept(), decline: () => translation?.decline() };
  translation = await openAppTranslation({
    enabled: parseHost(chosen[TRANSLATION_HOST_KEY]) === "local",
    // The listed pairs of the reader's native language, never studied × native: an Italian pair is
    // routed before any ships, and its models must not be fetched (add-lingua-italian-baseline 7.1).
    pairs: () => appPairs(port),
    native: () => port.nativeLanguage(),
    catalogue: () => loadBundledCatalogue(),
    db: modelDb(),
    engine: appEngine(),
    download: appDownload,
    local,
    onChange: (view) => {
      renderModelOffer(offer, view, language, offerActions);
      // Re-offered from a card's note: the offer, below the review, brought into view.
      if (reoffered && view.phase === "offer") offer.scrollIntoView?.({ block: "nearest" });
      reoffered = false;
    },
  });
  renderModelOffer(offer, translation.view(), language, offerActions);

  // « Traduction en ligne » in the app (D8): its own session, opened on the first press — or from the
  // line offered on opening, which also synchronises the reviews (add-lingua-app-deck-sync D11).
  let clients: Clients | null = null;
  const line = $("account-line");
  // An email account signs in in the page (add-lingua-app-email-signin): the extension's own flow.
  emailSignIn = emailPanel({
    dialog: $("email-panel") as HTMLDialogElement,
    root: $("email-root"),
    cancel: $("email-cancel") as HTMLButtonElement,
    language,
    locale: appAccountLocale(session.sharedLanguage, navigator.language),
  });
  let wasSignedIn = false;
  const account = appAccount({
    send,
    emailSignIn: emailSignIn.signIn,
    clients: () => (clients ??= createClients(createTransport(() => account.token()))),
    native: () => port.nativeLanguage(),
    chosen: chosen[ONLINE_TRANSLATION_KEY] === true,
    relayedAccess: session.onlineAccess,
    local,
    onChange: () => {
      drawAccount();
      void review?.refresh();
      // Signed in or out: the synchronisation reads the account again (D7).
      if (account.signedIn() !== wasSignedIn) {
        wasSignedIn = account.signedIn();
        void deckSync?.accountChanged();
      }
    },
  });
  setTokenRefresher(() => account.refresh());
  const drawAccount = (): void =>
    renderAccountLine(
      line,
      {
        relevant: account.online.offered() || account.signedOut(),
        signedIn: account.signedIn(),
        signedOut: account.signedOut(),
        name: account.name(),
        sync: deckSync?.line(),
        offerSync:
          !account.signedIn() && typeof session.sharedAccount === "string" && !syncOfferSetAside(local, Date.now()),
      },
      language,
      {
        signIn: () => void account.signIn(),
        signOut: () => void account.signOut(),
        later: () => {
          setSyncOfferAside(local, Date.now());
          drawAccount();
        },
      },
    );
  await account.resume();
  wasSignedIn = account.signedIn();
  // The reviews synchronise with the server on the extension's account (add-lingua-app-deck-sync):
  // the opening's exchange is waited at most a few seconds, the loading line showing meanwhile.
  deckSync = appDeckSync({
    port,
    store,
    pendingOps: session.pendingOps,
    sharedAccount: session.sharedAccount,
    sharedErasedAt: session.sharedErasedAt,
    discardPending: session.discardPending,
    signedIn: () => account.signedIn(),
    // The id the line's name already read (add-lingua-app-email-signin D9), not a second request.
    userId: () => account.userId(),
    clients: () => (clients ??= createClients(createTransport(() => account.token()))),
    local,
    reviewing: () => review?.reviewing() ?? false,
    onApplied: () => review?.refresh(),
    onChange: () => drawAccount(),
    now: () => Date.now(),
  });
  drawAccount();
  await deckSync.open();

  review = mountReview($("review-root"), port, store, {
    now,
    prefs,
    interfaceLanguage: language,
    translator: translation.source,
    // A card nothing can translate says how (add-lingua-review-translation-fallback D12); its
    // download link re-offers the offer above.
    translationHint: (studied) => {
      const hint = translation?.hint(studied) ?? null;
      if (hint?.kind !== "download") return hint;
      return {
        kind: "download",
        download: () => {
          reoffered = true;
          hint.download();
        },
      };
    },
    online: account.online,
    // « Séance terminée »: the app's own count, for StoreKit's request — no prompt in the page (D11).
    onSessionFinished: session.sessionFinished,
    // A session started within StoreKit's two seconds: the app does not ask then (D11).
    onSessionStarted: session.sessionStarted,
  });

  async function show(view: "review" | "stats"): Promise<void> {
    $("view-review").hidden = view !== "review";
    $("view-stats").hidden = view !== "stats";
    for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
      b.classList.toggle("active", b.dataset.view === view);
    }
    if (view === "review") {
      await review?.refresh();
    } else {
      await mountStats($("view-stats"), port, store, undefined, language);
    }
  }
  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.addEventListener("click", () => void show(b.dataset.view === "stats" ? "stats" : "review"));
  }
  $("loading").hidden = true;
  $("views").hidden = false;
  await show("review");
}

/**
 * The app came back to the foreground (ViewController.swift): the extension may have saved since.
 * Read the shared state again, unless a session is under way — its queue stays as it is — or the email
 * sign-in is open, whose code step the reader came back from Mail for (add-lingua-app-email-signin D6).
 */
(globalThis as { linguaForeground?: () => void }).linguaForeground = () => {
  if (!reloadsOnForeground({ reviewing: review?.reviewing() ?? false, emailPanelOpen: emailSignIn?.isOpen() ?? false }))
    return;
  location.reload();
};

/**
 * The app goes to the background (ViewController.swift, add-lingua-app-deck-sync D5): push what waits
 * at once; the app awaits it, within the time the system gives.
 */
(globalThis as { linguaBackground?: () => Promise<void> }).linguaBackground = () =>
  deckSync?.background() ?? Promise.resolve();

void main();
