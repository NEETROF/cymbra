import { studiedOf } from "../analyzer/pairs.ts";
import type { TranslatorSource } from "../translate/create-port.ts";
import type { DownloadEvent } from "../translate/host/downloads.ts";
import type { EngineAccess } from "../translate/host/engine.ts";
import type { ModelDb } from "../translate/host/model-db.ts";
import { type ModelCatalogue, modelsFor, routeOf, totalSize } from "../translate/host/model-manifest.ts";
import { answerTranslation, answerWarm } from "../translate/host/relay.ts";
import type { ModelFailure } from "../translate/setting.ts";
import type { TranslatorPort } from "../translate/port.ts";

// « Traduction étendue » in the Cymbra Lingua app (add-lingua-review-translation-fallback D7): the
// extension's engine, in a classic worker the page owns, on the app's own copy of the models — kept in
// the IndexedDB database `lingua-model` of the page's origin, downloaded by the extension's own code
// from models.cymbra.app and checked against the catalogue the build carries. The app follows the
// extension's choice on the device: the relayed setting. On, the review offers once, quietly, to
// download what a pair of the reader's deck lacks; nothing is fetched before the reader accepts. Off,
// the copy is deleted at the next opening. WebKit refuses `persist()` to the app's origin (spike 1.4),
// so the copy is evictable: a pair downloaded before and found missing is offered again.
//
// No worker is constructed here: the page's engine and download hosts are handed in (app/workers.ts),
// so this runs under test with doubles, and the placement lint keeps the engine off the page's thread.

/** Where the app remembers what it offered: the pairs the reader declined, and those it downloaded. */
export const APP_MODELS_KEY = "cymbra-lingua-app-models";

interface Remembered {
  declined: string[];
  downloaded: string[];
}

/** What the review shows of the models, beside the cards. */
export type AppModelView =
  | { phase: "none" }
  /** « Traduire aussi dans l'app ? Téléchargement de 26 Mo » — `bytes` to fetch for `pairs`. */
  | { phase: "offer"; pairs: string[]; bytes: number }
  | { phase: "downloading"; received: number; total: number }
  | { phase: "failed"; reason: ModelFailure };

export interface AppTranslationDeps {
  /** « Traduction étendue » is on in the extension of this device (the relayed setting). */
  enabled: boolean;
  /** The reader's pairs: each studied language with their native language. */
  pairs: () => Promise<string[]>;
  /** The reader's native language: the half of a pair a request does not name. */
  native: () => Promise<string>;
  /** The catalogue the build carries (model-manifest.json, served from the embedded extension). */
  catalogue: () => Promise<ModelCatalogue>;
  db: Pick<ModelDb, "complete" | "erase">;
  /** The engine's channel to its worker. */
  engine: Pick<EngineAccess, "translate" | "warm">;
  /** Start a download of `models` in the download worker; its events reported to `report`. */
  download: (models: string[], report: (event: DownloadEvent) => void) => void;
  /** The page's own storage, where the offer is remembered; null where it cannot be used. */
  local: Pick<Storage, "getItem" | "setItem"> | null;
  /** The view changed: the page repaints the offer. */
  onChange?: (view: AppModelView) => void;
}

export interface AppTranslation {
  /** The translator for a sentence in `language`: only once its pair's models are all on the device. */
  source: TranslatorSource;
  view(): AppModelView;
  /** « Télécharger »: fetch what the offered pairs lack. */
  accept(): void;
  /** « Plus tard »: not offered again, unless downloaded once and evicted since. */
  decline(): void;
}

function remembered(local: AppTranslationDeps["local"]): Remembered {
  try {
    const raw = JSON.parse(local?.getItem(APP_MODELS_KEY) ?? "{}") as Partial<Record<keyof Remembered, unknown>>;
    const list = (v: unknown): string[] =>
      Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : [];
    return { declined: list(raw.declined), downloaded: list(raw.downloaded) };
  } catch {
    return { declined: [], downloaded: [] };
  }
}

function remember(local: AppTranslationDeps["local"], value: Remembered): void {
  try {
    local?.setItem(APP_MODELS_KEY, JSON.stringify(value));
  } catch {
    /* kept for this launch only */
  }
}

const without = (list: string[], removed: string[]): string[] => list.filter((x) => !removed.includes(x));
const union = (list: string[], added: string[]): string[] => [...new Set([...list, ...added])];

/** Open the app's translation: the copy deleted when the setting is off, else what is ready read. */
export async function openAppTranslation(deps: AppTranslationDeps): Promise<AppTranslation> {
  let view: AppModelView = { phase: "none" };
  const ready = new Set<string>();
  const set = (next: AppModelView): void => {
    view = next;
    deps.onChange?.(view);
  };

  if (!deps.enabled) {
    // Off in the extension: the app keeps no copy (D7). Nothing remembered stays either.
    await deps.db.erase().catch(() => undefined);
    remember(deps.local, { declined: [], downloaded: [] });
    return { source: () => null, view: () => view, accept: () => {}, decline: () => {} };
  }

  let catalogue: ModelCatalogue | null = null;
  let offered: string[] = [];
  /** A download is running: its progress is the view, whatever a reading finds meanwhile. */
  let downloading = false;

  /** Read what is on the device: the ready pairs, and what to offer for the others. */
  async function evaluate(): Promise<void> {
    try {
      catalogue ??= await deps.catalogue();
      const pairs = (await deps.pairs()).filter((pair) => routeOf(catalogue!, pair).length > 0);
      ready.clear();
      for (const pair of pairs) {
        const complete = await Promise.all(routeOf(catalogue, pair).map((m) => deps.db.complete(m)));
        if (complete.every(Boolean)) ready.add(pair);
      }
      const memory = remembered(deps.local);
      offered = pairs.filter(
        (pair) => !ready.has(pair) && (!memory.declined.includes(pair) || memory.downloaded.includes(pair)),
      );
      if (downloading) return;
      if (offered.length === 0) {
        set({ phase: "none" });
        return;
      }
      const lacking: number[] = [];
      for (const model of modelsFor(catalogue, offered)) {
        if (!(await deps.db.complete(model))) lacking.push(totalSize(model));
      }
      set({ phase: "offer", pairs: [...offered], bytes: lacking.reduce((a, b) => a + b, 0) });
    } catch (e: unknown) {
      console.warn("[Cymbra Lingua] the app's models could not be read:", e);
      ready.clear();
      set({ phase: "none" });
    }
  }

  await evaluate();

  const port: TranslatorPort = {
    translate: (request) =>
      answerTranslation(
        {
          engine: deps.engine,
          native: deps.native,
          ready: async (pair) => ready.has(pair),
          // A route said to be ready that gave no answer: the copy may have been evicted.
          onNotReady: () => void evaluate(),
        },
        request,
      ),
    warm: (language) =>
      void answerWarm(
        { engine: deps.engine, native: deps.native, ready: async (pair) => ready.has(pair), onNotReady: () => {} },
        language,
      ),
  };

  return {
    source: (language) => ([...ready].some((pair) => studiedOf(pair) === language) ? port : null),
    view: () => view,
    accept() {
      if (view.phase !== "offer" && view.phase !== "failed") return;
      if (!catalogue || offered.length === 0) return;
      const pairs = [...offered];
      const models = modelsFor(catalogue, pairs).map((m) => m.version);
      downloading = true;
      set({ phase: "downloading", received: 0, total: view.phase === "offer" ? view.bytes : 0 });
      deps.download(models, (event) => {
        if (event.kind === "progress") {
          set({ phase: "downloading", received: event.received, total: event.total });
          return;
        }
        downloading = false;
        if (event.kind === "failed") {
          // A checksum that does not match, a network cut: nothing is recorded as downloaded.
          set({ phase: "failed", reason: event.reason });
          return;
        }
        const memory = remembered(deps.local);
        remember(deps.local, {
          declined: without(memory.declined, pairs),
          downloaded: union(memory.downloaded, pairs),
        });
        void evaluate();
      });
    },
    decline() {
      if (view.phase !== "offer" && view.phase !== "failed") return;
      const memory = remembered(deps.local);
      remember(deps.local, {
        declined: union(memory.declined, offered),
        downloaded: without(memory.downloaded, offered),
      });
      offered = [];
      set({ phase: "none" });
    },
  };
}
