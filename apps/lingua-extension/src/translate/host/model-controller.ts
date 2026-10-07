// « Traduction étendue », as the background runs it (add-lingua-translation-delivery D2–D5).
// The only writer of the setting and of where its models stand: surfaces ask (model-messages.ts)
// and follow the two storage keys (setting.ts).
//
// - The models are those the reader's pairs need — the shipped pairs of their native language that
//   study each of their accepted languages — the union of their routes in the package's catalogue
//   (generalise-lingua-translation-model-state D2, generalise-lingua-translation-routes-by-pair D5).
//   A shared model is one model.
// - Turning it on writes the choice, then starts downloading every needed model in the engine's
//   host. Nothing loads the engine: the first translation asked does (D6).
// - Turning it off — cancelling included — writes the choice first, so a late report from the host
//   lands on a setting that is off and is ignored; then it stops the engine and the download and
//   deletes every model. A cancelled download keeps nothing.
// - Whatever went wrong is recorded for the setting to explain, and nothing restarts on its own:
//   a download its host dropped is "interrupted", a model the browser removed is "removed", a model a
//   language the reader added needs is "missing", and all of them wait for the reader to ask (A model
//   the browser removed is not fetched again unasked).
// - Reconciling follows the reader's pairs: a model no pair needs any more is deleted (model-state
//   D3). The native language changing changes the pairs, and the models with them (routes-by-pair D5).
// - It is offered wherever it runs and one of the reader's pairs has a route: the controller
//   exists only in a variant that carries the engine, Firefox for Android included
//   (add-lingua-translation-android D1).
//
// Commands and reports run one at a time, in the order they came: "off" right after "on" must not
// be overtaken by the download's first report.

import type { ModelCommand, ModelCost, ModelStatus } from "../model-messages.ts";
import {
  ABSENT,
  loadTranslationSetting,
  MODEL_STATE_KEY,
  type ModelState,
  pairReady,
  parseHost,
  parseModelState,
  saveTranslationSetting,
  type SettingArea,
  TRANSLATION_HOST_KEY,
  type TranslationHost,
} from "../setting.ts";
import type { DownloadEvent } from "./downloads.ts";
import type { ModelDb } from "./model-db.ts";
import {
  type ModelCatalogue,
  type ModelManifest,
  modelsFor,
  routeOf,
  totalSize,
  unpackedSize,
} from "./model-manifest.ts";

/** The engine's host, as the controller drives it: the offscreen document, or the event page. */
export interface ModelHostAccess {
  /**
   * Start downloading what the device lacks of `models` (catalogue ids); progress and the outcome
   * come back through `onEvent`.
   */
  startDownload(models: string[]): Promise<void>;
  cancelDownload(): Promise<void>;
  downloading(): Promise<boolean>;
  /** Stop the engine and give its memory back (on Chromium, close the offscreen document). */
  shutDown(): Promise<void>;
}

export interface ModelControllerDeps {
  /** chrome.storage.local: on this device, never synced. */
  area: SettingArea;
  host: ModelHostAccess;
  db: Pick<ModelDb, "complete" | "erase" | "prune">;
  /** The package's catalogue of models. */
  catalogue: () => Promise<ModelCatalogue>;
  /**
   * The reader's pairs, from their profile: the shipped pairs of their native language that study
   * each of their accepted languages (routes-by-pair D5).
   */
  pairs: () => Promise<string[]>;
  log?: (message: string, detail?: unknown) => void;
}

/** What the reader's pairs need: the catalogue, the pairs, and the union of their routes. */
interface Needs {
  catalogue: ModelCatalogue;
  pairs: string[];
  needed: ModelManifest[];
}

/** Whether a route the reader's pairs need goes through another language (add-lingua-spanish-translation-pivot D4). */
function pivots({ catalogue, pairs }: Needs): boolean {
  return pairs.some((pair) => routeOf(catalogue, pair).length > 1);
}

/**
 * A stored `ready` or `missing` from before pairs were recorded (routes-by-pair D3): it names
 * languages, or nothing. It reads as the same pairs, and is rewritten with them once.
 */
function namesNoPairs(raw: unknown): boolean {
  const s = raw as { phase?: unknown; pairs?: unknown } | null | undefined;
  return (s?.phase === "ready" || s?.phase === "missing") && !Array.isArray(s.pairs);
}

const LOG = (message: string, detail?: unknown): void => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

const sum = (models: ModelManifest[], size: (m: ModelManifest) => number): number =>
  models.reduce((total, model) => total + size(model), 0);

export class ModelController {
  private queue: Promise<unknown> = Promise.resolve();

  constructor(private readonly deps: ModelControllerDeps) {}

  /** One command from a surface. */
  handle(op: ModelCommand): Promise<ModelStatus> {
    switch (op) {
      case "enable":
        return this.enable();
      case "disable":
        return this.disable();
      case "resume":
        return this.resume();
      default:
        return this.status();
    }
  }

  /** Where things stand, reconciled with what is really running, really stored, and really needed. */
  status(): Promise<ModelStatus> {
    return this.serial(() => this.reconcile());
  }

  /**
   * What a background that has just started fixes before anything else: a download that died with
   * the previous page is "interrupted". It reads neither the catalogue nor the reader's profile,
   * which a woken service worker should not pay for; `status` does the rest.
   */
  recover(): Promise<void> {
    return this.serial(async () => {
      const { host, state } = await loadTranslationSetting(this.deps.area);
      if (host !== "local" || state.phase !== "downloading" || (await this.deps.host.downloading())) return;
      await saveTranslationSetting(this.deps.area, {
        state: { phase: "interrupted", received: state.received, total: state.total },
      });
    });
  }

  /**
   * Whether a translation through `pair` can be asked right now, from what is recorded — the database
   * is not read (model-state D5, routes-by-pair D3).
   */
  async ready(pair: string): Promise<boolean> {
    const { host, state } = await loadTranslationSetting(this.deps.area);
    return pairReady(host, state, pair);
  }

  enable(): Promise<ModelStatus> {
    return this.serial(async () => {
      const current = await loadTranslationSetting(this.deps.area);
      if (current.host === "local") return this.reconcile();
      await this.download();
      return this.current();
    });
  }

  disable(): Promise<ModelStatus> {
    return this.serial(async () => {
      await saveTranslationSetting(this.deps.area, { host: "none", state: ABSENT });
      await this.quietly("stop the download", () => this.deps.host.cancelDownload());
      await this.quietly("stop the engine", () => this.deps.host.shutDown());
      await this.quietly("delete the models", () => this.deps.db.erase());
      return this.current();
    });
  }

  /** Try again, resume, download again or download what is missing: fetch whatever the device lacks. */
  resume(): Promise<ModelStatus> {
    return this.serial(async () => {
      const { host, state } = await loadTranslationSetting(this.deps.area);
      if (host !== "local") return this.reconcile();
      if (state.phase === "downloading" && (await this.deps.host.downloading())) return this.reconcile();
      const reconciled = await this.reconcile();
      if (reconciled.state.phase === "ready") return reconciled;
      await this.download();
      return this.current();
    });
  }

  /** A report from the host. Only a download the setting still expects may move its state. */
  onEvent(event: DownloadEvent): Promise<void> {
    return this.serial(async () => {
      const { host, state } = await loadTranslationSetting(this.deps.area);
      if (host !== "local" || state.phase !== "downloading") return;
      const next: ModelState =
        event.kind === "progress"
          ? { phase: "downloading", received: event.received, total: event.total }
          : event.kind === "done"
            ? await this.settled(state, await this.needs())
            : { phase: "failed", reason: event.reason };
      await saveTranslationSetting(this.deps.area, { state: next });
    });
  }

  private async download(): Promise<void> {
    const needs = await this.needs();
    const models = needs?.needed ?? [];
    await saveTranslationSetting(this.deps.area, {
      host: "local",
      state: { phase: "downloading", received: 0, total: sum(models, totalSize) },
    });
    try {
      await this.deps.host.startDownload(models.map((model) => model.version));
    } catch (e) {
      (this.deps.log ?? LOG)("the model download could not start:", e);
      await saveTranslationSetting(this.deps.area, { state: { phase: "failed", reason: "unknown" } });
    }
  }

  private async reconcile(): Promise<ModelStatus> {
    const got = await this.deps.area.get([TRANSLATION_HOST_KEY, MODEL_STATE_KEY]);
    const host = parseHost(got[TRANSLATION_HOST_KEY]);
    const state = parseModelState(got[MODEL_STATE_KEY]);
    const needs = await this.needs();
    if (host === "none") {
      if (state.phase !== "absent") await saveTranslationSetting(this.deps.area, { state: ABSENT });
      return this.answer(host, ABSENT, needs);
    }
    const next = await this.observed(state, needs);
    // Written when it differs from what was read — the states compared, never their stored text: the
    // storage hands an object's keys back in its own order — and once for a state stored without
    // pairs, which reads as the same pairs and is rewritten with them (routes-by-pair D3).
    if (JSON.stringify(next) !== JSON.stringify(state) || namesNoPairs(got[MODEL_STATE_KEY])) {
      await saveTranslationSetting(this.deps.area, { state: next });
    }
    return this.answer(host, next, needs);
  }

  /** What the recorded state really is: a download nobody runs, a model nobody stores or needs. */
  private async observed(state: ModelState, needs: Needs | null): Promise<ModelState> {
    switch (state.phase) {
      case "downloading":
        return (await this.deps.host.downloading())
          ? state
          : { phase: "interrupted", received: state.received, total: state.total };
      case "ready":
      case "missing":
        return this.settled(state, needs);
      case "absent":
        // On, with nothing recorded: the state was lost with the browser's storage. The models are
        // the reader's to download again, never ours.
        return { phase: "removed" };
      default:
        return state;
    }
  }

  /**
   * Where the needed models stand once nothing is downloading: every one complete is `ready`; one
   * the state had recorded complete and is gone is `removed`; one never downloaded is `missing`.
   * Models no pair needs any more are deleted (model-state D3). The state is written with pairs: one
   * read from a release that named languages is rewritten here (routes-by-pair D3).
   */
  private async settled(recorded: ModelState, needs: Needs | null): Promise<ModelState> {
    if (!needs) return recorded; // the catalogue cannot be read: change nothing, delete nothing
    const { catalogue, pairs, needed } = needs;
    const complete: string[] = [];
    for (const model of needed) if (await this.stored(model)) complete.push(model.version);
    await this.quietly("delete the models no pair needs", () => this.deps.db.prune(needed));
    const translatable = pairs.filter((pair) => {
      const route = routeOf(catalogue, pair);
      return route.length > 0 && route.every((model) => complete.includes(model.version));
    });
    const missing = needed.filter((model) => !complete.includes(model.version));
    if (missing.length === 0) return { phase: "ready", models: complete, pairs: translatable };
    // A `ready` written before the models were named stood for every model it needed then.
    const had =
      recorded.phase === "ready" && recorded.models.length === 0
        ? needed.map((model) => model.version)
        : recorded.phase === "ready" || recorded.phase === "missing"
          ? recorded.models
          : [];
    if (missing.some((model) => had.includes(model.version))) return { phase: "removed" };
    return { phase: "missing", models: complete, pairs: translatable, total: sum(missing, totalSize) };
  }

  private async stored(model: ModelManifest): Promise<boolean> {
    try {
      return await this.deps.db.complete(model);
    } catch (e) {
      (this.deps.log ?? LOG)("could not read the stored model:", e);
      return false;
    }
  }

  /** The catalogue, the reader's pairs and the models they need; null when either cannot be read. */
  private async needs(): Promise<Needs | null> {
    try {
      const [catalogue, pairs] = await Promise.all([this.deps.catalogue(), this.deps.pairs()]);
      return { catalogue, pairs, needed: modelsFor(catalogue, pairs) };
    } catch (e) {
      (this.deps.log ?? LOG)("could not read what the reader's pairs need:", e);
      return null;
    }
  }

  private async current(): Promise<ModelStatus> {
    const { host, state } = await loadTranslationSetting(this.deps.area);
    return this.answer(host, state, await this.needs());
  }

  /**
   * The whole status: the setting, what the needed models cost (catalogue D4), and whether it is
   * offered at all — not while none of the reader's pairs has a route (model-state D3).
   */
  private answer(host: TranslationHost, state: ModelState, needs: Needs | null): ModelStatus {
    const offered = needs ? needs.needed.length > 0 : true;
    const cost: ModelCost | undefined =
      needs && needs.needed.length > 0
        ? {
            download: sum(needs.needed, totalSize),
            stored: sum(needs.needed, unpackedSize),
            ...(pivots(needs) ? { pivot: true } : {}),
          }
        : undefined;
    return { offered, host, state, ...(cost ? { cost } : {}) };
  }

  private async quietly(what: string, action: () => Promise<void>): Promise<void> {
    try {
      await action();
    } catch (e) {
      (this.deps.log ?? LOG)(`could not ${what}:`, e);
    }
  }

  private serial<T>(task: () => Promise<T>): Promise<T> {
    const run = this.queue.then(task, task);
    this.queue = run.catch(() => undefined);
    return run;
  }
}
