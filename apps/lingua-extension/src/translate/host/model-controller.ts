// « Traduction étendue », as the background runs it (add-lingua-translation-delivery D2–D5).
// The only writer of the setting and of where its model stands: surfaces ask (model-messages.ts)
// and follow the two storage keys (setting.ts).
//
// - Turning it on writes the choice, then starts the download in the engine's host. Nothing loads
//   the engine: the first translation asked does (D6).
// - Turning it off — cancelling included — writes the choice first, so a late report from the host
//   lands on a setting that is off and is ignored; then it stops the engine and the download and
//   deletes the model. A cancelled download keeps nothing.
// - Whatever went wrong is recorded for the setting to explain, and nothing restarts on its own:
//   a download its host dropped is "interrupted", a model the browser removed is "removed", and
//   both wait for the reader to ask (A model the browser removed is not fetched again unasked).
// - It is offered wherever it runs: the controller exists only in a variant that carries the engine,
//   Firefox for Android included (add-lingua-translation-android D1).
//
// Commands and reports run one at a time, in the order they came: "off" right after "on" must not
// be overtaken by the download's first report.

import type { ModelCommand, ModelStatus } from "../model-messages.ts";
import {
  ABSENT,
  loadTranslationSetting,
  type ModelState,
  saveTranslationSetting,
  type SettingArea,
  type TranslationSetting,
} from "../setting.ts";
import type { DownloadEvent } from "./downloads.ts";
import type { ModelDb } from "./model-db.ts";
import { type ModelManifest, totalSize } from "./model-manifest.ts";

/** The engine's host, as the controller drives it: the offscreen document, or the event page. */
export interface ModelHostAccess {
  /** Start downloading what the device lacks; progress and the outcome come back through `onEvent`. */
  startDownload(): Promise<void>;
  cancelDownload(): Promise<void>;
  downloading(): Promise<boolean>;
  /** Stop the engine and give its memory back (on Chromium, close the offscreen document). */
  shutDown(): Promise<void>;
}

export interface ModelControllerDeps {
  /** chrome.storage.local: on this device, never synced. */
  area: SettingArea;
  host: ModelHostAccess;
  db: Pick<ModelDb, "complete" | "erase">;
  manifest: () => Promise<ModelManifest>;
  log?: (message: string, detail?: unknown) => void;
}

const LOG = (message: string, detail?: unknown): void => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

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

  /** Where things stand, reconciled with what is really running and really stored. */
  status(): Promise<ModelStatus> {
    return this.serial(() => this.reconcile());
  }

  /** Whether a translation can be asked right now, from what is recorded — the database is not read. */
  async ready(): Promise<boolean> {
    const { host, state } = await loadTranslationSetting(this.deps.area);
    return host === "local" && state.phase === "ready";
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
      await this.quietly("delete the model", () => this.deps.db.erase());
      return this.current();
    });
  }

  /** Try again, resume, or download again: fetch whatever the device lacks. */
  resume(): Promise<ModelStatus> {
    return this.serial(async () => {
      const { host, state } = await loadTranslationSetting(this.deps.area);
      if (host !== "local") return this.reconcile();
      if (state.phase === "ready" || (state.phase === "downloading" && (await this.deps.host.downloading()))) {
        return this.reconcile();
      }
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
            ? { phase: "ready" }
            : { phase: "failed", reason: event.reason };
      await saveTranslationSetting(this.deps.area, { state: next });
    });
  }

  private async download(): Promise<void> {
    const total = await this.deps
      .manifest()
      .then(totalSize)
      .catch(() => 0);
    await saveTranslationSetting(this.deps.area, {
      host: "local",
      state: { phase: "downloading", received: 0, total },
    });
    try {
      await this.deps.host.startDownload();
    } catch (e) {
      (this.deps.log ?? LOG)("the model download could not start:", e);
      await saveTranslationSetting(this.deps.area, { state: { phase: "failed", reason: "unknown" } });
    }
  }

  private async reconcile(): Promise<ModelStatus> {
    const { host, state } = await loadTranslationSetting(this.deps.area);
    if (host === "none") {
      if (state.phase !== "absent") await saveTranslationSetting(this.deps.area, { state: ABSENT });
      return { offered: true, host, state: ABSENT };
    }
    const next = await this.observed(state);
    if (next !== state) await saveTranslationSetting(this.deps.area, { state: next });
    return { offered: true, host, state: next };
  }

  /** What the recorded state really is: a download nobody runs, a model nobody stores. */
  private async observed(state: ModelState): Promise<ModelState> {
    switch (state.phase) {
      case "downloading":
        return (await this.deps.host.downloading())
          ? state
          : { phase: "interrupted", received: state.received, total: state.total };
      case "ready":
        return (await this.stored()) ? state : { phase: "removed" };
      case "absent":
        // On, with nothing recorded: the state was lost with the browser's storage. The model is
        // the reader's to download again, never ours.
        return { phase: "removed" };
      default:
        return state;
    }
  }

  private async stored(): Promise<boolean> {
    try {
      return await this.deps.db.complete(await this.deps.manifest());
    } catch (e) {
      (this.deps.log ?? LOG)("could not read the stored model:", e);
      return false;
    }
  }

  private async current(): Promise<ModelStatus> {
    const setting: TranslationSetting = await loadTranslationSetting(this.deps.area);
    return { offered: true, ...setting };
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
