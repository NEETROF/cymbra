import { type GlueLoader, WasmAnalyzerPort, type WasmModule } from "../analyzer/engine.ts";
import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import { fillPage, setDocumentLanguage } from "../i18n/index.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import { applySurfaceLook } from "../reading/surface-look.ts";
import {
  COLOURS_KEY,
  colourPreferenceOf,
  nativeLanguageOfStored,
  READER_DISPLAY_KEY,
  readerDisplayOf,
  STORAGE_VERSION,
} from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";
import * as wasmGlue from "../wasm/pkg/lingua_wasm.js";
import { webkitSend } from "./bridge.ts";
import { openAppSession } from "./session.ts";
import { appCopy, renderLanguageDue, savedAtLine } from "./view.ts";

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

async function main(): Promise<void> {
  const send = webkitSend();
  if (!send) return;
  const session = await openAppSession({
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
    fallbackLanguage: (globalThis as { linguaLanguage?: string }).linguaLanguage,
  }).catch((e: unknown) => {
    console.warn("[Cymbra Lingua] could not open the shared state:", e);
    return null;
  });
  if (!session) {
    await send({ type: "review.unavailable" });
    return;
  }
  const { port, store, prefs, interfaceLanguage: language } = session;

  setDocumentLanguage(document, language);
  fillPage(document, appCopy(language));
  const looks = await prefs.get([COLOURS_KEY, READER_DISPLAY_KEY]);
  applySurfaceLook(
    document.documentElement,
    colourPreferenceOf(looks[COLOURS_KEY]),
    readerDisplayOf(looks[READER_DISPLAY_KEY]),
  );
  const saved = savedAtLine(language, session.savedAt);
  if (saved) {
    $("saved").textContent = saved;
    $("saved").hidden = false;
  }

  const languages = $("languages");
  const refreshHome = (): Promise<void> => renderLanguageDue(languages, port, now(), language);
  review = mountReview($("review-root"), port, store, { now, prefs, interfaceLanguage: language });
  // Every save of the backup is an answer: the home's counts follow it.
  session.onSaved(refreshHome);

  async function show(view: "review" | "stats"): Promise<void> {
    $("view-review").hidden = view !== "review";
    $("view-stats").hidden = view !== "stats";
    for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
      b.classList.toggle("active", b.dataset.view === view);
    }
    if (view === "review") {
      await refreshHome();
      await review?.refresh();
    } else {
      await mountStats($("view-stats"), port, store, undefined, language);
    }
  }
  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.addEventListener("click", () => void show(b.dataset.view === "stats" ? "stats" : "review"));
  }
  await show("review");
}

/**
 * The app came back to the foreground (ViewController.swift): the extension may have saved since.
 * Read the shared state again, unless a session is under way — its queue stays as it is.
 */
(globalThis as { linguaForeground?: () => void }).linguaForeground = () => {
  if (review?.reviewing()) return;
  location.reload();
};

void main();
